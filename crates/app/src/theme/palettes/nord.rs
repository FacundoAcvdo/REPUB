use crate::theme::tokens::ThemeTokens;
use gpui_kit::{rgb, rgba};

pub fn nord_theme() -> ThemeTokens {
    ThemeTokens {
        id: "nord",
        name: "Nord",
        is_dark: true,
        bg: rgb(0x2e3440).into(),
        bg_elevated: rgba(0xffffff0f).into(),
        bg_sunken: rgba(0x00000033).into(),
        bg_hover: rgba(0xd8dee914).into(),
        bg_active: rgba(0xd8dee926).into(),
        border: rgba(0xd8dee926).into(),
        border_subtle: rgba(0xd8dee914).into(),
        text: rgb(0xeceff4).into(),
        text_muted: rgb(0xd8dee9).into(),
        text_subtle: rgb(0x4c566a).into(),
        accent: rgb(0x88c0d0).into(),
        accent_hover: rgb(0x81a1c1).into(),
        accent_text: rgb(0x2e3440).into(),
        success: rgb(0xa3be8c).into(),
        warning: rgb(0xebcb8b).into(),
        danger: rgb(0xbf616a).into(),
    }
}
