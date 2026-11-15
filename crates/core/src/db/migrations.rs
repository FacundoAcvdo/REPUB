use rusqlite::Connection;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::CoreResult;

const V1_TABLES: &str = r#"
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
    ai_enabled INTEGER NOT NULL DEFAULT 1,
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

CREATE TABLE IF NOT EXISTS bookmarks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    book_id TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    page_number INTEGER NOT NULL,
    chapter_index INTEGER NOT NULL DEFAULT 0,
    chapter_title TEXT NOT NULL DEFAULT '',
    note TEXT NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS book_chunks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    book_id TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    chapter_index INTEGER NOT NULL,
    chapter_title TEXT NOT NULL DEFAULT '',
    page_start INTEGER NOT NULL,
    page_end INTEGER NOT NULL,
    chunk_index INTEGER NOT NULL,
    content TEXT NOT NULL,
    embedding BLOB
);

CREATE TABLE IF NOT EXISTS book_index_state (
    book_id TEXT PRIMARY KEY REFERENCES books(id) ON DELETE CASCADE,
    embedding_source TEXT NOT NULL,
    embedding_model TEXT NOT NULL,
    total_chunks INTEGER NOT NULL DEFAULT 0,
    indexed_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS chapter_summaries (
    book_id TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    chapter_index INTEGER NOT NULL,
    chapter_title TEXT NOT NULL DEFAULT '',
    page_start INTEGER NOT NULL,
    page_end INTEGER NOT NULL,
    summary TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (book_id, chapter_index)
);

CREATE VIRTUAL TABLE IF NOT EXISTS book_chunks_fts USING fts5(
    content,
    book_id UNINDEXED,
    chunk_id UNINDEXED
);
"#;

const V1_INDEXES: &str = r#"
CREATE INDEX IF NOT EXISTS idx_books_title ON books(title);
CREATE INDEX IF NOT EXISTS idx_books_author ON books(author);
CREATE INDEX IF NOT EXISTS idx_progress_last_read ON reading_progress(last_read_at DESC);
CREATE INDEX IF NOT EXISTS idx_progress_status ON reading_progress(status);
CREATE INDEX IF NOT EXISTS idx_sessions_date ON reading_sessions(date);
CREATE INDEX IF NOT EXISTS idx_bookmarks_book ON bookmarks(book_id, page_number);
CREATE INDEX IF NOT EXISTS idx_chunks_book_page ON book_chunks(book_id, page_start);
"#;

fn has_incompatible_pk(conn: &Connection, table: &str, pk_col: &str) -> CoreResult<bool> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        let col_type: String = row.get(2)?;
        if name == pk_col && !col_type.eq_ignore_ascii_case("TEXT") {
            return Ok(true);
        }
    }
    Ok(false)
}

fn reset_incompatible_legacy_tables(conn: &Connection) -> CoreResult<()> {
    if has_incompatible_pk(conn, "books", "id")?
        || has_incompatible_pk(conn, "reading_progress", "book_id")?
    {
        conn.execute_batch(
            "DROP TABLE IF EXISTS book_chunks;
             DROP TABLE IF EXISTS book_index_state;
             DROP TABLE IF EXISTS reading_sessions;
             DROP TABLE IF EXISTS reading_progress;
             DROP TABLE IF EXISTS books;",
        )?;
        conn.execute_batch(V1_TABLES)?;
    } else if has_incompatible_pk(conn, "book_chunks", "book_id")? {
        conn.execute_batch(
            "DROP TABLE IF EXISTS book_chunks;
             DROP TABLE IF EXISTS book_index_state;",
        )?;
        conn.execute_batch(V1_TABLES)?;
    }
    Ok(())
}

fn ensure_columns(conn: &Connection, table: &str, columns: &[(&str, &str)]) -> CoreResult<()> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let existing: HashSet<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(Result::ok)
        .collect();

    for &(col_name, col_def) in columns {
        if !existing.contains(col_name) {
            conn.execute(
                &format!("ALTER TABLE {table} ADD COLUMN {col_name} {col_def}"),
                [],
            )?;
        }
    }
    Ok(())
}

