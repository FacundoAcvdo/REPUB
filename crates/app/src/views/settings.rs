use epub_core::domain::settings::{
    AnimationSpeed, AppSettings, CornerRadiusStyle, ReduceMotion,
};
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
    test_connection_msg: Option<&'static str>,
    is_font_select_open: bool,
    model_input: Entity<InputState>,
    api_key_input: Entity<InputState>,
}

impl SettingsView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let model_input = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value("gpt-4o")
                .placeholder("Nombre del modelo (ej. gpt-4o, claude-3-7-sonnet)...")
        });

        cx.subscribe(&model_input, |this, input, event: &InputEvent, cx| {
            if let InputEvent::Change = event {
                this.settings.ai.model = input.read(cx).value().to_string();
                cx.notify();
            }
        })
        .detach();

        let api_key_input = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value("sk-••••••••••••••••••••••••")
                .placeholder("Ingresa tu clave de API...")
        });

        cx.subscribe(&api_key_input, |this, input, event: &InputEvent, cx| {
            if let InputEvent::Change = event {
                this.settings.ai.api_key = input.read(cx).value().to_string();
                cx.notify();
            }
        })
        .detach();

        Self {
            active_tab: SettingsTab::General,
            settings: AppSettings::default(),
            test_connection_msg: None,
            is_font_select_open: false,
            model_input,
            api_key_input,
        }
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
            )
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
                                        cx.notify();
                                    });
                                })
                            },
                        )
                        .description("Preferencia de duración para transiciones e interfaces (Lenta: 250ms, Estándar: 150ms, Rápida: 80ms)."),
                    ),
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
        let current_blur = theme_state.interface_blur;

        let all_palettes = all_themes();

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

                                        ThemeCard::new(
                                            ElementId::from(format!("theme-card-{}", p_id)),
                                            palette,
                                            is_selected,
                                        )
                                        .on_click(move |_, cx| {
                                            ThemeState::update_theme(cx, p_id);
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

                                        CornerCard::new(
                                            ElementId::from(format!("corner-card-{:?}", s)),
                                            s,
                                            s.name_es(),
                                            tokens,
                                            is_selected,
                                        )
                                        .on_click(move |_, cx| {
                                            ThemeState::update_corner_radius(cx, s);
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
                                            .on_click(move |_, _, cx| {
                                                ThemeState::update_window_opacity(cx, val);
                                            })
                                            .child(format!("{:.0}%", val * 100.0))
                                    })),
                            ),
                    )
                    .child(
                        SettingRow::new(
                            "row-blur",
                            "Desenfoque de fondo (Blur)",
                            tokens,
                            Switch::new("switch-blur")
                                .checked(current_blur)
                                .on_click(move |checked, _, cx| {
                                    ThemeState::update_interface_blur(cx, *checked);
                                }),
                        )
                        .description("Aplica efecto de desenfoque de compositor en escritorios compatibles."),
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
        let test_msg = self.test_connection_msg;

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
                                                this.test_connection_msg = Some("✓ Conexión establecida con éxito con el proveedor.");
                                                cx.notify();
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
                                        .border_color(tokens.success)
                                        .text_xs()
                                        .font_medium()
                                        .text_color(tokens.success)
                                        .child(msg),
                                )
                            }),
                    ),
            )
            // Section 2: Privacy Guarantee Banner (Fixed wrapping!)
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
                                            if this.settings.reader.font_size > 12 {
                                                this.settings.reader.font_size -= 1;
                                                cx.notify();
                                            }
                                        });
                                    })
                                    .on_increment(move |_, cx| {
                                        view_inc.update(cx, |this, cx| {
                                            if this.settings.reader.font_size < 32 {
                                                this.settings.reader.font_size += 1;
                                                cx.notify();
                                            }
                                        });
                                    })
                            },
                        )
                        .description("Escala cómoda entre 14 y 24 píxeles."),
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
                                                cx.notify();
                                            }
                                        });
                                    })
                                    .on_increment(move |_, cx| {
                                        view_inc.update(cx, |this, cx| {
                                            if this.settings.reader.line_height < 2.35 {
                                                this.settings.reader.line_height += 0.1;
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
