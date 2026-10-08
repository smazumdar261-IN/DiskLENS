use crate::history::{DeltaType, GrowthInsightItem, ScanHistoryItem, StorageHistoryManager};
use crate::models::FileNode;
use crate::theme::{truncate_middle, AppTheme};
use eframe::egui::{self, Color32, Margin, Pos2, RichText, Sense, Stroke, Ui, Vec2};
use egui_extras::{Column, TableBuilder};

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
            let total_avail = ui.available_width().max(320.0);

            ui.vertical(|ui| {
                ui.set_max_width(total_avail);
                ui.add_space(6.0);

                // 1. Header with Range Selector
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Storage History & Growth").size(18.0).strong().color(t.text));
                        ui.label(
                            RichText::new("Historical data is recorded locally after each completed scan session.")
                                .size(11.0)
                                .color(t.text_secondary),
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
                        ui.label(RichText::new("Range:").size(11.0).color(t.text_muted));
                    });
                });

                ui.add_space(10.0);

                // 2. Responsive Time-Series Chart (Stretches to 100% available width)
                render_responsive_chart_card(ui, &history_items, state, total_avail, &t);

                ui.add_space(10.0);

                // 3. Growth Insights Section (Responsive Dynamic Flex Columns)
                render_growth_insights_section(ui, &largest_growth, &shrinking_areas, total_avail, &t);

                ui.add_space(10.0);

                // 4. Snapshot History Table in a Scrollable Container Directly Below Chart
                render_snapshot_history_table(ui, &history_items, total_avail, &t);

                ui.add_space(20.0);
            });
        });
}

