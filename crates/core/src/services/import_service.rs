use std::path::{Path, PathBuf};

use crate::db::{BookRepository, Database, StoredBook};
use crate::domain::book::{Book, CoverPalette};
use crate::epub::{EpubParser, ParsedEpub};
use crate::error::{CoreError, CoreResult};

/// Gestiona las rutas internas de almacenamiento para libros EPUB y portadas en caché.
#[derive(Debug, Clone)]
pub struct LibraryStorage {
    root_dir: PathBuf,
}

impl LibraryStorage {
    /// Crea un gestor de almacenamiento en el directorio estándar del sistema operativo.
    pub fn default_storage() -> CoreResult<Self> {
        let root_dir = Database::default_data_dir().ok_or_else(|| {
            CoreError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No se pudo determinar el directorio de datos del usuario",
            ))
        })?;
        Ok(Self { root_dir })
    }

    /// Crea un gestor de almacenamiento con una ruta raíz personalizada (ideal para pruebas).
    pub fn with_root(root_dir: impl AsRef<Path>) -> Self {
        Self {
            root_dir: root_dir.as_ref().to_path_buf(),
        }
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    pub fn books_dir(&self) -> PathBuf {
        self.root_dir.join("books")
    }

    pub fn covers_dir(&self) -> PathBuf {
        self.root_dir.join("covers")
    }

    pub fn ensure_dirs(&self) -> CoreResult<()> {
        std::fs::create_dir_all(self.books_dir())?;
        std::fs::create_dir_all(self.covers_dir())?;
        Ok(())
    }
}

pub struct BookImportService;

impl BookImportService {
    /// Importa un archivo `.epub`, lo copia al almacén interno de la aplicación,
    /// extrae su portada y metadatos, y lo registra de forma transaccional en SQLite.
    pub fn import_file(
        db: &mut Database,
        storage: &LibraryStorage,
        source_path: impl AsRef<Path>,
    ) -> CoreResult<StoredBook> {
        let source_path = source_path.as_ref();
        let parsed = EpubParser::parse_file(source_path)?;
        Self::import_parsed(db, storage, Some(source_path), parsed)
    }

    /// Persiste un EPUB ya parseado en el almacén interno y en SQLite.
    pub fn import_parsed(
        db: &mut Database,
        storage: &LibraryStorage,
        source_path: Option<&Path>,
        parsed: ParsedEpub,
    ) -> CoreResult<StoredBook> {
        storage.ensure_dirs()?;

        let book_id = generate_book_id(&parsed.metadata.title, &parsed.metadata.author);

        let dest_epub_str = if let Some(src) = source_path {
            let dest_epub_path = storage.books_dir().join(format!("{book_id}.epub"));
            std::fs::copy(src, &dest_epub_path)?;
            Some(dest_epub_path.to_string_lossy().to_string())
        } else {
            None
        };

        let cover_path_str = if let Some((bytes, ext)) = &parsed.cover {
            let dest_cover = storage.covers_dir().join(format!("{book_id}.{ext}"));
            std::fs::write(&dest_cover, bytes)?;
            Some(dest_cover.to_string_lossy().to_string())
        } else {
            None
        };

        let total_chars: usize = parsed
            .chapters
            .iter()
            .map(|c| estimate_plain_text_len(&c.raw_xhtml))
            .sum();
        let estimated_pages = ((total_chars / 1800) as u32).max(1);

        let book = Book {
            id: book_id.clone(),
            title: parsed.metadata.title,
            author: parsed.metadata.author,
            total_pages: estimated_pages,
            current_page: 0,
            genre: parsed.metadata.genre,
            cover_palette: palette_from_seed(&book_id),
            synopsis: parsed.metadata.description,
            chapters: Vec::new(),
        };

        BookRepository::upsert(
            db.conn_mut(),
            &book,
            dest_epub_str.as_deref(),
            cover_path_str.as_deref(),
            &parsed.metadata.language,
        )?;

        BookRepository::get_by_id(db.conn(), &book_id)?.ok_or_else(|| {
            CoreError::NotFound(format!("El libro importado '{book_id}' no se encontró en SQLite"))
        })
    }
}

