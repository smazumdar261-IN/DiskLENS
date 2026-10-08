use crate::actions::{FileSystem, RealFileSystem};
use crate::export::Exporter;
use crate::history::StorageHistoryManager;
use crate::models::{AnalysisCache, CategorySummary, ExtensionSummary, FileNode, ScanProgress};
use crate::scanner::{DefaultScanService, ScanHandle, ScanMessage, ScanService};
use crate::settings::AppSettings;
use crate::theme::AppTheme;
use crate::ui::categories_view::{render_categories_view, CategoriesViewState};
use crate::ui::command_palette::{render_command_palette, CommandAction, CommandPaletteState};
use crate::ui::docs_view::render_docs_view;
use crate::ui::duplicates_view::{render_duplicates_view, DuplicatesViewState};
use crate::ui::engine_view::{render_engine_view, EngineViewState};
use crate::ui::header::{render_header, HeaderProps};
use crate::ui::history_view::{render_history_view, HistoryViewState};
use crate::ui::largest_files_view::{render_largest_files_view, LargestFilesState};
use crate::ui::logs_view::{render_logs_view, LogsViewState};
use crate::ui::modals::ModalManager;
use crate::ui::mounts_view::{render_mounts_view, MountsViewState};
use crate::ui::overview_view::{render_overview_view, OverviewAction, OverviewViewState};
use crate::ui::safe_trash_view::{render_safe_trash_view, SafeTrashViewState};
use crate::ui::scanning_view::render_scanning_view;
use crate::ui::settings_modal::{render_settings_modal, SettingsModalState};
use crate::ui::side_nav::render_side_nav;
use crate::ui::smart_view::{render_smart_view, SmartViewState};
use crate::ui::status_bar::render_status_bar;
use crate::ui::storage_view::{render_storage_view, StorageViewState};
use crate::ui::tree_view::{render_tree_view, TreeViewState};
use crate::ui::treemap_view::{render_treemap_view, TreemapViewState};
use crossbeam_channel::Receiver;
use eframe::egui::{self, Margin, RichText, SidePanel, TopBottomPanel};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Storage,
    Overview,
    Treemap,
    DirectoryTree,
    LargestFiles,
    Categories,
    Duplicates,
    SafeTrash,
    History,
    MountPoints,
    DiagnosticsSmart,
    SystemLogs,
    RustEngine,
    Documentation,
}

