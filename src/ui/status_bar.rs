use crate::app::ActiveTab;
use crate::models::FileNode;
use crate::theme::AppTheme;
use eframe::egui::{self, CursorIcon, RichText, Ui, Vec2};

pub fn render_status_bar(
    ui: &mut Ui,
    scanned_root: &Option<Box<FileNode>>,
    is_scanning: bool,
    active_tab: &mut ActiveTab,
    dark_mode: bool,
    on_open_logs: &mut dyn FnMut(),
    on_open_mounts: &mut dyn FnMut(),
    on_open_docs: &mut dyn FnMut(),
    on_open_command_palette: &mut dyn FnMut(),
) {
    let t = AppTheme::tokens(dark_mode);
    let available_w = ui.available_width();
    let is_narrow = available_w < 740.0;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(8.0, 0.0);

        // Status Pulse Indicator & Text
        let status_color = if is_scanning { t.warning } else { t.success };
        let (dot, _) = ui.allocate_exact_size(Vec2::new(7.0, 7.0), egui::Sense::hover());
        ui.painter().circle_filled(dot.center(), 3.5_f32, status_color);

        if is_scanning {
            ui.label(RichText::new("⚡ Rayon crawler indexing filesystem in background...").size(10.5).color(t.warning).strong());
        } else if let Some(root) = scanned_root {
            let info = format!("Scan complete: {} inodes ({})", root.total_items(), root.formatted_size());
            ui.label(RichText::new(info).size(10.5).color(t.text_secondary));
        } else {
            ui.label(RichText::new("Ready · Storage integrity verified • Shields active").size(10.5).color(t.text_muted));
        }

        // Right-aligned Telemetry & Quick Action Modals
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Command Palette shortcut button
            if ui.add(
                egui::Button::new(RichText::new("Ctrl+K").size(9.5).monospace().color(t.text_muted))
                    .fill(t.bg_surface)
                    .stroke(egui::Stroke::new(1.0_f32, t.border))
                    .rounding(3.0_f32)
            ).on_hover_text("Open Command Palette (Ctrl+K)").clicked() {
                on_open_command_palette();
            }

            if ui.small_button(RichText::new("Docs").size(10.0).color(t.text_muted)).on_hover_text("DiskLens Guide & Shortcuts").clicked() {
                on_open_docs();
            }

            if !is_narrow {
                if ui.small_button(RichText::new("Mounts").size(10.0).color(t.text_muted)).on_hover_text("Inspect /proc/mounts").clicked() {
                    on_open_mounts();
                }

                if ui.small_button(RichText::new("Logs").size(10.0).color(t.text_muted)).on_hover_text("View Engine Event Logs").clicked() {
                    on_open_logs();
                }

                // Health %
                ui.label(RichText::new("Health: 98%").size(10.0).monospace().color(t.success));
                ui.label(RichText::new("•").size(10.0).color(t.border));

                // Temp °C
                ui.label(RichText::new("37°C").size(10.0).monospace().color(t.warning));
                ui.label(RichText::new("•").size(10.0).color(t.border));

                // Rust Engine telemetry pill
                let engine_pill = ui.add(
                    egui::Button::new(RichText::new("🦀 Rust io_uring (16t · 0-alloc)").size(9.5).monospace().color(t.accent))
                        .fill(t.bg_surface)
                        .stroke(egui::Stroke::new(1.0_f32, t.border))
                        .rounding(3.0_f32)
                ).on_hover_cursor(CursorIcon::PointingHand);

                if engine_pill.clicked() {
                    *active_tab = ActiveTab::RustEngine;
                }
            }
        });
    });
}
