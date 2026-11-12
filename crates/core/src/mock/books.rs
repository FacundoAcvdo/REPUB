use crate::domain::book::{Book, Chapter, CoverPalette, Genre};

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
            chapters: mock_sample_chapters("El Faro de Alejandría"),
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
            chapters: mock_sample_chapters("Memorias del Silencio"),
        },
        Book {
            id: "b3".into(),
            title: "Crónicas de la Niebla".into(),
            author: "Clara Benítez".into(),
            total_pages: 420,
            current_page: 420,
            genre: Genre::Mystery,
            cover_palette: CoverPalette {
                primary: 0x3d2b3d,
                secondary: 0x63474d,
                accent: 0xaa767c,
                text: 0xffffff,
            },
            synopsis: "Una investigación misteriosa en las colinas brumosas del norte.".into(),
            chapters: mock_sample_chapters("Crónicas de la Niebla"),
        },
        Book {
            id: "b4".into(),
            title: "El Archipiélago de Cristal".into(),
            author: "Sebastián Alarcón".into(),
            total_pages: 512,
            current_page: 128,
            genre: Genre::SciFi,
            cover_palette: CoverPalette {
                primary: 0x0f2027,
                secondary: 0x203a43,
                accent: 0x2c5364,
                text: 0x8ef9f3,
            },
            synopsis: "Exploración en planetas oceánicos gobernados por redes cristalinas biológicas.".into(),
            chapters: mock_sample_chapters("El Archipiélago de Cristal"),
        },
        Book {
            id: "b5".into(),
            title: "La Geometría del Deseo".into(),
            author: "Ariadna Morel".into(),
            total_pages: 195,
            current_page: 0,
            genre: Genre::Poetry,
            cover_palette: CoverPalette {
                primary: 0x4a154b,
                secondary: 0x611f69,
                accent: 0xecb22e,
                text: 0xffffff,
            },
            synopsis: "Poemas sobre la distancia, el tiempo y la arquitectura del recuerdo.".into(),
            chapters: mock_sample_chapters("La Geometría del Deseo"),
        },
        Book {
            id: "b6".into(),
            title: "Breve Historia de las Sombras".into(),
            author: "Mateo Sforza".into(),
            total_pages: 360,
            current_page: 360,
            genre: Genre::History,
            cover_palette: CoverPalette {
                primary: 0x42240c,
                secondary: 0x673917,
                accent: 0xc19a6b,
                text: 0xffffff,
            },
            synopsis: "Las figuras olvidadas detrás de las grandes revoluciones de occidente.".into(),
            chapters: mock_sample_chapters("Breve Historia de las Sombras"),
        },
        Book {
            id: "b7".into(),
            title: "Singularidad y Conciencia".into(),
            author: "Dr. Kenji Takahashi".into(),
            total_pages: 480,
            current_page: 340,
            genre: Genre::Science,
            cover_palette: CoverPalette {
                primary: 0x141e30,
                secondary: 0x243b55,
                accent: 0x00d2ff,
                text: 0xffffff,
            },
            synopsis: "Fronteras entre la neurobiología y las redes neuronales artificiales profundas.".into(),
            chapters: mock_sample_chapters("Singularidad y Conciencia"),
        },
        Book {
            id: "b8".into(),
            title: "El Reino del Otoño Perpetuo".into(),
            author: "Valeria Lindqvist".into(),
            total_pages: 620,
            current_page: 55,
            genre: Genre::Fantasy,
            cover_palette: CoverPalette {
                primary: 0x3e1f47,
                secondary: 0x5a2d67,
                accent: 0xf39c12,
                text: 0xffffff,
            },
            synopsis: "Una epopeya donde las estaciones se detuvieron tras la caída de la corte solar.".into(),
            chapters: mock_sample_chapters("El Reino del Otoño Perpetuo"),
        },
        Book {
            id: "b9".into(),
            title: "Cartas desde el Estuario".into(),
            author: "Daniel Ocampo".into(),
            total_pages: 230,
            current_page: 0,
            genre: Genre::Biography,
            cover_palette: CoverPalette {
                primary: 0x1f3c3d,
                secondary: 0x325d5e,
                accent: 0xa8d0db,
                text: 0xffffff,
            },
            synopsis: "Correspondencia autobiográfica durante la construcción del canal interoceánico.".into(),
            chapters: mock_sample_chapters("Cartas desde el Estuario"),
        },
        Book {
            id: "b10".into(),
            title: "Los Cielos de Calcuta".into(),
            author: "Ananya Sen".into(),
            total_pages: 310,
            current_page: 180,
            genre: Genre::Fiction,
            cover_palette: CoverPalette {
                primary: 0x59253a,
                secondary: 0x78244c,
                accent: 0xf1c40f,
                text: 0xffffff,
            },
            synopsis: "Retrato íntimo de tres generaciones entrelazadas por la música clásica india.".into(),
            chapters: mock_sample_chapters("Los Cielos de Calcuta"),
        },
        Book {
            id: "b11".into(),
            title: "Teoría del Vértigo".into(),
            author: "Lucas Von Berg".into(),
            total_pages: 260,
            current_page: 0,
            genre: Genre::Philosophy,
            cover_palette: CoverPalette {
                primary: 0x222629,
                secondary: 0x474b4f,
                accent: 0x86c232,
                text: 0xffffff,
            },
            synopsis: "Reflexiones sobre la aceleración tecnológica y la pérdida del centro existencial.".into(),
            chapters: mock_sample_chapters("Teoría del Vértigo"),
        },
        Book {
            id: "b12".into(),
            title: "El Último Meridiano".into(),
            author: "Camila Arismendi".into(),
            total_pages: 380,
            current_page: 95,
            genre: Genre::SciFi,
            cover_palette: CoverPalette {
                primary: 0x0b132b,
                secondary: 0x1c2541,
                accent: 0x48cae4,
                text: 0xffffff,
            },
            synopsis: "Un thriller espacial en los límites de la zona habitable del sistema solar.".into(),
            chapters: mock_sample_chapters("El Último Meridiano"),
        },
    ]
}

