use crate::app::ActiveTab;
use crate::models::FileNode;
use crate::theme::AppTheme;
use eframe::egui::{self, Color32, CursorIcon, Pos2, Rect, RichText, Sense, Stroke, Ui, Vec2};

/// Renders modern Lucide-style outline vector icons with crisp antialiased strokes.
fn draw_outline_icon(painter: &egui::Painter, center: Pos2, icon_type: &str, color: Color32) {
    let stroke = Stroke::new(1.35_f32, color);
    let cx = center.x;
    let cy = center.y;

    match icon_type {
        "home" => {
            // Roof triangle
            painter.line_segment(
                [
                    Pos2::new(cx - 6.5_f32, cy + 0.5_f32),
                    Pos2::new(cx, cy - 5.5_f32),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(cx, cy - 5.5_f32),
                    Pos2::new(cx + 6.5_f32, cy + 0.5_f32),
                ],
                stroke,
            );
            // Walls
            painter.line_segment(
                [
                    Pos2::new(cx - 5.0_f32, cy),
                    Pos2::new(cx - 5.0_f32, cy + 6.0_f32),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(cx + 5.0_f32, cy),
                    Pos2::new(cx + 5.0_f32, cy + 6.0_f32),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(cx - 5.0_f32, cy + 6.0_f32),
                    Pos2::new(cx + 5.0_f32, cy + 6.0_f32),
                ],
                stroke,
            );
            // Door
            painter.line_segment(
                [
                    Pos2::new(cx - 1.8_f32, cy + 6.0_f32),
                    Pos2::new(cx - 1.8_f32, cy + 2.5_f32),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(cx + 1.8_f32, cy + 6.0_f32),
                    Pos2::new(cx + 1.8_f32, cy + 2.5_f32),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(cx - 1.8_f32, cy + 2.5_f32),
                    Pos2::new(cx + 1.8_f32, cy + 2.5_f32),
                ],
                stroke,
            );
        }
        "overview" => {
            // Baseline
            painter.line_segment(
                [
                    Pos2::new(cx - 6.0_f32, cy + 6.0_f32),
                    Pos2::new(cx + 6.0_f32, cy + 6.0_f32),
                ],
                stroke,
            );
            // Bar 1
            painter.line_segment(
                [
                    Pos2::new(cx - 4.2_f32, cy + 6.0_f32),
                    Pos2::new(cx - 4.2_f32, cy + 1.0_f32),
                ],
                stroke,
            );
            // Bar 2
            painter.line_segment(
                [Pos2::new(cx, cy + 6.0_f32), Pos2::new(cx, cy - 5.5_f32)],
                stroke,
            );
            // Bar 3
            painter.line_segment(
                [
                    Pos2::new(cx + 4.2_f32, cy + 6.0_f32),
                    Pos2::new(cx + 4.2_f32, cy - 2.0_f32),
                ],
                stroke,
            );
        }
        "duplicates" => {
            // Back rectangle
            let back = Rect::from_min_max(
                Pos2::new(cx - 5.5_f32, cy - 5.5_f32),
                Pos2::new(cx + 2.0_f32, cy + 2.0_f32),
            );
            painter.rect_stroke(back, 1.5_f32, stroke);
            // Front rectangle
            let front = Rect::from_min_max(
                Pos2::new(cx - 2.0_f32, cy - 2.0_f32),
                Pos2::new(cx + 5.5_f32, cy + 5.5_f32),
            );
            painter.rect_filled(
                front,
                1.5_f32,
                if color == Color32::WHITE {
                    Color32::from_rgb(37, 99, 235)
                } else {
                    Color32::from_rgb(15, 23, 42)
                },
            );
            painter.rect_stroke(front, 1.5_f32, stroke);
        }
        "trash" => {
            // Lid bar
            painter.line_segment(
                [
                    Pos2::new(cx - 6.0_f32, cy - 3.5_f32),
                    Pos2::new(cx + 6.0_f32, cy - 3.5_f32),
                ],
                stroke,
            );
            // Handle
            painter.line_segment(
                [
                    Pos2::new(cx - 2.0_f32, cy - 3.5_f32),
                    Pos2::new(cx - 2.0_f32, cy - 5.5_f32),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(cx - 2.0_f32, cy - 5.5_f32),
                    Pos2::new(cx + 2.0_f32, cy - 5.5_f32),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(cx + 2.0_f32, cy - 5.5_f32),
                    Pos2::new(cx + 2.0_f32, cy - 3.5_f32),
                ],
                stroke,
            );
            // Bin Body
            painter.line_segment(
                [
                    Pos2::new(cx - 4.5_f32, cy - 3.5_f32),
                    Pos2::new(cx - 3.5_f32, cy + 6.0_f32),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(cx + 4.5_f32, cy - 3.5_f32),
                    Pos2::new(cx + 3.5_f32, cy + 6.0_f32),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(cx - 3.5_f32, cy + 6.0_f32),
                    Pos2::new(cx + 3.5_f32, cy + 6.0_f32),
                ],
                stroke,
            );
            // Ribs
            let thin_stroke = Stroke::new(1.0_f32, color);
            painter.line_segment(
                [
                    Pos2::new(cx - 1.4_f32, cy - 1.0_f32),
                    Pos2::new(cx - 1.4_f32, cy + 3.8_f32),
                ],
                thin_stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(cx + 1.4_f32, cy - 1.0_f32),
                    Pos2::new(cx + 1.4_f32, cy + 3.8_f32),
                ],
                thin_stroke,
            );
        }
        "history" => {
            // Zigzag line pointing up-right
            painter.line_segment(
                [
                    Pos2::new(cx - 6.0_f32, cy + 4.5_f32),
                    Pos2::new(cx - 2.0_f32, cy + 0.5_f32),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(cx - 2.0_f32, cy + 0.5_f32),
                    Pos2::new(cx + 1.5_f32, cy + 3.5_f32),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(cx + 1.5_f32, cy + 3.5_f32),
                    Pos2::new(cx + 6.0_f32, cy - 4.0_f32),
                ],
                stroke,
            );
            // Arrowhead
            painter.line_segment(
                [
                    Pos2::new(cx + 2.5_f32, cy - 4.0_f32),
                    Pos2::new(cx + 6.0_f32, cy - 4.0_f32),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(cx + 6.0_f32, cy - 4.0_f32),
                    Pos2::new(cx + 6.0_f32, cy - 0.5_f32),
                ],
                stroke,
            );
        }
        "volumes" => {
            // Top layer diamond
            painter.line_segment(
                [
                    Pos2::new(cx, cy - 5.0_f32),
                    Pos2::new(cx + 6.0_f32, cy - 2.0_f32),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(cx + 6.0_f32, cy - 2.0_f32),
                    Pos2::new(cx, cy + 1.0_f32),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(cx, cy + 1.0_f32),
                    Pos2::new(cx - 6.0_f32, cy - 2.0_f32),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(cx - 6.0_f32, cy - 2.0_f32),
                    Pos2::new(cx, cy - 5.0_f32),
                ],
                stroke,
            );
            // Bottom layer
            painter.line_segment(
                [
                    Pos2::new(cx - 6.0_f32, cy + 1.5_f32),
                    Pos2::new(cx, cy + 4.5_f32),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(cx, cy + 4.5_f32),
                    Pos2::new(cx + 6.0_f32, cy + 1.5_f32),
                ],
                stroke,
            );
        }
        "health" => {
            // Activity pulse line
            painter.line_segment(
                [
                    Pos2::new(cx - 6.5_f32, cy + 1.0_f32),
                    Pos2::new(cx - 3.5_f32, cy + 1.0_f32),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(cx - 3.5_f32, cy + 1.0_f32),
                    Pos2::new(cx - 1.5_f32, cy - 4.5_f32),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(cx - 1.5_f32, cy - 4.5_f32),
                    Pos2::new(cx + 1.0_f32, cy + 5.0_f32),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(cx + 1.0_f32, cy + 5.0_f32),
                    Pos2::new(cx + 2.5_f32, cy - 1.0_f32),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(cx + 2.5_f32, cy - 1.0_f32),
                    Pos2::new(cx + 4.0_f32, cy + 1.0_f32),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(cx + 4.0_f32, cy + 1.0_f32),
                    Pos2::new(cx + 6.5_f32, cy + 1.0_f32),
                ],
                stroke,
            );
        }
        "logs" => {
            // Page rectangle
            let page = Rect::from_min_max(
                Pos2::new(cx - 4.8_f32, cy - 6.0_f32),
                Pos2::new(cx + 4.8_f32, cy + 6.0_f32),
            );
            painter.rect_stroke(page, 1.5_f32, stroke);
            // Text lines
            let thin_stroke = Stroke::new(1.0_f32, color);
            painter.line_segment(
                [
                    Pos2::new(cx - 2.5_f32, cy - 2.5_f32),
                    Pos2::new(cx + 2.5_f32, cy - 2.5_f32),
                ],
                thin_stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(cx - 2.5_f32, cy + 0.5_f32),
                    Pos2::new(cx + 2.5_f32, cy + 0.5_f32),
                ],
                thin_stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(cx - 2.5_f32, cy + 3.5_f32),
                    Pos2::new(cx + 1.0_f32, cy + 3.5_f32),
                ],
                thin_stroke,
            );
        }
        "about" => {
            // Info circle
            painter.circle_stroke(center, 6.2_f32, stroke);
            // Dot
            painter.circle_filled(Pos2::new(cx, cy - 2.8_f32), 0.9_f32, color);
            // Stem
            painter.line_segment(
                [Pos2::new(cx, cy - 0.5_f32), Pos2::new(cx, cy + 3.5_f32)],
                stroke,
            );
        }
        _ => {}
    }
}

