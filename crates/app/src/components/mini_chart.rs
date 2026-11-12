use gpui_kit::assets::IconName;
use gpui_kit::base::StyledExt as _;
use gpui_kit::component::Icon;
use gpui_kit::*;

use crate::theme::tokens::ThemeTokens;

#[derive(IntoElement)]
pub struct WeeklyActivityChart {
    id: ElementId,
    activity: [u32; 7],
    tokens: ThemeTokens,
    radius: Pixels,
}

impl WeeklyActivityChart {
    pub fn new(
        id: impl Into<ElementId>,
        activity: [u32; 7],
        tokens: ThemeTokens,
        radius: Pixels,
    ) -> Self {
        Self {
            id: id.into(),
            activity,
            tokens,
            radius,
        }
    }
}

impl RenderOnce for WeeklyActivityChart {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let tokens = self.tokens;
        let radius = self.radius;

        let days = ["Lun", "Mar", "Mié", "Jue", "Vie", "Sáb", "Dom"];
        let max_val = self.activity.iter().copied().max().unwrap_or(1).max(1);
        let total_pages: u32 = self.activity.iter().sum();
        let avg_pages = total_pages / 7;

        div()
            .id(self.id)
            .flex()
            .flex_col()
            .w_full()
            .p(px(20.0))
            .rounded(radius)
            .bg(tokens.bg_elevated)
            .border_1()
            .border_color(tokens.border)
            .gap(px(16.0))
            // Header Row: Title & Total summary
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                Icon::new(IconName::TrendingUp)
                                    .size(px(18.0))
                                    .text_color(tokens.accent),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_semibold()
                                    .text_color(tokens.text)
                                    .child("Ritmo de Lectura Semanal"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .text_xs()
                                    .font_medium()
                                    .text_color(tokens.text_muted)
                                    .child(format!("{} páginas esta semana • Promedio {} pág/día", total_pages, avg_pages)),
                            ),
                    ),
            )
            // Bars Container
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_end()
                    .justify_between()
                    .w_full()
                    .h(px(130.0))
                    .pt(px(10.0))
                    .pb(px(4.0))
                    .gap(px(12.0))
                    .children(self.activity.iter().enumerate().map(|(idx, &val)| {
                        let day_name = days[idx];
                        let ratio = (val as f32 / max_val as f32).clamp(0.08, 1.0);
                        let is_peak = val == max_val;

                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_end()
                            .flex_1()
                            .h_full()
                            .gap(px(6.0))
                            // Value above bar
                            .child(
                                div()
                                    .text_xs()
                                    .font_semibold()
                                    .text_color(if is_peak { tokens.accent } else { tokens.text_muted })
                                    .child(format!("{}", val)),
                            )
                            // Bar Track & Fill
                            .child(
                                div()
                                    .relative()
                                    .w_full()
                                    .max_w(px(38.0))
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .justify_end()
                                    .rounded_full()
                                    .bg(tokens.bg_sunken)
                                    .overflow_hidden()
                                    .child(
                                        div()
                                            .w_full()
                                            .h(relative(ratio))
                                            .rounded_full()
                                            .bg(if is_peak { tokens.accent } else { tokens.border })
                                            .hover(|s| s.bg(tokens.accent)),
                                    ),
                            )
                            // Day Label below bar
                            .child(
                                div()
                                    .text_xs()
                                    .font_medium()
                                    .text_color(if is_peak { tokens.text } else { tokens.text_muted })
                                    .child(day_name),
                            )
                    })),
            )
    }
}
