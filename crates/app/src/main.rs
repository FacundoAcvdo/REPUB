pub mod components;
pub mod i18n;
pub mod theme;
pub mod views;

use gpui_kit::*;

use crate::components::sidebar::NavDestination;
use crate::theme::init_theme;
use crate::theme::state::ThemeState;
use crate::views::ShellView;

actions!(app, [ToggleSidebar, OpenGallery]);

pub struct AppRoot {
    shell: Entity<ShellView>,
}

impl AppRoot {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let shell = cx.new(|cx| ShellView::new(window, cx));
        Self { shell }
    }
}

impl Render for AppRoot {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme_state = ThemeState::global(cx);
        let shell = self.shell.clone();

        div()
            .flex()
            .size_full()
            .bg(theme_state.window_bg())
            .on_action({
                let shell = shell.clone();
                move |_: &ToggleSidebar, _, cx| {
                    shell.update(cx, |this, cx| {
                        this.toggle_sidebar(cx);
                    });
                }
            })
            .on_action({
                let shell = shell.clone();
                move |_: &OpenGallery, _, cx| {
                    shell.update(cx, |this, cx| {
                        this.navigate_to(NavDestination::Gallery, cx);
                    });
                }
            })
            .child(shell)
    }
}

fn main() {
    gpui_kit::application()
        .with_assets(gpui_kit::assets::AllAssets)
        .run(|cx| {
            gpui_kit::init(cx);
            init_theme(cx);

            cx.bind_keys([
                KeyBinding::new("ctrl-shift-g", OpenGallery, None),
                KeyBinding::new("ctrl-b", ToggleSidebar, None),
            ]);

            gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                        None,
                        size(px(1280.0), px(820.0)),
                        cx,
                    ))),
                    titlebar: Some(TitlebarOptions {
                        title: Some("EPUB Reader".into()),
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
