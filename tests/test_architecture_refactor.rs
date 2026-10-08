use disk_lens::actions::{FileSystem, MockFileSystem};
use disk_lens::app::{ActiveTab, AnalysisSession, AnalysisState, DiskLensApp};
use disk_lens::models::FileNode;
use disk_lens::scanner::{DefaultScanService, ScanHandle, ScanMessage, ScanService};
use disk_lens::settings::AppSettings;
use disk_lens::ui::overview_view::OverviewAction;
use eframe::egui;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn test_analysis_session_lifecycle() {
    let mut session = AnalysisSession::new("/home/user".to_string());
    assert!(session.is_idle());
    assert!(!session.is_ready());
    assert!(!session.is_scanning());
    assert_eq!(session.target_path(), None);

    // 1. Scanning state
    let target = PathBuf::from("/test/target");
    session.set_scanning(target.clone());
    assert!(session.is_scanning());
    assert!(!session.is_ready());
    assert_eq!(session.target_path(), Some(target.as_path()));
    assert_eq!(session.path_input, target.display().to_string());

    // 2. Ready state
    let mut root = FileNode::new_dir("target".to_string(), target.clone(), None);
    root.children.push(FileNode::new_file(
        "file1.txt".to_string(),
        target.join("file1.txt"),
        1024,
        None,
    ));
    root.recalculate_stats();
    session.set_ready(Box::new(root));
    assert!(session.is_ready());
    assert!(!session.is_scanning());
    assert!(session.scanned_root.is_some());
    assert!(session.analysis_cache.is_some());

    // 3. Node pruning through session
    let file_path = target.join("file1.txt");
    session.remove_node(&file_path);
    assert_eq!(session.scanned_root.as_ref().unwrap().size, 0);

    // 4. Failed state
    session.set_failed(target.clone(), "Permission Denied".to_string());
    assert!(!session.is_ready());
    assert_eq!(session.scan_error, Some("Permission Denied".to_string()));
    assert!(matches!(session.analysis_state, AnalysisState::Failed { .. }));

    // 5. Invalidation
    session.invalidate();
    assert!(session.is_idle());
    assert!(session.scanned_root.is_none());
    assert!(session.analysis_cache.is_none());
}

#[test]
fn test_file_system_mocking_and_inversion() {
    let mock_fs = MockFileSystem::new();
    let test_path = PathBuf::from("/tmp/test_file.txt");

    // Success actions
    assert!(mock_fs.open_in_file_manager(&test_path).is_ok());
    assert!(mock_fs.open_file(&test_path).is_ok());
    assert!(mock_fs.open_terminal(&test_path).is_ok());
    assert!(mock_fs.move_to_trash(&test_path).is_ok());
    assert!(mock_fs.permanent_delete(&test_path).is_ok());

    assert_eq!(mock_fs.opened_paths.lock().unwrap().len(), 3);
    assert_eq!(mock_fs.trashed_paths.lock().unwrap().len(), 1);
    assert_eq!(mock_fs.deleted_paths.lock().unwrap().len(), 1);

    // Simulated error injection
    let err_fs = MockFileSystem::with_error("Access Denied (Mocked)".to_string());
    assert_eq!(
        err_fs.open_file(&test_path).unwrap_err(),
        "Access Denied (Mocked)"
    );
    assert_eq!(
        err_fs.move_to_trash(&test_path).unwrap_err(),
        "Access Denied (Mocked)"
    );
}

#[test]
fn test_scan_service_abstraction_and_app_injection() {
    struct MockScanService;

    impl ScanService for MockScanService {
        fn start_scan(
            &self,
            root_path: PathBuf,
            _settings: AppSettings,
        ) -> (ScanHandle, crossbeam_channel::Receiver<ScanMessage>) {
            let (tx, rx) = crossbeam_channel::unbounded();
            let handle = ScanHandle::new();
            let fake_node = FileNode::new_dir("mock".to_string(), root_path, None);
            let _ = tx.send(ScanMessage::Complete(Box::new(fake_node)));
            (handle, rx)
        }
    }

    let mock_service = Arc::new(MockScanService);
    let mock_fs = Arc::new(MockFileSystem::new());
    let mut app = DiskLensApp::with_services(mock_service, mock_fs);

    let test_dir = std::env::temp_dir();
    app.start_scan(test_dir.clone());
    assert!(app.session.is_scanning);

    // Pump UI loop / messages
    let ctx = egui::Context::default();
    let mut raw_input = egui::RawInput::default();
    raw_input.screen_rect = Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(800.0, 600.0)));
    let _ = ctx.run(raw_input, |ctx| {
        app.render_app_ui(ctx);
    });

    // Scanner should have transitioned to ready via MockScanService
    assert!(app.session.is_ready());
    assert_eq!(app.selected_tab, ActiveTab::Overview);
}

#[test]
fn test_overview_action_purity_and_dispatch() {
    let mock_fs = Arc::new(MockFileSystem::new());
    let scan_service = Arc::new(DefaultScanService);
    let mut app = DiskLensApp::with_services(scan_service, mock_fs.clone());

    // 1. Test Navigation Action
    app.handle_overview_action(OverviewAction::Navigate(ActiveTab::Treemap));
    assert_eq!(app.selected_tab, ActiveTab::Storage); // not ready yet, so navigation guarded

    // 2. Test File System Action via Intent
    let test_file = PathBuf::from("/test/file.mp4");
    app.handle_overview_action(OverviewAction::OpenInFileManager(test_file.clone()));
    assert_eq!(mock_fs.opened_paths.lock().unwrap().as_slice(), &[test_file]);
}
