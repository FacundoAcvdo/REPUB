use std::rc::Rc;
use epub_core::domain::book::{Book, BookFilter, BookSort};
use epub_core::mock::books::{generate_mock_catalog, mock_books};
use gpui_kit::assets::IconName;
use gpui_kit::base::StyledExt as _;
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::component::Icon;
use gpui_kit::*;

use crate::components::book_card::{BookCard, BookRow};
use crate::components::library_toolbar::LibraryToolbar;
use crate::theme::state::ThemeState;

pub struct LibraryView {
    books: Vec<Book>,
    filtered_books: Vec<Book>,
    search_input: Entity<InputState>,
    search_query: String,
    filter: BookFilter,
    sort: BookSort,
    is_grid_view: bool,
    is_1000_mode: bool,
    notification: Option<String>,
    grid_scroll_handle: UniformListScrollHandle,
    list_scroll_handle: UniformListScrollHandle,
    on_open_book: Option<Rc<dyn Fn(Book, &mut Window, &mut App) + 'static>>,
}

impl LibraryView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Buscar por título, autor o género...")
        });

        cx.subscribe(&search_input, |this, input, event: &InputEvent, cx| {
            if let InputEvent::Change = event {
                this.search_query = input.read(cx).value().to_string();
                this.update_filtered_books();
                cx.notify();
            }
        })
        .detach();

        let initial_books = mock_books();
        let mut this = Self {
            books: initial_books,
            filtered_books: Vec::new(),
            search_input,
            search_query: String::new(),
            filter: BookFilter::All,
            sort: BookSort::TitleAsc,
            is_grid_view: true,
            is_1000_mode: false,
            notification: None,
            grid_scroll_handle: UniformListScrollHandle::new(),
            list_scroll_handle: UniformListScrollHandle::new(),
            on_open_book: None,
        };
        this.update_filtered_books();
        this
    }

    pub fn set_on_open_book(&mut self, handler: impl Fn(Book, &mut Window, &mut App) + 'static) {
        self.on_open_book = Some(Rc::new(handler));
    }

    pub fn focus_search(&self, window: &mut Window, cx: &mut App) {
        self.search_input.update(cx, |input, cx| {
            input.focus(window, cx);
        });
    }

    fn update_filtered_books(&mut self) {
        let query = self.search_query.trim().to_lowercase();
        let current_filter = self.filter;

        let mut result: Vec<Book> = self
            .books
            .iter()
            .filter(|b| b.matches_filter(current_filter))
            .filter(|b| {
                if query.is_empty() {
                    true
                } else {
                    b.matches_search(&query)
                }
            })
            .cloned()
            .collect();

        match self.sort {
            BookSort::TitleAsc => {
                result.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
            }
            BookSort::TitleDesc => {
                result.sort_by(|a, b| b.title.to_lowercase().cmp(&a.title.to_lowercase()));
            }
            BookSort::AuthorAsc => {
                result.sort_by(|a, b| a.author.to_lowercase().cmp(&b.author.to_lowercase()));
            }
            BookSort::RecentProgress => {
                result.sort_by(|a, b| {
                    b.progress_pct()
                        .partial_cmp(&a.progress_pct())
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            BookSort::TotalPagesDesc => {
                result.sort_by(|a, b| b.total_pages.cmp(&a.total_pages));
            }
        }

        self.filtered_books = result;
    }

    pub fn set_filter(&mut self, filter: BookFilter, cx: &mut Context<Self>) {
        if self.filter != filter {
            self.filter = filter;
            self.update_filtered_books();
            cx.notify();
        }
    }

    pub fn cycle_sort(&mut self, cx: &mut Context<Self>) {
        self.sort = match self.sort {
            BookSort::TitleAsc => BookSort::TitleDesc,
            BookSort::TitleDesc => BookSort::AuthorAsc,
            BookSort::AuthorAsc => BookSort::RecentProgress,
            BookSort::RecentProgress => BookSort::TotalPagesDesc,
            BookSort::TotalPagesDesc => BookSort::TitleAsc,
        };
        self.update_filtered_books();
        cx.notify();
    }

    pub fn toggle_view_mode(&mut self, is_grid: bool, cx: &mut Context<Self>) {
        self.is_grid_view = is_grid;
        cx.notify();
    }

    pub fn toggle_1000_catalog(&mut self, cx: &mut Context<Self>) {
        self.is_1000_mode = !self.is_1000_mode;
        if self.is_1000_mode {
            self.books = generate_mock_catalog(1000);
            self.notification = Some("Catálogo de 1.000 libros cargado. Grilla virtualizada activa.".into());
        } else {
            self.books = mock_books();
            self.notification = None;
        }
        self.update_filtered_books();
        cx.notify();
    }

    pub fn show_import_notice(&mut self, cx: &mut Context<Self>) {
        self.notification = Some("Importar EPUB estará disponible en la siguiente fase".into());
        cx.notify();
    }

    pub fn clear_notification(&mut self, cx: &mut Context<Self>) {
        self.notification = None;
        cx.notify();
    }

    pub fn clear_filters(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.filter = BookFilter::All;
        self.search_query.clear();
        self.search_input.update(cx, |input, cx| {
            input.set_value("", window, cx);
        });
        self.update_filtered_books();
        cx.notify();
    }
}

impl Render for LibraryView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme_state = ThemeState::global(cx);
        let tokens = *theme_state.tokens();
        let radius = theme_state.radius();

        let total_count = self.books.len();
        let filtered_count = self.filtered_books.len();

        let view_entity = cx.entity().clone();

        let view_filter = view_entity.clone();
        let view_sort = view_entity.clone();
        let view_mode = view_entity.clone();
        let view_1000 = view_entity.clone();
        let view_import = view_entity.clone();
        let view_clear_notif = view_entity.clone();

        let toolbar = LibraryToolbar::new(
            "library-toolbar",
            tokens,
            radius,
            self.search_input.clone(),
            self.filter,
            self.sort,
            self.is_grid_view,
            self.is_1000_mode,
            total_count,
            filtered_count,
        )
        .on_filter(move |f, _, cx| {
            view_filter.update(cx, |this, cx| this.set_filter(f, cx));
        })
        .on_sort(move |_, cx| {
            view_sort.update(cx, |this, cx| this.cycle_sort(cx));
        })
        .on_toggle_view(move |is_grid, _, cx| {
            view_mode.update(cx, |this, cx| this.toggle_view_mode(is_grid, cx));
        })
        .on_toggle_1000(move |_, cx| {
            view_1000.update(cx, |this, cx| this.toggle_1000_catalog(cx));
        })
        .on_import(move |_, cx| {
            view_import.update(cx, |this, cx| this.show_import_notice(cx));
        });

        let notification_element = self.notification.as_ref().map(|msg| {
            let msg = msg.clone();
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .px(px(16.0))
                .py(px(10.0))
                .rounded(radius)
                .bg(tokens.bg_elevated)
                .border_1()
                .border_color(tokens.accent)
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(10.0))
                        .child(
                            Icon::new(IconName::Check)
                                .size(px(16.0))
                                .text_color(tokens.accent),
                        )
                        .child(
                            div()
                                .text_sm()
                                .font_medium()
                                .text_color(tokens.text)
                                .child(msg),
                        ),
                )
                .child(
                    div()
                        .id("btn-close-notification")
                        .p(px(4.0))
                        .rounded_full()
                        .hover(|s| s.bg(tokens.bg_hover))
                        .cursor_pointer()
                        .on_click(move |_, _, cx| {
                            view_clear_notif.update(cx, |this, cx| this.clear_notification(cx));
                        })
                        .child(
                            Icon::new(IconName::Close)
                                .size(px(14.0))
                                .text_color(tokens.text_muted),
                        ),
                )
        });

        // Content Area: Empty State vs Grid vs List
        let content_element = if filtered_count == 0 {
            let view_reset = view_entity.clone();
            div()
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(16.0))
                .py(px(64.0))
                .rounded(radius)
                .border_1()
                .border_color(tokens.border)
                .bg(tokens.bg_elevated)
                .child(
                    Icon::new(IconName::BookOpen)
                        .size(px(48.0))
                        .text_color(tokens.text_muted),
                )
                .child(
                    div()
                        .text_lg()
                        .font_semibold()
                        .text_color(tokens.text)
                        .child("No se encontraron libros"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(tokens.text_muted)
                        .child("Prueba modificando el término de búsqueda o seleccionando otro filtro."),
                )
                .child(
                    div()
                        .id("btn-reset-filters")
                        .mt(px(8.0))
                        .px(px(16.0))
                        .py(px(8.0))
                        .rounded(radius)
                        .bg(tokens.bg_sunken)
                        .border_1()
                        .border_color(tokens.border)
                        .hover(|s| s.bg(tokens.bg_hover))
                        .cursor_pointer()
                        .on_click(move |_, window, cx| {
                            view_reset.update(cx, |this, cx| this.clear_filters(window, cx));
                        })
                        .child(
                            div()
                                .text_sm()
                                .font_medium()
                                .text_color(tokens.accent)
                                .child("Restablecer filtros"),
                        ),
                )
                .into_any_element()
        } else if self.is_grid_view {
            // Dynamic columns calculation:
            // Available width = Window width - (Sidebar approx 195px + padding 56px + scrollbar 20px)
            let viewport_w = window.viewport_size().width;
            let available_w = (viewport_w - px(270.0)).max(px(320.0));
            // Each card is 160px + 20px gap = 180px
            let items_per_row = ((available_w.as_f32() + 20.0) / 180.0).floor().max(2.0) as usize;

            let row_count = (filtered_count + items_per_row - 1) / items_per_row;
            let books_rc = Rc::new(self.filtered_books.clone());
            let on_open_book = self.on_open_book.clone();

            uniform_list(
                "library-virtual-grid",
                row_count,
                move |range, _window, _cx| {
                    let mut rows = Vec::with_capacity(range.len());
                    for row_idx in range {
                        let start = row_idx * items_per_row;
                        let end = (start + items_per_row).min(books_rc.len());
                        let row_books = &books_rc[start..end];

                        let on_open_book_row = on_open_book.clone();
                        let row_div = div()
                            .id(ElementId::from(format!("book-row-{}", row_idx)))
                            .h(px(320.0))
                            .flex()
                            .flex_row()
                            .gap(px(20.0))
                            .pb(px(20.0))
                            .children(row_books.iter().map(|book| {
                                let book_for_click = book.clone();
                                let cb = on_open_book_row.clone();
                                BookCard::new(
                                    ElementId::from(book.id.clone()),
                                    book.clone(),
                                    tokens,
                                    radius,
                                )
                                .on_click(move |_, window, cx| {
                                    if let Some(ref handler) = cb {
                                        handler(book_for_click.clone(), window, cx);
                                    }
                                })
                            }));

                        rows.push(row_div);
                    }
                    rows
                },
            )
            .track_scroll(&self.grid_scroll_handle)
            .size_full()
            .flex_1()
            .into_any_element()
        } else {
            // Virtualized List mode: 1 book per row
            let books_rc = Rc::new(self.filtered_books.clone());
            let on_open_book = self.on_open_book.clone();

            uniform_list(
                "library-virtual-list",
                filtered_count,
                move |range, _window, _cx| {
                    let mut rows = Vec::with_capacity(range.len());
                    for idx in range {
                        if let Some(book) = books_rc.get(idx) {
                            let book_for_click = book.clone();
                            let cb = on_open_book.clone();
                            let row_element = div()
                                .id(ElementId::from(format!("book-list-row-{}", idx)))
                                .h(px(76.0))
                                .pb(px(8.0))
                                .child(
                                    BookRow::new(
                                        ElementId::from(book.id.clone()),
                                        book.clone(),
                                        tokens,
                                        radius,
                                    )
                                    .on_click(move |_, window, cx| {
                                        if let Some(ref handler) = cb {
                                            handler(book_for_click.clone(), window, cx);
                                        }
                                    }),
                                );
                            rows.push(row_element);
                        }
                    }
                    rows
                },
            )
            .track_scroll(&self.list_scroll_handle)
            .size_full()
            .flex_1()
            .into_any_element()
        };

        div()
            .flex()
            .flex_col()
            .size_full()
            .px(px(28.0))
            .pt(px(20.0))
            .pb(px(16.0))
            .gap(px(16.0))
            .child(toolbar)
            .children(notification_element)
            .child(content_element)
    }
}
