pub struct I18n;

impl I18n {
    // App info
    pub const APP_TITLE: &'static str = "EPUB Reader";

    // Nav
    pub const NAV_HOME: &'static str = "Inicio";
    pub const NAV_LIBRARY: &'static str = "Biblioteca";
    pub const NAV_SETTINGS: &'static str = "Ajustes";
    pub const NAV_GALLERY: &'static str = "Galería de Componentes";
    pub const NAV_READER: &'static str = "Lector";

    // Home view
    pub const HOME_TITLE: &'static str = "Inicio";
    pub const HOME_SUBTITLE: &'static str = "Bienvenido de nuevo. Tu biblioteca personal y métricas de lectura al día.";
    pub const HOME_CONTINUE_READING: &'static str = "CONTINUAR LEYENDO";
    pub const HOME_RESUME_BTN: &'static str = "Reanudar lectura";
    pub const HOME_STATS_TITLE: &'static str = "ESTADÍSTICAS GENERALES";
    pub const HOME_ACTIVITY_TITLE: &'static str = "ACTIVIDAD SEMANAL";
    pub const HOME_DAYS: [&'static str; 7] = ["Lun", "Mar", "Mié", "Jue", "Vie", "Sáb", "Dom"];

    // Library view
    pub const LIBRARY_TITLE: &'static str = "Biblioteca";
    pub const LIBRARY_SEARCH_PLACEHOLDER: &'static str = "Buscar por título, autor o género...";
    pub const LIBRARY_IMPORT_HINT: &'static str = "Importar EPUB estará disponible en la siguiente fase.";
    pub const LIBRARY_EMPTY: &'static str = "No se encontraron libros que coincidan con la búsqueda.";
    pub const LIBRARY_RESET_FILTERS: &'static str = "Restablecer filtros";

    // Reader view
    pub const READER_BACK: &'static str = "Biblioteca";
    pub const READER_BACK_HINT: &'static str = "(Esc)";
    pub const READER_ZOOM_RESET_HINT: &'static str = "Restablecer al 100%";
    pub const READER_NAV_HINT: &'static str = "← → / Espacio: Navegar  •  Ctrl +/-: Zoom  •  Esc: Salir";

    // Settings tabs
    pub const TAB_GENERAL: &'static str = "General";
    pub const TAB_APPEARANCE: &'static str = "Apariencia";
    pub const TAB_AI: &'static str = "Proveedores (IA)";
    pub const TAB_SHEET: &'static str = "Hoja";

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

    // Shortcuts Hints
    pub const HINT_GALLERY_TOGGLE: &'static str = "Presiona Ctrl+Shift+G para alternar la Galería";
    pub const HINT_SIDEBAR_TOGGLE: &'static str = "Presiona Ctrl+B para alternar la barra lateral";
    pub const HINT_SEARCH: &'static str = "Presiona Ctrl+F para buscar libros";
}
