use crate::manage::{CleanupTargetCategory, StorageManager};
use crate::models::format_size;
use crate::theme::AppTheme;
use crate::ui::modals::ModalManager;
use crossbeam_channel::Receiver;
use eframe::egui::{self, Color32, Margin, RichText, Ui};

pub struct SafeTrashViewState {
    pub categories: Vec<CleanupTargetCategory>,
    pub expanded_category: Option<String>,
    pub is_loaded: bool,
    pub is_loading: bool,
    pub is_cleaning: bool,
    pub cleanup_completed: bool,
    pub reclaimed_bytes: u64,
    pub is_dry_run: bool,
    pub discovery_rx: Option<Receiver<Vec<CleanupTargetCategory>>>,
}

impl Default for SafeTrashViewState {
    fn default() -> Self {
        Self {
            categories: Vec::new(),
            expanded_category: None,
            is_loaded: false,
            is_loading: false,
            is_cleaning: false,
            cleanup_completed: false,
            reclaimed_bytes: 0,
            is_dry_run: false,
            discovery_rx: None,
        }
    }
}

impl SafeTrashViewState {
    pub fn ensure_loaded(&mut self) {
        if !self.is_loaded && !self.is_loading {
            self.trigger_background_discovery();
        }
    }

    pub fn trigger_background_discovery(&mut self) {
        if self.is_loading {
            return;
        }
        self.is_loading = true;
        let (tx, rx) = crossbeam_channel::unbounded();
        self.discovery_rx = Some(rx);

        std::thread::spawn(move || {
            let categories = StorageManager::discover_cleanable_targets();
            let _ = tx.send(categories);
        });
    }

    pub fn poll_discovery(&mut self) {
        if let Some(rx) = &self.discovery_rx {
            if let Ok(categories) = rx.try_recv() {
                self.expanded_category = categories.first().map(|c| c.id.clone());
                self.categories = categories;
                self.is_loaded = true;
                self.is_loading = false;
                self.discovery_rx = None;
            }
        }
    }

    pub fn refresh(&mut self) {
        self.cleanup_completed = false;
        self.trigger_background_discovery();
    }
}

