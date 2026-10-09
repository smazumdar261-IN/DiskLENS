use crate::settings::{AppSettings, SymlinkPolicy, ThemePreference};
use crate::theme::AppTheme;
use crate::ui::modals::ModalManager;
use eframe::egui::{self, Color32, Margin, RichText, Ui, Vec2};

pub struct SettingsModalState {
    pub is_open: bool,
    pub active_tab: usize, // 0 = Scanning, 1 = Appearance, 2 = Safety, 3 = Performance, 4 = Advanced
    pub settings: AppSettings,
    pub new_exclusion_path: String,
    pub show_add_exclusion: bool,
}

impl Default for SettingsModalState {
    fn default() -> Self {
        Self {
            is_open: false,
            active_tab: 0,
            settings: AppSettings::load(),
            new_exclusion_path: String::new(),
            show_add_exclusion: false,
        }
    }
}

pub fn render_settings_modal(
    ctx: &egui::Context,
    state: &mut SettingsModalState,
    modals: &mut ModalManager,
    dark_mode: &mut bool,
) {
    if !state.is_open {
        return;
    }

    let t = AppTheme::tokens(*dark_mode);
    let screen_rect = ctx.screen_rect();
    let modal_w = (screen_rect.width() - 32.0).clamp(320.0, 720.0);
    let modal_h = (screen_rect.height() - 48.0).clamp(380.0, 560.0);
    let mut is_open = state.is_open;

    egui::Window::new("⚙ Settings")
        .open(&mut is_open)
        .collapsible(false)
        .resizable(false)
        .fixed_size(Vec2::new(modal_w, modal_h))
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(t.bg_surface)
                .stroke(egui::Stroke::new(1.0_f32, t.border))
                .inner_margin(Margin::same(0.0)),
        )
        .show(ctx, |ui| {
            ui.set_width(modal_w);
            ui.set_height(modal_h);

            // 1. Modal Top Header (Matches SettingsModal.tsx)
            egui::Frame::none()
                .fill(t.bg_elevated)
                .inner_margin(Margin::symmetric(14.0, 10.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("⚙").size(16.0).color(t.accent));
                        ui.label(
                            RichText::new("DiskLens Engine Settings")
                                .size(15.0)
                                .strong()
                                .color(t.text),
                        );
                    });
                });

            ui.separator();

            // 2. Modal Body: Responsive Left Sidebar + Vertical Content Area
            let is_narrow = modal_w < 580.0;
            let sidebar_w = if is_narrow { 120.0 } else { 160.0 };

            ui.horizontal(|ui| {
                // Left Tabs Sidebar
                egui::Frame::none()
                    .fill(t.bg_surface)
                    .inner_margin(Margin::same(8.0))
                    .show(ui, |ui| {
                        ui.set_width(sidebar_w);
                        ui.set_min_height(modal_h - 110.0);

                        let tabs = [
                            ("Scanning", "📡"),
                            ("Appearance", "🎨"),
                            ("Safety", "🛡"),
                            ("Performance", "⚡"),
                            ("Advanced", "🔧"),
                        ];

                        ui.vertical(|ui| {
                            for (idx, (name, icon)) in tabs.iter().enumerate() {
                                let is_active = state.active_tab == idx;
                                let label_str = if is_narrow {
                                    format!("{} {}", icon, name)
                                } else {
                                    format!("{}  {}", icon, name)
                                };

                                let btn = egui::Button::new(
                                    RichText::new(label_str).size(12.0).strong().color(
                                        if is_active {
                                            t.accent
                                        } else {
                                            t.text_secondary
                                        },
                                    ),
                                )
                                .fill(if is_active {
                                    t.bg_elevated
                                } else {
                                    Color32::TRANSPARENT
                                })
                                .stroke(if is_active {
                                    egui::Stroke::new(1.0_f32, t.accent)
                                } else {
                                    egui::Stroke::NONE
                                })
                                .rounding(4.0_f32);

                                if ui.add_sized([sidebar_w - 4.0, 32.0], btn).clicked() {
                                    state.active_tab = idx;
                                }
                                ui.add_space(3.0);
                            }
                        });
                    });

                ui.separator();

                // Right Content Pane (Strictly Vertical with Allocated Width)
                ui.vertical(|ui| {
                    let content_w = (ui.available_width() - 16.0).max(200.0);
                    ui.set_width(content_w);

                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .max_height(modal_h - 110.0)
                        .show(ui, |ui| {
                            ui.set_width(content_w);
                            ui.add_space(8.0);
                            render_tab_content(ui, state, dark_mode, &t, content_w);
                            ui.add_space(16.0);
                        });
                });
            });

            ui.separator();

            // 3. Modal Footer Actions (Reset, Cancel, Save)
            egui::Frame::none()
                .fill(t.bg_elevated)
                .inner_margin(Margin::symmetric(14.0, 10.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .button("↺ Reset Defaults")
                            .on_hover_text("Restore default engine and crawler configuration")
                            .clicked()
                        {
                            state.settings.reset_defaults();
                            *dark_mode = true;
                            modals.show_toast("Reset settings to defaults".to_string(), false);
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let save_btn = egui::Button::new(
                                RichText::new("Save Changes").strong().color(Color32::WHITE),
                            )
                            .fill(t.accent)
                            .rounding(4.0_f32);

                            if ui.add_sized([110.0, 28.0], save_btn).clicked() {
                                if let Err(e) = state.settings.save() {
                                    modals
                                        .show_toast(format!("Error saving settings: {}", e), true);
                                } else {
                                    modals.show_toast(
                                        "Settings saved successfully".to_string(),
                                        false,
                                    );
                                    state.is_open = false;
                                }
                            }

                            if ui
                                .add_sized([80.0, 28.0], egui::Button::new("Cancel"))
                                .clicked()
                            {
                                state.is_open = false;
                            }
                        });
                    });
                });
        });

    if !is_open {
        state.is_open = false;
    }
}

