use crate::theme::tokens::ThemeTokens;
use gpui_kit::{rgb, rgba};

pub fn dracula_theme() -> ThemeTokens {
    ThemeTokens {
        id: "dracula",
        name: "Dracula",
        is_dark: true,
        bg: rgb(0x282a36).into(),
        bg_elevated: rgba(0xffffff0f).into(),
        bg_sunken: rgba(0x00000033).into(),
        bg_hover: rgba(0xf8f8f214).into(),
        bg_active: rgba(0xf8f8f226).into(),
        border: rgba(0x6272a440).into(),
        border_subtle: rgba(0x6272a426).into(),
        text: rgb(0xf8f8f2).into(),
        text_muted: rgb(0xbd93f9).into(),
        text_subtle: rgb(0x6272a4).into(),
        accent: rgb(0xbd93f9).into(),
        accent_hover: rgb(0xff79c6).into(),
        accent_text: rgb(0x282a36).into(),
        success: rgb(0x50fa7b).into(),
        warning: rgb(0xf1fa8c).into(),
        danger: rgb(0xff5555).into(),
    }
}
