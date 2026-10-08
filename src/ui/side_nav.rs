use crate::app::ActiveTab;
use crate::models::FileNode;
use crate::theme::{truncate_middle, AppTheme};
use eframe::egui::{self, Color32, CursorIcon, Margin, RichText, Sense, Ui, Vec2};

pub fn render_side_nav(
    ui: &mut Ui,
    active_tab: &mut ActiveTab,
    scanned_root: &Option<Box<FileNode>>,
    hogs_count: usize,
    dark_mode: bool,
    is_compact: bool,
    is_view_available: &dyn Fn(ActiveTab) -> bool,
) {
    let t = AppTheme::tokens(dark_mode);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.vertical(|ui| {
                // 1. Active Target Widget (Clickable to switch storage target)
                if let Some(root) = scanned_root {
                    let (widget_rect, response) = ui.allocate_exact_size(
                        Vec2::new(ui.available_width(), if is_compact { 38.0 } else { 50.0 }),
                        Sense::click(),
                    );

                    let loc_name = root.path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    let display_name = if loc_name.is_empty() { root.path.display().to_string() } else { loc_name };

                    let response = response
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .on_hover_text(format!("Target: {}\nSize: {}\nClick to choose another drive or location", root.path.display(), root.formatted_size()));

                    let is_hovered = response.hovered();
                    let bg = if is_hovered { t.bg_elevated } else { t.bg_container };
                    ui.painter().rect_filled(widget_rect, 6.0_f32, bg);
                    ui.painter().rect_stroke(widget_rect, 6.0_f32, egui::Stroke::new(1.0_f32, if is_hovered { t.accent } else { t.border }));

                    if is_compact {
                        ui.painter().text(
                            widget_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "💾",
                            egui::FontId::proportional(16.0),
                            t.accent,
                        );
                    } else {
                        let text_x = widget_rect.min.x + 10.0;

                        ui.painter().text(
                            egui::pos2(text_x, widget_rect.min.y + 8.0),
                            egui::Align2::LEFT_TOP,
                            truncate_middle(&display_name, 16),
                            egui::FontId::proportional(12.5),
                            t.text,
                        );

                        let meta_str = format!("{} · {} items", root.formatted_size(), root.total_items());
                        ui.painter().text(
                            egui::pos2(text_x, widget_rect.min.y + 26.0),
                            egui::Align2::LEFT_TOP,
                            meta_str,
                            egui::FontId::proportional(10.0),
                            t.text_muted,
                        );
                    }

                    if response.clicked() {
                        *active_tab = ActiveTab::Storage;
                    }

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(4.0);
                }

                // 2. Navigation Sections (Consolidated Architecture)
                let sections: [(&str, &[(ActiveTab, &str, &str, &str, Option<String>)]); 2] = [
                    (
                        "ANALYSIS",
                        &[
                            (ActiveTab::Storage, "🏠", "Storage Home", "", None),
                            (ActiveTab::Overview, "📊", "Overview", "1", None),
                            (ActiveTab::DirectoryTree, "🔍", "Explorer", "2", if hogs_count > 0 { Some(format!("{}", hogs_count.min(99))) } else { None }),
                            (ActiveTab::Duplicates, "📋", "Duplicates", "3", None),
                            (ActiveTab::SafeTrash, "🗑", "Safe Trash", "C", None),
                        ],
                    ),
                    (
                        "MAINTENANCE & HARDWARE",
                        &[
                            (ActiveTab::History, "📈", "History & Growth", "4", None),
                            (ActiveTab::MountPoints, "🖴", "Mounts & Subvolumes", "M", None),
                            (ActiveTab::DiagnosticsSmart, "🩺", "S.M.A.R.T. Health", "H", Some("98%".to_string())),
                            (ActiveTab::SystemLogs, "📜", "Event Logs", "L", None),
                            (ActiveTab::Documentation, "📖", "User Manual", "D", None),
                        ],
                    ),
                ];

                for (section_title, items) in sections {
                    if !is_compact {
                        ui.add_space(4.0);
                        ui.label(RichText::new(section_title).size(9.0).monospace().strong().color(t.text_muted));
                        ui.add_space(2.0);
                    }

                    for (tab, icon, label, shortcut, badge) in items {
                        let is_available = is_view_available(*tab);
                        let is_selected = if *tab == ActiveTab::DirectoryTree {
                            matches!(*active_tab, ActiveTab::DirectoryTree | ActiveTab::Treemap | ActiveTab::LargestFiles | ActiveTab::Categories)
                        } else {
                            *active_tab == *tab
                        };
                        let item_h = 30.0;

                        if is_available {
                            // Enabled state: fully interactive
                            let (item_rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), item_h), Sense::click());
                            let response = response
                                .on_hover_cursor(CursorIcon::PointingHand)
                                .on_hover_text(format!("{} (Shortcut: {})", label, shortcut));

                            let is_hovered = response.hovered();

                            let bg = if is_selected {
                                Color32::from_rgb(62, 69, 101)
                            } else if is_hovered {
                                Color32::from_rgb(36, 40, 59)
                            } else {
                                Color32::TRANSPARENT
                            };

                            ui.painter().rect_filled(item_rect, 4.0_f32, bg);
                            if is_selected {
                                ui.painter().rect_stroke(item_rect, 4.0_f32, egui::Stroke::new(1.0_f32, t.accent));
                            }

                            if is_compact {
                                let icon_color = if is_selected { t.accent } else { t.text };
                                ui.painter().text(
                                    item_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    *icon,
                                    egui::FontId::proportional(15.0),
                                    icon_color,
                                );
                            } else {
                                let text_x = item_rect.min.x + 10.0;
                                let text_color = if is_selected { Color32::WHITE } else { t.text };

                                ui.painter().text(
                                    egui::pos2(text_x, item_rect.center().y),
                                    egui::Align2::LEFT_CENTER,
                                    format!("{}  {}", icon, label),
                                    egui::FontId::proportional(11.5),
                                    text_color,
                                );

                                if let Some(b_text) = badge {
                                    let badge_x = item_rect.max.x - 30.0;
                                    let badge_rect = egui::Rect::from_center_size(
                                        egui::pos2(badge_x, item_rect.center().y),
                                        Vec2::new(20.0, 14.0),
                                    );
                                    ui.painter().rect_filled(badge_rect, 3.0_f32, Color32::from_rgba_premultiplied(247, 118, 142, 40));
                                    ui.painter().text(
                                        badge_rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        b_text,
                                        egui::FontId::proportional(9.5),
                                        t.danger,
                                    );
                                } else if !shortcut.is_empty() {
                                    ui.painter().text(
                                        egui::pos2(item_rect.max.x - 8.0, item_rect.center().y),
                                        egui::Align2::RIGHT_CENTER,
                                        *shortcut,
                                        egui::FontId::monospace(9.5),
                                        t.text_muted,
                                    );
                                }
                            }

                            if response.clicked() {
                                *active_tab = *tab;
                            }
                        } else {
                            // Disabled state: non-interactive, muted opacity, no hover glow, not allowed cursor
                            let (item_rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), item_h), Sense::hover());
                            let _ = response
                                .on_hover_cursor(CursorIcon::NotAllowed)
                                .on_hover_text("Select a folder or scan a storage target first");

                            // Subtle disabled colors
                            let disabled_text_color = Color32::from_rgba_unmultiplied(120, 125, 145, 120);
                            let disabled_shortcut_color = Color32::from_rgba_unmultiplied(90, 95, 115, 90);

                            if is_compact {
                                ui.painter().text(
                                    item_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    *icon,
                                    egui::FontId::proportional(15.0),
                                    disabled_text_color,
                                );
                            } else {
                                let text_x = item_rect.min.x + 10.0;

                                ui.painter().text(
                                    egui::pos2(text_x, item_rect.center().y),
                                    egui::Align2::LEFT_CENTER,
                                    format!("{}  {}", icon, label),
                                    egui::FontId::proportional(11.5),
                                    disabled_text_color,
                                );

                                if !shortcut.is_empty() {
                                    ui.painter().text(
                                        egui::pos2(item_rect.max.x - 8.0, item_rect.center().y),
                                        egui::Align2::RIGHT_CENTER,
                                        *shortcut,
                                        egui::FontId::monospace(9.5),
                                        disabled_shortcut_color,
                                    );
                                }
                            }
                            // Mouse clicks on disabled items are strictly ignored and do not modify *active_tab
                        }

                        ui.add_space(1.0);
                    }
                }

                // Bottom Engine Status Footer
                if !is_compact {
                    ui.add_space(8.0);
                    egui::Frame::none()
                        .fill(t.bg_surface)
                        .rounding(4.0_f32)
                        .stroke(egui::Stroke::new(1.0_f32, t.border))
                        .inner_margin(Margin::same(6.0))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Engine:").size(9.5).color(t.text_muted));
                                ui.label(RichText::new("Rust / egui").size(9.5).strong().color(t.text));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new("60 FPS").size(9.5).strong().color(t.success));
                                });
                            });
                        });
                }
            });
        });
}
