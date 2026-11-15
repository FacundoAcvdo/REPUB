use rusqlite::{params, Connection, OptionalExtension};

use crate::db::migrations::current_unix_timestamp;
use crate::error::CoreResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookmarkRecord {
    pub id: i64,
    pub book_id: String,
    pub page_number: u32,
    pub chapter_index: u32,
    pub chapter_title: String,
    pub note: String,
    pub created_at: i64,
}

pub struct BookmarkRepository;

impl BookmarkRepository {
    /// Lista todos los marcadores de un libro ordenados por número de página ascendente.
    pub fn list_for_book(conn: &Connection, book_id: &str) -> CoreResult<Vec<BookmarkRecord>> {
        let mut stmt = conn.prepare(
            "SELECT id, book_id, page_number, chapter_index, chapter_title, note, created_at
             FROM bookmarks
             WHERE book_id = ?1
             ORDER BY page_number ASC, created_at DESC",
        )?;

        let rows = stmt.query_map(params![book_id], |row| {
            Ok(BookmarkRecord {
                id: row.get("id")?,
                book_id: row.get("book_id")?,
                page_number: row.get::<_, i64>("page_number")?.max(1) as u32,
                chapter_index: row.get::<_, i64>("chapter_index")?.max(0) as u32,
                chapter_title: row.get("chapter_title")?,
                note: row.get("note")?,
                created_at: row.get("created_at")?,
            })
        })?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    /// Indica si existe un marcador guardado en la página indicada del libro.
    pub fn is_bookmarked(conn: &Connection, book_id: &str, page_number: u32) -> CoreResult<bool> {
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM bookmarks WHERE book_id = ?1 AND page_number = ?2)",
            params![book_id, page_number],
            |row| row.get(0),
        )?;
        Ok(exists)
    }

    /// Alterna un marcador en la página actual. Devuelve `true` si se creó o `false` si se eliminó.
    pub fn toggle(
        conn: &Connection,
        book_id: &str,
        page_number: u32,
        chapter_index: u32,
        chapter_title: &str,
    ) -> CoreResult<bool> {
        let existing_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM bookmarks WHERE book_id = ?1 AND page_number = ?2 LIMIT 1",
                params![book_id, page_number],
                |row| row.get(0),
            )
            .optional()?;

        if let Some(id) = existing_id {
            conn.execute("DELETE FROM bookmarks WHERE id = ?1", params![id])?;
            return Ok(false);
        }

        let now = current_unix_timestamp();
        conn.execute(
            "INSERT INTO bookmarks (book_id, page_number, chapter_index, chapter_title, note, created_at)
             VALUES (?1, ?2, ?3, ?4, '', ?5)",
            params![book_id, page_number, chapter_index, chapter_title, now],
        )?;
        Ok(true)
    }

    /// Crea o actualiza un marcador con nota en la página indicada.
    pub fn save_with_note(
        conn: &Connection,
        book_id: &str,
        page_number: u32,
        chapter_index: u32,
        chapter_title: &str,
        note: &str,
    ) -> CoreResult<()> {
        let existing_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM bookmarks WHERE book_id = ?1 AND page_number = ?2 LIMIT 1",
                params![book_id, page_number],
                |row| row.get(0),
            )
            .optional()?;

        let trimmed_note = note.trim();
        if let Some(id) = existing_id {
            conn.execute(
                "UPDATE bookmarks SET note = ?1, chapter_title = ?2 WHERE id = ?3",
                params![trimmed_note, chapter_title, id],
            )?;
        } else {
            let now = current_unix_timestamp();
            conn.execute(
                "INSERT INTO bookmarks (book_id, page_number, chapter_index, chapter_title, note, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    book_id,
                    page_number,
                    chapter_index,
                    chapter_title,
                    trimmed_note,
                    now
                ],
            )?;
        }
        Ok(())
    }

    /// Elimina un marcador por su identificador.
    pub fn delete(conn: &Connection, bookmark_id: i64) -> CoreResult<bool> {
        let affected = conn.execute("DELETE FROM bookmarks WHERE id = ?1", params![bookmark_id])?;
        Ok(affected > 0)
    }
}
