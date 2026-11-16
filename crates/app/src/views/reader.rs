use std::rc::Rc;
use std::time::Instant;
use epub_core::db::{
    BookRepository, BookmarkRecord, BookmarkRepository, Database, HighlightRecord,
    HighlightRepository, SettingsRepository,
};
use epub_core::domain::book::Book;
use epub_core::epub::{PaginatedBook, PaginationConfig};
use epub_core::services::{AssistantAnswer, RagService, ReaderService};
use gpui_kit::assets::IconName;
use gpui_kit::base::StyledExt as _;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::Icon;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::theme::reader_settings::ReaderSettingsState;
use crate::theme::state::ThemeState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReaderPanelTab {
    Toc,
    Bookmarks,
    Highlights,
}

pub struct ReaderView {
    book: Option<Book>,
    current_page: u32,
    total_pages: u32,
    zoom_level: f32,
    is_panel_open: bool,
    panel_tab: ReaderPanelTab,
    bookmarks: Vec<BookmarkRecord>,
    note_input: Entity<InputState>,
    highlights: Vec<HighlightRecord>,
    highlight_note_input: Entity<InputState>,
    highlight_edit_note_input: Entity<InputState>,
    editing_highlight_id: Option<i64>,
    selected_highlight_color: String,
    is_quote_popover_open: bool,
    ai_enabled_for_book: bool,
    is_ai_drawer_open: bool,
    ai_drawer_width: f32,
    is_resizing_ai_drawer: bool,
    ai_resize_start_x: f32,
    ai_resize_start_width: f32,
    ai_question_input: Entity<InputState>,
    ai_selected_quote: Option<String>,
    ai_answer: Option<AssistantAnswer>,
    ai_error: Option<String>,
    is_ai_loading: bool,
    current_paragraphs: Vec<String>,
    paragraph_layouts: Vec<TextLayout>,
    drag_anchor: Option<(usize, usize)>,
    selection_range: Option<(usize, usize, usize)>,
    is_dragging_text: bool,
    tooltip_pos: Option<Point<Pixels>>,
    last_activity_at: Instant,
    focus_handle: FocusHandle,
    on_close: Option<Rc<dyn Fn(&mut Window, &mut App) + 'static>>,
}

