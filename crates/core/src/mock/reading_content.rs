#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageContent {
    pub chapter_number: u32,
    pub chapter_title: String,
    pub paragraphs: Vec<String>,
}

pub fn mock_sample_text() -> &'static str {
    "La luz declinaba suavemente sobre la orilla, extendiendo un manto de sombras largas que se perdían entre las piedras pulidas por la marea. A lo lejos, la silueta del faro comenzaba a destellar con su pulso mecánico y calmo, un latido de fuego blanco en medio de la penumbra creciente.\n\nNo había prisa en el aire; sólo la cadencia monótona de las olas rompiendo contra el espigón y el murmullo lejano de un viento que traía consigo el aroma a salitre y a maderas viejas. En momentos así, la lectura dejaba de ser un mero ejercicio de contemplación para convertirse en un refugio impenetrable, donde cada palabra escrita resonaba como un eco antiguo recuperado del olvido."
}

/// Genera el contenido literario ficticio para una página determinada de un libro.
/// Proporciona párrafos elegantes, reflexivos y sin derechos de autor en español.
pub fn get_page_content(book_title: &str, page_number: u32, total_pages: u32) -> PageContent {
    let pages_per_chapter = 25.max(total_pages / 12);
    let chapter_number = (page_number.saturating_sub(1) / pages_per_chapter) + 1;
    let page_in_chapter = ((page_number.saturating_sub(1)) % pages_per_chapter) + 1;

    let chapter_names = [
        "El despertar de la bruma",
        "Los signos del horizonte",
        "El pulso de la marea",
        "Memoria de los días inmóviles",
        "La arquitectura del silencio",
        "Caminos entre las sombras",
        "El lenguaje de las piedras",
        "Ecos del viejo puerto",
        "La frontera de cristal",
        "Hojas que arrastra el viento",
        "El instante suspendido",
        "Hacia la última orilla",
    ];

    let chapter_title = format!(
        "Capítulo {}: {}",
        roman_numeral(chapter_number),
        chapter_names[(chapter_number as usize - 1) % chapter_names.len()]
    );

    let seed = (page_number as usize * 17) + book_title.len() * 31;

    let paragraph_pool = [
        "La luz declinaba suavemente sobre la orilla, extendiendo un manto de sombras largas que se perdían entre las piedras pulidas por la marea. A lo lejos, la silueta del faro comenzaba a destellar con su pulso mecánico y calmo, un latido de fuego blanco en medio de la penumbra creciente.",
        "No había prisa en el aire; sólo la cadencia monótona de las olas rompiendo contra el espigón y el murmullo lejano de un viento que traía consigo el aroma a salitre y a maderas viejas. En momentos así, la lectura dejaba de ser un mero ejercicio de contemplación para convertirse en un refugio impenetrable.",
        "Los cuadernos apilados sobre la mesa de nogal guardaban fragmentos de conversaciones inconclusas, mapas trazados a mano con tintas descoloridas y preguntas que el tiempo no se había molestado en responder. Cada anotación al margen era como una huella dejada en la nieve fresca, frágil pero indeleble.",
        "Avanzar una página implicaba aceptar el vértigo de lo desconocido. No porque los hechos narrados fueran violentos o inesperados, sino porque poseían esa extraña cualidad de reflejar pensamientos que uno creía haber sepultado en lo más íntimo de la memoria.",
        "El crujido sutil del papel al girar la hoja era el único sonido que competía con la respiración pausada de la noche. En la penumbra de la estancia, la tipografía cobraba un relieve casi escultórico, líneas de plomo convertidas en senderos por los que el pensamiento discurría sin ataduras.",
        "Había una serenidad profunda en aquel aislamiento voluntario. Las campanas de la torre sonaron tres veces a la distancia, un eco apagado que apenas logró rozar los cristales antes de disolverse en la niebla que ascendía desde el valle.",
        "Todo viaje interior exige una cierta renuncia a la velocidad del mundo exterior. Aquellos que buscan respuestas inmediatas rara vez comprenden que el verdadero hallazgo reside en la pausa, en el espacio en blanco que separa una frase de la siguiente.",
        "La tinta negra sobre el fondo marfil parecía absorber la quietud del cuarto. Los personajes no tenían prisa por revelar sus secretos; aguardaban con paciencia la mirada atenta de quien supiera descifrar no sólo lo dicho, sino los silencios calculados entre párrafo y párrafo.",
        "Recordó entonces las palabras que su abuelo solía repetir al caer la tarde: los libros no cambian con los años, pero nosotros renacemos distintos cada vez que volvemos a abrirlos bajo una nueva estación de la vida.",
        "El viento arreció levemente contra los postigos, agitando las cortinas de lino. Por una grieta en la madera se filtró un destello de luna, dibujando un arco pálido sobre el suelo de roble que parecía señalar la continuidad exacta del relato.",
        "Al mirar por la ventana hacia el horizonte, los contornos de las colinas parecían fundirse con el cielo nocturno en una sola gradación de grises y añiles profundos. Nada perturbaba el orden silencioso de las cosas bien dispuestas.",
        "Y sin embargo, en esa aparente inmovilidad, todo se transformaba. La idea esbozada al principio del capítulo había madurado, dejando tras de sí una estela de reflexiones que transformarían para siempre la comprensión del camino recorrido.",
    ];

    let p1 = paragraph_pool[seed % paragraph_pool.len()].to_string();
    let p2 = paragraph_pool[(seed + 3) % paragraph_pool.len()].to_string();
    let p3 = paragraph_pool[(seed + 7) % paragraph_pool.len()].to_string();

    let mut paragraphs = vec![p1, p2, p3];

    // Para la primera página del capítulo, agregamos un párrafo introductorio contextual
    if page_in_chapter == 1 {
        paragraphs.insert(
            0,
            format!(
                "Las primeras líneas de {} marcaban un punto de inflexión en el curso de los acontecimientos.",
                chapter_title
            ),
        );
    }

    PageContent {
        chapter_number,
        chapter_title,
        paragraphs,
    }
}

fn roman_numeral(n: u32) -> &'static str {
    match n {
        1 => "I",
        2 => "II",
        3 => "III",
        4 => "IV",
        5 => "V",
        6 => "VI",
        7 => "VII",
        8 => "VIII",
        9 => "IX",
        10 => "X",
        11 => "XI",
        12 => "XII",
        _ => "XIII+",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_page_content_bounds() {
        let content_p1 = get_page_content("El Faro", 1, 300);
        assert_eq!(content_p1.chapter_number, 1);
        assert!(!content_p1.paragraphs.is_empty());

        let content_p85 = get_page_content("El Faro", 85, 300);
        assert!(content_p85.chapter_number > 1);
        assert!(!content_p85.paragraphs.is_empty());

        let content_last = get_page_content("El Faro", 300, 300);
        assert!(!content_last.paragraphs.is_empty());
    }
}
