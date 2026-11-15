use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::db::{BookRepository, Database, ProgressRepository};
use crate::domain::book::Book;
use crate::epub::{EpubParser, PaginatedBook, PaginationConfig, Paginator};
use crate::error::CoreResult;
use crate::mock::reading_content::{get_page_content, PageContent};
use crate::services::StatsService;

pub struct ReaderService;

impl ReaderService {
    /// Carga y pagina todos los capítulos de un archivo `.epub` real desde disco.
    pub fn load_book_pages(
        epub_path: impl AsRef<Path>,
        config: PaginationConfig,
    ) -> CoreResult<PaginatedBook> {
        let parsed = EpubParser::parse_file(epub_path)?;
        Ok(Paginator::paginate(&parsed.chapters, config))
    }

    /// Resuelve el contenido de una página (1-based). Si el libro posee capítulos reales
    /// cargados, devuelve su texto real paginado; de lo contrario recurre al generador mock.
    pub fn resolve_page(book: &Book, page_number: u32) -> PageContent {
        if !book.chapters.is_empty() {
            let paginated = PaginatedBook::from_chapters(book.chapters.clone());
            if let Some(resolved) = paginated.resolve_page(page_number) {
                return PageContent {
                    chapter_number: resolved.chapter_number,
                    chapter_title: resolved.chapter_title,
                    paragraphs: resolved.paragraphs,
                };
            }
        }

        get_page_content(&book.title, page_number, book.total_pages)
    }

    /// Persiste el progreso actual del libro, la última marca temporal y la actividad de la sesión
    /// en SQLite. Si el libro todavía no existía en la base local, lo registra automáticamente.
    pub fn record_reading_activity(
        db: &mut Database,
        book: &Book,
        current_page: u32,
        current_chapter: u32,
        pages_delta: u32,
        elapsed_seconds: u64,
    ) -> CoreResult<()> {
        if BookRepository::get_by_id(db.conn(), &book.id)?.is_none() {
            BookRepository::upsert(db.conn_mut(), book, None, None, "es")?;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        ProgressRepository::update_page(
            db.conn(),
            &book.id,
            current_page,
            current_chapter,
            book.total_pages,
            now,
        )?;

        let bounded_secs = elapsed_seconds.min(300);
        if pages_delta > 0 || bounded_secs > 0 {
            let today = StatsService::today_iso();
            ProgressRepository::record_session(db.conn_mut(), &book.id, &today, 0, pages_delta)?;
            ProgressRepository::add_reading_seconds(db.conn(), &book.id, bounded_secs)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::book::{CoverPalette, Genre};
    use crate::epub::ParsedChapter;

    #[test]
    fn test_xhtml_extraction_and_pagination_mapping() {
        let ch1 = ParsedChapter {
            index: 0,
            id: "c1".into(),
            title: "Capítulo I".into(),
            raw_xhtml: r#"
                <html>
                  <head><title>Ignored</title><style>body { color: red; }</style></head>
                  <body>
                    <script>alert('x');</script>
                    <h1>Capítulo I</h1>
                    <p>Primer párrafo con &ldquo;comillas&rdquo; y &mdash;guiones&mdash;.</p>
                    <p>Segundo párrafo de prueba.</p>
                  </body>
                </html>
            "#
            .into(),
        };

        let ch2 = ParsedChapter {
            index: 1,
            id: "c2".into(),
            title: "Capítulo II".into(),
            raw_xhtml: format!("<p>{}</p>", "Palabra larga de ejemplo ".repeat(40)),
        };

        let paginated = Paginator::paginate(&[ch1, ch2], PaginationConfig { chars_per_page: 250 });
        assert!(paginated.total_pages >= 3);
        assert_eq!(paginated.chapter_start_page(0), Some(1));
        assert!(paginated.chapter_start_page(1).unwrap() > 1);

        let p1 = paginated.resolve_page(1).unwrap();
        assert_eq!(p1.chapter_number, 1);
        assert_eq!(p1.chapter_title, "Capítulo I");
        assert!(!p1.paragraphs.iter().any(|p| p.contains("alert")));
        assert!(!p1.paragraphs.iter().any(|p| p.contains("color: red")));
        assert!(p1.paragraphs.iter().any(|p| p.contains("“comillas”")));

        let ch2_start = paginated.chapter_start_page(1).unwrap();
        let p_ch2 = paginated.resolve_page(ch2_start).unwrap();
        assert_eq!(p_ch2.chapter_number, 2);
        assert_eq!(p_ch2.chapter_title, "Capítulo II");
    }

    #[test]
    fn test_reader_service_resolves_real_chapters() {
        let mut book = Book {
            id: "test".into(),
            title: "Libro Real".into(),
            author: "Autor".into(),
            total_pages: 0,
            current_page: 1,
            genre: Genre::Fiction,
            cover_palette: CoverPalette {
                primary: 0,
                secondary: 0,
                accent: 0,
                text: 0,
            },
            synopsis: String::new(),
            chapters: Vec::new(),
        };

        let paginated = Paginator::paginate(
            &[ParsedChapter {
                index: 0,
                id: "ch1".into(),
                title: "Prólogo".into(),
                raw_xhtml: "<p>Texto real del libro.</p>".into(),
            }],
            PaginationConfig::default(),
        );
        book.chapters = paginated.chapters;
        book.total_pages = paginated.total_pages;

        let content = ReaderService::resolve_page(&book, 1);
        assert_eq!(content.chapter_title, "Prólogo");
        assert_eq!(content.paragraphs, vec!["Texto real del libro."]);
    }

    #[test]
    fn test_paragraph_splits_across_pages_when_space_remains() {
        let p1 = "Primer párrafo corto.".to_string();
        let p2 = "Palabra ".repeat(60);
        let ch = ParsedChapter {
            index: 0,
            id: "c1".into(),
            title: "1".into(),
            raw_xhtml: format!("<h1>1</h1><p>{p1}</p><p>{p2}</p>"),
        };

        let paginated = Paginator::paginate(&[ch], PaginationConfig { chars_per_page: 250 });
        assert!(paginated.total_pages >= 2);

        let page1 = paginated.resolve_page(1).unwrap();
        assert_eq!(page1.paragraphs.len(), 2);
        assert_eq!(page1.paragraphs[0], "Primer párrafo corto.");
        assert!(page1.paragraphs[1].starts_with("Palabra"));

        let page2 = paginated.resolve_page(2).unwrap();
        assert!(!page2.paragraphs.is_empty());
        assert!(page2.paragraphs[0].starts_with("Palabra"));
    }
}
