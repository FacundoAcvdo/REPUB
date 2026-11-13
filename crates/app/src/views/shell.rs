use epub_core::domain::book::Book;
use gpui_kit::*;

use crate::components::sidebar::{NavDestination, Sidebar};
use crate::components::topbar::TopBar;
use crate::theme::state::ThemeState;
use crate::views::gallery::GalleryView;
use crate::views::home::HomeView;
use crate::views::library::LibraryView;
use crate::views::reader::ReaderView;
use crate::views::settings::SettingsView;

pub struct ShellView {
    is_sidebar_open: bool,
    current_dest: NavDestination,
    back_history: Vec<NavDestination>,
    forward_history: Vec<NavDestination>,
    home_view: Entity<HomeView>,
    library_view: Entity<LibraryView>,
    settings_view: Entity<SettingsView>,
    gallery_view: Entity<GalleryView>,
    reader_view: Entity<ReaderView>,
}

impl ShellView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let home_view = cx.new(|_| HomeView::new());
        let library_view = cx.new(|cx| LibraryView::new(window, cx));
        let settings_view = cx.new(|cx| SettingsView::new(window, cx));
        let gallery_view = cx.new(|_| GalleryView::new());
        let reader_view = cx.new(|cx| ReaderView::new(cx));

        let entity = cx.entity().clone();

        // Conectar evento de abrir libro desde la Biblioteca
        let entity_lib = entity.clone();
        library_view.update(cx, |this, _| {
            this.set_on_open_book(move |book, window, cx| {
                entity_lib.update(cx, |this, cx| {
                    this.open_book(book, window, cx);
                });
            });
        });

        // Conectar evento de reanudar libro desde Inicio
        let entity_home = entity.clone();
        home_view.update(cx, |this, _| {
            this.set_on_open_book(move |book, window, cx| {
                entity_home.update(cx, |this, cx| {
                    this.open_book(book, window, cx);
                });
            });
        });

        // Conectar evento de salir/cerrar del lector
        let entity_reader = entity.clone();
        reader_view.update(cx, |this, _| {
            this.set_on_close(move |_, cx| {
                entity_reader.update(cx, |this, cx| {
                    this.go_back(cx);
                });
            });
        });

        Self {
            is_sidebar_open: true,
            current_dest: NavDestination::Home,
            back_history: Vec::new(),
            forward_history: Vec::new(),
            home_view,
            library_view,
            settings_view,
            gallery_view,
            reader_view,
        }
    }

    pub fn open_book(&mut self, book: Book, window: &mut Window, cx: &mut Context<Self>) {
        self.reader_view.update(cx, |reader, cx| {
            reader.load_book(book, cx);
            reader.focus(window, cx);
        });
        self.navigate_to(NavDestination::Reader, cx);
    }

    pub fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.is_sidebar_open = !self.is_sidebar_open;
        cx.notify();
    }

    pub fn navigate_to(&mut self, dest: NavDestination, cx: &mut Context<Self>) {
        if self.current_dest == dest {
            return;
        }
        self.back_history.push(self.current_dest);
        self.forward_history.clear();
        self.current_dest = dest;
        cx.notify();
    }

    pub fn go_back(&mut self, cx: &mut Context<Self>) {
        if let Some(prev) = self.back_history.pop() {
            self.forward_history.push(self.current_dest);
            self.current_dest = prev;
            cx.notify();
        } else {
            // Si el historial está vacío (ej. vino directamente al lector), vuelve a Biblioteca
            if self.current_dest == NavDestination::Reader {
                self.current_dest = NavDestination::Library;
                cx.notify();
            }
        }
    }

    pub fn go_forward(&mut self, cx: &mut Context<Self>) {
        if let Some(next) = self.forward_history.pop() {
            self.back_history.push(self.current_dest);
            self.current_dest = next;
            cx.notify();
        }
    }

    pub fn focus_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.navigate_to(NavDestination::Library, cx);
        self.library_view.update(cx, |lib, cx| {
            lib.focus_search(window, cx);
        });
    }

    pub fn title_for_dest(&self) -> &'static str {
        match self.current_dest {
            NavDestination::Home => "Inicio",
            NavDestination::Library => "Biblioteca",
            NavDestination::Settings => "Ajustes",
            NavDestination::Gallery => "Galería de Componentes y Diseño",
            NavDestination::Reader => "Lector",
        }
    }
}

impl Render for ShellView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme_state = ThemeState::global(cx);
        let tokens = theme_state.tokens();
        let radius = theme_state.radius();

        // En modo Lector Inmersivo, la hoja ocupa el 100% del área de la ventana
        if self.current_dest == NavDestination::Reader {
            return div()
                .flex()
                .size_full()
                .text_color(tokens.text)
                .child(self.reader_view.clone())
                .into_any_element();
        }

        let entity = cx.entity().clone();
        let entity_nav = entity.clone();
        let entity_toggle = entity.clone();
        let entity_back = entity.clone();
        let entity_fwd = entity.clone();

        let sidebar = Sidebar::new(
            self.is_sidebar_open,
            self.current_dest,
            *tokens,
            radius,
        )
        .on_navigate(move |dest, _, cx| {
            entity_nav.update(cx, |this, cx| this.navigate_to(dest, cx));
        });

        let topbar = TopBar::new(
            self.title_for_dest(),
            self.is_sidebar_open,
            !self.back_history.is_empty(),
            !self.forward_history.is_empty(),
            *tokens,
            radius,
        )
        .on_toggle_sidebar(move |_, cx| {
            entity_toggle.update(cx, |this, cx| this.toggle_sidebar(cx));
        })
        .on_go_back(move |_, cx| {
            entity_back.update(cx, |this, cx| this.go_back(cx));
        })
        .on_go_forward(move |_, cx| {
            entity_fwd.update(cx, |this, cx| this.go_forward(cx));
        });

        let active_content = match self.current_dest {
            NavDestination::Home => self.home_view.clone().into_any_element(),
            NavDestination::Library => self.library_view.clone().into_any_element(),
            NavDestination::Settings => self.settings_view.clone().into_any_element(),
            NavDestination::Gallery => self.gallery_view.clone().into_any_element(),
            NavDestination::Reader => self.reader_view.clone().into_any_element(),
        };

        div()
            .flex()
            .flex_row()
            .size_full()
            .text_color(tokens.text)
            .child(sidebar)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .h_full()
                    .overflow_hidden()
                    .child(topbar)
                    .child(
                        div()
                            .flex_1()
                            .w_full()
                            .overflow_hidden()
                            .child(active_content),
                    ),
            )
            .into_any_element()
    }
}
