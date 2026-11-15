use std::io::{Read, Seek};
use std::path::Path;

use epub::doc::{EpubDoc, NavPoint};

use crate::domain::book::Genre;
use crate::error::{CoreError, CoreResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMetadata {
    pub title: String,
    pub author: String,
    pub description: String,
    pub language: String,
    pub genre: Genre,
    pub identifier: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TocEntry {
    pub label: String,
    pub chapter_index: Option<usize>,
    pub children: Vec<TocEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedChapter {
    pub index: usize,
    pub id: String,
    pub title: String,
    pub raw_xhtml: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEpub {
    pub metadata: ParsedMetadata,
    /// Bytes de la imagen de portada y extensión normalizada (`"jpg"`, `"png"`, etc.).
    pub cover: Option<(Vec<u8>, String)>,
    pub toc: Vec<TocEntry>,
    pub chapters: Vec<ParsedChapter>,
}

pub struct EpubParser;

impl EpubParser {
    /// Abre y parsea un archivo `.epub` desde disco.
    pub fn parse_file(path: impl AsRef<Path>) -> CoreResult<ParsedEpub> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(CoreError::NotFound(format!(
                "No se encontró el archivo EPUB: {}",
                path.display()
            )));
        }
        let mut doc = EpubDoc::new(path)?;
        Self::parse_doc(&mut doc)
    }

    /// Extrae metadatos, portada, TOC y capítulos de un `EpubDoc` ya abierto.
    pub fn parse_doc<R: Read + Seek>(doc: &mut EpubDoc<R>) -> CoreResult<ParsedEpub> {
        let title = doc
            .get_title()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| "Sin título".to_string());

        let author = doc
            .mdata("creator")
            .map(|m| m.value.trim().to_string())
            .filter(|a| !a.is_empty())
            .unwrap_or_else(|| "Autor desconocido".to_string());

        let description = doc
            .mdata("description")
            .map(|m| m.value.trim().to_string())
            .unwrap_or_default();

        let language = doc
            .mdata("language")
            .map(|m| m.value.trim().to_string())
            .filter(|l| !l.is_empty())
            .unwrap_or_else(|| "es".to_string());

        let subject = doc
            .mdata("subject")
            .map(|m| m.value.as_str())
            .unwrap_or("");
        let genre = map_subject_to_genre(subject);

        let identifier = doc.get_release_identifier().or_else(|| {
            doc.mdata("identifier").map(|m| m.value.trim().to_string())
        });

        let cover = doc.get_cover().map(|(bytes, mime)| {
            let ext = mime_to_extension(&mime).to_string();
            (bytes, ext)
        });

        let toc = doc
            .toc
            .iter()
            .map(|nav| convert_nav_point(doc, nav))
            .collect::<Vec<_>>();

        let num_chapters = doc.get_num_chapters();
        let mut chapters = Vec::with_capacity(num_chapters);

        for idx in 0..num_chapters {
            if !doc.set_current_chapter(idx) {
                continue;
            }
            let id = doc
                .get_current_id()
                .unwrap_or_else(|| format!("chapter-{idx}"));
            let raw_xhtml = doc
                .get_current_str()
                .map(|(content, _mime)| content)
                .unwrap_or_default();

            let chapter_title = find_toc_title(&toc, idx).unwrap_or_default();

            chapters.push(ParsedChapter {
                index: idx,
                id,
                title: chapter_title,
                raw_xhtml,
            });
        }

        Ok(ParsedEpub {
            metadata: ParsedMetadata {
                title,
                author,
                description,
                language,
                genre,
                identifier,
            },
            cover,
            toc,
            chapters,
        })
    }
}

fn convert_nav_point<R: Read + Seek>(doc: &EpubDoc<R>, nav: &NavPoint) -> TocEntry {
    let chapter_index = doc.resource_uri_to_chapter(&nav.content);
    let children = nav
        .children
        .iter()
        .map(|child| convert_nav_point(doc, child))
        .collect();

    TocEntry {
        label: nav.label.trim().to_string(),
        chapter_index,
        children,
    }
}

fn find_toc_title(toc: &[TocEntry], target_chapter: usize) -> Option<String> {
    for entry in toc {
        if entry.chapter_index == Some(target_chapter) && !entry.label.is_empty() {
            return Some(entry.label.clone());
        }
        if let Some(found) = find_toc_title(&entry.children, target_chapter) {
            return Some(found);
        }
    }
    None
}

fn mime_to_extension(mime: &str) -> &'static str {
    let lower = mime.to_lowercase();
    if lower.contains("png") {
        "png"
    } else if lower.contains("webp") {
        "webp"
    } else if lower.contains("gif") {
        "gif"
    } else if lower.contains("svg") {
        "svg"
    } else {
        "jpg"
    }
}

fn map_subject_to_genre(subject: &str) -> Genre {
    let s = subject.to_lowercase();
    if s.contains("sci-fi") || s.contains("science fiction") || s.contains("ciencia ficción") {
        Genre::SciFi
    } else if s.contains("fantasy") || s.contains("fantasía") {
        Genre::Fantasy
    } else if s.contains("mystery") || s.contains("thriller") || s.contains("misterio") {
        Genre::Mystery
    } else if s.contains("history") || s.contains("historia") {
        Genre::History
    } else if s.contains("philosophy") || s.contains("filosofía") {
        Genre::Philosophy
    } else if s.contains("poetry") || s.contains("poesía") {
        Genre::Poetry
    } else if s.contains("science") || s.contains("ciencia") {
        Genre::Science
    } else if s.contains("biography") || s.contains("biografía") {
        Genre::Biography
    } else {
        Genre::Fiction
    }
}
