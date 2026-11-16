use std::collections::HashMap;
use std::rc::Rc;
use epub_core::db::{BookRepository, Database, ProgressRepository};
use epub_core::domain::book::{Book, BookFilter, BookSort};
use epub_core::services::{BookImportService, LibraryStorage};
use gpui_kit::assets::IconName;
use gpui_kit::base::StyledExt as _;
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::component::Icon;
use gpui_kit::*;

use crate::components::book_card::{BookCard, BookRow};
use crate::components::library_toolbar::LibraryToolbar;
use crate::theme::state::ThemeState;

#[derive(Clone, Default)]
struct BookUiMeta {
    cover_path: Option<String>,
    is_favorite: bool,
}

pub struct LibraryView {
    books: Vec<Book>,
    filtered_books: Vec<Book>,
    stored_meta: HashMap<String, BookUiMeta>,
    total_db_count: usize,
    search_input: Entity<InputState>,
    search_query: String,
    filter: BookFilter,
    sort: BookSort,
    is_grid_view: bool,
    notification: Option<String>,
    grid_scroll_handle: UniformListScrollHandle,
    list_scroll_handle: UniformListScrollHandle,
    on_open_book: Option<Rc<dyn Fn(Book, &mut Window, &mut App) + 'static>>,
    context_menu: Option<(Book, Point<Pixels>)>,
}