impl ReaderView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let default_book: Option<Book> = None;
        let (current_page, total_pages) = (1, 1);
        let note_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Nota opcional para esta página...")
        });
        let highlight_note_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Nota opcional para esta cita...")
        });
        let highlight_edit_note_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Editar nota de la cita...")
        });
        let ai_question_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Pregunta sobre lo leído hasta aquí...")
        });

        Self {
            book: default_book,
            current_page,
            total_pages,
            zoom_level: 1.0,
            is_panel_open: false,
            panel_tab: ReaderPanelTab::Toc,
            bookmarks: Vec::new(),
            note_input,
            highlights: Vec::new(),
            highlight_note_input,
            highlight_edit_note_input,
            editing_highlight_id: None,
            selected_highlight_color: "amber".to_string(),
            is_quote_popover_open: false,
            ai_enabled_for_book: true,
            is_ai_drawer_open: false,
            ai_drawer_width: 380.0,
            is_resizing_ai_drawer: false,
            ai_resize_start_x: 0.0,
            ai_resize_start_width: 380.0,
            ai_question_input,
            ai_selected_quote: None,
            ai_answer: None,
            ai_error: None,
            is_ai_loading: false,
            current_paragraphs: Vec::new(),
            paragraph_layouts: Vec::new(),
            drag_anchor: None,
            selection_range: None,
            is_dragging_text: false,
            tooltip_pos: None,
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
                                    let new_total = paginated.total_pages.max(1);
                                    if new_total != book.total_pages {
                                        book.total_pages = new_total;
                                        let _ = BookRepository::update_total_pages(
                                            db.conn(),
                                            &book.id,
                                            new_total,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut ai_enabled = true;
        if let Some(db_path) = Database::default_db_path() {
            if db_path.exists() {
                if let Ok(db) = Database::open(&db_path) {
                    ai_enabled = BookRepository::is_ai_enabled(db.conn(), &book.id).unwrap_or(true);
                }
            }
        }

        self.total_pages = book.total_pages.max(1);
        self.current_page = book.current_page.max(1).min(self.total_pages);
        self.ai_enabled_for_book = ai_enabled;
        if !ai_enabled {
            self.is_ai_drawer_open = false;
        }
        self.ai_selected_quote = None;
        self.ai_answer = None;
        self.ai_error = None;
        self.selection_range = None;
        self.tooltip_pos = None;
        self.drag_anchor = None;
        self.is_quote_popover_open = false;
        self.editing_highlight_id = None;
        self.last_activity_at = Instant::now();
        self.book = Some(book);
        self.reload_bookmarks();
        self.reload_highlights();
        cx.notify();
    }

    fn reload_highlights(&mut self) {
        self.highlights.clear();
        let Some(ref book) = self.book else {
            return;
        };
        if let Some(db_path) = Database::default_db_path() {
            if db_path.exists() {
                if let Ok(db) = Database::open(&db_path) {
                    if let Ok(list) = HighlightRepository::list_for_book(db.conn(), &book.id) {
                        self.highlights = list;
                    }
                }
            }
        }
    }

    pub fn save_selection_as_highlight(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some((p_idx, s, e)) = self.selection_range else {
            return;
        };
        let Some(para) = self.current_paragraphs.get(p_idx) else {
            return;
        };
        if s >= e
            || e > para.len()
            || !para.is_char_boundary(s)
            || !para.is_char_boundary(e)
        {
            return;
        }
        let selected_text = para[s..e].trim().to_string();
        if selected_text.is_empty() {
            return;
        }
        let Some(ref book) = self.book else {
            return;
        };
        let note = self.highlight_note_input.read(cx).value().to_string();
        let color = self.selected_highlight_color.clone();
        let page = self.current_page;

        if let Some(db_path) = Database::default_db_path() {
            if let Ok(db) = Database::open(&db_path) {
                let _ = HighlightRepository::create(
                    db.conn(),
                    &book.id,
                    page,
                    p_idx,
                    s,
                    e,
                    &selected_text,
                    &note,
                    &color,
                );
            }
        }

        self.highlight_note_input.update(cx, |state, cx| {
            state.set_value("", window, cx);
        });
        self.is_quote_popover_open = false;
        self.selection_range = None;
        self.tooltip_pos = None;
        self.drag_anchor = None;
        self.is_dragging_text = false;
        self.reload_highlights();
        cx.notify();
    }

    pub fn delete_highlight(&mut self, id: i64, cx: &mut Context<Self>) {
        if let Some(db_path) = Database::default_db_path() {
            if let Ok(db) = Database::open(&db_path) {
                let _ = HighlightRepository::delete(db.conn(), id);
            }
        }
        if self.editing_highlight_id == Some(id) {
            self.editing_highlight_id = None;
        }
        self.reload_highlights();
        cx.notify();
    }

    pub fn start_editing_highlight(
        &mut self,
        hl: &HighlightRecord,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.editing_highlight_id = Some(hl.id);
        let existing_note = hl.note.clone();
        self.highlight_edit_note_input.update(cx, |state, cx| {
            state.set_value(existing_note, window, cx);
        });
        cx.notify();
    }

    pub fn save_edited_highlight(
        &mut self,
        id: i64,
        color: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let note = self.highlight_edit_note_input.read(cx).value().to_string();
        if let Some(db_path) = Database::default_db_path() {
            if let Ok(db) = Database::open(&db_path) {
                let _ = HighlightRepository::update_note_and_color(db.conn(), id, &note, color);
            }
        }
        self.highlight_edit_note_input.update(cx, |state, cx| {
            state.set_value("", window, cx);
        });
        self.editing_highlight_id = None;
        self.reload_highlights();
        cx.notify();
    }

    fn reload_bookmarks(&mut self) {
        self.bookmarks.clear();
        let Some(ref book) = self.book else {
            return;
        };
        if let Some(db_path) = Database::default_db_path() {
            if let Ok(db) = Database::open(db_path) {
                if let Ok(list) = BookmarkRepository::list_for_book(db.conn(), &book.id) {
                    self.bookmarks = list;
                }
            }
        }
    }

    fn current_chapter_info(&self) -> (u32, String) {
        let Some(ref book) = self.book else {
            return (0, String::new());
        };
        let paginated = PaginatedBook::from_chapters(book.chapters.clone());
        let ch_idx = paginated.chapter_index_for_page(self.current_page).unwrap_or(0) as u32;
        let ch_title = ReaderService::resolve_page(book, self.current_page).chapter_title;
        (ch_idx, ch_title)
    }

    pub fn toggle_panel(&mut self, cx: &mut Context<Self>) {
        self.is_panel_open = !self.is_panel_open;
        cx.notify();
    }

    pub fn set_panel_tab(&mut self, tab: ReaderPanelTab, cx: &mut Context<Self>) {
        self.panel_tab = tab;
        cx.notify();
    }

    pub fn toggle_bookmark(&mut self, cx: &mut Context<Self>) {
        let Some(book_id) = self.book.as_ref().map(|b| b.id.clone()) else {
            return;
        };
        let (ch_idx, ch_title) = self.current_chapter_info();
        if let Some(db_path) = Database::default_db_path() {
            if let Ok(db) = Database::open(db_path) {
                if BookmarkRepository::toggle(
                    db.conn(),
                    &book_id,
                    self.current_page,
                    ch_idx,
                    &ch_title,
                )
                .is_ok()
                {
                    self.reload_bookmarks();
                    cx.notify();
                }
            }
        }
    }

    pub fn save_bookmark_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(book_id) = self.book.as_ref().map(|b| b.id.clone()) else {
            return;
        };
        let note = self.note_input.read(cx).value().to_string();
        let (ch_idx, ch_title) = self.current_chapter_info();
        if let Some(db_path) = Database::default_db_path() {
            if let Ok(db) = Database::open(db_path) {
                if BookmarkRepository::save_with_note(
                    db.conn(),
                    &book_id,
                    self.current_page,
                    ch_idx,
                    &ch_title,
                    &note,
                )
                .is_ok()
                {
                    self.note_input.update(cx, |input, cx| {
                        input.set_value("", window, cx);
                    });
                    self.reload_bookmarks();
                    cx.notify();
                }
            }
        }
    }

    pub fn delete_bookmark(&mut self, bookmark_id: i64, cx: &mut Context<Self>) {
        if let Some(db_path) = Database::default_db_path() {
            if let Ok(db) = Database::open(db_path) {
                if BookmarkRepository::delete(db.conn(), bookmark_id).is_ok() {
                    self.reload_bookmarks();
                    cx.notify();
                }
            }
        }
    }

    pub fn jump_to_page(&mut self, page: u32, cx: &mut Context<Self>) {
        let target = page.clamp(1, self.total_pages);
        if target != self.current_page {
            self.current_page = target;
            self.clear_selection_internal();
            self.persist_progress(0);
            cx.notify();
        }
    }

    fn clear_selection_internal(&mut self) {
        self.selection_range = None;
        self.drag_anchor = None;
        self.tooltip_pos = None;
        self.is_dragging_text = false;
    }

    pub fn selected_text(&self) -> Option<String> {
        let (p_idx, start, end) = self.selection_range?;
        if end <= start {
            return None;
        }
        let para = self.current_paragraphs.get(p_idx)?;
        let s = start.min(para.len());
        let e = end.min(para.len());
        if e <= s {
            return None;
        }
        let slice = para.get(s..e)?.trim();
        if slice.is_empty() {
            None
        } else {
            Some(slice.to_string())
        }
    }

    pub fn toggle_ai_for_book(&mut self, cx: &mut Context<Self>) {
        self.ai_enabled_for_book = !self.ai_enabled_for_book;
        if !self.ai_enabled_for_book {
            self.is_ai_drawer_open = false;
            self.tooltip_pos = None;
        }
        if let Some(ref book) = self.book {
            if let Some(db_path) = Database::default_db_path() {
                if let Ok(db) = Database::open(db_path) {
                    let _ =
                        BookRepository::set_ai_enabled(db.conn(), &book.id, self.ai_enabled_for_book);
                }
            }
        }
        cx.notify();
    }

    pub fn toggle_ai_drawer(&mut self, cx: &mut Context<Self>) {
        if !self.ai_enabled_for_book {
            return;
        }
        self.is_ai_drawer_open = !self.is_ai_drawer_open;
        cx.notify();
    }

    pub fn open_ai_with_selection(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.ai_enabled_for_book {
            return;
        }
        if let Some(quote) = self.selected_text() {
            self.ai_selected_quote = Some(quote);
        }
        self.tooltip_pos = None;
        self.is_ai_drawer_open = true;
        self.ai_question_input.update(cx, |input, cx| {
            input.focus(window, cx);
        });
        cx.notify();
    }

    pub fn copy_selection(&mut self, cx: &mut Context<Self>) {
        if let Some(text) = self.selected_text() {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
        self.clear_selection_internal();
        cx.notify();
    }

    pub fn ask_ai(&mut self, preset_question: Option<String>, cx: &mut Context<Self>) {
        if !self.ai_enabled_for_book || self.is_ai_loading {
            return;
        }
        let Some(book) = self.book.clone() else {
            return;
        };

        let question = match preset_question {
            Some(q) => q,
            None => self.ai_question_input.read(cx).value().trim().to_string(),
        };
        if question.is_empty() {
            self.ai_error = Some("Escribe una pregunta o elige una acción rápida.".into());
            cx.notify();
            return;
        }

        let current_page = self.current_page;
        let selected_quote = self.ai_selected_quote.clone();
        self.is_ai_loading = true;
        self.ai_error = None;
        cx.notify();

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    let db_path = Database::default_db_path().ok_or_else(|| {
                        "No se pudo acceder a la base de datos local.".to_string()
                    })?;
                    let mut db = Database::open(db_path).map_err(|e| e.to_string())?;
                    let app_settings =
                        SettingsRepository::load_settings(db.conn()).map_err(|e| e.to_string())?;

                    RagService::ask_assistant(
                        db.conn_mut(),
                        &book,
                        current_page,
                        selected_quote.as_deref(),
                        &question,
                        &app_settings.ai,
                    )
                    .map_err(|e| e.to_string())
                })
                .await;

            let _ = this.update(cx, |view, cx| {
                view.is_ai_loading = false;
                match result {
                    Ok(ans) => {
                        view.ai_answer = Some(ans);
                        view.ai_error = None;
                    }
                    Err(err) => {
                        view.ai_error = Some(err);
                    }
                }
                cx.notify();
            });
        })
        .detach();
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
            self.clear_selection_internal();
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
            self.clear_selection_internal();
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

        self.current_paragraphs = page_content.paragraphs.clone();
        self.paragraph_layouts.clear();

        let view_prev = cx.entity().clone();
        let view_next = cx.entity().clone();
        let view_hud_back = cx.entity().clone();
        let view_font_dec = cx.entity().clone();
        let view_font_inc = cx.entity().clone();
        let view_zoom_out = cx.entity().clone();
        let view_zoom_in = cx.entity().clone();
        let view_zoom_reset = cx.entity().clone();
        let view_toggle_panel = cx.entity().clone();
        let view_toggle_bm = cx.entity().clone();
        let view_toggle_ai_book = cx.entity().clone();
        let view_toggle_ai_drawer = cx.entity().clone();
        let is_current_bookmarked = self
            .bookmarks
            .iter()
            .any(|b| b.page_number == self.current_page);

        div()
            .id("reader-container")
            .track_focus(&self.focus_handle)
            .on_mouse_up(MouseButton::Left, cx.listener(|this, event: &MouseUpEvent, _, cx| {
                if this.is_dragging_text {
                    this.is_dragging_text = false;
                    if this.selected_text().is_some() {
                        this.tooltip_pos = Some(event.position);
                    } else {
                        this.tooltip_pos = None;
                    }
                    cx.notify();
                }
            }))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                let key = event.keystroke.key.to_lowercase();
                let is_ctrl = event.keystroke.modifiers.control || event.keystroke.modifiers.platform;
                let typing_input = this.note_input.focus_handle(cx).is_focused(window)
                    || this.ai_question_input.focus_handle(cx).is_focused(window);

                if is_ctrl && key == "d" {
                    this.toggle_bookmark(cx);
                } else if is_ctrl && key == "t" {
                    this.toggle_panel(cx);
                } else if is_ctrl && (key == "+" || key == "=") {
                    this.zoom_in(cx);
                } else if is_ctrl && (key == "-" || key == "_") {
                    this.zoom_out(cx);
                } else if is_ctrl && key == "0" {
                    this.zoom_reset(cx);
                } else if key == "escape" {
                    if typing_input {
                        this.focus_handle.focus(window, cx);
                    } else if this.tooltip_pos.is_some() {
                        this.clear_selection_internal();
                        cx.notify();
                    } else {
                        this.close(window, cx);
                    }
                } else if !typing_input
                    && (key == "left"
                        || key == "arrowleft"
                        || key == "pageup"
                        || (key == " " && event.keystroke.modifiers.shift))
                {
                    this.prev_page(cx);
                } else if !typing_input
                    && (key == "right"
                        || key == "arrowright"
                        || key == "pagedown"
                        || (key == " " && !event.keystroke.modifiers.shift))
                {
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
                    // Lado izquierdo: Botón Volver compacto + Índice
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(6.0))
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
                            .child(
                                div()
                                    .id("btn-reader-toc")
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(5.0))
                                    .px(px(8.0))
                                    .py(px(3.0))
                                    .rounded(radius)
                                    .cursor_pointer()
                                    .bg(if self.is_panel_open {
                                        tokens.bg_hover
                                    } else {
                                        gpui_kit::transparent_black()
                                    })
                                    .hover(|s| s.bg(tokens.bg_hover))
                                    .on_click(move |_, _, cx| {
                                        view_toggle_panel.update(cx, |this, cx| {
                                            this.toggle_panel(cx);
                                        });
                                    })
                                    .child(
                                        Icon::new(IconName::List)
                                            .size(px(13.0))
                                            .text_color(if self.is_panel_open {
                                                tokens.accent
                                            } else {
                                                tokens.text_muted
                                            }),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_medium()
                                            .text_color(tokens.text)
                                            .child("Índice"),
                                    ),
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
                            // Toggle IA por libro
                            .child(
                                div()
                                    .id("btn-reader-ai-book-toggle")
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(4.0))
                                    .px(px(6.0))
                                    .py(px(2.0))
                                    .rounded(px(4.0))
                                    .border_1()
                                    .border_color(if self.ai_enabled_for_book {
                                        tokens.accent.opacity(0.6)
                                    } else {
                                        tokens.border.opacity(0.6)
                                    })
                                    .bg(tokens.bg_sunken)
                                    .cursor_pointer()
                                    .hover(|s| s.bg(tokens.bg_hover))
                                    .on_click(move |_, _, cx| {
                                        view_toggle_ai_book.update(cx, |this, cx| {
                                            this.toggle_ai_for_book(cx);
                                        });
                                    })
                                    .child(
                                        Icon::new(IconName::Sparkles)
                                            .size(px(11.0))
                                            .text_color(if self.ai_enabled_for_book {
                                                tokens.accent
                                            } else {
                                                tokens.text_muted
                                            }),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(10.0))
                                            .font_medium()
                                            .text_color(if self.ai_enabled_for_book {
                                                tokens.text
                                            } else {
                                                tokens.text_muted
                                            })
                                            .child(if self.ai_enabled_for_book {
                                                "IA: Activa"
                                            } else {
                                                "IA: Off"
                                            }),
                                    ),
                            )
                            // Botón abrir panel Asistente IA
                            .when(self.ai_enabled_for_book, |this| {
                                this.child(
                                    div()
                                        .id("btn-reader-ai-drawer")
                                        .px(px(7.0))
                                        .py(px(2.0))
                                        .rounded(px(4.0))
                                        .border_1()
                                        .border_color(if self.is_ai_drawer_open {
                                            tokens.accent
                                        } else {
                                            tokens.border.opacity(0.6)
                                        })
                                        .bg(if self.is_ai_drawer_open {
                                            tokens.accent
                                        } else {
                                            tokens.bg_sunken
                                        })
                                        .cursor_pointer()
                                        .hover(|s| s.opacity(0.9))
                                        .on_click(move |_, _, cx| {
                                            view_toggle_ai_drawer.update(cx, |this, cx| {
                                                this.toggle_ai_drawer(cx);
                                            });
                                        })
                                        .child(
                                            div()
                                                .text_size(px(10.0))
                                                .font_semibold()
                                                .text_color(if self.is_ai_drawer_open {
                                                    tokens.accent_text
                                                } else {
                                                    tokens.accent
                                                })
                                                .child("Preguntar IA"),
                                        ),
                                )
                            })
                            // Botón Marcador (Ctrl+D)
                            .child(
                                div()
                                    .id("btn-reader-bookmark")
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(4.0))
                                    .px(px(6.0))
                                    .py(px(2.0))
                                    .rounded(px(4.0))
                                    .border_1()
                                    .border_color(if is_current_bookmarked {
                                        tokens.accent
                                    } else {
                                        tokens.border.opacity(0.6)
                                    })
                                    .bg(tokens.bg_sunken)
                                    .cursor_pointer()
                                    .hover(|s| s.bg(tokens.bg_hover))
                                    .on_click(move |_, _, cx| {
                                        view_toggle_bm.update(cx, |this, cx| {
                                            this.toggle_bookmark(cx);
                                        });
                                    })
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(if is_current_bookmarked {
                                                tokens.accent
                                            } else {
                                                tokens.text_muted
                                            })
                                            .child(if is_current_bookmarked { "★" } else { "☆" }),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(10.0))
                                            .text_color(tokens.text_muted)
                                            .child("Ctrl+D"),
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
                    .flex_row()
                    .flex_1()
                    .w_full()
                    .h_full()
                    .relative()
                    .overflow_hidden()
                    .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _window, cx| {
                        if this.is_resizing_ai_drawer {
                            let cur_x: f32 = event.position.x.into();
                            let delta = this.ai_resize_start_x - cur_x;
                            this.ai_drawer_width =
                                (this.ai_resize_start_width + delta).clamp(300.0, 760.0);
                            cx.notify();
                        }
                    }))
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(|this, _event: &MouseUpEvent, _window, cx| {
                            if this.is_resizing_ai_drawer {
                                this.is_resizing_ai_drawer = false;
                                cx.notify();
                            }
                        }),
                    )
                    // Panel lateral desplegable: Índice (TOC) y Marcadores
                    .when(self.is_panel_open, |this| {
                        let view_tab_toc = cx.entity().clone();
                        let view_tab_bm = cx.entity().clone();
                        let view_tab_hl = cx.entity().clone();
                        let view_save_note = cx.entity().clone();
                        let view_jump = cx.entity().clone();
                        let view_del_bm = cx.entity().clone();
                        let panel_tab = self.panel_tab;
                        let current_page = self.current_page;
                        let bookmarks = self.bookmarks.clone();
                        let highlights = self.highlights.clone();
                        let editing_hl_id = self.editing_highlight_id;

                        let mut toc_entries: Vec<(u32, String, bool)> = Vec::new();
                        if let Some(ref b) = self.book {
                            let mut offset: u32 = 1;
                            for (idx, ch) in b.chapters.iter().enumerate() {
                                let len = (ch.pages.len() as u32).max(1);
                                let start_page = offset;
                                let is_active =
                                    current_page >= start_page && current_page < start_page + len;
                                let label = if ch.title.trim().is_empty() {
                                    format!("Sección {}", idx + 1)
                                } else {
                                    ch.title.clone()
                                };
                                toc_entries.push((start_page, label, is_active));
                                offset += len;
                            }
                        }

                        this.child(
                            div()
                                .id("reader-side-panel")
                                .w(px(280.0))
                                .h_full()
                                .flex()
                                .flex_col()
                                .bg(tokens.bg_elevated)
                                .border_r_1()
                                .border_color(tokens.border)
                                // Tabs del panel
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .border_b_1()
                                        .border_color(tokens.border)
                                        .p(px(6.0))
                                        .gap(px(6.0))
                                        .child(
                                            div()
                                                .id("tab-panel-toc")
                                                .flex_1()
                                                .py(px(6.0))
                                                .rounded(px(4.0))
                                                .text_xs()
                                                .font_semibold()
                                                .text_center()
                                                .cursor_pointer()
                                                .bg(if panel_tab == ReaderPanelTab::Toc {
                                                    tokens.bg_sunken
                                                } else {
                                                    gpui_kit::transparent_black()
                                                })
                                                .text_color(if panel_tab == ReaderPanelTab::Toc {
                                                    tokens.accent
                                                } else {
                                                    tokens.text_muted
                                                })
                                                .on_click(move |_, _, cx| {
                                                    view_tab_toc.update(cx, |this, cx| {
                                                        this.set_panel_tab(ReaderPanelTab::Toc, cx);
                                                    });
                                                })
                                                .child("Índice"),
                                        )
                                        .child(
                                            div()
                                                .id("tab-panel-bookmarks")
                                                .flex_1()
                                                .py(px(6.0))
                                                .rounded(px(4.0))
                                                .text_xs()
                                                .font_semibold()
                                                .text_center()
                                                .cursor_pointer()
                                                .bg(if panel_tab == ReaderPanelTab::Bookmarks {
                                                    tokens.bg_sunken
                                                } else {
                                                    gpui_kit::transparent_black()
                                                })
                                                .text_color(if panel_tab == ReaderPanelTab::Bookmarks {
                                                    tokens.accent
                                                } else {
                                                    tokens.text_muted
                                                })
                                                .on_click(move |_, _, cx| {
                                                    view_tab_bm.update(cx, |this, cx| {
                                                        this.set_panel_tab(
                                                            ReaderPanelTab::Bookmarks,
                                                            cx,
                                                        );
                                                    });
                                                })
                                                .child(format!("Marcadores ({})", bookmarks.len())),
                                        )
                                        .child(
                                            div()
                                                .id("tab-panel-highlights")
                                                .flex_1()
                                                .py(px(6.0))
                                                .rounded(px(4.0))
                                                .text_xs()
                                                .font_semibold()
                                                .text_center()
                                                .cursor_pointer()
                                                .bg(if panel_tab == ReaderPanelTab::Highlights {
                                                    tokens.bg_sunken
                                                } else {
                                                    gpui_kit::transparent_black()
                                                })
                                                .text_color(if panel_tab == ReaderPanelTab::Highlights {
                                                    tokens.accent
                                                } else {
                                                    tokens.text_muted
                                                })
                                                .on_click(move |_, _, cx| {
                                                    view_tab_hl.update(cx, |this, cx| {
                                                        this.set_panel_tab(
                                                            ReaderPanelTab::Highlights,
                                                            cx,
                                                        );
                                                    });
                                                })
                                                .child(format!("Citas ({})", highlights.len())),
                                        ),
                                )
                                // Contenido del panel
                                .child(
                                    div()
                                        .id("reader-panel-scroll")
                                        .flex_1()
                                        .overflow_y_scroll()
                                        .p(px(10.0))
                                        .flex()
                                        .flex_col()
                                        .gap(px(6.0))
                                        .map(|panel| {
                                            if panel_tab == ReaderPanelTab::Highlights {
                                                if highlights.is_empty() {
                                                    return panel.child(
                                                        div()
                                                            .p(px(12.0))
                                                            .text_xs()
                                                            .text_color(tokens.text_muted)
                                                            .child("Seleccioná texto en una página y elegí \"Guardar cita\" para guardar pasajes y notas aquí."),
                                                    );
                                                }
                                                return panel.children(highlights.into_iter().map(|hl| {
                                                    let hl_id = hl.id;
                                                    let hl_page = hl.page_number;
                                                    let hl_color = hl.color.clone();
                                                    let hl_for_edit = hl.clone();
                                                    let solid = highlight_solid_color(&hl_color, tokens.accent);
                                                    let is_editing = editing_hl_id == Some(hl_id);
                                                    let view_j = view_jump.clone();

                                                    div()
                                                        .id(ElementId::from(format!("hl-item-{hl_id}")))
                                                        .flex()
                                                        .flex_col()
                                                        .gap(px(5.0))
                                                        .p(px(9.0))
                                                        .rounded(px(6.0))
                                                        .border_1()
                                                        .border_color(tokens.border)
                                                        .border_l_3()
                                                        .border_color(solid)
                                                        .bg(tokens.bg)
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .flex_row()
                                                                .items_center()
                                                                .justify_between()
                                                                .child(
                                                                    div()
                                                                        .id(ElementId::from(format!("hl-jump-{hl_id}")))
                                                                        .text_xs()
                                                                        .font_semibold()
                                                                        .text_color(solid)
                                                                        .cursor_pointer()
                                                                        .on_click(move |_, _, cx| {
                                                                            view_j.update(cx, |this, cx| {
                                                                                this.jump_to_page(hl_page, cx);
                                                                            });
                                                                        })
                                                                        .child(format!("Pág. {hl_page}")),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .flex()
                                                                        .flex_row()
                                                                        .gap(px(6.0))
                                                                        .child(
                                                                            div()
                                                                                .id(ElementId::from(format!("hl-edit-{hl_id}")))
                                                                                .text_size(px(10.5))
                                                                                .text_color(tokens.text_muted)
                                                                                .cursor_pointer()
                                                                                .hover(|s| s.text_color(tokens.accent))
                                                                                .on_click(cx.listener(move |this, _, window, cx| {
                                                                                    this.start_editing_highlight(&hl_for_edit, window, cx);
                                                                                }))
                                                                                .child("Editar"),
                                                                        )
                                                                        .child(
                                                                            div()
                                                                                .id(ElementId::from(format!("hl-del-{hl_id}")))
                                                                                .text_xs()
                                                                                .text_color(tokens.text_muted)
                                                                                .cursor_pointer()
                                                                                .hover(|s| s.text_color(tokens.danger))
                                                                                .on_click(cx.listener(move |this, _, _window, cx| {
                                                                                    this.delete_highlight(hl_id, cx);
                                                                                }))
                                                                                .child("×"),
                                                                        ),
                                                                ),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .italic()
                                                                .text_color(tokens.text)
                                                                .child(format!("\"{}\"", hl.selected_text)),
                                                        )
                                                        .when(!is_editing && !hl.note.trim().is_empty(), |card| {
                                                            card.child(
                                                                div()
                                                                    .p(px(6.0))
                                                                    .rounded(px(4.0))
                                                                    .bg(tokens.bg_sunken)
                                                                    .text_xs()
                                                                    .text_color(tokens.text_muted)
                                                                    .child(hl.note),
                                                            )
                                                        })
                                                        .when(is_editing, |card| {
                                                            let c_save = hl_color.clone();
                                                            card.child(
                                                                div()
                                                                    .flex()
                                                                    .flex_col()
                                                                    .gap(px(6.0))
                                                                    .pt(px(4.0))
                                                                    .child(Input::new(&self.highlight_edit_note_input))
                                                                    .child(
                                                                        div()
                                                                            .flex()
                                                                            .flex_row()
                                                                            .gap(px(6.0))
                                                                            .child(
                                                                                div()
                                                                                    .id(ElementId::from(format!("hl-save-edit-{hl_id}")))
                                                                                    .px(px(8.0))
                                                                                    .py(px(4.0))
                                                                                    .rounded(px(4.0))
                                                                                    .bg(tokens.accent)
                                                                                    .text_size(px(11.0))
                                                                                    .text_color(tokens.accent_text)
                                                                                    .cursor_pointer()
                                                                                    .on_click(cx.listener(move |this, _, window, cx| {
                                                                                        this.save_edited_highlight(hl_id, &c_save, window, cx);
                                                                                    }))
                                                                                    .child("Guardar"),
                                                                            )
                                                                            .child(
                                                                                div()
                                                                                    .id(ElementId::from(format!("hl-cancel-edit-{hl_id}")))
                                                                                    .px(px(8.0))
                                                                                    .py(px(4.0))
                                                                                    .rounded(px(4.0))
                                                                                    .bg(tokens.bg_sunken)
                                                                                    .text_size(px(11.0))
                                                                                    .text_color(tokens.text_muted)
                                                                                    .cursor_pointer()
                                                                                    .on_click(cx.listener(|this, _, _window, cx| {
                                                                                        this.editing_highlight_id = None;
                                                                                        cx.notify();
                                                                                    }))
                                                                                    .child("Cancelar"),
                                                                            ),
                                                                    ),
                                                            )
                                                        })
                                                }));
                                            }
                                            if panel_tab == ReaderPanelTab::Toc {
                                                panel.children(toc_entries.into_iter().enumerate().map(
                                                    |(idx, (start_page, label, is_active))| {
                                                        let view_j = view_jump.clone();
                                                        div()
                                                            .id(ElementId::from(format!(
                                                                "toc-item-{idx}"
                                                            )))
                                                            .flex()
                                                            .flex_row()
                                                            .items_center()
                                                            .justify_between()
                                                            .px(px(10.0))
                                                            .py(px(7.0))
                                                            .rounded(px(4.0))
                                                            .cursor_pointer()
                                                            .bg(if is_active {
                                                                tokens.bg_sunken
                                                            } else {
                                                                gpui_kit::transparent_black()
                                                            })
                                                            .hover(|s| s.bg(tokens.bg_hover))
                                                            .on_click(move |_, _, cx| {
                                                                view_j.update(cx, |this, cx| {
                                                                    this.jump_to_page(start_page, cx);
                                                                });
                                                            })
                                                            .child(
                                                                div()
                                                                    .text_xs()
                                                                    .font_medium()
                                                                    .text_color(if is_active {
                                                                        tokens.accent
                                                                    } else {
                                                                        tokens.text
                                                                    })
                                                                    .overflow_hidden()
                                                                    .text_ellipsis()
                                                                    .child(label),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_size(px(10.0))
                                                                    .text_color(tokens.text_muted)
                                                                    .child(format!("p.{start_page}")),
                                                            )
                                                    },
                                                ))
                                            } else {
                                                panel
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .flex_col()
                                                            .gap(px(6.0))
                                                            .p(px(8.0))
                                                            .mb(px(6.0))
                                                            .rounded(px(6.0))
                                                            .bg(tokens.bg_sunken)
                                                            .border_1()
                                                            .border_color(tokens.border)
                                                            .child(Input::new(&self.note_input))
                                                            .child(
                                                                div()
                                                                    .id("btn-save-bookmark-note")
                                                                    .py(px(5.0))
                                                                    .rounded(px(4.0))
                                                                    .bg(tokens.accent)
                                                                    .text_xs()
                                                                    .font_semibold()
                                                                    .text_center()
                                                                    .text_color(tokens.accent_text)
                                                                    .cursor_pointer()
                                                                    .hover(|s| s.opacity(0.9))
                                                                    .on_click(move |_, window, cx| {
                                                                        view_save_note.update(
                                                                            cx,
                                                                            |this, cx| {
                                                                                this.save_bookmark_note(
                                                                                    window, cx,
                                                                                );
                                                                            },
                                                                        );
                                                                    })
                                                                    .child(format!(
                                                                        "Guardar marcador en pág. {}",
                                                                        current_page
                                                                    )),
                                                            ),
                                                    )
                                                    .children(bookmarks.into_iter().map(|bm| {
                                                        let view_j = view_jump.clone();
                                                        let view_d = view_del_bm.clone();
                                                        let bm_id = bm.id;
                                                        let bm_page = bm.page_number;
                                                        div()
                                                            .id(ElementId::from(format!(
                                                                "bm-item-{bm_id}"
                                                            )))
                                                            .flex()
                                                            .flex_col()
                                                            .gap(px(3.0))
                                                            .p(px(8.0))
                                                            .rounded(px(6.0))
                                                            .border_1()
                                                            .border_color(tokens.border)
                                                            .bg(tokens.bg)
                                                            .hover(|s| s.bg(tokens.bg_hover))
                                                            .cursor_pointer()
                                                            .on_click(move |_, _, cx| {
                                                                view_j.update(cx, |this, cx| {
                                                                    this.jump_to_page(bm_page, cx);
                                                                });
                                                            })
                                                            .child(
                                                                div()
                                                                    .flex()
                                                                    .flex_row()
                                                                    .items_center()
                                                                    .justify_between()
                                                                    .child(
                                                                        div()
                                                                            .text_xs()
                                                                            .font_semibold()
                                                                            .text_color(tokens.accent)
                                                                            .child(format!(
                                                                                "★ Pág. {}",
                                                                                bm_page
                                                                            )),
                                                                    )
                                                                    .child(
                                                                        div()
                                                                            .id(ElementId::from(
                                                                                format!(
                                                                                    "bm-del-{bm_id}"
                                                                                ),
                                                                            ))
                                                                            .px(px(4.0))
                                                                            .text_xs()
                                                                            .text_color(
                                                                                tokens.text_muted,
                                                                            )
                                                                            .hover(|s| {
                                                                                s.text_color(
                                                                                    tokens.text,
                                                                                )
                                                                            })
                                                                            .on_click(move |_, _, cx| {
                                                                                cx.stop_propagation();
                                                                                view_d.update(
                                                                                    cx,
                                                                                    |this, cx| {
                                                                                        this.delete_bookmark(
                                                                                            bm_id,
                                                                                            cx,
                                                                                        );
                                                                                    },
                                                                                );
                                                                            })
                                                                            .child("×"),
                                                                    ),
                                                            )
                                                            .when(
                                                                !bm.chapter_title.trim().is_empty(),
                                                                |this| {
                                                                    this.child(
                                                                        div()
                                                                            .text_size(px(10.5))
                                                                            .text_color(
                                                                                tokens.text_muted,
                                                                            )
                                                                            .child(bm.chapter_title),
                                                                    )
                                                                },
                                                            )
                                                            .when(
                                                                !bm.note.trim().is_empty(),
                                                                |this| {
                                                                    this.child(
                                                                        div()
                                                                            .text_xs()
                                                                            .text_color(tokens.text)
                                                                            .child(bm.note),
                                                                    )
                                                                },
                                                            )
                                                    }))
                                            }
                                        }),
                                ),
                        )
                    })
                    // Hoja de Lectura Centrada
                    .child(
                        div()
                            .id("reader-sheet-content")
                            .flex()
                            .flex_col()
                            .flex_1()
                            .h_full()
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
                                            .when(!page_content.chapter_title.trim().is_empty(), |this| {
                                                this.child(
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
                                                )
                                            }),
                                    )
                                    // Párrafos seleccionables del libro
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap(px(effective_gap))
                                            .text_size(px(effective_font_size))
                                            .line_height(relative(line_height))
                                            .text_color(text_color)
                                            .children(page_content.paragraphs.into_iter().enumerate().map(|(p_idx, p)| {
                                                let p_len = p.len();
                                                let mut runs: Vec<(std::ops::Range<usize>, HighlightStyle)> = self
                                                    .highlights
                                                    .iter()
                                                    .filter(|h| {
                                                        h.page_number == current_page
                                                            && h.paragraph_index == p_idx
                                                            && h.char_start < h.char_end
                                                            && h.char_end <= p_len
                                                    })
                                                    .map(|h| {
                                                        (
                                                            h.char_start..h.char_end,
                                                            HighlightStyle {
                                                                background_color: Some(highlight_bg_color(
                                                                    &h.color,
                                                                    tokens.accent,
                                                                )),
                                                                ..Default::default()
                                                            },
                                                        )
                                                    })
                                                    .collect();

                                                if let Some((sel_p, s, e)) = self.selection_range {
                                                    if sel_p == p_idx && s < e && e <= p_len {
                                                        runs.push((
                                                            s..e,
                                                            HighlightStyle {
                                                                background_color: Some(
                                                                    tokens.accent.opacity(0.28),
                                                                ),
                                                                ..Default::default()
                                                            },
                                                        ));
                                                    }
                                                }

                                                runs = normalize_non_overlapping_runs(&p, runs);

                                                let styled = if runs.is_empty() {
                                                    StyledText::new(p)
                                                } else {
                                                    let base_para_style = TextStyle {
                                                        color: text_color,
                                                        font_size: px(effective_font_size).into(),
                                                        line_height: relative(line_height).into(),
                                                        font_family: font_family.into(),
                                                        ..Default::default()
                                                    };
                                                    StyledText::new(p)
                                                        .with_default_highlights(&base_para_style, runs)
                                                };
                                                let layout_down = styled.layout().clone();
                                                let layout_move = styled.layout().clone();
                                                self.paragraph_layouts.push(styled.layout().clone());

                                                div()
                                                    .id(ElementId::from(format!("reader-para-{p_idx}")))
                                                    .cursor_text()
                                                    .on_mouse_down(
                                                        MouseButton::Left,
                                                        cx.listener(move |this, event: &MouseDownEvent, _, cx| {
                                                            let offset = match layout_down.index_for_position(event.position) {
                                                                Ok(i) | Err(i) => i,
                                                            };
                                                            this.drag_anchor = Some((p_idx, offset));
                                                            this.selection_range = Some((p_idx, offset, offset));
                                                            this.is_dragging_text = true;
                                                            this.tooltip_pos = None;
                                                            cx.notify();
                                                        }),
                                                    )
                                                    .on_mouse_move(
                                                        cx.listener(move |this, event: &MouseMoveEvent, _, cx| {
                                                            if this.is_dragging_text {
                                                                if let Some((anchor_p, anchor_off)) = this.drag_anchor {
                                                                    if anchor_p == p_idx {
                                                                        let offset = match layout_move.index_for_position(event.position) {
                                                                            Ok(i) | Err(i) => i,
                                                                        };
                                                                        this.selection_range = Some((
                                                                            p_idx,
                                                                            anchor_off.min(offset),
                                                                            anchor_off.max(offset),
                                                                        ));
                                                                        cx.notify();
                                                                    }
                                                                }
                                                            }
                                                        }),
                                                    )
                                                    .child(styled)
                                            })),
                                    ),
                            ),
                    )
                    // Panel lateral derecho: Asistente Inteligente de Lectura (RAG)
                    .when(self.is_ai_drawer_open && self.ai_enabled_for_book, |this| {
                        let view_close_ai = cx.entity().clone();
                        let view_clear_quote = cx.entity().clone();
                        let view_ask = cx.entity().clone();
                        let view_q1 = cx.entity().clone();
                        let view_q2 = cx.entity().clone();
                        let view_q3 = cx.entity().clone();
                        let view_jump_cite = cx.entity().clone();
                        let current_page = self.current_page;
                        let quote = self.ai_selected_quote.clone();
                        let answer = self.ai_answer.clone();
                        let err = self.ai_error.clone();
                        let is_loading = self.is_ai_loading;

                        let drawer_width = self.ai_drawer_width;
                        let is_resizing = self.is_resizing_ai_drawer;

                        this.child(
                            div()
                                .id("reader-ai-drawer")
                                .w(px(drawer_width))
                                .h_full()
                                .relative()
                                .flex()
                                .flex_col()
                                .bg(tokens.bg_elevated)
                                .border_l_1()
                                .border_color(tokens.border)
                                // Tirador izquierdo para redimensionar el panel de IA
                                .child(
                                    div()
                                        .id("ai-drawer-resize-handle")
                                        .absolute()
                                        .top_0()
                                        .left_0()
                                        .bottom_0()
                                        .w(px(6.0))
                                        .cursor_col_resize()
                                        .bg(if is_resizing {
                                            tokens.accent.opacity(0.45)
                                        } else {
                                            gpui::transparent_black()
                                        })
                                        .hover(|s| s.bg(tokens.accent.opacity(0.35)))
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(|this, event: &MouseDownEvent, _window, cx| {
                                                this.is_resizing_ai_drawer = true;
                                                this.ai_resize_start_x = event.position.x.into();
                                                this.ai_resize_start_width = this.ai_drawer_width;
                                                cx.stop_propagation();
                                                cx.notify();
                                            }),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .px(px(14.0))
                                        .py(px(10.0))
                                        .border_b_1()
                                        .border_color(tokens.border)
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_semibold()
                                                        .text_color(tokens.text)
                                                        .child("Asistente IA (Anti-Spoilers)"),
                                                )
                                                .child(
                                                    div()
                                                        .text_size(px(10.5))
                                                        .text_color(tokens.text_muted)
                                                        .child(format!("Contexto acotado hasta pág. {current_page}")),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .id("btn-close-ai-drawer")
                                                .px(px(6.0))
                                                .py(px(2.0))
                                                .rounded(px(4.0))
                                                .cursor_pointer()
                                                .hover(|s| s.bg(tokens.bg_hover))
                                                .text_xs()
                                                .text_color(tokens.text_muted)
                                                .on_click(move |_, _, cx| {
                                                    view_close_ai.update(cx, |this, cx| {
                                                        this.is_ai_drawer_open = false;
                                                        cx.notify();
                                                    });
                                                })
                                                .child("×"),
                                        ),
                                )
                                .child(
                                    div()
                                        .id("reader-ai-scroll")
                                        .flex_1()
                                        .overflow_y_scroll()
                                        .p(px(12.0))
                                        .flex()
                                        .flex_col()
                                        .gap(px(10.0))
                                        .when_some(quote, |this, q| {
                                            this.child(
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .gap(px(4.0))
                                                    .p(px(10.0))
                                                    .rounded(px(6.0))
                                                    .bg(tokens.bg_sunken)
                                                    .border_l_2()
                                                    .border_color(tokens.accent)
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .flex_row()
                                                            .items_center()
                                                            .justify_between()
                                                            .child(
                                                                div()
                                                                    .text_size(px(10.0))
                                                                    .font_semibold()
                                                                    .text_color(tokens.accent)
                                                                    .child("Pasaje seleccionado"),
                                                            )
                                                            .child(
                                                                div()
                                                                    .id("btn-clear-ai-quote")
                                                                    .text_xs()
                                                                    .text_color(tokens.text_muted)
                                                                    .cursor_pointer()
                                                                    .on_click(move |_, _, cx| {
                                                                        view_clear_quote.update(cx, |this, cx| {
                                                                            this.ai_selected_quote = None;
                                                                            cx.notify();
                                                                        });
                                                                    })
                                                                    .child("×"),
                                                            ),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(tokens.text_muted)
                                                            .child(format!("\"{q}\"")),
                                                    ),
                                            )
                                        })
                                        // Acciones rápidas
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .flex_wrap()
                                                .gap(px(6.0))
                                                .child(
                                                    div()
                                                        .id("chip-ai-catchup")
                                                        .px(px(8.0))
                                                        .py(px(4.0))
                                                        .rounded_full()
                                                        .bg(tokens.bg_sunken)
                                                        .border_1()
                                                        .border_color(tokens.border)
                                                        .text_size(px(10.5))
                                                        .text_color(tokens.text)
                                                        .cursor_pointer()
                                                        .hover(|s| s.border_color(tokens.accent))
                                                        .on_click(move |_, _, cx| {
                                                            view_q1.update(cx, |this, cx| {
                                                                this.ask_ai(
                                                                    Some("¿Qué ocurrió antes en la trama hasta este punto?".into()),
                                                                    cx,
                                                                );
                                                            });
                                                        })
                                                        .child("¿Qué ocurrió antes?"),
                                                )
                                                .child(
                                                    div()
                                                        .id("chip-ai-chapter")
                                                        .px(px(8.0))
                                                        .py(px(4.0))
                                                        .rounded_full()
                                                        .bg(tokens.bg_sunken)
                                                        .border_1()
                                                        .border_color(tokens.border)
                                                        .text_size(px(10.5))
                                                        .text_color(tokens.text)
                                                        .cursor_pointer()
                                                        .hover(|s| s.border_color(tokens.accent))
                                                        .on_click(move |_, _, cx| {
                                                            view_q2.update(cx, |this, cx| {
                                                                this.ask_ai(
                                                                    Some("Resume lo que he leído del capítulo actual hasta esta página.".into()),
                                                                    cx,
                                                                );
                                                            });
                                                        })
                                                        .child("Resumir capítulo"),
                                                )
                                                .child(
                                                    div()
                                                        .id("chip-ai-explain")
                                                        .px(px(8.0))
                                                        .py(px(4.0))
                                                        .rounded_full()
                                                        .bg(tokens.bg_sunken)
                                                        .border_1()
                                                        .border_color(tokens.border)
                                                        .text_size(px(10.5))
                                                        .text_color(tokens.text)
                                                        .cursor_pointer()
                                                        .hover(|s| s.border_color(tokens.accent))
                                                        .on_click(move |_, _, cx| {
                                                            view_q3.update(cx, |this, cx| {
                                                                this.ask_ai(
                                                                    Some("Explica este pasaje o personaje según lo leído hasta ahora.".into()),
                                                                    cx,
                                                                );
                                                            });
                                                        })
                                                        .child("Explicar pasaje"),
                                                ),
                                        )
                                        // Input de pregunta + Enviar
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .gap(px(6.0))
                                                .child(Input::new(&self.ai_question_input))
                                                .child(
                                                    div()
                                                        .id("btn-send-ai-query")
                                                        .py(px(7.0))
                                                        .rounded(px(6.0))
                                                        .bg(tokens.accent)
                                                        .text_xs()
                                                        .font_semibold()
                                                        .text_center()
                                                        .text_color(tokens.accent_text)
                                                        .cursor_pointer()
                                                        .hover(|s| s.opacity(0.9))
                                                        .on_click(move |_, _, cx| {
                                                            view_ask.update(cx, |this, cx| {
                                                                this.ask_ai(None, cx);
                                                            });
                                                        })
                                                        .child(if is_loading {
                                                            "Consultando e indexando si es necesario..."
                                                        } else {
                                                            "Enviar consulta"
                                                        }),
                                                ),
                                        )
                                        .when_some(err, |this, e| {
                                            this.child(
                                                div()
                                                    .p(px(10.0))
                                                    .rounded(px(6.0))
                                                    .bg(tokens.bg_sunken)
                                                    .border_1()
                                                    .border_color(tokens.danger)
                                                    .text_size(px(13.0))
                                                    .text_color(tokens.danger)
                                                    .child(e),
                                            )
                                        })
                                        .when_some(answer, |this, ans| {
                                            this.child(
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .gap(px(10.0))
                                                    .p(px(14.0))
                                                    .rounded(px(8.0))
                                                    .bg(tokens.bg)
                                                    .border_1()
                                                    .border_color(tokens.border)
                                                    .child(render_markdown_message(
                                                        &ans.answer,
                                                        tokens.text,
                                                        tokens.accent,
                                                    ))
                                                    .when(!ans.cited_pages.is_empty(), |this| {
                                                        this.child(
                                                            div()
                                                                .flex()
                                                                .flex_row()
                                                                .flex_wrap()
                                                                .gap(px(4.0))
                                                                .pt(px(4.0))
                                                                .border_t_1()
                                                                .border_color(tokens.border)
                                                                .children(ans.cited_pages.into_iter().map(|p| {
                                                                    let view_jc = view_jump_cite.clone();
                                                                    div()
                                                                        .id(ElementId::from(format!("cite-p-{p}")))
                                                                        .px(px(6.0))
                                                                        .py(px(2.0))
                                                                        .rounded(px(4.0))
                                                                        .bg(tokens.bg_sunken)
                                                                        .text_size(px(10.0))
                                                                        .text_color(tokens.accent)
                                                                        .cursor_pointer()
                                                                        .on_click(move |_, _, cx| {
                                                                            view_jc.update(cx, |this, cx| {
                                                                                this.jump_to_page(p, cx);
                                                                            });
                                                                        })
                                                                        .child(format!("Pág. {p}"))
                                                                })),
                                                        )
                                                    }),
                                            )
                                        }),
                                ),
                        )
                    })
                    // Zonas de Clic Laterales Transparentes (estrechas para no interferir con la selección de texto)
                    .child(
                        div()
                            .id("reader-side-nav-left")
                            .absolute()
                            .top_0()
                            .left_0()
                            .bottom_0()
                            .w(px(64.0))
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
                            .w(px(64.0))
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
            // 4. Tooltip flotante al seleccionar texto con el ratón
            .when_some(self.tooltip_pos, |this, pos| {
                let top_y = (pos.y - px(46.0)).max(px(44.0));
                let left_x = (pos.x - px(110.0)).max(px(16.0));
                let ai_enabled = self.ai_enabled_for_book;
                let quote_open = self.is_quote_popover_open;
                let sel_color = self.selected_highlight_color.clone();
                let solid_tooltip_bg = gpui::Hsla {
                    a: 1.0,
                    ..tokens.bg
                };

                this.child(
                    div()
                        .id("selection-floating-tooltip")
                        .absolute()
                        .top(top_y)
                        .left(left_x)
                        .rounded(px(10.0))
                        .bg(solid_tooltip_bg)
                        .border_1()
                        .border_color(tokens.border)
                        .shadow_lg()
                        .on_mouse_down(MouseButton::Left, |_, _, cx| {
                            cx.stop_propagation();
                        })
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(6.0))
                                .px(px(8.0))
                                .py(px(6.0))
                                .rounded(px(10.0))
                                .bg(tokens.bg_elevated)
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(4.0))
                                .when(ai_enabled, |bar| {
                                    bar.child(
                                        div()
                                            .id("tooltip-ask-ai")
                                            .px(px(10.0))
                                            .py(px(4.0))
                                            .rounded(px(6.0))
                                            .bg(tokens.accent)
                                            .text_xs()
                                            .font_medium()
                                            .text_color(tokens.bg)
                                            .cursor_pointer()
                                            .hover(|s| s.opacity(0.9))
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.open_ai_with_selection(window, cx);
                                            }))
                                            .child("✨ Preguntar a la IA"),
                                    )
                                })
                                .child(
                                    div()
                                        .id("tooltip-save-quote")
                                        .px(px(8.0))
                                        .py(px(4.0))
                                        .rounded(px(6.0))
                                        .bg(if quote_open {
                                            tokens.bg_sunken
                                        } else {
                                            gpui_kit::transparent_black()
                                        })
                                        .text_xs()
                                        .font_medium()
                                        .text_color(tokens.text)
                                        .cursor_pointer()
                                        .hover(|s| s.bg(tokens.bg_hover))
                                        .on_click(cx.listener(|this, _, _window, cx| {
                                            this.is_quote_popover_open = !this.is_quote_popover_open;
                                            cx.notify();
                                        }))
                                        .child("🔖 Guardar cita"),
                                )
                                .child(
                                    div()
                                        .id("tooltip-copy-sel")
                                        .px(px(8.0))
                                        .py(px(4.0))
                                        .rounded(px(6.0))
                                        .text_xs()
                                        .text_color(tokens.text)
                                        .cursor_pointer()
                                        .hover(|s| s.bg(tokens.bg_hover))
                                        .on_click(cx.listener(|this, _, _window, cx| {
                                            this.copy_selection(cx);
                                        }))
                                        .child("Copiar"),
                                )
                                .child(
                                    div()
                                        .id("tooltip-dismiss-sel")
                                        .px(px(6.0))
                                        .py(px(4.0))
                                        .rounded(px(6.0))
                                        .text_xs()
                                        .text_color(tokens.text_muted)
                                        .cursor_pointer()
                                        .hover(|s| s.bg(tokens.bg_hover))
                                        .on_click(cx.listener(|this, _, _window, cx| {
                                            this.clear_selection_internal();
                                            this.is_quote_popover_open = false;
                                            cx.notify();
                                        }))
                                        .child("×"),
                                ),
                        )
                        .when(quote_open, |pop| {
                            pop.child(
                                div()
                                    .w(px(280.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(6.0))
                                    .pt(px(4.0))
                                    .border_t_1()
                                    .border_color(tokens.border)
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap(px(6.0))
                                            .child(
                                                div()
                                                    .text_size(px(11.0))
                                                    .text_color(tokens.text_muted)
                                                    .child("Color:"),
                                            )
                                            .children(["amber", "emerald", "sky", "rose"].into_iter().map(
                                                |c_name| {
                                                    let is_sel = sel_color == c_name;
                                                    let solid =
                                                        highlight_solid_color(c_name, tokens.accent);
                                                    div()
                                                        .id(ElementId::from(format!(
                                                            "hl-color-{c_name}"
                                                        )))
                                                        .w(px(16.0))
                                                        .h(px(16.0))
                                                        .rounded_full()
                                                        .bg(solid)
                                                        .border_2()
                                                        .border_color(if is_sel {
                                                            tokens.text
                                                        } else {
                                                            gpui_kit::transparent_black()
                                                        })
                                                        .cursor_pointer()
                                                        .on_click(cx.listener(
                                                            move |this, _, _window, cx| {
                                                                this.selected_highlight_color =
                                                                    c_name.to_string();
                                                                cx.notify();
                                                            },
                                                        ))
                                                },
                                            )),
                                    )
                                    .child(Input::new(&self.highlight_note_input))
                                    .child(
                                        div()
                                            .id("btn-confirm-save-quote")
                                            .py(px(5.0))
                                            .rounded(px(6.0))
                                            .bg(tokens.accent)
                                            .text_xs()
                                            .font_semibold()
                                            .text_center()
                                            .text_color(tokens.accent_text)
                                            .cursor_pointer()
                                            .hover(|s| s.opacity(0.9))
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.save_selection_as_highlight(window, cx);
                                            }))
                                            .child("Guardar cita y nota"),
                                    ),
                            )
                        }),
                        ),
                )
            })
    }
}

