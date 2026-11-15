use crate::domain::book::Chapter;
use crate::epub::parser::ParsedChapter;
use crate::epub::xhtml::extract_paragraphs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaginationConfig {
    pub chars_per_page: usize,
}

impl Default for PaginationConfig {
    fn default() -> Self {
        Self {
            chars_per_page: 2400,
        }
    }
}

impl PaginationConfig {
    /// Calcula la capacidad aproximada de caracteres por página según la tipografía y ancho de hoja.
    pub fn from_reader_settings(font_size: u32, line_height: f32, max_line_width: u32) -> Self {
        let base_font = font_size.clamp(8, 36) as f32;
        let base_lh = line_height.clamp(1.0, 2.5);
        let base_width = max_line_width.clamp(360, 1200) as f32;

        // Referencia estándar: 18px, 1.6 interlineado, 680px ancho ≈ 2400 caracteres
        let font_factor = 18.0 / base_font;
        let lh_factor = 1.6 / base_lh;
        let width_factor = base_width / 680.0;

        let estimated = (2400.0 * font_factor * lh_factor * width_factor).round() as usize;
        Self {
            chars_per_page: estimated.clamp(600, 5200),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPage {
    pub chapter_number: u32,
    pub chapter_title: String,
    pub paragraphs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaginatedBook {
    pub chapters: Vec<Chapter>,
    pub total_pages: u32,
    /// Offset 0-based en páginas globales donde inicia cada capítulo.
    chapter_offsets: Vec<u32>,
}

impl PaginatedBook {
    /// Construye un índice paginado a partir de capítulos ya divididos en páginas.
    pub fn from_chapters(chapters: Vec<Chapter>) -> Self {
        let mut chapter_offsets = Vec::with_capacity(chapters.len());
        let mut total_pages: u32 = 0;

        for ch in &chapters {
            chapter_offsets.push(total_pages);
            total_pages += ch.pages.len() as u32;
        }

        Self {
            chapters,
            total_pages,
            chapter_offsets,
        }
    }

    /// Resuelve una página global (1-based) a su capítulo correspondiente y lista de párrafos.
    pub fn resolve_page(&self, global_page: u32) -> Option<ResolvedPage> {
        if self.total_pages == 0 {
            return None;
        }
        let target = global_page.clamp(1, self.total_pages) - 1;

        for (idx, chapter) in self.chapters.iter().enumerate() {
            let start = self.chapter_offsets[idx];
            let len = chapter.pages.len() as u32;
            if target >= start && target < start + len {
                let local_idx = (target - start) as usize;
                let page_text = &chapter.pages[local_idx];
                let paragraphs = page_text
                    .split("\n\n")
                    .map(|p| p.trim().to_string())
                    .filter(|p| !p.is_empty())
                    .collect::<Vec<_>>();

                return Some(ResolvedPage {
                    chapter_number: (idx + 1) as u32,
                    chapter_title: chapter.title.clone(),
                    paragraphs,
                });
            }
        }

        None
    }

    /// Devuelve el índice 0-based del capítulo al que pertenece la página global (1-based).
    pub fn chapter_index_for_page(&self, global_page: u32) -> Option<usize> {
        if self.total_pages == 0 {
            return None;
        }
        let target = global_page.clamp(1, self.total_pages) - 1;
        for (idx, chapter) in self.chapters.iter().enumerate() {
            let start = self.chapter_offsets[idx];
            let len = chapter.pages.len() as u32;
            if target >= start && target < start + len {
                return Some(idx);
            }
        }
        None
    }

    /// Devuelve la página global inicial (1-based) de un capítulo dado su índice 0-based.
    pub fn chapter_start_page(&self, chapter_idx: usize) -> Option<u32> {
        self.chapter_offsets.get(chapter_idx).map(|off| off + 1)
    }
}

pub struct Paginator;

impl Paginator {
    /// Convierte capítulos XHTML parseados en capítulos paginados según `config`.
    pub fn paginate(parsed_chapters: &[ParsedChapter], config: PaginationConfig) -> PaginatedBook {
        let max_chars = config.chars_per_page.max(200);
        let mut chapters = Vec::new();

        for ch in parsed_chapters {
            let mut raw_paragraphs = extract_paragraphs(&ch.raw_xhtml);
            let title_trimmed = ch.title.trim();
            if !title_trimmed.is_empty() && raw_paragraphs.len() > 1 {
                if raw_paragraphs[0].trim().eq_ignore_ascii_case(title_trimmed) {
                    raw_paragraphs.remove(0);
                }
            }
            let pages = paginate_paragraphs(&raw_paragraphs, max_chars);
            if pages.is_empty() {
                continue;
            }
            chapters.push(Chapter {
                id: ch.id.clone(),
                title: ch.title.clone(),
                pages,
            });
        }

        if chapters.is_empty() {
            chapters.push(Chapter {
                id: "empty".into(),
                title: "Sin contenido".into(),
                pages: vec!["Este documento no contiene texto legible.".into()],
            });
        }

        PaginatedBook::from_chapters(chapters)
    }
}

fn paginate_paragraphs(paragraphs: &[String], max_chars: usize) -> Vec<String> {
    const MIN_SPLIT_REMAINING: usize = 80;
    let mut pages = Vec::new();
    let mut current_page = String::new();

    for paragraph in paragraphs {
        let mut remaining_text = paragraph.trim().to_string();
        if remaining_text.is_empty() {
            continue;
        }

        if !current_page.is_empty() {
            let used = current_page.chars().count();
            let para_len = remaining_text.chars().count();

            if used + 2 + para_len <= max_chars {
                current_page.push_str("\n\n");
                current_page.push_str(&remaining_text);
                continue;
            }

            let available = max_chars.saturating_sub(used + 2);
            if available >= MIN_SPLIT_REMAINING {
                let (head, tail) = split_at_word_limit(&remaining_text, available);
                if !head.is_empty() {
                    current_page.push_str("\n\n");
                    current_page.push_str(&head);
                    pages.push(std::mem::take(&mut current_page));
                    remaining_text = tail;
                } else {
                    pages.push(std::mem::take(&mut current_page));
                }
            } else {
                pages.push(std::mem::take(&mut current_page));
            }
        }

        while !remaining_text.is_empty() {
            if remaining_text.chars().count() <= max_chars {
                current_page = remaining_text;
                break;
            }
            let (head, tail) = split_at_word_limit(&remaining_text, max_chars);
            pages.push(head);
            remaining_text = tail;
        }
    }

    if !current_page.is_empty() {
        pages.push(current_page);
    }

    pages
}

fn split_at_word_limit(text: &str, max_chars: usize) -> (String, String) {
    let mut head = String::new();
    let mut tail = String::new();
    let mut overflowed = false;

    for word in text.split_whitespace() {
        if overflowed {
            if !tail.is_empty() {
                tail.push(' ');
            }
            tail.push_str(word);
            continue;
        }

        let sep = if head.is_empty() { 0 } else { 1 };
        let word_len = word.chars().count();
        let head_len = head.chars().count();

        if head_len + sep + word_len <= max_chars {
            if !head.is_empty() {
                head.push(' ');
            }
            head.push_str(word);
        } else if head.is_empty() {
            // Palabra única más larga que `max_chars`: cortar por caracteres para evitar bucle.
            let taken: String = word.chars().take(max_chars).collect();
            let rest: String = word.chars().skip(max_chars).collect();
            head = taken;
            tail = rest;
            overflowed = true;
        } else {
            overflowed = true;
            tail.push_str(word);
        }
    }

    (head, tail)
}