impl LibraryView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Buscar por título, autor o género...")
        });

        cx.subscribe(&search_input, |this, input, event: &InputEvent, cx| {
            if let InputEvent::Change = event {
                this.search_query = input.read(cx).value().to_string();
                this.refresh_catalog(cx);
            }
        })
        .detach();

        let mut this = Self {
            books: Vec::new(),
            filtered_books: Vec::new(),
            stored_meta: HashMap::new(),
            total_db_count: 0,
            search_input,
            search_query: String::new(),
            filter: BookFilter::All,
            sort: BookSort::TitleAsc,
            is_grid_view: true,
            notification: None,
            grid_scroll_handle: UniformListScrollHandle::new(),
            list_scroll_handle: UniformListScrollHandle::new(),
            on_open_book: None,
            context_menu: None,
        };
        this.load_from_sqlite();
        this
    }

    fn open_default_db() -> Option<Database> {
        Database::default_db_path().and_then(|path| Database::open(path).ok())
    }

    pub fn open_context_menu(&mut self, book: Book, pos: Point<Pixels>, cx: &mut Context<Self>) {
        self.context_menu = Some((book, pos));
        cx.notify();
    }

    pub fn close_context_menu(&mut self, cx: &mut Context<Self>) {
        if self.context_menu.is_some() {
            self.context_menu = None;
            cx.notify();
        }
    }

    pub fn delete_book(&mut self, book_id: &str, cx: &mut Context<Self>) {
        self.context_menu = None;
        if let Some(db) = Self::open_default_db() {
            match BookRepository::delete_with_files(db.conn(), book_id) {
                Ok(true) => {
                    self.notification = Some("Libro eliminado de la biblioteca.".to_string());
                    self.load_from_sqlite();
                }
                Ok(false) => {
                    self.notification = Some("No se encontró el libro a eliminar.".to_string());
                }
                Err(err) => {
                    self.notification = Some(format!("Error al eliminar libro: {err}"));
                }
            }
        } else {
            self.notification = Some("No se pudo abrir la base de datos local.".to_string());
        }
        cx.notify();
    }

    fn load_from_sqlite(&mut self) {
        let Some(db) = Self::open_default_db() else {
            return;
        };

        let total: usize = db
            .conn()
            .query_row("SELECT COUNT(*) FROM books", [], |row| row.get(0))
            .unwrap_or(0);
        self.total_db_count = total;

        if let Ok(stored_books) =
            BookRepository::list(db.conn(), self.filter, self.sort, &self.search_query)
        {
            self.stored_meta.clear();
            let mut books = Vec::with_capacity(stored_books.len());
            for item in stored_books {
                self.stored_meta.insert(
                    item.book.id.clone(),
                    BookUiMeta {
                        cover_path: item.cover_path,
                        is_favorite: item.is_favorite,
                    },
                );
                books.push(item.book);
            }
            self.filtered_books = books.clone();
            self.books = books;
        }
    }

    pub fn reload_from_db(&mut self, cx: &mut Context<Self>) {
        self.load_from_sqlite();
        cx.notify();
    }

    fn refresh_catalog(&mut self, cx: &mut Context<Self>) {
        self.load_from_sqlite();
        cx.notify();
    }

    pub fn set_on_open_book(&mut self, handler: impl Fn(Book, &mut Window, &mut App) + 'static) {
        self.on_open_book = Some(Rc::new(handler));
    }

    pub fn focus_search(&self, window: &mut Window, cx: &mut App) {
        self.search_input.update(cx, |input, cx| {
            input.focus(window, cx);
        });
    }

    pub fn set_filter(&mut self, filter: BookFilter, cx: &mut Context<Self>) {
        if self.filter != filter {
            self.filter = filter;
            self.refresh_catalog(cx);
        }
    }

    pub fn cycle_sort(&mut self, cx: &mut Context<Self>) {
        self.sort = match self.sort {
            BookSort::TitleAsc => BookSort::TitleDesc,
            BookSort::TitleDesc => BookSort::AuthorAsc,
            BookSort::AuthorAsc => BookSort::RecentProgress,
            BookSort::RecentProgress => BookSort::TotalPagesDesc,
            BookSort::TotalPagesDesc => BookSort::TitleAsc,
        };
        self.refresh_catalog(cx);
    }

    pub fn toggle_view_mode(&mut self, is_grid: bool, cx: &mut Context<Self>) {
        self.is_grid_view = is_grid;
        cx.notify();
    }

    pub fn toggle_favorite(&mut self, book_id: &str, cx: &mut Context<Self>) {
        if let Some(db) = Self::open_default_db() {
            if ProgressRepository::toggle_favorite(db.conn(), book_id).is_ok() {
                self.load_from_sqlite();
                cx.notify();
            }
        }
    }

    pub fn import_epub_dialog(&mut self, cx: &mut Context<Self>) {
        let paths_rx = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: true,
            prompt: Some("Seleccionar libros EPUB".into()),
        });

        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(paths))) = paths_rx.await else {
                return;
            };

            let epub_paths: Vec<std::path::PathBuf> = paths
                .into_iter()
                .filter(|p| {
                    p.extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext.eq_ignore_ascii_case("epub"))
                        .unwrap_or(false)
                })
                .collect();

            if epub_paths.is_empty() {
                let _ = this.update(cx, |view, cx| {
                    view.notification =
                        Some("Solo se admiten archivos con extensión .epub.".into());
                    cx.notify();
                });
                return;
            }

            let import_result = cx
                .background_executor()
                .spawn(async move {
                    let db_path = Database::default_db_path()
                        .ok_or_else(|| "No se pudo resolver la ruta de la base de datos".to_string())?;
                    let mut db = Database::open(db_path).map_err(|e| e.to_string())?;
                    let storage = LibraryStorage::default_storage()
                        .map_err(|e| e.to_string())?;

                    let mut imported = 0usize;
                    let mut last_title = String::new();
                    let mut errors = Vec::new();

                    for path in epub_paths {
                        match BookImportService::import_file(&mut db, &storage, &path) {
                            Ok(stored) => {
                                imported += 1;
                                last_title = stored.book.title;
                            }
                            Err(e) => errors.push(e.to_string()),
                        }
                    }

                    Ok::<_, String>((imported, last_title, errors))
                })
                .await;

            let _ = this.update(cx, |view, cx| {
                match import_result {
                    Ok((imported, last_title, errors)) => {
                        view.load_from_sqlite();
                        view.notification = if imported == 1 && errors.is_empty() {
                            Some(format!("Libro importado: \"{}\"", last_title))
                        } else if imported > 0 && errors.is_empty() {
                            Some(format!("{} libros importados correctamente.", imported))
                        } else if imported > 0 {
                            Some(format!(
                                "{} importados, {} con error ({}).",
                                imported,
                                errors.len(),
                                errors[0]
                            ))
                        } else {
                            Some(format!(
                                "No se pudo importar: {}",
                                errors.first().map(String::as_str).unwrap_or("error desconocido")
                            ))
                        };
                    }
                    Err(err) => {
                        view.notification = Some(format!("Error al importar: {}", err));
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn clear_notification(&mut self, cx: &mut Context<Self>) {
        self.notification = None;
        cx.notify();
    }

    pub fn clear_filters(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.filter = BookFilter::All;
        self.search_query.clear();
        self.search_input.update(cx, |input, cx| {
            input.set_value("", window, cx);
        });
        self.refresh_catalog(cx);
    }
}

impl Render for LibraryView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme_state = ThemeState::global(cx);
        let tokens = *theme_state.tokens();
        let radius = theme_state.radius();

        let total_count = self.total_db_count;
        let filtered_count = self.filtered_books.len();

        let view_entity = cx.entity().clone();

        let view_filter = view_entity.clone();
        let view_sort = view_entity.clone();
        let view_mode = view_entity.clone();
        let view_import = view_entity.clone();
        let view_clear_notif = view_entity.clone();

        let toolbar = LibraryToolbar::new(
            "library-toolbar",
            tokens,
            radius,
            self.search_input.clone(),
            self.filter,
            self.sort,
            self.is_grid_view,
            total_count,
            filtered_count,
        )
        .on_filter(move |f, _, cx| {
            view_filter.update(cx, |this, cx| this.set_filter(f, cx));
        })
        .on_sort(move |_, cx| {
            view_sort.update(cx, |this, cx| this.cycle_sort(cx));
        })
        .on_toggle_view(move |is_grid, _, cx| {
            view_mode.update(cx, |this, cx| this.toggle_view_mode(is_grid, cx));
        })
        .on_import(move |_, cx| {
            view_import.update(cx, |this, cx| this.import_epub_dialog(cx));
        });

        let notification_element = self.notification.as_ref().map(|msg| {
            let msg = msg.clone();
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .px(px(16.0))
                .py(px(10.0))
                .rounded(radius)
                .bg(tokens.bg_elevated)
                .border_1()
                .border_color(tokens.accent)
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(10.0))
                        .child(
                            Icon::new(IconName::Check)
                                .size(px(16.0))
                                .text_color(tokens.accent),
                        )
                        .child(
                            div()
                                .text_sm()
                                .font_medium()
                                .text_color(tokens.text)
                                .child(msg),
                        ),
                )
                .child(
                    div()
                        .id("btn-close-notification")
                        .p(px(4.0))
                        .rounded_full()
                        .hover(|s| s.bg(tokens.bg_hover))
                        .cursor_pointer()
                        .on_click(move |_, _, cx| {
                            view_clear_notif.update(cx, |this, cx| this.clear_notification(cx));
                        })
                        .child(
                            Icon::new(IconName::Close)
                                .size(px(14.0))
                                .text_color(tokens.text_muted),
                        ),
                )
        });

        // Content Area: Empty State vs Grid vs List
        let content_element = if filtered_count == 0 {
            let view_action = view_entity.clone();
            let is_library_empty = total_count == 0;
            div()
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(16.0))
                .py(px(64.0))
                .rounded(radius)
                .border_1()
                .border_color(tokens.border)
                .bg(tokens.bg_elevated)
                .child(
                    Icon::new(IconName::BookOpen)
                        .size(px(48.0))
                        .text_color(tokens.text_muted),
                )
                .child(
                    div()
                        .text_lg()
                        .font_semibold()
                        .text_color(tokens.text)
                        .child(if is_library_empty {
                            "Tu biblioteca está vacía"
                        } else {
                            "No se encontraron libros"
                        }),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(tokens.text_muted)
                        .child(if is_library_empty {
                            "Importa archivos .epub desde tu computadora para comenzar a leer."
                        } else {
                            "Prueba modificando el término de búsqueda o seleccionando otro filtro."
                        }),
                )
                .child(
                    div()
                        .id(if is_library_empty {
                            "btn-empty-import"
                        } else {
                            "btn-reset-filters"
                        })
                        .mt(px(8.0))
                        .px(px(16.0))
                        .py(px(8.0))
                        .rounded(radius)
                        .bg(if is_library_empty {
                            tokens.accent
                        } else {
                            tokens.bg_sunken
                        })
                        .border_1()
                        .border_color(if is_library_empty {
                            tokens.accent
                        } else {
                            tokens.border
                        })
                        .hover(|s| {
                            if is_library_empty {
                                s.opacity(0.9)
                            } else {
                                s.bg(tokens.bg_hover)
                            }
                        })
                        .cursor_pointer()
                        .on_click(move |_, window, cx| {
                            view_action.update(cx, |this, cx| {
                                if is_library_empty {
                                    this.import_epub_dialog(cx);
                                } else {
                                    this.clear_filters(window, cx);
                                }
                            });
                        })
                        .child(
                            div()
                                .text_sm()
                                .font_medium()
                                .text_color(if is_library_empty {
                                    tokens.accent_text
                                } else {
                                    tokens.accent
                                })
                                .child(if is_library_empty {
                                    "+ Importar EPUB"
                                } else {
                                    "Restablecer filtros"
                                }),
                        ),
                )
                .into_any_element()
        } else if self.is_grid_view {
            // Dynamic columns calculation:
            // Available width = Window width - (Sidebar approx 195px + padding 56px + scrollbar 20px)
            let viewport_w = window.viewport_size().width;
            let available_w = (viewport_w - px(270.0)).max(px(320.0));
            // Each card is 160px + 20px gap = 180px
            let items_per_row = ((available_w.as_f32() + 20.0) / 180.0).floor().max(2.0) as usize;

            let row_count = (filtered_count + items_per_row - 1) / items_per_row;
            let books_rc = Rc::new(self.filtered_books.clone());
            let meta_rc = Rc::new(self.stored_meta.clone());
            let on_open_book = self.on_open_book.clone();
            let view_fav = view_entity.clone();

            uniform_list(
                "library-virtual-grid",
                row_count,
                move |range, _window, _cx| {
                    let mut rows = Vec::with_capacity(range.len());
                    for row_idx in range {
                        let start = row_idx * items_per_row;
                        let end = (start + items_per_row).min(books_rc.len());
                        let row_books = &books_rc[start..end];

                        let on_open_book_row = on_open_book.clone();
                        let meta_row = meta_rc.clone();
                        let view_fav_row = view_fav.clone();
                        let row_div = div()
                            .id(ElementId::from(format!("book-row-{}", row_idx)))
                            .h(px(320.0))
                            .flex()
                            .flex_row()
                            .gap(px(20.0))
                            .pb(px(20.0))
                            .children(row_books.iter().map(|book| {
                                let book_for_click = book.clone();
                                let book_for_ctx = book.clone();
                                let cb = on_open_book_row.clone();
                                let meta = meta_row.get(&book.id).cloned().unwrap_or_default();
                                let view_fav_card = view_fav_row.clone();
                                let view_ctx_card = view_fav_row.clone();
                                BookCard::new(
                                    ElementId::from(book.id.clone()),
                                    book.clone(),
                                    tokens,
                                    radius,
                                )
                                .cover_path(meta.cover_path)
                                .is_favorite(meta.is_favorite)
                                .on_click(move |_, window, cx| {
                                    if let Some(ref handler) = cb {
                                        handler(book_for_click.clone(), window, cx);
                                    }
                                })
                                .on_toggle_favorite(move |id, _, cx| {
                                    view_fav_card.update(cx, |this, cx| {
                                        this.toggle_favorite(id, cx);
                                    });
                                })
                                .on_right_click(move |_, pos, _, cx| {
                                    view_ctx_card.update(cx, |this, cx| {
                                        this.open_context_menu(book_for_ctx.clone(), pos, cx);
                                    });
                                })
                            }));

                        rows.push(row_div);
                    }
                    rows
                },
            )
            .track_scroll(&self.grid_scroll_handle)
            .size_full()
            .flex_1()
            .into_any_element()
        } else {
            // Virtualized List mode: 1 book per row
            let books_rc = Rc::new(self.filtered_books.clone());
            let meta_rc = Rc::new(self.stored_meta.clone());
            let on_open_book = self.on_open_book.clone();
            let view_fav = view_entity.clone();

            uniform_list(
                "library-virtual-list",
                filtered_count,
                move |range, _window, _cx| {
                    let mut rows = Vec::with_capacity(range.len());
                    for idx in range {
                        if let Some(book) = books_rc.get(idx) {
                            let book_for_click = book.clone();
                            let book_for_ctx = book.clone();
                            let cb = on_open_book.clone();
                            let meta = meta_rc.get(&book.id).cloned().unwrap_or_default();
                            let view_fav_row = view_fav.clone();
                            let view_ctx_row = view_fav.clone();
                            let row_element = div()
                                .id(ElementId::from(format!("book-list-row-{}", idx)))
                                .h(px(76.0))
                                .pb(px(8.0))
                                .child(
                                    BookRow::new(
                                        ElementId::from(book.id.clone()),
                                        book.clone(),
                                        tokens,
                                        radius,
                                    )
                                    .cover_path(meta.cover_path)
                                    .is_favorite(meta.is_favorite)
                                    .on_click(move |_, window, cx| {
                                        if let Some(ref handler) = cb {
                                            handler(book_for_click.clone(), window, cx);
                                        }
                                    })
                                    .on_toggle_favorite(move |id, _, cx| {
                                        view_fav_row.update(cx, |this, cx| {
                                            this.toggle_favorite(id, cx);
                                        });
                                    })
                                    .on_right_click(move |_, pos, _, cx| {
                                        view_ctx_row.update(cx, |this, cx| {
                                            this.open_context_menu(book_for_ctx.clone(), pos, cx);
                                        });
                                    }),
                                );
                            rows.push(row_element);
                        }
                    }
                    rows
                },
            )
            .track_scroll(&self.list_scroll_handle)
            .size_full()
            .flex_1()
            .into_any_element()
        };

        let context_menu_overlay = self.context_menu.clone().map(|(book, pos)| {
            let solid_bg = gpui::Hsla { a: 1.0, ..tokens.bg };
            let open_cb = self.on_open_book.clone();
            let book_for_open = book.clone();
            let book_id_for_del = book.id.clone();

            div()
                .id("library-ctx-backdrop")
                .absolute()
                .inset_0()
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    this.close_context_menu(cx);
                }))
                .on_mouse_down(MouseButton::Right, cx.listener(|this, _, _, cx| {
                    this.close_context_menu(cx);
                }))
                .child(
                    anchored()
                        .position(pos)
                        .child(
                            div()
                                .id("library-ctx-menu")
                                .w(px(210.0))
                                .rounded(px(8.0))
                                .bg(solid_bg)
                                .shadow_lg()
                                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                    cx.stop_propagation();
                                })
                                .child(
                                    div()
                                        .p(px(6.0))
                                        .rounded(px(8.0))
                                        .border_1()
                                        .border_color(tokens.border)
                                        .bg(tokens.bg_elevated)
                                        .flex()
                                        .flex_col()
                                        .gap(px(2.0))
                                        .child(
                                            div()
                                                .px(px(8.0))
                                                .py(px(4.0))
                                                .text_xs()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(tokens.text_muted)
                                                .truncate()
                                                .child(book.title.clone()),
                                        )
                                        .child(div().h(px(1.0)).w_full().bg(tokens.border).my(px(2.0)))
                                        .child(
                                            div()
                                                .id("ctx-open-book")
                                                .px(px(8.0))
                                                .py(px(6.0))
                                                .rounded(px(5.0))
                                                .cursor_pointer()
                                                .hover(|s| s.bg(tokens.bg_hover))
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap(px(8.0))
                                                .text_sm()
                                                .text_color(tokens.text)
                                                .on_click(cx.listener(move |this, _, window, cx| {
                                                    this.close_context_menu(cx);
                                                    if let Some(ref handler) = open_cb {
                                                        handler(book_for_open.clone(), window, cx);
                                                    }
                                                }))
                                                .child(
                                                    Icon::new(IconName::BookOpen)
                                                        .size(px(14.0))
                                                        .text_color(tokens.text_muted),
                                                )
                                                .child("Abrir libro"),
                                        )
                                        .child(
                                            div()
                                                .id("ctx-delete-book")
                                                .px(px(8.0))
                                                .py(px(6.0))
                                                .rounded(px(5.0))
                                                .cursor_pointer()
                                                .hover(|s| s.bg(tokens.danger.opacity(0.12)))
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap(px(8.0))
                                                .text_sm()
                                                .text_color(tokens.danger)
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    this.delete_book(&book_id_for_del, cx);
                                                }))
                                                .child(
                                                    Icon::new(IconName::Trash)
                                                        .size(px(14.0))
                                                        .text_color(tokens.danger),
                                                )
                                                .child("Eliminar libro"),
                                        ),
                                ),
                        ),
                )
        });

        div()
            .relative()
            .flex()
            .flex_col()
            .size_full()
            .px(px(28.0))
            .pt(px(20.0))
            .pb(px(16.0))
            .gap(px(16.0))
            .child(toolbar)
            .children(notification_element)
            .child(content_element)
            .children(context_menu_overlay)
    }
}
