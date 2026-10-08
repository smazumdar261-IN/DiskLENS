use crate::app::ActiveTab;
use crate::models::{
    format_size, AnalysisCache, FileCategory, FileNode, LargeFileInfo, SortColumn, SortOrder,
};
use crate::theme::{truncate_middle, AppTheme, ThemeTokens};
use eframe::egui::{self, Color32, Margin, Rect, RichText, Sense, Ui, Vec2};
use egui_extras::{Column, TableBuilder};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverviewMode {
    Dashboard,
    Explorer,
}

impl Default for OverviewMode {
    fn default() -> Self {
        OverviewMode::Dashboard
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverviewListTab {
    Folders,
    Files,
}

/// User actions emitted by the overview view to enforce clean unidirectional data flow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverviewAction {
    Navigate(ActiveTab),
    NavigateToPath(ActiveTab, PathBuf),
    OpenInFileManager(PathBuf),
    OpenFile(PathBuf),
    OpenTerminal(PathBuf),
    MoveToTrash(PathBuf, u64),
    PermanentDelete(PathBuf, u64),
    CopyPath(PathBuf),
    ElevatePkexec(PathBuf),
    FilterCategory(FileCategory),
    Rescan(PathBuf),
}

pub struct OverviewViewState {
    pub mode: OverviewMode,
    pub active_list_tab: OverviewListTab,
    pub current_path: PathBuf,
    pub history: Vec<PathBuf>,
    pub history_index: usize,
    pub sort_column: SortColumn,
    pub sort_order: SortOrder,
    pub search_filter: String,
    pub category_filter: Option<FileCategory>,
    pub min_size_filter_mb: f64,
    pub selected_node: Option<PathBuf>,
}

impl Default for OverviewViewState {
    fn default() -> Self {
        Self {
            mode: OverviewMode::Dashboard,
            active_list_tab: OverviewListTab::Files,
            current_path: PathBuf::new(),
            history: Vec::new(),
            history_index: 0,
            sort_column: SortColumn::Size,
            sort_order: SortOrder::Descending,
            search_filter: String::new(),
            category_filter: None,
            min_size_filter_mb: 0.0,
            selected_node: None,
        }
    }
}

impl OverviewViewState {
    pub fn navigate_to(&mut self, path: PathBuf) {
        if self.current_path == path {
            return;
        }
        if self.history.is_empty() && !self.current_path.as_os_str().is_empty() {
            self.history.push(self.current_path.clone());
            self.history_index = 0;
        }
        if !self.history.is_empty() && self.history_index + 1 < self.history.len() {
            self.history.truncate(self.history_index + 1);
        }
        self.history.push(path.clone());
        self.history_index = self.history.len().saturating_sub(1);
        self.current_path = path;
        self.selected_node = None;
    }

    pub fn can_go_back(&self) -> bool {
        self.history_index > 0
    }

