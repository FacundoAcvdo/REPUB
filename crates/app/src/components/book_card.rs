use std::rc::Rc;
use epub_core::domain::book::Book;
use gpui_kit::base::StyledExt as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::theme::tokens::ThemeTokens;

fn hex_to_hsla(hex: u32, alpha: f32) -> Hsla {
    let r = ((hex >> 16) & 0xff) as f32 / 255.0;
    let g = ((hex >> 8) & 0xff) as f32 / 255.0;
    let b = (hex & 0xff) as f32 / 255.0;

    let max = r.max(g.max(b));
    let min = r.min(g.min(b));
    let delta = max - min;

    let l = (max + min) / 2.0;

    let s = if delta == 0.0 {
        0.0
    } else if l < 0.5 {
        delta / (max + min)
    } else {
        delta / (2.0 - max - min)
    };

    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        let seg = (g - b) / delta;
        if seg < 0.0 {
            (seg + 6.0) / 6.0
        } else {
            seg / 6.0
        }
    } else if max == g {
        ((b - r) / delta + 2.0) / 6.0
    } else {
        ((r - g) / delta + 4.0) / 6.0
    };

    hsla(h, s, l, alpha)
}

#[derive(IntoElement)]
pub struct BookCard {
    id: ElementId,
    book: Book,
    tokens: ThemeTokens,
    radius: Pixels,
    cover_path: Option<String>,
    is_favorite: bool,
    on_click: Option<Rc<dyn Fn(&str, &mut Window, &mut App) + 'static>>,
    on_toggle_favorite: Option<Rc<dyn Fn(&str, &mut Window, &mut App) + 'static>>,
}

impl BookCard {
    pub fn new(
        id: impl Into<ElementId>,
        book: Book,
        tokens: ThemeTokens,
        radius: Pixels,
    ) -> Self {
        Self {
            id: id.into(),
            book,
            tokens,
            radius,
            cover_path: None,
            is_favorite: false,
            on_click: None,
            on_toggle_favorite: None,
        }
    }

    pub fn cover_path(mut self, path: Option<String>) -> Self {
        self.cover_path = path;
        self
    }

