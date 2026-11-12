pub mod components;
pub mod i18n;
pub mod theme;
pub mod views;

use gpui_kit::base::StyledExt as _;
use gpui_kit::*;

use crate::theme::init_theme;
use crate::theme::state::ThemeState;
use crate::views::GalleryView;

actions!(app, [ToggleGallery]);

pub enum ActiveView {
    Gallery(Entity<GalleryView>),
    Welcome,
}

pub struct AppRoot {
    active_view: ActiveView,
}

impl AppRoot {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let gallery = cx.new(|_| GalleryView::new());
        Self {
            active_view: ActiveView::Gallery(gallery),
        }
    }

    pub fn toggle_gallery(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        match &self.active_view {
            ActiveView::Gallery(_) => {
                self.active_view = ActiveView::Welcome;
            }
            ActiveView::Welcome => {
                let gallery = cx.new(|_| GalleryView::new());
                self.active_view = ActiveView::Gallery(gallery);
            }
        }
        cx.notify();
    }
}

impl Render for AppRoot {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme_state = ThemeState::global(cx);
        let tokens = theme_state.tokens();

        div()
            .flex()
            .size_full()
            .bg(theme_state.window_bg())
            .text_color(tokens.text)
            .on_action(cx.listener(|this, _: &ToggleGallery, window, cx| {
                this.toggle_gallery(window, cx);
            }))
            .child(match &self.active_view {
                ActiveView::Gallery(gallery) => gallery.clone().into_any_element(),
                ActiveView::Welcome => div()
                    .flex()
                    .flex_col()
                    .size_full()
                    .items_center()
                    .justify_center()
                    .gap(px(16.0))
                    .child(
                        div()
                            .text_2xl()
                            .font_bold()
                            .child("EPUB Reader Desktop - Hito 1"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(tokens.text_muted)
                            .child("Presiona Ctrl + Shift + G para volver a la Galería de Diseño"),
                    )
                    .into_any_element(),
            })
    }
}

fn main() {
    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(|cx| {
            gpui_kit::init(cx);
            init_theme(cx);

            cx.bind_keys([KeyBinding::new("ctrl-shift-g", ToggleGallery, None)]);

            gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                        None,
                        size(px(1280.0), px(820.0)),
                        cx,
                    ))),
                    titlebar: Some(TitlebarOptions {
                        title: Some("EPUB Reader - Galería de Diseño".into()),
                        ..Default::default()
                    }),
                    window_background: WindowBackgroundAppearance::Transparent,
                    ..Default::default()
                },
                cx,
                |window, cx| cx.new(|cx| AppRoot::new(window, cx)),
            )
            .expect("falló al abrir la ventana de EPUB Reader");
        });
}
