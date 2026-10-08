use crate::models::{format_size, AnalysisCache, ExtensionSummary, FileCategory};
use crate::theme::AppTheme;
use eframe::egui::{self, Color32, Margin, Rect, RichText, Sense, Ui, Vec2};

pub struct CategoriesViewState {
    pub search_ext: String,
    pub sort_by_size: bool,
    pub selected_category: Option<FileCategory>,
}

impl Default for CategoriesViewState {
    fn default() -> Self {
        Self {
            search_ext: String::new(),
            sort_by_size: true,
            selected_category: None,
        }
    }
}

/// Densified File Types & Extension Analysis View.
/// Replaces the bulky 3x3 category grid with a single compact horizontal chip bar (~32px)
/// and standardizes the extension table to 26-28px rows with monospace telemetry.
pub fn render_categories_view(
    ui: &mut Ui,
    cache: &AnalysisCache,
    state: &mut CategoriesViewState,
    dark_mode: bool,
) {
    let t = AppTheme::tokens(dark_mode);
    let cat_summaries = &cache.category_summaries;
    let total_tree_size = cache.total_apparent_size.max(1);

    // Filter extension summaries
    let mut filtered_exts: Vec<&ExtensionSummary> = cache
        .extension_summaries
        .iter()
        .filter(|e| {
            if let Some(cat) = state.selected_category {
                if e.category != cat {
                    return false;
                }
            }
            if !state.search_ext.is_empty() {
                let q = state.search_ext.to_lowercase().trim_start_matches('.').to_string();
                if !e.extension.to_lowercase().contains(&q) {
                    return false;
                }
            }
            true
        })
        .collect();

    if state.sort_by_size {
        filtered_exts.sort_by(|a, b| b.total_size.cmp(&a.total_size));
    } else {
        filtered_exts.sort_by(|a, b| b.count.cmp(&a.count));
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let total_avail = ui.available_width().max(340.0);

            ui.vertical(|ui| {
                ui.set_max_width(total_avail);
                ui.add_space(4.0);

                // 1. Compact Horizontal Category Filter Chip Bar (Single Row, ~32px)
                egui::Frame::none()
                    .fill(t.bg_elevated)
                    .rounding(6.0_f32)
                    .stroke(egui::Stroke::new(1.0_f32, t.border))
                    .inner_margin(Margin::symmetric(8.0, 6.0))
                    .show(ui, |ui| {
                        ui.set_width((total_avail - 16.0).max(60.0));
                        egui::ScrollArea::horizontal()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 6.0;

                                    // "All Categories" Chip
                                    let is_all_sel = state.selected_category.is_none();
                                    let all_bg = if is_all_sel { t.accent.gamma_multiply(0.25) } else { t.bg_surface };
                                    let all_stroke = if is_all_sel { egui::Stroke::new(1.5_f32, t.accent) } else { egui::Stroke::new(1.0_f32, t.border) };

                                    let all_chip = egui::Button::new(
                                        RichText::new(format!("◆ All ({})", format_size(total_tree_size)))
                                            .size(11.0)
                                            .strong()
                                            .color(if is_all_sel { t.accent } else { t.text }),
                                    )
                                    .fill(all_bg)
                                    .stroke(all_stroke)
                                    .rounding(4.0_f32);

                                    if ui.add(all_chip).on_hover_text("Show all file types").clicked() {
                                        state.selected_category = None;
                                    }

                                    // Individual Category Chips
                                    for cat in cat_summaries {
                                        if cat.total_size == 0 { continue; }
                                        let is_sel = state.selected_category == Some(cat.category);
                                        let chip_bg = if is_sel {
                                            cat.category.color().gamma_multiply(0.25)
                                        } else {
                                            t.bg_surface
                                        };
                                        let chip_stroke = if is_sel {
                                            egui::Stroke::new(1.5_f32, cat.category.color())
                                        } else {
                                            egui::Stroke::new(1.0_f32, t.border)
                                        };

                                        let label_text = format!(
                                            "{} {} ({})",
                                            cat.category.icon(),
                                            cat.category.display_name(),
                                            format_size(cat.total_size)
                                        );

                                        let chip = egui::Button::new(
                                            RichText::new(label_text)
                                                .size(11.0)
                                                .color(if is_sel { cat.category.color() } else { t.text }),
                                        )
                                        .fill(chip_bg)
                                        .stroke(chip_stroke)
                                        .rounding(4.0_f32);

                                        if ui.add(chip).on_hover_text(format!("Filter by {}: {} files ({:.1}%)", cat.category.display_name(), cat.count, cat.percentage)).clicked() {
                                            if state.selected_category == Some(cat.category) {
                                                state.selected_category = None;
                                            } else {
                                                state.selected_category = Some(cat.category);
                                            }
                                        }
                                    }
                                });
                            });
                    });

                ui.add_space(8.0);

                // 2. Toolbar & Search Bar
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Filter Extension:").size(11.0).color(t.text_muted));
                    ui.add_sized(
                        [180.0, 24.0],
                        egui::TextEdit::singleline(&mut state.search_ext).hint_text("e.g. rs, mp4, tar..."),
                    );

                    if !state.search_ext.is_empty() && ui.small_button("✕").clicked() {
                        state.search_ext.clear();
                    }

                    if let Some(cat) = state.selected_category {
                        ui.label(RichText::new(format!("Filtering by: {}", cat.display_name())).size(11.0).color(cat.color()));
                        if ui.small_button("Clear Filter").clicked() {
                            state.selected_category = None;
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let sort_btn = ui.selectable_label(
                            state.sort_by_size,
                            if state.sort_by_size { "Sort: Size ↓" } else { "Sort: File Count ↓" },
                        );
                        if sort_btn.clicked() {
                            state.sort_by_size = !state.sort_by_size;
                        }

                        ui.label(
                            RichText::new(format!("Showing {} extensions", filtered_exts.len()))
                                .size(11.0)
                                .color(t.text_muted),
                        );
                    });
                });

                ui.add_space(6.0);

                // 3. Dense Extension Table Header
                let table_w = total_avail;
                egui::Frame::none()
                    .fill(t.bg_elevated)
                    .rounding(4.0_f32)
                    .stroke(egui::Stroke::new(1.0_f32, t.border))
                    .inner_margin(Margin::symmetric(10.0, 5.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.add_sized([160.0, 18.0], egui::Label::new(RichText::new("Extension / Format").strong().color(t.text_muted)));
                            ui.add_sized([140.0, 18.0], egui::Label::new(RichText::new("Category").strong().color(t.text_muted)));
                            let remaining = (table_w - 320.0).max(360.0);
                            let col_w = (remaining / 4.0).floor();
                            ui.add_sized([col_w, 18.0], egui::Label::new(RichText::new("File Count").strong().color(t.text_muted)));
                            ui.add_sized([col_w, 18.0], egui::Label::new(RichText::new("Logical Size").strong().color(t.text_muted)));
                            ui.add_sized([col_w, 18.0], egui::Label::new(RichText::new("Allocated Disk").strong().color(t.text_muted)));
                            ui.add_sized([col_w, 18.0], egui::Label::new(RichText::new("Space Share").strong().color(t.text_muted)));
                        });
                    });

                ui.add_space(2.0);

                // 4. Virtualized Dense Table Rows (26-28px Height)
                let row_height = 28.0_f32;
                let total_rows = filtered_exts.len();

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show_rows(ui, row_height, total_rows, |ui, row_range| {
                        for idx in row_range {
                            if let Some(ext) = filtered_exts.get(idx) {
                                let (row_rect, row_resp) = ui.allocate_exact_size(Vec2::new(table_w, 26.0), Sense::hover());
                                let is_hov = row_resp.hovered();

                                let bg = if is_hov {
                                    t.bg_elevated
                                } else if idx % 2 == 0 {
                                    t.bg_container
                                } else {
                                    Color32::TRANSPARENT
                                };

                                ui.painter().rect_filled(row_rect, 3.0_f32, bg);

                                // Col 1: Extension name
                                let ext_label = if ext.extension.is_empty() || ext.extension == "no extension" {
                                    "(extensionless)".to_string()
                                } else {
                                    format!(".{}", ext.extension)
                                };
                                ui.painter().text(
                                    egui::pos2(row_rect.min.x + 10.0, row_rect.center().y),
                                    egui::Align2::LEFT_CENTER,
                                    ext_label,
                                    egui::FontId::monospace(11.5),
                                    t.text,
                                );

                                // Col 2: Category Badge
                                let cat_badge_rect = Rect::from_min_size(
                                    egui::pos2(row_rect.min.x + 170.0, row_rect.center().y - 9.0),
                                    Vec2::new(120.0, 18.0),
                                );
                                ui.painter().rect_filled(cat_badge_rect, 3.0_f32, ext.category.color().gamma_multiply(0.18));
                                ui.painter().text(
                                    cat_badge_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    format!("{} {}", ext.category.icon(), ext.category.display_name()),
                                    egui::FontId::proportional(10.5),
                                    ext.category.color(),
                                );

                                let remaining = (table_w - 320.0).max(360.0);
                                let col_w = (remaining / 4.0).floor();
                                let base_x = row_rect.min.x + 310.0;

                                // Col 3: File Count (Monospace)
                                ui.painter().text(
                                    egui::pos2(base_x + col_w - 10.0, row_rect.center().y),
                                    egui::Align2::RIGHT_CENTER,
                                    format!("{}", ext.count),
                                    egui::FontId::monospace(11.5),
                                    t.text,
                                );

                                // Col 4: Apparent Size (Monospace)
                                ui.painter().text(
                                    egui::pos2(base_x + (col_w * 2.0) - 10.0, row_rect.center().y),
                                    egui::Align2::RIGHT_CENTER,
                                    format_size(ext.total_size),
                                    egui::FontId::monospace(11.5),
                                    t.text,
                                );

                                // Col 5: Allocated Size (Monospace)
                                ui.painter().text(
                                    egui::pos2(base_x + (col_w * 3.0) - 10.0, row_rect.center().y),
                                    egui::Align2::RIGHT_CENTER,
                                    format_size(ext.total_allocated_size),
                                    egui::FontId::monospace(11.5),
                                    t.cyan,
                                );

                                // Col 6: Share % and Bar
                                let bar_x = base_x + (col_w * 3.0) + 10.0;
                                let bar_w = (col_w - 70.0).max(30.0);
                                let mini_bar_rect = Rect::from_min_size(egui::pos2(bar_x, row_rect.center().y - 3.0), Vec2::new(bar_w, 6.0));
                                ui.painter().rect_filled(mini_bar_rect, 2.0_f32, t.bg_input);
                                let filled_w = (bar_w * (ext.percentage / 100.0)).clamp(1.0, bar_w);
                                let filled_rect = Rect::from_min_size(mini_bar_rect.min, Vec2::new(filled_w, mini_bar_rect.height()));
                                ui.painter().rect_filled(filled_rect, 2.0_f32, ext.category.color());

                                ui.painter().text(
                                    egui::pos2(base_x + (col_w * 4.0) - 10.0, row_rect.center().y),
                                    egui::Align2::RIGHT_CENTER,
                                    format!("{:.1}%", ext.percentage),
                                    egui::FontId::monospace(11.0),
                                    t.text_muted,
                                );
                            }
                        }
                    });

                ui.add_space(16.0);
            });
        });
}
