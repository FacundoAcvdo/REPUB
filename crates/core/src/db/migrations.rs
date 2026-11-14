use rusqlite::Connection;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::CoreResult;

const V1_INITIAL_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS books (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    genre TEXT NOT NULL,
    total_pages INTEGER NOT NULL DEFAULT 0,
    file_path TEXT,
    cover_path TEXT,
    cover_color TEXT NOT NULL,
    language TEXT NOT NULL DEFAULT 'es',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS reading_progress (
    book_id TEXT PRIMARY KEY REFERENCES books(id) ON DELETE CASCADE,
    current_page INTEGER NOT NULL DEFAULT 0,
    current_chapter INTEGER NOT NULL DEFAULT 0,
    percent REAL NOT NULL DEFAULT 0.0,
    last_read_at INTEGER,
    reading_time_seconds INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'Unread',
    is_favorite INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS reading_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    book_id TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    date TEXT NOT NULL,
    minutes_read INTEGER NOT NULL DEFAULT 0,
    pages_read INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_books_title ON books(title);
CREATE INDEX IF NOT EXISTS idx_books_author ON books(author);
CREATE INDEX IF NOT EXISTS idx_progress_last_read ON reading_progress(last_read_at DESC);
CREATE INDEX IF NOT EXISTS idx_progress_status ON reading_progress(status);
CREATE INDEX IF NOT EXISTS idx_sessions_date ON reading_sessions(date);
"#;

/// Ejecuta de forma idempotente todas las migraciones pendientes sobre la base de datos.
pub fn run_migrations(conn: &mut Connection) -> CoreResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at INTEGER NOT NULL
        );",
    )?;

    let migrations: &[(i64, &str)] = &[(1, V1_INITIAL_SCHEMA)];

    for &(version, sql) in migrations {
        let already_applied: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = ?1)",
            [version],
            |row| row.get(0),
        )?;

        if !already_applied {
            let tx = conn.transaction()?;
            tx.execute_batch(sql)?;
            let now = current_unix_timestamp();
            tx.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
                rusqlite::params![version, now],
            )?;
            tx.commit()?;
        }
    }

    Ok(())
}

pub(crate) fn current_unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
