use rusqlite::{params, Connection, OptionalExtension};

use crate::db::books_repo::{BookRepository, StoredBook};
use crate::error::{CoreError, CoreResult};

#[derive(Debug, Clone, PartialEq)]
pub struct ReadingProgressRecord {
    pub book_id: String,
    pub current_page: u32,
    pub current_chapter: u32,
    pub percent: f32,
    pub last_read_at: Option<i64>,
    pub reading_time_seconds: u64,
    pub status: String,
    pub is_favorite: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadingSessionRecord {
    pub id: i64,
    pub book_id: String,
    pub date: String,
    pub minutes_read: u32,
    pub pages_read: u32,
}

pub struct ProgressRepository;

impl ProgressRepository {
    /// Obtiene el progreso de lectura de un libro.
    pub fn get_progress(
        conn: &Connection,
        book_id: &str,
    ) -> CoreResult<Option<ReadingProgressRecord>> {
        let sql = "
            SELECT book_id, current_page, current_chapter, percent,
                   last_read_at, reading_time_seconds, status, is_favorite
            FROM reading_progress
            WHERE book_id = ?1
        ";

        conn.query_row(sql, params![book_id], |row| {
            let is_fav: i64 = row.get("is_favorite")?;
            let secs: i64 = row.get("reading_time_seconds")?;
            let percent: f64 = row.get("percent")?;

            Ok(ReadingProgressRecord {
                book_id: row.get("book_id")?,
                current_page: row.get::<_, i64>("current_page")?.max(0) as u32,
                current_chapter: row.get::<_, i64>("current_chapter")?.max(0) as u32,
                percent: percent as f32,
                last_read_at: row.get("last_read_at")?,
                reading_time_seconds: secs.max(0) as u64,
                status: row.get("status")?,
                is_favorite: is_fav != 0,
            })
        })
        .optional()
        .map_err(Into::into)
    }

    /// Actualiza la página y capítulo actual del libro, recalculando porcentaje y estado.
    pub fn update_page(
        conn: &Connection,
        book_id: &str,
        current_page: u32,
        current_chapter: u32,
        total_pages: u32,
        timestamp: i64,
    ) -> CoreResult<()> {
        let percent = if total_pages == 0 {
            0.0_f64
        } else {
            (current_page as f64 / total_pages as f64).clamp(0.0, 1.0)
        };
        let status = BookRepository::compute_status(current_page, total_pages);

        let updated = conn.execute(
            "UPDATE reading_progress
             SET current_page = ?1,
                 current_chapter = ?2,
                 percent = ?3,
                 last_read_at = ?4,
                 status = ?5
             WHERE book_id = ?6",
            params![current_page, current_chapter, percent, timestamp, status, book_id],
        )?;

        if updated == 0 {
            return Err(CoreError::NotFound(format!(
                "No existe progreso registrado para el libro '{book_id}'"
            )));
        }

        if total_pages > 0 {
            BookRepository::update_total_pages(conn, book_id, total_pages)?;
        }

        Ok(())
    }

    /// Alterna el estado de favorito de un libro y devuelve el nuevo valor booleano.
    pub fn toggle_favorite(conn: &Connection, book_id: &str) -> CoreResult<bool> {
        let current: Option<i64> = conn
            .query_row(
                "SELECT is_favorite FROM reading_progress WHERE book_id = ?1",
                params![book_id],
                |row| row.get(0),
            )
            .optional()?;

        let current_val = current.ok_or_else(|| {
            CoreError::NotFound(format!("Libro no encontrado en progreso: '{book_id}'"))
        })?;

        let next_val = if current_val == 0 { 1 } else { 0 };
        conn.execute(
            "UPDATE reading_progress SET is_favorite = ?1 WHERE book_id = ?2",
            params![next_val, book_id],
        )?;

        Ok(next_val != 0)
    }

    /// Devuelve el libro leído más recientemente (para la tarjeta "Continuar leyendo").
    pub fn get_most_recent_book(conn: &Connection) -> CoreResult<Option<StoredBook>> {
        let id: Option<String> = conn
            .query_row(
                "SELECT book_id
                 FROM reading_progress
                 WHERE last_read_at IS NOT NULL
                 ORDER BY last_read_at DESC
                 LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()?;

        match id {
            Some(book_id) => BookRepository::get_by_id(conn, &book_id),
            None => Ok(None),
        }
    }

    /// Registra actividad de lectura para una fecha ISO (`YYYY-MM-DD`) e incrementa el tiempo total.
    pub fn record_session(
        conn: &mut Connection,
        book_id: &str,
        date: &str,
        minutes_read: u32,
        pages_read: u32,
    ) -> CoreResult<()> {
        let tx = conn.transaction()?;

        let existing_id: Option<i64> = tx
            .query_row(
                "SELECT id FROM reading_sessions WHERE book_id = ?1 AND date = ?2 LIMIT 1",
                params![book_id, date],
                |row| row.get(0),
            )
            .optional()?;

        if let Some(session_id) = existing_id {
            tx.execute(
                "UPDATE reading_sessions
                 SET minutes_read = minutes_read + ?1,
                     pages_read = pages_read + ?2
                 WHERE id = ?3",
                params![minutes_read, pages_read, session_id],
            )?;
        } else {
            tx.execute(
                "INSERT INTO reading_sessions (book_id, date, minutes_read, pages_read)
                 VALUES (?1, ?2, ?3, ?4)",
                params![book_id, date, minutes_read, pages_read],
            )?;
        }

        let extra_secs = (minutes_read as i64) * 60;
        tx.execute(
            "UPDATE reading_progress
             SET reading_time_seconds = reading_time_seconds + ?1
             WHERE book_id = ?2",
            params![extra_secs, book_id],
        )?;

        tx.commit()?;
        Ok(())
    }

    /// Incrementa segundos activos de lectura en `reading_progress` sin esperar a completar un minuto.
    pub fn add_reading_seconds(conn: &Connection, book_id: &str, seconds: u64) -> CoreResult<()> {
        if seconds == 0 {
            return Ok(());
        }
        conn.execute(
            "UPDATE reading_progress
             SET reading_time_seconds = reading_time_seconds + ?1
             WHERE book_id = ?2",
            params![seconds as i64, book_id],
        )?;
        Ok(())
    }

    /// Obtiene todas las sesiones registradas desde una fecha ISO inclusive (`YYYY-MM-DD`).
    pub fn get_sessions_since(
        conn: &Connection,
        start_date: &str,
    ) -> CoreResult<Vec<ReadingSessionRecord>> {
        let mut stmt = conn.prepare(
            "SELECT id, book_id, date, minutes_read, pages_read
             FROM reading_sessions
             WHERE date >= ?1
             ORDER BY date ASC, id ASC",
        )?;

        let rows = stmt.query_map(params![start_date], |row| {
            Ok(ReadingSessionRecord {
                id: row.get("id")?,
                book_id: row.get("book_id")?,
                date: row.get("date")?,
                minutes_read: row.get::<_, i64>("minutes_read")?.max(0) as u32,
                pages_read: row.get::<_, i64>("pages_read")?.max(0) as u32,
            })
        })?;

        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }
}
