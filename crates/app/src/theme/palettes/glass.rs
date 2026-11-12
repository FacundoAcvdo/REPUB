use crate::theme::tokens::ThemeTokens;
use gpui_kit::{rgb, rgba};

pub fn glass_theme() -> ThemeTokens {
    ThemeTokens {
        id: "glass",
        name: "Glass",
        is_dark: true,
        bg: rgba(0x16181ecc).into(),
        bg_elevated: rgba(0xffffff14).into(),
        bg_sunken: rgba(0x00000033).into(),
        bg_hover: rgba(0xffffff1f).into(),
        bg_active: rgba(0xffffff2e).into(),
        border: rgba(0xffffff26).into(),
        border_subtle: rgba(0xffffff14).into(),
        text: rgb(0xf3f4f6).into(),
        text_muted: rgb(0x9ca3af).into(),
        text_subtle: rgb(0x6b7280).into(),
        accent: rgb(0x60a5fa).into(),
        accent_hover: rgb(0x93c5fd).into(),
        accent_text: rgb(0x0f172a).into(),
        success: rgb(0x4ade80).into(),
        warning: rgb(0xfacc15).into(),
        danger: rgb(0xf87171).into(),
    }
}