/// Renderiza respuestas de la IA limpiando Markdown básico (`###`, `**negrita**`, `*cursiva*`, listas `-`/`*`)
/// con tipografía legible en GPUI.
fn render_markdown_message(text: &str, text_color: Hsla, accent_color: Hsla) -> Div {
    let base_style = TextStyle {
        color: text_color,
        ..Default::default()
    };

    let mut container = div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .text_size(px(14.0))
        .line_height(relative(1.55))
        .text_color(text_color);

    for raw_line in text.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Encabezados (#, ##, ###)
        if let Some(heading) = trimmed
            .strip_prefix("### ")
            .or_else(|| trimmed.strip_prefix("## "))
            .or_else(|| trimmed.strip_prefix("# "))
        {
            let (clean, _) = parse_inline_markdown(heading.trim(), text_color, accent_color);
            container = container.child(
                div()
                    .w_full()
                    .pt(px(4.0))
                    .text_size(px(15.0))
                    .font_semibold()
                    .text_color(text_color)
                    .child(clean),
            );
            continue;
        }

        // Ítems de lista (- o *)
        if let Some(item) = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
            .or_else(|| trimmed.strip_prefix("• "))
        {
            let (clean, runs) = parse_inline_markdown(item.trim(), text_color, accent_color);
            container = container.child(
                div()
                    .w_full()
                    .flex()
                    .flex_row()
                    .items_start()
                    .gap(px(6.0))
                    .pl(px(4.0))
                    .child(
                        div()
                            .text_size(px(14.0))
                            .font_semibold()
                            .text_color(accent_color)
                            .child("•"),
                    )
                    .child(
                        div().flex_1().child(
                            StyledText::new(clean).with_default_highlights(&base_style, runs),
                        ),
                    ),
            );
            continue;
        }

        // Párrafo normal (con soporte para **negrita**, *cursiva* y `código`)
        let (clean, runs) = parse_inline_markdown(trimmed, text_color, accent_color);
        container = container.child(
            div().w_full().child(
                StyledText::new(clean).with_default_highlights(&base_style, runs),
            ),
        );
    }

    container
}