fn generate_book_id(title: &str, author: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in title.as_bytes().iter().chain(b"::").chain(author.as_bytes().iter()) {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("book_{hash:016x}")
}

fn palette_from_seed(seed: &str) -> CoverPalette {
    let palettes = [
        CoverPalette { primary: 0x1E293B, secondary: 0x334155, accent: 0x38BDF8, text: 0xF8FAFC },
        CoverPalette { primary: 0x31102F, secondary: 0x52214A, accent: 0xF472B6, text: 0xFDF2F8 },
        CoverPalette { primary: 0x14332B, secondary: 0x1E4D40, accent: 0x34D399, text: 0xECFDF5 },
        CoverPalette { primary: 0x3B1D14, secondary: 0x5C2E20, accent: 0xFB923C, text: 0xFFF7ED },
        CoverPalette { primary: 0x1E1B4B, secondary: 0x312E81, accent: 0xA78BFA, text: 0xF5F3FF },
    ];
    let idx = seed.bytes().fold(0usize, |acc, b| acc.wrapping_add(b as usize)) % palettes.len();
    palettes[idx]
}

fn estimate_plain_text_len(xhtml: &str) -> usize {
    let mut len = 0usize;
    let mut inside_tag = false;
    for ch in xhtml.chars() {
        match ch {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag && !ch.is_control() => len += 1,
            _ => {}
        }
    }
    len
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::book::Genre;
    use crate::epub::{ParsedChapter, ParsedMetadata};

    #[test]
    fn test_import_parsed_saves_cover_and_persists_in_sqlite() {
        let temp_root = std::env::temp_dir().join("epub_reader_test_import_parsed");
        let _ = std::fs::remove_dir_all(&temp_root);

        let storage = LibraryStorage::with_root(&temp_root);
        let mut db = Database::open_in_memory().unwrap();

        let cover_bytes = vec![0x89, 0x50, 0x4E, 0x47];
        let parsed = ParsedEpub {
            metadata: ParsedMetadata {
                title: "El Aleph".into(),
                author: "Jorge Luis Borges".into(),
                description: "Colección de cuentos.".into(),
                language: "es".into(),
                genre: Genre::Fiction,
                identifier: Some("urn:isbn:9780000000001".into()),
            },
            cover: Some((cover_bytes.clone(), "png".into())),
            toc: Vec::new(),
            chapters: vec![ParsedChapter {
                index: 0,
                id: "ch1".into(),
                title: "Capítulo 1".into(),
                raw_xhtml: "<p>La candente mañana de febrero...</p>".repeat(120),
            }],
        };

        let stored = BookImportService::import_parsed(&mut db, &storage, None, parsed)
            .expect("debe importar libro parseado");

        assert_eq!(stored.book.title, "El Aleph");
        assert_eq!(stored.book.author, "Jorge Luis Borges");
        assert!(stored.book.total_pages >= 2);
        assert_eq!(stored.language, "es");

        let cover_path = stored.cover_path.expect("debe tener ruta de portada");
        assert!(Path::new(&cover_path).exists());
        assert_eq!(std::fs::read(&cover_path).unwrap(), cover_bytes);

        let _ = std::fs::remove_dir_all(&temp_root);
    }

    #[test]
    fn test_import_file_copies_epub_and_cover() {
        let temp_root = std::env::temp_dir().join("epub_reader_test_import_file");
        let _ = std::fs::remove_dir_all(&temp_root);
        std::fs::create_dir_all(&temp_root).unwrap();

        let src_epub = temp_root.join("origen.epub");
        crate::epub::tests::write_sample_epub(&src_epub);

        let storage = LibraryStorage::with_root(temp_root.join("storage"));
        let mut db = Database::open_in_memory().unwrap();

        let stored = BookImportService::import_file(&mut db, &storage, &src_epub)
            .expect("debe importar archivo epub real");

        assert_eq!(stored.book.title, "Ficciones");
        assert_eq!(stored.book.author, "Jorge Luis Borges");

        let copied_epub = stored.file_path.expect("debe tener ruta del epub copiado");
        assert!(Path::new(&copied_epub).exists());
        assert!(copied_epub.ends_with(".epub"));

        let cover_path = stored.cover_path.expect("debe tener portada extraída");
        assert!(Path::new(&cover_path).exists());
        assert!(cover_path.ends_with(".png"));

        let _ = std::fs::remove_dir_all(&temp_root);
    }
}
