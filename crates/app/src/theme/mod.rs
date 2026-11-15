pub mod palettes;
pub mod radius;
pub mod reader_settings;
pub mod state;
pub mod tokens;
pub mod typography;

pub use palettes::*;
pub use radius::*;
pub use reader_settings::*;
pub use state::*;
pub use tokens::*;
pub use typography::*;

use epub_core::db::{Database, SettingsRepository};
use epub_core::domain::settings::AppSettings;
use gpui_kit::App;

pub fn init_theme(cx: &mut App) {
    let app_settings = Database::default_db_path()
        .and_then(|path| Database::open(path).ok())
        .and_then(|db| SettingsRepository::load_settings(db.conn()).ok())
        .unwrap_or_else(AppSettings::default);

    let state = ThemeState {
        current_theme: get_theme_by_id(&app_settings.appearance.theme_id),
        corner_radius: app_settings.appearance.corner_radius,
        window_opacity: app_settings.appearance.window_opacity.clamp(0.1, 1.0),
        interface_blur: app_settings.appearance.interface_blur,
    };
    state.sync_with_gpui(cx);
    cx.set_global(state);
    cx.set_global(ReaderSettingsState {
        settings: app_settings.reader,
    });
}
