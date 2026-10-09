use crate::app::ActiveTab;
use crate::theme::AppTheme;
use eframe::egui::{self, Align2, Color32, CursorIcon, RichText, Sense, Vec2, Window};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CommandPaletteItem {
    pub id: String,
    pub title: String,
    pub category: String,
    pub icon: String,
    pub shortcut: Option<String>,
    pub target_tab: Option<ActiveTab>,
}

#[derive(Default)]
pub struct CommandPaletteState {
    pub is_open: bool,
    pub query: String,
    pub selected_index: usize,
}

pub enum CommandAction {
    SwitchTab(ActiveTab),
    StartScan(PathBuf),
    OpenDirPicker,
    OpenSettings,
    OpenDocs,
    OpenMounts,
    OpenLogs,
    TriggerDryRunClean,
    ToggleTheme,
}

pub fn render_command_palette(
    ctx: &egui::Context,
    state: &mut CommandPaletteState,
    dark_mode: bool,
    is_view_available: &dyn Fn(ActiveTab) -> bool,
    on_action: &mut dyn FnMut(CommandAction),
) {
    if !state.is_open {
        return;
    }

    let t = AppTheme::tokens(dark_mode);
    let mut close_palette = false;
    let mut chosen_action = None;

    let items = [
        CommandPaletteItem {
            id: "scan-home".to_string(),
            title: "Run fast storage scan on Home directory (~/)".to_string(),
            category: "Scan Actions".to_string(),
            icon: "⚡".to_string(),
            shortcut: Some("S".to_string()),
            target_tab: None,
        },
        CommandPaletteItem {
            id: "scan-root".to_string(),
            title: "Run fast storage scan on Linux root filesystem (/)".to_string(),
            category: "Scan Actions".to_string(),
            icon: "🖥".to_string(),
            shortcut: None,
            target_tab: None,
        },
        CommandPaletteItem {
            id: "browse-folder".to_string(),
            title: "Browse directory path to analyze...".to_string(),
            category: "Scan Actions".to_string(),
            icon: "📁".to_string(),
            shortcut: None,
            target_tab: None,
        },
        CommandPaletteItem {
            id: "nav-storage".to_string(),
            title: "Open Storage Home & Mounted Drives".to_string(),
            category: "Analysis Views".to_string(),
            icon: "🏠".to_string(),
            shortcut: Some("Home".to_string()),
            target_tab: Some(ActiveTab::Storage),
        },
        CommandPaletteItem {
            id: "nav-overview".to_string(),
            title: "Open Storage Overview & Allocation Dashboard".to_string(),
            category: "Analysis Views".to_string(),
            icon: "📊".to_string(),
            shortcut: Some("1".to_string()),
            target_tab: Some(ActiveTab::Overview),
        },
        CommandPaletteItem {
            id: "nav-treemap".to_string(),
            title: "Open Interactive Treemap Visualizer".to_string(),
            category: "Analysis Views".to_string(),
            icon: "⊞".to_string(),
            shortcut: Some("2".to_string()),
            target_tab: Some(ActiveTab::Treemap),
        },
        CommandPaletteItem {
            id: "nav-tree".to_string(),
            title: "Open Hierarchical Folder Tree Explorer".to_string(),
            category: "Analysis Views".to_string(),
            icon: "📁".to_string(),
            shortcut: Some("3".to_string()),
            target_tab: Some(ActiveTab::DirectoryTree),
        },
        CommandPaletteItem {
            id: "nav-hogs".to_string(),
            title: "Inspect Space Hogs Leaderboard (Largest Files)".to_string(),
            category: "Analysis Views".to_string(),
            icon: "⚠".to_string(),
            shortcut: Some("4".to_string()),
            target_tab: Some(ActiveTab::LargestFiles),
        },
        CommandPaletteItem {
            id: "nav-categories".to_string(),
            title: "Analyze File Types & MIME Distribution".to_string(),
            category: "Analysis Views".to_string(),
            icon: "▤".to_string(),
            shortcut: Some("5".to_string()),
            target_tab: Some(ActiveTab::Categories),
        },
        CommandPaletteItem {
            id: "nav-duplicates".to_string(),
            title: "Find and deduplicate identical files (SHA-256)".to_string(),
            category: "Optimization".to_string(),
            icon: "📋".to_string(),
            shortcut: Some("6".to_string()),
            target_tab: Some(ActiveTab::Duplicates),
        },
        CommandPaletteItem {
            id: "nav-safe-trash".to_string(),
            title: "Safe Trash & Storage Reclamation (Caches, Docker, Logs)".to_string(),
            category: "Optimization".to_string(),
            icon: "🗑".to_string(),
            shortcut: Some("C".to_string()),
            target_tab: Some(ActiveTab::SafeTrash),
        },
        CommandPaletteItem {
            id: "clean-dry-run".to_string(),
            title: "Simulate Safe Cleanup Dry-Run".to_string(),
            category: "Optimization".to_string(),
            icon: "🛡".to_string(),
            shortcut: None,
            target_tab: None,
        },
        CommandPaletteItem {
            id: "nav-history".to_string(),
            title: "View Storage Growth Trends & History Snapshots".to_string(),
            category: "Optimization".to_string(),
            icon: "📈".to_string(),
            shortcut: Some("7".to_string()),
            target_tab: Some(ActiveTab::History),
        },
        CommandPaletteItem {
            id: "nav-mounts".to_string(),
            title: "Inspect Active Linux Mount Points (/proc/mounts)".to_string(),
            category: "System & Hardware".to_string(),
            icon: "🖴".to_string(),
            shortcut: Some("M".to_string()),
            target_tab: Some(ActiveTab::MountPoints),
        },
        CommandPaletteItem {
            id: "nav-smart".to_string(),
            title: "View NVMe / SATA S.M.A.R.T. Hardware Health".to_string(),
            category: "System & Hardware".to_string(),
            icon: "🩺".to_string(),
            shortcut: Some("H".to_string()),
            target_tab: Some(ActiveTab::DiagnosticsSmart),
        },
        CommandPaletteItem {
            id: "nav-logs".to_string(),
            title: "View DiskLens Telemetry & Storage Event Logs".to_string(),
            category: "System & Hardware".to_string(),
            icon: "📜".to_string(),
            shortcut: Some("L".to_string()),
            target_tab: Some(ActiveTab::SystemLogs),
        },
        CommandPaletteItem {
            id: "nav-engine".to_string(),
            title: "Open Rust Core Engine Architecture & Interactive CLI".to_string(),
            category: "Engine & System".to_string(),
            icon: "🦀".to_string(),
            shortcut: Some("8".to_string()),
            target_tab: Some(ActiveTab::RustEngine),
        },
        CommandPaletteItem {
            id: "settings".to_string(),
            title: "Open Application Preferences & Safety Policies".to_string(),
            category: "Engine & System".to_string(),
            icon: "⚙".to_string(),
            shortcut: Some(",".to_string()),
            target_tab: None,
        },
        CommandPaletteItem {
            id: "toggle-theme".to_string(),
            title: "Toggle Dark / Tokyo Night / Light Theme".to_string(),
            category: "Engine & System".to_string(),
            icon: "🌓".to_string(),
            shortcut: Some("T".to_string()),
            target_tab: None,
        },
        CommandPaletteItem {
            id: "docs".to_string(),
            title: "About DiskLens — Architecture, Telemetry & Shortcuts".to_string(),
            category: "Engine & System".to_string(),
            icon: "ℹ".to_string(),
            shortcut: Some("D".to_string()),
            target_tab: Some(ActiveTab::Documentation),
        },
    ];

    let query_lower = state.query.to_lowercase();
    let filtered_items: Vec<&CommandPaletteItem> = items
        .iter()
        .filter(|item| {
            if query_lower.is_empty() {
                true
            } else {
                item.title.to_lowercase().contains(&query_lower)
                    || item.category.to_lowercase().contains(&query_lower)
            }
        })
        .collect();

    // Keyboard navigation (Up/Down/Enter/Esc)
    if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
        if !filtered_items.is_empty() {
            state.selected_index = (state.selected_index + 1) % filtered_items.len();
        }
    }
    if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
        if !filtered_items.is_empty() {
            state.selected_index = if state.selected_index == 0 {
                filtered_items.len() - 1
            } else {
                state.selected_index - 1
            };
        }
    }
    if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
        if let Some(item) = filtered_items.get(state.selected_index) {
            let is_available = match item.target_tab {
                Some(tab) => is_view_available(tab),
                None => true,
            };

            if is_available {
                match item.id.as_str() {
                    "scan-home" => {
                        let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
                        chosen_action = Some(CommandAction::StartScan(PathBuf::from(home)));
                    }
                    "scan-root" => {
                        chosen_action = Some(CommandAction::StartScan(PathBuf::from("/")));
                    }
                    "browse-folder" => {
                        chosen_action = Some(CommandAction::OpenDirPicker);
                    }
                    "nav-storage" => {
                        chosen_action = Some(CommandAction::SwitchTab(ActiveTab::Storage))
                    }
                    "nav-overview" => {
                        chosen_action = Some(CommandAction::SwitchTab(ActiveTab::Overview))
                    }
                    "nav-treemap" => {
                        chosen_action = Some(CommandAction::SwitchTab(ActiveTab::Treemap))
                    }
                    "nav-tree" => {
                        chosen_action = Some(CommandAction::SwitchTab(ActiveTab::DirectoryTree))
                    }
                    "nav-hogs" => {
                        chosen_action = Some(CommandAction::SwitchTab(ActiveTab::LargestFiles))
                    }
                    "nav-categories" => {
                        chosen_action = Some(CommandAction::SwitchTab(ActiveTab::Categories))
                    }
                    "nav-duplicates" => {
                        chosen_action = Some(CommandAction::SwitchTab(ActiveTab::Duplicates))
                    }
                    "nav-safe-trash" => {
                        chosen_action = Some(CommandAction::SwitchTab(ActiveTab::SafeTrash))
                    }
                    "clean-dry-run" => chosen_action = Some(CommandAction::TriggerDryRunClean),
                    "nav-history" => {
                        chosen_action = Some(CommandAction::SwitchTab(ActiveTab::History))
                    }
                    "nav-mounts" => chosen_action = Some(CommandAction::OpenMounts),
                    "nav-smart" => {
                        chosen_action = Some(CommandAction::SwitchTab(ActiveTab::DiagnosticsSmart))
                    }
                    "nav-logs" => chosen_action = Some(CommandAction::OpenLogs),
                    "nav-engine" => {
                        chosen_action = Some(CommandAction::SwitchTab(ActiveTab::RustEngine))
                    }
                    "settings" => chosen_action = Some(CommandAction::OpenSettings),
                    "toggle-theme" => chosen_action = Some(CommandAction::ToggleTheme),
                    "docs" => chosen_action = Some(CommandAction::OpenDocs),
                    _ => {}
                }
                close_palette = true;
            }
        }
    }
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        close_palette = true;
    }

    let screen_size = ctx.screen_rect().size();
    let modal_width = 580.0_f32.min(screen_size.x - 32.0);
    let modal_height = 420.0_f32.min(screen_size.y - 48.0);

    Window::new("Command Dispatcher (Ctrl+K)")
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .collapsible(false)
        .resizable(false)
        .fixed_size(Vec2::new(modal_width, modal_height))
        .show(ctx, |ui| {
            // 1. Search Box Header
            ui.horizontal(|ui| {
                ui.label(RichText::new("🔍").size(15.0).color(t.accent));
                let edit_resp = ui.add(
                    egui::TextEdit::singleline(&mut state.query)
                        .hint_text("Type a command or jump to view (e.g. 'scan', 'treemap', 'dupes', 'smart')...")
                        .desired_width(modal_width - 80.0),
                );
                edit_resp.request_focus();

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✕").clicked() {
                        close_palette = true;
                    }
                });
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            // 2. Action Items List
            egui::ScrollArea::vertical()
                .max_height(modal_height - 90.0)
                .show(ui, |ui| {
                    if filtered_items.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(40.0);
                            ui.label(RichText::new("No matching storage tools or actions found.").size(12.0).color(t.text_muted));
                        });
                    } else {
                        for (idx, item) in filtered_items.iter().enumerate() {
                            let is_available = match item.target_tab {
                                Some(tab) => is_view_available(tab),
                                None => true,
                            };
                            let is_selected = idx == state.selected_index;

                            if is_available {
                                let (row_rect, row_resp) = ui.allocate_exact_size(
                                    Vec2::new(ui.available_width(), 32.0),
                                    Sense::click(),
                                );

                                let is_hovered = row_resp.hovered();
                                let bg = if is_selected {
                                    Color32::from_rgb(62, 69, 101)
                                } else if is_hovered {
                                    t.bg_elevated
                                } else {
                                    Color32::TRANSPARENT
                                };

                                ui.painter().rect_filled(row_rect, 4.0_f32, bg);
                                if is_selected {
                                    ui.painter().rect_stroke(row_rect, 4.0_f32, egui::Stroke::new(1.0_f32, t.accent));
                                }

                                // Icon
                                ui.painter().text(
                                    egui::pos2(row_rect.min.x + 12.0, row_rect.center().y),
                                    egui::Align2::LEFT_CENTER,
                                    &item.icon,
                                    egui::FontId::proportional(14.0),
                                    if is_selected { t.accent } else { t.text },
                                );

                                // Title
                                ui.painter().text(
                                    egui::pos2(row_rect.min.x + 36.0, row_rect.center().y),
                                    egui::Align2::LEFT_CENTER,
                                    &item.title,
                                    egui::FontId::proportional(12.0),
                                    if is_selected { Color32::WHITE } else { t.text },
                                );

                                // Category / Shortcut on Right
                                if let Some(sc) = &item.shortcut {
                                    ui.painter().text(
                                        egui::pos2(row_rect.max.x - 10.0, row_rect.center().y),
                                        egui::Align2::RIGHT_CENTER,
                                        sc,
                                        egui::FontId::monospace(10.5),
                                        t.text_muted,
                                    );
                                } else {
                                    ui.painter().text(
                                        egui::pos2(row_rect.max.x - 10.0, row_rect.center().y),
                                        egui::Align2::RIGHT_CENTER,
                                        &item.category,
                                        egui::FontId::proportional(10.0),
                                        t.text_muted,
                                    );
                                }

                                if row_resp.clicked() {
                                    state.selected_index = idx;
                                    match item.id.as_str() {
                                        "scan-home" => {
                                            let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
                                            chosen_action = Some(CommandAction::StartScan(PathBuf::from(home)));
                                        }
                                        "scan-root" => {
                                            chosen_action = Some(CommandAction::StartScan(PathBuf::from("/")));
                                        }
                                        "browse-folder" => {
                                            chosen_action = Some(CommandAction::OpenDirPicker);
                                        }
                                        "nav-storage" => chosen_action = Some(CommandAction::SwitchTab(ActiveTab::Storage)),
                                        "nav-overview" => chosen_action = Some(CommandAction::SwitchTab(ActiveTab::Overview)),
                                        "nav-treemap" => chosen_action = Some(CommandAction::SwitchTab(ActiveTab::Treemap)),
                                        "nav-tree" => chosen_action = Some(CommandAction::SwitchTab(ActiveTab::DirectoryTree)),
                                        "nav-hogs" => chosen_action = Some(CommandAction::SwitchTab(ActiveTab::LargestFiles)),
                                        "nav-categories" => chosen_action = Some(CommandAction::SwitchTab(ActiveTab::Categories)),
                                        "nav-duplicates" => chosen_action = Some(CommandAction::SwitchTab(ActiveTab::Duplicates)),
                                        "nav-safe-trash" => chosen_action = Some(CommandAction::SwitchTab(ActiveTab::SafeTrash)),
                                        "clean-dry-run" => chosen_action = Some(CommandAction::TriggerDryRunClean),
                                        "nav-history" => chosen_action = Some(CommandAction::SwitchTab(ActiveTab::History)),
                                        "nav-mounts" => chosen_action = Some(CommandAction::OpenMounts),
                                        "nav-smart" => chosen_action = Some(CommandAction::SwitchTab(ActiveTab::DiagnosticsSmart)),
                                        "nav-logs" => chosen_action = Some(CommandAction::OpenLogs),
                                        "nav-engine" => chosen_action = Some(CommandAction::SwitchTab(ActiveTab::RustEngine)),
                                        "settings" => chosen_action = Some(CommandAction::OpenSettings),
                                        "toggle-theme" => chosen_action = Some(CommandAction::ToggleTheme),
                                        "docs" => chosen_action = Some(CommandAction::OpenDocs),
                                        _ => {}
                                    }
                                    close_palette = true;
                                }
                            } else {
                                // Disabled item in Command Palette
                                let (row_rect, row_resp) = ui.allocate_exact_size(
                                    Vec2::new(ui.available_width(), 32.0),
                                    Sense::hover(),
                                );
                                let _ = row_resp
                                    .on_hover_cursor(CursorIcon::NotAllowed)
                                    .on_hover_text("Select a folder or scan a storage target first");

                                let disabled_text = Color32::from_rgba_unmultiplied(120, 125, 145, 120);

                                // Icon muted
                                ui.painter().text(
                                    egui::pos2(row_rect.min.x + 12.0, row_rect.center().y),
                                    egui::Align2::LEFT_CENTER,
                                    &item.icon,
                                    egui::FontId::proportional(14.0),
                                    disabled_text,
                                );

                                // Title muted
                                ui.painter().text(
                                    egui::pos2(row_rect.min.x + 36.0, row_rect.center().y),
                                    egui::Align2::LEFT_CENTER,
                                    &item.title,
                                    egui::FontId::proportional(12.0),
                                    disabled_text,
                                );

                                // Right badge indicating disabled reason
                                ui.painter().text(
                                    egui::pos2(row_rect.max.x - 10.0, row_rect.center().y),
                                    egui::Align2::RIGHT_CENTER,
                                    "[Requires Scan]",
                                    egui::FontId::monospace(10.0),
                                    Color32::from_rgba_unmultiplied(180, 120, 120, 160),
                                );
                            }

                            ui.add_space(2.0);
                        }
                    }
                });

            // 3. Footer shortcuts hint
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Navigate: ↑ ↓  •  Select: ↵  •  Close: Esc").size(10.5).monospace().color(t.text_muted));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new("DiskLens Command Dispatcher").size(10.0).color(t.accent));
                });
            });
        });

    if let Some(action) = chosen_action {
        on_action(action);
    }

    if close_palette {
        state.is_open = false;
        state.query.clear();
        state.selected_index = 0;
    }
}
