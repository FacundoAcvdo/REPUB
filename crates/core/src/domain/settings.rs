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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EmbeddingSource {
    #[default]
    Unconfigured,
    Local,
    Cloud,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalEmbeddingModelInfo {
    pub id: &'static str,
    pub display_name: &'static str,
    pub dimensions: usize,
    pub approx_size_label: &'static str,
    pub languages: &'static str,
    pub description: &'static str,
    pub recommended: bool,
}

impl LocalEmbeddingModelInfo {
    pub fn all() -> &'static [LocalEmbeddingModelInfo] {
        &[
            LocalEmbeddingModelInfo {
                id: "multilingual-e5-small",
                display_name: "multilingual-e5-small",
                dimensions: 384,
                approx_size_label: "~120 MB",
                languages: "Español, Inglés (+90 idiomas)",
                description: "Mejor equilibrio entre velocidad en CPU, tamaño y precisión literaria.",
                recommended: true,
            },
            LocalEmbeddingModelInfo {
                id: "multilingual-e5-base",
                display_name: "multilingual-e5-base",
                dimensions: 768,
                approx_size_label: "~450 MB",
                languages: "Español, Inglés (+90 idiomas)",
                description: "Mayor riqueza semántica en novelas largas; se beneficia de GPU o buena RAM.",
                recommended: false,
            },
            LocalEmbeddingModelInfo {
                id: "multilingual-e5-large",
                display_name: "multilingual-e5-large",
                dimensions: 1024,
                approx_size_label: "~2.2 GB",
                languages: "Multilingüe (+100 idiomas)",
                description: "Máxima calidad para textos complejos y contexto largo; mayor uso de disco/GPU.",
                recommended: false,
            },
            LocalEmbeddingModelInfo {
                id: "all-MiniLM-L6-v2",
                display_name: "all-MiniLM-L6-v2",
                dimensions: 384,
                approx_size_label: "~90 MB",
                languages: "Principalmente Inglés",
                description: "Opción ultraliviana y rápida si lees principalmente en inglés.",
                recommended: false,
            },
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiSettings {
    pub provider: String,
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub privacy_accepted: bool,
    #[serde(default)]
    pub embedding_source: EmbeddingSource,
    #[serde(default)]
    pub local_embedding_model: Option<String>,
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            provider: "OpenAI".into(),
            api_key: String::new(),
            model: "gpt-4o-mini".into(),
            base_url: String::new(),
            privacy_accepted: true,
            embedding_source: EmbeddingSource::Unconfigured,
            local_embedding_model: None,
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
