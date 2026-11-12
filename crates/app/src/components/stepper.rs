use std::rc::Rc;
use gpui_kit::base::StyledExt as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::theme::tokens::ThemeTokens;

#[derive(IntoElement)]
pub struct Stepper {
    id: ElementId,
    value_text: String,
    tokens: ThemeTokens,
    on_decrement: Option<Rc<dyn Fn(&mut Window, &mut App) + 'static>>,
    on_increment: Option<Rc<dyn Fn(&mut Window, &mut App) + 'static>>,
}

impl Stepper {
    pub fn new(id: impl Into<ElementId>, value_text: impl Into<String>, tokens: ThemeTokens) -> Self {
        Self {
            id: id.into(),
            value_text: value_text.into(),
            tokens,
            on_decrement: None,
            on_increment: None,
        }
    }

    pub fn on_decrement(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_decrement = Some(Rc::new(handler));
        self
    }

    pub fn on_increment(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_increment = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Stepper {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let tokens = self.tokens;

        div()
            .id(self.id)
            .flex()
            .flex_row()
            .items_center()
            .rounded_full()
            .bg(tokens.bg_sunken)
            .border_1()
            .border_color(tokens.border)
            .p(px(2.0))
            .child(
                // Decrement button
                div()
                    .id("step-dec")
                    .w(px(28.0))
                    .h(px(28.0))
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .text_sm()
                    .font_bold()
                    .text_color(tokens.text)
                    .hover(|s| s.bg(tokens.bg_hover))
                    .active(|s| s.bg(tokens.bg_active))
                    .when_some(self.on_decrement, |this, handler| {
                        this.on_click(move |_, window, cx| handler(window, cx))
                    })
                    .child("-"),
            )
            .child(
                // Value text
                div()
                    .px(px(12.0))
                    .text_sm()
                    .font_medium()
                    .text_color(tokens.text)
                    .child(self.value_text),
            )
            .child(
                // Increment button
                div()
                    .id("step-inc")
                    .w(px(28.0))
                    .h(px(28.0))
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .text_sm()
                    .font_bold()
                    .text_color(tokens.text)
                    .hover(|s| s.bg(tokens.bg_hover))
                    .active(|s| s.bg(tokens.bg_active))
                    .when_some(self.on_increment, |this, handler| {
                        this.on_click(move |_, window, cx| handler(window, cx))
                    })
                    .child("+"),
            )
    }
}
