use std::rc::Rc;
use epub_core::domain::book::{BookFilter, BookSort};
use gpui_kit::assets::IconName;
use gpui_kit::base::StyledExt as _;
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::Icon;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::theme::tokens::ThemeTokens;

#[derive(IntoElement)]
pub struct LibraryToolbar {
    id: ElementId,
    tokens: ThemeTokens,
    radius: Pixels,
    search_input: Entity<InputState>,
    current_filter: BookFilter,
    current_sort: BookSort,
    is_grid_view: bool,
    is_1000_mode: bool,
    total_books: usize,
    filtered_books: usize,
    on_filter: Option<Rc<dyn Fn(BookFilter, &mut Window, &mut App) + 'static>>,
    on_sort: Option<Rc<dyn Fn(&mut Window, &mut App) + 'static>>,
    on_toggle_view: Option<Rc<dyn Fn(bool, &mut Window, &mut App) + 'static>>,
    on_toggle_1000: Option<Rc<dyn Fn(&mut Window, &mut App) + 'static>>,
    on_import: Option<Rc<dyn Fn(&mut Window, &mut App) + 'static>>,
}

impl LibraryToolbar {
    pub fn new(
        id: impl Into<ElementId>,
        tokens: ThemeTokens,
        radius: Pixels,
        search_input: Entity<InputState>,
        current_filter: BookFilter,
        current_sort: BookSort,
        is_grid_view: bool,
        is_1000_mode: bool,
        total_books: usize,
        filtered_books: usize,
    ) -> Self {
        Self {
            id: id.into(),
            tokens,
            radius,
            search_input,
            current_filter,
            current_sort,
            is_grid_view,
            is_1000_mode,
            total_books,
            filtered_books,
            on_filter: None,
            on_sort: None,
            on_toggle_view: None,
            on_toggle_1000: None,
            on_import: None,
        }
    }

