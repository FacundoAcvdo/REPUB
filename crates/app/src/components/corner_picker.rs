use epub_core::domain::settings::CornerRadiusStyle;
use gpui_kit::base::StyledExt as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::theme::radius::RadiusTokens;
use crate::theme::tokens::ThemeTokens;

#[derive(IntoElement)]
pub struct CornerCard {
    id: ElementId,
    style: CornerRadiusStyle,
    tokens: ThemeTokens,
    is_selected: bool,
    label: &'static str,
    on_click: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
}

impl CornerCard {
    pub fn new(
        id: impl Into<ElementId>,
        style: CornerRadiusStyle,
        label: &'static str,
        tokens: ThemeTokens,
        is_selected: bool,
    ) -> Self {
        Self {
            id: id.into(),
            style,
            label,
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

impl RenderOnce for CornerCard {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let is_selected = self.is_selected;
        let tokens = self.tokens;
        let radius = self.style.base_radius();

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
            // Preview box
            .child(
                div()
                    .w(px(110.0))
                    .h(px(54.0))
                    .rounded(px(8.0))
                    .bg(tokens.bg_sunken)
                    .border(if is_selected { px(2.0) } else { px(1.0) })
                    .border_color(if is_selected {
                        tokens.accent
                    } else {
                        tokens.border
                    })
                    .p(px(8.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        // Inner sample with the corner radius
                        div()
                            .w(px(70.0))
                            .h(px(32.0))
                            .rounded(radius)
                            .border(px(1.5))
                            .border_color(if is_selected {
                                tokens.accent
                            } else {
                                tokens.text_muted
                            })
                            .bg(tokens.bg_elevated),
                    ),
            )
            // Text Label
            .child(
                div()
                    .text_xs()
                    .font_medium()
                    .text_color(if is_selected {
                        tokens.accent
                    } else {
                        tokens.text_muted
                    })
                    .child(self.label),
            )
    }
}