pub fn generate_mock_catalog(count: usize) -> Vec<Book> {
    let base = mock_books();
    if count <= base.len() {
        return base[..count].to_vec();
    }

    let mut result = Vec::with_capacity(count);
    result.extend(base);

    let prefixes = [
        "El Secreto de", "La Ciudad de", "Memorias de", "El Arte de", "Crónicas de",
        "El Ecosistema de", "Reflexiones sobre", "La Travesía de", "Voces en", "El Enigma de",
        "Viaje hacia", "La Danza de", "El Manuscrito de", "Luz sobre", "Caminos hacia",
    ];

    let nouns = [
        "los Glaciares", "la Medianoche", "las Esferas", "los Vientos Antiguos", "la Selva Negra",
        "los Nombres Perdidos", "el Laberinto", "las Sombras Cálidas", "los Puentes de Hierro",
        "las Arenas Blancas", "el Infinito Silencio", "las Cartas no Escritas", "la Frontera Azul",
    ];

    let authors = [
        "Julián Vélez", "Sofía Carrizo", "Guillermo Bauer", "Noelia Paredes", "Arturo Méndez",
        "Beatriz Soler", "Gonzalo Rivas", "Lucía Herrera", "Tomás Quiroga", "Inés Navarro",
        "Mariano Duarte", "Fernanda Gil", "Esteban Varela", "Catalina Prado", "Ignacio Soto",
    ];

    let genres = [
        Genre::Fiction, Genre::SciFi, Genre::History, Genre::Philosophy,
        Genre::Poetry, Genre::Science, Genre::Biography, Genre::Fantasy, Genre::Mystery,
    ];

    let palettes = [
        CoverPalette { primary: 0x1b263b, secondary: 0x415a77, accent: 0xe0e1dd, text: 0xffffff },
        CoverPalette { primary: 0x2b2d42, secondary: 0x8d99ae, accent: 0xef233c, text: 0xffffff },
        CoverPalette { primary: 0x14213d, secondary: 0x000000, accent: 0xfca311, text: 0xffffff },
        CoverPalette { primary: 0x003049, secondary: 0x669bbc, accent: 0xfdf0d5, text: 0xffffff },
        CoverPalette { primary: 0x264653, secondary: 0x2a9d8f, accent: 0xe76f51, text: 0xffffff },
        CoverPalette { primary: 0x3d315b, secondary: 0x444b6e, accent: 0x9ab87a, text: 0xffffff },
        CoverPalette { primary: 0x2e1c2b, secondary: 0x4a1942, accent: 0x893168, text: 0xffffff },
        CoverPalette { primary: 0x1a1a24, secondary: 0x2d3142, accent: 0xbfc0c0, text: 0xffffff },
        CoverPalette { primary: 0x03254c, secondary: 0x1167b1, accent: 0x187bcd, text: 0xffffff },
        CoverPalette { primary: 0x283618, secondary: 0x606c38, accent: 0xdda15e, text: 0xffffff },
    ];

    let mut id_counter = result.len() + 1;
    while result.len() < count {
        let i = result.len();
        let prefix = prefixes[i % prefixes.len()];
        let noun = nouns[(i / prefixes.len()) % nouns.len()];
        let title = format!("{} {}", prefix, noun);
        let author = authors[i % authors.len()].to_string();
        let genre = genres[(i * 7) % genres.len()];
        let palette = palettes[i % palettes.len()];

        let total_pages = 180 + ((i as u32 * 37) % 520);
        let current_page = match i % 4 {
            0 => 0, // Por leer
            1 => total_pages, // Completado
            _ => ((i as u32 * 23) % total_pages).min(total_pages - 1), // En progreso
        };

        result.push(Book {
            id: format!("b{}", id_counter),
            title,
            author,
            total_pages,
            current_page,
            genre,
            cover_palette: palette,
            synopsis: format!("Una fascinante obra literaria sobre {}, escrita con profundidad y elegancia.", noun),
            chapters: mock_sample_chapters("Capítulo General"),
        });

        id_counter += 1;
    }

    result
}

