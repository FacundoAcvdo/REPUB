use crate::domain::achievement::Achievement;

pub fn mock_achievements() -> Vec<Achievement> {
    vec![
        Achievement {
            id: "ach_first_book".into(),
            title: "Primera Página".into(),
            description: "Termina la lectura de tu primer libro completo en la aplicación.".into(),
            unlocked: true,
            unlocked_at: Some("12 oct 2026".into()),
            icon_name: "book-open".into(),
        },
        Achievement {
            id: "ach_streak_7".into(),
            title: "Hábito Imparable".into(),
            description: "Mantén una racha de lectura continua durante 7 días consecutivos.".into(),
            unlocked: true,
            unlocked_at: Some("04 nov 2026".into()),
            icon_name: "flame".into(),
        },
        Achievement {
            id: "ach_night_reader".into(),
            title: "Búho Nocturno".into(),
            description: "Disfruta de más de una hora de lectura durante la medianoche.".into(),
            unlocked: true,
            unlocked_at: Some("08 nov 2026".into()),
            icon_name: "moon".into(),
        },
        Achievement {
            id: "ach_centenary".into(),
            title: "Centenario en un Día".into(),
            description: "Devora 100 páginas de tus obras favoritas en una sola jornada.".into(),
            unlocked: false,
            unlocked_at: None,
            icon_name: "zap".into(),
        },
        Achievement {
            id: "ach_collector".into(),
            title: "Gran Coleccionista".into(),
            description: "Organiza más de 20 libros en tu biblioteca personal.".into(),
            unlocked: false,
            unlocked_at: None,
            icon_name: "library".into(),
        },
        Achievement {
            id: "ach_classics".into(),
            title: "Erudito Clásico".into(),
            description: "Completa 3 libros del género Filosofía, Historia o Ciencia Ficción.".into(),
            unlocked: false,
            unlocked_at: None,
            icon_name: "award".into(),
        },
    ]
}
