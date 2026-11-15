pub mod bookmarks_repo;
pub mod books_repo;
pub mod highlights_repo;
pub mod migrations;
pub mod progress_repo;
pub mod settings_repo;

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::error::CoreResult;

pub use bookmarks_repo::{BookmarkRecord, BookmarkRepository};
pub use books_repo::{BookRepository, StoredBook};
pub use highlights_repo::{HighlightRecord, HighlightRepository};
pub use progress_repo::{ProgressRepository, ReadingProgressRecord, ReadingSessionRecord};
pub use settings_repo::SettingsRepository;

/// Contenedor principal de conexión SQLite para la biblioteca local.
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Abre (o crea) la base de datos SQLite en la ruta indicada, habilita claves foráneas
    /// y modo WAL, y ejecuta las migraciones pendientes.
    pub fn open(path: impl AsRef<Path>) -> CoreResult<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let mut conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;",
        )?;
        migrations::run_migrations(&mut conn)?;

        Ok(Self { conn })
    }

    /// Abre una base de datos SQLite en memoria (`:memory:`), ideal para pruebas unitarias.
    pub fn open_in_memory() -> CoreResult<Self> {
        let mut conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        migrations::run_migrations(&mut conn)?;

        Ok(Self { conn })
    }

    /// Devuelve el directorio de datos estándar de la aplicación (`~/.local/share/epub-reader`
    /// en Linux o `%LOCALAPPDATA%/epub-reader` en Windows).
    pub fn default_data_dir() -> Option<PathBuf> {
        if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
            if !xdg_data.trim().is_empty() {
                return Some(PathBuf::from(xdg_data).join("epub-reader"));
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            if !home.trim().is_empty() {
                return Some(
                    PathBuf::from(home)
                        .join(".local")
                        .join("share")
                        .join("epub-reader"),
                );
            }
        }
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            if !local_app_data.trim().is_empty() {
                return Some(PathBuf::from(local_app_data).join("epub-reader"));
            }
        }
        if let Ok(app_data) = std::env::var("APPDATA") {
            if !app_data.trim().is_empty() {
                return Some(PathBuf::from(app_data).join("epub-reader"));
            }
        }
        None
    }

    /// Devuelve la ruta por defecto del archivo SQLite (`library.db`).
    pub fn default_db_path() -> Option<PathBuf> {
        Self::default_data_dir().map(|dir| dir.join("library.db"))
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::book::{Book, BookFilter, BookSort, CoverPalette, Genre};
    use crate::domain::settings::{AppSettings, CornerRadiusStyle};

    fn sample_book(id: &str, title: &str, author: &str, total_pages: u32, current_page: u32) -> Book {
        Book {
            id: id.into(),
            title: title.into(),
            author: author.into(),
            total_pages,
            current_page,
            genre: Genre::SciFi,
            cover_palette: CoverPalette {
                primary: 0x1E293B,
                secondary: 0x334155,
                accent: 0x38BDF8,
                text: 0xF8FAFC,
            },
            synopsis: "Sinopsis de prueba.".into(),
            chapters: Vec::new(),
        }
    }

    #[test]
    fn test_migrations_are_idempotent() {
        let mut db = Database::open_in_memory().expect("debe abrir DB en memoria");
        migrations::run_migrations(db.conn_mut()).expect("segunda ejecución debe ser idempotente");

        let count: i64 = db
            .conn()
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_book_crud_filter_search_and_sort() {
        let mut db = Database::open_in_memory().unwrap();

        let b1 = sample_book("b1", "Dune", "Frank Herbert", 500, 150);
        let b2 = sample_book("b2", "Fundación", "Isaac Asimov", 320, 320);
        let b3 = sample_book("b3", "El Hobbit", "J.R.R. Tolkien", 300, 0);

        BookRepository::upsert(db.conn_mut(), &b1, Some("/books/b1.epub"), None, "es").unwrap();
        BookRepository::upsert(db.conn_mut(), &b2, Some("/books/b2.epub"), None, "es").unwrap();
        BookRepository::upsert(db.conn_mut(), &b3, None, None, "es").unwrap();

        assert_eq!(BookRepository::count(db.conn()).unwrap(), 3);

        let fetched = BookRepository::get_by_id(db.conn(), "b1").unwrap().unwrap();
        assert_eq!(fetched.book.title, "Dune");
        assert_eq!(fetched.book.current_page, 150);
        assert_eq!(fetched.file_path.as_deref(), Some("/books/b1.epub"));

        let reading = BookRepository::list(db.conn(), BookFilter::Reading, BookSort::TitleAsc, "").unwrap();
        assert_eq!(reading.len(), 1);
        assert_eq!(reading[0].book.id, "b1");

        let completed = BookRepository::list(db.conn(), BookFilter::Completed, BookSort::TitleAsc, "").unwrap();
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].book.id, "b2");

        let unread = BookRepository::list(db.conn(), BookFilter::Unread, BookSort::TitleAsc, "").unwrap();
        assert_eq!(unread.len(), 1);
        assert_eq!(unread[0].book.id, "b3");

        let search_asimov = BookRepository::list(db.conn(), BookFilter::All, BookSort::TitleAsc, "asimov").unwrap();
        assert_eq!(search_asimov.len(), 1);
        assert_eq!(search_asimov[0].book.id, "b2");

        let sorted_pages = BookRepository::list(db.conn(), BookFilter::All, BookSort::TotalPagesDesc, "").unwrap();
        assert_eq!(sorted_pages[0].book.id, "b1");

        assert!(BookRepository::delete(db.conn(), "b3").unwrap());
        assert_eq!(BookRepository::count(db.conn()).unwrap(), 2);
        assert!(ProgressRepository::get_progress(db.conn(), "b3").unwrap().is_none());
    }

    #[test]
    fn test_progress_favorites_and_sessions() {
        let mut db = Database::open_in_memory().unwrap();
        let b1 = sample_book("b1", "Crónicas Marcianas", "Ray Bradbury", 200, 0);
        let b2 = sample_book("b2", "Solaris", "Stanisław Lem", 250, 0);

        BookRepository::upsert(db.conn_mut(), &b1, None, None, "es").unwrap();
        BookRepository::upsert(db.conn_mut(), &b2, None, None, "es").unwrap();

        ProgressRepository::update_page(db.conn(), "b1", 50, 3, 200, 1000).unwrap();
        ProgressRepository::update_page(db.conn(), "b2", 250, 10, 250, 2000).unwrap();

        let prog1 = ProgressRepository::get_progress(db.conn(), "b1").unwrap().unwrap();
        assert_eq!(prog1.current_page, 50);
        assert_eq!(prog1.current_chapter, 3);
        assert_eq!(prog1.status, "Reading");
        assert!((prog1.percent - 0.25).abs() < 0.001);

        let prog2 = ProgressRepository::get_progress(db.conn(), "b2").unwrap().unwrap();
        assert_eq!(prog2.status, "Completed");

        // Si el lector repagina a un total distinto, `books.total_pages` se sincroniza.
        ProgressRepository::update_page(db.conn(), "b2", 180, 10, 180, 2100).unwrap();
        let synced = BookRepository::get_by_id(db.conn(), "b2").unwrap().unwrap();
        assert_eq!(synced.book.total_pages, 180);
        assert!(synced.book.is_completed());
        assert!((synced.book.progress_pct() - 1.0).abs() < 0.001);

        let recent = ProgressRepository::get_most_recent_book(db.conn()).unwrap().unwrap();
        assert_eq!(recent.book.id, "b2");

        assert!(ProgressRepository::toggle_favorite(db.conn(), "b1").unwrap());
        assert!(!ProgressRepository::toggle_favorite(db.conn(), "b1").unwrap());

        ProgressRepository::record_session(db.conn_mut(), "b1", "2026-11-13", 15, 12).unwrap();
        ProgressRepository::record_session(db.conn_mut(), "b1", "2026-11-13", 10, 8).unwrap();

        let sessions = ProgressRepository::get_sessions_since(db.conn(), "2026-11-01").unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].minutes_read, 25);
        assert_eq!(sessions[0].pages_read, 20);

        let prog1_after = ProgressRepository::get_progress(db.conn(), "b1").unwrap().unwrap();
        assert_eq!(prog1_after.reading_time_seconds, 25 * 60);
    }

    #[test]
    fn test_settings_persistence() {
        let db = Database::open_in_memory().unwrap();

        let initial = SettingsRepository::load_settings(db.conn()).unwrap();
        assert_eq!(initial, AppSettings::default());

        let mut custom = AppSettings::default();
        custom.appearance.theme_id = "tokyo-night".into();
        custom.appearance.corner_radius = CornerRadiusStyle::Circular;
        custom.appearance.window_opacity = 0.85;
        custom.reader.font_size = 20;

        SettingsRepository::save_settings(db.conn(), &custom).unwrap();
        let reloaded = SettingsRepository::load_settings(db.conn()).unwrap();
        assert_eq!(reloaded, custom);
    }

    #[test]
    fn test_filter_favorites() {
        let mut db = Database::open_in_memory().unwrap();
        let b1 = sample_book("fav-1", "Dune", "Frank Herbert", 400, 0);
        let b2 = sample_book("fav-2", "Fundación", "Isaac Asimov", 300, 0);

        BookRepository::upsert(db.conn_mut(), &b1, None, None, "es").unwrap();
        BookRepository::upsert(db.conn_mut(), &b2, None, None, "es").unwrap();

        assert!(ProgressRepository::toggle_favorite(db.conn(), "fav-1").unwrap());

        let favs =
            BookRepository::list(db.conn(), BookFilter::Favorites, BookSort::TitleAsc, "").unwrap();
        assert_eq!(favs.len(), 1);
        assert_eq!(favs[0].book.id, "fav-1");
        assert!(favs[0].is_favorite);
    }

    #[test]
    fn test_legacy_schema_upgrades_missing_columns() {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE reading_progress (
                book_id TEXT PRIMARY KEY,
                current_page INTEGER NOT NULL DEFAULT 0
            );",
        )
        .unwrap();

        migrations::run_migrations(&mut conn).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM reading_progress WHERE last_read_at IS NULL AND status = 'Unread' AND is_favorite = 0",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_legacy_integer_pk_rebuilds_for_text_ids() {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE books (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL
            );
            CREATE TABLE reading_progress (
                book_id INTEGER PRIMARY KEY,
                current_page INTEGER NOT NULL DEFAULT 0
            );",
        )
        .unwrap();

        migrations::run_migrations(&mut conn).unwrap();

        let b = sample_book("uuid-text-id", "El Aleph", "Jorge Luis Borges", 180, 0);
        BookRepository::upsert(&mut conn, &b, None, None, "es").unwrap();
        let loaded = BookRepository::get_by_id(&conn, "uuid-text-id").unwrap();
        assert!(loaded.is_some());
    }

    #[test]
    fn test_bookmarks_crud_and_toggle() {
        let mut db = Database::open_in_memory().unwrap();
        let b = sample_book("bm-1", "Rayuela", "Julio Cortázar", 500, 0);
        BookRepository::upsert(db.conn_mut(), &b, None, None, "es").unwrap();

        assert!(!BookmarkRepository::is_bookmarked(db.conn(), "bm-1", 12).unwrap());
        assert!(BookmarkRepository::toggle(db.conn(), "bm-1", 12, 1, "Capítulo 2").unwrap());
        assert!(BookmarkRepository::is_bookmarked(db.conn(), "bm-1", 12).unwrap());

        BookmarkRepository::save_with_note(
            db.conn(),
            "bm-1",
            45,
            3,
            "Capítulo 5",
            "Pasaje importante sobre la Maga",
        )
        .unwrap();

        let list = BookmarkRepository::list_for_book(db.conn(), "bm-1").unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].page_number, 12);
        assert_eq!(list[1].page_number, 45);
        assert_eq!(list[1].note, "Pasaje importante sobre la Maga");

        // Toggle removes existing bookmark on page 12
        assert!(!BookmarkRepository::toggle(db.conn(), "bm-1", 12, 1, "Capítulo 2").unwrap());
        assert_eq!(BookmarkRepository::list_for_book(db.conn(), "bm-1").unwrap().len(), 1);
    }

    #[test]
    fn test_highlights_crud_and_page_filter() {
        let mut db = Database::open_in_memory().unwrap();
        let b = sample_book("hl-1", "Ensayo sobre la ceguera", "José Saramago", 310, 0);
        BookRepository::upsert(db.conn_mut(), &b, None, None, "es").unwrap();

        let h1 = HighlightRepository::create(
            db.conn(),
            "hl-1",
            12,
            1,
            5,
            28,
            "la ceguera blanca",
            "Tema central",
            "amber",
        )
        .unwrap();
        let h2 = HighlightRepository::create(
            db.conn(),
            "hl-1",
            45,
            0,
            0,
            15,
            "dentro del pabellón",
            "",
            "emerald",
        )
        .unwrap();

        let all = HighlightRepository::list_for_book(db.conn(), "hl-1").unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].selected_text, "la ceguera blanca");
        assert_eq!(all[1].color, "emerald");

        let p12 = HighlightRepository::list_for_page(db.conn(), "hl-1", 12).unwrap();
        assert_eq!(p12.len(), 1);
        assert_eq!(p12[0].id, h1.id);

        HighlightRepository::update_note_and_color(db.conn(), h2.id, "Nota editada", "sky").unwrap();
        let p45 = HighlightRepository::list_for_page(db.conn(), "hl-1", 45).unwrap();
        assert_eq!(p45[0].note, "Nota editada");
        assert_eq!(p45[0].color, "sky");

        HighlightRepository::delete(db.conn(), h1.id).unwrap();
        assert_eq!(HighlightRepository::list_for_book(db.conn(), "hl-1").unwrap().len(), 1);
    }
}
