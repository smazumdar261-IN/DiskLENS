use crate::theme::AppTheme;
use crate::ui::modals::ModalManager;
use eframe::egui::{self, Margin, RichText, Ui};

pub struct LogsViewState {
    pub search_query: String,
    pub level_filter: String,
}

impl Default for LogsViewState {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            level_filter: "ALL".to_string(),
        }
    }
}

pub fn render_logs_view(
    ui: &mut Ui,
    state: &mut LogsViewState,
    modals: &ModalManager,
    dark_mode: bool,
) {
    let t = AppTheme::tokens(dark_mode);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let scrollbar_margin = ui.spacing().scroll.bar_width + 14.0;
            let available_w = (ui.available_width() - scrollbar_margin).max(320.0);
            let content_w = available_w;

            ui.vertical(|ui| {
                ui.set_max_width(content_w);
                ui.add_space(8.0);

                // 1. Top Banner & Filters
                egui::Frame::none()
                    .fill(t.bg_elevated)
                    .rounding(8.0_f32)
                    .stroke(egui::Stroke::new(1.0_f32, t.border))
                    .inner_margin(Margin::same(16.0))
                    .show(ui, |ui| {
                        let is_stacked = content_w < 680.0;
                        if is_stacked {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("📜").size(24.0).color(t.purple));
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new("Storage Event & Journal Logs").size(15.0).strong().color(t.text));
                                            ui.label(RichText::new("JOURNALD STREAM").size(10.0).monospace().strong().color(t.purple));
                                        });
                                        ui.label(RichText::new("Live kernel, udev, btrfs scrub, and engine disk telemetry events.").size(11.5).color(t.text_secondary));
                                    });
                                });

                                ui.add_space(10.0);

                                ui.horizontal(|ui| {
                                    ui.add(egui::TextEdit::singleline(&mut state.search_query).hint_text("Search log messages...").desired_width(180.0));
                                    egui::ComboBox::from_id_salt("log_level_filter")
                                        .selected_text(&state.level_filter)
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut state.level_filter, "ALL".to_string(), "All Levels");
                                            ui.selectable_value(&mut state.level_filter, "INFO".to_string(), "INFO Only");
                                            ui.selectable_value(&mut state.level_filter, "WARN".to_string(), "WARN Only");
                                            ui.selectable_value(&mut state.level_filter, "ERROR".to_string(), "ERROR Only");
                                            ui.selectable_value(&mut state.level_filter, "DEBUG".to_string(), "DEBUG Only");
                                        });
                                });
                            });
                        } else {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("📜").size(28.0).color(t.purple));
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("Storage Event & Journal Logs").size(16.0).strong().color(t.text));
                                        ui.label(RichText::new("JOURNALD STREAM").size(10.0).monospace().strong().color(t.purple));
                                    });
                                    ui.label(RichText::new("Live kernel, udev, btrfs scrub, and engine disk telemetry events.").size(11.5).color(t.text_secondary));
                                });

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    egui::ComboBox::from_id_salt("log_level_filter")
                                        .selected_text(&state.level_filter)
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut state.level_filter, "ALL".to_string(), "All Levels");
                                            ui.selectable_value(&mut state.level_filter, "INFO".to_string(), "INFO Only");
                                            ui.selectable_value(&mut state.level_filter, "WARN".to_string(), "WARN Only");
                                            ui.selectable_value(&mut state.level_filter, "ERROR".to_string(), "ERROR Only");
                                            ui.selectable_value(&mut state.level_filter, "DEBUG".to_string(), "DEBUG Only");
                                        });

                                    ui.add(egui::TextEdit::singleline(&mut state.search_query).hint_text("Search log messages...").desired_width(180.0));
                                });
                            });
                        }
                    });

                ui.add_space(12.0);

                // 2. Filtered Log Entries Feed
                let q_lower = state.search_query.to_lowercase();
                let filtered_logs: Vec<&(String, String, String)> = modals
                    .log_messages
                    .iter()
                    .filter(|(_, level, msg)| {
                        let matches_level = state.level_filter == "ALL" || level == &state.level_filter;
                        let matches_q = q_lower.is_empty() || msg.to_lowercase().contains(&q_lower);
                        matches_level && matches_q
                    })
                    .collect();

                egui::Frame::none()
                    .fill(t.bg_container)
                    .rounding(8.0_f32)
                    .stroke(egui::Stroke::new(1.0_f32, t.border))
                    .inner_margin(Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("EVENT STREAM").size(11.0).strong().color(t.text_muted));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(RichText::new(format!("{} Events recorded", filtered_logs.len())).size(10.5).color(t.text_secondary));
                            });
                        });

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(6.0);

                        if filtered_logs.is_empty() {
                            ui.vertical_centered(|ui| {
                                ui.add_space(40.0);
                                ui.label(RichText::new("No matching storage log events found.").size(12.0).color(t.text_muted));
                            });
                        } else {
                            for (time, level, msg) in filtered_logs {
                                let lvl_color = match level.as_str() {
                                    "ERROR" => t.danger,
                                    "WARN" => t.warning,
                                    "INFO" => t.accent,
                                    "DEBUG" => t.purple,
                                    _ => t.text_muted,
                                };

                                egui::Frame::none()
                                    .fill(t.bg_surface)
                                    .rounding(4.0_f32)
                                    .stroke(egui::Stroke::new(1.0_f32, t.border))
                                    .inner_margin(Margin::symmetric(10.0, 6.0))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(time).size(10.5).monospace().color(t.text_muted));
                                            ui.label(RichText::new(level).size(10.5).monospace().strong().color(lvl_color));
                                            ui.label(RichText::new("[engine]").size(10.5).monospace().color(t.purple));
                                            ui.label(RichText::new(msg).size(11.5).color(t.text));
                                        });
                                    });

                                ui.add_space(4.0);
                            }
                        }
                    });

                ui.add_space(20.0);
            });
        });
}