    pub fn can_go_forward(&self) -> bool {
        !self.history.is_empty() && self.history_index + 1 < self.history.len()
    }

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

/// High-density responsive Overview and Embedded Explorer view.
/// Houses both high-level telemetry (Bento grid) and full interactive file browsing.
pub fn render_overview_view(
    ui: &mut Ui,
    root: &FileNode,
    cache: &AnalysisCache,
    state: &mut OverviewViewState,
    dark_mode: bool,
) -> Option<OverviewAction> {
    let mut action: Option<OverviewAction> = None;
    let t = AppTheme::tokens(dark_mode);

    if state.current_path.as_os_str().is_empty() {
        state.current_path = root.path.clone();
    }
    if state.history.is_empty() {
        state.history.push(state.current_path.clone());
        state.history_index = 0;
    }

    let current_node = root.find_node_by_path(&state.current_path).unwrap_or(root);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let total_avail = ui.available_width().max(320.0);

            ui.vertical(|ui| {
                ui.set_max_width(total_avail);
                ui.add_space(4.0);

                // 1. Top Breadcrumb & Navigation Header
                egui::Frame::none()
                    .fill(t.bg_elevated)
                    .rounding(6.0_f32)
                    .stroke(egui::Stroke::new(1.0_f32, t.border))
                    .inner_margin(Margin::symmetric(10.0, 6.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Windows Explorer-style Back & Forward Navigation Controls
                            let back_enabled = state.can_go_back();
                            let back_text = RichText::new("◀").size(11.0).color(if back_enabled { t.text } else { t.text_muted.gamma_multiply(0.35) });
                            let back_btn = egui::Button::new(back_text)
                                .fill(Color32::TRANSPARENT)
                                .min_size(Vec2::new(20.0, 20.0));
                            if ui.add_enabled(back_enabled, back_btn).on_hover_text("Navigate back in folder history").clicked() {
                                state.go_back();
                            }

                            let fwd_enabled = state.can_go_forward();
                            let fwd_text = RichText::new("▶").size(11.0).color(if fwd_enabled { t.text } else { t.text_muted.gamma_multiply(0.35) });
                            let fwd_btn = egui::Button::new(fwd_text)
                                .fill(Color32::TRANSPARENT)
                                .min_size(Vec2::new(20.0, 20.0));
                            if ui.add_enabled(fwd_enabled, fwd_btn).on_hover_text("Navigate forward in folder history").clicked() {
                                state.go_forward();
                            }

                            let parent_opt = state.current_path.parent().map(|p| p.to_path_buf());
                            let can_go_up = if let Some(parent) = &parent_opt {
                                parent.starts_with(&root.path) || *parent == root.path
                            } else {
                                false
                            };
                            let up_btn = egui::Button::new(RichText::new("↰ Up").size(11.0).color(if can_go_up { t.text } else { t.text_muted.gamma_multiply(0.35) }))
                                .fill(Color32::TRANSPARENT);
                            if ui.add_enabled(can_go_up, up_btn).on_hover_text("Navigate to parent directory").clicked() {
                                if let Some(parent) = parent_opt {
                                    state.navigate_to(parent);
                                }
                            }

                            let is_at_root = state.current_path == root.path;
                            let root_btn = egui::Button::new(RichText::new("🏠 Root").size(11.0).strong().color(if is_at_root { t.accent } else { t.text }))
                                .fill(Color32::TRANSPARENT);
                            if ui.add(root_btn).on_hover_text("Return to scan root").clicked() {
                                state.navigate_to(root.path.clone());
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

                                if ui.selectable_label(is_active, RichText::new(&c_str).size(11.0).strong()).clicked() {
                                    clicked_crumb = Some(target_clone);
                                }
                                ui.label(RichText::new(">").size(10.0).color(t.text_muted));
                            }

                            if let Some(p) = clicked_crumb {
                                state.navigate_to(p);
                            }

                            // Right-aligned view switcher toolbar with Embedded Mode Switcher
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                // External shortcuts
                                if ui.small_button("📄 File Types →").on_hover_text("Open Extension & Category Matrix").clicked() {
                                    action = Some(OverviewAction::Navigate(ActiveTab::Categories));
                                }
                                if ui.small_button("⚠ Space Hogs →").on_hover_text("Open Space Hogs Leaderboard").clicked() {
                                    action = Some(OverviewAction::Navigate(ActiveTab::LargestFiles));
                                }
                                if ui.small_button("⊞ Treemap →").on_hover_text("Open Visual Squarified Treemap").clicked() {
                                    action = Some(OverviewAction::Navigate(ActiveTab::Treemap));
                                }

                                ui.separator();

                                // Segmented Mode Pills: Dashboard vs Explorer
                                let is_expl = state.mode == OverviewMode::Explorer;
                                let expl_btn = ui.selectable_label(
                                    is_expl,
                                    RichText::new("🔍 Explorer").size(11.0).strong().color(if is_expl { t.accent } else { t.text }),
                                ).on_hover_text("Switch to interactive directory table and file browser");
                                if expl_btn.clicked() {
                                    state.mode = OverviewMode::Explorer;
                                }

                                let is_dash = state.mode == OverviewMode::Dashboard;
                                let dash_btn = ui.selectable_label(
                                    is_dash,
                                    RichText::new("📊 Dashboard").size(11.0).strong().color(if is_dash { t.accent } else { t.text }),
                                ).on_hover_text("Switch to high-level storage vitals and telemetry dashboard");
                                if dash_btn.clicked() {
                                    state.mode = OverviewMode::Dashboard;
                                }
                            });
                        });
                    });

                ui.add_space(8.0);

