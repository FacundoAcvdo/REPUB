use gpui_kit::assets::IconName;
use gpui_kit::base::StyledExt as _;
use gpui_kit::component::Icon;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::theme::tokens::ThemeTokens;

#[derive(IntoElement)]
pub struct SettingRow {
    id: ElementId,
    title: &'static str,
    description: Option<&'static str>,
    tokens: ThemeTokens,
    control: AnyElement,
}

impl SettingRow {
    pub fn new(
        id: impl Into<ElementId>,
        title: &'static str,
        tokens: ThemeTokens,
        control: impl IntoElement,
    ) -> Self {
        Self {
            id: id.into(),
            title,
            description: None,
            tokens,
            control: control.into_any_element(),
        }
    }

    pub fn description(mut self, desc: &'static str) -> Self {
        self.description = Some(desc);
        self
    }
}

impl RenderOnce for SettingRow {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let tokens = self.tokens;

        div()
            .id(self.id)
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .w_full()
            .px(px(16.0))
            .py(px(12.0))
            .min_h(px(52.0))
            .gap(px(16.0))
            // Left: Title and optional description
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .text_sm()
                            .font_medium()
                            .text_color(tokens.text)
                            .whitespace_normal()
                            .child(self.title),
                    )
                    .when_some(self.description, |this, desc| {
                        this.child(
                            div()
                                .text_xs()
                                .text_color(tokens.text_muted)
                                .whitespace_normal()
                                .child(desc),
                        )
                    }),
            )
            // Right: Interactive control
            .child(
                div()
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_end()
                    .child(self.control),
            )
    }
}

#[derive(IntoElement)]
pub struct SettingsSection {
    id: ElementId,
    title: &'static str,
    icon: Option<IconName>,
    tokens: ThemeTokens,
    radius: Pixels,
    children: Vec<AnyElement>,
}

impl SettingsSection {
    pub fn new(
        id: impl Into<ElementId>,
        title: &'static str,
        tokens: ThemeTokens,
        radius: Pixels,
    ) -> Self {
        Self {
            id: id.into(),
            title,
            icon: None,
            tokens,
            radius,
            children: Vec::new(),
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children.extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }
}

impl RenderOnce for SettingsSection {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let tokens = self.tokens;
        let radius = self.radius;

        div()
            .id(self.id)
            .flex()
            .flex_col()
            .gap(px(8.0))
            .w_full()
            // Section Header: Small uppercase with icon
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(6.0))
                    .px(px(4.0))
                    .when_some(self.icon, |this, icon| {
                        this.child(
                            Icon::new(icon)
                                .size(px(13.0))
                                .text_color(tokens.text_muted),
                        )
                    })
                    .child(
                        div()
                            .text_size(px(11.0))
                            .font_semibold()
                            .text_color(tokens.text_muted)
                            .child(self.title.to_uppercase()),
                    ),
            )
            // Elevated container card with rows
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .rounded(radius)
                    .bg(tokens.bg_elevated)
                    .border_1()
                    .border_color(tokens.border)
                    .overflow_hidden()
                    .children(self.children.into_iter().enumerate().map(|(i, row)| {
                        if i > 0 {
                            div()
                                .flex()
                                .flex_col()
                                .w_full()
                                .border_t_1()
                                .border_color(tokens.border)
                                .child(row)
                        } else {
                            div()
                                .flex()
                                .flex_col()
                                .w_full()
                                .child(row)
                        }
                    })),
            )
    }
}
