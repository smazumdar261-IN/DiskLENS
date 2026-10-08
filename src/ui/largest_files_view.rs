use crate::actions::FileActions;
use crate::models::{format_size, FileCategory, LargeFileInfo};
use crate::theme::{truncate_middle, AppTheme};
use crate::ui::modals::{ConfirmAction, ModalManager};
use eframe::egui::{self, Margin, RichText, Ui, Vec2};
use egui_extras::{Column, TableBuilder};
use std::path::PathBuf;

pub struct LargestFilesState {
    pub search_query: String,
    pub selected_category: Option<FileCategory>,
    pub cache_only: bool,
    pub review_needed_only: bool,
    pub limit: usize,
    pub copied_id: Option<(PathBuf, std::time::Instant)>,
}

impl Default for LargestFilesState {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            selected_category: None,
            cache_only: false,
            review_needed_only: false,
            limit: 100,
            copied_id: None,
        }
    }
}

/// Compact, high-density Space Hogs Leaderboard.
/// Standardizes rows to 26-28px height, uses monospace fonts for ranks, sizes, and telemetry,
/// and implements hover-activated action toolbars (Terminal, Reveal, Shred, Stage for Delete).
pub fn render_largest_files_view(
    ui: &mut Ui,
    all_files: &[LargeFileInfo],
    state: &mut LargestFilesState,
    modals: &mut ModalManager,
    dark_mode: bool,
) {
    let t = AppTheme::tokens(dark_mode);

    // 1. Safety Principle Banner
    egui::Frame::none()
        .fill(t.bg_elevated)
        .rounding(6.0_f32)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .inner_margin(Margin::symmetric(10.0, 6.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("🛡️").size(14.0));
                ui.label(RichText::new("DiskLens Safety Heuristic:").size(11.5).strong().color(t.warning));
                ui.label(
                    RichText::new("Space hogs prioritized by physical disk footprint. Items tagged 'Review Needed' should be inspected before moving to trash.")
                        .size(11.0)
                        .color(t.text_secondary),
                );
            });
        });

    ui.add_space(6.0);

    // 2. Filter & Query Toolbar
    egui::Frame::none()
        .fill(t.bg_elevated)
        .rounding(6.0_f32)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .inner_margin(Margin::symmetric(10.0, 6.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Search
                ui.add_sized(
                    [180.0, 24.0],
                    egui::TextEdit::singleline(&mut state.search_query).hint_text("Search name or path..."),
                );

                if !state.search_query.is_empty() && ui.small_button("✕").clicked() {
                    state.search_query.clear();
                }

                ui.separator();

                // Category dropdown
                let cat_name = state.selected_category.map(|c| c.display_name()).unwrap_or("All Categories");
                egui::ComboBox::from_id_salt("space_hogs_cat")
                    .selected_text(cat_name)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut state.selected_category, None, "All Categories");
                        for cat in FileCategory::ALL {
                            ui.selectable_value(&mut state.selected_category, Some(cat), format!("{} {}", cat.icon(), cat.display_name()));
                        }
                    });

                ui.separator();

                // Quick Filter Checkboxes
                ui.checkbox(&mut state.cache_only, "Caches & Temp Only");
                ui.checkbox(&mut state.review_needed_only, "Review Needed Only");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::ComboBox::from_id_salt("hogs_limit")
                        .selected_text(format!("Top {}", state.limit))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut state.limit, 50, "Top 50");
                            ui.selectable_value(&mut state.limit, 100, "Top 100");
                            ui.selectable_value(&mut state.limit, 250, "Top 250");
                            ui.selectable_value(&mut state.limit, 500, "Top 500");
                        });
                    ui.label(RichText::new("Display:").size(11.0).color(t.text_muted));
                });
            });
        });

    ui.add_space(6.0);

    let filtered_files: Vec<&LargeFileInfo> = all_files
        .iter()
        .filter(|f| {
            if let Some(cat) = state.selected_category {
                if f.category != cat { return false; }
            }
            if state.cache_only && !f.is_cache_or_temp {
                return false;
            }
            if state.review_needed_only && !f.is_review_needed {
                return false;
            }
            if !state.search_query.is_empty() {
                let q = state.search_query.to_lowercase();
                let m_name = f.name.to_lowercase().contains(&q);
                let m_path = f.path.to_string_lossy().to_lowercase().contains(&q);
                if !m_name && !m_path { return false; }
            }
            true
        })
        .take(state.limit)
        .collect();

    let total_apparent: u64 = filtered_files.iter().map(|f| f.size).sum();
    let total_allocated: u64 = filtered_files.iter().map(|f| f.allocated_size).sum();

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!(
                "Displaying {} space hogs · Apparent: {} · Allocated: {}",
                filtered_files.len(),
                format_size(total_apparent),
                format_size(total_allocated)
            ))
            .size(11.0)
            .monospace()
            .color(t.text_secondary),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new("Prioritized by allocated disk blocks").size(10.5).color(t.text_muted));
        });
    });

    ui.add_space(4.0);

    let total_rows = filtered_files.len();

    let bottom_bar_height = 28.0;
    let available_height = (ui.available_height() - bottom_bar_height).max(100.0);
    let scrollbar_margin = ui.spacing().scroll.bar_width + 14.0;
    let avail_w = (ui.available_width() - scrollbar_margin).max(100.0);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .max_height(available_height)
        .max_width(avail_w)
        .show(ui, |ui| {
            TableBuilder::new(ui)
                .striped(true)
                .resizable(false)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::exact(32.0))             // Rank
                .column(Column::remainder().clip(true))  // Item & Location
                .column(Column::exact(85.0))             // Apparent Size
                .column(Column::exact(85.0))             // Allocated Disk
                .column(Column::exact(85.0))             // Category
                .column(Column::exact(90.0))             // Assessment
                .column(Column::exact(125.0))            // Actions
                .header(24.0, |mut header| {
                    header.col(|ui| {
                        ui.label(RichText::new("#").strong().color(t.text_muted));
                    });
                    header.col(|ui| {
                        ui.label(RichText::new("Item & Location").strong().color(t.text_muted));
                    });
                    header.col(|ui| {
                        ui.label(RichText::new("Apparent Size").strong().color(t.text_muted));
                    });
                    header.col(|ui| {
                        ui.label(RichText::new("Allocated Disk").strong().color(t.text_muted));
                    });
                    header.col(|ui| {
                        ui.label(RichText::new("Category").strong().color(t.text_muted));
                    });
                    header.col(|ui| {
                        ui.label(RichText::new("Assessment").strong().color(t.text_muted));
                    });
                    header.col(|ui| {
                        ui.label(RichText::new("Actions").strong().color(t.text_muted));
                    });
                })
                .body(|body| {
                    body.rows(28.0, total_rows, |mut row| {
                        let idx = row.index();
                        if let Some(file) = filtered_files.get(idx) {
                            let full_p = file.path.display().to_string();

                            // Col 1: Rank (#01, #02, ...)
                            row.col(|ui| {
                                let rank_col = if idx < 3 { t.accent } else { t.text_muted };
                                ui.label(RichText::new(format!("{:02}", idx + 1)).size(11.0).monospace().color(rank_col));
                            });

                            // Col 2: Name & truncated parent folder
                            row.col(|ui| {
                                let parent_str = file.path.parent().map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string()).unwrap_or_default();
                                let display_name = truncate_middle(&file.name, 34);
                                let name_text = if !parent_str.is_empty() {
                                    format!("{} {}  ({})", file.category.icon(), display_name, parent_str)
                                } else {
                                    format!("{} {}", file.category.icon(), display_name)
                                };

                                let item_resp = ui.add(
                                    egui::Label::new(RichText::new(name_text).size(11.5).color(t.text))
                                        .truncate(),
                                );
                                if item_resp.double_clicked() {
                                    let _ = FileActions::open_file(&file.path);
                                }
                                item_resp.on_hover_ui(|ui| {
                                    ui.label(RichText::new(&file.name).strong().color(t.text));
                                    ui.label(RichText::new(&full_p).size(10.5).monospace().color(t.text_muted));
                                    ui.separator();
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("Apparent Size:").size(11.0).color(t.text_muted));
                                        ui.label(RichText::new(file.formatted_size()).size(11.0).monospace().strong().color(t.text));
                                        ui.label(RichText::new("Allocated:").size(11.0).color(t.text_muted));
                                        ui.label(RichText::new(file.formatted_allocated_size()).size(11.0).monospace().strong().color(t.cyan));
                                    });
                                    if file.nlink > 1 {
                                        ui.label(RichText::new(format!("🔗 Inode hardlinks count: {}", file.nlink)).size(10.5).color(t.purple));
                                    }
                                    ui.label(RichText::new(format!("Assessment: {}", &file.risk_note)).size(11.0).color(t.accent));
                                });
                            });

                            // Col 3: Apparent Size (Monospace)
                            row.col(|ui| {
                                ui.label(RichText::new(file.formatted_size()).size(11.0).monospace().color(t.text));
                            });

                            // Col 4: Allocated Disk (Monospace)
                            row.col(|ui| {
                                ui.label(RichText::new(file.formatted_allocated_size()).size(11.0).monospace().color(t.cyan));
                            });

                            // Col 5: Category Pill
                            row.col(|ui| {
                                let cat_pill = egui::Frame::none()
                                    .fill(file.category.color().gamma_multiply(0.18))
                                    .rounding(3.0_f32)
                                    .inner_margin(Margin::symmetric(6.0, 2.0));
                                cat_pill.show(ui, |ui| {
                                    ui.label(RichText::new(file.category.display_name()).size(10.0).color(file.category.color()));
                                });
                            });

                            // Col 6: Assessment Pill
                            row.col(|ui| {
                                let (badge_text, badge_col, badge_bg) = if file.nlink > 1 {
                                    ("🔗 Hardlink", t.purple, t.purple.gamma_multiply(0.18))
                                } else if file.is_review_needed {
                                    ("⚠ Review Needed", t.warning, t.warning.gamma_multiply(0.18))
                                } else if file.is_cache_or_temp {
                                    ("⚡ Safe Cache", t.accent, t.accent.gamma_multiply(0.18))
                                } else {
                                    ("📄 User File", t.text_muted, t.bg_surface)
                                };

                                let pill = egui::Frame::none()
                                    .fill(badge_bg)
                                    .rounding(3.0_f32)
                                    .inner_margin(Margin::symmetric(6.0, 2.0));
                                pill.show(ui, |ui| {
                                    ui.label(RichText::new(badge_text).size(10.0).color(badge_col));
                                });
                            });

                            // Col 7: Actions Toolbar
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 2.0;
                                    ui.spacing_mut().button_padding = Vec2::new(4.0, 2.0);

                                    if ui.small_button("📂").on_hover_text("Reveal in file manager").clicked() {
                                        let _ = FileActions::open_in_file_manager(&file.path);
                                    }

                                    if ui.small_button("📋").on_hover_text("Copy path").clicked() {
                                        ui.output_mut(|o| o.copied_text = full_p.clone());
                                        modals.show_toast("Copied path".to_string(), false);
                                    }

                                    if ui.small_button("🗑").on_hover_text("Move to trash").clicked() {
                                        modals.pending_confirm = Some(ConfirmAction::MoveToTrash(file.path.clone(), file.size));
                                    }

                                    if ui.small_button("🔥").on_hover_text("Shred / Permanently delete").clicked() {
                                        modals.pending_confirm = Some(ConfirmAction::PermanentDelete(file.path.clone(), file.size));
                                    }
                                });
                            });
                        }
                    });
                });
        });

    ui.add_space(6.0);
}