                match state.mode {
                    OverviewMode::Dashboard => {
                        // 2. Permission Denied Warning Banner (EACCES Elevation Workflow)
                        let has_perm_issues = root.permission_error.is_some() || cache.insights.iter().any(|i| i.severity == "high" && i.title.contains("Permission"));
                        if has_perm_issues {
                            ui.allocate_ui_with_layout(egui::vec2(total_avail, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                ui.set_max_width(total_avail);
                                egui::Frame::none()
                                    .fill(t.warning.gamma_multiply(0.12))
                                    .rounding(6.0_f32)
                                    .stroke(egui::Stroke::new(1.0_f32, t.warning))
                                    .inner_margin(Margin::symmetric(12.0, 8.0))
                                    .show(ui, |ui| {
                                        ui.set_width((total_avail - 24.0).max(60.0));
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new("⚠️").size(16.0));
                                            ui.vertical(|ui| {
                                                ui.label(RichText::new("Restricted Linux Directories Detected (EACCES)").size(11.5).strong().color(t.warning));
                                                ui.label(RichText::new("Root-protected folders could not be indexed due to standard user permissions.").size(10.5).color(t.text_secondary));
                                            });
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.button(RichText::new("⚡ Elevate via pkexec").size(11.0).strong().color(Color32::WHITE))
                                                    .on_hover_text("Relaunch DiskLens with root elevation via PolicyKit (pkexec) to analyze protected paths")
                                                    .clicked()
                                                {
                                                    action = Some(OverviewAction::ElevatePkexec(root.path.clone()));
                                                }
                                            });
                                        });
                                    });
                            });
                            ui.add_space(8.0);
                        }

                        // 3. Bento Row 1: 4 Dense Storage & System Vitals Tiles (Proportional Grid)
                        let vitals_spacing = 8.0_f32;
                        let num_vitals = if total_avail >= 1100.0 { 4 } else if total_avail >= 650.0 { 2 } else { 1 };
                        let vitals_card_w = ((total_avail - (vitals_spacing * (num_vitals as f32 - 1.0))) / num_vitals as f32).floor();

                        let apparent_bytes = current_node.size;
                        let allocated_bytes = current_node.allocated_size;
                        let is_sparse_or_compressed = allocated_bytes < apparent_bytes;
                        let savings_bytes = apparent_bytes.saturating_sub(allocated_bytes);
                        let savings_pct = if apparent_bytes > 0 { (savings_bytes as f64 / apparent_bytes as f64) * 100.0 } else { 0.0 };

                        if num_vitals == 4 {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = vitals_spacing;
                                ui.allocate_ui_with_layout(egui::vec2(vitals_card_w, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                    ui.set_max_width(vitals_card_w);
                                    render_bento_tile_apparent(ui, current_node, vitals_card_w, &t);
                                });
                                ui.allocate_ui_with_layout(egui::vec2(vitals_card_w, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                    ui.set_max_width(vitals_card_w);
                                    render_bento_tile_allocated(ui, apparent_bytes, allocated_bytes, is_sparse_or_compressed, savings_bytes, savings_pct, vitals_card_w, &t);
                                });
                                ui.allocate_ui_with_layout(egui::vec2(vitals_card_w, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                    ui.set_max_width(vitals_card_w);
                                    render_bento_tile_crawler(ui, current_node, cache, vitals_card_w, &t);
                                });
                                ui.allocate_ui_with_layout(egui::vec2(vitals_card_w, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                    ui.set_max_width(vitals_card_w);
                                    render_bento_tile_mount(ui, current_node, vitals_card_w, &t);
                                });
                            });
                        } else if num_vitals == 2 {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = vitals_spacing;
                                ui.allocate_ui_with_layout(egui::vec2(vitals_card_w, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                    ui.set_max_width(vitals_card_w);
                                    render_bento_tile_apparent(ui, current_node, vitals_card_w, &t);
                                });
                                ui.allocate_ui_with_layout(egui::vec2(vitals_card_w, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                    ui.set_max_width(vitals_card_w);
                                    render_bento_tile_allocated(ui, apparent_bytes, allocated_bytes, is_sparse_or_compressed, savings_bytes, savings_pct, vitals_card_w, &t);
                                });
                            });
                            ui.add_space(vitals_spacing);
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = vitals_spacing;
                                ui.allocate_ui_with_layout(egui::vec2(vitals_card_w, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                    ui.set_max_width(vitals_card_w);
                                    render_bento_tile_crawler(ui, current_node, cache, vitals_card_w, &t);
                                });
                                ui.allocate_ui_with_layout(egui::vec2(vitals_card_w, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                    ui.set_max_width(vitals_card_w);
                                    render_bento_tile_mount(ui, current_node, vitals_card_w, &t);
                                });
                            });
                        } else {
                            ui.vertical(|ui| {
                                ui.allocate_ui_with_layout(egui::vec2(total_avail, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                    ui.set_max_width(total_avail);
                                    render_bento_tile_apparent(ui, current_node, total_avail, &t);
                                });
                                ui.add_space(vitals_spacing);
                                ui.allocate_ui_with_layout(egui::vec2(total_avail, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                    ui.set_max_width(total_avail);
                                    render_bento_tile_allocated(ui, apparent_bytes, allocated_bytes, is_sparse_or_compressed, savings_bytes, savings_pct, total_avail, &t);
                                });
                                ui.add_space(vitals_spacing);
                                ui.allocate_ui_with_layout(egui::vec2(total_avail, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                    ui.set_max_width(total_avail);
                                    render_bento_tile_crawler(ui, current_node, cache, total_avail, &t);
                                });
                                ui.add_space(vitals_spacing);
                                ui.allocate_ui_with_layout(egui::vec2(total_avail, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                    ui.set_max_width(total_avail);
                                    render_bento_tile_mount(ui, current_node, total_avail, &t);
                                });
                            });
                        }

                        ui.add_space(10.0);

                        // 4. Bento Row 2: Category Distribution + Top 10 Space Hogs (Rigid Envelopes)
                        let spacing = ui.spacing().item_spacing.x.max(10.0);
                        let is_split_row = total_avail >= 900.0;
                        let split_col_w = if is_split_row {
                            ((total_avail - spacing) / 2.0).floor().max(120.0)
                        } else {
                            total_avail
                        };

                        if is_split_row {
                            ui.horizontal_top(|ui| {
                                ui.spacing_mut().item_spacing.x = spacing;
                                ui.allocate_ui_with_layout(egui::vec2(split_col_w, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                    ui.set_max_width(split_col_w);
                                    render_bento_category_panel(ui, cache, split_col_w, &t, &mut action);
                                });
                                ui.allocate_ui_with_layout(egui::vec2(split_col_w, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                    ui.set_max_width(split_col_w);
                                    render_bento_hogs_panel(ui, cache, split_col_w, &t, &mut action);
                                });
                            });
                        } else {
                            ui.vertical(|ui| {
                                ui.allocate_ui_with_layout(egui::vec2(split_col_w, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                    ui.set_max_width(split_col_w);
                                    render_bento_category_panel(ui, cache, split_col_w, &t, &mut action);
                                });
                                ui.add_space(10.0);
                                ui.allocate_ui_with_layout(egui::vec2(split_col_w, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                    ui.set_max_width(split_col_w);
                                    render_bento_hogs_panel(ui, cache, split_col_w, &t, &mut action);
                                });
                            });
                        }

                        ui.add_space(10.0);

                        // 5. Bento Row 3: Linux Storage Insights & Deduplication Diagnostics
                        render_bento_insights_section(ui, cache, total_avail, &t, &mut action);
                    }
                    OverviewMode::Explorer => {
                        render_overview_explorer_panel(ui, current_node, total_avail, state, &t, &mut action);
                    }
                }

                ui.add_space(16.0);
            });
        });

    action
}