/// Time-series trend chart stretching to full container width with decimated X-axis labels
fn render_responsive_chart_card(
    ui: &mut Ui,
    history_items: &[ScanHistoryItem],
    state: &mut HistoryViewState,
    avail_w: f32,
    t: &crate::theme::ThemeTokens,
) {
    egui::Frame::none()
        .fill(t.bg_elevated)
        .rounding(6.0_f32)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            let inner_w = (avail_w - 24.0).max(100.0);
            ui.set_width(inner_w);

            ui.horizontal(|ui| {
                ui.label(RichText::new("📈 Usage Trend Over Time").size(12.5).strong().color(t.text));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{} recorded sessions", history_items.len()))
                            .size(10.5)
                            .monospace()
                            .color(t.text_muted),
                    );
                });
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            let chart_h = 175.0_f32;
            let (chart_rect, response) = ui.allocate_exact_size(Vec2::new(inner_w, chart_h), Sense::hover());
            ui.painter().rect_filled(chart_rect, 4.0_f32, t.bg_surface);
            ui.painter().rect_stroke(chart_rect, 4.0_f32, Stroke::new(1.0_f32, t.border));

            if history_items.is_empty() {
                ui.painter().text(
                    chart_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Record at least 2 scans to render storage trend curves",
                    egui::FontId::proportional(11.5),
                    t.text_muted,
                );
                return;
            }

            let max_size: u64 = history_items.iter().map(|h| h.total_size_bytes).max().unwrap_or(100).max(1);
            let point_count = history_items.len();
            let padding_x = 35.0_f32;
            let usable_w = (chart_rect.width() - (padding_x * 2.0)).max(10.0);
            let step_x = if point_count > 1 {
                usable_w / (point_count as f32 - 1.0)
            } else {
                0.0
            };

            let mut points: Vec<Pos2> = Vec::new();
            for (idx, item) in history_items.iter().rev().enumerate() {
                let px = chart_rect.min.x + padding_x + (idx as f32 * step_x);
                let ratio = (item.total_size_bytes as f32 / max_size as f32).clamp(0.06, 0.92);
                let py = chart_rect.max.y - 28.0 - (ratio * (chart_h - 52.0));
                points.push(Pos2::new(px, py));
            }

            // Draw horizontal grid lines & size markers
            for i in 1..=3 {
                let gy = chart_rect.max.y - 28.0 - (i as f32 * (chart_h - 52.0) / 4.0);
                ui.painter().line_segment(
                    [Pos2::new(chart_rect.min.x + 8.0, gy), Pos2::new(chart_rect.max.x - 8.0, gy)],
                    Stroke::new(1.0_f32, t.border.gamma_multiply(0.6)),
                );
            }

            // Draw area gradient/polygon below trend line
            if points.len() >= 2 {
                for pair in points.windows(2) {
                    ui.painter().line_segment([pair[0], pair[1]], Stroke::new(2.5_f32, t.accent));
                }
            }

            // Decimate X-axis date labels to strictly prevent text overlaps across any window width!
            // Each date label needs at least 70px to avoid collisions.
            let max_visible_labels = (usable_w / 70.0).floor().max(2.0) as usize;
            let stride = if point_count > max_visible_labels {
                ((point_count - 1) as f32 / (max_visible_labels - 1) as f32).ceil().max(1.0) as usize
            } else {
                1
            };

            let mouse_pos = response.hover_pos();
            let mut hovered_item_info = None;

            for (idx, pt) in points.iter().enumerate() {
                let is_hovered = mouse_pos.map_or(false, |mp| mp.distance(*pt) < 14.0);

                let dot_rad = if is_hovered { 6.0 } else { 3.5 };
                let dot_col = if is_hovered { t.cyan } else { t.accent };
                ui.painter().circle_filled(*pt, dot_rad, dot_col);
                ui.painter().circle_stroke(*pt, dot_rad, Stroke::new(1.0_f32, Color32::WHITE));

                if let Some(item) = history_items.iter().rev().nth(idx) {
                    if is_hovered {
                        hovered_item_info = Some((item.date.clone(), item.total_size_formatted.clone(), *pt));
                    }

                    // Decimated date labeling: render if aligned with stride or last point
                    let should_render_label = idx % stride == 0 || idx == point_count - 1;
                    if should_render_label {
                        let date_str = item.date.split_whitespace().next().unwrap_or("");
                        ui.painter().text(
                            Pos2::new(pt.x, chart_rect.max.y - 12.0),
                            egui::Align2::CENTER_CENTER,
                            date_str,
                            egui::FontId::monospace(9.5),
                            t.text_muted,
                        );
                    }
                }
            }

            // Interactive hover tooltip card
            if let Some((date, size, pt)) = hovered_item_info {
                state.hovered_point = Some((date.clone(), size.clone(), pt));
                let tooltip_text = format!("{} • {}", date, size);
                let tooltip_pos = Pos2::new(pt.x.clamp(chart_rect.min.x + 60.0, chart_rect.max.x - 60.0), pt.y - 16.0);
                ui.painter().text(
                    tooltip_pos,
                    egui::Align2::CENTER_BOTTOM,
                    tooltip_text,
                    egui::FontId::monospace(10.5),
                    Color32::WHITE,
                );
            }
        });
}

/// Growth insights cards displayed responsively (2-col on wide, 1-col on narrow)
fn render_growth_insights_section(
    ui: &mut Ui,
    largest_growth: &[GrowthInsightItem],
    shrinking_areas: &[GrowthInsightItem],
    avail_w: f32,
    t: &crate::theme::ThemeTokens,
) {
    let is_wide = avail_w >= 800.0;
    let spacing = 10.0_f32;
    let col_w = if is_wide {
        ((avail_w - spacing) / 2.0).floor()
    } else {
        avail_w
    };

    if is_wide {
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = spacing;
            render_insight_tile(ui, "📈 Expanding Directories", largest_growth, t.danger, col_w, t);
            render_insight_tile(ui, "📉 Shrinking Areas", shrinking_areas, t.success, col_w, t);
        });
    } else {
        ui.vertical(|ui| {
            render_insight_tile(ui, "📈 Expanding Directories", largest_growth, t.danger, col_w, t);
            ui.add_space(8.0);
            render_insight_tile(ui, "📉 Shrinking Areas", shrinking_areas, t.success, col_w, t);
        });
    }
}

