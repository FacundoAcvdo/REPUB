use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverPalette {
    pub primary: u32,
    pub secondary: u32,
    pub accent: u32,
    pub text: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Genre {
    Fiction,
    SciFi,
    History,
    Philosophy,
    Poetry,
    Science,
    Biography,
    Fantasy,
    Mystery,
}

impl Genre {
    pub fn name_es(&self) -> &'static str {
        match self {
            Genre::Fiction => "Ficción",
            Genre::SciFi => "Ciencia Ficción",
            Genre::History => "Historia",
            Genre::Philosophy => "Filosofía",
            Genre::Poetry => "Poesía",
            Genre::Science => "Ciencia",
            Genre::Biography => "Biografía",
            Genre::Fantasy => "Fantasía",
            Genre::Mystery => "Misterio",
        }
    }

    pub fn all() -> &'static [Genre] {
        &[
            Genre::Fiction,
            Genre::SciFi,
            Genre::History,
            Genre::Philosophy,
            Genre::Poetry,
            Genre::Science,
            Genre::Biography,
            Genre::Fantasy,
            Genre::Mystery,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BookSort {
    TitleAsc,
    TitleDesc,
    AuthorAsc,
    RecentProgress,
    TotalPagesDesc,
}

impl BookSort {
    pub fn name_es(&self) -> &'static str {
        match self {
            BookSort::TitleAsc => "Título (A-Z)",
            BookSort::TitleDesc => "Título (Z-A)",
            BookSort::AuthorAsc => "Autor (A-Z)",
            BookSort::RecentProgress => "Mayor progreso",
            BookSort::TotalPagesDesc => "Más páginas",
        }
    }

    pub fn all() -> &'static [BookSort] {
        &[
            BookSort::TitleAsc,
            BookSort::TitleDesc,
            BookSort::AuthorAsc,
            BookSort::RecentProgress,
            BookSort::TotalPagesDesc,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BookFilter {
    All,
    Reading,
    Completed,
    Unread,
}

impl BookFilter {
    pub fn name_es(&self) -> &'static str {
        match self {
            BookFilter::All => "Todos",
            BookFilter::Reading => "En progreso",
            BookFilter::Completed => "Completados",
            BookFilter::Unread => "Por leer",
        }
    }

    pub fn all() -> &'static [BookFilter] {
        &[
            BookFilter::All,
            BookFilter::Reading,
            BookFilter::Completed,
            BookFilter::Unread,
        ]
    }
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

    pub fn is_reading(&self) -> bool {
        self.current_page > 0 && self.current_page < self.total_pages
    }

    pub fn is_completed(&self) -> bool {
        self.total_pages > 0 && self.current_page >= self.total_pages
    }

    pub fn is_unread(&self) -> bool {
        self.current_page == 0
    }

    pub fn matches_filter(&self, filter: BookFilter) -> bool {
        match filter {
            BookFilter::All => true,
            BookFilter::Reading => self.is_reading(),
            BookFilter::Completed => self.is_completed(),
            BookFilter::Unread => self.is_unread(),
        }
    }

    pub fn matches_search(&self, query: &str) -> bool {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return true;
        }
        self.title.to_lowercase().contains(&q)
            || self.author.to_lowercase().contains(&q)
            || self.genre.name_es().to_lowercase().contains(&q)
    }
}
