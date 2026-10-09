use crate::actions::FileActions;
use crate::duplicates::{DuplicateDetector, DuplicateGroup};
use crate::models::{format_size, FileNode};
use crate::theme::{truncate_middle, AppTheme};
use crate::ui::modals::{ConfirmAction, ModalManager};
use crossbeam_channel::Receiver;
use eframe::egui::{self, Color32, Margin, RichText, Ui};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub struct DuplicatesViewState {
    pub duplicate_groups: Vec<DuplicateGroup>,
    pub search_query: String,
    pub category_filter: String,
    pub sort_option: usize, // 0 = Total Size, 1 = Group Size, 2 = Name
    pub collapsed_groups: HashSet<String>,
    pub has_analyzed: bool,
    pub is_analyzing: bool,
    pub scan_rx: Option<Receiver<Vec<DuplicateGroup>>>,
    pub progress_counter: Option<Arc<AtomicUsize>>,
    pub total_candidate_files: usize,
}

impl Default for DuplicatesViewState {
    fn default() -> Self {
        Self {
            duplicate_groups: Vec::new(),
            search_query: String::new(),
            category_filter: "All".to_string(),
            sort_option: 0,
            collapsed_groups: HashSet::new(),
            has_analyzed: false,
            is_analyzing: false,
            scan_rx: None,
            progress_counter: None,
            total_candidate_files: 0,
        }
    }
}

impl DuplicatesViewState {
    pub fn trigger_background_scan(&mut self, root: &FileNode) {
        if self.is_analyzing {
            return;
        }
        let candidates = DuplicateDetector::collect_candidates(root, 1024 * 1024);
        let total_files: usize = candidates.iter().map(|(_, p)| p.len()).sum();
        if total_files == 0 {
            self.duplicate_groups.clear();
            self.has_analyzed = true;
            self.is_analyzing = false;
            self.scan_rx = None;
            self.progress_counter = None;
            self.total_candidate_files = 0;
            return;
        }

        let (tx, rx) = crossbeam_channel::unbounded();
        let counter = Arc::new(AtomicUsize::new(0));
        self.progress_counter = Some(counter.clone());
        self.total_candidate_files = total_files;
        self.scan_rx = Some(rx);
        self.is_analyzing = true;

        std::thread::spawn(move || {
            let groups =
                DuplicateDetector::compute_duplicates_from_candidates(candidates, Some(counter));
            let _ = tx.send(groups);
        });
    }
}