impl ActiveTab {
    /// Returns true if this view strictly depends on completed filesystem analysis data.
    pub fn requires_analysis_data(&self) -> bool {
        matches!(
            self,
            ActiveTab::Overview
                | ActiveTab::Treemap
                | ActiveTab::DirectoryTree
                | ActiveTab::LargestFiles
                | ActiveTab::Categories
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnalysisState {
    /// No target has been analyzed yet (fresh start on Storage Home)
    Idle,
    /// Real filesystem analysis is currently running
    Scanning { target_path: PathBuf },
    /// Analysis finished successfully and real data is available
    Ready { target_path: PathBuf },
    /// Analysis failed (e.g. target path doesn't exist, permission denied, etc.)
    Failed { target_path: PathBuf, error: String },
}

impl AnalysisState {
    pub fn is_ready(&self) -> bool {
        matches!(self, AnalysisState::Ready { .. })
    }

    pub fn is_scanning(&self) -> bool {
        matches!(self, AnalysisState::Scanning { .. })
    }

    pub fn is_idle(&self) -> bool {
        matches!(self, AnalysisState::Idle)
    }

    pub fn target_path(&self) -> Option<&Path> {
        match self {
            AnalysisState::Scanning { target_path }
            | AnalysisState::Ready { target_path }
            | AnalysisState::Failed { target_path, .. } => Some(target_path.as_path()),
            AnalysisState::Idle => None,
        }
    }
}

/// Encapsulates the entire filesystem analysis session state and active domain model.
#[derive(Debug, Clone)]
pub struct AnalysisSession {
    pub analysis_state: AnalysisState,
    pub path_input: String,
    pub scanned_root: Option<Box<FileNode>>,
    pub analysis_cache: Option<AnalysisCache>,
    pub scan_error: Option<String>,
    pub progress: ScanProgress,
    pub is_scanning: bool,
}

impl Default for AnalysisSession {
    fn default() -> Self {
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
        Self {
            analysis_state: AnalysisState::Idle,
            path_input: home_dir,
            scanned_root: None,
            analysis_cache: None,
            scan_error: None,
            progress: ScanProgress::default(),
            is_scanning: false,
        }
    }
}

impl AnalysisSession {
    pub fn new(path_input: String) -> Self {
        Self {
            path_input,
            ..Default::default()
        }
    }

    pub fn is_ready(&self) -> bool {
        self.analysis_state.is_ready() && self.scanned_root.is_some()
    }

    pub fn is_scanning(&self) -> bool {
        self.analysis_state.is_scanning()
    }

    pub fn is_idle(&self) -> bool {
        self.analysis_state.is_idle()
    }

    pub fn target_path(&self) -> Option<&Path> {
        self.analysis_state.target_path()
    }

    pub fn invalidate(&mut self) {
        self.scanned_root = None;
        self.analysis_cache = None;
        self.scan_error = None;
        self.is_scanning = false;
        self.analysis_state = AnalysisState::Idle;
    }

    pub fn set_scanning(&mut self, path: PathBuf) {
        self.scanned_root = None;
        self.analysis_cache = None;
        self.scan_error = None;
        self.path_input = path.display().to_string();
        self.analysis_state = AnalysisState::Scanning { target_path: path };
        self.is_scanning = true;
        self.progress = ScanProgress::default();
    }

    pub fn set_ready(&mut self, root: Box<FileNode>) {
        self.is_scanning = false;
        self.scan_error = None;
        self.analysis_state = AnalysisState::Ready { target_path: root.path.clone() };
        self.analysis_cache = Some(AnalysisCache::build(&root));
        self.scanned_root = Some(root);
    }

    pub fn set_failed(&mut self, target_path: PathBuf, error: String) {
        self.is_scanning = false;
        self.scanned_root = None;
        self.analysis_cache = None;
        self.scan_error = Some(error.clone());
        self.analysis_state = AnalysisState::Failed { target_path, error };
    }

    pub fn set_cancelled(&mut self) {
        self.is_scanning = false;
        self.scanned_root = None;
        self.analysis_cache = None;
        self.analysis_state = AnalysisState::Idle;
    }

    pub fn remove_node(&mut self, target_path: &Path) {
        if let Some(root) = &mut self.scanned_root {
            root.prune_path(target_path);
            self.analysis_cache = Some(AnalysisCache::build(root));
        }
    }
}

pub struct DiskLensApp {
    pub selected_tab: ActiveTab,
    pub session: AnalysisSession,
    pub scan_service: Arc<dyn ScanService>,
    pub file_system: Arc<dyn FileSystem>,
    pub scan_handle: Option<ScanHandle>,
    pub scan_receiver: Option<Receiver<ScanMessage>>,
    pub global_search: String,
    pub storage_state: StorageViewState,
    pub overview_state: OverviewViewState,
    pub tree_state: TreeViewState,
    pub treemap_state: TreemapViewState,
    pub largest_state: LargestFilesState,
    pub categories_state: CategoriesViewState,
    pub duplicates_state: DuplicatesViewState,
    pub safe_trash_state: SafeTrashViewState,
    pub history_state: HistoryViewState,
    pub mounts_state: MountsViewState,
    pub smart_state: SmartViewState,
    pub logs_state: LogsViewState,
    pub engine_state: EngineViewState,
    pub settings_state: SettingsModalState,
    pub command_palette_state: CommandPaletteState,
    pub modals: ModalManager,
    pub dark_mode: bool,
}


impl Default for DiskLensApp {
    fn default() -> Self {
        let settings = AppSettings::load();
        let dark_mode = settings.theme_preference != crate::settings::ThemePreference::Light;

        Self {
            selected_tab: ActiveTab::Storage,
            session: AnalysisSession::default(),
            scan_service: Arc::new(DefaultScanService),
            file_system: Arc::new(RealFileSystem),
            scan_handle: None,
            scan_receiver: None,
            global_search: String::new(),
            storage_state: StorageViewState::new(),
            overview_state: OverviewViewState::default(),
            tree_state: TreeViewState::default(),
            treemap_state: TreemapViewState::default(),
            largest_state: LargestFilesState::default(),
            categories_state: CategoriesViewState::default(),
            duplicates_state: DuplicatesViewState::default(),
            safe_trash_state: SafeTrashViewState::default(),
            history_state: HistoryViewState::default(),
            mounts_state: MountsViewState::default(),
            smart_state: SmartViewState::default(),
            logs_state: LogsViewState::default(),
            engine_state: EngineViewState::default(),
            settings_state: SettingsModalState::default(),
            command_palette_state: CommandPaletteState::default(),
            modals: ModalManager::new(),
            dark_mode,
        }
    }
}

impl DiskLensApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        AppTheme::setup_fonts(&cc.egui_ctx);
        let app = Self::default();
        if app.dark_mode {
            AppTheme::apply_dark(&cc.egui_ctx);
        } else {
            AppTheme::apply_light(&cc.egui_ctx);
        }
        app
    }

    pub fn with_services(
        scan_service: Arc<dyn ScanService>,
        file_system: Arc<dyn FileSystem>,
    ) -> Self {
        Self {
            scan_service,
            file_system,
            ..Default::default()
        }
    }

    /// Single source of truth for whether a view is currently accessible.
    /// Views requiring completed filesystem analysis (Overview, Treemap, Folder Tree, Space Hogs, File Types)
    /// are strictly unavailable until valid analysis data is ready.
    pub fn is_view_available(&self, tab: ActiveTab) -> bool {
        if tab.requires_analysis_data() {
            self.session.is_ready()
        } else {
            true
        }
    }

    /// Attempts to navigate to the specified tab.
    /// Returns true if navigation was successful, false if the tab is currently unavailable.
    pub fn navigate_to(&mut self, tab: ActiveTab) -> bool {
        if self.is_view_available(tab) {
            self.selected_tab = tab;
            true
        } else {
            false
        }
    }

    pub fn handle_overview_action(&mut self, action: OverviewAction) {
        match action {
            OverviewAction::Navigate(tab) => {
                self.navigate_to(tab);
            }
            OverviewAction::OpenInFileManager(path) => {
                if let Err(e) = self.file_system.open_in_file_manager(&path) {
                    self.modals.show_toast(e.clone(), true);
                    self.modals.log("ERROR", &e);
                }
            }
            OverviewAction::OpenFile(path) => {
                if let Err(e) = self.file_system.open_file(&path) {
                    self.modals.show_toast(e.clone(), true);
                    self.modals.log("ERROR", &e);
                }
            }
            OverviewAction::OpenTerminal(path) => {
                if let Err(e) = self.file_system.open_terminal(&path) {
                    self.modals.show_toast(e.clone(), true);
                    self.modals.log("ERROR", &e);
                }
            }
            OverviewAction::MoveToTrash(path, size) => {
                self.modals.pending_confirm = Some(crate::ui::modals::ConfirmAction::MoveToTrash(path, size));
            }
            OverviewAction::PermanentDelete(path, size) => {
                self.modals.pending_confirm = Some(crate::ui::modals::ConfirmAction::PermanentDelete(path, size));
            }
            OverviewAction::CopyPath(path) => {
                let p_str = path.display().to_string();
                self.modals.show_toast(format!("Copied path: {}", p_str), false);
            }
            OverviewAction::ElevatePkexec(path) => {
                let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("/home/sayantan/disk-analyzer-gui/target/release/disk-lens"));
                let p_str = path.display().to_string();
                let spawn_res = std::process::Command::new("pkexec")
                    .arg(&exe)
                    .arg(&p_str)
                    .spawn();
                match spawn_res {
                    Ok(_) => {
                        self.modals.show_toast("Launched elevated DiskLens via pkexec".to_string(), false);
                        self.modals.log("INFO", "Spawned pkexec elevation process");
                    }
                    Err(e) => {
                        let cmd_str = format!("pkexec {} \"{}\"", exe.display(), p_str);
                        self.modals.show_toast(format!("Elevation error: {}. Run: {}", e, cmd_str), true);
                        self.modals.log("ERROR", &format!("pkexec launch failed: {}. Manual command: {}", e, cmd_str));
                    }
                }
            }
            OverviewAction::FilterCategory(cat) => {
                self.categories_state.selected_category = Some(cat);
                self.navigate_to(ActiveTab::Categories);
            }
            OverviewAction::Rescan(path) => {
                self.start_scan(path);
            }
        }
    }

