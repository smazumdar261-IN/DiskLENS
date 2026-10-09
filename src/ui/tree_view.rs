use crate::actions::FileActions;
use crate::models::{format_size, FileCategory, FileNode, SortColumn, SortOrder};
use crate::theme::AppTheme;
use crate::ui::modals::{ConfirmAction, ModalManager};
use crate::ui::treemap_view::TreemapViewState;
use eframe::egui::{self, Color32, Margin, Rect, RichText, Sense, Ui, Vec2};
use egui_extras::{Column, TableBuilder};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplorerMode {
    Tree,
    Treemap,
}

impl Default for ExplorerMode {
    fn default() -> Self {
        ExplorerMode::Tree
    }
}

pub struct TreeViewState {
    pub mode: ExplorerMode,
    pub current_path: PathBuf,
    pub history: Vec<PathBuf>,
    pub history_index: usize,
    pub sort_column: SortColumn,
    pub sort_order: SortOrder,
    pub search_filter: String,
    pub category_filter: Option<FileCategory>,
    pub min_size_filter_mb: f64,
    pub selected_node: Option<PathBuf>,
    pub treemap_state: TreemapViewState,
}

impl Default for TreeViewState {
    fn default() -> Self {
        Self {
            mode: ExplorerMode::Tree,
            current_path: PathBuf::new(),
            history: Vec::new(),
            history_index: 0,
            sort_column: SortColumn::Size,
            sort_order: SortOrder::Descending,
            search_filter: String::new(),
            category_filter: None,
            min_size_filter_mb: 0.0,
            selected_node: None,
            treemap_state: TreemapViewState::default(),
        }
    }
}

impl TreeViewState {
    pub fn navigate_to(&mut self, path: PathBuf) {
        if self.current_path == path {
            return;
        }
        if !self.history.is_empty() && self.history_index + 1 < self.history.len() {
            self.history.truncate(self.history_index + 1);
        }
        self.history.push(path.clone());
        self.history_index = self.history.len() - 1;
        self.current_path = path;
        self.selected_node = None;
    }

    #[allow(dead_code)]
    pub fn can_go_back(&self) -> bool {
        self.history_index > 0
    }

    #[allow(dead_code)]
    pub fn can_go_forward(&self) -> bool {
        !self.history.is_empty() && self.history_index + 1 < self.history.len()
    }

