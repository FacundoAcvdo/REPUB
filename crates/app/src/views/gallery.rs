use epub_core::domain::settings::CornerRadiusStyle;
use gpui_kit::base::Disableable as _;
use gpui_kit::base::StyledExt as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::component::switch::Switch;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::components::corner_picker::CornerCard;
use crate::components::segmented::{SegmentedControl, SegmentedItem};
use crate::components::stepper::Stepper;
use crate::components::theme_card::ThemeCard;
use crate::i18n::I18n;
use crate::theme::palettes::all_themes;
use crate::theme::state::ThemeState;
use crate::theme::typography::{ReaderTypography, TypographyScale};

pub struct GalleryView {
    sample_text_size: u32,
    sample_switch_active: bool,
    sample_segmented_id: &'static str,
}

impl GalleryView {
    pub fn new() -> Self {
        Self {
            sample_text_size: 14,
            sample_switch_active: true,
            sample_segmented_id: "system",
        }
    }

    pub fn decrement_text_size(&mut self, cx: &mut Context<Self>) {
        if self.sample_text_size > 10 {
            self.sample_text_size -= 1;
            cx.notify();
        }
    }

    pub fn increment_text_size(&mut self, cx: &mut Context<Self>) {
        if self.sample_text_size < 32 {
            self.sample_text_size += 1;
            cx.notify();
        }
    }

    pub fn set_segmented_id(&mut self, id: &'static str, cx: &mut Context<Self>) {
        self.sample_segmented_id = id;
        cx.notify();
    }

    pub fn set_switch_active(&mut self, active: bool, cx: &mut Context<Self>) {
        self.sample_switch_active = active;
        cx.notify();
    }
}