fn render_tab_content(
    ui: &mut Ui,
    state: &mut SettingsModalState,
    dark_mode: &mut bool,
    t: &crate::theme::ThemeTokens,
    content_w: f32,
) {
    ui.vertical(|ui| {
        ui.set_width(content_w);

        match state.active_tab {
            0 => {
                // 1. Scanning Configuration
                render_section_card(ui, "Symlink Traversal Policy", content_w, t, |ui| {
                    ui.radio_value(&mut state.settings.symlink_policy, SymlinkPolicy::DontFollow, "Don't follow symlinks (fastest, safest)");
                    ui.add_space(2.0);
                    ui.radio_value(&mut state.settings.symlink_policy, SymlinkPolicy::FollowWithinRoot, "Follow only within scan root directory");
                    ui.add_space(2.0);
                    ui.radio_value(&mut state.settings.symlink_policy, SymlinkPolicy::FollowAll, "Follow all symlinks across filesystem");
                });

                ui.add_space(10.0);

                render_section_card(ui, "Scan Targets & Boundaries", content_w, t, |ui| {
                    ui.checkbox(&mut state.settings.include_hidden_files, "Include Hidden Files (dotfiles like .config, .cache)");
                    ui.add_space(2.0);
                    ui.checkbox(&mut state.settings.cross_filesystem_boundaries, "Cross filesystem boundaries (scan mounted disks inside root)");
                });

                ui.add_space(10.0);

                render_section_card(ui, "Excluded Paths", content_w, t, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Paths skipped during filesystem traversal:").size(11.0).color(t.text_secondary));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("+ Add Path").clicked() {
                                state.show_add_exclusion = !state.show_add_exclusion;
                            }
                        });
                    });

                    if state.show_add_exclusion {
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.add_sized([180.0, 22.0], egui::TextEdit::singleline(&mut state.new_exclusion_path).hint_text("e.g. /var/lib/docker"));
                            if ui.small_button("Add").clicked() {
                                if !state.new_exclusion_path.trim().is_empty() {
                                    state.settings.exclusions.push(state.new_exclusion_path.trim().to_string());
                                    state.new_exclusion_path.clear();
                                    state.show_add_exclusion = false;
                                }
                            }
                            if ui.small_button("Cancel").clicked() {
                                state.show_add_exclusion = false;
                            }
                        });
                    }

                    ui.add_space(6.0);
                    let mut to_remove = None;
                    for (idx, path) in state.settings.exclusions.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("•  {}", path)).size(11.5).monospace().color(t.text));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("🗑").on_hover_text("Remove exclusion").clicked() {
                                    to_remove = Some(idx);
                                }
                            });
                        });
                        ui.add_space(2.0);
                    }
                    if let Some(idx) = to_remove {
                        state.settings.exclusions.remove(idx);
                    }
                });

                ui.add_space(10.0);

                render_section_card(ui, "Rayon Crawler Concurrency", content_w, t, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Worker Threads:").size(11.5).color(t.text));
                        ui.label(
                            RichText::new(format!("{} threads", state.settings.worker_threads))
                                .size(12.0)
                                .strong()
                                .monospace()
                                .color(t.accent),
                        );
                    });
                    ui.add_space(4.0);
                    ui.add(egui::Slider::new(&mut state.settings.worker_threads, 1..=16).text("worker threads"));
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new("Higher thread counts utilize more CPU cores for high-speed indexing across NVMe and SSD drives.")
                            .size(10.5)
                            .color(t.text_muted),
                    );
                });
            }
            1 => {
                // 2. Appearance Configuration
                render_section_card(ui, "Theme & Color Scheme", content_w, t, |ui| {
                    ui.horizontal(|ui| {
                        let is_dark = *dark_mode;
                        let dark_btn = egui::Button::new(RichText::new("🌙  Dark (Tokyo Night)").strong().color(if is_dark { t.accent } else { t.text }))
                            .fill(if is_dark { t.bg_surface } else { Color32::TRANSPARENT })
                            .stroke(if is_dark { egui::Stroke::new(1.0_f32, t.accent) } else { egui::Stroke::new(1.0_f32, t.border) })
                            .rounding(4.0_f32);

                        let light_btn = egui::Button::new(RichText::new("☀  Light (Modern Slate)").strong().color(if !is_dark { t.accent } else { t.text }))
                            .fill(if !is_dark { t.bg_surface } else { Color32::TRANSPARENT })
                            .stroke(if !is_dark { egui::Stroke::new(1.0_f32, t.accent) } else { egui::Stroke::new(1.0_f32, t.border) })
                            .rounding(4.0_f32);

                        if ui.add_sized([160.0, 36.0], dark_btn).clicked() {
                            *dark_mode = true;
                            state.settings.theme_preference = ThemePreference::Dark;
                        }

                        if ui.add_sized([160.0, 36.0], light_btn).clicked() {
                            *dark_mode = false;
                            state.settings.theme_preference = ThemePreference::Light;
                        }
                    });
                });

                ui.add_space(10.0);

                render_section_card(ui, "Typography & Information Density", content_w, t, |ui| {
                    ui.label(
                        RichText::new("Rendered with high-contrast system fonts and Tokyo Night / Modern Slate color tokens for maximum data density across 800x600 to 4K displays.")
                            .size(11.0)
                            .color(t.text_secondary)
                    );
                });
            }
            2 => {
                // 3. Safety Configuration
                render_section_card(ui, "Deletion Safeguards", content_w, t, |ui| {
                    ui.checkbox(&mut state.settings.confirm_before_trash, "Always confirm before moving items to Freedesktop Trash");
                    ui.add_space(4.0);
                    ui.checkbox(&mut state.settings.confirm_before_permanent_delete, "Require double confirmation before permanent removal");
                });

                ui.add_space(10.0);

                render_section_card(ui, "Protected Linux System Hierarchies", content_w, t, |ui| {
                    ui.label(
                        RichText::new("DiskLens strictly guards critical system root paths (/boot, /etc, /usr, /lib, /bin, /sbin) against accidental bulk modification or automated deletion.")
                            .size(11.0)
                            .color(t.text_secondary)
                    );
                });
            }
            3 => {
                // 4. Performance Telemetry
                render_section_card(ui, "Kernel I/O Optimization", content_w, t, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Syscall Strategy:").size(11.0).color(t.text_muted));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new("Linux statx (batching)").size(11.5).strong().color(t.success));
                        });
                    });
                    ui.add_space(3.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Memory Model:").size(11.0).color(t.text_muted));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new("Zero-allocation directory tree").size(11.5).strong().color(t.accent));
                        });
                    });
                    ui.add_space(3.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Analysis Cache:").size(11.0).color(t.text_muted));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new("Precomputed immutable cache").size(11.5).strong().color(t.text));
                        });
                    });
                });
            }
            4 => {
                // 5. Advanced Diagnostics
                render_section_card(ui, "Diagnostics & Trace Logging", content_w, t, |ui| {
                    ui.checkbox(&mut state.settings.enable_sparse_detection, "Enable Sparse file & virtual block image detection");
                    ui.add_space(4.0);
                    ui.checkbox(&mut state.settings.enable_debug_logging, "Verbose Rust engine trace logs in ~/.local/share/disklens/engine.log");
                });
            }
            _ => {}
        }
    });
}

fn render_section_card(
    ui: &mut Ui,
    title: &str,
    width: f32,
    t: &crate::theme::ThemeTokens,
    add_contents: impl FnOnce(&mut Ui),
) {
    egui::Frame::none()
        .fill(t.bg_elevated)
        .rounding(8.0_f32)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            ui.set_width(width - 24.0);
            ui.vertical(|ui| {
                ui.label(RichText::new(title).size(13.0).strong().color(t.text));
                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);
                add_contents(ui);
            });
        });
}
