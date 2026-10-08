use crate::models::{format_size, AnalysisCache, ExtensionSummary, FileCategory};
use crate::theme::AppTheme;
use eframe::egui::{self, Margin, Rect, RichText, Sense, Ui, Vec2};
use egui_extras::{Column, TableBuilder};

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

    let total_avail = ui.available_width().max(340.0);

    ui.vertical(|ui| {
        ui.set_max_width(total_avail);

        // 1. Compact Horizontal Category Filter Chip Bar
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(6.0, 4.0);

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

        ui.add_space(4.0);

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

        ui.separator();

        // 3. Dense Extension Table using TableBuilder (fills remaining viewport height)
        let total_rows = filtered_exts.len();

                TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .vscroll(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(Column::initial(160.0).at_least(110.0).resizable(true))
                    .column(Column::initial(140.0).at_least(100.0).resizable(true))
                    .column(Column::initial(100.0).at_least(75.0).resizable(true))
                    .column(Column::initial(110.0).at_least(80.0).resizable(true))
                    .column(Column::initial(110.0).at_least(80.0).resizable(true))
                    .column(Column::remainder().at_least(110.0))
                    .header(24.0, |mut header| {
                        header.col(|ui| {
                            ui.label(RichText::new("Extension / Format").strong().color(t.text_muted));
                        });
                        header.col(|ui| {
                            ui.label(RichText::new("Category").strong().color(t.text_muted));
                        });
                        header.col(|ui| {
                            ui.label(RichText::new("File Count").strong().color(t.text_muted));
                        });
                        header.col(|ui| {
                            ui.label(RichText::new("Logical Size").strong().color(t.text_muted));
                        });
                        header.col(|ui| {
                            ui.label(RichText::new("Allocated Disk").strong().color(t.text_muted));
                        });
                        header.col(|ui| {
                            ui.label(RichText::new("Space Share").strong().color(t.text_muted));
                        });
                    })
                    .body(|body| {
                        body.rows(28.0, total_rows, |mut row| {
                            let idx = row.index();
                            if let Some(ext) = filtered_exts.get(idx) {
                                // Col 1: Extension name
                                row.col(|ui| {
                                    let ext_label = if ext.extension.is_empty() || ext.extension == "no extension" {
                                        "(extensionless)".to_string()
                                    } else {
                                        format!(".{}", ext.extension)
                                    };
                                    ui.label(RichText::new(ext_label).size(11.5).monospace().color(t.text));
                                });

                                // Col 2: Category Badge
                                row.col(|ui| {
                                    let cat_badge = egui::Frame::none()
                                        .fill(ext.category.color().gamma_multiply(0.18))
                                        .rounding(3.0_f32)
                                        .inner_margin(Margin::symmetric(6.0, 2.0));
                                    cat_badge.show(ui, |ui| {
                                        ui.label(
                                            RichText::new(format!("{} {}", ext.category.icon(), ext.category.display_name()))
                                                .size(10.5)
                                                .color(ext.category.color()),
                                        );
                                    });
                                });

                                // Col 3: File Count
                                row.col(|ui| {
                                    ui.label(RichText::new(format!("{}", ext.count)).size(11.0).monospace().color(t.text));
                                });

                                // Col 4: Logical Size
                                row.col(|ui| {
                                    ui.label(RichText::new(format_size(ext.total_size)).size(11.0).monospace().color(t.text));
                                });

                                // Col 5: Allocated Disk
                                row.col(|ui| {
                                    ui.label(RichText::new(format_size(ext.total_allocated_size)).size(11.0).monospace().color(t.cyan));
                                });

                                // Col 6: Space Share
                                row.col(|ui| {
                                    ui.horizontal(|ui| {
                                        let bar_w = 50.0_f32;
                                        let (bar_rect, _) = ui.allocate_exact_size(Vec2::new(bar_w, 6.0), Sense::hover());
                                        ui.painter().rect_filled(bar_rect, 2.0_f32, t.bg_input);
                                        let filled_w = (bar_w * (ext.percentage / 100.0)).clamp(1.0, bar_w);
                                        let filled_rect = Rect::from_min_size(bar_rect.min, Vec2::new(filled_w, bar_rect.height()));
                                        ui.painter().rect_filled(filled_rect, 2.0_f32, ext.category.color());

                                        ui.label(RichText::new(format!("{:.1}%", ext.percentage)).size(11.0).monospace().color(t.text_muted));
                                    });
                                });
                            }
                        });
                });
        });
}