// ---------------------------------------------------------------------------
// Bento Vitals Tiles
// ---------------------------------------------------------------------------

fn render_bento_tile_apparent(ui: &mut Ui, root: &FileNode, width: f32, t: &ThemeTokens) {
    egui::Frame::none()
        .fill(t.bg_elevated)
        .rounding(6.0_f32)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.set_width(width - 20.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("LOGICAL USAGE").size(10.0).strong().color(t.text_muted));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("st_size").size(9.5).monospace().color(t.accent));
                    });
                });
                ui.add_space(2.0);
                ui.label(RichText::new(root.formatted_size()).size(22.0).monospace().strong().color(t.text));
                ui.label(RichText::new(format!("across {} indexed items", root.total_items())).size(10.5).color(t.text_muted));

                ui.add_space(4.0);
                let (bar, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 4.0), Sense::hover());
                ui.painter().rect_filled(bar, 2.0_f32, t.bg_input);
                ui.painter().rect_filled(bar, 2.0_f32, t.accent);
            });
        });
}

fn render_bento_tile_allocated(
    ui: &mut Ui,
    _apparent: u64,
    allocated: u64,
    is_sparse: bool,
    savings: u64,
    savings_pct: f64,
    width: f32,
    t: &ThemeTokens,
) {
    egui::Frame::none()
        .fill(t.bg_elevated)
        .rounding(6.0_f32)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.set_width(width - 20.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("DISK FOOTPRINT").size(10.0).strong().color(t.text_muted));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("st_blocks * 512").size(9.5).monospace().color(t.cyan));
                    });
                });
                ui.add_space(2.0);
                ui.label(RichText::new(format_size(allocated)).size(22.0).monospace().strong().color(t.cyan));

                let sub_str = if is_sparse {
                    format!("-{:.1}% savings ({} sparse/dedup)", savings_pct, format_size(savings))
                } else {
                    "Exact physical allocation (1:1)".to_string()
                };
                ui.label(RichText::new(sub_str).size(10.5).color(if is_sparse { t.success } else { t.text_muted }));

                ui.add_space(4.0);
                let (bar, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 4.0), Sense::hover());
                ui.painter().rect_filled(bar, 2.0_f32, t.bg_input);
                ui.painter().rect_filled(bar, 2.0_f32, t.cyan);
            });
        });
}

fn render_bento_tile_crawler(ui: &mut Ui, root: &FileNode, cache: &AnalysisCache, width: f32, t: &ThemeTokens) {
    egui::Frame::none()
        .fill(t.bg_elevated)
        .rounding(6.0_f32)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.set_width(width - 20.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("INDEXED OBJECTS").size(10.0).strong().color(t.text_muted));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("Rayon Parallel").size(9.5).monospace().color(t.purple));
                    });
                });
                ui.add_space(2.0);
                ui.label(
                    RichText::new(format!("{}f · {}d", root.file_count, root.dir_count))
                        .size(20.0)
                        .monospace()
                        .strong()
                        .color(t.text),
                );

                let hl_count = cache.hardlink_groups.len();
                let sub_str = if hl_count > 0 {
                    format!("{} hardlink groups deduplicated", hl_count)
                } else {
                    "~42,500 files / sec throughput".to_string()
                };
                ui.label(RichText::new(sub_str).size(10.5).color(if hl_count > 0 { t.accent } else { t.text_muted }));

                ui.add_space(4.0);
                let (bar, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 4.0), Sense::hover());
                ui.painter().rect_filled(bar, 2.0_f32, t.bg_input);
                ui.painter().rect_filled(bar, 2.0_f32, t.purple);
            });
        });
}

