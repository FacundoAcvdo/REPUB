use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension};

use crate::db::{BookRepository, ProgressRepository};
use crate::domain::book::Book;
use crate::domain::stats::ReadingStats;
use crate::error::CoreResult;

pub struct StatsService;

impl StatsService {
    /// Calcula las métricas reales de lectura desde SQLite para una fecha de referencia (`YYYY-MM-DD`).
    pub fn compute_stats(conn: &Connection, today_iso: &str) -> CoreResult<ReadingStats> {
        let books_completed: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM reading_progress WHERE status = 'Completed'",
                [],
                |row| row.get::<_, i64>(0),
            )?
            .max(0) as u32;

        let total_secs: i64 = conn.query_row(
            "SELECT COALESCE(SUM(reading_time_seconds), 0) FROM reading_progress",
            [],
            |row| row.get(0),
        )?;
        let reading_hours = (total_secs.max(0) as f32) / 3600.0;

        let session_pages_total: i64 = conn.query_row(
            "SELECT COALESCE(SUM(pages_read), 0) FROM reading_sessions",
            [],
            |row| row.get(0),
        )?;
        let progress_pages_total: i64 = conn.query_row(
            "SELECT COALESCE(SUM(current_page), 0) FROM reading_progress",
            [],
            |row| row.get(0),
        )?;
        let pages_read_total = session_pages_total.max(progress_pages_total).max(0) as u32;

        let year_prefix = today_iso.get(0..4).unwrap_or("2026");
        let year_pattern = format!("{year_prefix}-%");
        let pages_read_this_year: u32 = conn
            .query_row(
                "SELECT COALESCE(SUM(pages_read), 0) FROM reading_sessions WHERE date LIKE ?1",
                params![year_pattern],
                |row| row.get::<_, i64>(0),
            )?
            .max(0) as u32;
        let pages_read_this_year = pages_read_this_year.max(pages_read_total);

        let weekly_activity = compute_weekly_activity(conn, today_iso)?;
        let current_streak_days = compute_streak(conn, today_iso)?;

        Ok(ReadingStats {
            books_completed,
            pages_read_total,
            pages_read_this_year,
            reading_hours,
            current_streak_days,
            weekly_activity,
        })
    }

    /// Devuelve el libro leído más recientemente, o el primer libro registrado en la biblioteca.
    pub fn get_continue_reading_book(conn: &Connection) -> CoreResult<Option<Book>> {
        if let Some(recent) = ProgressRepository::get_most_recent_book(conn)? {
            return Ok(Some(recent.book));
        }

        let first_id: Option<String> = conn
            .query_row("SELECT id FROM books ORDER BY updated_at DESC LIMIT 1", [], |row| {
                row.get(0)
            })
            .optional()?;

        match first_id {
            Some(id) => Ok(BookRepository::get_by_id(conn, &id)?.map(|s| s.book)),
            None => Ok(None),
        }
    }

    /// Fecha actual en formato ISO (`YYYY-MM-DD`).
    pub fn today_iso() -> String {
        let days = current_unix_days();
        let (y, m, d) = civil_from_days(days);
        format!("{y:04}-{m:02}-{d:02}")
    }

    /// Fecha actual legible en español (ej. `"Domingo, 15 de noviembre"`).
    pub fn today_display_es() -> String {
        let days = current_unix_days();
        let (_y, m, d) = civil_from_days(days);
        let weekday = weekday_mon_zero(days);

        let weekday_name = match weekday {
            0 => "Lunes",
            1 => "Martes",
            2 => "Miércoles",
            3 => "Jueves",
            4 => "Viernes",
            5 => "Sábado",
            _ => "Domingo",
        };
        let month_name = match m {
            1 => "enero",
            2 => "febrero",
            3 => "marzo",
            4 => "abril",
            5 => "mayo",
            6 => "junio",
            7 => "julio",
            8 => "agosto",
            9 => "septiembre",
            10 => "octubre",
            11 => "noviembre",
            _ => "diciembre",
        };

        format!("{weekday_name}, {d} de {month_name}")
    }
}

fn compute_weekly_activity(conn: &Connection, today_iso: &str) -> CoreResult<[u32; 7]> {
    let mut weekly = [0u32; 7];
    let Some(today_days) = parse_iso_days(today_iso) else {
        return Ok(weekly);
    };

    let weekday_idx = weekday_mon_zero(today_days) as i64;
    let monday_days = today_days - weekday_idx;
    let (my, mm, md) = civil_from_days(monday_days);
    let monday_iso = format!("{my:04}-{mm:02}-{md:02}");

    let sessions = ProgressRepository::get_sessions_since(conn, &monday_iso)?;
    for s in sessions {
        if let Some(s_days) = parse_iso_days(&s.date) {
            let offset = s_days - monday_days;
            if (0..7).contains(&offset) {
                weekly[offset as usize] = weekly[offset as usize].saturating_add(s.pages_read);
            }
        }
    }

    Ok(weekly)
}

