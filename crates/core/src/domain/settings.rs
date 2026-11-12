use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationSpeed {
    Slow,
    Standard,
    Fast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReduceMotion {
    System,
    Always,
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CornerRadiusStyle {
    Square,
    Subtle,
    Rounded,
    Circular,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub language: String,
    pub home_view: String,
    pub reduce_motion: ReduceMotion,
    pub animation_speed: AnimationSpeed,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiSettings {
    pub provider: String,
    pub api_key: String,
    pub model: String,
    pub base_url: Option<String>,
    pub privacy_accepted: bool,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    pub general: GeneralSettings,
    pub appearance: AppearanceSettings,
    pub ai: AiSettings,
    pub reader: ReaderSheetSettings,
}
