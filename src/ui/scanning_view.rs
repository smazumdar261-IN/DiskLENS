use crate::models::{format_size, ScanProgress};
use crate::theme::{truncate_middle, AppTheme};
use eframe::egui::{self, Color32, Margin, Rect, RichText, Sense, Ui, Vec2};
use sysinfo::Disks;

pub fn render_scanning_view(
    ui: &mut Ui,
    target_path_str: &str,
    progress: &ScanProgress,
    dark_mode: bool,
    on_toggle_pause: &mut dyn FnMut(),
    on_cancel: &mut dyn FnMut(),
) {
    let t = AppTheme::tokens(dark_mode);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let total_avail = ui.available_width().max(320.0);

            ui.vertical(|ui| {
                ui.set_max_width(total_avail);
                ui.add_space(8.0);

                // 1. Top Scanning Header Status
                ui.horizontal(|ui| {
                    let (spinner_rect, _) =
                        ui.allocate_exact_size(Vec2::new(16.0, 16.0), Sense::hover());
                    ui.painter().circle_filled(
                        spinner_rect.center(),
                        6.0,
                        if progress.is_paused {
                            t.warning
                        } else {
                            t.accent
                        },
                    );

                    let status_text = if progress.is_paused {
                        "Scanning Paused"
                    } else {
                        "Traversing File System Tree..."
                    };
                    ui.label(RichText::new(status_text).size(16.0).strong().color(t.text));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let (pulse_rect, _) =
                            ui.allocate_exact_size(Vec2::new(8.0, 8.0), Sense::hover());
                        ui.painter().circle_filled(
                            pulse_rect.center(),
                            4.0,
                            if progress.is_paused {
                                t.warning
                            } else {
                                t.success
                            },
                        );
                        ui.label(
                            RichText::new(if progress.is_paused {
                                "PAUSED"
                            } else {
                                "ACTIVE"
                            })
                            .size(11.0)
                            .strong()
                            .monospace()
                            .color(if progress.is_paused {
                                t.warning
                            } else {
                                t.success
                            }),
                        );
                    });
                });

                ui.add_space(8.0);

                // 2. Active Path & Pulsing Progress Bar Card (Rigid envelope & jitter elimination)
                ui.allocate_ui_with_layout(
                    egui::vec2(total_avail, 0.0),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.set_max_width(total_avail);
                        egui::Frame::none()
                            .fill(t.bg_elevated)
                            .rounding(6.0_f32)
                            .stroke(egui::Stroke::new(1.0_f32, t.border))
                            .inner_margin(Margin::same(12.0))
                            .show(ui, |ui| {
                                let inner_w = (total_avail - 24.0).max(60.0);
                                ui.set_width(inner_w);

                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("Target Location:")
                                            .size(11.0)
                                            .color(t.text_muted),
                                    );
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(target_path_str)
                                                .size(12.0)
                                                .strong()
                                                .color(t.text),
                                        )
                                        .truncate(),
                                    )
                                    .on_hover_text(target_path_str);
                                });

                                ui.add_space(4.0);

                                // Current Path Traversing: dedicated width allocation prevents parent container jitter
                                let curr_p = if progress.current_path.is_empty() {
                                    target_path_str
                                } else {
                                    progress.current_path.as_str()
                                };
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("Current:").size(11.0).color(t.text_muted),
                                    );
                                    let bytes_str = format!(
                                        "{} bytes indexed",
                                        format_size(progress.scanned_bytes)
                                    );
                                    let bytes_w = 145.0_f32;
                                    let path_avail_w = (inner_w - 75.0 - bytes_w).max(40.0);

                                    ui.add_sized(
                                        [path_avail_w, 18.0],
                                        egui::Label::new(
                                            RichText::new(curr_p)
                                                .size(11.0)
                                                .monospace()
                                                .color(t.cyan),
                                        )
                                        .truncate(),
                                    )
                                    .on_hover_text(curr_p);

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                RichText::new(bytes_str)
                                                    .size(11.0)
                                                    .monospace()
                                                    .color(t.text),
                                            );
                                        },
                                    );
                                });

                                ui.add_space(8.0);

                                // Progress Bar
                                let (bar_rect, _) =
                                    ui.allocate_exact_size(Vec2::new(inner_w, 8.0), Sense::hover());
                                ui.painter().rect_filled(bar_rect, 4.0_f32, t.bg_input);

                                // Animated progress indicator based on steady indexing
                                let sim_w = (bar_rect.width()
                                    * ((progress.elapsed_secs * 0.15).fract() as f32))
                                    .clamp(20.0, bar_rect.width());
                                let sim_rect = Rect::from_min_size(
                                    bar_rect.min,
                                    Vec2::new(sim_w, bar_rect.height()),
                                );
                                ui.painter().rect_filled(
                                    sim_rect,
                                    4.0_f32,
                                    if progress.is_paused {
                                        t.warning
                                    } else {
                                        t.accent
                                    },
                                );
                            });
                    },
                );

                ui.add_space(10.0);

                // 3. Responsive 5-Column Metrics Bento Grid
                let rate = if progress.elapsed_secs > 0.0 {
                    (progress.scanned_files + progress.scanned_dirs) as f64 / progress.elapsed_secs
                } else {
                    0.0
                };

                let metrics = [
                    ("SCANNED SIZE", format_size(progress.scanned_bytes), t.text),
                    (
                        "FILES INDEXED",
                        format!("{}", progress.scanned_files),
                        t.text,
                    ),
                    ("DIRECTORIES", format!("{}", progress.scanned_dirs), t.text),
                    ("I/O RATE", format!("{:.0} items/s", rate), t.success),
                    (
                        "ELAPSED",
                        format!("{:.1}s", progress.elapsed_secs),
                        t.warning,
                    ),
                ];

                let spacing = 8.0_f32;
                if total_avail >= 900.0 {
                    // 5 columns
                    let card_w = ((total_avail - (spacing * 4.0)) / 5.0).floor();
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = spacing;
                        for (label, val, col) in &metrics {
                            ui.allocate_ui_with_layout(
                                egui::vec2(card_w, 0.0),
                                egui::Layout::top_down(egui::Align::LEFT),
                                |ui| {
                                    ui.set_max_width(card_w);
                                    render_metric_cell(ui, label, val, *col, card_w, &t);
                                },
                            );
                        }
                    });
                } else if total_avail >= 600.0 {
                    // 3 columns top + 2 columns bottom
                    let card_w3 = ((total_avail - (spacing * 2.0)) / 3.0).floor();
                    let card_w2 = ((total_avail - spacing) / 2.0).floor();
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = spacing;
                        for (label, val, col) in &metrics[0..3] {
                            ui.allocate_ui_with_layout(
                                egui::vec2(card_w3, 0.0),
                                egui::Layout::top_down(egui::Align::LEFT),
                                |ui| {
                                    ui.set_max_width(card_w3);
                                    render_metric_cell(ui, label, val, *col, card_w3, &t);
                                },
                            );
                        }
                    });
                    ui.add_space(spacing);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = spacing;
                        for (label, val, col) in &metrics[3..5] {
                            ui.allocate_ui_with_layout(
                                egui::vec2(card_w2, 0.0),
                                egui::Layout::top_down(egui::Align::LEFT),
                                |ui| {
                                    ui.set_max_width(card_w2);
                                    render_metric_cell(ui, label, val, *col, card_w2, &t);
                                },
                            );
                        }
                    });
                } else {
                    // 2 columns top + 2 columns mid + 1 full bottom
                    let card_w2 = ((total_avail - spacing) / 2.0).floor();
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = spacing;
                        for (label, val, col) in &metrics[0..2] {
                            ui.allocate_ui_with_layout(
                                egui::vec2(card_w2, 0.0),
                                egui::Layout::top_down(egui::Align::LEFT),
                                |ui| {
                                    ui.set_max_width(card_w2);
                                    render_metric_cell(ui, label, val, *col, card_w2, &t);
                                },
                            );
                        }
                    });
                    ui.add_space(spacing);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = spacing;
                        for (label, val, col) in &metrics[2..4] {
                            ui.allocate_ui_with_layout(
                                egui::vec2(card_w2, 0.0),
                                egui::Layout::top_down(egui::Align::LEFT),
                                |ui| {
                                    ui.set_max_width(card_w2);
                                    render_metric_cell(ui, label, val, *col, card_w2, &t);
                                },
                            );
                        }
                    });
                    ui.add_space(spacing);
                    ui.allocate_ui_with_layout(
                        egui::vec2(total_avail, 0.0),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            ui.set_max_width(total_avail);
                            render_metric_cell(
                                ui,
                                metrics[4].0,
                                &metrics[4].1,
                                metrics[4].2,
                                total_avail,
                                &t,
                            );
                        },
                    );
                }

                ui.add_space(10.0);

                // 4. Main Split Area: Issues Table (Left) + Diagnostics & Controls (Right)
                let is_stacked_split = total_avail < 900.0;

                if is_stacked_split {
                    ui.vertical(|ui| {
                        ui.allocate_ui_with_layout(
                            egui::vec2(total_avail, 0.0),
                            egui::Layout::top_down(egui::Align::LEFT),
                            |ui| {
                                ui.set_max_width(total_avail);
                                render_issues_card(ui, progress, total_avail, &t);
                            },
                        );
                        ui.add_space(10.0);
                        ui.allocate_ui_with_layout(
                            egui::vec2(total_avail, 0.0),
                            egui::Layout::top_down(egui::Align::LEFT),
                            |ui| {
                                ui.set_max_width(total_avail);
                                render_diagnostics_and_controls(
                                    ui,
                                    target_path_str,
                                    progress,
                                    total_avail,
                                    &t,
                                    on_toggle_pause,
                                    on_cancel,
                                );
                            },
                        );
                    });
                } else {
                    let split_spacing = 10.0_f32;
                    let left_w = ((total_avail - split_spacing) * 0.62).floor();
                    let right_w = (total_avail - split_spacing - left_w).floor();

                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = split_spacing;
                        ui.allocate_ui_with_layout(
                            egui::vec2(left_w, 0.0),
                            egui::Layout::top_down(egui::Align::LEFT),
                            |ui| {
                                ui.set_max_width(left_w);
                                render_issues_card(ui, progress, left_w, &t);
                            },
                        );
                        ui.allocate_ui_with_layout(
                            egui::vec2(right_w, 0.0),
                            egui::Layout::top_down(egui::Align::LEFT),
                            |ui| {
                                ui.set_max_width(right_w);
                                render_diagnostics_and_controls(
                                    ui,
                                    target_path_str,
                                    progress,
                                    right_w,
                                    &t,
                                    on_toggle_pause,
                                    on_cancel,
                                );
                            },
                        );
                    });
                }

                ui.add_space(24.0);
            });
        });
}