    pub fn is_favorite(mut self, is_favorite: bool) -> Self {
        self.is_favorite = is_favorite;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    pub fn on_toggle_favorite(
        mut self,
        handler: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle_favorite = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for BookCard {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let book = self.book;
        let tokens = self.tokens;
        let radius = self.radius;
        let book_id = book.id.clone();
        let on_click = self.on_click;
        let on_toggle_favorite = self.on_toggle_favorite;
        let is_favorite = self.is_favorite;
        let real_cover = self
            .cover_path
            .filter(|p| !p.is_empty() && std::path::Path::new(p).exists())
            .map(std::path::PathBuf::from);
        let has_real_cover = real_cover.is_some();

        let primary_color = hex_to_hsla(book.cover_palette.primary, 1.0);
        let secondary_color = hex_to_hsla(book.cover_palette.secondary, 0.85);
        let accent_color = hex_to_hsla(book.cover_palette.accent, 1.0);
        let text_color = hex_to_hsla(book.cover_palette.text, 0.95);

        let progress = book.progress_pct();

        div()
            .id(self.id)
            .flex()
            .flex_col()
            .w(px(160.0))
            .cursor_pointer()
            .group("book-card")
            .when_some(on_click, |this, handler| {
                let id = book_id.clone();
                this.on_click(move |_, window, cx| handler(&id, window, cx))
            })
            // 2:3 Cover Container
            .child(
                div()
                    .w(px(160.0))
                    .h(px(240.0))
                    .rounded(radius)
                    .overflow_hidden()
                    .border_1()
                    .border_color(tokens.border)
                    .bg(primary_color)
                    .relative()
                    .flex()
                    .flex_col()
                    .justify_between()
                    .p(px(12.0))
                    .group_hover("book-card", |s| {
                        s.border_color(tokens.accent)
                    })
                    .when_some(real_cover, |this, path| {
                        this.child(
                            img(path)
                                .absolute()
                                .inset_0()
                                .size_full()
                                .object_fit(ObjectFit::Cover),
                        )
                    })
                    // Book Spine Crease Effect (left 8px)
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .bottom_0()
                            .left_0()
                            .w(px(9.0))
                            .flex()
                            .flex_row()
                            .child(
                                div()
                                    .w(px(7.0))
                                    .h_full()
                                    .bg(hsla(0.0, 0.0, 0.0, 0.35)),
                            )
                            .child(
                                div()
                                    .w(px(2.0))
                                    .h_full()
                                    .bg(hsla(0.0, 0.0, 1.0, 0.12)),
                            ),
                    )
                    // Top: Favorite star & Genre badge
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .pl(px(4.0))
                            .child(
                                div()
                                    .id(ElementId::from(format!("fav-card-{}", book_id)))
                                    .w(px(22.0))
                                    .h(px(22.0))
                                    .rounded_full()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .bg(hsla(0.0, 0.0, 0.0, 0.35))
                                    .border_1()
                                    .border_color(hsla(0.0, 0.0, 1.0, 0.15))
                                    .text_size(px(11.0))
                                    .text_color(if is_favorite {
                                        tokens.accent
                                    } else {
                                        hsla(0.0, 0.0, 1.0, 0.75)
                                    })
                                    .when_some(on_toggle_favorite, |this, handler| {
                                        let id = book_id.clone();
                                        this.on_click(move |_, window, cx| {
                                            cx.stop_propagation();
                                            handler(&id, window, cx);
                                        })
                                    })
                                    .child(if is_favorite { "★" } else { "☆" }),
                            )
                            .child(
                                div()
                                    .px(px(6.0))
                                    .py(px(2.0))
                                    .rounded_full()
                                    .bg(hsla(0.0, 0.0, 0.0, 0.45))
                                    .border_1()
                                    .border_color(hsla(0.0, 0.0, 1.0, 0.15))
                                    .text_size(px(9.0))
                                    .font_bold()
                                    .text_color(accent_color)
                                    .child(book.genre.name_es().to_uppercase()),
                            ),
                    )
                    // Center: Procedural Title & decorative lines (only when no real cover image)
                    .when(!has_real_cover, |this| {
                        this.child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .justify_center()
                                .gap(px(6.0))
                                .px(px(6.0))
                                // Subtle geometric accent icon or line
                                .child(
                                    div()
                                        .w(px(28.0))
                                        .h(px(2.0))
                                        .rounded_full()
                                        .bg(accent_color),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .font_bold()
                                        .text_color(text_color)
                                        .text_center()
                                        .line_height(relative(1.2))
                                        .child(book.title.clone()),
                                ),
                        )
                    })
                    // Bottom: Author on cover (if procedural) & discrete 2px progress bar
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(6.0))
                            .when(!has_real_cover, |this| {
                                this.child(
                                    div()
                                        .text_size(px(11.0))
                                        .text_center()
                                        .text_color(secondary_color)
                                        .child(book.author.clone()),
                                )
                            })
                            // 2px Progress bar at bottom of cover
                            .child(
                                div()
                                    .w_full()
                                    .h(px(2.0))
                                    .rounded_full()
                                    .bg(hsla(0.0, 0.0, 0.0, 0.4))
                                    .child(
                                        div()
                                            .h_full()
                                            .w(relative(progress))
                                            .rounded_full()
                                            .bg(if book.is_completed() {
                                                tokens.success
                                            } else {
                                                tokens.accent
                                            }),
                                    ),
                            ),
                    ),
            )
            // Information block below the cover
            .child(
                div()
                    .flex()
                    .flex_col()
                    .pt(px(8.0))
                    .gap(px(2.0))
                    .child(
                        div()
                            .text_sm()
                            .font_medium()
                            .text_color(tokens.text)
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(book.title.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(tokens.text_muted)
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(book.author.clone()),
                    )
                    .child(
                        div()
                            .pt(px(2.0))
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .text_size(px(10.5))
                            .child(
                                if book.is_completed() {
                                    div()
                                        .font_medium()
                                        .text_color(tokens.success)
                                        .child("✓ Completado")
                                } else if book.is_reading() {
                                    div()
                                        .text_color(tokens.text_muted)
                                        .child(format!(
                                            "{}% • {}/{} pág",
                                            (progress * 100.0) as u32,
                                            book.current_page,
                                            book.total_pages
                                        ))
                                } else {
                                    div()
                                        .text_color(tokens.text_muted)
                                        .child(format!("{} páginas", book.total_pages))
                                },
                            ),
                    ),
            )
    }
}

#[derive(IntoElement)]
pub struct BookRow {
    id: ElementId,
    book: Book,
    tokens: ThemeTokens,
    radius: Pixels,
    cover_path: Option<String>,
    is_favorite: bool,
    on_click: Option<Rc<dyn Fn(&str, &mut Window, &mut App) + 'static>>,
    on_toggle_favorite: Option<Rc<dyn Fn(&str, &mut Window, &mut App) + 'static>>,
}

impl BookRow {
    pub fn new(
        id: impl Into<ElementId>,
        book: Book,
        tokens: ThemeTokens,
        radius: Pixels,
    ) -> Self {
        Self {
            id: id.into(),
            book,
            tokens,
            radius,
            cover_path: None,
            is_favorite: false,
            on_click: None,
            on_toggle_favorite: None,
        }
    }

