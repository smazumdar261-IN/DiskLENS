use crate::app::ActiveTab;
use crate::models::FileNode;
use crate::theme::{truncate_middle, AppTheme};
use eframe::egui::{self, Color32, RichText, Ui, Vec2};
use std::path::PathBuf;

pub struct HeaderProps<'a> {
    pub active_tab: &'a mut ActiveTab,
    pub is_scanning: bool,
    pub scanned_root: &'a Option<Box<FileNode>>,
    pub dark_mode: bool,
    pub search_query: &'a mut String,
    pub one_filesystem: bool,
    pub permission_errors: usize,
    pub on_theme_change: &'a mut dyn FnMut(bool),
    pub on_refresh_storage: &'a mut dyn FnMut(),
    pub on_rescan: &'a mut dyn FnMut(PathBuf),
    pub on_toggle_one_fs: &'a mut dyn FnMut(),
    pub on_elevate_pkexec: &'a mut dyn FnMut(),
    pub on_export: &'a mut dyn FnMut(bool), // true = json, false = csv
    pub on_open_mounts: &'a mut dyn FnMut(),
    pub on_open_logs: &'a mut dyn FnMut(),
    pub on_open_docs: &'a mut dyn FnMut(),
    pub on_open_dir_picker: &'a mut dyn FnMut(),
    pub on_open_settings: &'a mut dyn FnMut(),
    pub on_open_command_palette: &'a mut dyn FnMut(),
}

