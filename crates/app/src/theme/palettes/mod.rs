pub mod catppuccin;
pub mod dark;
pub mod dracula;
pub mod glass;
pub mod light;
pub mod nord;
pub mod solarized_dark;
pub mod tokyo_night;

use crate::theme::tokens::ThemeTokens;

pub use catppuccin::*;
pub use dark::*;
pub use dracula::*;
pub use glass::*;
pub use light::*;
pub use nord::*;
pub use solarized_dark::*;
pub use tokyo_night::*;

pub fn all_themes() -> Vec<ThemeTokens> {
    vec![
        dark_theme(),
        light_theme(),
        tokyo_night_theme(),
        catppuccin_frappe_theme(),
        catppuccin_latte_theme(),
        catppuccin_macchiato_theme(),
        catppuccin_mocha_theme(),
        solarized_dark_theme(),
        nord_theme(),
        dracula_theme(),
        glass_theme(),
    ]
}

pub fn get_theme_by_id(id: &str) -> ThemeTokens {
    match id {
        "light" => light_theme(),
        "tokyo_night" => tokyo_night_theme(),
        "catppuccin_frappe" => catppuccin_frappe_theme(),
        "catppuccin_latte" => catppuccin_latte_theme(),
        "catppuccin_macchiato" => catppuccin_macchiato_theme(),
        "catppuccin_mocha" => catppuccin_mocha_theme(),
        "solarized_dark" => solarized_dark_theme(),
        "nord" => nord_theme(),
        "dracula" => dracula_theme(),
        "glass" => glass_theme(),
        _ => dark_theme(),
    }
}