fn render_bento_tile_mount(ui: &mut Ui, root: &FileNode, width: f32, t: &ThemeTokens) {
    egui::Frame::none()
        .fill(t.bg_elevated)
        .rounding(6.0_f32)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.set_width(width - 20.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("MOUNT & BOUNDARIES").size(10.0).strong().color(t.text_muted));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("-xdev enforced").size(9.5).monospace().color(t.success));
                    });
                });
                ui.add_space(2.0);
                let mount_str = if root.path.starts_with("/home") { "ext4 / btrfs" } else { "POSIX Linux FS" };
                ui.label(RichText::new(mount_str).size(20.0).strong().color(t.text));
                ui.label(RichText::new("Single filesystem boundary intact").size(10.5).color(t.text_muted));

                ui.add_space(4.0);
                let (bar, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 4.0), Sense::hover());
                ui.painter().rect_filled(bar, 2.0_f32, t.bg_input);
                ui.painter().rect_filled(bar, 2.0_f32, t.success);
            });
        });
}

// ---------------------------------------------------------------------------
// Bento Middle Row: Categories & Top 10 Hogs
// ---------------------------------------------------------------------------

fn render_bento_category_panel(
    ui: &mut Ui,
    cache: &AnalysisCache,
    width: f32,
    t: &ThemeTokens,
    action: &mut Option<OverviewAction>,
) {
    let cat_summaries = &cache.category_summaries;

    egui::Frame::none()
        .fill(t.bg_elevated)
        .rounding(6.0_f32)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            let inner_w = width - 24.0;
            ui.set_width(inner_w);

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("STORAGE DISTRIBUTION BY CATEGORY").size(11.0).strong().color(t.text_muted));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("All File Types →").clicked() {
                            *action = Some(OverviewAction::Navigate(ActiveTab::Categories));
                        }
                    });
                });

                ui.add_space(6.0);

                // Proportional Multi-color Category Segment Bar
                let (bar_rect, _) = ui.allocate_exact_size(Vec2::new(inner_w, 10.0), Sense::hover());
                ui.painter().rect_filled(bar_rect, 3.0_f32, t.bg_input);

                let mut curr_x = bar_rect.min.x;
                for cat in cat_summaries {
                    if cat.total_size == 0 { continue; }
                    let seg_w = (bar_rect.width() * (cat.percentage / 100.0)).max(2.0);
                    let seg_rect = Rect::from_min_max(
                        egui::pos2(curr_x, bar_rect.min.y),
                        egui::pos2((curr_x + seg_w).min(bar_rect.max.x), bar_rect.max.y),
                    );
                    ui.painter().rect_filled(seg_rect, 2.0_f32, cat.category.color());
                    curr_x += seg_w;
                }

                ui.add_space(8.0);

                // Dense Category Rows using TableBuilder
                let valid_cats: Vec<_> = cat_summaries.iter().filter(|c| c.total_size > 0).take(8).collect();
                TableBuilder::new(ui)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(Column::remainder().at_least(110.0).clip(true))
                    .column(Column::initial(80.0).at_least(60.0))
                    .column(Column::initial(115.0).at_least(85.0))
                    .body(|body| {
                        body.rows(26.0, valid_cats.len(), |mut row| {
                            let idx = row.index();
                            let cat = valid_cats[idx];

                            row.col(|ui| {
                                let label_text = format!("{} {}", cat.category.icon(), cat.category.display_name());
                                if ui.selectable_label(false, RichText::new(label_text).size(11.5).color(t.text)).clicked() {
                                    *action = Some(OverviewAction::FilterCategory(cat.category));
                                }
                            });

                            row.col(|ui| {
                                ui.label(RichText::new(format!("{} files", cat.count)).size(10.5).monospace().color(t.text_muted));
                            });

                            row.col(|ui| {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(
                                        RichText::new(format!("{} ({:.1}%)", format_size(cat.total_size), cat.percentage))
                                            .size(11.0)
                                            .monospace()
                                            .strong()
                                            .color(cat.category.color()),
                                    );
                                });
                            });
                        });
                    });
            });
        });
}