fn render_metric_cell(
    ui: &mut Ui,
    label: &str,
    val: &str,
    col: Color32,
    width: f32,
    t: &crate::theme::ThemeTokens,
) {
    egui::Frame::none()
        .fill(t.bg_elevated)
        .rounding(6.0_f32)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.set_width(width - 20.0);
            ui.label(RichText::new(label).size(10.0).strong().color(t.text_muted));
            ui.add_space(2.0);
            ui.label(RichText::new(val).size(15.0).strong().color(col));
        });
}

fn render_issues_card(
    ui: &mut Ui,
    progress: &ScanProgress,
    width: f32,
    t: &crate::theme::ThemeTokens,
) {
    egui::Frame::none()
        .fill(t.bg_elevated)
        .rounding(6.0_f32)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            let inner_w = width - 24.0;
            ui.set_width(inner_w);

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Issues Encountered")
                        .size(13.0)
                        .strong()
                        .color(t.text),
                );
                ui.label(
                    RichText::new(format!("({})", progress.issues.len()))
                        .size(11.0)
                        .monospace()
                        .color(if progress.issues.is_empty() {
                            t.success
                        } else {
                            t.danger
                        }),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new("Non-fatal permission / symlink skips")
                            .size(10.5)
                            .color(t.text_muted),
                    );
                });
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(4.0);

            if progress.issues.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(24.0);
                    ui.label(
                        RichText::new("✓ Zero I/O errors or inaccessible paths encountered.")
                            .size(11.5)
                            .color(t.success),
                    );
                    ui.add_space(24.0);
                });
            } else {
                let issues_max_h = (ui.available_height() - 30.0).clamp(160.0, 320.0);
                egui::ScrollArea::vertical()
                    .max_height(issues_max_h)
                    .show(ui, |ui| {
                        for issue in &progress.issues {
                            ui.horizontal(|ui| {
                                let badge_color = if issue.severity == "error" {
                                    t.danger
                                } else {
                                    t.warning
                                };
                                ui.label(
                                    RichText::new(issue.issue_type.display_name())
                                        .size(10.0)
                                        .monospace()
                                        .strong()
                                        .color(badge_color),
                                );

                                let path_str = issue.path.display().to_string();
                                let trunc_p = truncate_middle(&path_str, 28);
                                ui.label(
                                    RichText::new(trunc_p).size(11.0).monospace().color(t.text),
                                );

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui
                                            .small_button("📋")
                                            .on_hover_text("Copy path")
                                            .clicked()
                                        {
                                            ui.output_mut(|o| o.copied_text = path_str);
                                        }
                                        ui.label(
                                            RichText::new(&issue.timestamp)
                                                .size(10.0)
                                                .color(t.text_muted),
                                        );
                                    },
                                );
                            });
                            ui.add_space(2.0);
                        }
                    });
            }
        });
}

