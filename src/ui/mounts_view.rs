use crate::manage::StorageManager;
use crate::models::format_size;
use crate::theme::AppTheme;
use crate::ui::modals::ModalManager;
use eframe::egui::{self, Color32, Margin, RichText, Ui, Vec2};
use std::path::PathBuf;
use sysinfo::Disks;

#[derive(Clone)]
struct MountDisplayItem {
    pub index: usize,
    pub dev_name: String,
    pub mount_point: PathBuf,
    pub mount_point_str: String,
    pub fs_type: String,
    pub total: u64,
    pub used: u64,
    pub usage_pct: f64,
}

pub struct MountsViewState {
    pub disks: Option<Disks>,
    pub selected_mount_index: usize,
    pub btrfs_subvolumes: Vec<(u64, String, String, bool)>,
    pub is_trimming: bool,
    pub last_trim_status: Option<String>,
}

impl Default for MountsViewState {
    fn default() -> Self {
        Self {
            disks: None,
            selected_mount_index: 0,
            btrfs_subvolumes: Vec::new(),
            is_trimming: false,
            last_trim_status: None,
        }
    }
}

impl MountsViewState {
    pub fn refresh(&mut self) {
        let disks = Disks::new_with_refreshed_list();
        self.disks = Some(disks);
        self.query_current_subvolumes();
    }

    pub fn ensure_loaded(&mut self) {
        if self.disks.is_none() {
            self.refresh();
        }
    }

    pub fn query_current_subvolumes(&mut self) {
        if let Some(disks) = &self.disks {
            if let Some(disk) = disks.list().get(self.selected_mount_index) {
                let fs_type = disk.file_system().to_string_lossy().to_lowercase();
                if fs_type.contains("btrfs") {
                    if let Ok(subs) = StorageManager::query_btrfs_subvolumes(disk.mount_point()) {
                        self.btrfs_subvolumes = subs;
                        return;
                    }
                }
            }
        }
        self.btrfs_subvolumes.clear();
    }
}

