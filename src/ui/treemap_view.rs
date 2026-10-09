use crate::actions::FileActions;
use crate::app::ActiveTab;
use crate::models::{FileCategory, FileNode};
use crate::theme::{truncate_middle, AppTheme};
use crate::treemap::{get_node_color, TreemapItem, TreemapLayout};
use crate::ui::modals::{ConfirmAction, ModalManager};
use eframe::egui::{self, Color32, Margin, Pos2, RichText, Sense, Ui, Vec2};
use std::path::PathBuf;

pub struct TreemapViewState {
    pub current_path: PathBuf,
    pub max_depth: usize,
    pub hovered_item: Option<TreemapItem>,
    pub selected_item: Option<TreemapItem>,
}

impl Default for TreemapViewState {
    fn default() -> Self {
        Self {
            current_path: PathBuf::new(),
            max_depth: 3,
            hovered_item: None,
            selected_item: None,
        }
    }
}

pub fn render_treemap_view(
    ui: &mut Ui,
    root: &FileNode,
    state: &mut TreemapViewState,
    active_tab: &mut ActiveTab,
    modals: &mut ModalManager,
    dark_mode: bool,
    on_navigate_path: &mut dyn FnMut(PathBuf),
    _on_rescan: &mut dyn FnMut(PathBuf),
) {
    let t = AppTheme::tokens(dark_mode);

    if state.current_path.as_os_str().is_empty() {
        state.current_path = root.path.clone();
    }

    let current_node = root.find_node_by_path(&state.current_path).unwrap_or(root);

    let scrollbar_margin = ui.spacing().scroll.bar_width + 14.0;

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let total_w = (ui.available_width() - scrollbar_margin).max(320.0);

            ui.vertical(|ui| {
                ui.set_max_width(total_w);
                ui.add_space(2.0);

                // 1. Sticky Navigation Bar & Controls
                egui::Frame::none()
                    .fill(t.bg_elevated)
                    .rounding(8.0_f32)
                    .stroke(egui::Stroke::new(1.0_f32, t.border))
                    .inner_margin(Margin::symmetric(10.0, 6.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Direct Jump to Overview Dashboard
                            if ui.button(RichText::new("📊 Overview").size(11.0)).on_hover_text("Return to Overview dashboard").clicked() {
                                *active_tab = ActiveTab::Overview;
                            }

                            ui.separator();

                            // Up One Level button
                            let can_go_up = state.current_path != root.path && state.current_path.parent().is_some();
                            if ui.add_enabled(
                                can_go_up,
                                egui::Button::new(RichText::new("⬆ Up One Level").size(11.5))
                            ).on_hover_text("Navigate to parent directory").clicked() {
                                if let Some(parent) = state.current_path.parent() {
                                    if parent.starts_with(&root.path) || parent == root.path {
                                        let p = parent.to_path_buf();
                                        state.current_path = p.clone();
                                        state.selected_item = None;
                                        on_navigate_path(p);
                                    }
                                }
                            }

                            // Root button
                            if ui.add(
                                egui::Button::new(RichText::new("🏠 Root").size(11.5).strong().color(t.accent))
                                    .fill(Color32::TRANSPARENT)
                            ).on_hover_text("Return to scan root").clicked() {
                                state.current_path = root.path.clone();
                                state.selected_item = None;
                                on_navigate_path(root.path.clone());
                            }

                            ui.separator();

                            // Breadcrumb path components
                            let rel_path = state
                                .current_path
                                .strip_prefix(root.path.parent().unwrap_or(&root.path))
                                .unwrap_or(&state.current_path)
                                .to_path_buf();

                            let mut accum = root.path.parent().unwrap_or(&root.path).to_path_buf();
                            let mut clicked_crumb = None;

                            for comp in rel_path.iter() {
                                accum.push(comp);
                                let c_str = comp.to_string_lossy().to_string();
                                let is_active = accum == state.current_path;
                                let target_clone = accum.clone();

                                if ui.selectable_label(is_active, RichText::new(&c_str).size(11.5).strong()).clicked() {
                                    clicked_crumb = Some(target_clone);
                                }
                                ui.label(RichText::new(">").size(10.0).color(t.text_muted));
                            }

                            if let Some(p) = clicked_crumb {
                                state.current_path = p.clone();
                                state.selected_item = None;
                                on_navigate_path(p);
                            }

                            // Right controls: Depth slider
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Depth:").size(11.0).color(t.text_muted));
                                    if ui.small_button("−").clicked() && state.max_depth > 1 {
                                        state.max_depth -= 1;
                                    }
                                    ui.label(RichText::new(format!("{}", state.max_depth)).size(11.5).strong().color(t.accent));
                                    if ui.small_button("+").clicked() && state.max_depth < 6 {
                                        state.max_depth += 1;
                                    }
                                });
                            });
                        });
                    });

                ui.add_space(6.0);

                // 2. Treemap Canvas + Right Inspector Split (Clamped Layout)
                let is_wide = total_w >= 640.0;
                let spacing = if is_wide { 6.0 } else { 0.0 };
                let inspector_w = if is_wide { (total_w * 0.28).clamp(210.0, 260.0) } else { 0.0 };
                let canvas_w = if is_wide { (total_w - inspector_w - spacing).max(100.0) } else { total_w };
                let canvas_h = (ui.available_height() - 76.0).max(200.0);

                let mut clicked_item: Option<TreemapItem> = None;
                let mut right_clicked_item: Option<TreemapItem> = None;
                let mut new_hovered: Option<TreemapItem> = None;
                let mut drill_down_path = None;

                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = spacing;
                    // Main Treemap Stage
        let (response, painter) = ui.allocate_painter(Vec2::new(canvas_w, canvas_h), Sense::click_and_drag());
        let canvas_rect = response.rect;
        let mouse_pos = ui.input(|i| i.pointer.hover_pos());

        if canvas_rect.width() > 10.0 && canvas_rect.height() > 10.0 {
            let layout = TreemapLayout::build(current_node, canvas_rect, state.max_depth);

            painter.rect_filled(canvas_rect, 4.0_f32, t.bg_input);
            painter.rect_stroke(canvas_rect, 4.0_f32, egui::Stroke::new(1.0_f32, t.border));

            if let Some(pos) = mouse_pos {
                if canvas_rect.contains(pos) {
                    let mut best: Option<&TreemapItem> = None;
                    for item in &layout.items {
                        if item.rect.contains(pos) {
                            match best {
                                None => best = Some(item),
                                Some(prev) => {
                                    if (item.rect.width() * item.rect.height()) < (prev.rect.width() * prev.rect.height()) {
                                        best = Some(item);
                                    }
                                }
                            }
                        }
                    }
                    new_hovered = best.cloned();
                }
            }

            let is_any_h = new_hovered.is_some();

            for item in &layout.items {
                let is_h = match &new_hovered {
                    Some(h) => h.path == item.path,
                    None => false,
                };
                let is_sel = match &state.selected_item {
                    Some(s) => s.path == item.path,
                    None => false,
                };

                let (mut fill_col, mut stroke) = get_node_color(item, is_h);

                if is_any_h && !is_h && !is_sel {
                    fill_col = fill_col.gamma_multiply(0.70);
                }

                if is_h || is_sel {
                    stroke = egui::Stroke::new(2.0_f32, Color32::WHITE);
                }

                painter.rect(item.rect, 2.0_f32, fill_col, stroke);

                if item.rect.width() > 44.0 && item.rect.height() > 18.0 {
                    let icon = if item.is_dir { "📁 " } else { "" };
                    let label = format!("{}{}", icon, item.name);

                    let f_size = if item.rect.width() > 80.0 && item.rect.height() > 32.0 { 11.5 } else { 9.5 };
                    let t_pos = Pos2::new(item.rect.min.x + 4.0, item.rect.min.y + 3.0);
                    painter.text(t_pos, egui::Align2::LEFT_TOP, truncate_middle(&label, 16), egui::FontId::proportional(f_size), Color32::WHITE);

                    if item.rect.width() > 60.0 && item.rect.height() > 30.0 {
                        let sz_pos = Pos2::new(item.rect.min.x + 4.0, item.rect.min.y + 4.0 + f_size + 1.0);
                        painter.text(sz_pos, egui::Align2::LEFT_TOP, crate::models::format_size(item.size), egui::FontId::monospace(f_size - 1.0), Color32::WHITE.gamma_multiply(0.85));
                    }
                }
            }

            if response.clicked() {
                if let Some(item) = &new_hovered {
                    clicked_item = Some(item.clone());
                }
            }
            if response.secondary_clicked() {
                if let Some(item) = &new_hovered {
                    right_clicked_item = Some(item.clone());
                }
            }
        }

        state.hovered_item = new_hovered;

        if let Some(item) = clicked_item {
            if item.is_dir {
                let p = item.path.clone();
                state.current_path = p.clone();
                state.selected_item = None;
                on_navigate_path(p);
            } else {
                state.selected_item = Some(item);
            }
        }

        // Right Inspector Panel (Matches TreemapView.tsx)
        if is_wide {
            let inspect_clone = state.selected_item.clone().or_else(|| state.hovered_item.clone());

            egui::Frame::none()
                .fill(t.bg_elevated)
                .rounding(8.0_f32)
                .stroke(egui::Stroke::new(1.0_f32, t.border))
                .inner_margin(Margin::same(12.0))
                .show(ui, |ui| {
                    ui.set_width(inspector_w);
                    ui.set_height(canvas_h);

                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("NODE INSPECTOR").size(10.5).strong().color(t.text_muted));
                            if let Some(item) = &inspect_clone {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(item.category.display_name()).size(10.0).color(item.category.color()));
                                });
                            }
                        });

                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(6.0);

                        if let Some(item) = &inspect_clone {
                            ui.label(RichText::new(&item.name).size(14.0).strong().color(t.text));
                            ui.add_space(2.0);

                            // Path with copy button
                            let p_str = item.path.display().to_string();
                            egui::Frame::none()
                                .fill(t.bg_surface)
                                .rounding(4.0_f32)
                                .inner_margin(Margin::same(6.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(truncate_middle(&p_str, 24)).size(10.5).monospace().color(t.text_muted));
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.small_button("📋").on_hover_text("Copy path").clicked() {
                                                ui.output_mut(|o| o.copied_text = p_str.clone());
                                                modals.show_toast("Copied path".to_string(), false);
                                            }
                                        });
                                    });
                                });

                            ui.add_space(8.0);

                            // Metrics
                            let total_v = current_node.size.max(1);
                            let share_pct = (item.size as f64 / total_v as f64) * 100.0;

                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Allocated Size:").size(11.0).color(t.text_muted));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(crate::models::format_size(item.size)).size(12.5).strong().color(t.accent));
                                });
                            });

                            ui.horizontal(|ui| {
                                ui.label(RichText::new("View Share:").size(11.0).color(t.text_muted));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(format!("{:.1}%", share_pct)).size(11.5).strong().color(t.text));
                                });
                            });

                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Type:").size(11.0).color(t.text_muted));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(if item.is_dir { "Directory" } else { "File" }).size(11.0).color(t.text_secondary));
                                });
                            });

                            ui.add_space(12.0);

                            // Action buttons
                            if ui.add_sized([ui.available_width(), 26.0], egui::Button::new("👁 Quick Preview")).on_hover_text("Preview file content and metadata").clicked() {
                                modals.pending_preview = Some(item.path.clone());
                            }

                            if ui.add_sized([ui.available_width(), 26.0], egui::Button::new("✏ Rename...")).on_hover_text("Rename this file or directory").clicked() {
                                modals.pending_rename = Some((item.path.clone(), item.name.clone()));
                            }

                            if item.is_dir {
                                if ui.add_sized([ui.available_width(), 26.0], egui::Button::new("📂 Explore Node →")).clicked() {
                                    drill_down_path = Some(item.path.clone());
                                }
                            } else {
                                if ui.add_sized([ui.available_width(), 26.0], egui::Button::new("📄 Open File")).clicked() {
                                    let _ = FileActions::open_file(&item.path);
                                }
                            }

                            if ui.add_sized([ui.available_width(), 26.0], egui::Button::new("📁 Show in File Manager")).clicked() {
                                let _ = FileActions::open_in_file_manager(&item.path);
                            }

                            if ui.add_sized([ui.available_width(), 26.0], egui::Button::new(RichText::new("🗑 Move to Trash...").color(t.danger))).clicked() {
                                modals.pending_confirm = Some(ConfirmAction::MoveToTrash(item.path.clone(), item.size));
                            }
                        } else {
                            // Default Inspector: Show telemetry of current viewport directory instead of empty dead space
                            ui.label(RichText::new(&current_node.name).size(13.5).strong().color(t.text));
                            ui.add_space(2.0);

                            let cur_p = current_node.path.display().to_string();
                            egui::Frame::none()
                                .fill(t.bg_surface)
                                .rounding(4.0_f32)
                                .inner_margin(Margin::same(6.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(truncate_middle(&cur_p, 24)).size(10.5).monospace().color(t.text_muted));
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.small_button("📋").on_hover_text("Copy path").clicked() {
                                                ui.output_mut(|o| o.copied_text = cur_p.clone());
                                                modals.show_toast("Copied path".to_string(), false);
                                            }
                                        });
                                    });
                                });

                            ui.add_space(8.0);

                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Apparent Size:").size(11.0).color(t.text_muted));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(current_node.formatted_size()).size(12.0).monospace().strong().color(t.accent));
                                });
                            });

                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Allocated Disk:").size(11.0).color(t.text_muted));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(current_node.formatted_allocated_size()).size(12.0).monospace().strong().color(t.cyan));
                                });
                            });

                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Contained Items:").size(11.0).color(t.text_muted));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new(format!("{}f · {}d", current_node.file_count, current_node.dir_count)).size(11.0).monospace().color(t.text));
                                });
                            });

                            ui.add_space(10.0);

                            if ui.add_sized([ui.available_width(), 26.0], egui::Button::new("📂 Open in File Manager")).clicked() {
                                let _ = FileActions::open_in_file_manager(&current_node.path);
                            }

                            if ui.add_sized([ui.available_width(), 26.0], egui::Button::new("📟 Open Terminal Here")).clicked() {
                                let _ = FileActions::open_terminal(&current_node.path);
                            }

                            if ui.add_sized([ui.available_width(), 26.0], egui::Button::new("📁 Reveal in Folder Tree")).clicked() {
                                *active_tab = ActiveTab::DirectoryTree;
                            }

                            ui.add_space(8.0);
                            ui.label(RichText::new("Hover or click any treemap block to inspect individual files and subfolders.").size(10.0).italics().color(t.text_muted));
                        }
                    });
                });
        }
    });

    if let Some(p) = drill_down_path {
        state.current_path = p;
        state.selected_item = None;
    }

    if let Some(item) = right_clicked_item {
        state.selected_item = Some(item);
    }

    ui.add_space(6.0);

    // 3. Bottom Category Color Legend
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(10.0, 4.0);
        ui.label(RichText::new("Color Legend:").size(10.5).color(t.text_muted).strong());

        for cat in FileCategory::ALL {
            ui.horizontal(|ui| {
                let (dot, _) = ui.allocate_exact_size(Vec2::new(7.0, 7.0), Sense::hover());
                ui.painter().rect_filled(dot, 2.0_f32, cat.color());
                ui.label(RichText::new(cat.display_name()).size(10.5).color(t.text_secondary));
            });
        }
    });

    ui.add_space(8.0);
            });
        });
}
