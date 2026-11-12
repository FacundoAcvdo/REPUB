use crate::theme::tokens::ThemeTokens;
use gpui_kit::{rgb, rgba};

pub fn dark_theme() -> ThemeTokens {
    ThemeTokens {
        id: "dark",
        name: "Oscuro",
        is_dark: true,
        bg: rgb(0x121316).into(),
        bg_elevated: rgba(0xffffff0f).into(),
        bg_sunken: rgba(0x00000033).into(),
        bg_hover: rgba(0xffffff14).into(),
        bg_active: rgba(0xffffff24).into(),
        border: rgba(0xffffff1a).into(),
        border_subtle: rgba(0xffffff0d).into(),
        text: rgb(0xf3f4f6).into(),
        text_muted: rgb(0x9ca3af).into(),
        text_subtle: rgb(0x6b7280).into(),
        accent: rgb(0x3b82f6).into(),
        accent_hover: rgb(0x60a5fa).into(),
        accent_text: rgb(0xffffff).into(),
        success: rgb(0x22c55e).into(),
        warning: rgb(0xeab308).into(),
        danger: rgb(0xef4444).into(),
    }
}
