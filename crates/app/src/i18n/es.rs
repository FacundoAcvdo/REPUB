pub struct I18n;

impl I18n {
    // App info
    pub const APP_TITLE: &'static str = "EPUB Reader";

    // Nav
    pub const NAV_HOME: &'static str = "Inicio";
    pub const NAV_LIBRARY: &'static str = "Biblioteca";
    pub const NAV_SETTINGS: &'static str = "Ajustes";
    pub const NAV_GALLERY: &'static str = "Galería (Dev)";

    // Gallery view
    pub const GALLERY_TITLE: &'static str = "Galería de Componentes y Tokens de Diseño";
    pub const GALLERY_SUBTITLE: &'static str = "Entorno de auditoría interactivo para verificar tokens semánticos, temas y controles.";
    pub const GALLERY_THEMES: &'static str = "PALETAS Y TEMAS PREESTABLECIDOS";
    pub const GALLERY_CORNERS: &'static str = "ESQUINAS Y RADIOS DE CURVATURA";
    pub const GALLERY_CONTROLS: &'static str = "CONTROLES INTERACTIVOS Y BOTONES";
    pub const GALLERY_WINDOW: &'static str = "VENTANA Y TRANSPARENCIA";
    pub const GALLERY_TYPOGRAPHY: &'static str = "JERARQUÍA TIPOGRÁFICA Y LECTURA";
    pub const GALLERY_TOKENS_SWATCH: &'static str = "MUESTRARIO DE TOKENS SEMÁNTICOS";

    // Corner styles
    pub const CORNER_SQUARE: &'static str = "Rectas";
    pub const CORNER_SUBTLE: &'static str = "Sutiles";
    pub const CORNER_ROUNDED: &'static str = "Redondeadas";
    pub const CORNER_CIRCULAR: &'static str = "Circulares";

    // Common buttons
    pub const BTN_PRIMARY: &'static str = "Botón Principal";
    pub const BTN_OUTLINE: &'static str = "Botón Contorno";
    pub const BTN_GHOST: &'static str = "Botón Fantasma";
    pub const BTN_DANGER: &'static str = "Acción Destructiva";
    pub const BTN_DISABLED: &'static str = "Deshabilitado";

    // Window controls
    pub const WINDOW_OPACITY: &'static str = "Opacidad de la ventana";
    pub const WINDOW_BLUR: &'static str = "Desenfoque de la interfaz";

    // Hints
    pub const HINT_GALLERY_TOGGLE: &'static str = "Presiona Ctrl+Shift+G para alternar la Galería";
}
