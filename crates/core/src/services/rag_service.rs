use std::collections::{HashMap, HashSet};

use rusqlite::{params, Connection, OptionalExtension};

use crate::db::migrations::current_unix_timestamp;
use crate::domain::book::{Book, Chapter};
use crate::domain::settings::{AiSettings, EmbeddingSource};
use crate::error::{CoreError, CoreResult};
use crate::services::ai_client::AiClient;
use crate::services::local_embeddings::LocalEmbeddings;

#[derive(Debug, Clone, PartialEq)]
pub struct BookChunkDraft {
    pub chapter_index: u32,
    pub chapter_title: String,
    pub page_start: u32,
    pub page_end: u32,
    pub chunk_index: u32,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetrievedChunk {
    pub id: i64,
    pub chapter_index: u32,
    pub chapter_title: String,
    pub page_start: u32,
    pub page_end: u32,
    pub content: String,
    pub score: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexSummary {
    pub indexed_books: u32,
    pub total_chunks: u32,
    pub embedding_source: String,
    pub embedding_model: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChapterSummaryRecord {
    pub chapter_index: u32,
    pub chapter_title: String,
    pub page_start: u32,
    pub page_end: u32,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssistantAnswer {
    pub answer: String,
    pub cited_pages: Vec<u32>,
    pub used_chunks: usize,
}

pub struct RagService;

impl RagService {
    /// Divide los capítulos paginados de un libro en fragmentos contextuales con coordenadas de página.
    pub fn chunk_book(chapters: &[Chapter]) -> Vec<BookChunkDraft> {
        const TARGET_CHARS: usize = 950;
        const OVERLAP_CHARS: usize = 140;

        let mut chunks = Vec::new();
        let mut global_page: u32 = 1;
        let mut chunk_counter: u32 = 0;

        for (ch_idx, chapter) in chapters.iter().enumerate() {
            let mut buffer = String::new();
            let mut buffer_start_page = global_page;

            for page_text in &chapter.pages {
                let trimmed = page_text.trim();
                if trimmed.is_empty() {
                    global_page += 1;
                    continue;
                }

                if buffer.is_empty() {
                    buffer_start_page = global_page;
                } else {
                    buffer.push_str("\n\n");
                }
                buffer.push_str(trimmed);

                while buffer.chars().count() >= TARGET_CHARS {
                    let (head, rest) = split_chunk_boundary(&buffer, TARGET_CHARS, OVERLAP_CHARS);
                    chunks.push(BookChunkDraft {
                        chapter_index: ch_idx as u32,
                        chapter_title: chapter.title.clone(),
                        page_start: buffer_start_page,
                        page_end: global_page,
                        chunk_index: chunk_counter,
                        content: head,
                    });
                    chunk_counter += 1;
                    buffer = rest;
                    buffer_start_page = global_page;
                }

                global_page += 1;
            }

            if !buffer.trim().is_empty() {
                let end_page = global_page.saturating_sub(1).max(buffer_start_page);
                chunks.push(BookChunkDraft {
                    chapter_index: ch_idx as u32,
                    chapter_title: chapter.title.clone(),
                    page_start: buffer_start_page,
                    page_end: end_page,
                    chunk_index: chunk_counter,
                    content: buffer.trim().to_string(),
                });
                chunk_counter += 1;
            }
        }

        chunks
    }

    /// Devuelve el estado global actual del índice RAG en SQLite.
    pub fn current_index_summary(conn: &Connection) -> CoreResult<Option<IndexSummary>> {
        let indexed_books: i64 =
            conn.query_row("SELECT COUNT(*) FROM book_index_state", [], |r| r.get(0))?;
        if indexed_books == 0 {
            return Ok(None);
        }

        let total_chunks: i64 = conn.query_row(
            "SELECT COALESCE(SUM(total_chunks), 0) FROM book_index_state",
            [],
            |r| r.get(0),
        )?;

        let meta: Option<(String, String)> = conn
            .query_row(
                "SELECT embedding_source, embedding_model FROM book_index_state ORDER BY indexed_at DESC LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;

        Ok(meta.map(|(embedding_source, embedding_model)| IndexSummary {
            indexed_books: indexed_books.max(0) as u32,
            total_chunks: total_chunks.max(0) as u32,
            embedding_source,
            embedding_model,
        }))
    }

    /// Indica si un libro específico ya se encuentra indexado en SQLite.
    pub fn is_book_indexed(conn: &Connection, book_id: &str) -> CoreResult<bool> {
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM book_index_state WHERE book_id = ?1)",
            params![book_id],
            |r| r.get(0),
        )?;
        Ok(exists)
    }

    /// Elimina todos los chunks, índice FTS5 y estado de indexación en SQLite (y modelos locales en disco).
    pub fn clear_all_embeddings_and_models(conn: &mut Connection) -> CoreResult<u64> {
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM book_chunks_fts", [])?;
        tx.execute("DELETE FROM book_chunks", [])?;
        tx.execute("DELETE FROM book_index_state", [])?;
        tx.commit()?;

        LocalEmbeddings::delete_all_models()
    }

    /// Limpia únicamente el índice vectorial en SQLite (útil al reindexar con otro modelo).
    pub fn clear_index_only(conn: &mut Connection) -> CoreResult<()> {
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM book_chunks_fts", [])?;
        tx.execute("DELETE FROM book_chunks", [])?;
        tx.execute("DELETE FROM book_index_state", [])?;
        tx.commit()?;
        Ok(())
    }

    /// Indexa un libro en SQLite (chunks + FTS5 + embeddings según `AiSettings`).
    pub fn index_book(
        conn: &mut Connection,
        book_id: &str,
        chapters: &[Chapter],
        ai_settings: &AiSettings,
    ) -> CoreResult<usize> {
        let drafts = Self::chunk_book(chapters);
        if drafts.is_empty() {
            return Ok(0);
        }

        let (source_label, model_label, embeddings) =
            Self::compute_embeddings_for_drafts(&drafts, ai_settings)?;

        let tx = conn.transaction()?;
        tx.execute("DELETE FROM book_chunks_fts WHERE book_id = ?1", params![book_id])?;
        tx.execute("DELETE FROM book_chunks WHERE book_id = ?1", params![book_id])?;

        for (idx, draft) in drafts.iter().enumerate() {
            let emb_blob = embeddings
                .as_ref()
                .and_then(|vecs| vecs.get(idx))
                .map(|v| encode_f32_vec(v));

            tx.execute(
                "INSERT INTO book_chunks (
                    book_id, chapter_index, chapter_title, page_start, page_end,
                    chunk_index, content, embedding
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    book_id,
                    draft.chapter_index,
                    draft.chapter_title,
                    draft.page_start,
                    draft.page_end,
                    draft.chunk_index,
                    draft.content,
                    emb_blob,
                ],
            )?;
            let chunk_id = tx.last_insert_rowid();
            tx.execute(
                "INSERT INTO book_chunks_fts (content, book_id, chunk_id) VALUES (?1, ?2, ?3)",
                params![draft.content, book_id, chunk_id],
            )?;
        }

        let now = current_unix_timestamp();
        tx.execute(
            "INSERT INTO book_index_state (book_id, embedding_source, embedding_model, total_chunks, indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(book_id) DO UPDATE SET
                embedding_source = excluded.embedding_source,
                embedding_model = excluded.embedding_model,
                total_chunks = excluded.total_chunks,
                indexed_at = excluded.indexed_at",
            params![book_id, source_label, model_label, drafts.len() as i64, now],
        )?;
        tx.commit()?;

        Ok(drafts.len())
    }

    fn compute_embeddings_for_drafts(
        drafts: &[BookChunkDraft],
        ai_settings: &AiSettings,
    ) -> CoreResult<(String, String, Option<Vec<Vec<f32>>>)> {
        let texts: Vec<String> = drafts.iter().map(|d| d.content.clone()).collect();
        match ai_settings.embedding_source {
            EmbeddingSource::Local => {
                let model_id = ai_settings
                    .local_embedding_model
                    .as_deref()
                    .ok_or_else(|| {
                        CoreError::InvalidInput(
                            "Selecciona un modelo de embeddings local en Ajustes → Proveedores (IA)."
                                .into(),
                        )
                    })?;
                let mut all_vecs = Vec::with_capacity(texts.len());
                for batch in texts.chunks(32) {
                    let mut vecs = LocalEmbeddings::embed_texts(model_id, batch.to_vec())?;
                    all_vecs.append(&mut vecs);
                }
                Ok(("local".into(), model_id.to_string(), Some(all_vecs)))
            }
            EmbeddingSource::Cloud => {
                let mut all_vecs = Vec::with_capacity(texts.len());
                for batch in texts.chunks(32) {
                    let mut vecs = AiClient::cloud_embeddings(ai_settings, batch)?;
                    all_vecs.append(&mut vecs);
                }
                Ok((
                    "cloud".into(),
                    format!("{}-embeddings", ai_settings.provider),
                    Some(all_vecs),
                ))
            }
            EmbeddingSource::Unconfigured => Err(CoreError::InvalidInput(
                "Configura primero tu motor de embeddings (Local o Cloud) en Ajustes → Proveedores (IA)."
                    .into(),
            )),
        }
    }

    fn embed_query(query: &str, ai_settings: &AiSettings) -> Option<Vec<f32>> {
        match ai_settings.embedding_source {
            EmbeddingSource::Local => ai_settings
                .local_embedding_model
                .as_deref()
                .and_then(|m| LocalEmbeddings::embed_texts(m, vec![query.to_string()]).ok())
                .and_then(|mut v| v.pop()),
            EmbeddingSource::Cloud => {
                AiClient::cloud_embeddings(ai_settings, &[query.to_string()])
                    .ok()
                    .and_then(|mut v| v.pop())
            }
            EmbeddingSource::Unconfigured => None,
        }
    }

    /// Recuperación Híbrida + Anti-Spoilers (`page_start <= current_page`) + Recencia Narrativa.
    pub fn retrieve_context(
        conn: &Connection,
        book_id: &str,
        query: &str,
        current_page: u32,
        query_embedding: Option<&[f32]>,
        top_k: usize,
    ) -> CoreResult<Vec<RetrievedChunk>> {
        let mut stmt = conn.prepare(
            "SELECT id, chapter_index, chapter_title, page_start, page_end, content, embedding
             FROM book_chunks
             WHERE book_id = ?1 AND page_start <= ?2
             ORDER BY page_start ASC, chunk_index ASC",
        )?;

        let rows = stmt.query_map(params![book_id, current_page], |row| {
            let blob: Option<Vec<u8>> = row.get("embedding")?;
            Ok((
                row.get::<_, i64>("id")?,
                row.get::<_, i64>("chapter_index")?.max(0) as u32,
                row.get::<_, String>("chapter_title")?,
                row.get::<_, i64>("page_start")?.max(1) as u32,
                row.get::<_, i64>("page_end")?.max(1) as u32,
                row.get::<_, String>("content")?,
                blob.map(|b| decode_f32_vec(&b)),
            ))
        })?;

        let mut candidates = Vec::new();
        for r in rows {
            candidates.push(r?);
        }
        if candidates.is_empty() {
            return Ok(Vec::new());
        }

        // Rama léxica (FTS5)
        let fts_scores = query_fts_scores(conn, book_id, query)?;
        let query_terms = tokenize_terms(query);
        let panoramic = is_panoramic_query(query);
        let recency_weight = if panoramic { 0.05 } else { 0.35 };
        let relevance_weight = 1.0 - recency_weight;

        let mut scored: Vec<RetrievedChunk> = candidates
            .iter()
            .map(
                |(id, ch_idx, ch_title, p_start, p_end, content, emb_opt)| {
                    let semantic = match (query_embedding, emb_opt.as_deref()) {
                        (Some(q), Some(c)) if q.len() == c.len() && !q.is_empty() => {
                            cosine_similarity(q, c).max(0.0)
                        }
                        _ => 0.0,
                    };

                    let fts_boost = fts_scores.get(id).copied().unwrap_or(0.0);
                    let overlap = lexical_overlap(&query_terms, content);
                    let relevance = if query_embedding.is_some() {
                        (0.65 * semantic + 0.25 * fts_boost + 0.10 * overlap).min(1.0)
                    } else {
                        (0.60 * fts_boost + 0.40 * overlap).min(1.0)
                    };

                    let recency = (*p_start as f32 / current_page.max(1) as f32).clamp(0.0, 1.0);
                    let final_score = relevance_weight * relevance + recency_weight * recency;

                    RetrievedChunk {
                        id: *id,
                        chapter_index: *ch_idx,
                        chapter_title: ch_title.clone(),
                        page_start: *p_start,
                        page_end: *p_end,
                        content: content.clone(),
                        score: final_score,
                    }
                },
            )
            .collect();

        // Ordenar por score descendente
        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut selected: Vec<RetrievedChunk> = if panoramic {
            select_stratified_by_chapter(&scored, top_k)
        } else {
            scored.into_iter().take(top_k).collect()
        };
        let mut seen_ids: HashSet<i64> = selected.iter().map(|c| c.id).collect();

        // Ventana de Contexto Inmediato: incluir últimos 2 chunks previos a current_page
        for (id, ch_idx, ch_title, p_start, p_end, content, _) in candidates.iter().rev().take(2) {
            if !seen_ids.contains(id) {
                seen_ids.insert(*id);
                selected.push(RetrievedChunk {
                    id: *id,
                    chapter_index: *ch_idx,
                    chapter_title: ch_title.clone(),
                    page_start: *p_start,
                    page_end: *p_end,
                    content: content.clone(),
                    score: 1.0,
                });
            }
        }

        // Orden cronológico ascendente para el LLM (fundamental para "qué fue lo último")
        selected.sort_by_key(|c| (c.page_start, c.chapter_index, c.id));
        Ok(selected)
    }

    /// Ejecuta la consulta completa: asegura indexación, recupera contexto anti-spoilers y consulta al LLM.
    pub fn ask_assistant(
        conn: &mut Connection,
        book: &Book,
        current_page: u32,
        selected_text: Option<&str>,
        question: &str,
        ai_settings: &AiSettings,
    ) -> CoreResult<AssistantAnswer> {
        if ai_settings.embedding_source == EmbeddingSource::Unconfigured {
            return Err(CoreError::InvalidInput(
                "Primero configura un proveedor de IA y un motor de embeddings en Ajustes → Proveedores (IA)."
                    .into(),
            ));
        }

        if !Self::is_book_indexed(conn, &book.id)? {
            Self::index_book(conn, &book.id, &book.chapters, ai_settings)?;
        }

        let panoramic = is_panoramic_query(question);
        let top_k = if panoramic { 18 } else { 8 };

        if panoramic {
            let _ = Self::ensure_completed_chapter_summaries(
                conn,
                &book.id,
                current_page,
                ai_settings,
            );
        }

        let chapter_summaries =
            Self::list_chapter_summaries_until(conn, &book.id, current_page).unwrap_or_default();

        let retrieval_query = match selected_text {
            Some(sel) if !sel.trim().is_empty() => format!("{question} {sel}"),
            _ => question.to_string(),
        };
        let q_emb = Self::embed_query(&retrieval_query, ai_settings);
        let chunks = Self::retrieve_context(
            conn,
            &book.id,
            &retrieval_query,
            current_page,
            q_emb.as_deref(),
            top_k,
        )?;

        let mut context_block = String::new();
        if !chapter_summaries.is_empty() {
            context_block.push_str("=== RESUMEN CRONOLÓGICO DE CAPÍTULOS COMPLETADOS ===\n");
            for cs in &chapter_summaries {
                let label = if cs.chapter_title.trim().is_empty() {
                    format!("Capítulo {}", cs.chapter_index + 1)
                } else {
                    cs.chapter_title.clone()
                };
                context_block.push_str(&format!(
                    "- [{label}, Págs. {}-{}]: {}\n",
                    cs.page_start, cs.page_end, cs.summary
                ));
            }
            context_block.push_str("\n=== FRAGMENTOS DETALLADOS DEL LIBRO ===\n");
        }

        let mut cited_pages = Vec::new();
        for (i, ch) in chunks.iter().enumerate() {
            let is_latest = i + 1 == chunks.len();
            let ch_label = if ch.chapter_title.trim().is_empty() {
                format!("Sección {}", ch.chapter_index + 1)
            } else {
                ch.chapter_title.clone()
            };
            let recency_tag = if is_latest {
                " — LECTURA MÁS RECIENTE"
            } else {
                ""
            };
            context_block.push_str(&format!(
                "[{ch_label}, Págs. {}-{}{recency_tag}]\n{}\n\n",
                ch.page_start, ch.page_end, ch.content
            ));
            if !cited_pages.contains(&ch.page_start) {
                cited_pages.push(ch.page_start);
            }
        }

        let system_prompt = format!(
            "Eres un asistente literario inteligente integrado en un lector de EPUB.\n\
             Libro: \"{}\" de {}.\n\
             Progreso actual del lector: página {} de {}.\n\
             REGLAS ESTRICTAS:\n\
             1. ANTI-SPOILERS: Responde ÚNICAMENTE con información presente en los fragmentos proporcionados (que corresponden solo hasta la página {}). Jamás reveles ni insinúes eventos futuros.\n\
             2. CRONOLOGÍA NARRATIVA: Los fragmentos están ordenados cronológicamente de lo más antiguo a lo más reciente. Si el usuario pregunta 'qué fue lo último que se sabe de...' o el estado actual, prioriza los fragmentos más cercanos al final (etiquetados como más recientes).\n\
             3. HONESTIDAD: Si la respuesta no aparece en lo leído hasta ahora, dilo claramente sin inventar hechos.\n\
             4. Responde en español, de forma clara, concisa y citando entre corchetes las páginas relevantes cuando sea útil (ej. [Pág. 42]).",
            book.title, book.author, current_page, book.total_pages, current_page
        );

        let user_prompt = match selected_text {
            Some(sel) if !sel.trim().is_empty() => format!(
                "FRAGMENTOS DEL LIBRO HASTA LA PÁGINA {current_page}:\n{context_block}\n\
                 PASAJE SELECCIONADO POR EL LECTOR:\n\"{sel}\"\n\n\
                 PREGUNTA DEL LECTOR:\n{question}"
            ),
            _ => format!(
                "FRAGMENTOS DEL LIBRO HASTA LA PÁGINA {current_page}:\n{context_block}\n\
                 PREGUNTA DEL LECTOR:\n{question}"
            ),
        };

        let answer = AiClient::chat_completion(ai_settings, &system_prompt, &user_prompt)?;
        Ok(AssistantAnswer {
            answer,
            cited_pages,
            used_chunks: chunks.len(),
        })
    }

    /// Devuelve los resúmenes cacheados de capítulos completamente leídos (`page_end <= current_page`).
    pub fn list_chapter_summaries_until(
        conn: &Connection,
        book_id: &str,
        current_page: u32,
    ) -> CoreResult<Vec<ChapterSummaryRecord>> {
        let mut stmt = conn.prepare(
            "SELECT chapter_index, chapter_title, page_start, page_end, summary
             FROM chapter_summaries
             WHERE book_id = ?1 AND page_end <= ?2
             ORDER BY chapter_index ASC",
        )?;
        let rows = stmt.query_map(params![book_id, current_page], |row| {
            Ok(ChapterSummaryRecord {
                chapter_index: row.get::<_, i64>(0)?.max(0) as u32,
                chapter_title: row.get(1)?,
                page_start: row.get::<_, i64>(2)?.max(1) as u32,
                page_end: row.get::<_, i64>(3)?.max(1) as u32,
                summary: row.get(4)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    /// Genera bajo demanda y guarda en SQLite resúmenes de capítulos ya completados (`page_end <= current_page`).
    pub fn ensure_completed_chapter_summaries(
        conn: &Connection,
        book_id: &str,
        current_page: u32,
        ai_settings: &AiSettings,
    ) -> CoreResult<usize> {
        let mut stmt = conn.prepare(
            "SELECT chapter_index, MAX(chapter_title), MIN(page_start), MAX(page_end)
             FROM book_chunks
             WHERE book_id = ?1
             GROUP BY chapter_index
             HAVING MAX(page_end) <= ?2
             ORDER BY chapter_index ASC",
        )?;
        let completed: Vec<(u32, String, u32, u32)> = stmt
            .query_map(params![book_id, current_page], |row| {
                Ok((
                    row.get::<_, i64>(0)?.max(0) as u32,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?.max(1) as u32,
                    row.get::<_, i64>(3)?.max(1) as u32,
                ))
            })?
            .filter_map(Result::ok)
            .collect();

        let existing: HashSet<u32> = Self::list_chapter_summaries_until(conn, book_id, current_page)?
            .into_iter()
            .map(|c| c.chapter_index)
            .collect();

        let mut generated = 0usize;
        for (ch_idx, ch_title, p_start, p_end) in completed {
            if existing.contains(&ch_idx) {
                continue;
            }
            let mut ch_stmt = conn.prepare(
                "SELECT content FROM book_chunks
                 WHERE book_id = ?1 AND chapter_index = ?2
                 ORDER BY chunk_index ASC",
            )?;
            let parts: Vec<String> = ch_stmt
                .query_map(params![book_id, ch_idx], |row| row.get::<_, String>(0))?
                .filter_map(Result::ok)
                .collect();
            if parts.is_empty() {
                continue;
            }
            let mut joined = parts.join("\n");
            if joined.chars().count() > 12_000 {
                joined = joined.chars().take(12_000).collect();
            }

            let sys = "Eres un asistente literario. Resume en 3 a 5 oraciones claras en español los eventos principales, personajes y lugares de este capítulo, sin inventar nada.";
            let label = if ch_title.trim().is_empty() {
                format!("Capítulo {}", ch_idx + 1)
            } else {
                ch_title.clone()
            };
            let usr = format!("TEXTO DEL {label} (Págs. {p_start}-{p_end}):\n{joined}");

            if let Ok(summary) = AiClient::chat_completion(ai_settings, sys, &usr) {
                let now = current_unix_timestamp();
                conn.execute(
                    "INSERT INTO chapter_summaries (
                        book_id, chapter_index, chapter_title, page_start, page_end, summary, created_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                     ON CONFLICT(book_id, chapter_index) DO UPDATE SET
                        chapter_title = excluded.chapter_title,
                        page_start = excluded.page_start,
                        page_end = excluded.page_end,
                        summary = excluded.summary,
                        created_at = excluded.created_at",
                    params![book_id, ch_idx, ch_title, p_start, p_end, summary, now],
                )?;
                generated += 1;
            }
        }
        Ok(generated)
    }
}

fn is_panoramic_query(query: &str) -> bool {
    let q = query.to_lowercase();
    const PATTERNS: &[&str] = &[
        "desde que",
        "eventos",
        "sucesos",
        "qué pasó",
        "que paso",
        "qué ocurrió",
        "que ocurrio",
        "qué ha pasado",
        "que ha pasado",
        "resumen",
        "resumir",
        "resume",
        "historia",
        "hasta ahora",
        "cronología",
        "cronologia",
        "todo lo que",
        "capítulos anteriores",
        "capitulos anteriores",
        "antes en",
    ];
    PATTERNS.iter().any(|p| q.contains(p))
}

fn select_stratified_by_chapter(scored: &[RetrievedChunk], top_k: usize) -> Vec<RetrievedChunk> {
    if scored.len() <= top_k {
        return scored.to_vec();
    }
    let mut by_chapter: HashMap<u32, Vec<&RetrievedChunk>> = HashMap::new();
    for ch in scored {
        by_chapter.entry(ch.chapter_index).or_default().push(ch);
    }
    let mut chapter_keys: Vec<u32> = by_chapter.keys().copied().collect();
    chapter_keys.sort_unstable();

    let mut selected = Vec::with_capacity(top_k);
    let mut seen_ids = HashSet::new();

    // Ronda 1 y 2: tomar hasta los 2 mejores chunks de cada capítulo para cubrir toda la línea temporal
    for round in 0..2 {
        for &ch_idx in &chapter_keys {
            if selected.len() >= top_k {
                break;
            }
            if let Some(list) = by_chapter.get(&ch_idx) {
                if let Some(&chunk) = list.get(round) {
                    if seen_ids.insert(chunk.id) {
                        selected.push(chunk.clone());
                    }
                }
            }
        }
    }

    // Rellenar los lugares restantes con los chunks de mayor score global
    for ch in scored {
        if selected.len() >= top_k {
            break;
        }
        if seen_ids.insert(ch.id) {
            selected.push(ch.clone());
        }
    }

    selected
}

fn split_chunk_boundary(text: &str, target: usize, overlap: usize) -> (String, String) {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= target {
        return (text.to_string(), String::new());
    }

    let mut cut = target;
    while cut > target / 2 && !chars[cut].is_whitespace() {
        cut -= 1;
    }
    if cut <= target / 2 {
        cut = target;
    }

    let head: String = chars[..cut].iter().collect();
    let overlap_start = cut.saturating_sub(overlap);
    let rest: String = chars[overlap_start..].iter().collect();
    (head.trim().to_string(), rest.trim().to_string())
}

fn encode_f32_vec(vec: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vec.len() * 4);
    for &f in vec {
        bytes.extend_from_slice(&f.to_le_bytes());
    }
    bytes
}

fn decode_f32_vec(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    for (&x, &y) in a.iter().zip(b.iter()) {
        dot += x * y;
        na += x * x;
        nb += y * y;
    }
    let denom = na.sqrt() * nb.sqrt();
    if denom == 0.0 {
        0.0
    } else {
        dot / denom
    }
}

fn tokenize_terms(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .map(|w| w.trim().to_lowercase())
        .filter(|w| w.chars().count() >= 3)
        .collect()
}

fn lexical_overlap(terms: &[String], content: &str) -> f32 {
    if terms.is_empty() {
        return 0.0;
    }
    let lower = content.to_lowercase();
    let hits = terms.iter().filter(|t| lower.contains(t.as_str())).count();
    (hits as f32 / terms.len() as f32).clamp(0.0, 1.0)
}

fn query_fts_scores(
    conn: &Connection,
    book_id: &str,
    query: &str,
) -> CoreResult<HashMap<i64, f32>> {
    let terms = tokenize_terms(query);
    if terms.is_empty() {
        return Ok(HashMap::new());
    }
    let fts_query = terms.join(" OR ");
    let mut map = HashMap::new();

    if let Ok(mut stmt) = conn.prepare(
        "SELECT chunk_id, bm25(book_chunks_fts) AS rank
         FROM book_chunks_fts
         WHERE book_chunks_fts MATCH ?1 AND book_id = ?2
         ORDER BY rank ASC
         LIMIT 25",
    ) {
        if let Ok(rows) = stmt.query_map(params![fts_query, book_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?))
        }) {
            for (idx, r) in rows.flatten().enumerate() {
                let score = 1.0 / (1.0 + idx as f32 * 0.15);
                map.insert(r.0, score);
            }
        }
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{BookRepository, Database};
    use crate::domain::book::{CoverPalette, Genre};

    fn sample_book_with_chapters() -> Book {
        Book {
            id: "cien-anos".into(),
            title: "Cien años de soledad".into(),
            author: "Gabriel García Márquez".into(),
            total_pages: 6,
            current_page: 4,
            genre: Genre::Fiction,
            cover_palette: CoverPalette {
                primary: 0,
                secondary: 0,
                accent: 0,
                text: 0,
            },
            synopsis: String::new(),
            chapters: vec![
                Chapter {
                    id: "c1".into(),
                    title: "Capítulo 1".into(),
                    pages: vec![
                        "El coronel Aureliano Buendía recordaba aquella tarde remota en que su padre lo llevó a conocer el hielo en Macondo.".into(),
                        "Macondo era entonces una aldea de veinte casas de barro y cañabrava.".into(),
                    ],
                },
                Chapter {
                    id: "c2".into(),
                    title: "Capítulo 2".into(),
                    pages: vec![
                        "Muchos meses después, el coronel Aureliano Buendía partió hacia la guerra civil con veintiún hombres.".into(),
                        "Úrsula esperaba noticias en la casa grande de Macondo.".into(),
                    ],
                },
                Chapter {
                    id: "c3".into(),
                    title: "Capítulo 3 (Futuro)".into(),
                    pages: vec![
                        "SPOILER FUTURO: El coronel Aureliano Buendía regresó años después a fabricar pescaditos de oro hasta el final.".into(),
                        "SPOILER FINAL: Toda la estirpe desapareció en el viento.".into(),
                    ],
                },
            ],
        }
    }

    #[test]
    fn test_rag_anti_spoiler_excludes_future_pages() {
        let mut db = Database::open_in_memory().unwrap();
        let book = sample_book_with_chapters();
        BookRepository::upsert(db.conn_mut(), &book, None, None, "es").unwrap();

        // Insertar chunks directamente (sin red) para probar recuperación anti-spoilers
        let drafts = RagService::chunk_book(&book.chapters);
        for d in &drafts {
            db.conn()
                .execute(
                    "INSERT INTO book_chunks (book_id, chapter_index, chapter_title, page_start, page_end, chunk_index, content, embedding)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL)",
                    params![
                        book.id,
                        d.chapter_index,
                        d.chapter_title,
                        d.page_start,
                        d.page_end,
                        d.chunk_index,
                        d.content,
                    ],
                )
                .unwrap();
        }

        // Lector en página 4 (antes del Capítulo 3 que empieza en pág. 5)
        let retrieved = RagService::retrieve_context(
            db.conn(),
            &book.id,
            "coronel Aureliano Buendía pescaditos de oro",
            4,
            None,
            5,
        )
        .unwrap();

        assert!(!retrieved.is_empty());
        assert!(retrieved.iter().all(|c| c.page_start <= 4));
        assert!(!retrieved.iter().any(|c| c.content.contains("SPOILER")));
    }

    #[test]
    fn test_rag_narrative_recency_orders_chronologically() {
        let mut db = Database::open_in_memory().unwrap();
        let book = sample_book_with_chapters();
        BookRepository::upsert(db.conn_mut(), &book, None, None, "es").unwrap();

        let drafts = RagService::chunk_book(&book.chapters);
        for d in &drafts {
            db.conn()
                .execute(
                    "INSERT INTO book_chunks (book_id, chapter_index, chapter_title, page_start, page_end, chunk_index, content, embedding)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL)",
                    params![
                        book.id,
                        d.chapter_index,
                        d.chapter_title,
                        d.page_start,
                        d.page_end,
                        d.chunk_index,
                        d.content,
                    ],
                )
                .unwrap();
        }

        let retrieved = RagService::retrieve_context(
            db.conn(),
            &book.id,
            "coronel Aureliano Buendía",
            4,
            None,
            4,
        )
        .unwrap();

        // Debe estar ordenado cronológicamente ascendente
        for pair in retrieved.windows(2) {
            assert!(pair[0].page_start <= pair[1].page_start);
        }
        // El último chunk debe ser el más reciente hasta la página 4 (capítulo 2)
        assert!(retrieved.last().unwrap().page_start >= 3);
    }

    #[test]
    fn test_book_ai_toggle_persists() {
        let mut db = Database::open_in_memory().unwrap();
        let book = sample_book_with_chapters();
        BookRepository::upsert(db.conn_mut(), &book, None, None, "es").unwrap();

        assert!(BookRepository::is_ai_enabled(db.conn(), &book.id).unwrap());
        BookRepository::set_ai_enabled(db.conn(), &book.id, false).unwrap();
        assert!(!BookRepository::is_ai_enabled(db.conn(), &book.id).unwrap());

        let stored = BookRepository::get_by_id(db.conn(), &book.id).unwrap().unwrap();
        assert!(!stored.ai_enabled);
    }

    #[test]
    fn test_panoramic_query_samples_across_chapters() {
        let mut db = Database::open_in_memory().unwrap();
        let mut book = sample_book_with_chapters();
        book.chapters = (0..5)
            .map(|i| Chapter {
                id: format!("ch-{}", i + 1),
                title: format!("Capítulo {}", i + 1),
                pages: vec![format!(
                    "Contenido extenso del capítulo {} con sucesos en el pabellón {}.",
                    i + 1,
                    i + 1
                )],
            })
            .collect();
        book.total_pages = 10;
        BookRepository::upsert(db.conn_mut(), &book, None, None, "es").unwrap();

        for (i, ch) in book.chapters.iter().enumerate() {
            let p_start = (i * 2 + 1) as u32;
            db.conn()
                .execute(
                    "INSERT INTO book_chunks (book_id, chapter_index, chapter_title, page_start, page_end, chunk_index, content)
                     VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6)",
                    params![book.id, i as u32, ch.title, p_start, p_start + 1, ch.pages[0]],
                )
                .unwrap();
        }

        let retrieved = RagService::retrieve_context(
            db.conn(),
            &book.id,
            "¿Qué eventos importantes sucedieron desde que entraron al pabellón?",
            10,
            None,
            10,
        )
        .unwrap();

        let chapters_covered: HashSet<u32> = retrieved.iter().map(|c| c.chapter_index).collect();
        assert!(chapters_covered.contains(&0));
        assert!(chapters_covered.contains(&1));
        assert!(chapters_covered.contains(&2));
        assert!(chapters_covered.contains(&4));
    }

    #[test]
    fn test_chapter_summaries_respect_anti_spoiler_page_end() {
        let mut db = Database::open_in_memory().unwrap();
        let book = sample_book_with_chapters();
        BookRepository::upsert(db.conn_mut(), &book, None, None, "es").unwrap();

        db.conn()
            .execute(
                "INSERT INTO chapter_summaries (book_id, chapter_index, chapter_title, page_start, page_end, summary, created_at)
                 VALUES (?1, 0, 'Cap 1', 1, 10, 'Resumen cap 1', 100),
                        (?1, 1, 'Cap 2', 11, 25, 'Resumen cap 2 futuro', 100)",
                params![book.id],
            )
            .unwrap();

        let visible = RagService::list_chapter_summaries_until(db.conn(), &book.id, 15).unwrap();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].chapter_index, 0);
        assert_eq!(visible[0].summary, "Resumen cap 1");
    }
}