fn parse_inline_markdown(
    input: &str,
    bold_color: Hsla,
    accent_color: Hsla,
) -> (String, Vec<(std::ops::Range<usize>, HighlightStyle)>) {
    let mut clean = String::with_capacity(input.len());
    let mut runs = Vec::new();
    let mut rest = input;

    while !rest.is_empty() {
        if let Some(after_open) = rest.strip_prefix("**") {
            if let Some(end_idx) = after_open.find("**") {
                let inner = &after_open[..end_idx];
                let start = clean.len();
                clean.push_str(inner);
                let end = clean.len();
                if start < end {
                    runs.push((
                        start..end,
                        HighlightStyle {
                            font_weight: Some(FontWeight::SEMIBOLD),
                            color: Some(bold_color),
                            ..Default::default()
                        },
                    ));
                }
                rest = &after_open[end_idx + 2..];
                continue;
            }
        }

        if let Some(after_open) = rest.strip_prefix('`') {
            if let Some(end_idx) = after_open.find('`') {
                let inner = &after_open[..end_idx];
                let start = clean.len();
                clean.push_str(inner);
                let end = clean.len();
                if start < end {
                    runs.push((
                        start..end,
                        HighlightStyle {
                            font_weight: Some(FontWeight::MEDIUM),
                            color: Some(accent_color),
                            ..Default::default()
                        },
                    ));
                }
                rest = &after_open[end_idx + 1..];
                continue;
            }
        }

        if let Some(after_open) = rest.strip_prefix('*') {
            if let Some(end_idx) = after_open.find('*') {
                if end_idx > 0 {
                    let inner = &after_open[..end_idx];
                    let start = clean.len();
                    clean.push_str(inner);
                    let end = clean.len();
                    runs.push((
                        start..end,
                        HighlightStyle {
                            font_style: Some(FontStyle::Italic),
                            ..Default::default()
                        },
                    ));
                    rest = &after_open[end_idx + 1..];
                    continue;
                }
            }
        }

        let ch = rest.chars().next().unwrap();
        clean.push(ch);
        rest = &rest[ch.len_utf8()..];
    }

    (clean, runs)
}

