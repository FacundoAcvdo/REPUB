use crate::domain::book::{Book, CoverPalette, Genre};

pub fn mock_books() -> Vec<Book> {
    vec![
        Book {
            id: "b1".into(),
            title: "El Faro de Alejandría".into(),
            author: "Elena Rostova".into(),
            total_pages: 340,
            current_page: 85,
            genre: Genre::Fiction,
            cover_palette: CoverPalette {
                primary: 0x1e3a5f,
                secondary: 0x2d5f8b,
                accent: 0xe0a96d,
                text: 0xffffff,
            },
            synopsis: "Una crónica sobre la memoria, el mar y las luces que guían en la tormenta.".into(),
            chapters: vec![],
        },
        Book {
            id: "b2".into(),
            title: "Memorias del Silencio".into(),
            author: "Marco Valerio".into(),
            total_pages: 280,
            current_page: 210,
            genre: Genre::Philosophy,
            cover_palette: CoverPalette {
                primary: 0x2c3531,
                secondary: 0x116466,
                accent: 0xd1e8e2,
                text: 0xffffff,
            },
            synopsis: "Tratado íntimo sobre la quietud y el pensamiento contemporáneo.".into(),
            chapters: vec![],
        },
    ]
}
