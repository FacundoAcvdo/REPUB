use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{Icon, IconName};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::theme::tokens::ThemeTokens;

#[derive(IntoElement)]
pub struct TopBar {
    title: &'static str,
    sidebar_open: bool,
    can_go_back: bool,
    can_go_forward: bool,
    tokens: ThemeTokens,
    radius: Pixels,
    on_toggle_sidebar: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
    on_go_back: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
    on_go_forward: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
}

impl TopBar {
    pub fn new(
        title: &'static str,
        sidebar_open: bool,
        can_go_back: bool,
        can_go_forward: bool,
        tokens: ThemeTokens,
        radius: Pixels,
    ) -> Self {
        Self {
            title,
            sidebar_open,
            can_go_back,
            can_go_forward,
            tokens,
            radius,
            on_toggle_sidebar: None,
            on_go_back: None,
            on_go_forward: None,
        }
    }

    pub fn on_toggle_sidebar(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_toggle_sidebar = Some(Box::new(handler));
        self
    }

    pub fn on_go_back(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_go_back = Some(Box::new(handler));
        self
    }

    pub fn on_go_forward(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_go_forward = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for TopBar {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let tokens = self.tokens;
        let radius = self.radius;
        let can_back = self.can_go_back;
        let can_fwd = self.can_go_forward;

        let sidebar_icon = if self.sidebar_open {
            IconName::PanelLeftClose
        } else {
            IconName::PanelLeftOpen
        };

        div()
            .w_full()
            .h(px(48.0))
            .px(px(16.0))
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .border_b_1()
            .border_color(tokens.border)
            .bg(rgba(0x00000014))
            // Left: Sidebar Toggle + Nav History (Back / Forward)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(6.0))
                    // Sidebar Toggle Button
                    .child(
                        div()
                            .id("btn_toggle_sidebar")
                            .w(px(32.0))
                            .h(px(32.0))
                            .rounded(radius)
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .hover(|s| s.bg(tokens.bg_hover))
                            .active(|s| s.bg(tokens.bg_active))
                            .when_some(self.on_toggle_sidebar, |this, handler| {
                                this.on_click(move |_, window, cx| handler(window, cx))
                            })
                            .child(
                                Icon::new(sidebar_icon)
                                    .size(px(18.0))
                                    .text_color(tokens.text),
                            ),
                    )
                    // Back Button
                    .child(
                        div()
                            .id("btn_nav_back")
                            .w(px(30.0))
                            .h(px(30.0))
                            .rounded(radius)
                            .flex()
                            .items_center()
                            .justify_center()
                            .when(can_back, |s| {
                                s.cursor_pointer()
                                    .hover(|s| s.bg(tokens.bg_hover))
                                    .active(|s| s.bg(tokens.bg_active))
                            })
                            .when(!can_back, |s| s.opacity(0.35).cursor_not_allowed())
                            .when(can_back, |this| {
                                this.when_some(self.on_go_back, |t, handler| {
                                    t.on_click(move |_, window, cx| handler(window, cx))
                                })
                            })
                            .child(
                                Icon::new(IconName::ChevronLeft)
                                    .size(px(18.0))
                                    .text_color(tokens.text),
                            ),
                    )
                    // Forward Button
                    .child(
                        div()
                            .id("btn_nav_fwd")
                            .w(px(30.0))
                            .h(px(30.0))
                            .rounded(radius)
                            .flex()
                            .items_center()
                            .justify_center()
                            .when(can_fwd, |s| {
                                s.cursor_pointer()
                                    .hover(|s| s.bg(tokens.bg_hover))
                                    .active(|s| s.bg(tokens.bg_active))
                            })
                            .when(!can_fwd, |s| s.opacity(0.35).cursor_not_allowed())
                            .when(can_fwd, |this| {
                                this.when_some(self.on_go_forward, |t, handler| {
                                    t.on_click(move |_, window, cx| handler(window, cx))
                                })
                            })
                            .child(
                                Icon::new(IconName::ChevronRight)
                                    .size(px(18.0))
                                    .text_color(tokens.text),
                            ),
                    ),
            )
            // Center: Location / Title
            .child(
                div()
                    .text_sm()
                    .font_semibold()
                    .text_color(tokens.text)
                    .child(self.title),
            )
            // Right: Balanced spacer so title stays centered
            .child(
                div()
                    .w(px(98.0))
                    .h(px(30.0)),
            )
    }
}
