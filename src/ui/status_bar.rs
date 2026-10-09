use crate::app::ActiveTab;
use crate::models::FileNode;
use crate::theme::AppTheme;
use eframe::egui::{self, Color32, RichText, Ui, Vec2};

pub fn render_status_bar(
    ui: &mut Ui,
    scanned_root: &Option<Box<FileNode>>,
    is_scanning: bool,
    _active_tab: &mut ActiveTab,
    dark_mode: bool,
    _on_open_logs: &mut dyn FnMut(),
    _on_open_mounts: &mut dyn FnMut(),
    _on_open_docs: &mut dyn FnMut(),
    on_open_command_palette: &mut dyn FnMut(),
) {
    let t = AppTheme::tokens(dark_mode);
    let available_w = ui.available_width();
    let is_narrow = available_w < 780.0;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(8.0, 0.0);

        // Status Pulse Indicator & Text
        let status_color = if is_scanning {
            t.warning
        } else {
            Color32::from_rgb(16, 185, 129)
        };
        let (dot, _) = ui.allocate_exact_size(Vec2::new(7.0, 7.0), egui::Sense::hover());
        ui.painter()
            .circle_filled(dot.center(), 3.5_f32, status_color);

        if is_scanning {
            ui.label(
                RichText::new("Indexing filesystem in background...")
                    .size(11.0)
                    .color(t.warning)
                    .strong(),
            );
        } else if let Some(root) = scanned_root {
            let info = format!(
                "Ready · {} files ({})",
                root.total_items(),
                root.formatted_size()
            );
            ui.label(RichText::new(info).size(11.0).color(if dark_mode {
                Color32::from_rgb(203, 213, 225)
            } else {
                Color32::from_rgb(71, 85, 105)
            }));
        } else {
            ui.label(RichText::new("Ready").size(11.0).color(if dark_mode {
                Color32::from_rgb(203, 213, 225)
            } else {
                Color32::from_rgb(71, 85, 105)
            }));
        }

        // Right-aligned Footer Credit matching reference screenshot
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Command Palette shortcut button
            if ui
                .add(
                    egui::Button::new(
                        RichText::new("Ctrl+K")
                            .size(9.5)
                            .monospace()
                            .color(t.text_muted),
                    )
                    .fill(t.bg_surface)
                    .stroke(egui::Stroke::new(1.0_f32, t.border))
                    .rounding(3.0_f32),
                )
                .on_hover_text("Open Command Palette (Ctrl+K)")
                .clicked()
            {
                on_open_command_palette();
            }

            ui.add_space(4.0);

            if !is_narrow {
                let credit_color = if dark_mode {
                    Color32::from_rgb(148, 163, 184)
                } else {
                    Color32::from_rgb(100, 116, 139)
                };
                let sep_color = if dark_mode {
                    Color32::from_rgb(51, 65, 85)
                } else {
                    Color32::from_rgb(203, 213, 225)
                };

                ui.label(
                    RichText::new("Open Source under GPL-3.0")
                        .size(10.5)
                        .color(credit_color),
                );
                ui.label(RichText::new("|").size(10.5).color(sep_color));

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(3.0, 0.0);
                    ui.label(RichText::new("Made with").size(10.5).color(credit_color));
                    ui.label(
                        RichText::new("♥")
                            .size(10.5)
                            .color(Color32::from_rgb(239, 68, 68)),
                    ); // Red heart
                    ui.label(
                        RichText::new("by Sayantan Mazumdar")
                            .size(10.5)
                            .color(credit_color),
                    );
                });

                ui.label(RichText::new("|").size(10.5).color(sep_color));
                ui.label(
                    RichText::new("DiskLens © 2026")
                        .size(10.5)
                        .color(credit_color),
                );
            } else {
                let credit_color = if dark_mode {
                    Color32::from_rgb(148, 163, 184)
                } else {
                    Color32::from_rgb(100, 116, 139)
                };
                ui.label(
                    RichText::new("Sayantan Mazumdar · GPL-3.0")
                        .size(10.0)
                        .color(credit_color),
                );
            }
        });
    });
}