    pub fn cover_path(mut self, path: Option<String>) -> Self {
        self.cover_path = path;
        self
    }

    pub fn is_favorite(mut self, is_favorite: bool) -> Self {
        self.is_favorite = is_favorite;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    pub fn on_toggle_favorite(
        mut self,
        handler: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle_favorite = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for BookRow {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let book = self.book;
        let tokens = self.tokens;
        let radius = self.radius;
        let book_id = book.id.clone();
        let on_click = self.on_click;
        let on_toggle_favorite = self.on_toggle_favorite;
        let is_favorite = self.is_favorite;
        let real_cover = self
            .cover_path
            .filter(|p| !p.is_empty() && std::path::Path::new(p).exists())
            .map(std::path::PathBuf::from);

        let primary_color = hex_to_hsla(book.cover_palette.primary, 1.0);
        let accent_color = hex_to_hsla(book.cover_palette.accent, 1.0);
        let progress = book.progress_pct();

        div()
            .id(self.id)
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .w_full()
            .px(px(16.0))
            .py(px(10.0))
            .rounded(radius)
            .border_1()
            .border_color(tokens.border)
            .bg(tokens.bg_elevated)
            .hover(|s| s.bg(tokens.bg_hover).border_color(tokens.accent))
            .cursor_pointer()
            .when_some(on_click, |this, handler| {
                let id = book_id.clone();
                this.on_click(move |_, window, cx| handler(&id, window, cx))
            })
            // Left: Thumbnail + Title + Author
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(16.0))
                    // Mini thumbnail (40x60 px)
                    .child(
                        div()
                            .w(px(40.0))
                            .h(px(60.0))
                            .rounded(px(4.0))
                            .overflow_hidden()
                            .bg(primary_color)
                            .border_1()
                            .border_color(tokens.border)
                            .flex()
                            .items_center()
                            .justify_center()
                            .map(|this| {
                                if let Some(path) = real_cover {
                                    this.child(
                                        img(path)
                                            .size_full()
                                            .object_fit(ObjectFit::Cover),
                                    )
                                } else {
                                    this.child(
                                        div()
                                            .w(px(16.0))
                                            .h(px(2.0))
                                            .rounded_full()
                                            .bg(accent_color),
                                    )
                                }
                            }),
                    )
                    // Details
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(
                                div()
                                    .text_sm()
                                    .font_medium()
                                    .text_color(tokens.text)
                                    .child(book.title.clone()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(tokens.text_muted)
                                    .child(book.author.clone()),
                            ),
                    ),
            )
            // Right: Favorite + Genre Badge + Progress bar + Status
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(16.0))
                    .child(
                        div()
                            .id(ElementId::from(format!("fav-row-{}", book_id)))
                            .w(px(26.0))
                            .h(px(26.0))
                            .rounded_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .hover(|s| s.bg(tokens.bg_sunken))
                            .text_size(px(13.0))
                            .text_color(if is_favorite {
                                tokens.accent
                            } else {
                                tokens.text_muted
                            })
                            .when_some(on_toggle_favorite, |this, handler| {
                                let id = book_id.clone();
                                this.on_click(move |_, window, cx| {
                                    cx.stop_propagation();
                                    handler(&id, window, cx);
                                })
                            })
                            .child(if is_favorite { "★" } else { "☆" }),
                    )
                    // Genre badge
                    .child(
                        div()
                            .px(px(8.0))
                            .py(px(3.0))
                            .rounded_full()
                            .bg(tokens.bg_sunken)
                            .border_1()
                            .border_color(tokens.border)
                            .text_xs()
                            .font_medium()
                            .text_color(tokens.text_muted)
                            .child(book.genre.name_es()),
                    )
                    // Progress bar & text
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_end()
                            .gap(px(4.0))
                            .w(px(130.0))
                            .child(
                                div()
                                    .text_xs()
                                    .font_medium()
                                    .text_color(if book.is_completed() {
                                        tokens.success
                                    } else {
                                        tokens.text
                                    })
                                    .child(if book.is_completed() {
                                        "Completado".to_string()
                                    } else if book.is_reading() {
                                        format!("{}% ({}/{})", (progress * 100.0) as u32, book.current_page, book.total_pages)
                                    } else {
                                        format!("{} págs", book.total_pages)
                                    }),
                            )
                            .child(
                                div()
                                    .w_full()
                                    .h(px(3.0))
                                    .rounded_full()
                                    .bg(tokens.bg_sunken)
                                    .child(
                                        div()
                                            .h_full()
                                            .w(relative(progress))
                                            .rounded_full()
                                            .bg(if book.is_completed() {
                                                tokens.success
                                            } else {
                                                tokens.accent
                                            }),
                                    ),
                            ),
                    ),
            )
    }
}