impl Render for GalleryView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme_state = ThemeState::global(cx).clone();
        let tokens = theme_state.tokens();
        let current_theme_id = tokens.id;
        let current_radius = theme_state.corner_radius;
        let window_opacity = theme_state.window_opacity;
        let interface_blur = theme_state.interface_blur;
        let radius = theme_state.radius();

        let entity = cx.entity().clone();

        div()
            .flex()
            .flex_col()
            .size_full()
            .overflow_y_scrollbar()
            .text_color(tokens.text)
            .p(px(32.0))
            .gap(px(32.0))
            // Top Header
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_size(TypographyScale::TITLE_LG)
                                    .font_bold()
                                    .text_color(tokens.text)
                                    .child(I18n::GALLERY_TITLE),
                            )
                            .child(
                                div()
                                    .px(px(10.0))
                                    .py(px(4.0))
                                    .rounded_full()
                                    .bg(tokens.bg_elevated)
                                    .border_1()
                                    .border_color(tokens.border)
                                    .text_xs()
                                    .font_medium()
                                    .text_color(tokens.accent)
                                    .child(I18n::HINT_GALLERY_TOGGLE),
                            ),
                    )
                    .child(
                        div()
                            .text_size(TypographyScale::BASE)
                            .text_color(tokens.text_muted)
                            .child(I18n::GALLERY_SUBTITLE),
                    ),
            )
            // SECTION 1: Palettes & Themes
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(12.0))
                    .p(px(20.0))
                    .rounded(radius)
                    .bg(tokens.bg_elevated)
                    .border_1()
                    .border_color(tokens.border)
                    .child(
                        div()
                            .text_size(TypographyScale::MICRO)
                            .font_bold()
                            .text_color(tokens.text_muted)
                            .child(I18n::GALLERY_THEMES),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(14.0))
                            .children(all_themes().into_iter().map(|preset| {
                                let id = preset.id;
                                let is_selected = id == current_theme_id;
                                ThemeCard::new(id, preset, is_selected).on_click(move |_, cx| {
                                    ThemeState::update_theme(cx, id);
                                })
                            })),
                    ),
            )
            // SECTION 2: Corner Radius Picker
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(12.0))
                    .p(px(20.0))
                    .rounded(radius)
                    .bg(tokens.bg_elevated)
                    .border_1()
                    .border_color(tokens.border)
                    .child(
                        div()
                            .text_size(TypographyScale::MICRO)
                            .font_bold()
                            .text_color(tokens.text_muted)
                            .child(I18n::GALLERY_CORNERS),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(16.0))
                            .child(
                                CornerCard::new(
                                    "corner_square",
                                    CornerRadiusStyle::Square,
                                    I18n::CORNER_SQUARE,
                                    *tokens,
                                    current_radius == CornerRadiusStyle::Square,
                                )
                                .on_click(|_, cx| {
                                    ThemeState::update_corner_radius(cx, CornerRadiusStyle::Square);
                                }),
                            )
                            .child(
                                CornerCard::new(
                                    "corner_subtle",
                                    CornerRadiusStyle::Subtle,
                                    I18n::CORNER_SUBTLE,
                                    *tokens,
                                    current_radius == CornerRadiusStyle::Subtle,
                                )
                                .on_click(|_, cx| {
                                    ThemeState::update_corner_radius(cx, CornerRadiusStyle::Subtle);
                                }),
                            )
                            .child(
                                CornerCard::new(
                                    "corner_rounded",
                                    CornerRadiusStyle::Rounded,
                                    I18n::CORNER_ROUNDED,
                                    *tokens,
                                    current_radius == CornerRadiusStyle::Rounded,
                                )
                                .on_click(|_, cx| {
                                    ThemeState::update_corner_radius(cx, CornerRadiusStyle::Rounded);
                                }),
                            )
                            .child(
                                CornerCard::new(
                                    "corner_circular",
                                    CornerRadiusStyle::Circular,
                                    I18n::CORNER_CIRCULAR,
                                    *tokens,
                                    current_radius == CornerRadiusStyle::Circular,
                                )
                                .on_click(|_, cx| {
                                    ThemeState::update_corner_radius(cx, CornerRadiusStyle::Circular);
                                }),
                            ),
                    ),
            )
            // SECTION 3: Window Opacity & Blur
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(16.0))
                    .p(px(20.0))
                    .rounded(radius)
                    .bg(tokens.bg_elevated)
                    .border_1()
                    .border_color(tokens.border)
                    .child(
                        div()
                            .text_size(TypographyScale::MICRO)
                            .font_bold()
                            .text_color(tokens.text_muted)
                            .child(I18n::GALLERY_WINDOW),
                    )
                    // Row 1: Opacity selector pills
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
                                            .text_size(TypographyScale::BASE)
                                            .font_medium()
                                            .child(I18n::WINDOW_OPACITY),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(tokens.text_muted)
                                            .child("Regula el canal alfa de la ventana base"),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(8.0))
                                    .children([100, 90, 80, 70, 60].into_iter().map(|pct| {
                                        let alpha = pct as f32 / 100.0;
                                        let is_active = (window_opacity * 100.0).round() as u32 == pct;
                                        div()
                                            .id(ElementId::from(format!("op_{}", pct)))
                                            .px(px(12.0))
                                            .py(px(5.0))
                                            .rounded_full()
                                            .cursor_pointer()
                                            .text_xs()
                                            .font_medium()
                                            .when(is_active, |s| {
                                                s.bg(tokens.accent)
                                                    .text_color(tokens.accent_text)
                                            })
                                            .when(!is_active, |s| {
                                                s.bg(tokens.bg_sunken)
                                                    .text_color(tokens.text_muted)
                                                    .border_1()
                                                    .border_color(tokens.border)
                                                    .hover(|s| s.bg(tokens.bg_hover))
                                            })
                                            .on_click(move |_, window, cx| {
                                                ThemeState::update_window_opacity(cx, alpha);
                                                window.set_background_appearance(WindowBackgroundAppearance::Transparent);
                                            })
                                            .child(format!("{}%", pct))
                                    })),
                            ),
                    )
                    // Row 2: Blur toggle
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
                                            .text_size(TypographyScale::BASE)
                                            .font_medium()
                                            .child(I18n::WINDOW_BLUR),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(tokens.text_muted)
                                            .child("Efecto de desenfoque combinado con opacidad"),
                                    ),
                            )
                            .child(
                                Switch::new("gallery_blur_switch")
                                    .checked(interface_blur)
                                    .on_click(move |checked, window, cx| {
                                        let is_blur = *checked;
                                        ThemeState::update_interface_blur(cx, is_blur);
                                        window.set_background_appearance(if is_blur {
                                            WindowBackgroundAppearance::Blurred
                                        } else {
                                            WindowBackgroundAppearance::Transparent
                                        });
                                    }),
                            ),
                    ),
            )
            // SECTION 4: Interactive Controls & Buttons
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(16.0))
                    .p(px(20.0))
                    .rounded(radius)
                    .bg(tokens.bg_elevated)
                    .border_1()
                    .border_color(tokens.border)
                    .child(
                        div()
                            .text_size(TypographyScale::MICRO)
                            .font_bold()
                            .text_color(tokens.text_muted)
                            .child(I18n::GALLERY_CONTROLS),
                    )
                    // Button variants
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .items_center()
                            .gap(px(12.0))
                            .child(
                                Button::new("btn_primary")
                                    .label(I18n::BTN_PRIMARY)
                                    .primary(),
                            )
                            .child(
                                Button::new("btn_outline")
                                    .label(I18n::BTN_OUTLINE)
                                    .outline(),
                            )
                            .child(
                                Button::new("btn_ghost")
                                    .label(I18n::BTN_GHOST)
                                    .ghost(),
                            )
                            .child(
                                Button::new("btn_danger")
                                    .label(I18n::BTN_DANGER)
                                    .danger(),
                            )
                            .child(
                                Button::new("btn_disabled")
                                    .label(I18n::BTN_DISABLED)
                                    .disabled(true),
                            ),
                    )
                    // Stepper, Switch & Segmented Control
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .items_center()
                            .gap(px(24.0))
                            .pt(px(8.0))
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(10.0))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_medium()
                                            .text_color(tokens.text_muted)
                                            .child("Tamaño:"),
                                    )
                                    .child({
                                        let entity_dec = entity.clone();
                                        let entity_inc = entity.clone();
                                        Stepper::new(
                                            "gallery_stepper",
                                            format!("{} px", self.sample_text_size),
                                            *tokens,
                                        )
                                        .on_decrement(move |_, cx| {
                                            entity_dec.update(cx, |this, cx| {
                                                this.decrement_text_size(cx);
                                            });
                                        })
                                        .on_increment(move |_, cx| {
                                            entity_inc.update(cx, |this, cx| {
                                                this.increment_text_size(cx);
                                            });
                                        })
                                    }),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(10.0))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_medium()
                                            .text_color(tokens.text_muted)
                                            .child("Movimiento:"),
                                    )
                                    .child({
                                        let entity_seg = entity.clone();
                                        SegmentedControl::new(
                                            "gallery_segmented",
                                            vec![
                                                SegmentedItem {
                                                    id: "system",
                                                    label: "Sistema",
                                                },
                                                SegmentedItem {
                                                    id: "always",
                                                    label: "Siempre",
                                                },
                                                SegmentedItem {
                                                    id: "never",
                                                    label: "Nunca",
                                                },
                                            ],
                                            self.sample_segmented_id,
                                            *tokens,
                                        )
                                        .on_select(move |selected, _, cx| {
                                            entity_seg.update(cx, |this, cx| {
                                                this.set_segmented_id(selected, cx);
                                            });
                                        })
                                    }),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(10.0))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_medium()
                                            .text_color(tokens.text_muted)
                                            .child("Interruptor:"),
                                    )
                                    .child({
                                        let entity_sw = entity.clone();
                                        Switch::new("sample_switch")
                                            .checked(self.sample_switch_active)
                                            .on_click(move |checked, _, cx| {
                                                let val = *checked;
                                                entity_sw.update(cx, |this, cx| {
                                                    this.set_switch_active(val, cx);
                                                });
                                            })
                                    }),
                            ),
                    ),
            )
            // SECTION 5: Typography Scale & Reading Sheet Preview
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(16.0))
                    .p(px(20.0))
                    .rounded(radius)
                    .bg(tokens.bg_elevated)
                    .border_1()
                    .border_color(tokens.border)
                    .child(
                        div()
                            .text_size(TypographyScale::MICRO)
                            .font_bold()
                            .text_color(tokens.text_muted)
                            .child(I18n::GALLERY_TYPOGRAPHY),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(10.0))
                            .child(
                                div()
                                    .text_size(TypographyScale::TITLE_LG)
                                    .font_bold()
                                    .child("Título Grande (26px negrita)"),
                            )
                            .child(
                                div()
                                    .text_size(TypographyScale::TITLE_SM)
                                    .font_semibold()
                                    .child("Título Secundario (20px semi-negrita)"),
                            )
                            .child(
                                div()
                                    .text_size(TypographyScale::BASE)
                                    .child("Texto base de la interfaz (14px regular) para controles, menús y formularios."),
                            )
                            .child(
                                div()
                                    .text_size(TypographyScale::CAPTION)
                                    .text_color(tokens.text_muted)
                                    .child("Texto secundario de leyenda o pie (12px atenuado)."),
                            ),
                    )
                    // Reading Sheet Preview Card
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(8.0))
                            .p(px(20.0))
                            .rounded(radius)
                            .bg(tokens.bg_sunken)
                            .border_1()
                            .border_color(tokens.border)
                            .child(
                                div()
                                    .text_xs()
                                    .font_semibold()
                                    .text_color(tokens.accent)
                                    .child("VISTA PREVIA DE TIPOGRAFÍA DE HOJA (SERIF)"),
                            )
                            .child(
                                div()
                                    .font_family(ReaderTypography::SERIF_FALLBACK)
                                    .text_size(px(16.0))
                                    .line_height(px(26.0))
                                    .text_color(tokens.text)
                                    .max_w(ReaderTypography::MAX_COLUMN_WIDTH)
                                    .child(
                                        "«La luz declinaba suavemente sobre la orilla, extendiendo un manto de sombras largas que se perdían entre las piedras pulidas por la marea. A lo lejos, la silueta del faro comenzaba a destellar con su pulso mecánico y calmo, un latido de fuego blanco en medio de la penumbra creciente...»",
                                    ),
                            ),
                    ),
            )
            // SECTION 6: Token Swatches
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(12.0))
                    .p(px(20.0))
                    .rounded(radius)
                    .bg(tokens.bg_elevated)
                    .border_1()
                    .border_color(tokens.border)
                    .child(
                        div()
                            .text_size(TypographyScale::MICRO)
                            .font_bold()
                            .text_color(tokens.text_muted)
                            .child(I18n::GALLERY_TOKENS_SWATCH),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(12.0))
                            .children([
                                ("bg", tokens.bg),
                                ("bg_elevated", tokens.bg_elevated),
                                ("bg_sunken", tokens.bg_sunken),
                                ("border", tokens.border),
                                ("accent", tokens.accent),
                                ("text", tokens.text),
                                ("text_muted", tokens.text_muted),
                                ("success", tokens.success),
                                ("warning", tokens.warning),
                                ("danger", tokens.danger),
                            ].into_iter().map(|(name, color)| {
                                div()
                                    .flex()
                                    .flex_col()
                                    .items_center()
                                    .gap(px(4.0))
                                    .child(
                                        div()
                                            .w(px(54.0))
                                            .h(px(32.0))
                                            .rounded(px(6.0))
                                            .bg(color)
                                            .border_1()
                                            .border_color(tokens.border),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(tokens.text_muted)
                                            .child(name),
                                    )
                            })),
                    ),
            )
    }
}
