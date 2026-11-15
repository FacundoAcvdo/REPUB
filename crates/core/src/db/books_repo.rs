use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::db::migrations::current_unix_timestamp;
use crate::domain::book::{Book, BookFilter, BookSort, CoverPalette, Genre};
use crate::error::CoreResult;

/// Registro completo de libro almacenado en SQLite junto a sus metadatos de archivo y progreso.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredBook {
    pub book: Book,
    pub file_path: Option<String>,
    pub cover_path: Option<String>,
    pub language: String,
    pub is_favorite: bool,
    pub ai_enabled: bool,
    pub last_read_at: Option<i64>,
    pub reading_time_seconds: u64,
}

pub struct BookRepository;

impl BookRepository {
    /// Inserta o actualiza un libro en SQLite e inicializa su registro en `reading_progress`
    /// dentro de una transacción atómica.
    pub fn upsert(
        conn: &mut Connection,
        book: &Book,
        file_path: Option<&str>,
        cover_path: Option<&str>,
        language: &str,
    ) -> CoreResult<()> {
        let now = current_unix_timestamp();
        let genre_json = serde_json::to_string(&book.genre)?;
        let palette_json = serde_json::to_string(&book.cover_palette)?;

        let status = Self::compute_status(book.current_page, book.total_pages);
        let percent = book.progress_pct() as f64;

        let tx = conn.transaction()?;

        tx.execute(
            "INSERT INTO books (
                id, title, author, description, genre, total_pages,
                file_path, cover_path, cover_color, language, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                author = excluded.author,
                description = excluded.description,
                genre = excluded.genre,
                total_pages = excluded.total_pages,
                file_path = COALESCE(excluded.file_path, books.file_path),
                cover_path = COALESCE(excluded.cover_path, books.cover_path),
                cover_color = excluded.cover_color,
                language = excluded.language,
                updated_at = excluded.updated_at",
            params![
                book.id,
                book.title,
                book.author,
                book.synopsis,
                genre_json,
                book.total_pages,
                file_path,
                cover_path,
                palette_json,
                language,
                now,
                now,
            ],
        )?;

        tx.execute(
            "INSERT INTO reading_progress (
                book_id, current_page, current_chapter, percent,
                last_read_at, reading_time_seconds, status, is_favorite
             ) VALUES (?1, ?2, 0, ?3, NULL, 0, ?4, 0)
             ON CONFLICT(book_id) DO NOTHING",
            params![book.id, book.current_page, percent, status],
        )?;

