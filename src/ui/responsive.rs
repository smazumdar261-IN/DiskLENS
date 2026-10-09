use crate::theme::{AppTheme, ThemeTokens};
use eframe::egui::{self, Color32, Margin, RichText, Ui};
use egui_extras::{Column, TableBuilder};

/// Reusable responsive card container adhering to egui/eframe best practices.
/// Dynamically expands to `ui.available_width()` without rigid pixel clamps.
#[allow(dead_code)]
pub fn responsive_card<R>(
    ui: &mut Ui,
    title: &str,
    dark_mode: bool,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    let t = AppTheme::tokens(dark_mode);
    egui::Frame::none()
        .fill(t.bg_elevated)
        .rounding(6.0_f32)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical(|ui| {
                ui.label(RichText::new(title).size(12.5).strong().color(t.text));
                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);
                add_contents(ui)
            })
        })
        .inner
        .inner
}

/// Reusable responsive card with a right-aligned header toolbar/action.
#[allow(dead_code)]
pub fn responsive_card_with_header<R>(
    ui: &mut Ui,
    title: &str,
    dark_mode: bool,
    add_header_actions: impl FnOnce(&mut Ui),
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    let t = AppTheme::tokens(dark_mode);
    egui::Frame::none()
        .fill(t.bg_elevated)
        .rounding(6.0_f32)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(title).size(12.5).strong().color(t.text));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        add_header_actions(ui);
                    });
                });
                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);
                add_contents(ui)
            })
        })
        .inner
        .inner
}

/// Reusable responsive metric / stat card for dashboards.
#[allow(dead_code)]
pub fn responsive_metric_tile(
    ui: &mut Ui,
    title: &str,
    value: &str,
    subtitle: &str,
    color: Color32,
    icon: &str,
    t: &ThemeTokens,
) {
    egui::Frame::none()
        .fill(t.bg_container)
        .rounding(6.0_f32)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(title)
                            .size(10.0)
                            .monospace()
                            .color(t.text_muted),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(icon).size(13.0).color(color));
                    });
                });
                ui.add_space(2.0);
                ui.label(
                    RichText::new(value)
                        .size(17.0)
                        .strong()
                        .monospace()
                        .color(color),
                );
                ui.add_space(2.0);
                ui.add(
                    egui::Label::new(RichText::new(subtitle).size(10.5).color(t.text_secondary))
                        .truncate(),
                )
                .on_hover_text(subtitle);
            });
        });
}

/// Check TableBuilder compilation
#[allow(dead_code)]
fn check_table_builder_api(ui: &mut Ui) {
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .vscroll(true)
        .column(Column::initial(100.0).resizable(true))
        .column(Column::remainder())
        .header(24.0, |mut header| {
            header.col(|ui| {
                ui.label("Col 1");
            });
            header.col(|ui| {
                ui.label("Col 2");
            });
        })
        .body(|body| {
            body.rows(26.0, 10, |mut row| {
                row.set_selected(true);
                let _resp = row.response();
                row.col(|ui| {
                    ui.label("Item");
                });
                row.col(|ui| {
                    ui.label("Details");
                });
            });
        });
}