pub fn render_duplicates_view(
    ui: &mut Ui,
    root: Option<&FileNode>,
    state: &mut DuplicatesViewState,
    modals: &mut ModalManager,
    dark_mode: bool,
) {
    let t = AppTheme::tokens(dark_mode);

    let Some(root) = root else {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(80.0);
                    egui::Frame::none()
                        .fill(t.bg_elevated)
                        .rounding(8.0_f32)
                        .stroke(egui::Stroke::new(1.0_f32, t.border))
                        .inner_margin(Margin::same(24.0))
                        .show(ui, |ui| {
                            ui.set_max_width(520.0);
                            ui.vertical_centered(|ui| {
                                ui.label(RichText::new("📋").size(36.0).color(t.accent));
                                ui.add_space(6.0);
                                ui.label(RichText::new("No Storage Target Analyzed").size(16.0).strong().color(t.text));
                                ui.add_space(6.0);
                                ui.label(
                                    RichText::new("Duplicate detection performs cryptographic SHA-256 byte-for-byte comparisons across analyzed files. Please select a drive or folder from Storage Home to begin.")
                                        .size(12.0)
                                        .color(t.text_secondary),
                                );
                            });
                        });
                });
            });
        return;
    };

    // Poll background receiver
    if state.is_analyzing {
        ui.ctx().request_repaint(); // Redraw UI at 60 FPS while background thread works
        if let Some(rx) = &state.scan_rx {
            if let Ok(groups) = rx.try_recv() {
                state.duplicate_groups = groups;
                state.has_analyzed = true;
                state.is_analyzing = false;
                state.scan_rx = None;
                state.progress_counter = None;
            }
        }
    }

    // Initial analysis trigger if not yet performed
    if !state.has_analyzed && !state.is_analyzing {
        state.trigger_background_scan(root);
    }

    // While analyzing without existing cached groups, show smooth indeterminate/determinate loader
    if state.is_analyzing && state.duplicate_groups.is_empty() {
        let processed = state
            .progress_counter
            .as_ref()
            .map(|c| c.load(Ordering::Relaxed))
            .unwrap_or(0);
        let total = state.total_candidate_files;
        let fraction = if total > 0 {
            (processed as f32 / total as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(80.0);
                    egui::Frame::none()
                        .fill(t.bg_elevated)
                        .rounding(8.0_f32)
                        .stroke(egui::Stroke::new(1.0_f32, t.border))
                        .inner_margin(Margin::same(24.0))
                        .show(ui, |ui| {
                            ui.set_max_width(520.0);
                            ui.vertical_centered(|ui| {
                                ui.spinner();
                                ui.add_space(10.0);
                                ui.label(RichText::new("Scanning Duplicates").size(16.0).strong().color(t.text));
                                ui.add_space(8.0);
                                ui.add(
                                    egui::ProgressBar::new(fraction)
                                        .show_percentage()
                                        .animate(true)
                                );
                                ui.add_space(8.0);
                                ui.label(
                                    RichText::new(format!("Hashed {} of {} candidates ({:.0}%)", processed, total, fraction * 100.0))
                                        .size(12.0)
                                        .monospace()
                                        .color(t.text_secondary),
                                );
                                ui.add_space(6.0);
                                ui.label(
                                    RichText::new("Comparing byte-for-byte SHA-256 hashes in background via Rayon. UI remains responsive at 60 FPS.")
                                        .size(11.0)
                                        .color(t.text_muted),
                                );
                            });
                        });
                });
            });
        return;
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let scrollbar_margin = ui.spacing().scroll.bar_width + 14.0;
            let available_w = (ui.available_width() - scrollbar_margin).max(320.0);
            let content_w = available_w;

            ui.vertical(|ui| {
                ui.set_max_width(content_w);
                ui.add_space(6.0);

                // 1. Top Header & Action Controls (Matches DuplicatesView.tsx)
                ui.horizontal_wrapped(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Duplicate Finder").size(18.0).strong().color(t.text));
                        ui.label(RichText::new("Identify and safely review SHA-256 byte-for-byte duplicate candidates.").size(11.0).color(t.text_secondary));
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Move to Trash Button
                        let selected_items_count: usize = state
                            .duplicate_groups
                            .iter()
                            .flat_map(|g| &g.items)
                            .filter(|i| i.selected_for_trash)
                            .count();

                        let selected_bytes: u64 = state
                            .duplicate_groups
                            .iter()
                            .flat_map(|g| &g.items)
                            .filter(|i| i.selected_for_trash)
                            .map(|i| i.size_bytes)
                            .sum();

                        let trash_label = if selected_items_count > 0 {
                            format!("🗑 MOVE TO TRASH ({})", selected_items_count)
                        } else {
                            "🗑 MOVE TO TRASH (0)".to_string()
                        };

                        let trash_btn = ui.add_enabled(
                            selected_items_count > 0,
                            egui::Button::new(RichText::new(trash_label).size(11.5).strong().color(Color32::WHITE))
                                .fill(if selected_items_count > 0 { Color32::from_rgb(180, 50, 70) } else { t.bg_elevated })
                                .rounding(4.0_f32)
                        );

                        if trash_btn.clicked() {
                            let first_path = state
                                .duplicate_groups
                                .iter()
                                .flat_map(|g| &g.items)
                                .find(|i| i.selected_for_trash)
                                .map(|i| i.path.clone())
                                .unwrap_or_else(|| PathBuf::from("duplicates"));

                            modals.pending_confirm = Some(ConfirmAction::MoveToTrash(first_path, selected_bytes));
                        }

                        // Review Selection Button (auto-selects all non-originals)
                        let review_btn = ui.add(
                            egui::Button::new(RichText::new("☑ REVIEW SELECTION").size(11.5).strong().color(t.accent))
                                .fill(t.bg_elevated)
                                .stroke(egui::Stroke::new(1.0_f32, t.border))
                                .rounding(4.0_f32)
                        ).on_hover_text("Auto-select all duplicate copies while preserving original files");

                        if review_btn.clicked() {
                            for grp in &mut state.duplicate_groups {
                                for (idx, item) in grp.items.iter_mut().enumerate() {
                                    item.selected_for_trash = idx > 0; // keep first as original
                                }
                            }
                            modals.show_toast("Selected redundant duplicate copies".to_string(), false);
                        }

                        if ui.small_button("Deselect All").clicked() {
                            for grp in &mut state.duplicate_groups {
                                for item in &mut grp.items {
                                    item.selected_for_trash = false;
                                }
                            }
                        }

                        let rescan_btn = ui.add_enabled(
                            !state.is_analyzing,
                            egui::Button::new(RichText::new("🔄 Rescan").size(11.0))
                        );
                        if rescan_btn.clicked() {
                            state.trigger_background_scan(root);
                            modals.show_toast("Started background duplicate scan".to_string(), false);
                        }
                    });
                });

                if state.is_analyzing {
                    ui.add_space(6.0);
                    let processed = state
                        .progress_counter
                        .as_ref()
                        .map(|c| c.load(Ordering::Relaxed))
                        .unwrap_or(0);
                    let total = state.total_candidate_files;
                    let fraction = if total > 0 {
                        (processed as f32 / total as f32).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            RichText::new(format!("Rescanning in background: {}/{} candidates ({:.0}%)", processed, total, fraction * 100.0))
                                .size(11.0)
                                .color(t.accent)
                        );
                    });
                }

                ui.add_space(10.0);

                // 2. Summary Cards Bento (Total Duplicates, Groups, Potential Savings)
                let total_dup_bytes: u64 = state.duplicate_groups.iter().map(|g| g.total_size_bytes).sum();
                let potential_savings_bytes: u64 = state.duplicate_groups.iter().map(|g| g.potential_savings()).sum();
                let group_count = state.duplicate_groups.len();

                let is_narrow_bento = content_w < 720.0;
                let spacing = 8.0_f32;
                let card_h = 88.0_f32;
                if is_narrow_bento {
                    ui.vertical(|ui| {
                        render_summary_card(ui, "TOTAL DUPLICATES", &format_size(total_dup_bytes), "Across all duplicate clusters", t.text, &t, content_w, card_h);
                        ui.add_space(spacing);
                        render_summary_card(ui, "GROUPS IDENTIFIED", &format!("{}", group_count), "Identical SHA-256 file groups", t.accent, &t, content_w, card_h);
                        ui.add_space(spacing);
                        render_summary_card(ui, "POTENTIAL SAVINGS", &format_size(potential_savings_bytes), "Reclaimable redundant storage", t.success, &t, content_w, card_h);
                    });
                } else {
                    let card_w = ((content_w - (spacing * 2.0)) / 3.0).floor();
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = spacing;
                        ui.allocate_ui_with_layout(egui::vec2(card_w, card_h), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                            ui.set_max_width(card_w);
                            render_summary_card(ui, "TOTAL DUPLICATES", &format_size(total_dup_bytes), "Across all duplicate clusters", t.text, &t, card_w, card_h);
                        });
                        ui.allocate_ui_with_layout(egui::vec2(card_w, card_h), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                            ui.set_max_width(card_w);
                            render_summary_card(ui, "GROUPS IDENTIFIED", &format!("{}", group_count), "Identical SHA-256 file groups", t.accent, &t, card_w, card_h);
                        });
                        ui.allocate_ui_with_layout(egui::vec2(card_w, card_h), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                            ui.set_max_width(card_w);
                            render_summary_card(ui, "POTENTIAL SAVINGS", &format_size(potential_savings_bytes), "Reclaimable redundant storage", t.success, &t, card_w, card_h);
                        });
                    });
                }

                ui.add_space(10.0);

                // 3. Safety Notice Banner (Matches DuplicatesView.tsx)
                egui::Frame::none()
                    .fill(t.bg_elevated)
                    .rounding(8.0_f32)
                    .stroke(egui::Stroke::new(1.0_f32, t.border))
                    .inner_margin(Margin::same(10.0))
                    .show(ui, |ui| {
                        ui.set_width(content_w - 20.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("⚠").size(16.0).color(t.warning));
                            ui.label(RichText::new("Safety Notice:").strong().color(t.warning));
                            ui.label(
                                RichText::new("DiskLens never deletes duplicates automatically. Review paths carefully before staging items to Freedesktop Trash.")
                                    .size(11.5)
                                    .color(t.text_secondary)
                            );
                        });
                    });

                ui.add_space(10.0);

                // 4. Query Filter Bar
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("Filter:").size(11.5).color(t.text_muted));
                    ui.add_sized(
                        [160.0, 24.0],
                        egui::TextEdit::singleline(&mut state.search_query).hint_text("Search duplicate file..."),
                    );

                    if !state.search_query.is_empty() && ui.small_button("✕").clicked() {
                        state.search_query.clear();
                    }

                    ui.separator();
                    ui.label(RichText::new("Category:").size(11.5).color(t.text_muted));

                    let categories = ["All", "Videos", "Archives", "Images", "Code", "Documents", "Other"];
                    for cat in categories {
                        let is_sel = state.category_filter == cat;
                        if ui.selectable_label(is_sel, cat).clicked() {
                            state.category_filter = cat.to_string();
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let sort_labels = ["Total Size ↓", "Group Size ↓", "File Name A-Z"];
                        egui::ComboBox::from_id_salt("dup_sort_combo")
                            .selected_text(sort_labels[state.sort_option])
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut state.sort_option, 0, sort_labels[0]);
                                ui.selectable_value(&mut state.sort_option, 1, sort_labels[1]);
                                ui.selectable_value(&mut state.sort_option, 2, sort_labels[2]);
                            });
                        ui.label(RichText::new("Sort by:").size(11.0).color(t.text_muted));
                    });
                });

                ui.add_space(8.0);

                let mut filtered_groups: Vec<&mut DuplicateGroup> = state
                    .duplicate_groups
                    .iter_mut()
                    .filter(|g| {
                        if state.category_filter != "All" && g.category.display_name() != state.category_filter {
                            return false;
                        }
                        if !state.search_query.is_empty() {
                            let q = state.search_query.to_lowercase();
                            let match_name = g.name.to_lowercase().contains(&q);
                            let match_paths = g.items.iter().any(|i| i.path.to_string_lossy().to_lowercase().contains(&q));
                            if !match_name && !match_paths {
                                return false;
                            }
                        }
                        true
                    })
                    .collect();

                match state.sort_option {
                    0 => filtered_groups.sort_by(|a, b| b.total_size_bytes.cmp(&a.total_size_bytes)),
                    1 => filtered_groups.sort_by(|a, b| b.items.len().cmp(&a.items.len())),
                    2 => filtered_groups.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
                    _ => {}
                }

                // 5. Main Duplicate Accordion Groups List
                if filtered_groups.is_empty() {
                    egui::Frame::none()
                        .fill(t.bg_elevated)
                        .rounding(8.0_f32)
                        .stroke(egui::Stroke::new(1.0_f32, t.border))
                        .inner_margin(Margin::same(24.0))
                        .show(ui, |ui| {
                            ui.set_width(content_w - 24.0);
                            ui.vertical_centered(|ui| {
                                ui.label(RichText::new("✨").size(28.0));
                                ui.add_space(4.0);
                                ui.label(RichText::new("No Duplicate Files Found").size(14.0).strong().color(t.text));
                                ui.label(RichText::new("Your analyzed storage target has zero redundant duplicate file collisions (>1 MB).").size(11.0).color(t.text_secondary));
                            });
                        });
                } else {
                    let card_inner_w = content_w - 20.0;
                    for grp in filtered_groups {
                        let is_collapsed = state.collapsed_groups.contains(&grp.id);
                        let selected_count = grp.items.iter().filter(|i| i.selected_for_trash).count();

                        egui::Frame::none()
                            .fill(t.bg_elevated)
                            .rounding(8.0_f32)
                            .stroke(egui::Stroke::new(1.0_f32, t.border))
                            .inner_margin(Margin::same(10.0))
                            .show(ui, |ui| {
                                ui.set_width(card_inner_w);

                                // Group Accordion Header
                                ui.horizontal(|ui| {
                                    let toggle_label = if is_collapsed { "▶" } else { "▼" };
                                    if ui.small_button(toggle_label).clicked() {
                                        if is_collapsed {
                                            state.collapsed_groups.remove(&grp.id);
                                        } else {
                                            state.collapsed_groups.insert(grp.id.clone());
                                        }
                                    }

                                    ui.label(RichText::new(grp.category.icon()).size(14.0));
                                    ui.label(RichText::new(&grp.name).size(13.0).strong().color(t.text));

                                    // Badge items
                                    ui.label(
                                        RichText::new(format!("{} files", grp.items.len()))
                                            .size(10.5)
                                            .monospace()
                                            .color(t.text_muted)
                                    );

                                    if selected_count > 0 {
                                        ui.label(
                                            RichText::new(format!("{} marked", selected_count))
                                                .size(10.5)
                                                .monospace()
                                                .color(t.danger)
                                        );
                                    }

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(
                                            RichText::new(format_size(grp.total_size_bytes))
                                                .size(13.0)
                                                .strong()
                                                .color(grp.category.color()),
                                        );
                                        ui.label(
                                            RichText::new(format!("({} each)", format_size(grp.size_bytes_per_file)))
                                                .size(10.5)
                                                .color(t.text_muted),
                                        );
                                    });
                                });

                                // Items Table
                                if !is_collapsed {
                                    ui.add_space(6.0);
                                    ui.separator();
                                    ui.add_space(4.0);

                                    for item in &mut grp.items {
                                        ui.horizontal(|ui| {
                                            ui.checkbox(&mut item.selected_for_trash, "");

                                            if item.is_original {
                                                ui.label(
                                                    RichText::new("ORIGINAL")
                                                        .size(9.5)
                                                        .monospace()
                                                        .strong()
                                                        .color(t.accent)
                                                );
                                            } else {
                                                ui.label(
                                                    RichText::new("MATCH")
                                                        .size(9.5)
                                                        .monospace()
                                                        .strong()
                                                        .color(t.success)
                                                );
                                            }

                                            let path_str = item.path.display().to_string();
                                            let path_avail_w = (card_inner_w - 300.0).max(60.0);
                                            let max_p = (path_avail_w / 7.5) as usize;
                                            let short_path = truncate_middle(&path_str, max_p.clamp(12, 60));
                                            ui.add_sized(
                                                [path_avail_w, 18.0],
                                                egui::Label::new(
                                                    RichText::new(short_path)
                                                        .size(11.5)
                                                        .monospace()
                                                        .color(if item.selected_for_trash { t.danger } else { t.text })
                                                ).truncate(),
                                            ).on_hover_text(&path_str);

                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.small_button("📂").on_hover_text("Reveal in file manager").clicked() {
                                                    let _ = FileActions::open_in_file_manager(&item.path);
                                                }
                                                if ui.small_button("📋").on_hover_text("Copy full path").clicked() {
                                                    ui.output_mut(|o| o.copied_text = item.path.display().to_string());
                                                    modals.show_toast("Copied path".to_string(), false);
                                                }
                                                ui.label(RichText::new(format_size(item.size_bytes)).size(11.5).color(t.text));
                                                ui.label(RichText::new(&item.modified_date).size(10.5).color(t.text_muted));
                                            });
                                        });
                                        ui.add_space(2.0);
                                    }
                                }
                            });
                        ui.add_space(6.0);
                    }
                }

                ui.add_space(24.0);
            });
        });
}

fn render_summary_card(
    ui: &mut Ui,
    title: &str,
    value: &str,
    subtitle: &str,
    accent: Color32,
    t: &crate::theme::ThemeTokens,
    width: f32,
    card_h: f32,
) {
    egui::Frame::none()
        .fill(t.bg_elevated)
        .rounding(8.0_f32)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            ui.set_width(width - 24.0);
            ui.set_height(card_h - 24.0);
            ui.label(RichText::new(title).size(10.5).strong().color(t.text_muted));
            ui.add_space(2.0);
            ui.label(RichText::new(value).size(22.0).strong().color(accent));
            ui.add_space(2.0);
            ui.add(
                egui::Label::new(RichText::new(subtitle).size(10.5).color(t.text_secondary))
                    .truncate(),
            );
        });
}