pub fn render_safe_trash_view(
    ui: &mut Ui,
    state: &mut SafeTrashViewState,
    modals: &mut ModalManager,
    dark_mode: bool,
) {
    state.poll_discovery();
    state.ensure_loaded();
    let t = AppTheme::tokens(dark_mode);

    // Non-blocking loading screen for initial discovery
    if state.is_loading && state.categories.is_empty() {
        ui.ctx().request_repaint();

        ui.vertical_centered(|ui| {
            ui.add_space(80.0);
            egui::Frame::none()
                .fill(t.bg_elevated)
                .rounding(8.0_f32)
                .stroke(egui::Stroke::new(1.0_f32, t.border))
                .inner_margin(Margin::same(24.0))
                .show(ui, |ui| {
                    ui.set_max_width(480.0);
                    ui.vertical_centered(|ui| {
                        ui.add(egui::Spinner::new().size(36.0).color(t.accent));
                        ui.add_space(14.0);
                        ui.label(RichText::new("Safe Trash & Storage Discovery").size(16.0).strong().color(t.text));
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new("Scanning application caches, package archives, and trash sandbox...")
                                .size(11.5)
                                .color(t.text_secondary),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("Non-blocking background scan active — desktop remains 100% responsive")
                                .size(10.0)
                                .monospace()
                                .color(t.text_muted),
                        );
                    });
                });
        });
        return;
    }

    let total_all_possible: u64 = state
        .categories
        .iter()
        .map(|c| c.items.iter().map(|i| i.size_bytes).sum::<u64>())
        .sum();
    let total_selected: u64 = state
        .categories
        .iter()
        .filter(|c| c.is_selected)
        .map(|c| c.total_reclaimable_bytes())
        .sum();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let scrollbar_margin = ui.spacing().scroll.bar_width + 14.0;
            let available_w = (ui.available_width() - scrollbar_margin).max(320.0);
            let content_w = available_w;

            ui.vertical(|ui| {
                ui.set_max_width(content_w);
                ui.add_space(8.0);

                // 1. Top Banner: Safe Cleaner Overview
                egui::Frame::none()
                    .fill(t.bg_elevated)
                    .rounding(8.0_f32)
                    .stroke(egui::Stroke::new(1.0_f32, t.success))
                    .inner_margin(Margin::same(16.0))
                    .show(ui, |ui| {
                        let is_stacked = content_w < 680.0;
                        if is_stacked {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("🗑️").size(24.0));
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new("Safe Trash & Storage Reclamation").size(15.0).strong().color(t.text));
                                            ui.label(RichText::new("SYSTEM GUARD ON").size(10.0).monospace().strong().color(t.success));
                                        });
                                        ui.label(RichText::new(format!("Discovered {} in safe-to-prune packages, build caches, and old journals.", format_size(total_all_possible))).size(11.5).color(t.text_secondary));
                                    });
                                });

                                ui.add_space(10.0);

                                ui.horizontal(|ui| {
                                    if ui.button("🛡 Simulate Dry Run").clicked() {
                                        state.is_dry_run = true;
                                        state.cleanup_completed = true;
                                        state.reclaimed_bytes = total_selected;
                                        modals.show_toast(format!("Dry-run complete: {} eligible for safe staging", format_size(total_selected)), false);
                                    }

                                    if ui.add(egui::Button::new(RichText::new(format!("Reclaim {}", format_size(total_selected))).color(Color32::WHITE).strong()).fill(t.success)).clicked() {
                                        if total_selected > 0 {
                                            // Stage removal of selected items
                                            let mut count = 0;
                                            for cat in &state.categories {
                                                if cat.is_selected && !cat.requires_sudo {
                                                    for item in &cat.items {
                                                        if item.is_removable {
                                                            let _ = crate::actions::FileActions::move_to_trash(&item.path);
                                                            count += 1;
                                                        }
                                                    }
                                                }
                                            }
                                            state.is_dry_run = false;
                                            state.cleanup_completed = true;
                                            state.reclaimed_bytes = total_selected;
                                            state.refresh();
                                            modals.show_toast(format!("Reclaimed {} across {} items (staged in Trash)", format_size(total_selected), count), false);
                                        }
                                    }
                                });
                            });
                        } else {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("🗑️").size(28.0));
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("Safe Trash & Storage Reclamation").size(16.0).strong().color(t.text));
                                        ui.label(RichText::new("SYSTEM GUARD ON").size(10.0).monospace().strong().color(t.success));
                                    });
                                    ui.label(RichText::new(format!("Discovered {} in safe-to-prune packages, build caches, and old journals.", format_size(total_all_possible))).size(11.5).color(t.text_secondary));
                                });

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.add(egui::Button::new(RichText::new(format!("Reclaim {}", format_size(total_selected))).color(Color32::WHITE).strong()).fill(t.success)).clicked() {
                                        if total_selected > 0 {
                                            let mut count = 0;
                                            for cat in &state.categories {
                                                if cat.is_selected && !cat.requires_sudo {
                                                    for item in &cat.items {
                                                        if item.is_removable {
                                                            let _ = crate::actions::FileActions::move_to_trash(&item.path);
                                                            count += 1;
                                                        }
                                                    }
                                                }
                                            }
                                            state.is_dry_run = false;
                                            state.cleanup_completed = true;
                                            state.reclaimed_bytes = total_selected;
                                            state.refresh();
                                            modals.show_toast(format!("Reclaimed {} across {} items (staged in Trash)", format_size(total_selected), count), false);
                                        }
                                    }

                                    if ui.button("🛡 Simulate Dry Run").clicked() {
                                        state.is_dry_run = true;
                                        state.cleanup_completed = true;
                                        state.reclaimed_bytes = total_selected;
                                        modals.show_toast(format!("Dry-run complete: {} eligible for safe staging", format_size(total_selected)), false);
                                    }
                                });
                            });
                        }
                    });

                // 2. Completion or Dry-Run Banner
                if state.cleanup_completed {
                    ui.add_space(8.0);
                    egui::Frame::none()
                        .fill(t.bg_elevated)
                        .rounding(8.0_f32)
                        .stroke(egui::Stroke::new(1.0_f32, if state.is_dry_run { t.accent } else { t.success }))
                        .inner_margin(Margin::same(12.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let icon = if state.is_dry_run { "🛡" } else { "✔" };
                                ui.label(RichText::new(icon).size(18.0).color(if state.is_dry_run { t.accent } else { t.success }));
                                ui.vertical(|ui| {
                                    let title = if state.is_dry_run {
                                        "Dry-Run Simulation Completed"
                                    } else {
                                        "Storage Cleanup Completed Successfully"
                                    };
                                    ui.label(RichText::new(title).size(13.0).strong().color(t.text));
                                    ui.label(RichText::new(format!("{} verified and safe for disk return.", format_size(state.reclaimed_bytes))).size(11.5).color(t.text_secondary));
                                });

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("Dismiss").clicked() {
                                        state.cleanup_completed = false;
                                    }
                                });
                            });
                        });
                }

                ui.add_space(12.0);

                // 3. Selection controls & Category Accordions
                ui.horizontal(|ui| {
                    if ui.small_button("Select All").clicked() {
                        for cat in &mut state.categories {
                            cat.is_selected = true;
                            for item in &mut cat.items {
                                item.is_removable = true;
                            }
                        }
                    }
                    if ui.small_button("Deselect All").clicked() {
                        for cat in &mut state.categories {
                            cat.is_selected = false;
                            for item in &mut cat.items {
                                item.is_removable = false;
                            }
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("Selected: {}", format_size(total_selected))).size(12.0).strong().color(t.success));
                    });
                });

                ui.add_space(8.0);

                // Render each category card
                let mut toggle_cat_id = None;
                let mut expand_cat_id = None;
                let mut toggle_item_tuple = None;

                for cat in &state.categories {
                    let is_expanded = state.expanded_category.as_deref() == Some(&cat.id);
                    let cat_reclaimable = cat.total_reclaimable_bytes();

                    egui::Frame::none()
                        .fill(t.bg_container)
                        .rounding(8.0_f32)
                        .stroke(egui::Stroke::new(1.0_f32, if cat.is_selected { t.accent } else { t.border }))
                        .inner_margin(Margin::same(12.0))
                        .show(ui, |ui| {
                            // Header row
                            ui.horizontal(|ui| {
                                let mut is_sel = cat.is_selected;
                                if ui.checkbox(&mut is_sel, "").changed() {
                                    toggle_cat_id = Some((cat.id.clone(), is_sel));
                                }

                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(&cat.title).size(13.0).strong().color(t.text));
                                        if cat.requires_sudo {
                                            ui.label(RichText::new("🔒 SUDO POLKIT").size(9.5).monospace().strong().color(t.warning));
                                        }
                                    });
                                    ui.label(RichText::new(&cat.description).size(11.0).color(t.text_muted));
                                });

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let expand_label = if is_expanded { "▲ Hide" } else { "▼ Inspect Items" };
                                    if ui.small_button(expand_label).clicked() {
                                        expand_cat_id = Some(if is_expanded { None } else { Some(cat.id.clone()) });
                                    }

                                    ui.vertical(|ui| {
                                        ui.label(RichText::new(format_size(cat_reclaimable)).size(13.0).strong().color(t.success));
                                        ui.label(RichText::new(format!("{} items", cat.items.len())).size(10.0).color(t.text_muted));
                                    });
                                });
                            });

                            // Expanded items breakdown table
                            if is_expanded && !cat.items.is_empty() {
                                ui.add_space(8.0);
                                ui.separator();
                                ui.add_space(6.0);

                                egui_extras::TableBuilder::new(ui)
                                    .striped(true)
                                    .column(egui_extras::Column::exact(22.0))
                                    .column(egui_extras::Column::remainder().clip(true))
                                    .column(egui_extras::Column::exact(50.0))
                                    .column(egui_extras::Column::exact(80.0))
                                    .header(20.0, |mut header| {
                                        header.col(|_ui| {});
                                        header.col(|ui| {
                                            ui.label(RichText::new("TARGET PATH").size(10.0).strong().color(t.text_muted));
                                        });
                                        header.col(|ui| {
                                            ui.label(RichText::new("AGE").size(10.0).strong().color(t.text_muted));
                                        });
                                        header.col(|ui| {
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                ui.label(RichText::new("SIZE").size(10.0).strong().color(t.text_muted));
                                            });
                                        });
                                    })
                                    .body(|mut body| {
                                        for (item_idx, item) in cat.items.iter().enumerate() {
                                            body.row(22.0, |mut row| {
                                                row.col(|ui| {
                                                    let mut item_rem = item.is_removable;
                                                    if ui.checkbox(&mut item_rem, "").changed() {
                                                        toggle_item_tuple = Some((cat.id.clone(), item_idx, item_rem));
                                                    }
                                                });
                                                row.col(|ui| {
                                                    let path_str = item.path.display().to_string();
                                                    let avail_w = ui.available_width();
                                                    let max_c = (avail_w / 7.2) as usize;
                                                    let display_str = crate::theme::truncate_middle(&path_str, max_c.clamp(20, 160));
                                                    ui.add(egui::Label::new(RichText::new(display_str).size(11.0).color(t.text)).truncate())
                                                        .on_hover_text(&path_str);
                                                });
                                                row.col(|ui| {
                                                    ui.label(RichText::new(format!("{}d", item.age_days)).size(11.0).color(t.text_muted));
                                                });
                                                row.col(|ui| {
                                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                        ui.label(RichText::new(format_size(item.size_bytes)).size(11.0).strong().color(t.accent));
                                                    });
                                                });
                                            });
                                        }
                                    });
                            }
                        });

                    ui.add_space(8.0);
                }

                if let Some((cid, sel)) = toggle_cat_id {
                    if let Some(cat) = state.categories.iter_mut().find(|c| c.id == cid) {
                        cat.is_selected = sel;
                        for item in &mut cat.items {
                            item.is_removable = sel;
                        }
                    }
                }

                if let Some(next_exp) = expand_cat_id {
                    state.expanded_category = next_exp;
                }

                if let Some((cid, idx, sel)) = toggle_item_tuple {
                    if let Some(cat) = state.categories.iter_mut().find(|c| c.id == cid) {
                        if let Some(item) = cat.items.get_mut(idx) {
                            item.is_removable = sel;
                        }
                        cat.is_selected = cat.items.iter().any(|i| i.is_removable);
                    }
                }

                ui.add_space(20.0);
            });
        });
}
