use crate::theme::tokens::ThemeTokens;
use gpui_kit::{rgb, rgba};

pub fn solarized_dark_theme() -> ThemeTokens {
    ThemeTokens {
        id: "solarized_dark",
        name: "Solarized Dark",
        is_dark: true,
        bg: rgb(0x002b36).into(),
        bg_elevated: rgba(0xffffff0d).into(),
        bg_sunken: rgba(0x00000033).into(),
        bg_hover: rgba(0x93a1a114).into(),
        bg_active: rgba(0x93a1a126).into(),
        border: rgba(0x586e7540).into(),
        border_subtle: rgba(0x586e7526).into(),
        text: rgb(0x839496).into(),
        text_muted: rgb(0x657b83).into(),
        text_subtle: rgb(0x586e75).into(),
        accent: rgb(0x268bd2).into(),
        accent_hover: rgb(0x2aa198).into(),
        accent_text: rgb(0xfdf6e3).into(),
        success: rgb(0x859900).into(),
        warning: rgb(0xb58900).into(),
        danger: rgb(0xdc322f).into(),
    }
}
