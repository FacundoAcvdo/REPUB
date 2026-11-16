#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod components;
pub mod i18n;
pub mod theme;
pub mod views;
pub mod window_state;

use gpui_kit::*;

use crate::components::sidebar::NavDestination;
use crate::theme::init_theme;
use crate::theme::state::ThemeState;
use crate::views::ShellView;
use crate::window_state::SavedWindowState;

actions!(
    app,
    [
        ToggleSidebar,
        OpenGallery,
        GoToHome,
        GoToLibrary,
        GoToSettings,
        SearchBooks
    ]
);

pub struct AppRoot {
    shell: Entity<ShellView>,
    _bounds_subscription: Subscription,
}

impl AppRoot {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let shell = cx.new(|cx| ShellView::new(window, cx));
        let _bounds_subscription = cx.observe_window_bounds(window, move |_, window, _| {
            let bounds = window.window_bounds();
            SavedWindowState::from_window_bounds(bounds).save();
        });

        window.on_window_should_close(cx, |window, _| {
            let bounds = window.window_bounds();
            SavedWindowState::from_window_bounds(bounds).save();
            true
        });

        Self {
            shell,
            _bounds_subscription,
        }
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
            .on_action({
                let shell = shell.clone();
                move |_: &GoToHome, _, cx| {
                    shell.update(cx, |this, cx| {
                        this.navigate_to(NavDestination::Home, cx);
                    });
                }
            })
            .on_action({
                let shell = shell.clone();
                move |_: &GoToLibrary, _, cx| {
                    shell.update(cx, |this, cx| {
                        this.navigate_to(NavDestination::Library, cx);
                    });
                }
            })
            .on_action({
                let shell = shell.clone();
                move |_: &GoToSettings, _, cx| {
                    shell.update(cx, |this, cx| {
                        this.navigate_to(NavDestination::Settings, cx);
                    });
                }
            })
            .on_action({
                let shell = shell.clone();
                move |_: &SearchBooks, window, cx| {
                    shell.update(cx, |this, cx| {
                        this.focus_search(window, cx);
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
                KeyBinding::new("ctrl-1", GoToHome, None),
                KeyBinding::new("ctrl-2", GoToLibrary, None),
                KeyBinding::new("ctrl-3", GoToSettings, None),
                KeyBinding::new("ctrl-,", GoToSettings, None),
                KeyBinding::new("ctrl-f", SearchBooks, None),
            ]);

            let blur = ThemeState::global(cx).interface_blur;
            let initial_appearance = if blur {
                WindowBackgroundAppearance::Blurred
            } else {
                WindowBackgroundAppearance::Transparent
            };

            let initial_bounds = if let Some(saved) = SavedWindowState::load() {
                saved.to_window_bounds()
            } else {
                WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1280.0), px(820.0)),
                    cx,
                ))
            };

            gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(initial_bounds),
                    titlebar: Some(TitlebarOptions {
                        title: Some("REPUB".into()),
                        ..Default::default()
                    }),
                    window_background: initial_appearance,
                    app_id: Some("repub".to_string()),
                    ..Default::default()
                },
                cx,
                |window, cx| {
                    window.set_background_appearance(initial_appearance);
                    cx.new(|cx| AppRoot::new(window, cx))
                },
            )
            .expect("falló al abrir la ventana de REPUB");
        });
}
