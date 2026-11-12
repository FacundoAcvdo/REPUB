use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationSpeed {
    Slow,
    Standard,
    Fast,
}

impl AnimationSpeed {
    pub fn name_es(&self) -> &'static str {
        match self {
            AnimationSpeed::Slow => "Lenta (250 ms)",
            AnimationSpeed::Standard => "Estándar (150 ms)",
            AnimationSpeed::Fast => "Rápida (80 ms)",
        }
    }

    pub fn all() -> &'static [AnimationSpeed] {
        &[AnimationSpeed::Slow, AnimationSpeed::Standard, AnimationSpeed::Fast]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReduceMotion {
    System,
    Always,
    Never,
}

impl ReduceMotion {
    pub fn name_es(&self) -> &'static str {
        match self {
            ReduceMotion::System => "Sistema",
            ReduceMotion::Always => "Siempre",
            ReduceMotion::Never => "Nunca",
        }
    }

    pub fn all() -> &'static [ReduceMotion] {
        &[ReduceMotion::System, ReduceMotion::Always, ReduceMotion::Never]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CornerRadiusStyle {
    Square,
    Subtle,
    Rounded,
    Circular,
}

impl CornerRadiusStyle {
    pub fn name_es(&self) -> &'static str {
        match self {
            CornerRadiusStyle::Square => "Rectas",
            CornerRadiusStyle::Subtle => "Sutiles",
            CornerRadiusStyle::Rounded => "Redondeadas",
            CornerRadiusStyle::Circular => "Circulares",
        }
    }

    pub fn all() -> &'static [CornerRadiusStyle] {
        &[
            CornerRadiusStyle::Square,
            CornerRadiusStyle::Subtle,
            CornerRadiusStyle::Rounded,
            CornerRadiusStyle::Circular,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub language: String,
    pub home_view: String,
    pub reduce_motion: ReduceMotion,
    pub animation_speed: AnimationSpeed,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            language: "Español".into(),
            home_view: "Inicio".into(),
            reduce_motion: ReduceMotion::System,
            animation_speed: AnimationSpeed::Standard,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppearanceSettings {
    pub theme_id: String,
    pub corner_radius: CornerRadiusStyle,
    pub font_family: String,
    pub font_size: u32,
    pub window_opacity: f32,
    pub interface_blur: bool,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme_id: "dark".into(),
            corner_radius: CornerRadiusStyle::Rounded,
            font_family: "Inter".into(),
            font_size: 14,
            window_opacity: 1.0,
            interface_blur: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiSettings {
    pub provider: String,
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub privacy_accepted: bool,
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            provider: "OpenAI".into(),
            api_key: "sk-••••••••••••••••••••••••".into(),
            model: "gpt-4o-mini".into(),
            base_url: "http://localhost:11434".into(),
            privacy_accepted: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReaderSheetSettings {
    pub font_family: String,
    pub font_size: u32,
    pub line_height: f32,
    pub side_margins: u32,
    pub max_line_width: u32,
    pub justify_text: bool,
    pub follow_app_theme: bool,
    pub page_transition: String,
    pub show_progress: bool,
    pub click_zones_enabled: bool,
}

impl Default for ReaderSheetSettings {
    fn default() -> Self {
        Self {
            font_family: "Serif (Georgia)".into(),
            font_size: 18,
            line_height: 1.6,
            side_margins: 48,
            max_line_width: 680,
            justify_text: true,
            follow_app_theme: true,
            page_transition: "Desvanecer".into(),
            show_progress: true,
            click_zones_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    pub general: GeneralSettings,
    pub appearance: AppearanceSettings,
    pub ai: AiSettings,
    pub reader: ReaderSheetSettings,
}
