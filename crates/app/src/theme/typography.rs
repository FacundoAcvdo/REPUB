use gpui_kit::{Pixels, px};

pub struct TypographyScale;

impl TypographyScale {
    /// 11px - Micro headers en mayúsculas (ej. INICIO, VENTANA, ESQUINAS)
    pub const MICRO: Pixels = px(11.0);
    /// 12px - Textos secundarios y leyendas
    pub const CAPTION: Pixels = px(12.0);
    /// 14px - Texto base de la interfaz (botones, etiquetas, filas)
    pub const BASE: Pixels = px(14.0);
    /// 16px - Títulos de tarjetas y subtítulos
    pub const SUBTITLE: Pixels = px(16.0);
    /// 20px - Encabezados de sección destacados
    pub const TITLE_SM: Pixels = px(20.0);
    /// 26px - Título principal de vistas (Ajustes, Biblioteca, Inicio)
    pub const TITLE_LG: Pixels = px(26.0);
}

pub struct ReaderTypography;

impl ReaderTypography {
    pub const DEFAULT_FONT_SIZE: Pixels = px(18.0);
    pub const DEFAULT_LINE_HEIGHT: f32 = 1.65;
    pub const MAX_COLUMN_WIDTH: Pixels = px(680.0);
    pub const SERIF_FALLBACK: &'static str = "Georgia, 'DejaVu Serif', 'Times New Roman', serif";
}