pub fn render_header(ui: &mut Ui, props: HeaderProps) {
    let t = AppTheme::tokens(props.dark_mode);
    let available_w = ui.available_width();
    let is_compact = available_w < 720.0;

    ui.vertical(|ui| {
        // 1. Linux CSD Window Header Strip
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(6.0, 0.0);

            // Rust Crab Icon & Brand Name
            if ui.add(
                egui::Button::new(RichText::new("🦀 DiskLens").size(12.0).strong().color(t.text))
                    .fill(Color32::TRANSPARENT)
                    .frame(false)
            ).on_hover_text("Open Rust Engine & Architecture Inspector").clicked() {
                *props.active_tab = ActiveTab::RustEngine;
            }

            ui.label(RichText::new("|").size(11.0).color(t.border));

            if !is_compact {
                let badge_btn = ui.add(
                    egui::Button::new(RichText::new("Rust 1.82 · io_uring").size(10.0).monospace().color(t.success))
                        .fill(t.bg_elevated)
                        .stroke(egui::Stroke::new(1.0_f32, t.border))
                        .rounding(3.0_f32)
                ).on_hover_text("High-performance safe Rust immediate-mode storage suite");

                if badge_btn.clicked() {
                    *props.active_tab = ActiveTab::RustEngine;
                }

                if available_w > 900.0 {
                    ui.label(
                        RichText::new("Rayon: 16t · 0-alloc · Wayland/wgpu")
                            .size(10.0)
                            .monospace()
                            .color(t.text_muted)
                    );
                }
            }

            // Center Target Info
            if let Some(root) = props.scanned_root {
                let loc_name = root.path.display().to_string();
                let short_loc = truncate_middle(&loc_name, if available_w > 900.0 { 36 } else { 18 });
                ui.add(
                    egui::Label::new(RichText::new(format!("— {}", short_loc)).size(10.5).color(t.text_muted))
                        .truncate(),
                ).on_hover_text(&loc_name);
            } else if props.is_scanning {
                ui.add(
                    egui::Label::new(RichText::new("⚡ Rayon crawler indexing filesystem...").size(10.5).color(t.warning).strong())
                        .truncate(),
                );
            }

            // Right-hand CSD Controls
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button(RichText::new("Rust CLI").size(10.0).monospace().color(t.accent)).on_hover_text("Open Rust Terminal & Engine Shell").clicked() {
                    *props.active_tab = ActiveTab::RustEngine;
                }
            });
        });

        ui.add_space(3.0);
        ui.separator();
        ui.add_space(3.0);

        // 2. Top Application Bar (Brand, Breadcrumbs, Search, Actions)
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 0.0);

            // Storage Home shortcut
            let home_btn = ui.add(
                egui::Button::new(RichText::new("◆ Storage Home").size(12.5).strong().color(if *props.active_tab == ActiveTab::Storage { t.accent } else { t.text }))
                    .fill(if *props.active_tab == ActiveTab::Storage { t.bg_elevated } else { Color32::TRANSPARENT })
                    .stroke(if *props.active_tab == ActiveTab::Storage { egui::Stroke::new(1.0_f32, t.accent) } else { egui::Stroke::NONE })
                    .rounding(4.0_f32)
            ).on_hover_text("Return to Drives and Storage Locations");

            if home_btn.clicked() {
                *props.active_tab = ActiveTab::Storage;
            }

            // Breadcrumb location when scanned
            if let Some(root) = props.scanned_root {
                if !is_compact {
                    ui.label(RichText::new("/").size(12.0).color(t.text_muted));
                    let root_name = root.name.clone();
                    let display_name = if root_name.is_empty() { root.path.display().to_string() } else { root_name };
                    let short_name = truncate_middle(&display_name, if available_w > 900.0 { 22 } else { 12 });
                    ui.add(
                        egui::Label::new(RichText::new(short_name).size(12.0).color(t.accent).strong())
                            .truncate(),
                    ).on_hover_text(&display_name);
                    ui.label(RichText::new(format!("({})", root.formatted_size())).size(10.5).monospace().color(t.text_muted));
                }
            }

            // Right-aligned Action Controls
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // More Options Menu
                ui.menu_button("⋮", |ui| {
                    ui.set_min_width(200.0);

                    ui.label(RichText::new("EXPORT & REPORTS").size(9.5).color(t.text_muted).strong());
                    if ui.button("📊 Export Space Hogs (CSV)").clicked() {
                        (props.on_export)(false);
                        ui.close_menu();
                    }
                    if ui.button("📄 Export Scan Report (JSON)").clicked() {
                        (props.on_export)(true);
                        ui.close_menu();
                    }

                    ui.separator();
                    ui.label(RichText::new("LINUX SYSTEM TOOLS").size(9.5).color(t.text_muted).strong());
                    if ui.button("📁 Scan Custom Folder...").clicked() {
                        (props.on_open_dir_picker)();
                        ui.close_menu();
                    }
                    if ui.button("💿 Mount Points & Btrfs").clicked() {
                        *props.active_tab = ActiveTab::MountPoints;
                        ui.close_menu();
                    }
                    if ui.button("🩺 S.M.A.R.T. Health Diagnostics").clicked() {
                        *props.active_tab = ActiveTab::DiagnosticsSmart;
                        ui.close_menu();
                    }
                    if ui.button("📜 Storage Event Logs").clicked() {
                        *props.active_tab = ActiveTab::SystemLogs;
                        ui.close_menu();
                    }
                    if ui.button("⚙ Engine Settings...").clicked() {
                        (props.on_open_settings)();
                        ui.close_menu();
                    }
                    if ui.button("📖 DiskLens Guide & Docs").clicked() {
                        *props.active_tab = ActiveTab::Documentation;
                        ui.close_menu();
                    }
                });

                // Settings Button
                if ui.small_button("⚙").on_hover_text("Open Engine Settings (,)").clicked() {
                    (props.on_open_settings)();
                }

                // Theme Toggle
                let theme_icon = if props.dark_mode { "☀" } else { "🌙" };
                let theme_tip = if props.dark_mode { "Switch to Light Theme (T)" } else { "Switch to Dark Theme (T)" };
                if ui.small_button(theme_icon).on_hover_text(theme_tip).clicked() {
                    (props.on_theme_change)(!props.dark_mode);
                }

                // Linux Mount Boundary Control: -xdev / One Filesystem toggle
                let xdev_label = if props.one_filesystem { "⛉ -xdev: ON" } else { "⛉ -xdev: OFF" };
                let xdev_col = if props.one_filesystem { t.success } else { t.text_muted };
                let xdev_bg = if props.one_filesystem { t.success.gamma_multiply(0.15) } else { t.bg_surface };
                let xdev_btn = ui.add(
                    egui::Button::new(RichText::new(xdev_label).size(10.0).monospace().color(xdev_col))
                        .fill(xdev_bg)
                        .stroke(egui::Stroke::new(1.0_f32, if props.one_filesystem { t.success } else { t.border }))
                        .rounding(3.0_f32),
                ).on_hover_text("Mount Boundary Control (-xdev): When ON, restricts scan traversal to the same device/filesystem. Click to toggle.");
                if xdev_btn.clicked() {
                    (props.on_toggle_one_fs)();
                }

                // Permission Warning & pkexec Elevation Badge
                if props.permission_errors > 0 {
                    let perm_btn = ui.add(
                        egui::Button::new(
                            RichText::new(format!("⚠️ {} Restricted", props.permission_errors))
                                .size(10.0)
                                .monospace()
                                .color(t.warning),
                        )
                        .fill(t.warning.gamma_multiply(0.18))
                        .stroke(egui::Stroke::new(1.0_f32, t.warning))
                        .rounding(3.0_f32),
                    ).on_hover_text("Directories blocked due to standard permissions (EACCES). Click to elevate via pkexec.");
                    if perm_btn.clicked() {
                        (props.on_elevate_pkexec)();
                    }
                }

                // Rescan Button
                if let Some(root) = props.scanned_root {
                    let rescan_btn = ui.add_enabled(
                        !props.is_scanning,
                        egui::Button::new(RichText::new("🔄 Rescan").size(11.5))
                    ).on_hover_text("Rescan current storage target (R)");

                    if rescan_btn.clicked() {
                        (props.on_rescan)(root.path.clone());
                    }
                } else {
                    let refresh_btn = ui.small_button("🔄 Refresh").on_hover_text("Refresh storage mounts");
                    if refresh_btn.clicked() {
                        (props.on_refresh_storage)();
                    }
                }

                // Command Palette (Ctrl+K) trigger button
                let cmd_btn = ui.add(
                    egui::Button::new(
                        RichText::new("🔍 Search & Commands (Ctrl+K)")
                            .size(11.0)
                            .color(t.text_secondary),
                    )
                    .fill(t.bg_input)
                    .stroke(egui::Stroke::new(1.0_f32, t.border))
                    .rounding(4.0_f32),
                );
                if cmd_btn.clicked() {
                    (props.on_open_command_palette)();
                }
            });
        });
    });
}
