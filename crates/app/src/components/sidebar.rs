use std::rc::Rc;
use gpui_kit::base::StyledExt as _;
use gpui_kit::component::{Icon, IconName};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::theme::tokens::ThemeTokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavDestination {
    Home,
    Library,
    Settings,
    Gallery,
    Reader,
}

#[derive(IntoElement)]
pub struct SidebarItem {
    id: ElementId,
    destination: NavDestination,
    icon: IconName,
    label: &'static str,
    is_active: bool,
    tokens: ThemeTokens,
    radius: Pixels,
    on_click: Option<Rc<dyn Fn(NavDestination, &mut Window, &mut App) + 'static>>,
}

impl SidebarItem {
    pub fn new(
        destination: NavDestination,
        icon: IconName,
        label: &'static str,
        is_active: bool,
        tokens: ThemeTokens,
        radius: Pixels,
    ) -> Self {
        let id_str = match destination {
            NavDestination::Home => "nav_home",
            NavDestination::Library => "nav_library",
            NavDestination::Settings => "nav_settings",
            NavDestination::Gallery => "nav_gallery",
            NavDestination::Reader => "nav_reader",
        };
        Self {
            id: ElementId::from(id_str),
            destination,
            icon,
            label,
            is_active,
            tokens,
            radius,
            on_click: None,
        }
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(NavDestination, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for SidebarItem {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let is_active = self.is_active;
        let tokens = self.tokens;
        let dest = self.destination;

        div()
            .id(self.id)
            .w_full()
            .h(px(38.0))
            .px(px(12.0))
            .rounded(self.radius)
            .flex()
            .flex_row()
            .items_center()
            .gap(px(10.0))
            .cursor_pointer()
            .when(is_active, |s| {
                s.bg(tokens.bg_elevated)
                    .text_color(tokens.text)
                    .font_semibold()
                    .border_1()
                    .border_color(tokens.border)
            })
            .when(!is_active, |s| {
                s.text_color(tokens.text_muted)
                    .font_medium()
                    .hover(|s| s.bg(tokens.bg_hover).text_color(tokens.text))
            })
            .when_some(self.on_click, |this, handler| {
                this.on_click(move |_, window, cx| handler(dest, window, cx))
            })
            .child(
                Icon::new(self.icon)
                    .size(px(18.0))
                    .text_color(if is_active {
                        tokens.accent
                    } else {
                        tokens.text_muted
                    }),
            )
            .child(
                div()
                    .text_sm()
                    .truncate()
                    .child(self.label),
            )
    }
}

#[derive(IntoElement)]
pub struct Sidebar {
    is_open: bool,
    active_dest: NavDestination,
    tokens: ThemeTokens,
    radius: Pixels,
    on_navigate: Option<Rc<dyn Fn(NavDestination, &mut Window, &mut App) + 'static>>,
}

impl Sidebar {
    pub fn new(
        is_open: bool,
        active_dest: NavDestination,
        tokens: ThemeTokens,
        radius: Pixels,
    ) -> Self {
        Self {
            is_open,
            active_dest,
            tokens,
            radius,
            on_navigate: None,
        }
    }

    pub fn on_navigate(
        mut self,
        handler: impl Fn(NavDestination, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_navigate = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Sidebar {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let is_open = self.is_open;
        let tokens = self.tokens;
        let active = self.active_dest;
        let radius = self.radius;
        let on_navigate = self.on_navigate;

        let make_item = |dest, icon, label| {
            let mut item = SidebarItem::new(dest, icon, label, active == dest, tokens, radius);
            if let Some(ref handler) = on_navigate {
                let handler = handler.clone();
                item.on_click = Some(handler);
            }
            item
        };

        div()
            .h_full()
            .flex()
            .flex_col()
            .when(is_open, |s| s.w(px(195.0)).p(px(10.0)))
            .when(!is_open, |s| s.w(px(0.0)).overflow_hidden().p(px(0.0)))
            .bg(rgba(0x0000001f))
            .border_r_1()
            .border_color(tokens.border)
            // Navigation Items
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(make_item(NavDestination::Home, IconName::LayoutDashboard, "Inicio"))
                    .child(make_item(NavDestination::Library, IconName::BookOpen, "Biblioteca"))
                    .child(make_item(NavDestination::Settings, IconName::Settings, "Ajustes")),
            )
    }
}
