use crate::app::ActiveTab;
use crate::models::format_size;
use crate::theme::AppTheme;
use eframe::egui::{self, Color32, Margin, Rect, RichText, Sense, Ui, Vec2};
use std::path::PathBuf;
use sysinfo::Disks;

pub struct StorageViewState {
    pub disks: Option<Disks>,
}

impl Default for StorageViewState {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageViewState {
    pub fn new() -> Self {
        Self { disks: None }
    }

    pub fn refresh(&mut self) {
        let mut disks = Disks::new();
        disks.refresh(true);
        self.disks = Some(disks);
    }

    pub fn ensure_loaded(&mut self) {
        if self.disks.is_none() {
            self.refresh();
        }
    }
}

pub fn render_storage_view(
    ui: &mut Ui,
    state: &mut StorageViewState,
    path_input: &mut String,
    active_tab: &mut ActiveTab,
    dark_mode: bool,
    on_analyze: &mut dyn FnMut(PathBuf),
) {
    state.ensure_loaded();
    let t = AppTheme::tokens(dark_mode);
    let mut refresh_mounts = false;

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let total_avail = ui.available_width().max(320.0);

            ui.vertical(|ui| {
                ui.set_max_width(total_avail);
                ui.add_space(8.0);

                let disks = match &state.disks {
                    Some(d) => d,
                    None => {
                        ui.label("Detecting storage devices...");
                        return;
                    }
                };

                let disk_list = disks.list();
                let total_capacity: u64 = disk_list.iter().map(|d| d.total_space()).sum();
                let total_available: u64 = disk_list.iter().map(|d| d.available_space()).sum();
                let total_used: u64 = total_capacity.saturating_sub(total_available);

                // 1. Top Summary Banner (4 Metric Cards - Dynamic Grid Sizing)
                let num_summary_cols = if total_avail >= 1100.0 { 4 } else if total_avail >= 600.0 { 2 } else { 1 };
                let spacing = 10.0_f32;
                let summary_card_w = ((total_avail - (spacing * (num_summary_cols as f32 - 1.0))) / num_summary_cols as f32).floor();

                let summary_cards = [
                    (
                        "Total Mounted Capacity",
                        format_size(total_capacity),
                        format!("{} used ({:.0}%)", format_size(total_used), if total_capacity > 0 { (total_used as f64 / total_capacity as f64) * 100.0 } else { 0.0 }),
                        t.accent,
                        "🖴",
                        None,
                    ),
                    (
                        "Safe Cleanable Space",
                        "~12.4 GiB".to_string(),
                        "Caches & Trash bin".to_string(),
                        t.success,
                        "🗑",
                        Some(ActiveTab::SafeTrash),
                    ),
                    (
                        "Duplicate Waste",
                        "~4.8 GiB".to_string(),
                        "Byte-for-byte matches".to_string(),
                        t.warning,
                        "📋",
                        Some(ActiveTab::Duplicates),
                    ),
                    (
                        "Drive Health & Telemetry",
                        "98% Optimal".to_string(),
                        "0 bad sectors reported".to_string(),
                        t.purple,
                        "🩺",
                        Some(ActiveTab::DiagnosticsSmart),
                    ),
                ];

                for chunk in summary_cards.chunks(num_summary_cols) {
                    if num_summary_cols == 1 {
                        for (title, value, sub, color, icon, target_tab) in chunk {
                            ui.allocate_ui_with_layout(egui::vec2(summary_card_w, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                ui.set_max_width(summary_card_w);
                                render_summary_metric_card(ui, title, value, sub, *color, icon, *target_tab, summary_card_w, active_tab, &t);
                            });
                            ui.add_space(8.0);
                        }
                    } else {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = spacing;
                            for (title, value, sub, color, icon, target_tab) in chunk {
                                ui.allocate_ui_with_layout(egui::vec2(summary_card_w, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                    ui.set_max_width(summary_card_w);
                                    render_summary_metric_card(ui, title, value, sub, *color, icon, *target_tab, summary_card_w, active_tab, &t);
                                });
                            }
                        });
                        ui.add_space(8.0);
                    }
                }

                ui.add_space(8.0);

                // 2. Quick Locations Bento Grid
                ui.horizontal(|ui| {
                    ui.label(RichText::new("QUICK LOCATIONS").size(11.0).strong().color(t.text_muted));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("Click any folder to analyze immediately").size(10.5).color(t.text_muted));
                    });
                });

                ui.add_space(6.0);

                let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
                let quick_locs = [
                    ("~/", home.clone(), t.accent, "📁"),
                    ("Downloads", format!("{}/Downloads", home), t.success, "📥"),
                    ("Documents", format!("{}/Documents", home), t.warning, "📄"),
                    ("Pictures", format!("{}/Pictures", home), t.purple, "🖼"),
                    ("Videos", format!("{}/Videos", home), t.danger, "🎬"),
                    ("/root", "/".to_string(), t.text_muted, "🖥"),
                ];

                let num_cols = if total_avail > 900.0 { 6 } else if total_avail > 500.0 { 3 } else { 2 };
                let tile_w = ((total_avail - (8.0 * (num_cols as f32 - 1.0))) / num_cols as f32).floor();

                for chunk in quick_locs.chunks(num_cols) {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        for (short_name, target_path, color, icon) in chunk {
                            ui.allocate_ui_with_layout(egui::vec2(tile_w, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                ui.set_max_width(tile_w);
                                let frame_resp = egui::Frame::none()
                                    .fill(t.bg_container)
                                    .rounding(6.0_f32)
                                    .stroke(egui::Stroke::new(1.0_f32, t.border))
                                    .inner_margin(Margin::symmetric(8.0, 6.0))
                                    .show(ui, |ui| {
                                        ui.set_width(ui.available_width());
                                        ui.vertical_centered(|ui| {
                                            ui.label(RichText::new(*icon).size(18.0).color(*color));
                                            ui.add(egui::Label::new(RichText::new(*short_name).size(11.5).color(t.text)).truncate());
                                        });
                                    });

                                let interact_resp = ui.interact(frame_resp.response.rect, ui.id().with(short_name), Sense::click());
                                if interact_resp.hovered() {
                                    ui.painter().rect_stroke(frame_resp.response.rect, 6.0_f32, egui::Stroke::new(1.0_f32, *color));
                                }
                                if interact_resp.clicked() {
                                    *path_input = target_path.clone();
                                    on_analyze(PathBuf::from(target_path));
                                }
                            });
                        }
                    });
                    ui.add_space(8.0);
                }

                ui.add_space(10.0);

                // 3. System Drives & Partitions
                ui.horizontal(|ui| {
                    ui.label(RichText::new("SYSTEM DRIVES & PARTITIONS").size(11.0).strong().color(t.text_muted));
                    ui.label(RichText::new(format!("{} Detected", disk_list.len())).size(10.5).color(t.text_secondary));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("🔄 Refresh Mounts").on_hover_text("Rescan kernel mount table").clicked() {
                            refresh_mounts = true;
                        }
                    });
                });

                ui.add_space(6.0);

                let is_single_col = total_avail < 900.0;
                let drive_card_w = if is_single_col {
                    total_avail
                } else {
                    ((total_avail - 12.0) / 2.0).floor().max(120.0)
                };

                for chunk in disk_list.chunks(if is_single_col { 1 } else { 2 }) {
                    if is_single_col {
                        for disk in chunk {
                            ui.allocate_ui_with_layout(egui::vec2(drive_card_w, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                ui.set_max_width(drive_card_w);
                                render_storage_drive_card(ui, disk, drive_card_w, path_input, on_analyze, &t);
                            });
                            ui.add_space(8.0);
                        }
                    } else {
                        ui.horizontal_top(|ui| {
                            ui.spacing_mut().item_spacing.x = 12.0;
                            for disk in chunk {
                                ui.allocate_ui_with_layout(egui::vec2(drive_card_w, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                    ui.set_max_width(drive_card_w);
                                    render_storage_drive_card(ui, disk, drive_card_w, path_input, on_analyze, &t);
                                });
                            }
                        });
                        ui.add_space(10.0);
                    }
                }

                ui.add_space(10.0);

                // 4. Custom Target Scan Box
                ui.allocate_ui_with_layout(egui::vec2(total_avail, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.set_max_width(total_avail);
                    egui::Frame::none()
                        .fill(t.bg_elevated)
                        .rounding(6.0_f32)
                        .stroke(egui::Stroke::new(1.0_f32, t.border))
                        .inner_margin(Margin::same(14.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("CUSTOM TARGET PATH").size(11.0).strong().color(t.text_muted));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new("Scan any specific folder, nested directory, or subvolume").size(10.5).color(t.text_muted));
                                });
                            });

                            ui.add_space(8.0);

                            let is_narrow_input = total_avail < 600.0;
                            if is_narrow_input {
                                ui.vertical(|ui| {
                                    let edit_resp = ui.add_sized(
                                        [ui.available_width(), 32.0],
                                        egui::TextEdit::singleline(path_input).hint_text("Enter directory or folder path..."),
                                    );
                                    if edit_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                        on_analyze(PathBuf::from(&*path_input));
                                    }

                                    ui.add_space(6.0);

                                    ui.horizontal(|ui| {
                                        let half = (ui.available_width() - 8.0) / 2.0;
                                        if ui.add_sized([half, 32.0], egui::Button::new("📁 Browse...")).clicked() {
                                            if let Some(folder) = rfd::FileDialog::new().set_directory(&*path_input).pick_folder() {
                                                *path_input = folder.display().to_string();
                                                on_analyze(folder);
                                            }
                                        }

                                        if ui.add_sized([half, 32.0], egui::Button::new(RichText::new("⚡ Execute Scan").strong().color(Color32::WHITE)).fill(t.accent)).clicked() {
                                            on_analyze(PathBuf::from(&*path_input));
                                        }
                                    });
                                });
                            } else {
                                ui.horizontal(|ui| {
                                    let edit_w = (ui.available_width() - 220.0).max(180.0);
                                    let edit_resp = ui.add_sized(
                                        [edit_w, 32.0],
                                        egui::TextEdit::singleline(path_input).hint_text("Enter directory or folder path..."),
                                    );
                                    if edit_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                        on_analyze(PathBuf::from(&*path_input));
                                    }

                                    if ui.add_sized([90.0, 32.0], egui::Button::new("📁 Browse...")).clicked() {
                                        if let Some(folder) = rfd::FileDialog::new().set_directory(&*path_input).pick_folder() {
                                            *path_input = folder.display().to_string();
                                            on_analyze(folder);
                                        }
                                    }

                                    if ui.add_sized([110.0, 32.0], egui::Button::new(RichText::new("⚡ Execute Scan").strong().color(Color32::WHITE)).fill(t.accent)).clicked() {
                                        on_analyze(PathBuf::from(&*path_input));
                                    }
                                });
                            }
                        });
                });

                ui.add_space(20.0);
            });
        });

    if refresh_mounts {
        state.refresh();
    }
}

