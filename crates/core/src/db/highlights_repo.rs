use rusqlite::{params, Connection};

use crate::db::migrations::current_unix_timestamp;
use crate::error::CoreResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightRecord {
    pub id: i64,
    pub book_id: String,
    pub page_number: u32,
    pub paragraph_index: usize,
    pub char_start: usize,
    pub char_end: usize,
    pub selected_text: String,
    pub note: String,
    pub color: String,
    pub created_at: i64,
}

pub struct HighlightRepository;

impl HighlightRepository {
    /// Crea y persiste una nueva cita/resaltado sobre texto seleccionado.
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        conn: &Connection,
        book_id: &str,
        page_number: u32,
        paragraph_index: usize,
        char_start: usize,
        char_end: usize,
        selected_text: &str,
        note: &str,
        color: &str,
    ) -> CoreResult<HighlightRecord> {
        let now = current_unix_timestamp();
        let clean_color = if color.trim().is_empty() {
            "amber"
        } else {
            color.trim()
        };
        conn.execute(
            "INSERT INTO highlights (
                book_id, page_number, paragraph_index, char_start, char_end,
                selected_text, note, color, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                book_id,
                page_number,
                paragraph_index as i64,
                char_start as i64,
                char_end as i64,
                selected_text.trim(),
                note.trim(),
                clean_color,
                now,
            ],
        )?;
        let id = conn.last_insert_rowid();
        Ok(HighlightRecord {
            id,
            book_id: book_id.to_string(),
            page_number,
            paragraph_index,
            char_start,
            char_end,
            selected_text: selected_text.trim().to_string(),
            note: note.trim().to_string(),
            color: clean_color.to_string(),
            created_at: now,
        })
    }

    /// Lista todas las citas y resaltados de un libro ordenados por página y posición.
    pub fn list_for_book(conn: &Connection, book_id: &str) -> CoreResult<Vec<HighlightRecord>> {
        let mut stmt = conn.prepare(
            "SELECT id, book_id, page_number, paragraph_index, char_start, char_end,
                    selected_text, note, color, created_at
             FROM highlights
             WHERE book_id = ?1
             ORDER BY page_number ASC, paragraph_index ASC, char_start ASC, id ASC",
        )?;

        let rows = stmt.query_map(params![book_id], Self::map_row)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// Lista los resaltados de una página específica de un libro.
    pub fn list_for_page(
        conn: &Connection,
        book_id: &str,
        page_number: u32,
    ) -> CoreResult<Vec<HighlightRecord>> {
        let mut stmt = conn.prepare(
            "SELECT id, book_id, page_number, paragraph_index, char_start, char_end,
                    selected_text, note, color, created_at
             FROM highlights
             WHERE book_id = ?1 AND page_number = ?2
             ORDER BY paragraph_index ASC, char_start ASC, id ASC",
        )?;

        let rows = stmt.query_map(params![book_id, page_number], Self::map_row)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// Actualiza la nota y color de una cita existente.
    pub fn update_note_and_color(
        conn: &Connection,
        id: i64,
        note: &str,
        color: &str,
    ) -> CoreResult<()> {
        let clean_color = if color.trim().is_empty() {
            "amber"
        } else {
            color.trim()
        };
        conn.execute(
            "UPDATE highlights SET note = ?1, color = ?2 WHERE id = ?3",
            params![note.trim(), clean_color, id],
        )?;
        Ok(())
    }

    /// Elimina una cita/resaltado por ID.
    pub fn delete(conn: &Connection, id: i64) -> CoreResult<()> {
        conn.execute("DELETE FROM highlights WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<HighlightRecord> {
        Ok(HighlightRecord {
            id: row.get("id")?,
            book_id: row.get("book_id")?,
            page_number: row.get::<_, i64>("page_number")?.max(1) as u32,
            paragraph_index: row.get::<_, i64>("paragraph_index")?.max(0) as usize,
            char_start: row.get::<_, i64>("char_start")?.max(0) as usize,
            char_end: row.get::<_, i64>("char_end")?.max(0) as usize,
            selected_text: row.get("selected_text")?,
            note: row.get("note")?,
            color: row.get("color")?,
            created_at: row.get("created_at")?,
        })
    }
}