    fn set_dark_mode(&mut self, ctx: &egui::Context, dark_mode: bool) {
        if self.dark_mode == dark_mode {
            return;
        }
        self.dark_mode = dark_mode;
        if dark_mode {
            AppTheme::apply_dark(ctx);
        } else {
            AppTheme::apply_light(ctx);
        }
        ctx.request_repaint();
    }

    pub fn start_scan(&mut self, path: PathBuf) {
        // 1. Target Validation
        if !path.exists() {
            let err_msg = format!("Target path does not exist: {}", path.display());
            self.session.set_failed(path.clone(), err_msg.clone());
            self.modals.show_toast(format!("Path not found: {}", path.display()), true);
            self.modals.log("ERROR", &format!("Target validation failed: '{}' does not exist", path.display()));
            return;
        }

        // 2. Invalidate previous analysis state and initiate session
        self.session.set_scanning(path.clone());
        self.duplicates_state.has_analyzed = false;
        self.tree_state.selected_node = None;
        self.treemap_state.current_path = path.clone();
        self.tree_state.current_path = path.clone();

        // 3. Start scan via decoupled ScanService
        let settings = AppSettings::load();
        let (handle, rx) = self.scan_service.start_scan(path.clone(), settings);
        self.scan_handle = Some(handle);
        self.scan_receiver = Some(rx);
        self.modals.log("INFO", &format!("Initiated scan on {}", path.display()));
    }

