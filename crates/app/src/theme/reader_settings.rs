use epub_core::domain::settings::ReaderSheetSettings;
use gpui_kit::{App, Global};

#[derive(Debug, Clone, PartialEq)]
pub struct ReaderSettingsState {
    pub settings: ReaderSheetSettings,
}

impl Global for ReaderSettingsState {}

impl Default for ReaderSettingsState {
    fn default() -> Self {
        Self {
            settings: ReaderSheetSettings::default(),
        }
    }
}

impl ReaderSettingsState {
    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    pub fn global_mut(cx: &mut App) -> &mut Self {
        cx.global_mut::<Self>()
    }

    pub fn update(cx: &mut App, settings: ReaderSheetSettings) {
        cx.set_global(Self { settings });
    }

    /// Mapea el nombre amigable de la fuente configurada a la fuente tipográfica disponible en el sistema.
    pub fn resolved_font_family(&self) -> &'static str {
        let name = self.settings.font_family.to_lowercase();
        if name.contains("sans") {
            "Liberation Sans"
        } else if name.contains("mono") {
            "Liberation Mono"
        } else {
            // Serif por defecto (Liberation Serif / DejaVu Serif / Georgia)
            "Liberation Serif"
        }
    }
}