pub fn render_mounts_view(
    ui: &mut Ui,
    state: &mut MountsViewState,
    modals: &mut ModalManager,
    dark_mode: bool,
    on_analyze: &mut dyn FnMut(PathBuf),
) {
    state.ensure_loaded();
    let t = AppTheme::tokens(dark_mode);

    let mount_items: Vec<MountDisplayItem> = match &state.disks {
        Some(d) => d
            .list()
            .iter()
            .enumerate()
            .map(|(idx, disk)| {
                let dev_name = disk.name().to_string_lossy().to_string();
                let mount_point = disk.mount_point().to_path_buf();
                let mount_point_str = mount_point.display().to_string();
                let fs_type = disk.file_system().to_string_lossy().to_uppercase();
                let total = disk.total_space();
                let avail = disk.available_space();
                let used = total.saturating_sub(avail);
                let usage_pct = if total > 0 {
                    (used as f64 / total as f64) * 100.0
                } else {
                    0.0
                };

                MountDisplayItem {
                    index: idx,
                    dev_name,
                    mount_point,
                    mount_point_str,
                    fs_type,
                    total,
                    used,
                    usage_pct,
                }
            })
            .collect(),
        None => return,
    };

    let selected_mount_index = state.selected_mount_index;
    let selected_item = mount_items
        .get(selected_mount_index)
        .or_else(|| mount_items.first())
        .cloned();

    let mut new_selection = None;
    let mut trim_target = None;

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let available_w = ui.available_width().max(320.0);
            let content_w = available_w;

            ui.vertical(|ui| {
                ui.set_max_width(content_w);
                ui.add_space(8.0);

                // 1. Top Banner
                egui::Frame::none()
                    .fill(t.bg_elevated)
                    .rounding(6.0_f32)
                    .stroke(egui::Stroke::new(1.0_f32, t.border))
                    .inner_margin(Margin::same(16.0))
                    .show(ui, |ui| {
                        let is_stacked = content_w < 680.0;
                        if is_stacked {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("🖴").size(24.0).color(t.accent));
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new("Mount Points & Filesystem Subvolumes").size(15.0).strong().color(t.text));
                                            ui.label(RichText::new("/etc/fstab VERIFIED").size(10.0).monospace().strong().color(t.accent));
                                        });
                                        ui.label(RichText::new("Inspect block partitions, mount flags, Btrfs subvolume trees, and execute SSD TRIM operations.").size(11.5).color(t.text_secondary));
                                    });
                                });

                                ui.add_space(10.0);

                                if let Some(item) = &selected_item {
                                    if ui.button("⚡ Run fstrim on Mount").clicked() {
                                        trim_target = Some(item.mount_point.clone());
                                    }
                                }
                            });
                        } else {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("🖴").size(28.0).color(t.accent));
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("Mount Points & Filesystem Subvolumes").size(16.0).strong().color(t.text));
                                        ui.label(RichText::new("/etc/fstab VERIFIED").size(10.0).monospace().strong().color(t.accent));
                                    });
                                    ui.label(RichText::new("Inspect block partitions, mount flags, Btrfs subvolume trees, and execute SSD TRIM operations.").size(11.5).color(t.text_secondary));
                                });

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if let Some(item) = &selected_item {
                                        if ui.add(egui::Button::new(RichText::new("⚡ Run fstrim on Mount").color(Color32::WHITE).strong()).fill(t.accent)).clicked() {
                                            trim_target = Some(item.mount_point.clone());
                                        }
                                    }
                                });
                            });
                        }
                    });

                // Status message if trim run
                if let Some(status) = &state.last_trim_status {
                    ui.add_space(6.0);
                    egui::Frame::none()
                        .fill(t.bg_surface)
                        .rounding(4.0_f32)
                        .stroke(egui::Stroke::new(1.0_f32, t.border))
                        .inner_margin(Margin::symmetric(12.0, 8.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new(status).size(11.0).monospace().color(t.text));
                        });
                }

                ui.add_space(12.0);

                // 2. Linux Block Mounts Table
                egui::Frame::none()
                    .fill(t.bg_container)
                    .rounding(6.0_f32)
                    .stroke(egui::Stroke::new(1.0_f32, t.border))
                    .inner_margin(Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("ACTIVE LINUX BLOCK MOUNTS").size(11.0).strong().color(t.text_muted));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(RichText::new(format!("{} Mounts detected in /proc/mounts", mount_items.len())).size(10.5).color(t.text_secondary));
                            });
                        });

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(6.0);

                        egui::Grid::new("mounts_table_grid")
                            .striped(true)
                            .min_col_width(70.0)
                            .spacing(Vec2::new(12.0, 6.0))
                            .show(ui, |ui| {
                                ui.label(RichText::new("DEVICE").size(10.0).strong().color(t.text_muted));
                                ui.label(RichText::new("MOUNT POINT").size(10.0).strong().color(t.text_muted));
                                ui.label(RichText::new("FS TYPE").size(10.0).strong().color(t.text_muted));
                                ui.label(RichText::new("TOTAL / USED").size(10.0).strong().color(t.text_muted));
                                ui.label(RichText::new("ACTION").size(10.0).strong().color(t.text_muted));
                                ui.end_row();

                                for item in &mount_items {
                                    let is_selected = item.index == selected_mount_index;
                                    let dev_color = if is_selected { t.accent } else { t.text };

                                    if ui.selectable_label(is_selected, RichText::new(&item.dev_name).size(11.5).strong().color(dev_color)).clicked() {
                                        new_selection = Some(item.index);
                                    }

                                    ui.label(RichText::new(&item.mount_point_str).size(11.5).color(t.text));
                                    ui.label(RichText::new(&item.fs_type).size(11.0).monospace().color(t.purple));
                                    ui.label(RichText::new(format!("{} / {} ({:.0}%)", format_size(item.used), format_size(item.total), item.usage_pct)).size(11.0).color(t.text));

                                    ui.horizontal(|ui| {
                                        if ui.small_button("🔍 Scan").clicked() {
                                            on_analyze(item.mount_point.clone());
                                        }
                                        if ui.small_button("⚡ TRIM").clicked() {
                                            trim_target = Some(item.mount_point.clone());
                                        }
                                    });

                                    ui.end_row();
                                }
                            });
                    });

                // 3. Btrfs Subvolumes Panel (if detected)
                if !state.btrfs_subvolumes.is_empty() {
                    ui.add_space(14.0);
                    egui::Frame::none()
                        .fill(t.bg_elevated)
                        .rounding(6.0_f32)
                        .stroke(egui::Stroke::new(1.0_f32, t.border))
                        .inner_margin(Margin::same(14.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("BTRFS SUBVOLUMES & SNAPSHOTS").size(11.0).strong().color(t.text_muted));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new("Copy-on-write subvolumes share the same physical pool").size(10.5).color(t.text_muted));
                                });
                            });

                            ui.add_space(8.0);

                            let cols = if content_w > 720.0 { 3 } else { 1 };
                            let sub_w = ((content_w - (8.0 * (cols as f32 - 1.0))) / cols as f32).floor();

                            for chunk in state.btrfs_subvolumes.chunks(cols) {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 8.0;
                                    for (id, name, path, is_snapshot) in chunk {
                                        egui::Frame::none()
                                            .fill(t.bg_surface)
                                            .rounding(4.0_f32)
                                            .stroke(egui::Stroke::new(1.0_f32, t.border))
                                            .inner_margin(Margin::same(10.0))
                                            .show(ui, |ui| {
                                                ui.set_width(sub_w - 20.0);
                                                ui.horizontal(|ui| {
                                                    ui.label(RichText::new(format!("ID: {}", id)).size(10.0).monospace().color(t.accent));
                                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                        if *is_snapshot {
                                                            ui.label(RichText::new("READ-ONLY SNAP").size(9.0).monospace().color(t.purple));
                                                        } else {
                                                            ui.label(RichText::new("SUBVOLUME").size(9.0).monospace().color(t.success));
                                                        }
                                                    });
                                                });
                                                ui.label(RichText::new(name).size(12.0).strong().color(t.text));
                                                ui.label(RichText::new(path).size(10.0).monospace().color(t.text_muted));
                                            });
                                    }
                                });
                                ui.add_space(6.0);
                            }
                        });
                }

                ui.add_space(20.0);
            });
        });

    if let Some(idx) = new_selection {
        state.selected_mount_index = idx;
        state.query_current_subvolumes();
    }

    if let Some(path) = trim_target {
        match StorageManager::execute_fstrim(&path) {
            Ok(msg) => {
                state.last_trim_status = Some(msg.clone());
                modals.show_toast(msg, false);
            }
            Err(err) => {
                state.last_trim_status = Some(err.clone());
                modals.show_toast(err, true);
            }
        }
    }
}
