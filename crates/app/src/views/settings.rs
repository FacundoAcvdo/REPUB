use epub_core::db::{BookRepository, Database, ProgressRepository, SettingsRepository};
use epub_core::domain::settings::{
    AnimationSpeed, AppSettings, CornerRadiusStyle, EmbeddingSource, LocalEmbeddingModelInfo,
    ReduceMotion,
};
use epub_core::services::{AiClient, LocalEmbeddings, RagService};
use gpui_kit::assets::IconName;
use gpui_kit::base::StyledExt as _;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::Icon;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::InteractiveElement as _;
use gpui_kit::*;

use crate::components::corner_picker::CornerCard;
use crate::components::segmented::{SegmentedControl, SegmentedItem};
use crate::components::settings_section::{SettingRow, SettingsSection};
use crate::components::stepper::Stepper;
use crate::components::theme_card::ThemeCard;
use crate::theme::palettes::all_themes;
use crate::theme::reader_settings::ReaderSettingsState;
use crate::theme::state::ThemeState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    General,
    Appearance,
    Providers,
    Sheet,
}

impl SettingsTab {
    pub fn name_es(&self) -> &'static str {
        match self {
            SettingsTab::General => "General",
            SettingsTab::Appearance => "Apariencia",
            SettingsTab::Providers => "Proveedores (IA)",
            SettingsTab::Sheet => "Hoja",
        }
    }

    pub fn icon(&self) -> IconName {
        match self {
            SettingsTab::General => IconName::SlidersHorizontal,
            SettingsTab::Appearance => IconName::Palette,
            SettingsTab::Providers => IconName::Sparkles,
            SettingsTab::Sheet => IconName::BookOpen,
        }
    }
}

pub struct SettingsView {
    active_tab: SettingsTab,
    settings: AppSettings,
    test_connection_msg: Option<String>,
    embedding_status_msg: Option<String>,
    is_downloading_model: bool,
    pending_model_switch: Option<(EmbeddingSource, Option<String>)>,
    is_font_select_open: bool,
    is_delete_data_modal_open: bool,
    del_opt_books: bool,
    del_opt_stats: bool,
    delete_data_status: Option<String>,
    model_input: Entity<InputState>,
    api_key_input: Entity<InputState>,
}