    #[allow(dead_code)]
    pub fn go_back(&mut self) -> Option<PathBuf> {
        if self.can_go_back() {
            self.history_index -= 1;
            let path = self.history[self.history_index].clone();
            self.current_path = path.clone();
            self.selected_node = None;
            Some(path)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn go_forward(&mut self) -> Option<PathBuf> {
        if self.can_go_forward() {
            self.history_index += 1;
            let path = self.history[self.history_index].clone();
            self.current_path = path.clone();
            self.selected_node = None;
            Some(path)
        } else {
            None
        }
    }
}

/// Unified Explorer View: Unifies Folder Tree, Space Hogs, Category filtering, and Treemap.
/// Mode 1: Tree & Space Hogs (Bounded TableBuilder with interactive category bar and sorting).
/// Mode 2: Treemap (Clamped Canvas with bounded inspector).
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
    if state.history.is_empty() {
        state.history.push(state.current_path.clone());
        state.history_index = 0;
    }

    let current_node = root.find_node_by_path(&state.current_path).unwrap_or(root);

    // 1. Breadcrumbs Bar with Windows Explorer-style Back & Forward Navigation
    egui::Frame::none()
        .fill(t.bg_elevated)
        .rounding(8.0_f32)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .inner_margin(Margin::symmetric(10.0, 6.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let parent_opt = state.current_path.parent().map(|p| p.to_path_buf());
                let can_go_up = if let Some(parent) = &parent_opt {
                    parent.starts_with(&root.path) || *parent == root.path
                } else {
                    false
                };
                let up_btn =
                    egui::Button::new(RichText::new("↰ Up").size(11.0).color(if can_go_up {
                        t.text
                    } else {
                        t.text_muted.gamma_multiply(0.35)
                    }))
                    .fill(Color32::TRANSPARENT);
                if ui
                    .add_enabled(can_go_up, up_btn)
                    .on_hover_text("Navigate to parent directory")
                    .clicked()
                {
                    if let Some(parent) = parent_opt {
                        state.navigate_to(parent);
                    }
                }

                let is_at_root = state.current_path == root.path;
                let root_btn = egui::Button::new(
                    RichText::new("🏠 Root")
                        .size(11.0)
                        .strong()
                        .color(if is_at_root { t.accent } else { t.text }),
                )
                .fill(Color32::TRANSPARENT);
                if ui
                    .add(root_btn)
                    .on_hover_text("Return to scan root")
                    .clicked()
                {
                    state.navigate_to(root.path.clone());
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

                    if ui
                        .selectable_label(is_active, RichText::new(&c_str).size(11.5).strong())
                        .clicked()
                    {
                        clicked_crumb = Some(target_clone);
                    }
                    ui.label(RichText::new(">").size(10.0).color(t.text_muted));
                }

                if let Some(p) = clicked_crumb {
                    state.navigate_to(p);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!(
                            "Allocated: {}",
                            current_node.formatted_allocated_size()
                        ))
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

    ui.add_space(4.0);

    // Tree & Space Hogs Table View

    // Interactive category distribution chips bar (top row)
    let mut cat_sizes = [0u64; FileCategory::ALL.len()];
    for child in &current_node.children {
        if let Some(pos) = FileCategory::ALL.iter().position(|c| *c == child.category) {
            cat_sizes[pos] += child.size;
        }
    }
    let total_children_size: u64 = cat_sizes.iter().sum();

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(6.0, 4.0);
        let is_all_sel = state.category_filter.is_none();
        let all_btn = ui.selectable_label(
            is_all_sel,
            RichText::new(format!("◆ All ({})", format_size(total_children_size)))
                .size(11.0)
                .strong(),
        );
        if all_btn.clicked() {
            state.category_filter = None;
        }

        for (idx, cat) in FileCategory::ALL.iter().enumerate() {
            let sz = cat_sizes[idx];
            if sz == 0 {
                continue;
            }
            let pct = if total_children_size > 0 {
                (sz as f64 / total_children_size as f64) * 100.0
            } else {
                0.0
            };
            let is_sel = state.category_filter == Some(*cat);
            let chip_text = format!(
                "{} {} ({} · {:.1}%)",
                cat.icon(),
                cat.display_name(),
                format_size(sz),
                pct
            );
            let chip_btn = ui.selectable_label(
                is_sel,
                RichText::new(chip_text).size(11.0).color(if is_sel {
                    cat.color()
                } else {
                    t.text
                }),
            );
            if chip_btn.clicked() {
                if is_sel {
                    state.category_filter = None;
                } else {
                    state.category_filter = Some(*cat);
                }
            }
        }
    });

    ui.add_space(4.0);

    // Filter, Mode Preset & Sorting Toolbar
    ui.horizontal(|ui| {
        // Preset Sorting Toggle: Hierarchical vs Space Hogs
        let is_hogs_mode =
            state.sort_column == SortColumn::Size && state.sort_order == SortOrder::Descending;
        let hogs_btn = ui.selectable_label(
            is_hogs_mode,
            RichText::new("⚠ Space Hogs (Size ↓)")
                .size(11.0)
                .strong()
                .color(if is_hogs_mode {
                    t.warning
                } else {
                    t.text_muted
                }),
        );
        if hogs_btn.clicked() {
            state.sort_column = SortColumn::Size;
            state.sort_order = SortOrder::Descending;
        }

        let is_tree_mode =
            state.sort_column == SortColumn::Name && state.sort_order == SortOrder::Ascending;
        let tree_btn = ui.selectable_label(
            is_tree_mode,
            RichText::new("🌲 Hierarchy (Name ↑)").size(11.0).strong(),
        );
        if tree_btn.clicked() {
            state.sort_column = SortColumn::Name;
            state.sort_order = SortOrder::Ascending;
        }

        ui.separator();

        ui.label(RichText::new("Filter:").size(11.0).color(t.text_muted));
        ui.add_sized(
            [140.0, 24.0],
            egui::TextEdit::singleline(&mut state.search_filter).hint_text("Search name/ext..."),
        );
        if !state.search_filter.is_empty() && ui.small_button("✕").clicked() {
            state.search_filter.clear();
        }

        ui.label(RichText::new("Min:").size(11.0).color(t.text_muted));
        ui.add(
            egui::DragValue::new(&mut state.min_size_filter_mb)
                .speed(1.0)
                .range(0.0..=10_000.0)
                .suffix(" MB"),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(cat) = state.category_filter {
                ui.label(
                    RichText::new(format!("Filtered: {}", cat.display_name()))
                        .size(10.5)
                        .color(cat.color()),
                );
                if ui.small_button("Clear").clicked() {
                    state.category_filter = None;
                }
            }
        });
    });