fn highlight_solid_color(color: &str, fallback: Hsla) -> Hsla {
    match color {
        "amber" => gpui::hsla(0.11, 0.90, 0.55, 1.0),
        "emerald" => gpui::hsla(0.42, 0.72, 0.45, 1.0),
        "sky" => gpui::hsla(0.56, 0.85, 0.56, 1.0),
        "rose" => gpui::hsla(0.94, 0.82, 0.60, 1.0),
        _ => fallback,
    }
}

fn highlight_bg_color(color: &str, fallback: Hsla) -> Hsla {
    highlight_solid_color(color, fallback).opacity(0.28)
}

/// Garantiza que todos los rangos de resaltado sean válidos, estén en límites UTF-8 y no se solapen,
/// evitando panics en `StyledText::with_default_highlights`.
fn normalize_non_overlapping_runs(
    text: &str,
    runs: Vec<(std::ops::Range<usize>, HighlightStyle)>,
) -> Vec<(std::ops::Range<usize>, HighlightStyle)> {
    let len = text.len();
    let mut valid: Vec<(std::ops::Range<usize>, HighlightStyle)> = runs
        .into_iter()
        .filter_map(|(r, style)| {
            let mut s = r.start.min(len);
            let mut e = r.end.min(len);
            while s > 0 && !text.is_char_boundary(s) {
                s -= 1;
            }
            while e < len && !text.is_char_boundary(e) {
                e += 1;
            }
            if s < e {
                Some((s..e, style))
            } else {
                None
            }
        })
        .collect();

    valid.sort_by_key(|(r, _)| (r.start, r.end));

    let mut out = Vec::with_capacity(valid.len());
    let mut cursor = 0usize;
    for (r, style) in valid {
        let s = r.start.max(cursor);
        if s < r.end {
            cursor = r.end;
            out.push((s..r.end, style));
        }
    }
    out
}