impl SettingsView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut settings = Database::default_db_path()
            .and_then(|path| Database::open(path).ok())
            .and_then(|db| SettingsRepository::load_settings(db.conn()).ok())
            .unwrap_or_else(AppSettings::default);
        settings.reader = ReaderSettingsState::global(cx).settings.clone();

        let initial_model = settings.ai.model.clone();
        let initial_api_key = settings.ai.api_key.clone();

        let model_input = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(initial_model)
                .placeholder("Nombre del modelo (ej. gpt-4o, claude-3-7-sonnet)...")
        });

        cx.subscribe(&model_input, |this, input, event: &InputEvent, cx| {
            if let InputEvent::Change = event {
                this.settings.ai.model = input.read(cx).value().to_string();
                this.persist();
                cx.notify();
            }
        })
        .detach();

        let api_key_input = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(initial_api_key)
                .placeholder("Ingresa tu clave de API...")
        });

        cx.subscribe(&api_key_input, |this, input, event: &InputEvent, cx| {
            if let InputEvent::Change = event {
                this.settings.ai.api_key = input.read(cx).value().to_string();
                this.persist();
                cx.notify();
            }
        })
        .detach();

        Self {
            active_tab: SettingsTab::General,
            settings,
            test_connection_msg: None,
            embedding_status_msg: None,
            is_downloading_model: false,
            pending_model_switch: None,
            is_font_select_open: false,
            is_delete_data_modal_open: false,
            del_opt_books: false,
            del_opt_stats: false,
            delete_data_status: None,
            model_input,
            api_key_input,
        }
    }

    fn persist(&self) {
        if let Some(db_path) = Database::default_db_path() {
            if let Ok(db) = Database::open(db_path) {
                let _ = SettingsRepository::save_settings(db.conn(), &self.settings);
            }
        }
    }

    pub fn execute_selected_data_deletion(&mut self, cx: &mut Context<Self>) {
        if !self.del_opt_books && !self.del_opt_stats {
            return;
        }

        let Some(db_path) = Database::default_db_path() else {
            self.delete_data_status = Some("✗ No se encontró la base de datos local.".into());
            self.is_delete_data_modal_open = false;
            cx.notify();
            return;
        };

        let Ok(db) = Database::open(db_path) else {
            self.delete_data_status = Some("✗ No se pudo abrir la base de datos local.".into());
            self.is_delete_data_modal_open = false;
            cx.notify();
            return;
        };

        let mut deleted_parts = Vec::new();
        let mut errors = Vec::new();

        if self.del_opt_books {
            match BookRepository::delete_all_with_files(db.conn()) {
                Ok(count) => deleted_parts.push(format!("Libros ({count})")),
                Err(err) => errors.push(format!("Libros: {err}")),
            }
        }

        if self.del_opt_stats {
            match ProgressRepository::clear_all_stats(db.conn()) {
                Ok(()) => deleted_parts.push("Estadísticas".to_string()),
                Err(err) => errors.push(format!("Estadísticas: {err}")),
            }
        }

        self.is_delete_data_modal_open = false;
        self.del_opt_books = false;
        self.del_opt_stats = false;

        self.delete_data_status = if errors.is_empty() {
            Some(format!("✓ Eliminado correctamente: {}.", deleted_parts.join(", ")))
        } else {
            Some(format!("✗ Errores al eliminar: {}", errors.join("; ")))
        };
        cx.notify();
    }

    fn run_test_connection(&mut self, cx: &mut Context<Self>) {
        let ai_settings = self.settings.ai.clone();
        self.test_connection_msg = Some("Probando conexión con el proveedor...".into());
        cx.notify();

        cx.spawn(async move |this, cx| {
            let res = cx
                .background_executor()
                .spawn(async move { AiClient::test_connection(&ai_settings) })
                .await;

            let _ = this.update(cx, |view, cx| {
                view.test_connection_msg = Some(match res {
                    Ok(ok) => format!("✓ {ok}"),
                    Err(err) => format!("✗ {err}"),
                });
                cx.notify();
            });
        })
        .detach();
    }

    fn current_index_info() -> (Option<epub_core::services::IndexSummary>, u64) {
        let summary = Database::default_db_path()
            .and_then(|p| Database::open(p).ok())
            .and_then(|db| RagService::current_index_summary(db.conn()).ok())
            .flatten();
        let disk_bytes = LocalEmbeddings::downloaded_bytes();
        (summary, disk_bytes)
    }

    fn request_embedding_config(
        &mut self,
        source: EmbeddingSource,
        local_model: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let (summary, _) = Self::current_index_info();
        if let Some(idx) = summary {
            let new_model_label = match source {
                EmbeddingSource::Local => local_model.clone().unwrap_or_default(),
                EmbeddingSource::Cloud => format!("{}-embeddings", self.settings.ai.provider),
                EmbeddingSource::Unconfigured => String::new(),
            };
            if idx.embedding_model != new_model_label && idx.indexed_books > 0 {
                self.pending_model_switch = Some((source, local_model));
                self.embedding_status_msg = Some(format!(
                    "Ya hay {} libro(s) indexados con '{}'. Cambiar de modelo requiere borrar o reindexar la base actual.",
                    idx.indexed_books, idx.embedding_model
                ));
                cx.notify();
                return;
            }
        }

        self.apply_embedding_config(source, local_model, false, cx);
    }

    fn apply_embedding_config(
        &mut self,
        source: EmbeddingSource,
        local_model: Option<String>,
        clear_existing_index: bool,
        cx: &mut Context<Self>,
    ) {
        self.pending_model_switch = None;
        if clear_existing_index {
            if let Some(db_path) = Database::default_db_path() {
                if let Ok(mut db) = Database::open(db_path) {
                    let _ = RagService::clear_index_only(db.conn_mut());
                }
            }
        }

        self.settings.ai.embedding_source = source;
        self.settings.ai.local_embedding_model = local_model.clone();
        self.persist();

        if source == EmbeddingSource::Local {
            if let Some(model_id) = local_model {
                self.is_downloading_model = true;
                self.embedding_status_msg = Some(format!(
                    "Descargando modelo '{}' en segundo plano... La primera vez puede demorar unos minutos.",
                    model_id
                ));
                cx.notify();

                cx.spawn(async move |this, cx| {
                    let res = cx
                        .background_executor()
                        .spawn(async move { LocalEmbeddings::ensure_downloaded(&model_id) })
                        .await;

                    let _ = this.update(cx, |view, cx| {
                        view.is_downloading_model = false;
                        view.embedding_status_msg = Some(match res {
                            Ok(_) => "✓ Modelo local listo en disco para indexación RAG.".into(),
                            Err(e) => format!("✗ Error al descargar el modelo: {e}"),
                        });
                        cx.notify();
                    });
                })
                .detach();
                return;
            }
        } else if source == EmbeddingSource::Cloud {
            self.embedding_status_msg =
                Some("✓ Embeddings configurados vía API Cloud del proveedor.".into());
        } else {
            self.embedding_status_msg = None;
        }
        cx.notify();
    }

    fn clear_all_models_and_embeddings(&mut self, cx: &mut Context<Self>) {
        self.pending_model_switch = None;
        if let Some(db_path) = Database::default_db_path() {
            if let Ok(mut db) = Database::open(db_path) {
                match RagService::clear_all_embeddings_and_models(db.conn_mut()) {
                    Ok(freed) => {
                        let mb = freed as f64 / (1024.0 * 1024.0);
                        self.settings.ai.embedding_source = EmbeddingSource::Unconfigured;
                        self.settings.ai.local_embedding_model = None;
                        self.persist();
                        self.embedding_status_msg = Some(format!(
                            "✓ Modelos locales y embeddings borrados ({:.1} MB liberados).",
                            mb
                        ));
                    }
                    Err(e) => {
                        self.embedding_status_msg = Some(format!("✗ Error al limpiar: {e}"));
                    }
                }
            }
        }
        cx.notify();
    }

    pub fn set_tab(&mut self, tab: SettingsTab, cx: &mut Context<Self>) {
        self.active_tab = tab;
        self.is_font_select_open = false;
        cx.notify();
    }
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme_state = ThemeState::global(cx);
        let tokens = *theme_state.tokens();
        let radius = theme_state.radius();

        let view_entity = cx.entity().clone();

        // Top Navigation Tabs
        let tabs = [
            SettingsTab::General,
            SettingsTab::Appearance,
            SettingsTab::Providers,
            SettingsTab::Sheet,
        ];

        let active_tab = self.active_tab;

        let nav_bar = div()
            .flex()
            .flex_row()
            .items_center()
            .p(px(3.0))
            .gap(px(4.0))
            .rounded_full()
            .bg(tokens.bg_sunken)
            .border_1()
            .border_color(tokens.border)
            .children(tabs.iter().map(move |tab| {
                let t = *tab;
                let is_active = t == active_tab;
                let view = view_entity.clone();

                div()
                    .id(ElementId::from(format!("tab-{:?}", t)))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(6.0))
                    .px(px(16.0))
                    .py(px(6.0))
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
                    .on_click(move |_, _, cx| {
                        view.update(cx, |this, cx| this.set_tab(t, cx));
                    })
                    .child(
                        Icon::new(t.icon())
                            .size(px(14.0))
                            .text_color(if is_active { tokens.accent } else { tokens.text_muted }),
                    )
                    .child(t.name_es())
            }));

        // Render tab contents
        let tab_content = match active_tab {
            SettingsTab::General => self.render_general_tab(tokens, radius, cx),
            SettingsTab::Appearance => self.render_appearance_tab(tokens, radius, cx),
            SettingsTab::Providers => self.render_providers_tab(tokens, radius, cx),
            SettingsTab::Sheet => self.render_sheet_tab(tokens, radius, cx),
        };

        let modal_overlay = if self.is_delete_data_modal_open {
            let solid_bg = gpui::Hsla { a: 1.0, ..tokens.bg };
            let has_selection = self.del_opt_books || self.del_opt_stats;

            let make_option = |id: &'static str,
                               label: &'static str,
                               desc: &'static str,
                               checked: bool,
                               toggle: fn(&mut SettingsView)| {
                div()
                    .id(id)
                    .p(px(12.0))
                    .rounded(radius)
                    .border_1()
                    .border_color(if checked { tokens.accent } else { tokens.border })
                    .bg(if checked {
                        tokens.accent.opacity(0.08)
                    } else {
                        tokens.bg_elevated
                    })
                    .cursor_pointer()
                    .hover(|s| s.bg(tokens.bg_hover))
                    .flex()
                    .flex_row()
                    .items_start()
                    .gap(px(12.0))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        toggle(this);
                        cx.notify();
                    }))
                    .child(
                        div()
                            .mt(px(2.0))
                            .w(px(18.0))
                            .h(px(18.0))
                            .flex_shrink_0()
                            .rounded(px(4.0))
                            .border_1()
                            .border_color(if checked { tokens.accent } else { tokens.border })
                            .bg(if checked {
                                tokens.accent
                            } else {
                                gpui::transparent_black()
                            })
                            .flex()
                            .items_center()
                            .justify_center()
                            .when(checked, |el| {
                                el.child(
                                    Icon::new(IconName::Check)
                                        .size(px(12.0))
                                        .text_color(tokens.bg),
                                )
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .flex_1()
                            .min_w(px(0.0))
                            .child(
                                div()
                                    .w_full()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(tokens.text)
                                    .child(label),
                            )
                            .child(
                                div()
                                    .w_full()
                                    .text_xs()
                                    .text_color(tokens.text_muted)
                                    .child(desc),
                            ),
                    )
            };

            Some(
                div()
                    .id("delete-data-modal-backdrop")
                    .absolute()
                    .inset_0()
                    .bg(gpui::black().opacity(0.55))
                    .flex()
                    .items_center()
                    .justify_center()
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                        this.is_delete_data_modal_open = false;
                        cx.notify();
                    }))
                    .child(
                        div()
                            .id("delete-data-modal-card")
                            .w(px(460.0))
                            .rounded(px(12.0))
                            .bg(solid_bg)
                            .shadow_lg()
                            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                cx.stop_propagation();
                            })
                            .child(
                                div()
                                    .p(px(22.0))
                                    .rounded(px(12.0))
                                    .border_1()
                                    .border_color(tokens.border)
                                    .bg(tokens.bg_elevated)
                                    .flex()
                                    .flex_col()
                                    .gap(px(16.0))
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap(px(10.0))
                                            .child(
                                                Icon::new(IconName::Trash)
                                                    .size(px(18.0))
                                                    .text_color(tokens.danger),
                                            )
                                            .child(
                                                div()
                                                    .text_base()
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(tokens.text)
                                                    .child("Eliminar datos locales"),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(tokens.text_muted)
                                            .child("Selecciona una o más categorías para borrar permanentemente:"),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap(px(8.0))
                                            .child(make_option(
                                                "del-opt-books",
                                                "Libros",
                                                "Elimina todos los libros importados, portadas, anotaciones y archivos EPUB guardados.",
                                                self.del_opt_books,
                                                |this| this.del_opt_books = !this.del_opt_books,
                                            ))
                                            .child(make_option(
                                                "del-opt-stats",
                                                "Estadísticas",
                                                "Reinicia el progreso de lectura, páginas leídas y sesiones de lectura registradas.",
                                                self.del_opt_stats,
                                                |this| this.del_opt_stats = !this.del_opt_stats,
                                            )),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .justify_end()
                                            .gap(px(10.0))
                                            .pt(px(4.0))
                                            .child(
                                                div()
                                                    .id("del-modal-cancel")
                                                    .px(px(14.0))
                                                    .py(px(8.0))
                                                    .rounded(radius)
                                                    .border_1()
                                                    .border_color(tokens.border)
                                                    .bg(tokens.bg_elevated)
                                                    .hover(|s| s.bg(tokens.bg_hover))
                                                    .cursor_pointer()
                                                    .text_sm()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(tokens.text)
                                                    .on_click(cx.listener(|this, _, _, cx| {
                                                        this.is_delete_data_modal_open = false;
                                                        cx.notify();
                                                    }))
                                                    .child("Cancelar"),
                                            )
                                            .child(
                                                div()
                                                    .id("del-modal-confirm")
                                                    .px(px(14.0))
                                                    .py(px(8.0))
                                                    .rounded(radius)
                                                    .bg(if has_selection {
                                                        tokens.danger
                                                    } else {
                                                        tokens.border
                                                    })
                                                    .when(has_selection, |el| {
                                                        el.cursor_pointer()
                                                            .hover(|s| s.opacity(0.9))
                                                            .on_click(cx.listener(|this, _, _, cx| {
                                                                this.execute_selected_data_deletion(cx);
                                                            }))
                                                    })
                                                    .text_sm()
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .text_color(if has_selection {
                                                        tokens.bg
                                                    } else {
                                                        tokens.text_muted
                                                    })
                                                    .child("Eliminar seleccionados"),
                                            ),
                                    ),
                            ),
                    ),
            )
        } else {
            None
        };

        div()
            .relative()
            .size_full()
            .child(
                div()
                    .id("settings-scroll-container")
                    .flex()
                    .flex_col()
                    .size_full()
                    .overflow_y_scroll()
                    .px(px(32.0))
                    .py(px(24.0))
                    .child(
                        // Centered Column constrained to Sonora's ~640-680px width
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .w_full()
                            .max_w(px(660.0))
                            .mx_auto()
                            .gap(px(24.0))
                            // Header
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .items_center()
                                    .gap(px(6.0))
                                    .child(
                                        div()
                                            .text_2xl()
                                            .font_bold()
                                            .text_color(tokens.text)
                                            .child("Ajustes"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(tokens.text_muted)
                                            .child("Personaliza tu experiencia de lectura, interfaz y modelos."),
                                    ),
                            )
                            // Nav Bar
                            .child(nav_bar)
                            // Active Tab Section
                            .child(tab_content),
                    ),
            )
            .children(modal_overlay)
    }
}

