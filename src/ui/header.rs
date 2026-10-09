use crate::app::ActiveTab;
use crate::models::FileNode;
use crate::theme::{truncate_middle, AppTheme};
use eframe::egui::{self, Color32, RichText, Ui, Vec2};
use std::path::PathBuf;

#[allow(dead_code)]
pub struct HeaderProps<'a> {
    pub active_tab: &'a mut ActiveTab,
    pub is_scanning: bool,
    pub scanned_root: &'a Option<Box<FileNode>>,
    pub dark_mode: bool,
    pub search_query: &'a mut String,
    pub one_filesystem: bool,
    pub permission_errors: usize,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub on_go_back: &'a mut dyn FnMut(),
    pub on_go_forward: &'a mut dyn FnMut(),
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
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 0.0);

            // Clean Brand Name
            ui.add(
                egui::Label::new(
                    RichText::new("DiskLens")
                        .size(13.0)
                        .strong()
                        .color(t.text),
                )
            );

            // Target location or indexing status
            if let Some(root) = props.scanned_root {
                let loc_name = root.path.display().to_string();
                let short_loc = truncate_middle(&loc_name, if available_w > 900.0 { 40 } else { 20 });
                ui.label(RichText::new("|").size(11.0).color(t.border));
                ui.add(
                    egui::Label::new(RichText::new(short_loc).size(11.0).color(t.text_muted))
                        .truncate(),
                ).on_hover_text(&loc_name);
            } else if props.is_scanning {
                ui.label(RichText::new("|").size(11.0).color(t.border));
                ui.add(
                    egui::Label::new(RichText::new("⚡ Indexing filesystem...").size(11.0).color(t.warning).strong())
                        .truncate(),
                );
            }
        });

        ui.add_space(3.0);
        ui.separator();
        ui.add_space(3.0);

        // 2. Top Application Bar (Brand, Breadcrumbs, Search, Actions)
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(6.0, 0.0);

            // Universal Back & Forward Navigation Controls
            let back_text = RichText::new("◀")
                .size(11.5)
                .color(if props.can_go_back { t.text } else { t.text_muted.gamma_multiply(0.35) });
            let back_btn = egui::Button::new(back_text)
                .fill(if props.can_go_back { t.bg_elevated } else { Color32::TRANSPARENT })
                .stroke(if props.can_go_back { egui::Stroke::new(1.0_f32, t.border) } else { egui::Stroke::NONE })
                .rounding(4.0_f32)
                .min_size(Vec2::new(24.0, 22.0));
            if ui.add_enabled(props.can_go_back, back_btn).on_hover_text("Navigate Back (Alt+Left)").clicked() {
                (props.on_go_back)();
            }

            let fwd_text = RichText::new("▶")
                .size(11.5)
                .color(if props.can_go_forward { t.text } else { t.text_muted.gamma_multiply(0.35) });
            let fwd_btn = egui::Button::new(fwd_text)
                .fill(if props.can_go_forward { t.bg_elevated } else { Color32::TRANSPARENT })
                .stroke(if props.can_go_forward { egui::Stroke::new(1.0_f32, t.border) } else { egui::Stroke::NONE })
                .rounding(4.0_f32)
                .min_size(Vec2::new(24.0, 22.0));
            if ui.add_enabled(props.can_go_forward, fwd_btn).on_hover_text("Navigate Forward (Alt+Right)").clicked() {
                (props.on_go_forward)();
            }

            ui.separator();

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
                // 1. Smart More Options Menu (custom vector-painted 3 dots, never tofu)
                let menu_resp = ui.menu_button("   ", |ui| {
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
                    ui.label(RichText::new("ACTIONS & TOOLS").size(9.5).color(t.text_muted).strong());
                    if ui.button("📁 Scan Custom Folder...").clicked() {
                        (props.on_open_dir_picker)();
                        ui.close_menu();
                    }
                    if ui.button("⚙ Engine Settings...").clicked() {
                        (props.on_open_settings)();
                        ui.close_menu();
                    }
                    if ui.button("ℹ About DiskLens...").clicked() {
                        (props.on_open_docs)();
                        ui.close_menu();
                    }
                });
                let m_rect = menu_resp.response.rect;
                let is_m_hover = menu_resp.response.hovered();
                let dot_col = if is_m_hover { t.accent } else { t.text };
                let cx = m_rect.center().x;
                let cy = m_rect.center().y;
                ui.painter().circle_filled(egui::Pos2::new(cx, cy - 4.5), 1.6, dot_col);
                ui.painter().circle_filled(egui::Pos2::new(cx, cy), 1.6, dot_col);
                ui.painter().circle_filled(egui::Pos2::new(cx, cy + 4.5), 1.6, dot_col);
                menu_resp.response.on_hover_text("More actions, settings and about DiskLens");

                // 3. Smart Theme Toggle Button (☀ in dark mode, ☾ in light mode)
                let theme_icon = if props.dark_mode { "☀" } else { "☾" };
                let theme_tip = if props.dark_mode { "Switch to Light Theme (T)" } else { "Switch to Dark Theme (T)" };
                let theme_btn = egui::Button::new(RichText::new(theme_icon).size(12.0).color(t.text))
                    .fill(t.bg_elevated)
                    .stroke(egui::Stroke::new(1.0_f32, t.border))
                    .rounding(4.0_f32)
                    .min_size(Vec2::new(26.0, 22.0));
                if ui.add(theme_btn).on_hover_text(theme_tip).clicked() {
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
