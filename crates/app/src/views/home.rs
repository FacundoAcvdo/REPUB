use std::rc::Rc;
use epub_core::db::{BookRepository, Database};
use epub_core::domain::book::Book;
use epub_core::domain::stats::ReadingStats;
use epub_core::mock::books::mock_books;
use epub_core::services::StatsService;
use gpui_kit::assets::IconName;
use gpui_kit::base::StyledExt as _;
use gpui_kit::component::Icon;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::components::continue_reading::ContinueReadingCard;
use crate::components::mini_chart::WeeklyActivityChart;
use crate::components::stat_card::StatCard;
use crate::theme::state::ThemeState;

pub struct HomeView {
    stats: ReadingStats,
    current_book: Book,
    current_cover_path: Option<String>,
    notification: Option<String>,
    on_open_book: Option<Rc<dyn Fn(Book, &mut Window, &mut App) + 'static>>,
}

impl HomeView {
    pub fn new() -> Self {
        let (stats, current_book, current_cover_path) = Self::load_from_db();

        Self {
            stats,
            current_book,
            current_cover_path,
            notification: None,
            on_open_book: None,
        }
    }

    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        let (stats, current_book, current_cover_path) = Self::load_from_db();
        self.stats = stats;
        self.current_book = current_book;
        self.current_cover_path = current_cover_path;
        cx.notify();
    }

    fn load_from_db() -> (ReadingStats, Book, Option<String>) {
        let fallback_book = mock_books().into_iter().next().unwrap();
        let today = StatsService::today_iso();

        if let Some(db_path) = Database::default_db_path() {
            if db_path.exists() {
                if let Ok(db) = Database::open(&db_path) {
                    let stats = StatsService::compute_stats(db.conn(), &today).unwrap_or(ReadingStats {
                        books_completed: 0,
                        pages_read_total: 0,
                        pages_read_this_year: 0,
                        reading_hours: 0.0,
                        current_streak_days: 0,
                        weekly_activity: [0; 7],
                    });
                    let book = StatsService::get_continue_reading_book(db.conn())
                        .ok()
                        .flatten()
                        .unwrap_or(fallback_book);
                    let cover_path = BookRepository::get_by_id(db.conn(), &book.id)
                        .ok()
                        .flatten()
                        .and_then(|s| s.cover_path);
                    return (stats, book, cover_path);
                }
            }
        }

        (
            ReadingStats {
                books_completed: 0,
                pages_read_total: 0,
                pages_read_this_year: 0,
                reading_hours: 0.0,
                current_streak_days: 0,
                weekly_activity: [0; 7],
            },
            fallback_book,
            None,
        )
    }

    pub fn set_on_open_book(&mut self, handler: impl Fn(Book, &mut Window, &mut App) + 'static) {
        self.on_open_book = Some(Rc::new(handler));
    }
}