        tx.commit()?;
        Ok(())
    }

    /// Obtiene un libro por su identificador único.
    pub fn get_by_id(conn: &Connection, id: &str) -> CoreResult<Option<StoredBook>> {
        let sql = "
            SELECT
                b.id, b.title, b.author, b.description, b.genre, b.total_pages,
                b.file_path, b.cover_path, b.cover_color, b.language,
                COALESCE(b.ai_enabled, 1) AS ai_enabled,
                COALESCE(p.current_page, 0) AS current_page,
                COALESCE(p.is_favorite, 0) AS is_favorite,
                p.last_read_at,
                COALESCE(p.reading_time_seconds, 0) AS reading_time_seconds
            FROM books b
            LEFT JOIN reading_progress p ON b.id = p.book_id
            WHERE b.id = ?1
        ";

        conn.query_row(sql, params![id], Self::map_row)
            .optional()
            .map_err(Into::into)
    }

    /// Lista libros aplicando filtros de estado, búsqueda textual y criterio de ordenamiento.
    pub fn list(
        conn: &Connection,
        filter: BookFilter,
        sort: BookSort,
        query: &str,
    ) -> CoreResult<Vec<StoredBook>> {
        let status_clause = match filter {
            BookFilter::All => "1=1",
            BookFilter::Reading => "p.status = 'Reading'",
            BookFilter::Completed => "p.status = 'Completed'",
            BookFilter::Unread => "COALESCE(p.status, 'Unread') = 'Unread'",
            BookFilter::Favorites => "COALESCE(p.is_favorite, 0) = 1",
        };

        let order_clause = match sort {
            BookSort::TitleAsc => "b.title COLLATE NOCASE ASC",
            BookSort::TitleDesc => "b.title COLLATE NOCASE DESC",
            BookSort::AuthorAsc => "b.author COLLATE NOCASE ASC, b.title COLLATE NOCASE ASC",
            BookSort::RecentProgress => "COALESCE(p.percent, 0.0) DESC, b.title COLLATE NOCASE ASC",
            BookSort::TotalPagesDesc => "b.total_pages DESC, b.title COLLATE NOCASE ASC",
        };

        let q = query.trim().to_lowercase();
        let search_pattern = format!("%{q}%");

        let sql = format!(
            "SELECT
                b.id, b.title, b.author, b.description, b.genre, b.total_pages,
                b.file_path, b.cover_path, b.cover_color, b.language,
                COALESCE(b.ai_enabled, 1) AS ai_enabled,
                COALESCE(p.current_page, 0) AS current_page,
                COALESCE(p.is_favorite, 0) AS is_favorite,
                p.last_read_at,
                COALESCE(p.reading_time_seconds, 0) AS reading_time_seconds
            FROM books b
            LEFT JOIN reading_progress p ON b.id = p.book_id
            WHERE ({status_clause})
              AND (?1 = '' OR LOWER(b.title) LIKE ?2 OR LOWER(b.author) LIKE ?2 OR LOWER(b.genre) LIKE ?2)
            ORDER BY {order_clause}"
        );

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params![q, search_pattern], Self::map_row)?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    /// Actualiza el total de páginas almacenado para un libro si cambió al repaginar.
    pub fn update_total_pages(
        conn: &Connection,
        book_id: &str,
        total_pages: u32,
    ) -> CoreResult<()> {
        conn.execute(
            "UPDATE books SET total_pages = ?1 WHERE id = ?2 AND total_pages != ?1",
            params![total_pages, book_id],
        )?;
        Ok(())
    }

    /// Habilita o deshabilita la asistencia de IA y la indexación RAG para un libro específico.
    pub fn set_ai_enabled(conn: &Connection, book_id: &str, enabled: bool) -> CoreResult<()> {
        let flag: i64 = if enabled { 1 } else { 0 };
        conn.execute(
            "UPDATE books SET ai_enabled = ?1 WHERE id = ?2",
            params![flag, book_id],
        )?;
        Ok(())
    }

    /// Indica si el libro tiene habilitada la asistencia de IA.
    pub fn is_ai_enabled(conn: &Connection, book_id: &str) -> CoreResult<bool> {
        let flag: Option<i64> = conn
            .query_row(
                "SELECT COALESCE(ai_enabled, 1) FROM books WHERE id = ?1",
                params![book_id],
                |row| row.get(0),
            )
            .optional()?;
        Ok(flag.unwrap_or(1) != 0)
    }

    /// Elimina un libro por su ID (las tablas relacionadas se eliminan por cascada).
    pub fn delete(conn: &Connection, id: &str) -> CoreResult<bool> {
        let affected = conn.execute("DELETE FROM books WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    /// Devuelve la cantidad total de libros registrados en la biblioteca.
    pub fn count(conn: &Connection) -> CoreResult<u32> {
        let total: i64 = conn.query_row("SELECT COUNT(*) FROM books", [], |row| row.get(0))?;
        Ok(total.max(0) as u32)
    }

    pub(crate) fn compute_status(current_page: u32, total_pages: u32) -> &'static str {
        if total_pages > 0 && current_page >= total_pages {
            "Completed"
        } else if current_page > 0 {
            "Reading"
        } else {
            "Unread"
        }
    }

    fn map_row(row: &Row<'_>) -> rusqlite::Result<StoredBook> {
        let genre_str: String = row.get("genre")?;
        let genre: Genre = serde_json::from_str(&genre_str).unwrap_or(Genre::Fiction);

        let palette_str: String = row.get("cover_color")?;
        let cover_palette: CoverPalette =
            serde_json::from_str(&palette_str).unwrap_or(CoverPalette {
                primary: 0x2D3142,
                secondary: 0x4F5D75,
                accent: 0xEF8354,
                text: 0xFFFFFF,
            });

        let book = Book {
            id: row.get("id")?,
            title: row.get("title")?,
            author: row.get("author")?,
            total_pages: row.get::<_, i64>("total_pages")?.max(0) as u32,
            current_page: row.get::<_, i64>("current_page")?.max(0) as u32,
            genre,
            cover_palette,
            synopsis: row.get("description")?,
            chapters: Vec::new(),
        };

        let is_fav_int: i64 = row.get("is_favorite")?;
        let ai_enabled_int: i64 = row.get("ai_enabled")?;
        let reading_secs: i64 = row.get("reading_time_seconds")?;

        Ok(StoredBook {
            book,
            file_path: row.get("file_path")?,
            cover_path: row.get("cover_path")?,
            language: row.get("language")?,
            is_favorite: is_fav_int != 0,
            ai_enabled: ai_enabled_int != 0,
            last_read_at: row.get("last_read_at")?,
            reading_time_seconds: reading_secs.max(0) as u64,
        })
    }
}