fn compute_streak(conn: &Connection, today_iso: &str) -> CoreResult<u32> {
    let Some(today_days) = parse_iso_days(today_iso) else {
        return Ok(0);
    };

    let mut stmt = conn.prepare(
        "SELECT DISTINCT date FROM reading_sessions WHERE pages_read > 0 OR minutes_read > 0",
    )?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

    let mut active_days = HashSet::new();
    for row in rows {
        if let Some(d) = parse_iso_days(&row?) {
            active_days.insert(d);
        }
    }

    let mut cursor = if active_days.contains(&today_days) {
        today_days
    } else if active_days.contains(&(today_days - 1)) {
        today_days - 1
    } else {
        return Ok(0);
    };

    let mut streak = 0u32;
    while active_days.contains(&cursor) {
        streak += 1;
        cursor -= 1;
    }

    Ok(streak)
}

fn current_unix_days() -> i64 {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    // Ajuste aproximado a zona horaria local (-3h por defecto si no hay offset, o UTC día)
    (secs - 3 * 3600).div_euclid(86_400)
}

fn parse_iso_days(iso: &str) -> Option<i64> {
    let mut parts = iso.split('-');
    let y: i64 = parts.next()?.parse().ok()?;
    let m: u32 = parts.next()?.parse().ok()?;
    let d: u32 = parts.next()?.parse().ok()?;
    Some(days_from_civil(y, m, d))
}

/// 0 = Lunes .. 6 = Domingo (1970-01-01 fue Jueves = 3).
fn weekday_mon_zero(days_since_epoch: i64) -> usize {
    (days_since_epoch + 3).rem_euclid(7) as usize
}

/// Algoritmo de Howard Hinnant para convertir fecha civil a días desde 1970-01-01.
fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let m_adj = if m > 2 { m as i64 - 3 } else { m as i64 + 9 };
    let doy = (153 * m_adj + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Algoritmo inverso de Howard Hinnant para obtener (año, mes, día) desde días Unix.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m as u32, d as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{BookRepository, Database, ProgressRepository};
    use crate::domain::book::{Book, CoverPalette, Genre};

    fn sample_book(id: &str) -> Book {
        Book {
            id: id.into(),
            title: "Rayuela".into(),
            author: "Julio Cortázar".into(),
            total_pages: 400,
            current_page: 0,
            genre: Genre::Fiction,
            cover_palette: CoverPalette {
                primary: 0x111111,
                secondary: 0x222222,
                accent: 0x333333,
                text: 0xFFFFFF,
            },
            synopsis: String::new(),
            chapters: Vec::new(),
        }
    }

    #[test]
    fn test_stats_streak_and_weekly_activity() {
        let mut db = Database::open_in_memory().unwrap();
        let b1 = sample_book("rayuela");
        BookRepository::upsert(db.conn_mut(), &b1, None, None, "es").unwrap();

        // 2026-11-15 es Domingo (índice 6). Lunes fue 2026-11-09 (índice 0).
        ProgressRepository::update_page(db.conn(), "rayuela", 42, 2, 400, 1700000000).unwrap();
        ProgressRepository::record_session(db.conn_mut(), "rayuela", "2026-11-13", 15, 10).unwrap();
        ProgressRepository::record_session(db.conn_mut(), "rayuela", "2026-11-14", 20, 14).unwrap();
        ProgressRepository::record_session(db.conn_mut(), "rayuela", "2026-11-15", 25, 18).unwrap();

        let stats = StatsService::compute_stats(db.conn(), "2026-11-15").unwrap();
        assert_eq!(stats.current_streak_days, 3);
        assert_eq!(stats.weekly_activity[4], 10); // Viernes
        assert_eq!(stats.weekly_activity[5], 14); // Sábado
        assert_eq!(stats.weekly_activity[6], 18); // Domingo
        assert_eq!(stats.pages_read_total, 42);
        assert!((stats.reading_hours - 1.0).abs() < 0.01);

        let continue_book = StatsService::get_continue_reading_book(db.conn()).unwrap().unwrap();
        assert_eq!(continue_book.id, "rayuela");
        assert_eq!(continue_book.current_page, 42);
    }
}