    fn cancel_scan(&mut self) {
        if let Some(handle) = &self.scan_handle {
            handle.cancel();
            self.modals
                .show_toast("Cancelling analysis...".to_string(), true);
            self.modals.log("WARN", "Scan cancelled by user");
        }
    }

    fn toggle_pause_scan(&mut self) {
        if let Some(handle) = &self.scan_handle {
            if handle.is_paused() {
                handle.resume();
                self.session.progress.is_paused = false;
                self.modals
                    .show_toast("Analysis resumed".to_string(), false);
                self.modals.log("INFO", "Scan resumed");
            } else {
                handle.pause();
                self.session.progress.is_paused = true;
                self.modals.show_toast("Analysis paused".to_string(), false);
                self.modals.log("INFO", "Scan paused");
            }
        }
    }

    fn check_scanner_messages(&mut self) {
        let mut messages = Vec::new();
        if let Some(rx) = &self.scan_receiver {
            while let Ok(msg) = rx.try_recv() {
                messages.push(msg);
            }
        }

        for msg in messages {
            match msg {
                ScanMessage::Progress(p) => {
                    self.session.progress = p;
                }
                ScanMessage::Complete(root) => {
                    self.scan_handle = None;
                    self.scan_receiver = None;
                    let elapsed = self.session.progress.elapsed_secs;
                    let _ = StorageHistoryManager::record_scan(&root, elapsed);

                    self.modals.show_toast(
                        format!(
                            "Analysis complete: {} ({:.2}s)",
                            root.formatted_size(),
                            elapsed
                        ),
                        false,
                    );
                    self.modals.log("INFO", &format!("Scan complete: {} files, {} dirs in {:.2}s", root.file_count, root.dir_count, elapsed));
                    self.tree_state.current_path = root.path.clone();
                    self.treemap_state.current_path = root.path.clone();
                    self.session.set_ready(root);
                    self.selected_tab = ActiveTab::Overview;
                }
                ScanMessage::Cancelled => {
                    self.scan_handle = None;
                    self.scan_receiver = None;
                    self.session.set_cancelled();
                    self.selected_tab = ActiveTab::Storage;
                    self.modals
                        .show_toast("Analysis was cancelled".to_string(), true);
                }
                ScanMessage::Error(err) => {
                    self.scan_handle = None;
                    self.scan_receiver = None;
                    let target = PathBuf::from(&self.session.path_input);
                    self.session.set_failed(target, err.clone());
                    self.selected_tab = ActiveTab::Storage;
                    self.modals.show_toast(format!("Error: {}", err), true);
                    self.modals.log("ERROR", &err);
                }
            }
        }
    }

    fn remove_node_from_tree(&mut self, target_path: &Path) {
        self.session.remove_node(target_path);
        self.duplicates_state.has_analyzed = false;
        self.modals.log("INFO", &format!("Removed node {} from analysis cache", target_path.display()));
    }

    fn export_report(&mut self, is_json: bool) {
        if let Some(root) = &self.session.scanned_root {
            let filter = if is_json { "json" } else { "csv" };
            let filename = if is_json {
                "disklens-scan-report.json"
            } else {
                "disklens-space-hogs.csv"
            };

            let dialog = rfd::FileDialog::new()
                .set_file_name(filename)
                .add_filter(filter, &[filter]);

            if let Some(save_path) = dialog.save_file() {
                let mut cat_stats = HashMap::new();
                root.collect_category_stats(&mut cat_stats);
                let total_size = root.size.max(1);

                let cat_summaries: Vec<CategorySummary> = cat_stats
                    .into_iter()
                    .map(|(cat, (count, app_size, alloc_size))| CategorySummary {
                        category: cat,
                        count,
                        total_size: app_size,
                        total_allocated_size: alloc_size,
                        percentage: (app_size as f32 / total_size as f32) * 100.0,
                    })
                    .collect();

                let mut ext_stats = HashMap::new();
                root.collect_extension_stats(&mut ext_stats);
                let ext_summaries: Vec<ExtensionSummary> = ext_stats
                    .into_iter()
                    .map(|(ext, (count, app_size, alloc_size, cat))| ExtensionSummary {
                        extension: ext,
                        count,
                        total_size: app_size,
                        total_allocated_size: alloc_size,
                        category: cat,
                        percentage: (app_size as f32 / total_size as f32) * 100.0,
                    })
                    .collect();

                let mut largest = Vec::new();
                root.collect_largest_files(&mut largest);
                largest.sort_by(|a, b| b.size.cmp(&a.size));

                let result = if is_json {
                    Exporter::export_json(
                        &save_path,
                        root,
                        &cat_summaries,
                        &ext_summaries,
                        &largest,
                    )
                } else {
                    Exporter::export_csv(&save_path, &largest)
                };

                match result {
                    Ok(_) => self
                        .modals
                        .show_toast(format!("Exported report to {}", save_path.display()), false),
                    Err(e) => self.modals.show_toast(e, true),
                }
            }
        } else {
            self.modals.show_toast(
                "No analysis data to export. Please analyze a location first.".to_string(),
                true,
            );
        }
    }
}

