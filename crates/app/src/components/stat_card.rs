use gpui_kit::assets::IconName;
use gpui_kit::base::StyledExt as _;
use gpui_kit::component::Icon;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::theme::tokens::ThemeTokens;

#[derive(IntoElement)]
pub struct StatCard {
    id: ElementId,
    title: &'static str,
    value: String,
    subtitle: Option<&'static str>,
    icon: IconName,
    accent_icon: bool,
    tokens: ThemeTokens,
    radius: Pixels,
}

impl StatCard {
    pub fn new(
        id: impl Into<ElementId>,
        title: &'static str,
        value: impl Into<String>,
        icon: IconName,
        tokens: ThemeTokens,
        radius: Pixels,
    ) -> Self {
        Self {
            id: id.into(),
            title,
            value: value.into(),
            subtitle: None,
            icon,
            accent_icon: false,
            tokens,
            radius,
        }
    }

    pub fn subtitle(mut self, sub: &'static str) -> Self {
        self.subtitle = Some(sub);
        self
    }

    pub fn accent_icon(mut self, accent: bool) -> Self {
        self.accent_icon = accent;
        self
    }
}

impl RenderOnce for StatCard {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let tokens = self.tokens;
        let radius = self.radius;

        div()
            .id(self.id)
            .flex()
            .flex_col()
            .justify_between()
            .flex_1()
            .min_w(px(140.0))
            .p(px(16.0))
            .rounded(radius)
            .bg(tokens.bg_elevated)
            .border_1()
            .border_color(tokens.border)
            .hover(|s| s.bg(tokens.bg_hover).border_color(tokens.accent))
            // Top Row: Title + Icon Badge
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .child(
                        div()
                            .text_xs()
                            .font_semibold()
                            .text_color(tokens.text_muted)
                            .child(self.title),
                    )
                    .child(
                        div()
                            .p(px(6.0))
                            .rounded_full()
                            .bg(if self.accent_icon {
                                tokens.accent
                            } else {
                                tokens.bg_sunken
                            })
                            .child(
                                Icon::new(self.icon)
                                    .size(px(15.0))
                                    .text_color(if self.accent_icon {
                                        tokens.accent_text
                                    } else {
                                        tokens.text_muted
                                    }),
                            ),
                    ),
            )
            // Bottom Section: Big Value + Subtitle
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .mt(px(12.0))
                    .child(
                        div()
                            .text_2xl()
                            .font_bold()
                            .text_color(tokens.text)
                            .child(self.value),
                    )
                    .when_some(self.subtitle, |this, sub| {
                        this.child(
                            div()
                                .text_xs()
                                .text_color(tokens.text_muted)
                                .child(sub),
                        )
                    }),
            )
    }
}
