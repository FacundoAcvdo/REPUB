use std::rc::Rc;
use gpui_kit::base::StyledExt as _;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::theme::tokens::ThemeTokens;

pub struct SegmentedItem {
    pub id: &'static str,
    pub label: &'static str,
}

#[derive(IntoElement)]
pub struct SegmentedControl {
    id: ElementId,
    items: Vec<SegmentedItem>,
    selected_id: &'static str,
    tokens: ThemeTokens,
    on_select: Option<Rc<dyn Fn(&'static str, &mut Window, &mut App) + 'static>>,
}

impl SegmentedControl {
    pub fn new(
        id: impl Into<ElementId>,
        items: Vec<SegmentedItem>,
        selected_id: &'static str,
        tokens: ThemeTokens,
    ) -> Self {
        Self {
            id: id.into(),
            items,
            selected_id,
            tokens,
            on_select: None,
        }
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(&'static str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for SegmentedControl {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let tokens = self.tokens;
        let selected_id = self.selected_id;
        let on_select = self.on_select;

        div()
            .id(self.id)
            .flex()
            .flex_row()
            .items_center()
            .p(px(3.0))
            .gap(px(2.0))
            .rounded_full()
            .bg(tokens.bg_sunken)
            .border_1()
            .border_color(tokens.border)
            .children(self.items.into_iter().map(move |item| {
                let is_active = item.id == selected_id;
                let item_id = item.id;
                let on_select_clone = on_select.clone();

                div()
                    .id(ElementId::from(item.id))
                    .px(px(14.0))
                    .py(px(5.0))
                    .rounded_full()
                    .cursor_pointer()
                    .text_xs()
                    .font_medium()
                    .when(is_active, |s| {
                        s.bg(tokens.bg_elevated)
                            .text_color(tokens.text)
                            .border_1()
                            .border_color(tokens.border)
                    })
                    .when(!is_active, |s| {
                        s.text_color(tokens.text_muted)
                            .hover(|s| s.text_color(tokens.text).bg(tokens.bg_hover))
                    })
                    .when_some(on_select_clone, |this, handler| {
                        this.on_click(move |_, window, cx| handler(item_id, window, cx))
                    })
                    .child(item.label)
            }))
    }
}
