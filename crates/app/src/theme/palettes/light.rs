use crate::theme::tokens::ThemeTokens;
use gpui_kit::{rgb, rgba};

pub fn light_theme() -> ThemeTokens {
    ThemeTokens {
        id: "light",
        name: "Claro",
        is_dark: false,
        bg: rgb(0xf8f9fa).into(),
        bg_elevated: rgba(0xffffff80).into(),
        bg_sunken: rgba(0x0000000f).into(),
        bg_hover: rgba(0x0000000f).into(),
        bg_active: rgba(0x00000014).into(),
        border: rgba(0x00000018).into(),
        border_subtle: rgba(0x0000000d).into(),
        text: rgb(0x18191b).into(),
        text_muted: rgb(0x6b7280).into(),
        text_subtle: rgb(0x9ca3af).into(),
        accent: rgb(0x2563eb).into(),
        accent_hover: rgb(0x1d4ed8).into(),
        accent_text: rgb(0xffffff).into(),
        success: rgb(0x16a34a).into(),
        warning: rgb(0xca8a04).into(),
        danger: rgb(0xdc2626).into(),
    }
}
