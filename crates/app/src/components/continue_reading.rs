use std::rc::Rc;
use epub_core::domain::book::Book;
use gpui_kit::assets::IconName;
use gpui_kit::base::StyledExt as _;
use gpui_kit::component::Icon;
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
pub struct ContinueReadingCard {
    id: ElementId,
    book: Book,
    cover_path: Option<String>,
    tokens: ThemeTokens,
    radius: Pixels,
    on_read: Option<Rc<dyn Fn(&mut Window, &mut App) + 'static>>,
}

impl ContinueReadingCard {
    pub fn new(
        id: impl Into<ElementId>,
        book: Book,
        tokens: ThemeTokens,
        radius: Pixels,
    ) -> Self {
        Self {
            id: id.into(),
            book,
            cover_path: None,
            tokens,
            radius,
            on_read: None,
        }
    }

    pub fn cover_path(mut self, path: Option<String>) -> Self {
        self.cover_path = path;
        self
    }

    pub fn on_read(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_read = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for ContinueReadingCard {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let tokens = self.tokens;
        let radius = self.radius;
        let book = &self.book;
        let progress = book.progress_pct();
        let on_read = self.on_read;
        let real_cover = self
            .cover_path
            .filter(|p| !p.is_empty() && std::path::Path::new(p).exists())
            .map(std::path::PathBuf::from);
        let has_real_cover = real_cover.is_some();

        let primary_color = hex_to_hsla(book.cover_palette.primary, 1.0);
        let secondary_color = hex_to_hsla(book.cover_palette.secondary, 1.0);

        div()
            .id(self.id)
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .p(px(20.0))
            .gap(px(24.0))
            .rounded(radius)
            .bg(tokens.bg_elevated)
            .border_1()
            .border_color(tokens.border)
            .hover(|s| s.bg(tokens.bg_hover).border_color(tokens.accent))
            .cursor_pointer()
            .when_some(on_read.clone(), |this, handler| {
                this.on_click(move |_, window, cx| handler(window, cx))
            })
            // Left: Book Cover Miniature (Proportion 2:3, ~110px x 165px)
            .child(
                div()
                    .flex_shrink_0()
                    .w(px(110.0))
                    .h(px(165.0))
                    .rounded(px(6.0))
                    .bg(primary_color)
                    .border_1()
                    .border_color(tokens.border)
                    .overflow_hidden()
                    .relative()
                    .shadow_md()
                    .when_some(real_cover, |this, path| {
                        this.child(
                            img(path)
                                .absolute()
                                .inset_0()
                                .size_full()
                                .object_fit(ObjectFit::Cover),
                        )
                    })
                    // Spine crease shadow on left edge
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .left_0()
                            .bottom_0()
                            .w(px(7.0))
                            .bg(hsla(0.0, 0.0, 0.0, 0.28)),
                    )
                    // Decorative geometric circle & procedural text (only when no real cover)
                    .when(!has_real_cover, |this| {
                        this.child(
                            div()
                                .absolute()
                                .top_0()
                                .right_0()
                                .w(px(65.0))
                                .h(px(65.0))
                                .rounded_full()
                                .bg(secondary_color)
                                .opacity(0.35),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .justify_between()
                                .size_full()
                                .p(px(10.0))
                                .pl(px(14.0))
                                .child(
                                    div()
                                        .px(px(6.0))
                                        .py(px(2.0))
                                        .rounded_full()
                                        .bg(hsla(0.0, 0.0, 0.0, 0.45))
                                        .text_xs()
                                        .font_bold()
                                        .text_color(hsla(0.0, 0.0, 1.0, 0.95))
                                        .child(book.genre.name_es()),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap(px(2.0))
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_bold()
                                                .text_color(hsla(0.0, 0.0, 1.0, 0.95))
                                                .line_clamp(2)
                                                .child(book.title.clone()),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(hsla(0.0, 0.0, 1.0, 0.75))
                                                .child(book.author.clone()),
                                        ),
                                ),
                        )
                    }),
            )
            // Right: Book Info, Progress, and Action Button
            .child(
                div()
                    .flex()
                    .flex_col()
                    .justify_between()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .gap(px(12.0))
                    // Header pill
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .px(px(10.0))
                                    .py(px(4.0))
                                    .rounded_full()
                                    .bg(tokens.bg_sunken)
                                    .border_1()
                                    .border_color(tokens.accent)
                                    .text_xs()
                                    .font_bold()
                                    .text_color(tokens.accent)
                                    .child("CONTINUAR LEYENDO"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(tokens.text_muted)
                                    .child("Última lectura: Hoy"),
                            ),
                    )
                    // Title and Author
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(
                                div()
                                    .text_xl()
                                    .font_bold()
                                    .text_color(tokens.text)
                                    .whitespace_normal()
                                    .child(book.title.clone()),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(tokens.text_muted)
                                    .child(format!("Por {} • Género: {}", book.author, book.genre.name_es())),
                            ),
                    )
                    // Progress bar & details
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(6.0))
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .text_xs()
                                    .font_medium()
                                    .child(
                                        div()
                                            .text_color(tokens.text)
                                            .child(format!("Página {} de {} páginas ({:.0}%)", book.current_page, book.total_pages, progress * 100.0)),
                                    )
                                    .child(
                                        div()
                                            .text_color(tokens.text_muted)
                                            .child("~1 h 45 min restantes"),
                                    ),
                            )
                            // Progress bar track
                            .child(
                                div()
                                    .relative()
                                    .w_full()
                                    .h(px(6.0))
                                    .rounded_full()
                                    .bg(tokens.bg_sunken)
                                    .overflow_hidden()
                                    .child(
                                        div()
                                            .h_full()
                                            .w(relative(progress))
                                            .rounded_full()
                                            .bg(tokens.accent),
                                    ),
                            ),
                    )
                    // Action button
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(10.0))
                            .pt(px(4.0))
                            .child(
                                div()
                                    .id("btn-resume-reading")
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(8.0))
                                    .px(px(18.0))
                                    .py(px(8.0))
                                    .rounded(radius)
                                    .bg(tokens.accent)
                                    .hover(|s| s.bg(tokens.accent_hover))
                                    .cursor_pointer()
                                    .when_some(on_read, |this, handler| {
                                        this.on_click(move |_, window, cx| handler(window, cx))
                                    })
                                    .child(
                                        Icon::new(IconName::Play)
                                            .size(px(14.0))
                                            .text_color(tokens.accent_text),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_semibold()
                                            .text_color(tokens.accent_text)
                                            .child("Reanudar lectura"),
                                    ),
                            ),
                    ),
            )
    }
}