impl Render for HomeView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme_state = ThemeState::global(cx);
        let tokens = *theme_state.tokens();
        let radius = theme_state.radius();

        let view = cx.entity().clone();
        let notification = self.notification.clone();

        div()
            .id("home-scroll-container")
            .flex()
            .flex_col()
            .size_full()
            .overflow_y_scroll()
            .px(px(32.0))
            .py(px(24.0))
            .child(
                // Centered Container (Max ~860px for ergonomic dashboard width)
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .max_w(px(860.0))
                    .mx_auto()
                    .gap(px(28.0))
                    // Header Row
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_start()
                            .justify_between()
                            .w_full()
                            .gap(px(16.0))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(
                                        div()
                                            .text_2xl()
                                            .font_bold()
                                            .text_color(tokens.text)
                                            .child("Inicio"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(tokens.text_muted)
                                            .child("Bienvenido de nuevo. Tu biblioteca personal y métricas de lectura al día."),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(6.0))
                                    .px(px(12.0))
                                    .py(px(6.0))
                                    .rounded_full()
                                    .bg(tokens.bg_sunken)
                                    .border_1()
                                    .border_color(tokens.border)
                                    .child(
                                        Icon::new(IconName::Calendar)
                                            .size(px(13.0))
                                            .text_color(tokens.text_muted),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_medium()
                                            .text_color(tokens.text_muted)
                                            .child(StatsService::today_display_es()),
                                    ),
                            ),
                    )
                    // Notification Banner if user clicks resume reading
                    .when_some(notification, |this, msg| {
                        let view_close = view.clone();
                        this.child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .p(px(12.0))
                                .rounded(radius)
                                .bg(tokens.bg_elevated)
                                .border_1()
                                .border_color(tokens.accent)
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap(px(8.0))
                                        .child(
                                            Icon::new(IconName::BookOpen)
                                                .size(px(16.0))
                                                .text_color(tokens.accent),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_medium()
                                                .text_color(tokens.text)
                                                .child(msg),
                                        ),
                                )
                                .child(
                                    div()
                                        .id("btn-close-notification")
                                        .px(px(8.0))
                                        .py(px(4.0))
                                        .rounded(px(4.0))
                                        .cursor_pointer()
                                        .hover(|s| s.bg(tokens.bg_hover))
                                        .on_click(move |_, _, cx| {
                                            view_close.update(cx, |this, cx| {
                                                this.notification = None;
                                                cx.notify();
                                            });
                                        })
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(tokens.text_muted)
                                                .child("Cerrar"),
                                        ),
                                ),
                        )
                    })
                    // Section 1: Continuar Leyendo (Featured Horizontal Card)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(10.0))
                            .w_full()
                            .child({
                                let book_to_read = self.current_book.clone();
                                let on_open_book = self.on_open_book.clone();
                                ContinueReadingCard::new(
                                    "card-continue-reading",
                                    self.current_book.clone(),
                                    tokens,
                                    radius,
                                )
                                .cover_path(self.current_cover_path.clone())
                                .on_read(move |window, cx| {
                                    if let Some(ref handler) = on_open_book {
                                        handler(book_to_read.clone(), window, cx);
                                    }
                                })
                            }),
                    )
                    // Section 2: Estadísticas Generales (4-card Grid)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(12.0))
                            .w_full()
                            .child(
                                div()
                                    .text_xs()
                                    .font_bold()
                                    .text_color(tokens.text_muted)
                                    .child("MÉTRICAS CLAVE"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .flex_wrap()
                                    .gap(px(16.0))
                                    .w_full()
                                    .child(
                                        StatCard::new(
                                            "stat-books",
                                            "LIBROS LEÍDOS",
                                            format!("{}", self.stats.books_completed),
                                            IconName::BookOpen,
                                            tokens,
                                            radius,
                                        )
                                        .subtitle("En tu biblioteca personal"),
                                    )
                                    .child(
                                        StatCard::new(
                                            "stat-pages",
                                            "PÁGINAS LEÍDAS",
                                            format!("{:#}", self.stats.pages_read_total),
                                            IconName::FileText,
                                            tokens,
                                            radius,
                                        )
                                        .subtitle(format!("{} páginas este año", self.stats.pages_read_this_year).leak()),
                                    )
                                    .child(
                                        StatCard::new(
                                            "stat-hours",
                                            "TIEMPO DEDICADO",
                                            format!("{:.1} h", self.stats.reading_hours),
                                            IconName::Clock,
                                            tokens,
                                            radius,
                                        )
                                        .subtitle("~35 min de promedio diario"),
                                    )
                                    .child(
                                        StatCard::new(
                                            "stat-streak",
                                            "RACHA ACTUAL",
                                            format!("{} días", self.stats.current_streak_days),
                                            IconName::Flame,
                                            tokens,
                                            radius,
                                        )
                                        .accent_icon(true)
                                        .subtitle("¡Tu récord histórico!"),
                                    ),
                            ),
                    )
                    // Section 3: Gráfico de Actividad Semanal (MiniBarChart)
                    .child(
                        WeeklyActivityChart::new(
                            "chart-weekly-activity",
                            self.stats.weekly_activity,
                            tokens,
                            radius,
                        ),
                    ),
            )
    }
}