    pub fn on_filter(
        mut self,
        handler: impl Fn(BookFilter, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_filter = Some(Rc::new(handler));
        self
    }

    pub fn on_sort(
        mut self,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_sort = Some(Rc::new(handler));
        self
    }

    pub fn on_toggle_view(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle_view = Some(Rc::new(handler));
        self
    }

    pub fn on_toggle_1000(
        mut self,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle_1000 = Some(Rc::new(handler));
        self
    }

    pub fn on_import(
        mut self,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_import = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for LibraryToolbar {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let tokens = self.tokens;
        let radius = self.radius;
        let current_filter = self.current_filter;
        let on_filter = self.on_filter;
        let on_sort = self.on_sort;
        let on_toggle_view = self.on_toggle_view;
        let on_toggle_1000 = self.on_toggle_1000;
        let on_import = self.on_import;
        let is_grid = self.is_grid_view;
        let is_1000 = self.is_1000_mode;

        div()
            .id(self.id)
            .flex()
            .flex_col()
            .gap(px(12.0))
            .w_full()
            // Top Row: Title & Stats on Left, Search + Actions on Right
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .gap(px(16.0))
                    // Left: Title + Count
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_baseline()
                            .gap(px(12.0))
                            .child(
                                div()
                                    .text_2xl()
                                    .font_bold()
                                    .text_color(tokens.text)
                                    .child("Biblioteca"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .font_medium()
                                    .text_color(tokens.text_muted)
                                    .child(if self.filtered_books == self.total_books {
                                        format!("{} libros", self.total_books)
                                    } else {
                                        format!("{}/{} libros", self.filtered_books, self.total_books)
                                    }),
                            ),
                    )
                    // Right: Search Input + 1,000 Catalog Button + Import Button
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(10.0))
                            // Search input container
                            .child(
                                div()
                                    .w(px(240.0))
                                    .child(Input::new(&self.search_input)),
                            )
                            // Toggle 1,000 mock catalog button
                            .child(
                                div()
                                    .id("btn-toggle-1000")
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(6.0))
                                    .px(px(12.0))
                                    .py(px(6.0))
                                    .rounded(radius)
                                    .border_1()
                                    .border_color(if is_1000 {
                                        tokens.accent
                                    } else {
                                        tokens.border
                                    })
                                    .bg(if is_1000 {
                                        tokens.bg_hover
                                    } else {
                                        tokens.bg_elevated
                                    })
                                    .hover(|s| s.bg(tokens.bg_hover))
                                    .cursor_pointer()
                                    .when_some(on_toggle_1000, |this, handler| {
                                        this.on_click(move |_, window, cx| handler(window, cx))
                                    })
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_medium()
                                            .text_color(if is_1000 {
                                                tokens.accent
                                            } else {
                                                tokens.text
                                            })
                                            .child(if is_1000 {
                                                "1.000 libros (Activo)"
                                            } else {
                                                "Probar 1.000 libros"
                                            }),
                                    ),
                            )
                            // + Importar EPUB Button
                            .child(
                                div()
                                    .id("btn-import-epub")
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(6.0))
                                    .px(px(14.0))
                                    .py(px(6.0))
                                    .rounded(radius)
                                    .bg(tokens.accent)
                                    .hover(|s| s.bg(tokens.accent_hover))
                                    .cursor_pointer()
                                    .when_some(on_import, |this, handler| {
                                        this.on_click(move |_, window, cx| handler(window, cx))
                                    })
                                    .child(
                                        Icon::new(IconName::Plus)
                                            .size(px(15.0))
                                            .text_color(tokens.accent_text),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_semibold()
                                            .text_color(tokens.accent_text)
                                            .child("Importar EPUB"),
                                    ),
                            ),
                    ),
            )
            // Bottom Row: Filter Pills on Left, Sort & View Mode on Right
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .gap(px(16.0))
                    // Filter Pills
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(6.0))
                            .children(BookFilter::all().iter().map(move |filter| {
                                let f = *filter;
                                let is_active = f == current_filter;
                                let on_filter_clone = on_filter.clone();

                                div()
                                    .id(ElementId::from(format!("filter-{:?}", f)))
                                    .px(px(12.0))
                                    .py(px(5.0))
                                    .rounded_full()
                                    .cursor_pointer()
                                    .text_xs()
                                    .font_medium()
                                    .when(is_active, |s| {
                                        s.bg(tokens.bg_elevated)
                                            .text_color(tokens.accent)
                                            .border_1()
                                            .border_color(tokens.accent)
                                    })
                                    .when(!is_active, |s| {
                                        s.bg(tokens.bg_sunken)
                                            .text_color(tokens.text_muted)
                                            .border_1()
                                            .border_color(tokens.border)
                                            .hover(|s| s.text_color(tokens.text).bg(tokens.bg_hover))
                                    })
                                    .when_some(on_filter_clone, move |this, handler| {
                                        this.on_click(move |_, window, cx| handler(f, window, cx))
                                    })
                                    .child(f.name_es())
                            })),
                    )
                    // Right: Sort Button + Grid/List Toggle
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(8.0))
                            // Sort cycle button
                            .child(
                                div()
                                    .id("btn-sort-cycle")
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(6.0))
                                    .px(px(12.0))
                                    .py(px(5.0))
                                    .rounded(radius)
                                    .border_1()
                                    .border_color(tokens.border)
                                    .bg(tokens.bg_sunken)
                                    .hover(|s| s.bg(tokens.bg_hover))
                                    .cursor_pointer()
                                    .when_some(on_sort, |this, handler| {
                                        this.on_click(move |_, window, cx| handler(window, cx))
                                    })
                                    .child(
                                        Icon::new(IconName::ArrowUpDown)
                                            .size(px(14.0))
                                            .text_color(tokens.text_muted),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_medium()
                                            .text_color(tokens.text)
                                            .child(self.current_sort.name_es()),
                                    ),
                            )
                            // View Mode Toggle (Grid / List)
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .p(px(2.0))
                                    .gap(px(2.0))
                                    .rounded(radius)
                                    .border_1()
                                    .border_color(tokens.border)
                                    .bg(tokens.bg_sunken)
                                    // Grid Icon Button
                                    .child({
                                        let on_view = on_toggle_view.clone();
                                        div()
                                            .id("btn-view-grid")
                                            .p(px(5.0))
                                            .rounded(radius)
                                            .cursor_pointer()
                                            .when(is_grid, |s| s.bg(tokens.bg_elevated).border_1().border_color(tokens.border))
                                            .when(!is_grid, |s| s.hover(|s| s.bg(tokens.bg_hover)))
                                            .when_some(on_view, |this, handler| {
                                                this.on_click(move |_, window, cx| handler(true, window, cx))
                                            })
                                            .child(
                                                Icon::new(IconName::LayoutGrid)
                                                    .size(px(15.0))
                                                    .text_color(if is_grid { tokens.accent } else { tokens.text_muted }),
                                            )
                                    })
                                    // List Icon Button
                                    .child({
                                        let on_view = on_toggle_view;
                                        div()
                                            .id("btn-view-list")
                                            .p(px(5.0))
                                            .rounded(radius)
                                            .cursor_pointer()
                                            .when(!is_grid, |s| s.bg(tokens.bg_elevated).border_1().border_color(tokens.border))
                                            .when(is_grid, |s| s.hover(|s| s.bg(tokens.bg_hover)))
                                            .when_some(on_view, |this, handler| {
                                                this.on_click(move |_, window, cx| handler(false, window, cx))
                                            })
                                            .child(
                                                Icon::new(IconName::List)
                                                    .size(px(15.0))
                                                    .text_color(if !is_grid { tokens.accent } else { tokens.text_muted }),
                                            )
                                    }),
                            ),
                    ),
            )
    }
}