    ui.add_space(4.0);

    let min_bytes = (state.min_size_filter_mb * 1024.0 * 1024.0) as u64;
    let mut display_children: Vec<&FileNode> = current_node
        .children
        .iter()
        .filter(|c| {
            if c.size < min_bytes {
                return false;
            }
            if let Some(cat) = state.category_filter {
                if c.category != cat {
                    return false;
                }
            }
            if !state.search_filter.is_empty() {
                return c
                    .name
                    .to_lowercase()
                    .contains(&state.search_filter.to_lowercase());
            }
            true
        })
        .collect();

    let sort_col = state.sort_column;
    let sort_ord = state.sort_order;
    display_children.sort_by(|a, b| {
        let cmp = match sort_col {
            SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortColumn::Size => a.size.cmp(&b.size),
            SortColumn::ItemCount => a.total_items().cmp(&b.total_items()),
            SortColumn::Type => {
                let type_a = if a.is_dir {
                    "Folder"
                } else {
                    a.category.display_name()
                };
                let type_b = if b.is_dir {
                    "Folder"
                } else {
                    b.category.display_name()
                };
                type_a.cmp(type_b)
            }
            SortColumn::Modified => a.modified.cmp(&b.modified),
        };
        if sort_ord == SortOrder::Descending {
            cmp.reverse()
        } else {
            cmp
        }
    });

    let total_parent_size = current_node.size.max(1);
    let mut navigate_to = None;
    let total_rows = display_children.len();