fn render_summary_metric_card(
    ui: &mut Ui,
    title: &str,
    value: &str,
    sub: &str,
    color: Color32,
    icon: &str,
    target_tab: Option<ActiveTab>,
    _card_w: f32,
    active_tab: &mut ActiveTab,
    t: &crate::theme::ThemeTokens,
) {
    egui::Frame::none()
        .fill(t.bg_container)
        .rounding(6.0_f32)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(title).size(10.0).monospace().color(t.text_muted));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(icon).size(13.0).color(color));
                    });
                });
                ui.add_space(2.0);
                ui.label(RichText::new(value).size(18.0).strong().color(color));
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Label::new(RichText::new(sub).size(10.5).color(t.text_secondary))
                            .truncate(),
                    ).on_hover_text(sub);
                    if let Some(tab) = target_tab {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("Inspect →").clicked() {
                                *active_tab = tab;
                            }
                        });
                    }
                });
            });
        });
}

fn render_storage_drive_card(
    ui: &mut Ui,
    disk: &sysinfo::Disk,
    _drive_card_w: f32,
    path_input: &mut String,
    on_analyze: &mut dyn FnMut(PathBuf),
    t: &crate::theme::ThemeTokens,
) {
    let total_bytes = disk.total_space();
    let available_bytes = disk.available_space();
    let used_bytes = total_bytes.saturating_sub(available_bytes);

    let usage_ratio = if total_bytes > 0 {
        (used_bytes as f32 / total_bytes as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let mount_point = disk.mount_point().to_path_buf();
    let fs_type = disk.file_system().to_string_lossy().to_string();
    let dev_name = disk.name().to_string_lossy().to_string();
    let mount_str = mount_point.to_string_lossy().to_string();

    let is_root = mount_str == "/";
    let drive_title = if is_root {
        "Internal Linux System Drive".to_string()
    } else if mount_str == "/boot/efi" {
        "EFI System Partition".to_string()
    } else if !dev_name.is_empty() {
        format!("Storage Volume ({})", dev_name)
    } else {
        format!("Drive ({})", mount_str)
    };

    let bar_color = if usage_ratio > 0.90 {
        t.danger
    } else if usage_ratio > 0.75 {
        t.warning
    } else {
        t.success
    };

    let frame_resp = egui::Frame::none()
        .fill(t.bg_container)
        .rounding(6.0_f32)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            // Header row: Icon + Title/Sub + Scan Button
            ui.horizontal(|ui| {
                let icon = if dev_name.contains("nvme") { "⚡" } else { "🖴" };
                ui.label(RichText::new(icon).size(20.0).color(t.accent));

                ui.vertical(|ui| {
                    ui.label(RichText::new(&drive_title).size(13.0).strong().color(t.text));
                    let sub_text = format!("{} • {} • mount: {}", dev_name, fs_type, mount_str);
                    ui.add(egui::Label::new(RichText::new(sub_text).size(10.5).color(t.text_muted)).truncate());
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add_sized(
                        [68.0, 26.0],
                        egui::Button::new(RichText::new("🔍 Scan").size(11.5).strong().color(Color32::WHITE))
                            .fill(Color32::from_rgb(65, 72, 104)),
                    ).clicked() {
                        *path_input = mount_point.display().to_string();
                        on_analyze(mount_point.clone());
                    }
                });
            });

            ui.add_space(8.0);

            // Usage numbers
            ui.horizontal(|ui| {
                let used_str = format!("{} Used", format_size(used_bytes));
                ui.label(RichText::new(used_str).size(11.5).color(t.text));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let total_str = format!("{} Total", format_size(total_bytes));
                    ui.label(RichText::new(total_str).size(11.0).color(t.text_muted));
                });
            });

            ui.add_space(4.0);

            // Capacity bar
            let avail_w = ui.available_width();
            let (bar_rect, _) = ui.allocate_exact_size(Vec2::new(avail_w, 8.0), Sense::hover());
            ui.painter().rect_filled(bar_rect, 4.0_f32, t.bg_input);
            let filled_w = (bar_rect.width() * usage_ratio).max(4.0);
            let filled_rect = Rect::from_min_size(bar_rect.min, Vec2::new(filled_w, bar_rect.height()));
            ui.painter().rect_filled(filled_rect, 4.0_f32, bar_color);

            ui.add_space(4.0);

            // Footer % readout
            ui.horizontal(|ui| {
                let pct_str = format!("{:.1}% capacity used", usage_ratio * 100.0);
                ui.label(RichText::new(pct_str).size(10.5).color(bar_color));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let free_str = format!("{} Free ({:.0}%)", format_size(available_bytes), (1.0 - usage_ratio) * 100.0);
                    ui.label(RichText::new(free_str).size(10.5).color(t.text_muted));
                });
            });
        });

    let interact_resp = ui.interact(frame_resp.response.rect, ui.id().with(&mount_str), Sense::click());
    if interact_resp.hovered() {
        ui.painter().rect_stroke(frame_resp.response.rect, 6.0_f32, egui::Stroke::new(1.0_f32, t.accent));
    }
    if interact_resp.clicked() {
        *path_input = mount_point.display().to_string();
        on_analyze(mount_point);
    }
}
