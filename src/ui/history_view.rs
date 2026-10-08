use crate::history::{DeltaType, GrowthInsightItem, ScanHistoryItem, StorageHistoryManager};
use crate::models::FileNode;
use crate::theme::{truncate_middle, AppTheme};
use eframe::egui::{self, Color32, Margin, Pos2, RichText, Sense, Stroke, Ui, Vec2};

pub struct HistoryViewState {
    pub selected_range: usize, // 0 = 3m, 1 = 6m, 2 = 1y
    pub hovered_point: Option<(String, String, Pos2)>,
}

impl Default for HistoryViewState {
    fn default() -> Self {
        Self {
            selected_range: 1, // 6m default
            hovered_point: None,
        }
    }
}

pub fn render_history_view(
    ui: &mut Ui,
    root: Option<&FileNode>,
    state: &mut HistoryViewState,
    dark_mode: bool,
) {
    let t = AppTheme::tokens(dark_mode);
    let target_path = root.map(|r| r.path.as_path());
    let history_items = StorageHistoryManager::get_history_items(target_path);
    let (largest_growth, shrinking_areas) = StorageHistoryManager::compute_growth_insights(target_path);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let available_w = ui.available_width().max(320.0);
            let content_w = available_w;

            ui.vertical(|ui| {
                ui.set_max_width(content_w);
                ui.add_space(8.0);

                // 1. Header with Range Selector (Matches StorageHistoryView.tsx)
                ui.horizontal_wrapped(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Storage History & Growth").size(18.0).strong().color(t.text));
                        ui.label(
                            RichText::new("Historical data is recorded locally after each completed scan session.")
                                .size(11.0)
                                .color(t.text_secondary)
                        );
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let ranges = ["1 Year", "6 Months", "3 Months"];
                        for (idx, label) in ranges.iter().enumerate() {
                            let actual_idx = 2 - idx;
                            let is_sel = state.selected_range == actual_idx;
                            if ui.selectable_label(is_sel, *label).clicked() {
                                state.selected_range = actual_idx;
                            }
                        }
                    });
                });

                ui.add_space(10.0);

                // 2. Bento Grid Layout: Hero Chart + Growth Insights Stack
                let is_stacked = content_w <= 840.0;
                let spacing = 10.0_f32;

                if is_stacked {
                    ui.vertical(|ui| {
                        render_chart_card(ui, &history_items, content_w, &t);
                        ui.add_space(spacing);
                        render_growth_insights_card(ui, &largest_growth, &shrinking_areas, content_w, &t);
                    });
                } else {
                    let left_w = ((content_w - spacing) * 0.62).floor();
                    let right_w = (content_w - spacing - left_w).floor();

                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = spacing;
                        render_chart_card(ui, &history_items, left_w, &t);
                        render_growth_insights_card(ui, &largest_growth, &shrinking_areas, right_w, &t);
                    });
                }

                ui.add_space(12.0);

                // 3. Storage Snapshots Delta Table
                egui::Frame::none()
                    .fill(t.bg_elevated)
                    .rounding(6.0_f32)
                    .stroke(egui::Stroke::new(1.0_f32, t.border))
                    .inner_margin(Margin::same(12.0))
                    .show(ui, |ui| {
                        let inner_w = content_w - 24.0;
                        ui.set_width(inner_w);

                        ui.horizontal(|ui| {
                            ui.label(RichText::new("RECENT STORAGE SNAPSHOTS").size(11.0).strong().color(t.text_muted));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(RichText::new(format!("{} recorded sessions", history_items.len())).size(10.5).color(t.text_muted));
                            });
                        });

                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(4.0);

                        if history_items.is_empty() {
                            ui.vertical_centered(|ui| {
                                ui.add_space(20.0);
                                ui.label(RichText::new("No prior scan history recorded yet for this directory.").italics().color(t.text_muted));
                                ui.label(RichText::new("Complete a scan to begin tracking storage deltas and historical usage trends.").size(10.5).color(t.text_muted));
                                ui.add_space(20.0);
                            });
                        } else {
                            // Table Header
                            ui.horizontal(|ui| {
                                ui.add_sized([120.0, 18.0], egui::Label::new(RichText::new("Scan Date / Time").strong().color(t.text_muted)));
                                let path_w = (inner_w - 380.0).max(120.0);
                                ui.add_sized([path_w, 18.0], egui::Label::new(RichText::new("Target Path").strong().color(t.text_muted)));
                                ui.add_sized([100.0, 18.0], egui::Label::new(RichText::new("Total Size").strong().color(t.text_muted)));
                                ui.label(RichText::new("Delta").strong().color(t.text_muted));
                            });

                            ui.separator();

                            for item in &history_items {
                                ui.horizontal(|ui| {
                                    ui.add_sized([120.0, 20.0], egui::Label::new(RichText::new(&item.date).size(11.5).color(t.text)));
                                    let path_w = (inner_w - 380.0).max(120.0);
                                    let trunc_p = truncate_middle(&item.root_path, 34);
                                    ui.add_sized([path_w, 20.0], egui::Label::new(RichText::new(trunc_p).size(11.5).monospace().color(t.text_secondary)));
                                    ui.add_sized([100.0, 20.0], egui::Label::new(RichText::new(&item.total_size_formatted).size(12.0).strong().color(t.text)));

                                    let (delta_col, delta_icon) = match item.delta_type {
                                        DeltaType::Increase => (t.danger, "+"),
                                        DeltaType::Decrease => (t.success, "-"),
                                        DeltaType::Neutral => (t.text_muted, "~"),
                                    };

                                    ui.label(
                                        RichText::new(format!("{} {}", delta_icon, &item.delta_formatted))
                                            .size(11.5)
                                            .monospace()
                                            .strong()
                                            .color(delta_col),
                                    );
                                });
                                ui.add_space(2.0);
                            }
                        }
                    });

                ui.add_space(24.0);
            });
        });
}