fn render_insight_tile(
    ui: &mut Ui,
    title: &str,
    items: &[GrowthInsightItem],
    badge_color: Color32,
    width: f32,
    t: &crate::theme::ThemeTokens,
) {
    egui::Frame::none()
        .fill(t.bg_elevated)
        .rounding(6.0_f32)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            let inner_w = (width - 24.0).max(60.0);
            ui.set_width(inner_w);

            ui.horizontal(|ui| {
                ui.label(RichText::new(title).size(12.0).strong().color(badge_color));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("{} areas", items.len())).size(10.0).monospace().color(t.text_muted));
                });
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            if items.is_empty() {
                ui.label(RichText::new("No significant directory changes recorded in this range.").size(10.5).color(t.text_muted));
            } else {
                for item in items.iter().take(5) {
                    ui.horizontal(|ui| {
                        let name_avail = (inner_w - 90.0).max(40.0);
                        let max_c = (name_avail / 7.5) as usize;
                        let trunc_name = truncate_middle(&item.path, max_c.clamp(10, 36));
                        ui.add(
                            egui::Label::new(RichText::new(trunc_name).size(11.0).color(t.text))
                                .truncate(),
                        ).on_hover_text(&item.path);

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(&item.change_formatted).size(11.0).monospace().strong().color(badge_color));
                        });
                    });
                    ui.add_space(2.0);
                }
            }
        });
}

/// Snapshot History Table rendered directly below the chart using TableBuilder
fn render_snapshot_history_table(
    ui: &mut Ui,
    history_items: &[ScanHistoryItem],
    avail_w: f32,
    t: &crate::theme::ThemeTokens,
) {
    egui::Frame::none()
        .fill(t.bg_elevated)
        .rounding(6.0_f32)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            let inner_w = (avail_w - 24.0).max(100.0);
            ui.set_width(inner_w);

            ui.horizontal(|ui| {
                ui.label(RichText::new("🗂 Recent Storage Snapshots").size(12.5).strong().color(t.text));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new("Chronological Scan Index")
                            .size(10.5)
                            .color(t.text_muted),
                    );
                });
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(4.0);

            if history_items.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(18.0);
                    ui.label(RichText::new("No prior scan history recorded yet for this directory.").italics().color(t.text_muted));
                    ui.label(RichText::new("Complete a scan to begin tracking storage deltas and historical usage trends.").size(10.5).color(t.text_muted));
                    ui.add_space(18.0);
                });
            } else {
                // TableBuilder with proportional resizable columns
                TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(Column::initial(135.0).at_least(100.0).resizable(true))
                    .column(Column::remainder().at_least(140.0))
                    .column(Column::initial(110.0).at_least(85.0).resizable(true))
                    .column(Column::initial(115.0).at_least(85.0).resizable(true))
                    .header(24.0, |mut header| {
                        header.col(|ui| {
                            ui.label(RichText::new("Scan Date / Time").strong().color(t.text_muted));
                        });
                        header.col(|ui| {
                            ui.label(RichText::new("Target Path").strong().color(t.text_muted));
                        });
                        header.col(|ui| {
                            ui.label(RichText::new("Total Size").strong().color(t.text_muted));
                        });
                        header.col(|ui| {
                            ui.label(RichText::new("Delta").strong().color(t.text_muted));
                        });
                    })
                    .body(|mut body| {
                        body.rows(26.0, history_items.len(), |mut row| {
                            let idx = row.index();
                            if let Some(item) = history_items.get(idx) {
                                row.col(|ui| {
                                    ui.label(RichText::new(&item.date).size(11.0).monospace().color(t.text));
                                });
                                row.col(|ui| {
                                    let full_p = &item.root_path;
                                    ui.add(
                                        egui::Label::new(RichText::new(full_p).size(11.0).monospace().color(t.text_secondary))
                                            .truncate(),
                                    ).on_hover_text(full_p);
                                });
                                row.col(|ui| {
                                    ui.label(RichText::new(&item.total_size_formatted).size(11.5).monospace().strong().color(t.text));
                                });
                                row.col(|ui| {
                                    let (delta_col, delta_icon) = match item.delta_type {
                                        DeltaType::Increase => (t.danger, "+"),
                                        DeltaType::Decrease => (t.success, "-"),
                                        DeltaType::Neutral => (t.text_muted, "~"),
                                    };
                                    ui.label(
                                        RichText::new(format!("{} {}", delta_icon, &item.delta_formatted))
                                            .size(11.0)
                                            .monospace()
                                            .strong()
                                            .color(delta_col),
                                    );
                                });
                            }
                        });
                    });
            }
        });
}
