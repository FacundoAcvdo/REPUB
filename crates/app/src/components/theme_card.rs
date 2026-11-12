use gpui_kit::base::StyledExt as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::theme::tokens::ThemeTokens;

#[derive(IntoElement)]
pub struct ThemeCard {
    id: ElementId,
    tokens: ThemeTokens,
    is_selected: bool,
    on_click: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
}

impl ThemeCard {
    pub fn new(id: impl Into<ElementId>, tokens: ThemeTokens, is_selected: bool) -> Self {
        Self {
            id: id.into(),
            tokens,
            is_selected,
            on_click: None,
        }
    }

    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for ThemeCard {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let is_selected = self.is_selected;
        let tokens = self.tokens;

        div()
            .id(self.id)
            .flex()
            .flex_col()
            .items_center()
            .gap_1()
            .cursor_pointer()
            .when_some(self.on_click, |this, handler| {
                this.on_click(move |_, window, cx| handler(window, cx))
            })
            // Mini preview box (approx 96x64 px)
            .child(
                div()
                    .w(px(96.0))
                    .h(px(64.0))
                    .rounded(px(8.0))
                    .overflow_hidden()
                    .bg(tokens.bg)
                    .border_2()
                    .border_color(if is_selected {
                        tokens.accent
                    } else {
                        tokens.border
                    })
                    .flex()
                    .flex_row()
                    .p(px(4.0))
                    .gap(px(4.0))
                    // Mock Sidebar
                    .child(
                        div()
                            .w(px(22.0))
                            .h_full()
                            .rounded(px(4.0))
                            .bg(tokens.bg_sunken)
                            .flex()
                            .flex_col()
                            .gap(px(3.0))
                            .p(px(2.0))
                            .child(div().w_full().h(px(3.0)).rounded_full().bg(tokens.text_muted))
                            .child(div().w_full().h(px(3.0)).rounded_full().bg(tokens.border))
                            .child(div().w_full().h(px(3.0)).rounded_full().bg(tokens.border)),
                    )
                    // Mock Main Area
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .flex()
                            .flex_col()
                            .gap(px(3.0))
                            // Mock Header
                            .child(
                                div()
                                    .w_full()
                                    .h(px(10.0))
                                    .rounded(px(3.0))
                                    .bg(tokens.bg_elevated)
                                    .flex()
                                    .items_center()
                                    .px(px(2.0))
                                    .child(div().w(px(16.0)).h(px(3.0)).rounded_full().bg(tokens.accent)),
                            )
                            // Mock Cards Grid
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_row()
                                    .gap(px(3.0))
                                    .child(
                                        div()
                                            .flex_1()
                                            .h_full()
                                            .rounded(px(3.0))
                                            .bg(tokens.bg_elevated),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .h_full()
                                            .rounded(px(3.0))
                                            .bg(tokens.bg_elevated),
                                    ),
                            ),
                    ),
            )
            // Label
            .child(
                div()
                    .text_xs()
                    .font_medium()
                    .text_color(if is_selected {
                        tokens.accent
                    } else {
                        tokens.text_muted
                    })
                    .child(tokens.name),
            )
    }
}