fn render_chart_card(
    ui: &mut Ui,
    history_items: &[ScanHistoryItem],
    width: f32,
    t: &crate::theme::ThemeTokens,
) {
    egui::Frame::none()
        .fill(t.bg_elevated)
        .rounding(6.0_f32)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            let inner_w = width - 24.0;
            ui.set_width(inner_w);

            ui.horizontal(|ui| {
                ui.label(RichText::new("Usage Trend Over Time").size(13.0).strong().color(t.text));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new("Area chart").size(10.5).color(t.text_muted));
                });
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            let chart_h = 160.0_f32;
            let (chart_rect, _) = ui.allocate_exact_size(Vec2::new(inner_w, chart_h), Sense::hover());
            ui.painter().rect_filled(chart_rect, 4.0_f32, t.bg_surface);

            if history_items.is_empty() {
                ui.painter().text(
                    chart_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Record at least 2 scans to render storage trend curves",
                    egui::FontId::proportional(11.5),
                    t.text_muted,
                );
            } else {
                let max_size: u64 = history_items.iter().map(|h| h.total_size_bytes).max().unwrap_or(100).max(1);
                let point_count = history_items.len();
                let step_x = if point_count > 1 {
                    (chart_rect.width() - 40.0) / (point_count as f32 - 1.0)
                } else {
                    0.0
                };

                let mut points: Vec<Pos2> = Vec::new();
                for (idx, item) in history_items.iter().rev().enumerate() {
                    let px = chart_rect.min.x + 20.0 + (idx as f32 * step_x);
                    let ratio = (item.total_size_bytes as f32 / max_size as f32).clamp(0.05, 0.95);
                    let py = chart_rect.max.y - 20.0 - (ratio * (chart_h - 40.0));
                    points.push(Pos2::new(px, py));
                }

                // Draw grid lines
                for i in 1..=3 {
                    let gy = chart_rect.max.y - 20.0 - (i as f32 * (chart_h - 40.0) / 4.0);
                    ui.painter().line_segment([Pos2::new(chart_rect.min.x + 10.0, gy), Pos2::new(chart_rect.max.x - 10.0, gy)], Stroke::new(1.0_f32, t.border));
                }

                // Draw trend lines and nodes
                for pair in points.windows(2) {
                    ui.painter().line_segment([pair[0], pair[1]], Stroke::new(2.0_f32, t.accent));
                }

                for (idx, pt) in points.iter().enumerate() {
                    ui.painter().circle_filled(*pt, 4.0, t.accent);
                    ui.painter().circle_stroke(*pt, 4.0, Stroke::new(1.0_f32, Color32::WHITE));

                    if let Some(item) = history_items.iter().rev().nth(idx) {
                        let date_str = item.date.split_whitespace().next().unwrap_or("");
                        ui.painter().text(Pos2::new(pt.x, chart_rect.max.y - 8.0), egui::Align2::CENTER_CENTER, date_str, egui::FontId::proportional(9.0), t.text_muted);
                    }
                }
            }
        });
}

fn render_growth_insights_card(
    ui: &mut Ui,
    largest_growth: &[GrowthInsightItem],
    shrinking_areas: &[GrowthInsightItem],
    width: f32,
    t: &crate::theme::ThemeTokens,
) {
    egui::Frame::none()
        .fill(t.bg_elevated)
        .rounding(6.0_f32)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            let inner_w = width - 24.0;
            ui.set_width(inner_w);

            ui.label(RichText::new("Growth Insights").size(13.0).strong().color(t.text));
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // Largest Growth Section
            ui.label(RichText::new("📈 Expanding Directories").size(11.0).strong().color(t.danger));
            ui.add_space(2.0);

            if largest_growth.is_empty() {
                ui.label(RichText::new("No significant directory growth detected.").size(10.5).color(t.text_muted));
            } else {
                for item in largest_growth {
                    ui.horizontal(|ui| {
                        let trunc_name = truncate_middle(&item.path, 16);
                        ui.label(RichText::new(trunc_name).size(11.0).color(t.text));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(&item.change_formatted).size(11.0).strong().color(t.danger));
                        });
                    });
                }
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            // Shrinking Areas Section
            ui.label(RichText::new("📉 Shrinking Areas").size(11.0).strong().color(t.success));
            ui.add_space(2.0);

            if shrinking_areas.is_empty() {
                ui.label(RichText::new("No shrinking directories recorded.").size(10.5).color(t.text_muted));
            } else {
                for item in shrinking_areas {
                    ui.horizontal(|ui| {
                        let trunc_name = truncate_middle(&item.path, 16);
                        ui.label(RichText::new(trunc_name).size(11.0).color(t.text));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(&item.change_formatted).size(11.0).strong().color(t.success));
                        });
                    });
                }
            }
        });
}
