use epub_core::db::{
    BookRepository, BookmarkRepository, Database, HighlightRepository, ProgressRepository,
};
use epub_core::domain::book::{Book, Chapter, CoverPalette, Genre};
use epub_core::services::{RagService, ReaderService, StatsService};
use rusqlite::params;

#[test]
fn test_e2e_full_reading_and_rag_lifecycle() {
    let mut db = Database::open_in_memory().expect("open in-memory db");

    // 1. Ingesta / alta de libro con 2 capítulos
    let book = Book {
        id: "e2e-repub-1".into(),
        title: "Ensayo sobre la ceguera".into(),
        author: "José Saramago".into(),
        total_pages: 4,
        current_page: 1,
        genre: Genre::Fiction,
        cover_palette: CoverPalette {
            primary: 0x2563EB,
            secondary: 0x1E40AF,
            accent: 0x60A5FA,
            text: 0xFFFFFF,
        },
        synopsis: "Novela de prueba E2E".into(),
        chapters: vec![
            Chapter {
                id: "ch1".into(),
                title: "Capítulo 1".into(),
                pages: vec![
                    "El semáforo cambió a verde pero el coche no se movió. El conductor gritó que estaba ciego.".into(),
                    "Lo llevaron al consultorio del oftalmólogo, quien no encontró ninguna lesión física.".into(),
                ],
            },
            Chapter {
                id: "ch2".into(),
                title: "Capítulo 2".into(),
                pages: vec![
                    "El gobierno decidió poner en cuarentena a los primeros ciegos en un manicomio abandonado.".into(),
                    "En el futuro capítulo final todos recuperan la vista tras la tormenta.".into(),
                ],
            },
        ],
    };

    BookRepository::upsert(db.conn_mut(), &book, None, None, "es").expect("upsert book");
    let loaded = BookRepository::get_by_id(db.conn(), &book.id)
        .expect("query book")
        .expect("book exists");
    assert_eq!(loaded.book.title, "Ensayo sobre la ceguera");

    // 2. Apertura en ReaderService y avance de páginas
    let page2 = ReaderService::resolve_page(&book, 2);
    assert!(page2.paragraphs[0].contains("oftalmólogo"));

    ProgressRepository::update_page(db.conn(), &book.id, 2, 0, 4, 1794780000).expect("update progress");
    ProgressRepository::add_reading_seconds(db.conn(), &book.id, 900).expect("add reading seconds");
    let stats = StatsService::compute_stats(db.conn(), "2026-11-15").expect("stats summary");
    assert!(stats.reading_hours > 0.0);
    assert!(stats.pages_read_total >= 2);

    // 3. Marcador + Cita resaltada con nota
    BookmarkRepository::save_with_note(db.conn(), &book.id, 2, 0, "Capítulo 1", "Diagnóstico inicial")
        .expect("save bookmark");
    assert!(BookmarkRepository::is_bookmarked(db.conn(), &book.id, 2).expect("is bookmarked"));

    let hl = HighlightRepository::create(
        db.conn(),
        &book.id,
        2,
        0,
        0,
        28,
        "Lo llevaron al consultorio",
        "Momento clave",
        "amber",
    )
    .expect("create highlight");
    assert_eq!(
        HighlightRepository::list_for_page(db.conn(), &book.id, 2)
            .expect("list page highlights")
            .len(),
        1
    );
    assert_eq!(hl.note, "Momento clave");

    // 4. Verificación de barrera anti-spoilers en RAG (página actual = 2)
    for (p_idx, text) in [
        (1u32, "El conductor gritó que estaba ciego en el semáforo."),
        (2u32, "El oftalmólogo examinó al primer ciego."),
        (3u32, "Los llevaron al manicomio abandonado."),
        (4u32, "SPOILER_FUTURO todos recuperan la vista."),
    ] {
        db.conn()
            .execute(
                "INSERT INTO book_chunks (book_id, chapter_index, chapter_title, page_start, page_end, chunk_index, content)
                 VALUES (?1, 0, 'Cap', ?2, ?2, ?2, ?3)",
                params![book.id, p_idx, text],
            )
            .expect("insert chunk");
    }

    let retrieved = RagService::retrieve_context(
        db.conn(),
        &book.id,
        "SPOILER_FUTURO recuperan la vista",
        2,
        None,
        6,
    )
    .expect("retrieve context");
    assert!(!retrieved.is_empty());
    assert!(
        retrieved
            .iter()
            .all(|c| c.page_start <= 2 && !c.content.contains("SPOILER_FUTURO")),
        "El RAG no debe filtrar ni devolver páginas futuras"
    );
}
