use epub_core::domain::settings::CornerRadiusStyle;
use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::{App, Global, Hsla, Pixels};

use crate::theme::palettes::get_theme_by_id;
use crate::theme::radius::RadiusTokens;
use crate::theme::tokens::ThemeTokens;

#[derive(Debug, Clone)]
pub struct ThemeState {
    pub current_theme: ThemeTokens,
    pub corner_radius: CornerRadiusStyle,
    pub window_opacity: f32,
    pub interface_blur: bool,
}

impl Global for ThemeState {}

impl Default for ThemeState {
    fn default() -> Self {
        Self {
            current_theme: get_theme_by_id("dark"),
            corner_radius: CornerRadiusStyle::Rounded,
            window_opacity: 1.0,
            interface_blur: false,
        }
    }
}

impl ThemeState {
    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    pub fn global_mut(cx: &mut App) -> &mut Self {
        cx.global_mut::<Self>()
    }

    pub fn tokens(&self) -> &ThemeTokens {
        &self.current_theme
    }

    pub fn window_bg(&self) -> Hsla {
        self.current_theme.bg_with_opacity(self.window_opacity)
    }

    pub fn radius(&self) -> Pixels {
        self.corner_radius.base_radius()
    }

    pub fn sm_radius(&self) -> Pixels {
        self.corner_radius.sm_radius()
    }

    pub fn lg_radius(&self) -> Pixels {
        self.corner_radius.lg_radius()
    }

    pub fn update_theme(cx: &mut App, theme_id: &str) {
        let mut state = cx.global::<Self>().clone();
        state.current_theme = get_theme_by_id(theme_id);
        state.sync_with_gpui(cx);
        cx.set_global(state);
    }

    pub fn update_corner_radius(cx: &mut App, style: CornerRadiusStyle) {
        let mut state = cx.global::<Self>().clone();
        state.corner_radius = style;
        state.sync_with_gpui(cx);
        cx.set_global(state);
    }

    pub fn update_window_opacity(cx: &mut App, opacity: f32) {
        let mut state = cx.global::<Self>().clone();
        state.window_opacity = opacity.clamp(0.1, 1.0);
        state.sync_with_gpui(cx);
        cx.set_global(state);
    }

    pub fn update_interface_blur(cx: &mut App, blur: bool) {
        let mut state = cx.global::<Self>().clone();
        state.interface_blur = blur;
        state.sync_with_gpui(cx);
        cx.set_global(state);
    }

    pub fn sync_with_gpui(&self, cx: &mut App) {
        let tokens = self.current_theme;
        let radius = self.radius();
        let lg_radius = self.lg_radius();
        let mode = if tokens.is_dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        };

        Theme::update(cx, |theme| {
            theme.mode = mode;
            theme.radius = radius;
            theme.radius_lg = lg_radius;

            theme.colors.background = tokens.bg;
            theme.colors.foreground = tokens.text;
            theme.colors.border = tokens.border;

            theme.colors.primary = tokens.accent;
            theme.colors.primary_hover = tokens.accent_hover;
            theme.colors.primary_foreground = tokens.accent_text;

            theme.colors.secondary = tokens.bg_elevated;
            theme.colors.secondary_foreground = tokens.text;

            theme.colors.muted = tokens.bg_sunken;
            theme.colors.muted_foreground = tokens.text_muted;

            theme.colors.accent = tokens.bg_hover;
            theme.colors.accent_foreground = tokens.text;

            theme.colors.sidebar = tokens.bg;
            theme.colors.sidebar_foreground = tokens.text;
            theme.colors.sidebar_border = tokens.border;
            theme.colors.sidebar_accent = tokens.bg_hover;
            theme.colors.sidebar_accent_foreground = tokens.text;

            theme.colors.danger = tokens.danger;
            theme.colors.success = tokens.success;
            theme.colors.warning = tokens.warning;
        });
    }
}
