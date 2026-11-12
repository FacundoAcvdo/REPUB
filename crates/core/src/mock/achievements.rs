use crate::domain::achievement::Achievement;

pub fn mock_achievements() -> Vec<Achievement> {
    vec![
        Achievement {
            id: "ach_first_book".into(),
            title: "Primera Página".into(),
            description: "Termina la lectura de tu primer libro completo.".into(),
            unlocked: true,
            unlocked_at: Some("12 oct 2026".into()),
            icon_name: "book-open".into(),
        },
        Achievement {
            id: "ach_streak_7".into(),
            title: "Hábito Imparable".into(),
            description: "Mantén una racha de lectura durante 7 días consecutivos.".into(),
            unlocked: true,
            unlocked_at: Some("04 nov 2026".into()),
            icon_name: "flame".into(),
        },
        Achievement {
            id: "ach_centenary".into(),
            title: "Centenario en un Día".into(),
            description: "Lee 100 páginas en una sola jornada.".into(),
            unlocked: false,
            unlocked_at: None,
            icon_name: "zap".into(),
        },
    ]
}