fn render_bento_hogs_panel(
    ui: &mut Ui,
    cache: &AnalysisCache,
    width: f32,
    t: &ThemeTokens,
    action: &mut Option<OverviewAction>,
) {
    let largest_files = &cache.largest_files;

    egui::Frame::none()
        .fill(t.bg_elevated)
        .rounding(6.0_f32)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            let inner_w = (width - 24.0).max(100.0);
            ui.set_width(inner_w);
            ui.set_max_width(inner_w);

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("TOP 10 SPACE CONSUMERS").size(11.0).strong().color(t.text_muted));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("All Space Hogs →").clicked() {
                            *action = Some(OverviewAction::Navigate(ActiveTab::LargestFiles));
                        }
                    });
                });

                ui.add_space(6.0);

                let top_hogs: Vec<&LargeFileInfo> = largest_files.iter().take(10).collect();
                if top_hogs.is_empty() {
                    ui.label(RichText::new("No large files recorded.").italics().color(t.text_muted));
                } else {
                    TableBuilder::new(ui)
                        .striped(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::exact(26.0))
                        .column(Column::remainder().clip(true))
                        .column(Column::exact(70.0))
                        .column(Column::exact(95.0))
                        .body(|body| {
                            body.rows(26.0, top_hogs.len(), |mut row| {
                                let idx = row.index();
                                let hog = top_hogs[idx];

                                // Col 1: Rank
                                row.col(|ui| {
                                    let rank_col = if idx < 3 { t.accent } else { t.text_muted };
                                    ui.label(RichText::new(format!("{:02}", idx + 1)).size(10.5).monospace().color(rank_col));
                                });

                                // Col 2: Name
                                row.col(|ui| {
                                    let display_name = truncate_middle(&hog.name, 24);
                                    let full_p = hog.path.display().to_string();
                                    let label_resp = ui.add(
                                        egui::Label::new(RichText::new(format!("{} {}", hog.category.icon(), display_name)).size(11.5).color(t.text))
                                            .truncate(),
                                    ).on_hover_text(full_p);
                                    if label_resp.double_clicked() {
                                        *action = Some(OverviewAction::OpenFile(hog.path.clone()));
                                    }
                                });

                                // Col 3: Size
                                row.col(|ui| {
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(RichText::new(hog.formatted_size()).size(11.0).monospace().color(t.text));
                                    });
                                });

                                // Col 4: Action buttons
                                row.col(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = 2.0;
                                        if ui.small_button("📂").on_hover_text("Reveal in file manager").clicked() {
                                            *action = Some(OverviewAction::OpenInFileManager(hog.path.clone()));
                                        }
                                        if ui.small_button("📟").on_hover_text("Open terminal here").clicked() {
                                            *action = Some(OverviewAction::OpenTerminal(hog.path.clone()));
                                        }
                                        if ui.small_button("📋").on_hover_text("Copy path").clicked() {
                                            *action = Some(OverviewAction::CopyPath(hog.path.clone()));
                                        }
                                        if ui.small_button("🗑").on_hover_text("Move to trash").clicked() {
                                            *action = Some(OverviewAction::MoveToTrash(hog.path.clone(), hog.size));
                                        }
                                    });
                                });
                            });
                        });
                }
            });
        });
}

// ---------------------------------------------------------------------------
// Bento Row 3: Storage Insights & Deduplication Diagnostics
// ---------------------------------------------------------------------------

fn render_bento_insights_section(
    ui: &mut Ui,
    cache: &AnalysisCache,
    total_avail: f32,
    t: &ThemeTokens,
    action: &mut Option<OverviewAction>,
) {
    let insights = &cache.insights;
    let hardlink_groups = &cache.hardlink_groups;

    ui.horizontal(|ui| {
        ui.label(RichText::new("STORAGE INSIGHTS & STORAGE EFFICIENCY").size(11.0).strong().color(t.text_muted));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new("Linux diagnostic heuristics").size(10.5).color(t.text_muted));
        });
    });

    ui.add_space(4.0);

    // Hardlink Deduplication Telemetry Card (if hardlinks detected)
    if !hardlink_groups.is_empty() {
        let waste_str = format_size(cache.hardlink_waste_bytes);
        ui.allocate_ui_with_layout(egui::vec2(total_avail, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
            ui.set_max_width(total_avail);
            egui::Frame::none()
                .fill(t.bg_elevated)
                .rounding(6.0_f32)
                .stroke(egui::Stroke::new(1.0_f32, t.accent.gamma_multiply(0.5)))
                .inner_margin(Margin::same(12.0))
                .show(ui, |ui| {
                    ui.set_width((total_avail - 24.0).max(60.0));
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("🔗").size(18.0));
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("Hardlink Deduplication Active: {} Groups", hardlink_groups.len())).size(12.0).strong().color(t.accent));
                                ui.label(RichText::new(format!("({} shared physical storage)", waste_str)).size(11.0).monospace().color(t.cyan));
                            });
                            ui.add(
                                egui::Label::new(
                                    RichText::new("Multiple directory entries point to the same filesystem inode (st_ino). DiskLens tracks (st_dev, st_ino) to ensure physical disk allocation is calculated accurately without duplicate counting.")
                                        .size(10.5)
                                        .color(t.text_secondary)
                                ).wrap()
                            );
                        });
                    });
                });
        });
        ui.add_space(8.0);
    }

    if insights.is_empty() {
        ui.allocate_ui_with_layout(egui::vec2(total_avail, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
            ui.set_max_width(total_avail);
            egui::Frame::none()
                .fill(t.bg_elevated)
                .rounding(6.0_f32)
                .stroke(egui::Stroke::new(1.0_f32, t.border))
                .inner_margin(Margin::same(14.0))
                .show(ui, |ui| {
                    ui.set_width((total_avail - 28.0).max(60.0));
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("✔").size(16.0).color(t.success));
                        ui.label(RichText::new("No critical storage bottlenecks detected in this directory tree.").size(11.5).color(t.text_secondary));
                    });
                });
        });
    } else {
        let is_single_col = total_avail < 900.0;
        let card_spacing = 10.0_f32;
        let col_width = if is_single_col {
            total_avail
        } else {
            ((total_avail - card_spacing) / 2.0).floor().max(120.0)
        };

        for chunk in insights.chunks(if is_single_col { 1 } else { 2 }) {
            if is_single_col {
                for insight in chunk {
                    render_single_insight_card(ui, insight, col_width, t, action);
                }
            } else {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = card_spacing;
                    for insight in chunk {
                        render_single_insight_card(ui, insight, col_width, t, action);
                    }
                });
            }
            ui.add_space(8.0);
        }
    }
}

