use std::rc::Rc;
use std::time::Instant;
use epub_core::db::{BookRepository, Database};
use epub_core::domain::book::Book;
use epub_core::epub::{PaginatedBook, PaginationConfig};
use epub_core::mock::books::mock_books;
use epub_core::services::ReaderService;
use gpui_kit::assets::IconName;
use gpui_kit::base::StyledExt as _;
use gpui_kit::component::Icon;
use gpui_kit::*;

use crate::theme::reader_settings::ReaderSettingsState;
use crate::theme::state::ThemeState;

pub struct ReaderView {
    book: Option<Book>,
    current_page: u32,
    total_pages: u32,
    zoom_level: f32,
    last_activity_at: Instant,
    focus_handle: FocusHandle,
    on_close: Option<Rc<dyn Fn(&mut Window, &mut App) + 'static>>,
}

impl ReaderView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let default_book = mock_books().into_iter().next();
        let (current_page, total_pages) = match &default_book {
            Some(b) => (b.current_page.max(1), b.total_pages),
            None => (1, 100),
        };

        Self {
            book: default_book,
            current_page,
            total_pages,
            zoom_level: 1.0,
            last_activity_at: Instant::now(),
            focus_handle: cx.focus_handle(),
            on_close: None,
        }
    }

    pub fn load_book(&mut self, mut book: Book, cx: &mut Context<Self>) {
        if book.chapters.is_empty() {
            let reader_settings = &ReaderSettingsState::global(cx).settings;
            let config = PaginationConfig::from_reader_settings(
                reader_settings.font_size,
                reader_settings.line_height,
                reader_settings.max_line_width,
            );

            if let Some(db_path) = Database::default_db_path() {
                if db_path.exists() {
                    if let Ok(db) = Database::open(&db_path) {
                        if let Ok(Some(stored)) = BookRepository::get_by_id(db.conn(), &book.id) {
                            if let Some(file_path) = stored.file_path {
                                if let Ok(paginated) =
                                    ReaderService::load_book_pages(&file_path, config)
                                {
                                    book.chapters = paginated.chapters;
                                    book.total_pages = paginated.total_pages.max(1);
                                }
                            }
                        }
                    }
                }
            }
        }

        self.total_pages = book.total_pages.max(1);
        self.current_page = book.current_page.max(1).min(self.total_pages);
        self.last_activity_at = Instant::now();
        self.book = Some(book);
        cx.notify();
    }

    fn persist_progress(&mut self, pages_delta: u32) {
        let elapsed_secs = self.last_activity_at.elapsed().as_secs();
        self.last_activity_at = Instant::now();

        let Some(ref book) = self.book else {
            return;
        };

        let chapter_idx = if !book.chapters.is_empty() {
            PaginatedBook::from_chapters(book.chapters.clone())
                .chapter_index_for_page(self.current_page)
                .unwrap_or(0) as u32
        } else {
            let pages_per_chapter = 25.max(self.total_pages / 12);
            self.current_page.saturating_sub(1) / pages_per_chapter
        };

        if let Some(db_path) = Database::default_db_path() {
            if let Ok(mut db) = Database::open(&db_path) {
                let _ = ReaderService::record_reading_activity(
                    &mut db,
                    book,
                    self.current_page,
                    chapter_idx,
                    pages_delta,
                    elapsed_secs,
                );
            }
        }
    }

    pub fn on_close(
        mut self,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_close = Some(Rc::new(handler));
        self
    }

    pub fn set_on_close(&mut self, handler: impl Fn(&mut Window, &mut App) + 'static) {
        self.on_close = Some(Rc::new(handler));
    }

    pub fn prev_page(&mut self, cx: &mut Context<Self>) {
        if self.current_page > 1 {
            self.current_page -= 1;
            if let Some(ref mut b) = self.book {
                b.current_page = self.current_page;
            }
            self.persist_progress(0);
            cx.notify();
        }
    }

    pub fn next_page(&mut self, cx: &mut Context<Self>) {
        if self.current_page < self.total_pages {
            self.current_page += 1;
            if let Some(ref mut b) = self.book {
                b.current_page = self.current_page;
            }
            self.persist_progress(1);
            cx.notify();
        }
    }

    pub fn zoom_in(&mut self, cx: &mut Context<Self>) {
        if self.zoom_level < 2.2 {
            self.zoom_level = ((self.zoom_level + 0.1) * 10.0).round() / 10.0;
            cx.notify();
        }
    }

    pub fn zoom_out(&mut self, cx: &mut Context<Self>) {
        if self.zoom_level > 0.6 {
            self.zoom_level = ((self.zoom_level - 0.1) * 10.0).round() / 10.0;
            cx.notify();
        }
    }

    pub fn zoom_reset(&mut self, cx: &mut Context<Self>) {
        self.zoom_level = 1.0;
        cx.notify();
    }

    pub fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.persist_progress(0);
        if let Some(ref handler) = self.on_close {
            let cb = handler.clone();
            cb(window, cx);
        }
    }

    pub fn focus(&self, window: &mut Window, cx: &mut App) {
        self.focus_handle.focus(window, cx);
    }
}