pub fn render_side_nav(
    ui: &mut Ui,
    active_tab: &mut ActiveTab,
    _scanned_root: &Option<Box<FileNode>>,
    _hogs_count: usize,
    dark_mode: bool,
    is_compact: bool,
    is_view_available: &dyn Fn(ActiveTab) -> bool,
) {
    let t = AppTheme::tokens(dark_mode);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.add_space(2.0_f32);

                // 1. Top Brand Header Block (matches reference screenshot)
                let brand_h = if is_compact { 38.0_f32 } else { 44.0_f32 };
                let (brand_rect, brand_resp) = ui
                    .allocate_exact_size(Vec2::new(ui.available_width(), brand_h), Sense::click());

                if brand_resp.hovered() {
                    ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
                }

                if brand_resp.clicked() {
                    *active_tab = ActiveTab::Storage;
                }

                // Brand icon squircle
                let sq_size = if is_compact { 28.0_f32 } else { 34.0_f32 };
                let sq_rect = Rect::from_min_size(
                    Pos2::new(
                        brand_rect.min.x
                            + (if is_compact {
                                (brand_rect.width() - sq_size) / 2.0_f32
                            } else {
                                2.0_f32
                            }),
                        brand_rect.min.y + (brand_h - sq_size) / 2.0_f32,
                    ),
                    Vec2::new(sq_size, sq_size),
                );
                let brand_blue = Color32::from_rgb(37, 99, 235); // #2563EB
                ui.painter().rect_filled(sq_rect, 7.0_f32, brand_blue);

                // Drive icon inside squircle
                let bc = sq_rect.center();
                let drive_rect =
                    Rect::from_center_size(bc, Vec2::new(sq_size * 0.58_f32, sq_size * 0.54_f32));
                ui.painter()
                    .rect_stroke(drive_rect, 2.0_f32, Stroke::new(1.3_f32, Color32::WHITE));
                // Disc platter circle
                ui.painter().circle_stroke(
                    Pos2::new(bc.x, bc.y - 1.0_f32),
                    sq_size * 0.16_f32,
                    Stroke::new(1.1_f32, Color32::WHITE),
                );
                // Drive LED / slot dot
                ui.painter().circle_filled(
                    Pos2::new(drive_rect.max.x - 3.0_f32, drive_rect.max.y - 3.0_f32),
                    0.9_f32,
                    Color32::WHITE,
                );

                if !is_compact {
                    let text_left = sq_rect.max.x + 8.0_f32;
                    ui.painter().text(
                        Pos2::new(text_left, brand_rect.min.y + 7.0_f32),
                        egui::Align2::LEFT_TOP,
                        "DiskLens",
                        egui::FontId::proportional(14.0_f32),
                        if dark_mode {
                            Color32::WHITE
                        } else {
                            Color32::from_rgb(15, 23, 42)
                        },
                    );

                    ui.painter().text(
                        Pos2::new(text_left, brand_rect.min.y + 24.0_f32),
                        egui::Align2::LEFT_TOP,
                        "Disk Space Analyzer",
                        egui::FontId::proportional(9.5_f32),
                        Color32::from_rgb(148, 163, 184),
                    );
                }

                ui.add_space(10.0_f32);

                // 2. Navigation Groups: EXPLORE, SYSTEM, SETTINGS
                struct NavEntry {
                    tab: ActiveTab,
                    icon_id: &'static str,
                    label: &'static str,
                    shortcut: &'static str,
                }

                let groups: [(&str, &[NavEntry]); 2] = [
                    (
                        "EXPLORE",
                        &[
                            NavEntry {
                                tab: ActiveTab::Storage,
                                icon_id: "home",
                                label: "Home",
                                shortcut: "",
                            },
                            NavEntry {
                                tab: ActiveTab::Overview,
                                icon_id: "overview",
                                label: "Overview",
                                shortcut: "1",
                            },
                            NavEntry {
                                tab: ActiveTab::Duplicates,
                                icon_id: "duplicates",
                                label: "Duplicates",
                                shortcut: "2",
                            },
                            NavEntry {
                                tab: ActiveTab::SafeTrash,
                                icon_id: "trash",
                                label: "Trash",
                                shortcut: "3",
                            },
                        ],
                    ),
                    (
                        "SYSTEM",
                        &[
                            NavEntry {
                                tab: ActiveTab::History,
                                icon_id: "history",
                                label: "History",
                                shortcut: "4",
                            },
                            NavEntry {
                                tab: ActiveTab::MountPoints,
                                icon_id: "volumes",
                                label: "Volumes",
                                shortcut: "M",
                            },
                            NavEntry {
                                tab: ActiveTab::DiagnosticsSmart,
                                icon_id: "health",
                                label: "Drive Health",
                                shortcut: "H",
                            },
                            NavEntry {
                                tab: ActiveTab::SystemLogs,
                                icon_id: "logs",
                                label: "Logs",
                                shortcut: "L",
                            },
                        ],
                    ),
                ];

                for (group_name, items) in groups {
                    if !is_compact {
                        ui.add_space(6.0_f32);
                        ui.label(
                            RichText::new(group_name)
                                .size(9.0_f32)
                                .strong()
                                .color(Color32::from_rgb(100, 116, 139)),
                        );
                        ui.add_space(3.0_f32);
                    } else {
                        ui.add_space(4.0_f32);
                    }

                    for entry in items {
                        let is_available = is_view_available(entry.tab);
                        let is_selected = if entry.tab == ActiveTab::Overview {
                            matches!(
                                *active_tab,
                                ActiveTab::Overview
                                    | ActiveTab::DirectoryTree
                                    | ActiveTab::Treemap
                                    | ActiveTab::LargestFiles
                                    | ActiveTab::Categories
                            )
                        } else {
                            *active_tab == entry.tab
                        };

                        let item_h = 32.0_f32;
                        let (item_rect, response) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), item_h),
                            if is_available {
                                Sense::click()
                            } else {
                                Sense::hover()
                            },
                        );

                        if is_available {
                            let is_hovered = response.hovered();
                            if is_hovered {
                                ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
                            }

                            // Active pill style: vibrant blue capsule matching reference screenshot
                            let active_pill_color = Color32::from_rgb(37, 99, 235); // #2563EB

                            let (bg, text_color, icon_color) = if is_selected {
                                (active_pill_color, Color32::WHITE, Color32::WHITE)
                            } else if is_hovered {
                                if dark_mode {
                                    (Color32::from_rgb(26, 33, 50), Color32::WHITE, t.accent)
                                } else {
                                    (
                                        Color32::from_rgb(238, 242, 255),
                                        Color32::from_rgb(15, 23, 42),
                                        active_pill_color,
                                    )
                                }
                            } else {
                                (
                                    Color32::TRANSPARENT,
                                    if dark_mode {
                                        Color32::from_rgb(203, 213, 225)
                                    } else {
                                        Color32::from_rgb(71, 85, 105)
                                    },
                                    if dark_mode {
                                        Color32::from_rgb(148, 163, 184)
                                    } else {
                                        Color32::from_rgb(100, 116, 139)
                                    },
                                )
                            };

                            if bg != Color32::TRANSPARENT {
                                ui.painter().rect_filled(item_rect, 6.0_f32, bg);
                            }

                            let icon_center = if is_compact {
                                item_rect.center()
                            } else {
                                Pos2::new(item_rect.min.x + 15.0_f32, item_rect.center().y)
                            };

                            draw_outline_icon(ui.painter(), icon_center, entry.icon_id, icon_color);

                            if !is_compact {
                                let label_x = item_rect.min.x + 32.0_f32;
                                ui.painter().text(
                                    Pos2::new(label_x, item_rect.center().y),
                                    egui::Align2::LEFT_CENTER,
                                    entry.label,
                                    egui::FontId::proportional(12.5_f32),
                                    text_color,
                                );

                                if !entry.shortcut.is_empty() && !is_selected {
                                    ui.painter().text(
                                        Pos2::new(item_rect.max.x - 6.0_f32, item_rect.center().y),
                                        egui::Align2::RIGHT_CENTER,
                                        entry.shortcut,
                                        egui::FontId::monospace(9.0_f32),
                                        Color32::from_rgb(100, 116, 139),
                                    );
                                }
                            }

                            if response.clicked() {
                                *active_tab = entry.tab;
                            }
                        } else {
                            // Disabled state
                            let _ = response
                                .on_hover_cursor(CursorIcon::NotAllowed)
                                .on_hover_text("Select a folder or scan a storage target first");

                            let disabled_color =
                                Color32::from_rgba_unmultiplied(100, 116, 139, 100);

                            let icon_center = if is_compact {
                                item_rect.center()
                            } else {
                                Pos2::new(item_rect.min.x + 15.0_f32, item_rect.center().y)
                            };

                            draw_outline_icon(
                                ui.painter(),
                                icon_center,
                                entry.icon_id,
                                disabled_color,
                            );

                            if !is_compact {
                                let label_x = item_rect.min.x + 32.0_f32;
                                ui.painter().text(
                                    Pos2::new(label_x, item_rect.center().y),
                                    egui::Align2::LEFT_CENTER,
                                    entry.label,
                                    egui::FontId::proportional(12.5_f32),
                                    disabled_color,
                                );
                            }
                        }

                        ui.add_space(1.0_f32);
                    }
                }
            });
        });
}