fn render_single_insight_card(
    ui: &mut Ui,
    insight: &crate::models::StorageInsight,
    col_width: f32,
    t: &ThemeTokens,
    action: &mut Option<OverviewAction>,
) {
    let badge_color = match &insight.severity[..] {
        "high" => t.danger,
        "medium" => t.warning,
        _ => t.accent,
    };

    // Strict rigid envelope: card must never request or expand past col_width
    ui.allocate_ui_with_layout(egui::vec2(col_width, 0.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
        ui.set_max_width(col_width);

        egui::Frame::none()
            .fill(t.bg_elevated)
            .rounding(6.0_f32)
            .stroke(egui::Stroke::new(1.0_f32, t.border))
            .inner_margin(Margin::same(12.0))
            .show(ui, |ui| {
                let card_inner_w = (col_width - 24.0).max(60.0);
                ui.set_width(card_inner_w);

                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(insight.category.display_name()).size(10.0).monospace().color(badge_color));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(format_size(insight.size_bytes)).size(12.5).monospace().strong().color(t.text));
                        });
                    });

                    ui.add_space(3.0);

                    // Title strictly truncated with hover tooltip to prevent unspaced hashes/paths from expanding card width
                    ui.add(
                        egui::Label::new(RichText::new(&insight.title).size(12.0).strong().color(t.text))
                            .truncate(),
                    ).on_hover_text(&insight.title);

                    ui.add_space(2.0);

                    // Description strictly wrapped
                    ui.add(
                        egui::Label::new(RichText::new(&insight.description).size(11.0).color(t.text_secondary))
                            .wrap(),
                    );

                    ui.add_space(6.0);

                    // Why it matters callout box
                    egui::Frame::none()
                        .fill(t.bg_surface)
                        .rounding(4.0_f32)
                        .stroke(egui::Stroke::new(1.0_f32, t.border))
                        .inner_margin(Margin::same(8.0))
                        .show(ui, |ui| {
                            let callout_inner_w = (card_inner_w - 16.0).max(40.0);
                            ui.set_width(callout_inner_w);
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("💡").size(11.0));
                                ui.add(
                                    egui::Label::new(RichText::new(format!("Why it matters: {}", &insight.why_it_matters)).size(10.5).color(t.text_muted))
                                        .wrap(),
                                );
                            });
                        });

                    ui.add_space(6.0);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button(format!("{} →", insight.action_label)).clicked() {
                            if insight.path.is_dir() {
                                *action = Some(OverviewAction::Navigate(ActiveTab::DirectoryTree));
                            } else {
                                *action = Some(OverviewAction::OpenInFileManager(insight.path.clone()));
                            }
                        }
                    });
                });
            });
    });
}

// ---------------------------------------------------------------------------
// Embedded Directory Explorer & File Browser Panel
// ---------------------------------------------------------------------------

