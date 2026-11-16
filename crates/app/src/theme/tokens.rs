use gpui_kit::Hsla;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThemeTokens {
    pub id: &'static str,
    pub name: &'static str,
    pub is_dark: bool,

    // Surfaces
    pub bg: Hsla,
    pub bg_elevated: Hsla,
    pub bg_sunken: Hsla,
    pub bg_hover: Hsla,
    pub bg_active: Hsla,

    // Borders & dividers
    pub border: Hsla,
    pub border_subtle: Hsla,

    // Typography colors
    pub text: Hsla,
    pub text_muted: Hsla,
    pub text_subtle: Hsla,

    // Accent
    pub accent: Hsla,
    pub accent_hover: Hsla,
    pub accent_text: Hsla,

    // Feedback states
    pub success: Hsla,
    pub warning: Hsla,
    pub danger: Hsla,
}

impl ThemeTokens {
    /// Return the window background color with the requested window opacity (0.0 to 1.0)
    pub fn bg_with_opacity(&self, opacity: f32) -> Hsla {
        let mut c = self.bg;
        let clamped = opacity.clamp(0.1, 1.0);
        // En Windows (DWM Acrylic/Mica), el compositor suma su propio tinte de fondo,
        // por lo que una curva perceptual hace más notable la transparencia en 60%-80%.
        #[cfg(target_os = "windows")]
        let effective = if clamped >= 0.99 {
            1.0
        } else {
            clamped.powf(1.65).clamp(0.08, 1.0)
        };
        #[cfg(not(target_os = "windows"))]
        let effective = clamped;
        c.a = effective;
        c
    }
}
