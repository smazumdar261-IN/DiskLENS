use crate::actions::FileActions;
use crate::theme::AppTheme;
use eframe::egui::{self, Align2, Color32, Margin, RichText, Vec2, Window};
use std::path::PathBuf;
use sysinfo::Disks;

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    MoveToTrash(PathBuf, u64),
    PermanentDelete(PathBuf, u64),
}

pub struct ModalManager {
    pub pending_confirm: Option<ConfirmAction>,
    pub is_dir_picker_open: bool,
    pub is_mount_points_open: bool,
    pub is_system_logs_open: bool,
    pub is_docs_open: bool,
    pub toast_message: Option<(String, bool, std::time::Instant)>, // (msg, is_error, timestamp)
    pub log_messages: Vec<(String, String, String)>, // (timestamp, level, message)
    pub custom_dir_path: String,
}

impl ModalManager {
    pub fn new() -> Self {
        let default_logs = vec![
            ("14:22:01".to_string(), "INFO".to_string(), "DiskLens engine initialized on Linux x86_64".to_string()),
            ("14:22:01".to_string(), "INFO".to_string(), "Rayon thread pool configured with 16 worker threads".to_string()),
            ("14:22:02".to_string(), "DEBUG".to_string(), "Enumerated kernel mounts via /proc/mounts".to_string()),
            ("14:22:02".to_string(), "INFO".to_string(), "Wayland wgpu surface created (144 Hz refresh target)".to_string()),
        ];

        let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());