fn prune_node_recursively(node: &mut FileNode, target: &Path) -> bool {
    if !node.is_dir {
        return false;
    }

    let mut removed = false;
    node.children.retain(|c| {
        if c.path == target {
            removed = true;
            false
        } else {
            true
        }
    });

    if removed {
        node.size = node.children.iter().map(|c| c.size).sum();
        node.file_count = node
            .children
            .iter()
            .map(|c| if c.is_dir { c.file_count } else { 1 })
            .sum();
        node.dir_count = node
            .children
            .iter()
            .map(|c| if c.is_dir { c.dir_count + 1 } else { 0 })
            .sum();
        return true;
    }

    for child in &mut node.children {
        if child.is_dir && target.starts_with(&child.path) {
            if prune_node_recursively(child, target) {
                node.size = node.children.iter().map(|c| c.size).sum();
                node.file_count = node
                    .children
                    .iter()
                    .map(|c| if c.is_dir { c.file_count } else { 1 })
                    .sum();
                node.dir_count = node
                    .children
                    .iter()
                    .map(|c| if c.is_dir { c.dir_count + 1 } else { 0 })
                    .sum();
                return true;
            }
        }
    }

    false
}

impl DiskLensApp {
    pub fn render_app_ui(&mut self, ctx: &egui::Context) {
        if self.session.is_scanning {
            ctx.request_repaint();
        }

        self.check_scanner_messages();

        // Global Keyboard Shortcuts (Ctrl+K, 1-8, C, M, H, L, D, R, T, S / Comma, Esc)
        let is_any_text_focused = ctx.memory(|m| m.focused().is_some());
        if !is_any_text_focused {
            if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::K)) {
                self.command_palette_state.is_open = !self.command_palette_state.is_open;
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Num1)) {
                self.navigate_to(ActiveTab::Overview);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Num2)) {
                self.navigate_to(ActiveTab::Treemap);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Num3)) {
                self.navigate_to(ActiveTab::DirectoryTree);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Num4)) {
                self.navigate_to(ActiveTab::LargestFiles);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Num5)) {
                self.navigate_to(ActiveTab::Categories);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Num6)) {
                self.navigate_to(ActiveTab::Duplicates);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Num7)) {
                self.navigate_to(ActiveTab::History);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Num8)) {
                self.navigate_to(ActiveTab::RustEngine);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::C)) {
                self.navigate_to(ActiveTab::SafeTrash);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::M)) {
                self.navigate_to(ActiveTab::MountPoints);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::H)) {
                self.navigate_to(ActiveTab::DiagnosticsSmart);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::L)) {
                self.navigate_to(ActiveTab::SystemLogs);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::D)) {
                self.navigate_to(ActiveTab::Documentation);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::R)) {
                if let Some(root) = &self.session.scanned_root {
                    self.start_scan(root.path.clone());
                }
            }
            if ctx.input(|i| i.key_pressed(egui::Key::T)) {
                self.set_dark_mode(ctx, !self.dark_mode);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Comma) || i.key_pressed(egui::Key::S)) {
                self.settings_state.is_open = !self.settings_state.is_open;
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.global_search.clear();
                self.settings_state.is_open = false;
                self.command_palette_state.is_open = false;
                self.modals.pending_confirm = None;
                self.modals.is_dir_picker_open = false;
                self.modals.is_mount_points_open = false;
                self.modals.is_system_logs_open = false;
                self.modals.is_docs_open = false;
            }
        }

        let mut rescan_target = None;
        let mut refresh_storage = false;
        let mut export_action = None;
        let mut requested_theme = None;
        let mut open_mounts = false;
        let mut open_logs = false;
        let mut open_docs = false;
        let mut open_dir_picker = false;
        let mut open_settings = false;
        let mut open_cmd_palette = false;
        let mut toggle_one_fs = false;
        let mut do_elevate = false;

        let perm_errors = self.session.progress.issues.iter().filter(|i| i.issue_type == crate::models::ScanIssueType::PermissionDenied).count();
        let one_fs = !self.settings_state.settings.cross_filesystem_boundaries;

        let t = AppTheme::tokens(self.dark_mode);

        // 1. Top CSD & Toolbar Header
        TopBottomPanel::top("top_header_panel")
            .frame(
                egui::Frame::none()
                    .fill(t.bg_surface)
                    .inner_margin(Margin::symmetric(14.0, 6.0)),
            )
            .show(ctx, |ui| {
                render_header(
                    ui,
                    HeaderProps {
                        active_tab: &mut self.selected_tab,
                        is_scanning: self.session.is_scanning,
                        scanned_root: &self.session.scanned_root,
                        dark_mode: self.dark_mode,
                        search_query: &mut self.global_search,
                        one_filesystem: one_fs,
                        permission_errors: perm_errors,
                        on_theme_change: &mut |dm| requested_theme = Some(dm),
                        on_refresh_storage: &mut || refresh_storage = true,
                        on_rescan: &mut |target| rescan_target = Some(target),
                        on_toggle_one_fs: &mut || toggle_one_fs = true,
                        on_elevate_pkexec: &mut || do_elevate = true,
                        on_export: &mut |is_json| export_action = Some(is_json),
                        on_open_mounts: &mut || open_mounts = true,
                        on_open_logs: &mut || open_logs = true,
                        on_open_docs: &mut || open_docs = true,
                        on_open_dir_picker: &mut || open_dir_picker = true,
                        on_open_settings: &mut || open_settings = true,
                        on_open_command_palette: &mut || open_cmd_palette = true,
                    },
                );
            });

        // 2. Persistent Bottom Status Bar
        TopBottomPanel::bottom("bottom_status_panel")
            .frame(
                egui::Frame::none()
                    .fill(t.bg_surface)
                    .stroke(egui::Stroke::new(1.0_f32, t.border))
                    .inner_margin(Margin::symmetric(14.0, 4.0)),
            )
            .show(ctx, |ui| {
                render_status_bar(
                    ui,
                    &self.session.scanned_root,
                    self.session.is_scanning,
                    &mut self.selected_tab,
                    self.dark_mode,
                    &mut || open_logs = true,
                    &mut || open_mounts = true,
                    &mut || open_docs = true,
                    &mut || open_cmd_palette = true,
                );
            });

        // 3. Left Navigation Sidebar
        let show_sidebar = true;
        if show_sidebar {
            let screen_w = ctx.screen_rect().width();
            let is_compact = screen_w < 860.0;
            let sidebar_w = if is_compact { 48.0 } else { 190.0 };

            SidePanel::left("left_nav_sidebar")
                .exact_width(sidebar_w)
                .resizable(false)
                .frame(
                    egui::Frame::none()
                        .fill(t.bg_surface)
                        .stroke(egui::Stroke::new(1.0_f32, t.border))
                        .inner_margin(Margin::symmetric(if is_compact { 4.0 } else { 8.0 }, 10.0)),
                )
                .show(ctx, |ui| {
                    let hogs_count = self.session.analysis_cache.as_ref().map(|c| c.largest_files.len()).unwrap_or(0);
                    let is_analysis_ready = self.session.is_ready();
                    let can_nav = move |tab: ActiveTab| if tab.requires_analysis_data() { is_analysis_ready } else { true };
                    render_side_nav(
                        ui,
                        &mut self.selected_tab,
                        &self.session.scanned_root,
                        hogs_count,
                        self.dark_mode,
                        is_compact,
                        &can_nav,
                    );
                });
        }

        if let Some(dark_mode) = requested_theme {
            self.set_dark_mode(ctx, dark_mode);
        }

        if refresh_storage {
            self.storage_state.refresh();
            self.modals.show_toast("Storage devices refreshed".to_string(), false);
        }

        if toggle_one_fs {
            self.settings_state.settings.cross_filesystem_boundaries = !self.settings_state.settings.cross_filesystem_boundaries;
            let _ = self.settings_state.settings.save();
            let state_str = if !self.settings_state.settings.cross_filesystem_boundaries { "Active (-xdev ON)" } else { "Disabled (-xdev OFF)" };
            self.modals.show_toast(format!("One Filesystem Boundary: {}", state_str), false);
            self.modals.log("INFO", &format!("Toggled mount boundary control: {}", state_str));
        }

        if do_elevate {
            let target_p = self.session.path_input.clone();
            let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("/home/sayantan/disk-analyzer-gui/target/release/disk-lens"));
            let spawn_res = std::process::Command::new("pkexec")
                .arg(&exe)
                .arg(&target_p)
                .spawn();
            match spawn_res {
                Ok(_) => {
                    self.modals.show_toast("Launched elevated DiskLens via pkexec".to_string(), false);
                    self.modals.log("INFO", "Spawned pkexec elevation process");
                }
                Err(e) => {
                    let cmd_str = format!("pkexec {} \"{}\"", exe.display(), target_p);
                    self.modals.show_toast(format!("Elevation error: {}. Run: {}", e, cmd_str), true);
                    self.modals.log("ERROR", &format!("pkexec launch failed: {}. Manual command: {}", e, cmd_str));
                }
            }
        }

        if open_mounts {
            self.navigate_to(ActiveTab::MountPoints);
        }

        if open_logs {
            self.navigate_to(ActiveTab::SystemLogs);
        }

        if open_docs {
            self.navigate_to(ActiveTab::Documentation);
        }

        if open_dir_picker {
            self.modals.is_dir_picker_open = true;
        }

        if open_settings {
            self.settings_state.is_open = true;
        }

        if open_cmd_palette {
            self.command_palette_state.is_open = true;
        }

        if let Some(is_json) = export_action {
            self.export_report(is_json);
        }

        if let Some(target) = rescan_target {
            self.start_scan(target);
        }

        // 4. Central Panel Surface
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(t.bg_base)
                    .inner_margin(Margin::symmetric(14.0, 10.0)),
            )
            .show(ctx, |ui| {
                // Scanning Dashboard View
                if self.session.is_scanning {
                    let mut do_pause = false;
                    let mut do_cancel = false;

                    render_scanning_view(
                        ui,
                        &self.session.path_input,
                        &self.session.progress,
                        self.dark_mode,
                        &mut || do_pause = true,
                        &mut || do_cancel = true,
                    );

                    if do_pause {
                        self.toggle_pause_scan();
                    }
                    if do_cancel {
                        self.cancel_scan();
                    }
                    return;
                }

                // Error State
                if let Some(err) = self.session.scan_error.clone() {
                    let mut dismiss = false;
                    ui.vertical_centered(|ui| {
                        ui.add_space(80.0);

                        egui::Frame::none()
                            .fill(t.bg_elevated)
                            .rounding(8.0_f32)
                            .stroke(egui::Stroke::new(1.0_f32, t.border))
                            .inner_margin(Margin::same(24.0))
                            .show(ui, |ui| {
                                ui.set_max_width(480.0);
                                ui.vertical_centered(|ui| {
                                    ui.label(RichText::new("⚠").size(36.0).color(t.danger));
                                    ui.add_space(6.0);
                                    ui.label(RichText::new("Unable to analyze location").size(16.0).strong().color(t.text));
                                    ui.add_space(4.0);
                                    ui.label(RichText::new(&err).size(12.0).color(t.text_muted));
                                    ui.add_space(16.0);

                                    if ui.button("← Return to Storage Home").clicked() {
                                        dismiss = true;
                                    }
                                });
                            });
                    });

                    if dismiss {
                        self.session.scan_error = None;
                        self.selected_tab = ActiveTab::Storage;
                    }
                    return;
                }

                // Guardrail: if selected_tab is somehow an unavailable tab, fall back to Storage Home
                if !self.is_view_available(self.selected_tab) {
                    self.selected_tab = ActiveTab::Storage;
                }

                let mut action_analyze = None;

                match self.selected_tab {
                    ActiveTab::Storage => {
                        render_storage_view(
                            ui,
                            &mut self.storage_state,
                            &mut self.session.path_input,
                            &mut self.selected_tab,
                            self.dark_mode,
                            &mut |path| action_analyze = Some(path),
                        );
                    }
                    ActiveTab::Overview => {
                        if let (Some(root), Some(cache)) = (&self.session.scanned_root, &self.session.analysis_cache) {
                            if let Some(action) = render_overview_view(
                                ui,
                                root,
                                cache,
                                &mut self.overview_state,
                                self.dark_mode,
                            ) {
                                self.handle_overview_action(action);
                            }
                        } else {
                            self.selected_tab = ActiveTab::Storage;
                        }
                    }
                    ActiveTab::Treemap => {
                        if let Some(root) = &self.session.scanned_root {
                            render_treemap_view(
                                ui,
                                root,
                                &mut self.treemap_state,
                                &mut self.selected_tab,
                                &mut self.modals,
                                self.dark_mode,
                                &mut |path| action_analyze = Some(path),
                            );
                        } else {
                            self.selected_tab = ActiveTab::Storage;
                        }
                    }
                    ActiveTab::DirectoryTree => {
                        if let Some(root) = &self.session.scanned_root {
                            render_tree_view(
                                ui,
                                root,
                                &mut self.tree_state,
                                &mut self.modals,
                                self.dark_mode,
                                &mut |path| action_analyze = Some(path),
                            );
                        } else {
                            self.selected_tab = ActiveTab::Storage;
                        }
                    }
                    ActiveTab::LargestFiles => {
                        if let Some(cache) = &self.session.analysis_cache {
                            render_largest_files_view(
                                ui,
                                &cache.largest_files,
                                &mut self.largest_state,
                                &mut self.modals,
                                self.dark_mode,
                            );
                        } else {
                            self.selected_tab = ActiveTab::Storage;
                        }
                    }
                    ActiveTab::Categories => {
                        if let Some(cache) = &self.session.analysis_cache {
                            render_categories_view(
                                ui,
                                cache,
                                &mut self.categories_state,
                                self.dark_mode,
                            );
                        } else {
                            self.selected_tab = ActiveTab::Storage;
                        }
                    }
                    ActiveTab::Duplicates => {
                        render_duplicates_view(
                            ui,
                            self.session.scanned_root.as_deref(),
                            &mut self.duplicates_state,
                            &mut self.modals,
                            self.dark_mode,
                        );
                    }
                    ActiveTab::SafeTrash => {
                        render_safe_trash_view(
                            ui,
                            &mut self.safe_trash_state,
                            &mut self.modals,
                            self.dark_mode,
                        );
                    }
                    ActiveTab::History => {
                        render_history_view(
                            ui,
                            self.session.scanned_root.as_deref(),
                            &mut self.history_state,
                            self.dark_mode,
                        );
                    }
                    ActiveTab::MountPoints => {
                        render_mounts_view(
                            ui,
                            &mut self.mounts_state,
                            &mut self.modals,
                            self.dark_mode,
                            &mut |path| action_analyze = Some(path),
                        );
                    }
                    ActiveTab::DiagnosticsSmart => {
                        render_smart_view(
                            ui,
                            &mut self.smart_state,
                            &mut self.modals,
                            self.dark_mode,
                        );
                    }
                    ActiveTab::SystemLogs => {
                        render_logs_view(
                            ui,
                            &mut self.logs_state,
                            &self.modals,
                            self.dark_mode,
                        );
                    }
                    ActiveTab::RustEngine => {
                        render_engine_view(
                            ui,
                            &mut self.engine_state,
                            self.dark_mode,
                        );
                    }
                    ActiveTab::Documentation => {
                        render_docs_view(
                            ui,
                            self.dark_mode,
                        );
                    }
                }

                if let Some(target) = action_analyze {
                    self.start_scan(target);
                }
            });

        // 5. Global Command Palette (Ctrl+K)
        let mut palette_action = None;
        let is_analysis_ready = self.session.is_ready();
        let can_nav_palette = move |tab: ActiveTab| if tab.requires_analysis_data() { is_analysis_ready } else { true };
        render_command_palette(
            ctx,
            &mut self.command_palette_state,
            self.dark_mode,
            &can_nav_palette,
            &mut |act| palette_action = Some(act),
        );

        if let Some(act) = palette_action {
            match act {
                CommandAction::SwitchTab(tab) => {
                    self.navigate_to(tab);
                }
                CommandAction::StartScan(path) => self.start_scan(path),
                CommandAction::OpenDirPicker => self.modals.is_dir_picker_open = true,
                CommandAction::OpenSettings => self.settings_state.is_open = true,
                CommandAction::OpenDocs => {
                    self.navigate_to(ActiveTab::Documentation);
                }
                CommandAction::OpenMounts => {
                    self.navigate_to(ActiveTab::MountPoints);
                }
                CommandAction::OpenLogs => {
                    self.navigate_to(ActiveTab::SystemLogs);
                }
                CommandAction::TriggerDryRunClean => {
                    self.navigate_to(ActiveTab::SafeTrash);
                    self.safe_trash_state.is_dry_run = true;
                    self.safe_trash_state.cleanup_completed = true;
                    self.safe_trash_state.reclaimed_bytes = self.safe_trash_state.categories.iter().map(|c| c.total_reclaimable_bytes()).sum();
                    self.modals.show_toast(format!("Dry-run complete: {} verified", crate::models::format_size(self.safe_trash_state.reclaimed_bytes)), false);
                }
                CommandAction::ToggleTheme => self.set_dark_mode(ctx, !self.dark_mode),
            }
        }

        // 6. Dialogs, Modals, Settings & Floating Toasts
        let mut deleted_path = None;
        let mut custom_scan_path = None;

        self.modals.show(
            ctx,
            self.dark_mode,
            &mut |p| deleted_path = Some(p.clone()),
            &mut |cp| custom_scan_path = Some(cp),
        );

        render_settings_modal(
            ctx,
            &mut self.settings_state,
            &mut self.modals,
            &mut self.dark_mode,
        );

        if let Some(path) = deleted_path {
            self.remove_node_from_tree(&path);
        }

        if let Some(cp) = custom_scan_path {
            self.start_scan(cp);
        }
    }
}

impl eframe::App for DiskLensApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render_app_ui(ctx);
    }
}