fn render_diagnostics_and_controls(
    ui: &mut Ui,
    target_path_str: &str,
    progress: &ScanProgress,
    width: f32,
    t: &crate::theme::ThemeTokens,
    on_toggle_pause: &mut dyn FnMut(),
    on_cancel: &mut dyn FnMut(),
) {
    ui.vertical(|ui| {
        let inner_w = width - 24.0;

        // Volume Diagnostics Card
        egui::Frame::none()
            .fill(t.bg_elevated)
            .rounding(6.0_f32)
            .stroke(egui::Stroke::new(1.0_f32, t.border))
            .inner_margin(Margin::same(12.0))
            .show(ui, |ui| {
                ui.set_width(inner_w);
                ui.label(
                    RichText::new("Volume Diagnostics")
                        .size(13.0)
                        .strong()
                        .color(t.text),
                );
                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                let mut disks = Disks::new();
                disks.refresh(true);
                let matching_disk = disks.list().iter().find(|d| {
                    target_path_str.starts_with(&d.mount_point().to_string_lossy().to_string())
                });

                let (fs_type, mount_p, used_pct) = match matching_disk {
                    Some(d) => (
                        d.file_system().to_string_lossy().to_string(),
                        d.mount_point().display().to_string(),
                        if d.total_space() > 0 {
                            (d.total_space() - d.available_space()) as f32 / d.total_space() as f32
                        } else {
                            0.0
                        },
                    ),
                    None => ("ext4".to_string(), target_path_str.to_string(), 0.45),
                };

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Filesystem:").size(11.0).color(t.text_muted));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(fs_type).size(11.5).strong().color(t.text));
                    });
                });

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Block Size:").size(11.0).color(t.text_muted));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("4096 B").size(11.0).color(t.text));
                    });
                });

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Mount:").size(11.0).color(t.text_muted));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(truncate_middle(&mount_p, 20))
                                .size(11.0)
                                .color(t.text),
                        );
                    });
                });

                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!("Capacity Used: {:.0}%", used_pct * 100.0))
                        .size(10.5)
                        .color(t.text_muted),
                );
                let (cbar, _) = ui.allocate_exact_size(Vec2::new(inner_w, 6.0), Sense::hover());
                ui.painter().rect_filled(cbar, 3.0_f32, t.bg_input);
                let filled_c = Rect::from_min_size(
                    cbar.min,
                    Vec2::new((cbar.width() * used_pct).max(4.0), cbar.height()),
                );
                ui.painter().rect_filled(filled_c, 3.0_f32, t.accent);
            });

        ui.add_space(8.0);

        // Control Actions Card
        egui::Frame::none()
            .fill(t.bg_elevated)
            .rounding(6.0_f32)
            .stroke(egui::Stroke::new(1.0_f32, t.border))
            .inner_margin(Margin::same(12.0))
            .show(ui, |ui| {
                ui.set_width(inner_w);

                let pause_label = if progress.is_paused {
                    "▶ Resume Scan"
                } else {
                    "⏸ Pause Scan"
                };
                if ui
                    .add_sized(
                        [inner_w, 30.0],
                        egui::Button::new(
                            RichText::new(pause_label).size(12.0).strong().color(t.text),
                        )
                        .fill(t.bg_surface)
                        .stroke(egui::Stroke::new(1.0_f32, t.border))
                        .rounding(4.0_f32),
                    )
                    .clicked()
                {
                    on_toggle_pause();
                }

                ui.add_space(4.0);

                if ui
                    .add_sized(
                        [inner_w, 28.0],
                        egui::Button::new(
                            RichText::new("✕ Cancel Scan")
                                .size(11.5)
                                .strong()
                                .color(t.danger),
                        )
                        .fill(t.bg_surface)
                        .stroke(egui::Stroke::new(1.0_f32, t.border))
                        .rounding(4.0_f32),
                    )
                    .clicked()
                {
                    on_cancel();
                }
            });
    });
}
