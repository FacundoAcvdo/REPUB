use crate::theme::tokens::ThemeTokens;
use gpui_kit::{rgb, rgba};

pub fn catppuccin_frappe_theme() -> ThemeTokens {
    ThemeTokens {
        id: "catppuccin_frappe",
        name: "Catppuccin Frappé",
        is_dark: true,
        bg: rgb(0x303446).into(),
        bg_elevated: rgba(0xffffff0f).into(),
        bg_sunken: rgba(0x00000033).into(),
        bg_hover: rgba(0xc6d0f514).into(),
        bg_active: rgba(0xc6d0f526).into(),
        border: rgba(0xc6d0f526).into(),
        border_subtle: rgba(0xc6d0f514).into(),
        text: rgb(0xc6d0f5).into(),
        text_muted: rgb(0xa5adce).into(),
        text_subtle: rgb(0x737994).into(),
        accent: rgb(0xca9ee6).into(),
        accent_hover: rgb(0xbabbf1).into(),
        accent_text: rgb(0x232634).into(),
        success: rgb(0xa6d189).into(),
        warning: rgb(0xe5c890).into(),
        danger: rgb(0xe78284).into(),
    }
}

pub fn catppuccin_latte_theme() -> ThemeTokens {
    ThemeTokens {
        id: "catppuccin_latte",
        name: "Catppuccin Latte",
        is_dark: false,
        bg: rgb(0xeff1f5).into(),
        bg_elevated: rgb(0xe6e9ef).into(),
        bg_sunken: rgb(0xdce0e8).into(),
        bg_hover: rgba(0x4c4f690f).into(),
        bg_active: rgba(0x4c4f691c).into(),
        border: rgba(0x4c4f691c).into(),
        border_subtle: rgba(0x4c4f690f).into(),
        text: rgb(0x4c4f69).into(),
        text_muted: rgb(0x6c6f85).into(),
        text_subtle: rgb(0x9ca0b0).into(),
        accent: rgb(0x8839ef).into(),
        accent_hover: rgb(0x7287fd).into(),
        accent_text: rgb(0xffffff).into(),
        success: rgb(0x40a02b).into(),
        warning: rgb(0xdf8e1d).into(),
        danger: rgb(0xd20f39).into(),
    }
}

pub fn catppuccin_macchiato_theme() -> ThemeTokens {
    ThemeTokens {
        id: "catppuccin_macchiato",
        name: "Catppuccin Macchiato",
        is_dark: true,
        bg: rgb(0x24273a).into(),
        bg_elevated: rgba(0xffffff0f).into(),
        bg_sunken: rgba(0x00000033).into(),
        bg_hover: rgba(0xcad3f514).into(),
        bg_active: rgba(0xcad3f526).into(),
        border: rgba(0xcad3f526).into(),
        border_subtle: rgba(0xcad3f514).into(),
        text: rgb(0xcad3f5).into(),
        text_muted: rgb(0xa5adcb).into(),
        text_subtle: rgb(0x6e738d).into(),
        accent: rgb(0xc6a0f6).into(),
        accent_hover: rgb(0xb7bdf8).into(),
        accent_text: rgb(0x1e2030).into(),
        success: rgb(0xa6da95).into(),
        warning: rgb(0xeed49f).into(),
        danger: rgb(0xed8796).into(),
    }
}

pub fn catppuccin_mocha_theme() -> ThemeTokens {
    ThemeTokens {
        id: "catppuccin_mocha",
        name: "Catppuccin Mocha",
        is_dark: true,
        bg: rgb(0x1e1e2e).into(),
        bg_elevated: rgba(0xffffff0f).into(),
        bg_sunken: rgba(0x00000033).into(),
        bg_hover: rgba(0xcdd6f414).into(),
        bg_active: rgba(0xcdd6f426).into(),
        border: rgba(0xcdd6f426).into(),
        border_subtle: rgba(0xcdd6f414).into(),
        text: rgb(0xcdd6f4).into(),
        text_muted: rgb(0xa6adc8).into(),
        text_subtle: rgb(0x6c7086).into(),
        accent: rgb(0xcba6f7).into(),
        accent_hover: rgb(0xb4befe).into(),
        accent_text: rgb(0x181825).into(),
        success: rgb(0xa6e3a1).into(),
        warning: rgb(0xf9e2af).into(),
        danger: rgb(0xf38ba8).into(),
    }
}