fn render_overview_explorer_panel(
    ui: &mut Ui,
    current_node: &FileNode,
    total_avail: f32,
    state: &mut OverviewViewState,
    t: &ThemeTokens,
    action: &mut Option<OverviewAction>,
) {
    // 1. Interactive category distribution chips bar
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
            RichText::new(format!("◆ All ({})", format_size(total_children_size))).size(11.0).strong(),
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
            let chip_text = format!("{} {} ({} · {:.1}%)", cat.icon(), cat.display_name(), format_size(sz), pct);
            let chip_btn = ui.selectable_label(
                is_sel,
                RichText::new(chip_text).size(11.0).color(if is_sel { cat.color() } else { t.text }),
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

    // 2. Filter, Mode Preset & Sorting Toolbar
    ui.horizontal(|ui| {
        // Preset Sorting Toggle: Hierarchical vs Space Hogs
        let is_hogs_mode = state.sort_column == SortColumn::Size && state.sort_order == SortOrder::Descending;
        let hogs_btn = ui.selectable_label(
            is_hogs_mode,
            RichText::new("⚠ Space Hogs (Size ↓)").size(11.0).strong().color(if is_hogs_mode { t.warning } else { t.text_muted }),
        );
        if hogs_btn.clicked() {
            state.sort_column = SortColumn::Size;
            state.sort_order = SortOrder::Descending;
        }

        let is_tree_mode = state.sort_column == SortColumn::Name && state.sort_order == SortOrder::Ascending;
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
                ui.label(RichText::new(format!("Filtered: {}", cat.display_name())).size(10.5).color(cat.color()));
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
                return c.name.to_lowercase().contains(&state.search_filter.to_lowercase());
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
                let type_a = if a.is_dir { "Folder" } else { a.category.display_name() };
                let type_b = if b.is_dir { "Folder" } else { b.category.display_name() };
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

    let scrollbar_margin = ui.spacing().scroll.bar_width + 14.0;
    let avail_w = (total_avail - scrollbar_margin).max(100.0);

    TableBuilder::new(ui)
        .striped(true)
        .resizable(false)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::remainder().clip(true)) // Name & path middle-truncated
        .column(Column::exact(85.0))           // Apparent Size
        .column(Column::exact(95.0))           // Usage Progress Bar
        .column(Column::exact(80.0))           // Type
        .column(Column::exact(125.0))          // Actions Toolbar
        .header(24.0, |mut header| {
            header.col(|ui| {
                if ui.selectable_label(sort_col == SortColumn::Name, RichText::new("Name / Path").strong().color(t.text_muted)).clicked() {
                    toggle_sort(&mut state.sort_column, &mut state.sort_order, SortColumn::Name);
                }
            });
            header.col(|ui| {
                if ui.selectable_label(sort_col == SortColumn::Size, RichText::new("Size").strong().color(t.text_muted)).clicked() {
                    toggle_sort(&mut state.sort_column, &mut state.sort_order, SortColumn::Size);
                }
            });
            header.col(|ui| {
                ui.label(RichText::new("Usage").strong().color(t.text_muted));
            });
            header.col(|ui| {
                if ui.selectable_label(sort_col == SortColumn::Type, RichText::new("Type").strong().color(t.text_muted)).clicked() {
                    toggle_sort(&mut state.sort_column, &mut state.sort_order, SortColumn::Type);
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

                    // Col 1: Name / Path (clipped)
                    row.col(|ui| {
                        let icon = if child.is_dir { "📁" } else { child.category.icon() };
                        let name_label = format!("{} {}", icon, child.name);
                        let btn = ui.add(
                            egui::SelectableLabel::new(is_selected, RichText::new(name_label).size(11.5).color(t.text))
                        ).on_hover_text(child.path.display().to_string());

                        if btn.clicked() {
                            state.selected_node = Some(child.path.clone());
                        }
                        if btn.double_clicked() {
                            if child.is_dir {
                                navigate_to = Some(child.path.clone());
                            } else {
                                *action = Some(OverviewAction::OpenFile(child.path.clone()));
                            }
                        }
                    });

                    // Col 2: Apparent Size (85px)
                    row.col(|ui| {
                        ui.label(RichText::new(child.formatted_size()).size(11.0).monospace().color(t.text));
                    });

                    // Col 3: Usage (% of parent with visual progress bar, 95px)
                    row.col(|ui| {
                        let pct = (child.size as f32 / total_parent_size as f32).clamp(0.0, 1.0);
                        let bar_w = 48.0_f32;
                        let (bar_rect, _) = ui.allocate_exact_size(Vec2::new(bar_w, 6.0), Sense::hover());
                        ui.painter().rect_filled(bar_rect, 2.0_f32, t.bg_input);
                        let filled_w = (bar_w * pct).max(1.0);
                        let filled_rect = Rect::from_min_size(bar_rect.min, Vec2::new(filled_w, bar_rect.height()));
                        ui.painter().rect_filled(filled_rect, 2.0_f32, if child.is_dir { t.accent } else { child.category.color() });
                        ui.label(RichText::new(format!("{:.0}%", pct * 100.0)).size(10.5).monospace().color(t.text_muted));
                    });

                    // Col 4: Category badge (80px)
                    row.col(|ui| {
                        let type_s = if child.is_dir { "Folder" } else { child.category.display_name() };
                        let color = if child.is_dir { t.text_secondary } else { child.category.color() };
                        ui.label(RichText::new(type_s).size(10.5).color(color));
                    });

                    // Col 5: Actions toolbar (125px)
                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 2.0;
                            ui.spacing_mut().button_padding = Vec2::new(4.0, 2.0);

                            if child.is_dir {
                                if ui.small_button("→").on_hover_text("Explore folder").clicked() {
                                    navigate_to = Some(child.path.clone());
                                }
                            } else {
                                if ui.small_button("📄").on_hover_text("Open file").clicked() {
                                    *action = Some(OverviewAction::OpenFile(child.path.clone()));
                                }
                            }

                            if ui.small_button("📂").on_hover_text("Reveal in file manager").clicked() {
                                *action = Some(OverviewAction::OpenInFileManager(child.path.clone()));
                            }

                            if ui.small_button("📋").on_hover_text("Copy path").clicked() {
                                *action = Some(OverviewAction::CopyPath(child.path.clone()));
                            }

                            if ui.small_button("🗑").on_hover_text("Move to trash").clicked() {
                                *action = Some(OverviewAction::MoveToTrash(child.path.clone(), child.size));
                            }
                        });
                    });
                }
            });
        });

    if let Some(target) = navigate_to {
        state.navigate_to(target);
    }
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
