use crate::actions::FileActions;
use crate::models::{FileNode, SortColumn, SortOrder};
use crate::theme::{truncate_middle, AppTheme};
use crate::ui::modals::{ConfirmAction, ModalManager};
use eframe::egui::{self, Color32, Margin, Rect, RichText, Sense, Ui, Vec2};
use std::path::PathBuf;

pub struct TreeViewState {
    pub current_path: PathBuf,
    pub sort_column: SortColumn,
    pub sort_order: SortOrder,
    pub search_filter: String,
    pub min_size_filter_mb: f64,
    pub selected_node: Option<PathBuf>,
}

impl Default for TreeViewState {
    fn default() -> Self {
        Self {
            current_path: PathBuf::new(),
            sort_column: SortColumn::Size,
            sort_order: SortOrder::Descending,
            search_filter: String::new(),
            min_size_filter_mb: 0.0,
            selected_node: None,
        }
    }
}

/// Compact, high-density Folder Tree Explorer.
/// Standardizes rows to 26-28px height, formats all numbers in monospace font,
/// displays both Apparent Size and Actual Disk Allocation, and implements hover-activated action toolbars.
pub fn render_tree_view(
    ui: &mut Ui,
    root: &FileNode,
    state: &mut TreeViewState,
    modals: &mut ModalManager,
    dark_mode: bool,
    _on_rescan: &mut dyn FnMut(PathBuf),
) {
    let t = AppTheme::tokens(dark_mode);

    if state.current_path.as_os_str().is_empty() {
        state.current_path = root.path.clone();
    }

    let current_node = root.find_node_by_path(&state.current_path).unwrap_or(root);

    // 1. Breadcrumbs & Top Navigation Bar
    egui::Frame::none()
        .fill(t.bg_elevated)
        .rounding(6.0_f32)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .inner_margin(Margin::symmetric(10.0, 6.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("↰ Up").on_hover_text("Go to parent directory").clicked() {
                    if let Some(parent) = state.current_path.parent() {
                        if parent.starts_with(&root.path) || parent == root.path {
                            state.current_path = parent.to_path_buf();
                            state.selected_node = None;
                        }
                    }
                }

                if ui.button("🏠 Root").on_hover_text("Return to scan root").clicked() {
                    state.current_path = root.path.clone();
                    state.selected_node = None;
                }

                ui.label(RichText::new("|").size(11.0).color(t.border));

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
                    state.current_path = p;
                    state.selected_node = None;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("Allocated: {}", current_node.formatted_allocated_size()))
                            .size(11.0)
                            .monospace()
                            .color(t.cyan),
                    );
                    ui.label(RichText::new("•").size(10.0).color(t.border));
                    ui.label(
                        RichText::new(format!("Apparent: {}", current_node.formatted_size()))
                            .size(11.5)
                            .monospace()
                            .strong()
                            .color(t.accent),
                    );
                });
            });
        });

    ui.add_space(6.0);

    // 2. Filter & Sort Toolbar
    ui.horizontal(|ui| {
        ui.label(RichText::new("Filter:").size(11.0).color(t.text_muted));
        ui.add_sized(
            [160.0, 24.0],
            egui::TextEdit::singleline(&mut state.search_filter).hint_text("Search name/ext..."),
        );

        if !state.search_filter.is_empty() && ui.small_button("✕").clicked() {
            state.search_filter.clear();
        }

        ui.separator();
        ui.label(RichText::new("Min Size:").size(11.0).color(t.text_muted));
        ui.add(
            egui::DragValue::new(&mut state.min_size_filter_mb)
                .speed(1.0)
                .range(0.0..=10_000.0)
                .suffix(" MB"),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let sort_name_label = if state.sort_column == SortColumn::Name {
                if state.sort_order == SortOrder::Ascending { "Name ↑" } else { "Name ↓" }
            } else {
                "Name"
            };
            if ui.selectable_label(state.sort_column == SortColumn::Name, sort_name_label).clicked() {
                toggle_sort(&mut state.sort_column, &mut state.sort_order, SortColumn::Name);
            }

            let sort_size_label = if state.sort_column == SortColumn::Size {
                if state.sort_order == SortOrder::Descending { "Size ↓" } else { "Size ↑" }
            } else {
                "Size"
            };
            if ui.selectable_label(state.sort_column == SortColumn::Size, sort_size_label).clicked() {
                toggle_sort(&mut state.sort_column, &mut state.sort_order, SortColumn::Size);
            }

            let sort_items_label = if state.sort_column == SortColumn::ItemCount {
                if state.sort_order == SortOrder::Descending { "Items ↓" } else { "Items ↑" }
            } else {
                "Items"
            };
            if ui.selectable_label(state.sort_column == SortColumn::ItemCount, sort_items_label).clicked() {
                toggle_sort(&mut state.sort_column, &mut state.sort_order, SortColumn::ItemCount);
            }

            ui.label(RichText::new("Sort:").size(11.0).color(t.text_muted));
        });
    });

    ui.add_space(6.0);

    let min_bytes = (state.min_size_filter_mb * 1024.0 * 1024.0) as u64;
    let mut display_children: Vec<&FileNode> = current_node
        .children
        .iter()
        .filter(|c| {
            if c.size < min_bytes {
                return false;
            }
            if !state.search_filter.is_empty() {
                return c.name.to_lowercase().contains(&state.search_filter.to_lowercase());
            }
            true
        })
        .collect();

    // Sort items
    let sort_col = state.sort_column;
    let sort_ord = state.sort_order;
    display_children.sort_by(|a, b| {
        let cmp = match sort_col {
            SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortColumn::Size => a.size.cmp(&b.size),
            SortColumn::ItemCount => a.total_items().cmp(&b.total_items()),
            SortColumn::Type => {
                let type_a = if a.is_dir { "Folder" } else { a.category.display_name() };
                let type_b = if b.is_dir { "Folder" } else { b.category.display_name() };
                type_a.cmp(type_b)
            }
            SortColumn::Modified => a.modified.cmp(&b.modified),
        };
        if sort_ord == SortOrder::Descending { cmp.reverse() } else { cmp }
    });

    let total_parent_size = current_node.size.max(1);
    let mut navigate_to = None;
    let available_w = ui.available_width().max(550.0);

    // 3. Compact Table Header
    egui::Frame::none()
        .fill(t.bg_elevated)
        .rounding(4.0_f32)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .inner_margin(Margin::symmetric(8.0, 5.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let name_w = (available_w - 620.0).max(140.0);
                ui.add_sized([name_w, 18.0], egui::Label::new(RichText::new("Name").strong().color(t.text_muted)));
                ui.add_sized([100.0, 18.0], egui::Label::new(RichText::new("Usage (Parent)").strong().color(t.text_muted)));
                ui.add_sized([85.0, 18.0], egui::Label::new(RichText::new("Apparent Size").strong().color(t.text_muted)));
                ui.add_sized([85.0, 18.0], egui::Label::new(RichText::new("Allocated Disk").strong().color(t.text_muted)));
                ui.add_sized([70.0, 18.0], egui::Label::new(RichText::new("Inodes / Items").strong().color(t.text_muted)));
                ui.add_sized([90.0, 18.0], egui::Label::new(RichText::new("Type").strong().color(t.text_muted)));
                ui.add_sized([110.0, 18.0], egui::Label::new(RichText::new("Modified").strong().color(t.text_muted)));
                ui.label(RichText::new("Actions").strong().color(t.text_muted));
            });
        });

    ui.add_space(2.0);

    // 4. Virtualized Standard 26-28px Height Table Rows
    let row_height = 28.0_f32;
    let total_rows = display_children.len();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show_rows(ui, row_height, total_rows, |ui, row_range| {
            for idx in row_range {
                if let Some(child) = display_children.get(idx) {
                    let is_selected = match &state.selected_node {
                        Some(p) => *p == child.path,
                        None => false,
                    };

                    let pct = (child.size as f32 / total_parent_size as f32).clamp(0.0, 1.0);
                    let (row_rect, row_response) = ui.allocate_exact_size(
                        Vec2::new(available_w, 26.0),
                        Sense::click(),
                    );
                    let is_hovered = row_response.hovered();

                    let row_bg = if is_selected {
                        Color32::from_rgb(62, 69, 101)
                    } else if is_hovered {
                        t.bg_elevated
                    } else if idx % 2 == 0 {
                        t.bg_container
                    } else {
                        Color32::TRANSPARENT
                    };

                    ui.painter().rect_filled(row_rect, 3.0_f32, row_bg);
                    if is_selected {
                        ui.painter().rect_stroke(row_rect, 3.0_f32, egui::Stroke::new(1.0_f32, t.accent));
                    }

                    let name_w = (available_w - 620.0).max(140.0);

                    // Col 1: Name
                    let icon = if child.is_dir { "📁" } else { child.category.icon() };
                    let max_c = (name_w / 8.0) as usize;
                    let display_n = truncate_middle(&child.name, max_c.clamp(14, 48));
                    ui.painter().text(
                        egui::pos2(row_rect.min.x + 8.0, row_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        format!("{} {}", icon, display_n),
                        egui::FontId::proportional(11.5),
                        t.text,
                    );

                    // Col 2: Usage Bar & Percentage
                    let bar_x = row_rect.min.x + name_w + 8.0;
                    let bar_rect = Rect::from_min_size(egui::pos2(bar_x, row_rect.center().y - 3.0), Vec2::new(55.0, 6.0));
                    ui.painter().rect_filled(bar_rect, 2.0_f32, t.bg_input);
                    let filled_w = (bar_rect.width() * pct).max(1.0);
                    let filled_rect = Rect::from_min_size(bar_rect.min, Vec2::new(filled_w, bar_rect.height()));
                    ui.painter().rect_filled(filled_rect, 2.0_f32, if child.is_dir { t.accent } else { child.category.color() });

                    ui.painter().text(
                        egui::pos2(bar_x + 62.0, row_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        format!("{:.0}%", pct * 100.0),
                        egui::FontId::monospace(10.5),
                        t.text_muted,
                    );

                    // Col 3: Apparent Size (Monospace)
                    let col3_x = bar_x + 100.0;
                    ui.painter().text(
                        egui::pos2(col3_x + 80.0, row_rect.center().y),
                        egui::Align2::RIGHT_CENTER,
                        child.formatted_size(),
                        egui::FontId::monospace(11.5),
                        t.text,
                    );

                    // Col 4: Allocated Size (Monospace)
                    let col4_x = col3_x + 85.0;
                    ui.painter().text(
                        egui::pos2(col4_x + 80.0, row_rect.center().y),
                        egui::Align2::RIGHT_CENTER,
                        child.formatted_allocated_size(),
                        egui::FontId::monospace(11.5),
                        t.cyan,
                    );

                    // Col 5: Items / Inodes (Monospace)
                    let col5_x = col4_x + 85.0;
                    let items_str = if child.is_dir { format!("{}", child.total_items()) } else { "1".to_string() };
                    ui.painter().text(
                        egui::pos2(col5_x + 65.0, row_rect.center().y),
                        egui::Align2::RIGHT_CENTER,
                        items_str,
                        egui::FontId::monospace(11.0),
                        t.text_muted,
                    );

                    // Col 6: Type
                    let col6_x = col5_x + 70.0;
                    let type_s = if child.is_dir { "Folder" } else { child.category.display_name() };
                    ui.painter().text(
                        egui::pos2(col6_x + 6.0, row_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        type_s,
                        egui::FontId::proportional(11.0),
                        if child.is_dir { t.text_secondary } else { child.category.color() },
                    );

                    // Col 7: Modified (Monospace)
                    let col7_x = col6_x + 90.0;
                    ui.painter().text(
                        egui::pos2(col7_x + 6.0, row_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        child.formatted_modified(),
                        egui::FontId::monospace(10.5),
                        t.text_muted,
                    );

                    // Col 8: Actions Toolbar
                    // On hover: show full interactive action toolbar (Terminal, Reveal, Trash, Shred)
                    // When not hovered: show clean subtle indicator, preventing table bloat!
                    let act_x = col7_x + 115.0;
                    let act_w = (row_rect.max.x - act_x - 4.0).max(60.0);
                    let act_rect = Rect::from_min_size(egui::pos2(act_x, row_rect.center().y - 11.0), Vec2::new(act_w, 22.0));

                    if is_hovered {
                        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(act_rect), |ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 2.0;

                                if child.is_dir {
                                    if ui.small_button("Explore →").on_hover_text("Open folder").clicked() {
                                        navigate_to = Some(child.path.clone());
                                    }
                                } else {
                                    if ui.small_button("📄").on_hover_text("Open file").clicked() {
                                        let _ = FileActions::open_file(&child.path);
                                    }
                                }

                                if ui.small_button("📂").on_hover_text("Reveal in file manager").clicked() {
                                    let _ = FileActions::open_in_file_manager(&child.path);
                                }

                                if ui.small_button("📟").on_hover_text("Open terminal here").clicked() {
                                    let _ = FileActions::open_terminal(&child.path);
                                }

                                if ui.small_button("📋").on_hover_text("Copy path").clicked() {
                                    let p_str = child.path.display().to_string();
                                    ui.output_mut(|o| o.copied_text = p_str);
                                    modals.show_toast("Copied path".to_string(), false);
                                }

                                if ui.small_button("🗑").on_hover_text("Move to trash").clicked() {
                                    modals.pending_confirm = Some(ConfirmAction::MoveToTrash(child.path.clone(), child.size));
                                }

                                if ui.small_button("🔥").on_hover_text("Shred / Permanently Delete").clicked() {
                                    modals.pending_confirm = Some(ConfirmAction::PermanentDelete(child.path.clone(), child.size));
                                }
                            });
                        });
                    } else if child.is_dir {
                        // Subtle indicator when not hovered
                        ui.painter().text(
                            egui::pos2(act_x + 6.0, row_rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            "Explore →",
                            egui::FontId::proportional(10.5),
                            t.text_muted,
                        );
                    }

                    if row_response.clicked() {
                        state.selected_node = Some(child.path.clone());
                    }

                    if row_response.double_clicked() {
                        if child.is_dir {
                            navigate_to = Some(child.path.clone());
                        } else {
                            let _ = FileActions::open_file(&child.path);
                        }
                    }
                }
            }
        });

    if let Some(target) = navigate_to {
        state.current_path = target;
        state.selected_node = None;
    }

    ui.add_space(14.0);
}

fn toggle_sort(column: &mut SortColumn, order: &mut SortOrder, clicked: SortColumn) {
    if *column == clicked {
        *order = match order {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        };
    } else {
        *column = clicked;
        *order = SortOrder::Descending;
    }
}
