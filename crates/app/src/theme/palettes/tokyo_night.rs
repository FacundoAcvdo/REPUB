use crate::theme::tokens::ThemeTokens;
use gpui_kit::{rgb, rgba};

pub fn tokyo_night_theme() -> ThemeTokens {
    ThemeTokens {
        id: "tokyo_night",
        name: "Tokyo Night",
        is_dark: true,
        bg: rgb(0x1a1b26).into(),
        bg_elevated: rgba(0xffffff0f).into(),
        bg_sunken: rgba(0x00000033).into(),
        bg_hover: rgba(0xc0caf514).into(),
        bg_active: rgba(0xc0caf526).into(),
        border: rgba(0xc0caf526).into(),
        border_subtle: rgba(0xc0caf514).into(),
        text: rgb(0xc0caf5).into(),
        text_muted: rgb(0x7aa2f7).into(),
        text_subtle: rgb(0x565f89).into(),
        accent: rgb(0x7aa2f7).into(),
        accent_hover: rgb(0xbb9af7).into(),
        accent_text: rgb(0x1a1b26).into(),
        success: rgb(0x9ece6a).into(),
        warning: rgb(0xe0af68).into(),
        danger: rgb(0xf7768e).into(),
    }
}