impl SettingsView {
    fn render_general_tab(
        &mut self,
        tokens: crate::theme::tokens::ThemeTokens,
        radius: Pixels,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let view = cx.entity().clone();

        let general = &self.settings.general;
        let reduce_motion = general.reduce_motion;
        let anim_speed = general.animation_speed;

        div()
            .flex()
            .flex_col()
            .w_full()
            .gap(px(20.0))
            // Section 1: Interfaz General
            .child(
                SettingsSection::new("sec-general", "Interfaz", tokens, radius)
                    .icon(IconName::SlidersHorizontal)
                    .child(
                        SettingRow::new(
                            "row-lang",
                            "Idioma de la aplicación",
                            tokens,
                            div()
                                .px(px(12.0))
                                .py(px(4.0))
                                .rounded_full()
                                .bg(tokens.bg_sunken)
                                .border_1()
                                .border_color(tokens.border)
                                .text_xs()
                                .font_medium()
                                .text_color(tokens.text)
                                .child("Español (Latinoamérica)"),
                        )
                        .description("Idioma principal para menús, avisos y navegación."),
                    )
                    .child(
                        SettingRow::new(
                            "row-home",
                            "Pantalla de inicio predeterminada",
                            tokens,
                            {
                                let view_home = view.clone();
                                let current_home = self.settings.general.home_view.clone();
                                SegmentedControl::new(
                                    "seg-home-view",
                                    vec![
                                        SegmentedItem { id: "home", label: "Inicio" },
                                        SegmentedItem { id: "library", label: "Biblioteca" },
                                    ],
                                    if current_home == "Biblioteca" { "library" } else { "home" },
                                    tokens,
                                )
                                .on_select(move |id, _, cx| {
                                    view_home.update(cx, |this, cx| {
                                        this.settings.general.home_view = if id == "library" {
                                            "Biblioteca".into()
                                        } else {
                                            "Inicio".into()
                                        };
                                        this.persist();
                                        cx.notify();
                                    });
                                })
                            },
                        )
                        .description("Vista que se muestra automáticamente al abrir el lector."),
                    ),
            )
            // Section 2: Movimiento y Animaciones
            .child(
                SettingsSection::new("sec-motion", "Movimiento y Accesibilidad", tokens, radius)
                    .icon(IconName::Bot)
                    .child(
                        SettingRow::new(
                            "row-reduce-motion",
                            "Reducir movimiento",
                            tokens,
                            {
                                let view_motion = view.clone();
                                SegmentedControl::new(
                                    "seg-reduce-motion",
                                    vec![
                                        SegmentedItem { id: "system", label: "Sistema" },
                                        SegmentedItem { id: "always", label: "Siempre" },
                                        SegmentedItem { id: "never", label: "Nunca" },
                                    ],
                                    match reduce_motion {
                                        ReduceMotion::System => "system",
                                        ReduceMotion::Always => "always",
                                        ReduceMotion::Never => "never",
                                    },
                                    tokens,
                                )
                                .on_select(move |id, _, cx| {
                                    view_motion.update(cx, |this, cx| {
                                        this.settings.general.reduce_motion = match id {
                                            "always" => ReduceMotion::Always,
                                            "never" => ReduceMotion::Never,
                                            _ => ReduceMotion::System,
                                        };
                                        this.persist();
                                        cx.notify();
                                    });
                                })
                            },
                        )
                        .description("Desactiva animaciones de repliegue y transiciones para evitar mareos."),
                    )
                    .child(
                        SettingRow::new(
                            "row-anim-speed",
                            "Velocidad de animaciones",
                            tokens,
                            {
                                let view_speed = view.clone();
                                SegmentedControl::new(
                                    "seg-anim-speed",
                                    vec![
                                        SegmentedItem { id: "slow", label: "Lenta" },
                                        SegmentedItem { id: "standard", label: "Estándar" },
                                        SegmentedItem { id: "fast", label: "Rápida" },
                                    ],
                                    match anim_speed {
                                        AnimationSpeed::Slow => "slow",
                                        AnimationSpeed::Standard => "standard",
                                        AnimationSpeed::Fast => "fast",
                                    },
                                    tokens,
                                )
                                .on_select(move |id, _, cx| {
                                    view_speed.update(cx, |this, cx| {
                                        this.settings.general.animation_speed = match id {
                                            "slow" => AnimationSpeed::Slow,
                                            "fast" => AnimationSpeed::Fast,
                                            _ => AnimationSpeed::Standard,
                                        };
                                        this.persist();
                                        cx.notify();
                                    });
                                })
                            },
                        )
                        .description("Preferencia de duración para transiciones e interfaces (Lenta: 250ms, Estándar: 150ms, Rápida: 80ms)."),
                    ),
            )
            .child(
                SettingsSection::new("local-data-section", "Gestión de datos locales", tokens, radius)
                    .child(
                        SettingRow::new(
                            "delete-data-row",
                            "Eliminar datos",
                            tokens,
                            div()
                                .id("open-delete-data-modal")
                                .px(px(14.0))
                                .py(px(7.0))
                                .rounded(radius)
                                .border_1()
                                .border_color(tokens.danger.opacity(0.6))
                                .bg(tokens.danger.opacity(0.1))
                                .hover(|s| s.bg(tokens.danger.opacity(0.18)))
                                .cursor_pointer()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(6.0))
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(tokens.danger)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.is_delete_data_modal_open = true;
                                    cx.notify();
                                }))
                                .child(
                                    Icon::new(IconName::Trash)
                                        .size(px(14.0))
                                        .text_color(tokens.danger),
                                )
                                .child("Eliminar datos..."),
                        )
                        .description("Abre un selector múltiple para elegir qué categorías borrar (libros o estadísticas)."),
                    )
                    .when_some(self.delete_data_status.clone(), |el, msg| {
                        el.child(
                            div()
                                .px(px(12.0))
                                .py(px(8.0))
                                .rounded(radius)
                                .bg(tokens.bg_elevated)
                                .border_1()
                                .border_color(tokens.border)
                                .text_xs()
                                .text_color(tokens.text_muted)
                                .child(msg),
                        )
                    }),
            )
            .into_any_element()
    }

    fn render_appearance_tab(
        &mut self,
        tokens: crate::theme::tokens::ThemeTokens,
        radius: Pixels,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme_state = ThemeState::global(cx);
        let current_theme_id = theme_state.current_theme.id;
        let current_radius = theme_state.corner_radius;
        let current_opacity = theme_state.window_opacity;

        let all_palettes = all_themes();
        let view_for_themes = cx.entity().clone();
        let view_for_corners = cx.entity().clone();
        let view_for_opacity = cx.entity().clone();

        div()
            .flex()
            .flex_col()
            .w_full()
            .gap(px(20.0))
            // Section 1: Themes Grid
            .child(
                SettingsSection::new("sec-theme", "Tema de la aplicación", tokens, radius)
                    .icon(IconName::Palette)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .p(px(16.0))
                            .gap(px(12.0))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(tokens.text_muted)
                                    .child("Selecciona una paleta de color armónica inspirada en editores modernos:"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .flex_wrap()
                                    .gap(px(14.0))
                                    .justify_center()
                                    .children(all_palettes.into_iter().map(move |palette| {
                                        let p_id = palette.id;
                                        let is_selected = p_id == current_theme_id;
                                        let view_theme = view_for_themes.clone();

                                        ThemeCard::new(
                                            ElementId::from(format!("theme-card-{}", p_id)),
                                            palette,
                                            is_selected,
                                        )
                                        .on_click(move |window, cx| {
                                            ThemeState::update_theme(Some(window), cx, p_id);
                                            view_theme.update(cx, |this, cx| {
                                                this.settings.appearance.theme_id = p_id.into();
                                                this.persist();
                                                cx.notify();
                                            });
                                        })
                                    })),
                            ),
                    ),
            )
            // Section 2: Corner Radius Style
            .child(
                SettingsSection::new("sec-corners", "Esquinas de la interfaz", tokens, radius)
                    .icon(IconName::Layers)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .p(px(16.0))
                            .gap(px(12.0))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(tokens.text_muted)
                                    .child("Elige el radio de curvatura para botones, tarjetas y modales:"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .flex_wrap()
                                    .gap(px(12.0))
                                    .justify_center()
                                    .children(CornerRadiusStyle::all().iter().map(move |style| {
                                        let s = *style;
                                        let is_selected = s == current_radius;
                                        let view_corner = view_for_corners.clone();

                                        CornerCard::new(
                                            ElementId::from(format!("corner-card-{:?}", s)),
                                            s,
                                            s.name_es(),
                                            tokens,
                                            is_selected,
                                        )
                                        .on_click(move |_, cx| {
                                            ThemeState::update_corner_radius(cx, s);
                                            view_corner.update(cx, |this, cx| {
                                                this.settings.appearance.corner_radius = s;
                                                this.persist();
                                                cx.notify();
                                            });
                                        })
                                    })),
                            ),
                    ),
            )
            // Section 3: Window Opacity & Blur with Dedicated Full-Width Slider (Point 1 fix!)
            .child(
                SettingsSection::new("sec-window", "Ventana y Translúcidez", tokens, radius)
                    .icon(IconName::Sparkles)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .p(px(16.0))
                            .gap(px(12.0))
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap(px(2.0))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_medium()
                                                    .text_color(tokens.text)
                                                    .child("Opacidad de la ventana"),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(tokens.text_muted)
                                                    .child("Permite ver el fondo de pantalla de tu escritorio detrás del visor."),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .px(px(10.0))
                                            .py(px(4.0))
                                            .rounded_full()
                                            .bg(tokens.bg_sunken)
                                            .border_1()
                                            .border_color(tokens.border)
                                            .text_xs()
                                            .font_bold()
                                            .text_color(tokens.accent)
                                            .child(format!("{:.0}%", current_opacity * 100.0)),
                                    ),
                            )
                            // Full-width Slider Track Bar
                            .child(
                                div()
                                    .relative()
                                    .w_full()
                                    .h(px(8.0))
                                    .rounded_full()
                                    .bg(tokens.bg_sunken)
                                    .overflow_hidden()
                                    .child(
                                        div()
                                            .h_full()
                                            .w(relative(current_opacity))
                                            .rounded_full()
                                            .bg(tokens.accent),
                                    ),
                            )
                            // Preset Buttons below Slider
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .pt(px(4.0))
                                    .children([1.0, 0.8, 0.6, 0.4, 0.2].into_iter().map(move |val| {
                                        let is_current = (current_opacity - val).abs() < 0.05;
                                        let view_op = view_for_opacity.clone();
                                        div()
                                            .id(ElementId::from(format!("op-preset-{:.0}", val * 100.0)))
                                            .px(px(12.0))
                                            .py(px(4.0))
                                            .rounded_full()
                                            .cursor_pointer()
                                            .text_xs()
                                            .font_medium()
                                            .when(is_current, |s| {
                                                s.bg(tokens.accent)
                                                    .text_color(tokens.accent_text)
                                            })
                                            .when(!is_current, |s| {
                                                s.bg(tokens.bg_sunken)
                                                    .text_color(tokens.text_muted)
                                                    .border_1()
                                                    .border_color(tokens.border)
                                                    .hover(|s| s.text_color(tokens.text).bg(tokens.bg_hover))
                                            })
                                            .on_click(move |_, window, cx| {
                                                ThemeState::update_window_opacity(Some(window), cx, val);
                                                view_op.update(cx, |this, cx| {
                                                    this.settings.appearance.window_opacity = val;
                                                    this.persist();
                                                    cx.notify();
                                                });
                                            })
                                            .child(format!("{:.0}%", val * 100.0))
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_start()
                            .gap(px(10.0))
                            .p(px(12.0))
                            .rounded(radius)
                            .bg(tokens.bg_sunken)
                            .border_1()
                            .border_color(tokens.border_subtle)
                            .child(
                                div()
                                    .flex_shrink_0()
                                    .pt(px(2.0))
                                    .child(
                                        Icon::new(IconName::Sparkles)
                                            .size(px(15.0))
                                            .text_color(tokens.accent),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .flex_1()
                                    .min_w_0()
                                    .gap(px(2.0))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_semibold()
                                            .text_color(tokens.text)
                                            .child("Desenfoque en Linux (KDE Plasma)"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(tokens.text_muted)
                                            .whitespace_normal()
                                            .child("Si estás ejecutando en Linux con BetterBlurDx, puedes forzar el efecto de desenfoque desde la configuración del plugin para que se aplique directamente sobre la ventana translúcida."),
                                    ),
                            ),
                    ),
            )
            // Section 4: Typography scale
            .child(
                SettingsSection::new("sec-ui-typo", "Tipografía de la interfaz", tokens, radius)
                    .icon(IconName::BookOpen)
                    .child(
                        SettingRow::new(
                            "row-font-family",
                            "Familia tipográfica de la app",
                            tokens,
                            div()
                                .px(px(12.0))
                                .py(px(4.0))
                                .rounded_full()
                                .bg(tokens.bg_sunken)
                                .border_1()
                                .border_color(tokens.border)
                                .text_xs()
                                .font_medium()
                                .text_color(tokens.text)
                                .child("Inter / Sans del sistema"),
                        )
                        .description("Tipografía utilizada para botones, títulos y menús."),
                    )
                    .child(
                        SettingRow::new(
                            "row-font-size",
                            "Tamaño base de texto",
                            tokens,
                            Stepper::new("step-font-size", format!("{} px", self.settings.appearance.font_size), tokens)
                                .on_decrement(|_, _| {})
                                .on_increment(|_, _| {}),
                        )
                        .description("Escala tipográfica predeterminada (14 px)."),
                    ),
            )
            .into_any_element()
    }

    fn render_providers_tab(
        &mut self,
        tokens: crate::theme::tokens::ThemeTokens,
        radius: Pixels,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let view = cx.entity().clone();
        let current_provider = self.settings.ai.provider.clone();
        let test_msg = self.test_connection_msg.clone();
        let current_emb_source = self.settings.ai.embedding_source;
        let current_local_model = self.settings.ai.local_embedding_model.clone();
        let emb_status = self.embedding_status_msg.clone();
        let is_downloading = self.is_downloading_model;
        let pending_switch = self.pending_model_switch.clone();
        let (index_summary, disk_bytes) = Self::current_index_info();

        let providers = [
            ("OpenAI", "openai"),
            ("Anthropic", "anthropic"),
            ("Gemini", "gemini"),
            ("Groq", "groq"),
        ];

        div()
            .flex()
            .flex_col()
            .w_full()
            .gap(px(20.0))
            // Section 1: Provider selection & configuration
            .child(
                SettingsSection::new("sec-ai-provider", "Proveedor de Inteligencia Artificial", tokens, radius)
                    .icon(IconName::Cpu)
                    // Row 1: Separate Provider Pills
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .p(px(16.0))
                            .gap(px(12.0))
                            .border_b_1()
                            .border_color(tokens.border)
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(2.0))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_medium()
                                            .text_color(tokens.text)
                                            .child("Proveedor activo"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(tokens.text_muted)
                                            .child("Elige el motor de inferencia para resúmenes y preguntas lectoras:"),
                                    ),
                            )
                            // Individual Pills Row
                            .child({
                                let view_prov = view.clone();
                                div()
                                    .flex()
                                    .flex_row()
                                    .flex_wrap()
                                    .items_center()
                                    .gap(px(10.0))
                                    .children(providers.into_iter().map(move |(name, id)| {
                                        let is_selected = current_provider == name;
                                        let view_pill = view_prov.clone();
                                        let prov_name = name.to_string();

                                        div()
                                            .id(ElementId::from(format!("pill-prov-{}", id)))
                                            .px(px(18.0))
                                            .py(px(8.0))
                                            .rounded_full()
                                            .cursor_pointer()
                                            .text_xs()
                                            .font_medium()
                                            .when(is_selected, |s| {
                                                s.bg(tokens.accent)
                                                    .text_color(tokens.accent_text)
                                                    .font_semibold()
                                            })
                                            .when(!is_selected, |s| {
                                                s.bg(tokens.bg_sunken)
                                                    .text_color(tokens.text_muted)
                                                    .border_1()
                                                    .border_color(tokens.border)
                                                    .hover(|s| s.bg(tokens.bg_hover).text_color(tokens.text))
                                            })
                                            .on_click(move |_, _, cx| {
                                                let p = prov_name.clone();
                                                view_pill.update(cx, |this, cx| {
                                                    this.settings.ai.provider = p;
                                                    this.test_connection_msg = None;
                                                    this.persist();
                                                    cx.notify();
                                                });
                                            })
                                            .child(name)
                                    }))
                            }),
                    )
                    // Row 2: Model as an editable input
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .p(px(16.0))
                            .gap(px(10.0))
                            .border_b_1()
                            .border_color(tokens.border)
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(2.0))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_medium()
                                            .text_color(tokens.text)
                                            .child("Modelo de inferencia"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(tokens.text_muted)
                                            .child("Escribe el nombre del modelo deseado (ej. gpt-4o, claude-3-7-sonnet, gemini-2.0-flash, llama-3.3-70b):"),
                                    ),
                            )
                            .child(
                                div()
                                    .w_full()
                                    .child(Input::new(&self.model_input)),
                            ),
                    )
                    // Row 3: API Key with input and button passed down
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .p(px(16.0))
                            .gap(px(10.0))
                            .border_b_1()
                            .border_color(tokens.border)
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(2.0))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_medium()
                                            .text_color(tokens.text)
                                            .child("Clave de API"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(tokens.text_muted)
                                            .child("Almacenada con cifrado localmente en tu sistema:"),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(10.0))
                                    .w_full()
                                    .child(
                                        div()
                                            .flex_1()
                                            .child(Input::new(&self.api_key_input)),
                                    )
                                    .child(
                                        div()
                                            .id("btn-modify-key")
                                            .px(px(16.0))
                                            .py(px(8.0))
                                            .rounded(radius)
                                            .bg(tokens.bg_elevated)
                                            .border_1()
                                            .border_color(tokens.border)
                                            .hover(|s| s.bg(tokens.bg_hover))
                                            .cursor_pointer()
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_medium()
                                                    .text_color(tokens.text)
                                                    .child("Guardar"),
                                            ),
                                    ),
                            ),
                    )
                    // Row 4: Base URL endpoint
                    .child(
                        SettingRow::new(
                            "row-base-url",
                            "URL base del servicio",
                            tokens,
                            div()
                                .px(px(12.0))
                                .py(px(4.0))
                                .rounded_full()
                                .bg(tokens.bg_sunken)
                                .border_1()
                                .border_color(tokens.border)
                                .text_xs()
                                .text_color(tokens.text_muted)
                                .child(match self.settings.ai.provider.as_str() {
                                    "Anthropic" => "https://api.anthropic.com/v1",
                                    "Gemini" => "https://generativelanguage.googleapis.com",
                                    "Groq" => "https://api.groq.com/openai/v1",
                                    _ => "https://api.openai.com/v1",
                                }),
                        )
                        .description("Endpoint oficial del proveedor o proxy local."),
                    )
                    // Row 5: Test Connection
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .p(px(16.0))
                            .gap(px(10.0))
                            .child(
                                div()
                                    .id("btn-test-connection")
                                    .px(px(16.0))
                                    .py(px(8.0))
                                    .rounded(radius)
                                    .bg(tokens.accent)
                                    .hover(|s| s.bg(tokens.accent_hover))
                                    .cursor_pointer()
                                    .on_click({
                                        let view_test = view.clone();
                                        move |_, _, cx| {
                                            view_test.update(cx, |this, cx| {
                                                this.run_test_connection(cx);
                                            });
                                        }
                                    })
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_semibold()
                                            .text_color(tokens.accent_text)
                                            .child("Probar conexión"),
                                    ),
                            )
                            .when_some(test_msg, |this, msg| {
                                this.child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap(px(6.0))
                                        .px(px(12.0))
                                        .py(px(6.0))
                                        .rounded(radius)
                                        .bg(tokens.bg_sunken)
                                        .border_1()
                                        .border_color(if msg.starts_with('✗') {
                                            tokens.danger
                                        } else {
                                            tokens.success
                                        })
                                        .text_xs()
                                        .font_medium()
                                        .text_color(if msg.starts_with('✗') {
                                            tokens.danger
                                        } else {
                                            tokens.success
                                        })
                                        .child(msg),
                                )
                            }),
                    ),
            )
            // Section 2: Motor de Embeddings (Indexación RAG)
            .child(
                SettingsSection::new(
                    "sec-embeddings",
                    "Motor de Embeddings (Indexación RAG)",
                    tokens,
                    radius,
                )
                .icon(IconName::Layers)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .p(px(16.0))
                        .gap(px(12.0))
                        .child(
                            div()
                                .text_xs()
                                .text_color(tokens.text_muted)
                                .child(
                                    "Ningún modelo se descarga al inicio. Elige cómo vectorizar tus libros; \
                                     una vez indexados, cambiar de modelo requiere reindexar o borrar la base actual.",
                                ),
                        )
                        // Mode selector: Local vs Cloud
                        .child({
                            let view_local = view.clone();
                            let view_cloud = view.clone();
                            div()
                                .flex()
                                .flex_row()
                                .gap(px(10.0))
                                .child(
                                    div()
                                        .id("emb-mode-local")
                                        .px(px(14.0))
                                        .py(px(8.0))
                                        .rounded(radius)
                                        .cursor_pointer()
                                        .border_1()
                                        .border_color(if current_emb_source == EmbeddingSource::Local {
                                            tokens.accent
                                        } else {
                                            tokens.border
                                        })
                                        .bg(if current_emb_source == EmbeddingSource::Local {
                                            tokens.bg_sunken
                                        } else {
                                            tokens.bg_elevated
                                        })
                                        .text_xs()
                                        .font_semibold()
                                        .text_color(if current_emb_source == EmbeddingSource::Local {
                                            tokens.accent
                                        } else {
                                            tokens.text
                                        })
                                        .on_click(move |_, _, cx| {
                                            view_local.update(cx, |this, cx| {
                                                if this.settings.ai.embedding_source
                                                    != EmbeddingSource::Local
                                                {
                                                    this.settings.ai.embedding_source =
                                                        EmbeddingSource::Local;
                                                    this.persist();
                                                    cx.notify();
                                                }
                                            });
                                        })
                                        .child("Local en tu PC (ONNX / GPU-CPU)"),
                                )
                                .child(
                                    div()
                                        .id("emb-mode-cloud")
                                        .px(px(14.0))
                                        .py(px(8.0))
                                        .rounded(radius)
                                        .cursor_pointer()
                                        .border_1()
                                        .border_color(if current_emb_source == EmbeddingSource::Cloud {
                                            tokens.accent
                                        } else {
                                            tokens.border
                                        })
                                        .bg(if current_emb_source == EmbeddingSource::Cloud {
                                            tokens.bg_sunken
                                        } else {
                                            tokens.bg_elevated
                                        })
                                        .text_xs()
                                        .font_semibold()
                                        .text_color(if current_emb_source == EmbeddingSource::Cloud {
                                            tokens.accent
                                        } else {
                                            tokens.text
                                        })
                                        .on_click(move |_, _, cx| {
                                            view_cloud.update(cx, |this, cx| {
                                                this.request_embedding_config(
                                                    EmbeddingSource::Cloud,
                                                    None,
                                                    cx,
                                                );
                                            });
                                        })
                                        .child("Cloud (API del proveedor)"),
                                )
                        })
                        // Local models list (only shown when Local is selected)
                        .when(current_emb_source == EmbeddingSource::Local, |this| {
                            this.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(8.0))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(tokens.text_muted)
                                            .child(
                                                "⚠ Aviso: Al seleccionar un modelo se descargará a tu disco (la primera vez puede demorar unos minutos) y podrá usar GPU o CPU según disponibilidad.",
                                            ),
                                    )
                                    .children(LocalEmbeddingModelInfo::all().iter().map(|info| {
                                        let m_id = info.id.to_string();
                                        let is_selected = current_local_model.as_deref() == Some(info.id);
                                        let view_m = view.clone();
                                        div()
                                            .id(ElementId::from(format!("local-emb-{}", info.id)))
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .justify_between()
                                            .p(px(12.0))
                                            .rounded(radius)
                                            .border_1()
                                            .border_color(if is_selected {
                                                tokens.accent
                                            } else {
                                                tokens.border
                                            })
                                            .bg(if is_selected {
                                                tokens.bg_sunken
                                            } else {
                                                tokens.bg
                                            })
                                            .cursor_pointer()
                                            .hover(|s| s.border_color(tokens.accent))
                                            .on_click(move |_, _, cx| {
                                                if !is_downloading {
                                                    let id = m_id.clone();
                                                    view_m.update(cx, |this, cx| {
                                                        this.request_embedding_config(
                                                            EmbeddingSource::Local,
                                                            Some(id),
                                                            cx,
                                                        );
                                                    });
                                                }
                                            })
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .gap(px(2.0))
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .flex_row()
                                                            .items_center()
                                                            .gap(px(8.0))
                                                            .child(
                                                                div()
                                                                    .text_xs()
                                                                    .font_semibold()
                                                                    .text_color(tokens.text)
                                                                    .child(info.display_name),
                                                            )
                                                            .when(info.recommended, |this| {
                                                                this.child(
                                                                    div()
                                                                        .px(px(6.0))
                                                                        .py(px(1.0))
                                                                        .rounded_full()
                                                                        .bg(tokens.accent)
                                                                        .text_size(px(10.0))
                                                                        .font_bold()
                                                                        .text_color(tokens.accent_text)
                                                                        .child("Recomendado"),
                                                                )
                                                            }),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_size(px(11.0))
                                                            .text_color(tokens.text_muted)
                                                            .child(format!(
                                                                "{} • {} dims • {}",
                                                                info.approx_size_label,
                                                                info.dimensions,
                                                                info.languages
                                                            )),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_size(px(11.0))
                                                            .text_color(tokens.text_muted)
                                                            .child(info.description),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_semibold()
                                                    .text_color(if is_selected {
                                                        tokens.accent
                                                    } else {
                                                        tokens.text_muted
                                                    })
                                                    .child(if is_selected {
                                                        "Activo"
                                                    } else {
                                                        "Descargar y usar"
                                                    }),
                                            )
                                    })),
                            )
                        })
                        // Confirmation banner when switching models on existing index
                        .when_some(pending_switch, |this, (src, model)| {
                            let view_confirm = view.clone();
                            let view_cancel = view.clone();
                            this.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(8.0))
                                    .p(px(12.0))
                                    .rounded(radius)
                                    .bg(tokens.bg_sunken)
                                    .border_1()
                                    .border_color(tokens.accent)
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_medium()
                                            .text_color(tokens.text)
                                            .child(
                                                "Cambiar de modelo invalida los embeddings actuales. ¿Deseas borrar el índice actual para usar el nuevo modelo?",
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .gap(px(8.0))
                                            .child(
                                                div()
                                                    .id("btn-confirm-reindex-switch")
                                                    .px(px(12.0))
                                                    .py(px(6.0))
                                                    .rounded(px(4.0))
                                                    .bg(tokens.accent)
                                                    .text_xs()
                                                    .font_semibold()
                                                    .text_color(tokens.accent_text)
                                                    .cursor_pointer()
                                                    .on_click(move |_, _, cx| {
                                                        let m = model.clone();
                                                        view_confirm.update(cx, |this, cx| {
                                                            this.apply_embedding_config(
                                                                src, m, true, cx,
                                                            );
                                                        });
                                                    })
                                                    .child("Borrar índice y cambiar"),
                                            )
                                            .child(
                                                div()
                                                    .id("btn-cancel-reindex-switch")
                                                    .px(px(12.0))
                                                    .py(px(6.0))
                                                    .rounded(px(4.0))
                                                    .bg(tokens.bg_elevated)
                                                    .border_1()
                                                    .border_color(tokens.border)
                                                    .text_xs()
                                                    .text_color(tokens.text)
                                                    .cursor_pointer()
                                                    .on_click(move |_, _, cx| {
                                                        view_cancel.update(cx, |this, cx| {
                                                            this.pending_model_switch = None;
                                                            this.embedding_status_msg = None;
                                                            cx.notify();
                                                        });
                                                    })
                                                    .child("Cancelar"),
                                            ),
                                    ),
                            )
                        })
                        // Status message
                        .when_some(emb_status, |this, msg| {
                            this.child(
                                div()
                                    .px(px(12.0))
                                    .py(px(8.0))
                                    .rounded(radius)
                                    .bg(tokens.bg_sunken)
                                    .border_1()
                                    .border_color(tokens.border)
                                    .text_xs()
                                    .text_color(tokens.text)
                                    .child(msg),
                            )
                        })
                        // Index & storage footer + delete button
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .pt(px(8.0))
                                .border_t_1()
                                .border_color(tokens.border)
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(tokens.text_muted)
                                        .child(match index_summary {
                                            Some(s) => format!(
                                                "Índice: {} libro(s), {} fragmentos • Modelos en disco: {:.1} MB",
                                                s.indexed_books,
                                                s.total_chunks,
                                                disk_bytes as f64 / (1024.0 * 1024.0)
                                            ),
                                            None => format!(
                                                "Sin libros indexados • Modelos en disco: {:.1} MB",
                                                disk_bytes as f64 / (1024.0 * 1024.0)
                                            ),
                                        }),
                                )
                                .child({
                                    let view_clear = view.clone();
                                    div()
                                        .id("btn-clear-models-embeddings")
                                        .px(px(12.0))
                                        .py(px(6.0))
                                        .rounded(px(4.0))
                                        .bg(tokens.bg_sunken)
                                        .border_1()
                                        .border_color(tokens.danger.opacity(0.6))
                                        .text_xs()
                                        .font_medium()
                                        .text_color(tokens.danger)
                                        .cursor_pointer()
                                        .hover(|s| s.bg(tokens.bg_hover))
                                        .on_click(move |_, _, cx| {
                                            view_clear.update(cx, |this, cx| {
                                                this.clear_all_models_and_embeddings(cx);
                                            });
                                        })
                                        .child("Borrar modelos y embeddings")
                                }),
                        ),
                ),
            )
            // Section 3: Privacy Guarantee Banner (Fixed wrapping!)
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_row()
                    .items_start()
                    .gap(px(14.0))
                    .p(px(16.0))
                    .rounded(radius)
                    .bg(tokens.bg_elevated)
                    .border_1()
                    .border_color(tokens.border)
                    .child(
                        div()
                            .flex_shrink_0()
                            .p(px(8.0))
                            .rounded_full()
                            .bg(tokens.bg_sunken)
                            .child(
                                Icon::new(IconName::Shield)
                                    .size(px(20.0))
                                    .text_color(tokens.accent),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w_0()
                            .gap(px(4.0))
                            .child(
                                div()
                                    .text_sm()
                                    .font_semibold()
                                    .text_color(tokens.text)
                                    .whitespace_normal()
                                    .child("Privacidad y Ejecución Local Garantizada"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(tokens.text_muted)
                                    .whitespace_normal()
                                    .child("Tus libros, notas, marcas y ritmo de lectura permanecen 100% en tu disco local. Las llamadas a IA son opcionales y solo se activan cuando tú solicitas una explicación o resumen."),
                            ),
                    ),
            )
            .into_any_element()
    }

    fn render_sheet_tab(
        &mut self,
        tokens: crate::theme::tokens::ThemeTokens,
        radius: Pixels,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let view = cx.entity().clone();
        let reader = &self.settings.reader;

        let font_size = reader.font_size;
        let line_height = reader.line_height;
        let justify = reader.justify_text;
        let follow_theme = reader.follow_app_theme;
        let font_family = reader.font_family.clone();
        let is_select_open = self.is_font_select_open;

        // Map abstract font family to Linux installed system font
        let font_system_name = if font_family.contains("Sans") {
            "Liberation Sans"
        } else if font_family.contains("Mono") {
            "Liberation Mono"
        } else {
            "Liberation Serif"
        };

        div()
            .flex()
            .flex_col()
            .w_full()
            .gap(px(20.0))
            // Section 1: Tipografía (Point 3 fix: Dropdown Select + Live Font Family preview)
            .child(
                SettingsSection::new("sec-sheet-typo", "Tipografía de lectura", tokens, radius)
                    .icon(IconName::BookOpen)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .p(px(16.0))
                            .gap(px(8.0))
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap(px(2.0))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_medium()
                                                    .text_color(tokens.text)
                                                    .child("Familia tipográfica"),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(tokens.text_muted)
                                                    .child("Selecciona la fuente para el cuerpo del texto en el lector:"),
                                            ),
                                    )
                                    // Custom Dropdown Trigger Button
                                    .child({
                                        let view_toggle = view.clone();
                                        div()
                                            .id("btn-font-select-trigger")
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap(px(8.0))
                                            .px(px(14.0))
                                            .py(px(6.0))
                                            .rounded(radius)
                                            .bg(tokens.bg_sunken)
                                            .border_1()
                                            .border_color(if is_select_open { tokens.accent } else { tokens.border })
                                            .hover(|s| s.bg(tokens.bg_hover))
                                            .cursor_pointer()
                                            .on_click(move |_, _, cx| {
                                                view_toggle.update(cx, |this, cx| {
                                                    this.is_font_select_open = !this.is_font_select_open;
                                                    cx.notify();
                                                });
                                            })
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_medium()
                                                    .text_color(tokens.text)
                                                    .child(font_family.clone()),
                                            )
                                            .child(
                                                Icon::new(if is_select_open { IconName::ChevronUp } else { IconName::ChevronDown })
                                                    .size(px(14.0))
                                                    .text_color(tokens.text_muted),
                                            )
                                    }),
                            )
                            // Dropdown options list when open
                            .when(is_select_open, |this| {
                                let view_opt1 = view.clone();
                                let view_opt2 = view.clone();
                                let view_opt3 = view.clone();

                                this.child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .mt(px(4.0))
                                        .p(px(4.0))
                                        .rounded(radius)
                                        .bg(tokens.bg_sunken)
                                        .border_1()
                                        .border_color(tokens.accent)
                                        // Option 1: Serif
                                        .child(
                                            div()
                                                .id("font-opt-serif")
                                                .px(px(12.0))
                                                .py(px(8.0))
                                                .rounded(px(4.0))
                                                .cursor_pointer()
                                                .hover(|s| s.bg(tokens.bg_hover))
                                                .on_click(move |_, _, cx| {
                                                    view_opt1.update(cx, |this, cx| {
                                                        this.settings.reader.font_family = "Serif (Georgia)".into();
                                                        this.is_font_select_open = false;
                                                        ReaderSettingsState::update(cx, this.settings.reader.clone());
                                                        this.persist();
                                                        cx.notify();
                                                    });
                                                })
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_medium()
                                                        .text_color(tokens.text)
                                                        .child("✓ Serif (Liberation Serif / Georgia) — Recomendada para lectura continua"),
                                                ),
                                        )
                                        // Option 2: Sans
                                        .child(
                                            div()
                                                .id("font-opt-sans")
                                                .px(px(12.0))
                                                .py(px(8.0))
                                                .rounded(px(4.0))
                                                .cursor_pointer()
                                                .hover(|s| s.bg(tokens.bg_hover))
                                                .on_click(move |_, _, cx| {
                                                    view_opt2.update(cx, |this, cx| {
                                                        this.settings.reader.font_family = "Sans (Inter)".into();
                                                        this.is_font_select_open = false;
                                                        ReaderSettingsState::update(cx, this.settings.reader.clone());
                                                        this.persist();
                                                        cx.notify();
                                                    });
                                                })
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_medium()
                                                        .text_color(tokens.text)
                                                        .child("• Sans-serif (Liberation Sans / Inter) — Moderna y limpia"),
                                                ),
                                        )
                                        // Option 3: Mono
                                        .child(
                                            div()
                                                .id("font-opt-mono")
                                                .px(px(12.0))
                                                .py(px(8.0))
                                                .rounded(px(4.0))
                                                .cursor_pointer()
                                                .hover(|s| s.bg(tokens.bg_hover))
                                                .on_click(move |_, _, cx| {
                                                    view_opt3.update(cx, |this, cx| {
                                                        this.settings.reader.font_family = "Mono (JetBrains)".into();
                                                        this.is_font_select_open = false;
                                                        ReaderSettingsState::update(cx, this.settings.reader.clone());
                                                        this.persist();
                                                        cx.notify();
                                                    });
                                                })
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_medium()
                                                        .text_color(tokens.text)
                                                        .child("• Monospace (Liberation Mono / JetBrains) — Código y técnica"),
                                                ),
                                        ),
                                )
                            }),
                    )
                    .child(
                        SettingRow::new(
                            "row-sheet-size",
                            "Tamaño de letra",
                            tokens,
                            {
                                let view_dec = view.clone();
                                let view_inc = view.clone();
                                Stepper::new("step-sheet-size", format!("{} px", font_size), tokens)
                                    .on_decrement(move |_, cx| {
                                        view_dec.update(cx, |this, cx| {
                                            if this.settings.reader.font_size > 6 {
                                                this.settings.reader.font_size -= 1;
                                                ReaderSettingsState::update(cx, this.settings.reader.clone());
                                                this.persist();
                                                cx.notify();
                                            }
                                        });
                                    })
                                    .on_increment(move |_, cx| {
                                        view_inc.update(cx, |this, cx| {
                                            if this.settings.reader.font_size < 36 {
                                                this.settings.reader.font_size += 1;
                                                ReaderSettingsState::update(cx, this.settings.reader.clone());
                                                this.persist();
                                                cx.notify();
                                            }
                                        });
                                    })
                            },
                        )
                        .description("Escala adaptable entre 6 y 36 píxeles."),
                    )
                    .child(
                        SettingRow::new(
                            "row-sheet-lh",
                            "Interlineado",
                            tokens,
                            {
                                let view_dec = view.clone();
                                let view_inc = view.clone();
                                Stepper::new("step-sheet-lh", format!("{:.1}x", line_height), tokens)
                                    .on_decrement(move |_, cx| {
                                        view_dec.update(cx, |this, cx| {
                                            if this.settings.reader.line_height > 1.25 {
                                                this.settings.reader.line_height -= 0.1;
                                                ReaderSettingsState::update(cx, this.settings.reader.clone());
                                                this.persist();
                                                cx.notify();
                                            }
                                        });
                                    })
                                    .on_increment(move |_, cx| {
                                        view_inc.update(cx, |this, cx| {
                                            if this.settings.reader.line_height < 2.35 {
                                                this.settings.reader.line_height += 0.1;
                                                ReaderSettingsState::update(cx, this.settings.reader.clone());
                                                this.persist();
                                                cx.notify();
                                            }
                                        });
                                    })
                            },
                        )
                        .description("Espaciado vertical entre líneas de párrafo."),
                    ),
            )
            // Section 2: Formato de Hoja
            .child(
                SettingsSection::new("sec-sheet-format", "Formato de Hoja", tokens, radius)
                    .icon(IconName::SlidersHorizontal)
                    .child(
                        SettingRow::new(
                            "row-justify",
                            "Justificar texto",
                            tokens,
                            {
                                let view_j = view.clone();
                                Switch::new("switch-justify")
                                    .checked(justify)
                                    .on_click(move |checked, _, cx| {
                                        view_j.update(cx, |this, cx| {
                                            this.settings.reader.justify_text = *checked;
                                            ReaderSettingsState::update(cx, this.settings.reader.clone());
                                            this.persist();
                                            cx.notify();
                                        });
                                    })
                            },
                        )
                        .description("Alinea uniformemente los márgenes izquierdo y derecho de los párrafos."),
                    )
                    .child(
                        SettingRow::new(
                            "row-follow-theme",
                            "Seguir tema de la aplicación",
                            tokens,
                            {
                                let view_t = view.clone();
                                Switch::new("switch-follow-theme")
                                    .checked(follow_theme)
                                    .on_click(move |checked, _, cx| {
                                        view_t.update(cx, |this, cx| {
                                            this.settings.reader.follow_app_theme = *checked;
                                            ReaderSettingsState::update(cx, this.settings.reader.clone());
                                            this.persist();
                                            cx.notify();
                                        });
                                    })
                            },
                        )
                        .description("Adapta el fondo y tinta de la hoja al tema activo de la interfaz."),
                    ),
            )
            // Section 3: Live Preview Box (Now visibly changes font_family and typography!)
            .child(
                SettingsSection::new("sec-live-preview", "Vista previa en vivo", tokens, radius)
                    .icon(IconName::Sparkles)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .p(px(20.0))
                            .gap(px(12.0))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(tokens.text_muted)
                                    .child(format!("Vista previa con fuente activa: {} ({}):", font_family, font_system_name)),
                            )
                            // Live Preview Sheet container applying font_family, font_size, line_height
                            .child(
                                div()
                                    .w_full()
                                    .p(px(24.0))
                                    .rounded(radius)
                                    .bg(tokens.bg_sunken)
                                    .border_1()
                                    .border_color(tokens.border)
                                    .font_family(font_system_name)
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap(px(12.0))
                                            .text_size(px(font_size as f32))
                                            .line_height(relative(line_height))
                                            .text_color(tokens.text)
                                            .child(
                                                div()
                                                    .font_bold()
                                                    .text_size(px((font_size as f32) * 1.22))
                                                    .child("Capítulo I: El Primer Resplandor"),
                                            )
                                            .child(
                                                div().child(
                                                    "La luz del faro cortaba la niebla con una persistencia casi obstinada. A lo lejos, el rumor del mar contra los arrecifes marcaba el compás de una noche que prometía extenderse más allá del amanecer. Cada palabra escrita en aquel viejo cuaderno parecía recuperar su peso exacto en medio de la quietud."
                                                ),
                                            ),
                                    ),
                            ),
                    ),
            )
            .into_any_element()
    }
}