fn ensure_v1_columns(conn: &Connection) -> CoreResult<()> {
    ensure_columns(
        conn,
        "books",
        &[
            ("description", "TEXT NOT NULL DEFAULT ''"),
            ("genre", "TEXT NOT NULL DEFAULT 'Otros'"),
            ("total_pages", "INTEGER NOT NULL DEFAULT 0"),
            ("file_path", "TEXT"),
            ("cover_path", "TEXT"),
            ("cover_color", "TEXT NOT NULL DEFAULT '#3B5998'"),
            ("language", "TEXT NOT NULL DEFAULT 'es'"),
            ("ai_enabled", "INTEGER NOT NULL DEFAULT 1"),
            ("created_at", "INTEGER NOT NULL DEFAULT 0"),
            ("updated_at", "INTEGER NOT NULL DEFAULT 0"),
        ],
    )?;

    ensure_columns(
        conn,
        "reading_progress",
        &[
            ("current_page", "INTEGER NOT NULL DEFAULT 0"),
            ("current_chapter", "INTEGER NOT NULL DEFAULT 0"),
            ("percent", "REAL NOT NULL DEFAULT 0.0"),
            ("last_read_at", "INTEGER"),
            ("reading_time_seconds", "INTEGER NOT NULL DEFAULT 0"),
            ("status", "TEXT NOT NULL DEFAULT 'Unread'"),
            ("is_favorite", "INTEGER NOT NULL DEFAULT 0"),
        ],
    )?;

    ensure_columns(
        conn,
        "reading_sessions",
        &[
            ("minutes_read", "INTEGER NOT NULL DEFAULT 0"),
            ("pages_read", "INTEGER NOT NULL DEFAULT 0"),
        ],
    )?;

    ensure_columns(
        conn,
        "book_chunks",
        &[
            ("chapter_index", "INTEGER NOT NULL DEFAULT 0"),
            ("chapter_title", "TEXT NOT NULL DEFAULT ''"),
            ("page_start", "INTEGER NOT NULL DEFAULT 1"),
            ("page_end", "INTEGER NOT NULL DEFAULT 1"),
            ("chunk_index", "INTEGER NOT NULL DEFAULT 0"),
            ("content", "TEXT NOT NULL DEFAULT ''"),
            ("embedding", "BLOB"),
        ],
    )?;

    Ok(())
}

/// Ejecuta de forma idempotente todas las migraciones pendientes sobre la base de datos.
pub fn run_migrations(conn: &mut Connection) -> CoreResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at INTEGER NOT NULL
        );",
    )?;

    let v1_applied: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 1)",
        [],
        |row| row.get(0),
    )?;

    if !v1_applied {
        let tx = conn.transaction()?;
        tx.execute_batch(V1_TABLES)?;
        reset_incompatible_legacy_tables(&tx)?;
        ensure_v1_columns(&tx)?;
        tx.execute_batch(V1_INDEXES)?;
        let now = current_unix_timestamp();
        tx.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (1, ?1)",
            rusqlite::params![now],
        )?;
        tx.commit()?;
    } else {
        reset_incompatible_legacy_tables(conn)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS bookmarks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                book_id TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
                page_number INTEGER NOT NULL,
                chapter_index INTEGER NOT NULL DEFAULT 0,
                chapter_title TEXT NOT NULL DEFAULT '',
                note TEXT NOT NULL DEFAULT '',
                created_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS book_chunks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                book_id TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
                chapter_index INTEGER NOT NULL,
                chapter_title TEXT NOT NULL DEFAULT '',
                page_start INTEGER NOT NULL,
                page_end INTEGER NOT NULL,
                chunk_index INTEGER NOT NULL,
                content TEXT NOT NULL,
                embedding BLOB
            );
            CREATE TABLE IF NOT EXISTS book_index_state (
                book_id TEXT PRIMARY KEY REFERENCES books(id) ON DELETE CASCADE,
                embedding_source TEXT NOT NULL,
                embedding_model TEXT NOT NULL,
                total_chunks INTEGER NOT NULL DEFAULT 0,
                indexed_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS chapter_summaries (
                book_id TEXT NOT NULL REFERENCES books(id) ON DELETE CASCADE,
                chapter_index INTEGER NOT NULL,
                chapter_title TEXT NOT NULL DEFAULT '',
                page_start INTEGER NOT NULL,
                page_end INTEGER NOT NULL,
                summary TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                PRIMARY KEY (book_id, chapter_index)
            );
            CREATE VIRTUAL TABLE IF NOT EXISTS book_chunks_fts USING fts5(
                content,
                book_id UNINDEXED,
                chunk_id UNINDEXED
            );",
        )?;
        ensure_v1_columns(conn)?;
        conn.execute_batch(V1_INDEXES)?;
    }

    Ok(())
}

pub(crate) fn current_unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