fn mock_sample_chapters(book_title: &str) -> Vec<Chapter> {
    vec![
        Chapter {
            id: "c1".into(),
            title: format!("Capítulo I: El Comienzo de {}", book_title),
            pages: vec![
                "La tarde caía sobre los techos de tejas viejas cuando llegó la primera noticia...".into(),
                "Nadie en el pueblo recordaba una calma tan densa, como si el aire contuviera el aliento.".into(),
            ],
        },
        Chapter {
            id: "c2".into(),
            title: "Capítulo II: Las Primeras Señales".into(),
            pages: vec![
                "Avanzaron siguiendo la línea irregular de los sauces que bordeaban el cauce seco.".into(),
                "El silencio que siguió a sus palabras fue más elocuente que cualquier respuesta articulada.".into(),
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::book::{BookFilter, BookSort};

    #[test]
    fn test_mock_books_count() {
        let books = mock_books();
        assert_eq!(books.len(), 12);
        assert!(books[0].progress_pct() > 0.0);
    }

    #[test]
    fn test_generate_1000_catalog() {
        let catalog = generate_mock_catalog(1000);
        assert_eq!(catalog.len(), 1000);
        
        let reading_count = catalog.iter().filter(|b| b.matches_filter(BookFilter::Reading)).count();
        let completed_count = catalog.iter().filter(|b| b.matches_filter(BookFilter::Completed)).count();
        let unread_count = catalog.iter().filter(|b| b.matches_filter(BookFilter::Unread)).count();

        assert!(reading_count > 0);
        assert!(completed_count > 0);
        assert!(unread_count > 0);
        assert_eq!(reading_count + completed_count + unread_count, 1000);
    }

    #[test]
    fn test_search_and_sort() {
        let catalog = generate_mock_catalog(50);
        let matches = catalog.iter().filter(|b| b.matches_search("Faro")).count();
        assert!(matches >= 1);

        assert_eq!(BookSort::all().len(), 5);
        let mut sorted = catalog.clone();
        sorted.sort_by(|a, b| a.title.cmp(&b.title));
        assert!(sorted[0].title <= sorted[1].title);
    }
}