        Self {
            pending_confirm: None,
            is_dir_picker_open: false,
            is_mount_points_open: false,
            is_system_logs_open: false,
            is_docs_open: false,
            toast_message: None,
            log_messages: default_logs,
            custom_dir_path: home_dir,
        }
    }

    pub fn log(&mut self, level: &str, msg: &str) {
        let time_str = chrono::Local::now().format("%H:%M:%S").to_string();
        self.log_messages.push((time_str, level.to_string(), msg.to_string()));
    }

    pub fn show_toast(&mut self, message: String, is_error: bool) {
        self.toast_message = Some((message, is_error, std::time::Instant::now()));
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        dark_mode: bool,
        on_deleted: &mut dyn FnMut(&PathBuf),
        on_select_custom_path: &mut dyn FnMut(PathBuf),
    ) {
        let t = AppTheme::tokens(dark_mode);

        // 1. Toast Notifications (Auto-dismiss after 4 seconds)
        if let Some((msg, is_error, timestamp)) = &self.toast_message {
            if timestamp.elapsed().as_secs() < 4 {
                egui::Area::new(egui::Id::new("toast_area"))
                    .anchor(Align2::RIGHT_BOTTOM, Vec2::new(-20.0, -40.0))
                    .show(ctx, |ui| {
                        let bg_color = if *is_error { t.danger } else { t.success };

                        egui::Frame::none()
                            .fill(bg_color)
                            .rounding(6.0_f32)
                            .inner_margin(Margin::symmetric(14.0, 10.0))
                            .shadow(egui::epaint::Shadow {
                                offset: Vec2::new(0.0, 4.0),
                                blur: 10.0,
                                spread: 0.0,
                                color: Color32::from_black_alpha(120),
                            })
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let icon = if *is_error { "⚠" } else { "✔" };
                                    ui.label(RichText::new(icon).size(16.0).color(Color32::WHITE).strong());
                                    ui.label(RichText::new(msg).color(Color32::WHITE).size(12.5).strong());
                                });
                            });
                    });
            } else {
                self.toast_message = None;
            }
        }

        // 2. Trash Confirmation Modal (Matches TrashConfirmationModal.tsx)
        if let Some(action) = self.pending_confirm.clone() {
            let mut close_modal = false;
            let (path, size, is_perm) = match &action {
                ConfirmAction::MoveToTrash(p, s) => (p, *s, false),
                ConfirmAction::PermanentDelete(p, s) => (p, *s, true),
            };

            let screen_size = ctx.screen_rect().size();
            let modal_width = 460.0_f32.min(screen_size.x - 32.0);

            Window::new("Confirm Destructive Action")
                .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
                .collapsible(false)
                .resizable(false)
                .fixed_size(Vec2::new(modal_width, 240.0))
                .show(ctx, |ui| {
                    ui.add_space(4.0);

                    // Target details box
                    egui::Frame::none()
                        .fill(t.bg_surface)
                        .rounding(6.0_f32)
                        .stroke(egui::Stroke::new(1.0_f32, t.border))
                        .inner_margin(Margin::same(12.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Target:").size(11.5).color(t.text_muted));
                                ui.label(RichText::new(path.file_name().unwrap_or_default().to_string_lossy()).size(12.5).strong().color(t.text));
                            });
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Space to Reclaim:").size(11.5).color(t.text_muted));
                                ui.label(RichText::new(crate::models::format_size(size)).size(13.0).strong().color(t.accent));
                            });
                            ui.label(RichText::new(path.display().to_string()).size(10.5).monospace().color(t.text_muted));
                        });

                    ui.add_space(8.0);

                    if is_perm {
                        ui.label(RichText::new("⚠ Permanent Deletion (unlink): Direct unrecoverable filesystem removal. This cannot be undone.").size(11.5).color(t.danger));
                    } else {
                        ui.label(RichText::new("🗑️ Move to Trash (Recommended): Sends file to ~/.local/share/Trash. Can be restored if needed.").size(11.5).color(t.text_secondary));
                    }

                    ui.add_space(14.0);

                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            close_modal = true;
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if is_perm {
                                if ui.add(egui::Button::new(RichText::new("Permanently Delete").color(Color32::WHITE).strong()).fill(t.danger)).clicked() {
                                    match FileActions::permanent_delete(path) {
                                        Ok(_) => {
                                            self.show_toast(format!("Permanently deleted: {}", path.file_name().unwrap_or_default().to_string_lossy()), false);
                                            on_deleted(path);
                                        }
                                        Err(e) => {
                                            self.show_toast(e, true);
                                        }
                                    }
                                    close_modal = true;
                                }
                            } else {
                                if ui.add(egui::Button::new(RichText::new("Move to Trash").color(Color32::WHITE).strong()).fill(t.accent)).clicked() {
                                    match FileActions::move_to_trash(path) {
                                        Ok(_) => {
                                            self.show_toast(format!("Moved to trash: {}", path.file_name().unwrap_or_default().to_string_lossy()), false);
                                            on_deleted(path);
                                        }
                                        Err(e) => {
                                            self.show_toast(e, true);
                                        }
                                    }
                                    close_modal = true;
                                }
                            }
                        });
                    });
                });

            if close_modal {
                self.pending_confirm = None;
            }
        }

        // 3. Mount Points Modal (Matches MountPointsModal.tsx)
        if self.is_mount_points_open {
            let mut close_mounts = false;
            let screen_size = ctx.screen_rect().size();
            let modal_width = 720.0_f32.min(screen_size.x - 32.0);
            let modal_height = 440.0_f32.min(screen_size.y - 48.0);

            Window::new("Linux Mount Table (/proc/mounts)")
                .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
                .collapsible(false)
                .fixed_size(Vec2::new(modal_width, modal_height))
                .show(ctx, |ui| {
                    ui.label(RichText::new("Active kernel file system mount points and disk devices:").size(11.5).color(t.text_secondary));
                    ui.add_space(6.0);

                    let disks = Disks::new_with_refreshed_list();

                    egui::ScrollArea::both()
                        .max_height(modal_height - 100.0)
                        .show(ui, |ui| {
                            egui::Grid::new("mount_points_grid")
                                .striped(true)
                                .min_col_width(80.0)
                                .spacing(Vec2::new(12.0, 6.0))
                                .show(ui, |ui| {
                                    ui.label(RichText::new("Mount Point").strong().color(t.accent));
                                    ui.label(RichText::new("Device").strong().color(t.text));
                                    ui.label(RichText::new("FS").strong().color(t.text));
                                    ui.label(RichText::new("Total").strong().color(t.text));
                                    ui.label(RichText::new("Used").strong().color(t.text));
                                    ui.label(RichText::new("Avail").strong().color(t.success));
                                    ui.end_row();

                                    for disk in &disks {
                                        let mount_path = disk.mount_point().display().to_string();
                                        let dev_name = disk.name().to_string_lossy().to_string();
                                        let fs_type = disk.file_system().to_string_lossy().to_string();
                                        let total = disk.total_space();
                                        let avail = disk.available_space();
                                        let used = total.saturating_sub(avail);
                                        let pct = if total > 0 { (used as f64 / total as f64) * 100.0 } else { 0.0 };

                                        ui.label(RichText::new(&mount_path).strong().color(t.accent));
                                        ui.label(RichText::new(&dev_name).color(t.text_muted));
                                        ui.label(RichText::new(&fs_type).color(t.text_muted));
                                        ui.label(RichText::new(crate::models::format_size(total)).color(t.text));
                                        ui.label(RichText::new(format!("{} ({:.0}%)", crate::models::format_size(used), pct)).color(t.text));
                                        ui.label(RichText::new(crate::models::format_size(avail)).color(t.success));
                                        ui.end_row();
                                    }
                                });
                        });

                    ui.add_space(10.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Close").clicked() {
                            close_mounts = true;
                        }
                    });
                });

            if close_mounts {
                self.is_mount_points_open = false;
            }
        }

        // 4. System Logs Modal (Matches SystemLogsModal.tsx)
        if self.is_system_logs_open {
            let mut close_logs = false;
            let screen_size = ctx.screen_rect().size();
            let modal_width = 640.0_f32.min(screen_size.x - 32.0);
            let modal_height = 420.0_f32.min(screen_size.y - 48.0);

            Window::new("DiskLens Engine Telemetry Logs")
                .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
                .collapsible(false)
                .fixed_size(Vec2::new(modal_width, modal_height))
                .show(ctx, |ui| {
                    ui.label(RichText::new("Background thread telemetry and scanning logs:").size(11.5).color(t.text_secondary));
                    ui.add_space(6.0);

                    egui::Frame::none()
                        .fill(Color32::from_rgb(13, 14, 21))
                        .rounding(4.0_f32)
                        .inner_margin(Margin::same(10.0))
                        .show(ui, |ui| {
                            egui::ScrollArea::both()
                                .max_height(modal_height - 110.0)
                                .show(ui, |ui| {
                                    for (time, level, msg) in &self.log_messages {
                                        let lvl_color = if level == "INFO" {
                                            t.accent
                                        } else if level == "DEBUG" {
                                            t.purple
                                        } else {
                                            t.warning
                                        };

                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(format!("[{}]", time)).size(10.5).monospace().color(t.text_muted));
                                            ui.label(RichText::new(level).size(10.5).strong().monospace().color(lvl_color));
                                            ui.label(RichText::new(msg).size(11.0).monospace().color(Color32::WHITE));
                                        });
                                    }
                                });
                        });

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Engine threads: 16 (Rayon crawler)").size(10.5).color(t.text_muted));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Close").clicked() {
                                close_logs = true;
                            }
                        });
                    });
                });

            if close_logs {
                self.is_system_logs_open = false;
            }
        }

        // 5. Docs & Shortcuts Modal (Matches DocsModal.tsx)
        if self.is_docs_open {
            let mut close_docs = false;
            let screen_size = ctx.screen_rect().size();
            let modal_width = 620.0_f32.min(screen_size.x - 32.0);
            let modal_height = 460.0_f32.min(screen_size.y - 48.0);

            Window::new("DiskLens Guide & Workflow Philosophy")
                .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
                .collapsible(false)
                .fixed_size(Vec2::new(modal_width, modal_height))
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(modal_height - 60.0)
                        .show(ui, |ui| {
                            ui.label(RichText::new("Core Design Workflow").size(14.0).strong().color(t.accent));
                            ui.add_space(2.0);
                            ui.label(RichText::new("1. Scan  →  2. Understand  →  3. Find what matters  →  4. Review  →  5. Act safely").size(12.0).strong().color(t.success));

                            ui.add_space(10.0);
                            ui.label(RichText::new("Safety First: Why DiskLens is Not an Automatic Cleaner").size(13.0).strong().color(t.text));
                            ui.label(RichText::new("Automatic cleaning tools risk deleting essential development repositories, database journals, or flatpak runtimes. DiskLens provides precision telemetry and high-contrast squarified treemaps so you can make informed decisions.").size(11.5).color(t.text_secondary));

                            ui.add_space(12.0);
                            ui.label(RichText::new("Keyboard Shortcuts").size(13.0).strong().color(t.text));

                            let shortcuts = [
                                ("1", "Overview Dashboard"),
                                ("2", "Visual Treemap Visualizer"),
                                ("3", "Folder Tree Explorer"),
                                ("4", "Space Hogs Leaderboard"),
                                ("5", "File Types Breakdown"),
                                ("6", "Duplicate Finder (SHA-256)"),
                                ("7", "Storage History & Growth"),
                                ("8", "Rust Engine & CLI Shell"),
                                ("R", "Rescan Target Directory"),
                                ("T", "Toggle Dark / Light Theme"),
                                (", / S", "Engine Settings Modal"),
                                ("Esc", "Dismiss Modals / Clear Search"),
                            ];

                            for (key, desc) in shortcuts {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(format!("[ {} ]", key)).size(11.0).monospace().strong().color(t.accent));
                                    ui.label(RichText::new(desc).size(11.5).color(t.text));
                                });
                            }
                        });

                    ui.add_space(8.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Got It").clicked() {
                            close_docs = true;
                        }
                    });
                });

            if close_docs {
                self.is_docs_open = false;
            }
        }

        // 6. Directory Picker Modal (Matches DirectoryPickerModal.tsx)
        if self.is_dir_picker_open {
            let mut close_picker = false;
            let mut chosen_path = None;
            let screen_size = ctx.screen_rect().size();
            let modal_width = 560.0_f32.min(screen_size.x - 32.0);
            let modal_height = 420.0_f32.min(screen_size.y - 48.0);

            Window::new("Select Target Directory")
                .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
                .collapsible(false)
                .fixed_size(Vec2::new(modal_width, modal_height))
                .show(ctx, |ui| {
                    ui.label(RichText::new("Type or browse directory path to analyze:").size(11.5).color(t.text_secondary));
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        let text_w = (modal_width - 160.0).max(120.0);
                        ui.add(egui::TextEdit::singleline(&mut self.custom_dir_path).desired_width(text_w));
                        if ui.button("📁 Native Dialog...").clicked() {
                            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                self.custom_dir_path = folder.display().to_string();
                            }
                        }
                    });

                    ui.add_space(10.0);

                    // Preset locations
                    ui.label(RichText::new("Quick Locations:").size(11.0).color(t.text_muted));
                    ui.horizontal_wrapped(|ui| {
                        let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
                        if ui.small_button("Home (~)").clicked() {
                            self.custom_dir_path = home.clone();
                        }
                        if ui.small_button("Downloads").clicked() {
                            self.custom_dir_path = format!("{}/Downloads", home);
                        }
                        if ui.small_button("Documents").clicked() {
                            self.custom_dir_path = format!("{}/Documents", home);
                        }
                        if ui.small_button("Pictures").clicked() {
                            self.custom_dir_path = format!("{}/Pictures", home);
                        }
                        if ui.small_button("Videos").clicked() {
                            self.custom_dir_path = format!("{}/Videos", home);
                        }
                        if ui.small_button("Root (/)").clicked() {
                            self.custom_dir_path = "/".to_string();
                        }
                    });

                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            close_picker = true;
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.add(egui::Button::new(RichText::new("Choose Folder & Analyze →").color(Color32::WHITE).strong()).fill(t.accent)).clicked() {
                                chosen_path = Some(PathBuf::from(self.custom_dir_path.trim()));
                                close_picker = true;
                            }
                        });
                    });
                });

            if let Some(p) = chosen_path {
                on_select_custom_path(p);
            }

            if close_picker {
                self.is_dir_picker_open = false;
            }
        }
    }
}
