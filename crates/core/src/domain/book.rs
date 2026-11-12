use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoverPalette {
    pub primary: u32,
    pub secondary: u32,
    pub accent: u32,
    pub text: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Genre {
    Fiction,
    SciFi,
    History,
    Philosophy,
    Poetry,
    Science,
    Biography,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chapter {
    pub id: String,
    pub title: String,
    pub pages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Book {
    pub id: String,
    pub title: String,
    pub author: String,
    pub total_pages: u32,
    pub current_page: u32,
    pub genre: Genre,
    pub cover_palette: CoverPalette,
    pub synopsis: String,
    pub chapters: Vec<Chapter>,
}

impl Book {
    pub fn progress_pct(&self) -> f32 {
        if self.total_pages == 0 {
            0.0
        } else {
            (self.current_page as f32 / self.total_pages as f32).clamp(0.0, 1.0)
        }
    }
}