    let bottom_bar_height = 28.0;
    let available_height = (ui.available_height() - bottom_bar_height).max(100.0);
    let scrollbar_margin = ui.spacing().scroll.bar_width + 14.0;
    let avail_w = (ui.available_width() - scrollbar_margin).max(100.0);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .max_height(available_height)
        .max_width(avail_w)
        .show(ui, |ui| {
            TableBuilder::new(ui)
                .striped(true)
                .resizable(false)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::remainder().clip(true)) // Name & path middle-truncated
                .column(Column::exact(85.0)) // Apparent Size
                .column(Column::exact(95.0)) // Usage Progress Bar
                .column(Column::exact(80.0)) // Type
                .column(Column::exact(125.0)) // Actions Toolbar
                .header(24.0, |mut header| {
                    header.col(|ui| {
                        if ui
                            .selectable_label(
                                sort_col == SortColumn::Name,
                                RichText::new("Name / Path").strong().color(t.text_muted),
                            )
                            .clicked()
                        {
                            toggle_sort(
                                &mut state.sort_column,
                                &mut state.sort_order,
                                SortColumn::Name,
                            );
                        }
                    });
                    header.col(|ui| {
                        if ui
                            .selectable_label(
                                sort_col == SortColumn::Size,
                                RichText::new("Size").strong().color(t.text_muted),
                            )
                            .clicked()
                        {
                            toggle_sort(
                                &mut state.sort_column,
                                &mut state.sort_order,
                                SortColumn::Size,
                            );
                        }
                    });
                    header.col(|ui| {
                        ui.label(RichText::new("Usage").strong().color(t.text_muted));
                    });
                    header.col(|ui| {
                        if ui
                            .selectable_label(
                                sort_col == SortColumn::Type,
                                RichText::new("Type").strong().color(t.text_muted),
                            )
                            .clicked()
                        {
                            toggle_sort(
                                &mut state.sort_column,
                                &mut state.sort_order,
                                SortColumn::Type,
                            );
                        }
                    });
                    header.col(|ui| {
                        ui.label(RichText::new("Actions").strong().color(t.text_muted));
                    });
                })
                .body(|body| {
                    body.rows(28.0, total_rows, |mut row| {
                        let idx = row.index();
                        if let Some(child) = display_children.get(idx) {
                            let is_selected = match &state.selected_node {
                                Some(p) => *p == child.path,
                                None => false,
                            };
                            row.set_selected(is_selected);

                            // Col 1: Name / Path (absorbs remainder, clipped)
                            row.col(|ui| {
                                let icon = if child.is_dir {
                                    "📁"
                                } else {
                                    child.category.icon()
                                };
                                let name_label = format!("{} {}", icon, child.name);
                                let btn = ui
                                    .add(egui::SelectableLabel::new(
                                        is_selected,
                                        RichText::new(name_label).size(11.5).color(t.text),
                                    ))
                                    .on_hover_text(child.path.display().to_string());

                                if btn.clicked() {
                                    state.selected_node = Some(child.path.clone());
                                }
                                if btn.double_clicked() {
                                    if child.is_dir {
                                        navigate_to = Some(child.path.clone());
                                    } else {
                                        let _ = FileActions::open_file(&child.path);
                                    }
                                }
                            });

                            // Col 2: Apparent Size (95px)
                            row.col(|ui| {
                                ui.label(
                                    RichText::new(child.formatted_size())
                                        .size(11.0)
                                        .monospace()
                                        .color(t.text),
                                );
                            });

                            // Col 3: Usage (% of parent with inline visual bar, 100px)
                            row.col(|ui| {
                                let pct =
                                    (child.size as f32 / total_parent_size as f32).clamp(0.0, 1.0);
                                let bar_w = 48.0_f32;
                                let (bar_rect, _) =
                                    ui.allocate_exact_size(Vec2::new(bar_w, 6.0), Sense::hover());
                                ui.painter().rect_filled(bar_rect, 2.0_f32, t.bg_input);
                                let filled_w = (bar_w * pct).max(1.0);
                                let filled_rect = Rect::from_min_size(
                                    bar_rect.min,
                                    Vec2::new(filled_w, bar_rect.height()),
                                );
                                ui.painter().rect_filled(
                                    filled_rect,
                                    2.0_f32,
                                    if child.is_dir {
                                        t.accent
                                    } else {
                                        child.category.color()
                                    },
                                );
                                ui.label(
                                    RichText::new(format!("{:.0}%", pct * 100.0))
                                        .size(10.5)
                                        .monospace()
                                        .color(t.text_muted),
                                );
                            });

                            // Col 4: Category badge (85px)
                            row.col(|ui| {
                                let type_s = if child.is_dir {
                                    "Folder"
                                } else {
                                    child.category.display_name()
                                };
                                let color = if child.is_dir {
                                    t.text_secondary
                                } else {
                                    child.category.color()
                                };
                                ui.label(RichText::new(type_s).size(10.5).color(color));
                            });

                            // Col 5: Actions toolbar (125px)
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 2.0;
                                    ui.spacing_mut().button_padding = Vec2::new(4.0, 2.0);

                                    if child.is_dir {
                                        if ui
                                            .small_button("→")
                                            .on_hover_text("Explore folder")
                                            .clicked()
                                        {
                                            navigate_to = Some(child.path.clone());
                                        }
                                    } else {
                                        if ui
                                            .small_button("📄")
                                            .on_hover_text("Open file")
                                            .clicked()
                                        {
                                            let _ = FileActions::open_file(&child.path);
                                        }
                                    }

                                    if ui
                                        .small_button("📂")
                                        .on_hover_text("Reveal in file manager")
                                        .clicked()
                                    {
                                        let _ = FileActions::open_in_file_manager(&child.path);
                                    }

                                    if ui.small_button("📋").on_hover_text("Copy path").clicked()
                                    {
                                        let p_str = child.path.display().to_string();
                                        ui.output_mut(|o| o.copied_text = p_str);
                                        modals.show_toast("Copied path".to_string(), false);
                                    }

                                    if ui
                                        .small_button("🗑")
                                        .on_hover_text("Move to trash")
                                        .clicked()
                                    {
                                        modals.pending_confirm = Some(ConfirmAction::MoveToTrash(
                                            child.path.clone(),
                                            child.size,
                                        ));
                                    }
                                });
                            });
                        }
                    });
                });
        });

    if let Some(target) = navigate_to {
        state.navigate_to(target.clone());
        state.treemap_state.current_path = target;
    }

    ui.add_space(2.0);
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