impl Render for ReaderView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme_state = ThemeState::global(cx);
        let tokens = *theme_state.tokens();
        let radius = theme_state.radius();

        let reader_state = ReaderSettingsState::global(cx);
        let reader_settings = &reader_state.settings;

        let book_title = self
            .book
            .as_ref()
            .map(|b| b.title.clone())
            .unwrap_or_else(|| "Sin Título".into());
        let book_author = self
            .book
            .as_ref()
            .map(|b| b.author.clone())
            .unwrap_or_else(|| "Autor Desconocido".into());

        let current_page = self.current_page;
        let total_pages = self.total_pages;
        let progress_pct = if total_pages > 0 {
            (current_page as f32 / total_pages as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // Contenido de la página actual (capítulos reales paginados o fallback mock)
        let page_content = match &self.book {
            Some(book) => ReaderService::resolve_page(book, current_page),
            None => ReaderService::resolve_page(
                &Book {
                    id: String::new(),
                    title: book_title.clone(),
                    author: book_author.clone(),
                    total_pages,
                    current_page,
                    genre: epub_core::domain::book::Genre::Fiction,
                    cover_palette: epub_core::domain::book::CoverPalette {
                        primary: 0,
                        secondary: 0,
                        accent: 0,
                        text: 0,
                    },
                    synopsis: String::new(),
                    chapters: Vec::new(),
                },
                current_page,
            ),
        };

        // Tipografía y apariencia según ajustes de Hoja y nivel de Zoom interactivo
        let zoom = self.zoom_level;
        let font_family = reader_state.resolved_font_family();
        let base_font_size = reader_settings.font_size as f32;
        let effective_font_size = (base_font_size * zoom).round();
        let effective_chapter_title_size = (24.0 * zoom).round();
        let effective_gap = (20.0 * zoom).round();

        let line_height = reader_settings.line_height;
        let base_max_width = (reader_settings.max_line_width as f32).max(480.0);
        let effective_max_line_width = (base_max_width * (1.0 + (zoom - 1.0) * 0.75)).round();

        let follow_theme = reader_settings.follow_app_theme;

        let text_color = if follow_theme {
            tokens.text
        } else {
            // Hoja clásica con tinta profunda
            hsla(30.0 / 360.0, 0.18, 0.16, 1.0)
        };

        let sheet_bg = if follow_theme {
            tokens.bg
        } else {
            // Hoja cálida color marfil / papel libro
            hsla(40.0 / 360.0, 0.25, 0.95, 1.0)
        };

        let view_prev = cx.entity().clone();
        let view_next = cx.entity().clone();
        let view_hud_back = cx.entity().clone();
        let view_font_dec = cx.entity().clone();
        let view_font_inc = cx.entity().clone();
        let view_zoom_out = cx.entity().clone();
        let view_zoom_in = cx.entity().clone();
        let view_zoom_reset = cx.entity().clone();

        div()
            .id("reader-container")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                let key = event.keystroke.key.to_lowercase();
                let is_ctrl = event.keystroke.modifiers.control || event.keystroke.modifiers.platform;

                if is_ctrl && (key == "+" || key == "=") {
                    this.zoom_in(cx);
                } else if is_ctrl && (key == "-" || key == "_") {
                    this.zoom_out(cx);
                } else if is_ctrl && key == "0" {
                    this.zoom_reset(cx);
                } else if key == "escape" {
                    this.close(window, cx);
                } else if key == "left" || key == "arrowleft" || key == "pageup" || (key == " " && event.keystroke.modifiers.shift) {
                    this.prev_page(cx);
                } else if key == "right" || key == "arrowright" || key == "pagedown" || (key == " " && !event.keystroke.modifiers.shift) {
                    this.next_page(cx);
                }
            }))
            .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, window, cx| {
                if window.modifiers().control || window.modifiers().platform {
                    let is_up = match event.delta {
                        ScrollDelta::Pixels(p) => p.y > px(0.0),
                        ScrollDelta::Lines(l) => l.y > 0.0,
                    };
                    let is_down = match event.delta {
                        ScrollDelta::Pixels(p) => p.y < px(0.0),
                        ScrollDelta::Lines(l) => l.y < 0.0,
                    };
                    if is_up {
                        this.zoom_in(cx);
                    } else if is_down {
                        this.zoom_out(cx);
                    }
                }
            }))
            .flex()
            .flex_col()
            .size_full()
            .bg(sheet_bg)
            .relative()
            .overflow_hidden()
            // 1. Barra Superior Delgada (Siempre visible)
            .child(
                div()
                    .id("reader-topbar-slim")
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .h(px(36.0))
                    .px(px(14.0))
                    .w_full()
                    .bg(tokens.bg_elevated)
                    .border_b_1()
                    .border_color(tokens.border.opacity(0.6))
                    // Lado izquierdo: Botón Volver compacto
                    .child(
                        div()
                            .id("btn-reader-back")
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(6.0))
                            .px(px(8.0))
                            .py(px(3.0))
                            .rounded(radius)
                            .cursor_pointer()
                            .hover(|s| s.bg(tokens.bg_hover))
                            .on_click(move |_, window, cx| {
                                view_hud_back.update(cx, |this, cx| {
                                    this.close(window, cx);
                                });
                            })
                            .child(
                                Icon::new(IconName::ChevronLeft)
                                    .size(px(14.0))
                                    .text_color(tokens.accent),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .font_medium()
                                    .text_color(tokens.text)
                                    .child("Biblioteca"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(tokens.text_muted)
                                    .child("(Esc)"),
                            ),
                    )
                    // Centro: Título y capítulo en una sola línea compacta
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(6.0))
                            .child(
                                div()
                                    .text_xs()
                                    .font_semibold()
                                    .text_color(tokens.text)
                                    .child(format!("{} — {}", book_title, book_author)),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(tokens.text_muted.opacity(0.6))
                                    .child("•"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(tokens.text_muted)
                                    .child(page_content.chapter_title.clone()),
                            ),
                    )
                    // Lado derecho: Zoom de hoja, tamaño de letra y porcentaje
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(8.0))
                            // Control de Zoom de la Hoja
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(1.0))
                                    .px(px(4.0))
                                    .py(px(2.0))
                                    .rounded(radius)
                                    .bg(tokens.bg_sunken)
                                    .border_1()
                                    .border_color(tokens.border)
                                    // Zoom Out
                                    .child(
                                        div()
                                            .id("btn-zoom-out")
                                            .p(px(2.0))
                                            .rounded(px(3.0))
                                            .cursor_pointer()
                                            .hover(|s| s.bg(tokens.bg_hover))
                                            .on_click(move |_, _, cx| {
                                                view_zoom_out.update(cx, |this, cx| this.zoom_out(cx));
                                            })
                                            .child(
                                                Icon::new(IconName::Minus)
                                                    .size(px(12.0))
                                                    .text_color(tokens.text_muted),
                                            ),
                                    )
                                    // Zoom Label (click para restablecer 100%)
                                    .child(
                                        div()
                                            .id("btn-zoom-reset")
                                            .px(px(4.0))
                                            .py(px(1.0))
                                            .rounded(px(3.0))
                                            .cursor_pointer()
                                            .hover(|s| s.bg(tokens.bg_hover))
                                            .on_click(move |_, _, cx| {
                                                view_zoom_reset.update(cx, |this, cx| this.zoom_reset(cx));
                                            })
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_medium()
                                                    .text_color(tokens.text)
                                                    .child(format!("{:.0}%", zoom * 100.0)),
                                            ),
                                    )
                                    // Zoom In
                                    .child(
                                        div()
                                            .id("btn-zoom-in")
                                            .p(px(2.0))
                                            .rounded(px(3.0))
                                            .cursor_pointer()
                                            .hover(|s| s.bg(tokens.bg_hover))
                                            .on_click(move |_, _, cx| {
                                                view_zoom_in.update(cx, |this, cx| this.zoom_in(cx));
                                            })
                                            .child(
                                                Icon::new(IconName::Plus)
                                                    .size(px(12.0))
                                                    .text_color(tokens.text_muted),
                                            ),
                                    ),
                            )
                            // Ajuste fino de Fuente A- / A+
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(1.0))
                                    .px(px(4.0))
                                    .py(px(2.0))
                                    .rounded(radius)
                                    .bg(tokens.bg_sunken)
                                    .border_1()
                                    .border_color(tokens.border)
                                    // Reducir tamaño A-
                                    .child(
                                        div()
                                            .id("btn-quick-font-dec")
                                            .px(px(4.0))
                                            .py(px(1.0))
                                            .rounded(px(3.0))
                                            .cursor_pointer()
                                            .hover(|s| s.bg(tokens.bg_hover))
                                            .on_click(move |_, _, cx| {
                                                view_font_dec.update(cx, |_, cx| {
                                                    let mut s = ReaderSettingsState::global(cx).settings.clone();
                                                    if s.font_size > 6 {
                                                        s.font_size -= 1;
                                                        ReaderSettingsState::update(cx, s);
                                                        cx.notify();
                                                    }
                                                });
                                            })
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_bold()
                                                    .text_color(tokens.text_muted)
                                                    .child("A-"),
                                            ),
                                    )
                                    // Indicador de tamaño en px
                                    .child(
                                        div()
                                            .px(px(3.0))
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_medium()
                                                    .text_color(tokens.text)
                                                    .child(format!("{}px", base_font_size as u32)),
                                            ),
                                    )
                                    // Aumentar tamaño A+
                                    .child(
                                        div()
                                            .id("btn-quick-font-inc")
                                            .px(px(4.0))
                                            .py(px(1.0))
                                            .rounded(px(3.0))
                                            .cursor_pointer()
                                            .hover(|s| s.bg(tokens.bg_hover))
                                            .on_click(move |_, _, cx| {
                                                view_font_inc.update(cx, |_, cx| {
                                                    let mut s = ReaderSettingsState::global(cx).settings.clone();
                                                    if s.font_size < 36 {
                                                        s.font_size += 1;
                                                        ReaderSettingsState::update(cx, s);
                                                        cx.notify();
                                                    }
                                                });
                                            })
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_bold()
                                                    .text_color(tokens.text_muted)
                                                    .child("A+"),
                                            ),
                                    ),
                            )
                            // Porcentaje de progreso
                            .child(
                                div()
                                    .text_xs()
                                    .font_medium()
                                    .text_color(tokens.accent)
                                    .child(format!("{:.0}%", progress_pct * 100.0)),
                            ),
                    ),
            )
            // 2. Área de Lectura y Capas Laterales de Navegación
            .child(
                div()
                    .flex()
                    .flex_1()
                    .w_full()
                    .h_full()
                    .relative()
                    .overflow_hidden()
                    // Hoja de Lectura Centrada
                    .child(
                        div()
                            .id("reader-sheet-content")
                            .flex()
                            .flex_col()
                            .size_full()
                            .overflow_y_scroll()
                            .pt(px(32.0))
                            .pb(px(56.0))
                            .px(px(24.0))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .w_full()
                                    .max_w(px(effective_max_line_width))
                                    .mx_auto()
                                    .font_family(font_family)
                                    .child(
                                        // Título del Capítulo
                                        div()
                                            .flex()
                                            .flex_col()
                                            .items_center()
                                            .text_center()
                                            .mb(px(32.0 * zoom))
                                            .gap(px(6.0))
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(tokens.text_muted)
                                                    .font_family("Inter")
                                                    .child(book_title),
                                            )
                                            .child(
                                                div()
                                                    .text_size(px(effective_chapter_title_size))
                                                    .font_bold()
                                                    .text_color(text_color)
                                                    .child(page_content.chapter_title.clone()),
                                            )
                                            .child(
                                                div()
                                                    .w(px(36.0 * zoom))
                                                    .h(px(2.0))
                                                    .rounded_full()
                                                    .bg(tokens.accent.opacity(0.4))
                                                    .mt(px(4.0)),
                                            ),
                                    )
                                    // Párrafos del texto ficticio
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap(px(effective_gap))
                                            .text_size(px(effective_font_size))
                                            .line_height(relative(line_height))
                                            .text_color(text_color)
                                            .children(page_content.paragraphs.into_iter().map(|p| {
                                                div()
                                                    .child(p)
                                            })),
                                    ),
                            ),
                    )
                    // Zonas de Clic Laterales Transparentes (Izquierda y Derecha para pasar página sin obstruir el centro)
                    .child(
                        div()
                            .id("reader-side-nav-left")
                            .absolute()
                            .top_0()
                            .left_0()
                            .bottom_0()
                            .w(relative(0.20))
                            .cursor_pointer()
                            .flex()
                            .items_center()
                            .justify_start()
                            .pl(px(16.0))
                            .group("zone_prev")
                            .on_click(move |_, _, cx| {
                                view_prev.update(cx, |this, cx| this.prev_page(cx));
                            })
                            .child(
                                div()
                                    .p(px(8.0))
                                    .rounded_full()
                                    .bg(tokens.bg_elevated.opacity(0.85))
                                    .border_1()
                                    .border_color(tokens.border)
                                    .opacity(0.0)
                                    .group_hover("zone_prev", |s| s.opacity(0.75))
                                    .child(
                                        Icon::new(IconName::ChevronLeft)
                                            .size(px(18.0))
                                            .text_color(tokens.text),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .id("reader-side-nav-right")
                            .absolute()
                            .top_0()
                            .right_0()
                            .bottom_0()
                            .w(relative(0.20))
                            .cursor_pointer()
                            .flex()
                            .items_center()
                            .justify_end()
                            .pr(px(16.0))
                            .group("zone_next")
                            .on_click(move |_, _, cx| {
                                view_next.update(cx, |this, cx| this.next_page(cx));
                            })
                            .child(
                                div()
                                    .p(px(8.0))
                                    .rounded_full()
                                    .bg(tokens.bg_elevated.opacity(0.85))
                                    .border_1()
                                    .border_color(tokens.border)
                                    .opacity(0.0)
                                    .group_hover("zone_next", |s| s.opacity(0.75))
                                    .child(
                                        Icon::new(IconName::ChevronRight)
                                            .size(px(18.0))
                                            .text_color(tokens.text),
                                    ),
                            ),
                    ),
            )
            // 3. Footer Ultra-Discreto: Número de página, atajos y Barra de Progreso fina de 1.5 px
            .child(
                div()
                    .id("reader-discrete-footer")
                    .flex()
                    .flex_col()
                    .w_full()
                    .bg(sheet_bg)
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .w_full()
                            .px(px(24.0))
                            .py(px(5.0))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(tokens.text_muted)
                                    .child(format!("pág. {} / {}", current_page, total_pages)),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(tokens.text_muted)
                                    .child("← → / Espacio: Navegar  •  Ctrl +/-: Zoom  •  Esc: Salir"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .font_medium()
                                    .text_color(tokens.accent)
                                    .child(format!("{:.0}%", progress_pct * 100.0)),
                            ),
                    )
                    // Barra de progreso ultra-delgada (1.5 px) en la base inferior
                    .child(
                        div()
                            .w_full()
                            .h(px(1.5))
                            .bg(tokens.border.opacity(0.4))
                            .child(
                                div()
                                    .h_full()
                                    .w(relative(progress_pct))
                                    .bg(tokens.accent),
                            ),
                    ),
            )
    }
}
