use disk_lens::app::{ActiveTab, AnalysisState, DiskLensApp};
use disk_lens::models::FileNode;
use eframe::egui::{self, Key, Pos2, Rect, Vec2};
use std::fs;
use std::path::PathBuf;

#[test]
fn test_01_fresh_application_launch_navigation_state() {
    let mut app = DiskLensApp::default();

    // On launch: Storage Home is active and analysis state is Idle
    assert_eq!(app.selected_tab, ActiveTab::Storage);
    assert!(app.session.analysis_state.is_idle());
    assert!(app.session.scanned_root.is_none());
    assert!(app.session.analysis_cache.is_none());

    // Strict Rule: Analysis-dependent views MUST be disabled in State 1
    assert!(
        !app.is_view_available(ActiveTab::Overview),
        "Overview must be disabled"
    );
    assert!(
        !app.is_view_available(ActiveTab::Treemap),
        "Treemap must be disabled"
    );
    assert!(
        !app.is_view_available(ActiveTab::DirectoryTree),
        "Folder Tree must be disabled"
    );
    assert!(
        !app.is_view_available(ActiveTab::LargestFiles),
        "Space Hogs must be disabled"
    );
    assert!(
        !app.is_view_available(ActiveTab::Categories),
        "File Types must be disabled"
    );

    // System management and diagnostic views MUST remain available
    assert!(app.is_view_available(ActiveTab::Storage));
    assert!(app.is_view_available(ActiveTab::SafeTrash));
    assert!(app.is_view_available(ActiveTab::History));
    assert!(app.is_view_available(ActiveTab::MountPoints));
    assert!(app.is_view_available(ActiveTab::DiagnosticsSmart));
    assert!(app.is_view_available(ActiveTab::SystemLogs));
    assert!(app.is_view_available(ActiveTab::RustEngine));
    assert!(app.is_view_available(ActiveTab::Documentation));

    // Calling navigate_to on disabled views must fail and NOT alter selected_tab
    assert!(!app.navigate_to(ActiveTab::Overview));
    assert_eq!(app.selected_tab, ActiveTab::Storage);

    assert!(!app.navigate_to(ActiveTab::Treemap));
    assert_eq!(app.selected_tab, ActiveTab::Storage);

    assert!(!app.navigate_to(ActiveTab::DirectoryTree));
    assert_eq!(app.selected_tab, ActiveTab::Storage);

    assert!(!app.navigate_to(ActiveTab::LargestFiles));
    assert_eq!(app.selected_tab, ActiveTab::Storage);

    assert!(!app.navigate_to(ActiveTab::Categories));
    assert_eq!(app.selected_tab, ActiveTab::Storage);
}

#[test]
fn test_02_valid_target_selected_and_scanning_state() {
    let temp_dir = std::env::temp_dir().join(format!(
        "disklens_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&temp_dir).expect("create tempdir");
    let test_file = temp_dir.join("sample.txt");
    fs::write(&test_file, b"test content").expect("write test file");

    let mut app = DiskLensApp::default();

    // Initiate scan on valid target
    app.start_scan(temp_dir.clone());

    assert!(app.session.is_scanning);
    assert!(app.session.analysis_state.is_scanning());
    assert_eq!(
        app.session.analysis_state.target_path(),
        Some(temp_dir.as_path())
    );

    // While scanning, analysis data is not yet ready; dependent views MUST remain disabled
    assert!(!app.is_view_available(ActiveTab::Overview));
    assert!(!app.is_view_available(ActiveTab::Treemap));
    assert!(!app.is_view_available(ActiveTab::DirectoryTree));
    assert!(!app.is_view_available(ActiveTab::LargestFiles));
    assert!(!app.is_view_available(ActiveTab::Categories));

    // Attempting navigation during scanning is rejected
    assert!(!app.navigate_to(ActiveTab::Overview));
    assert!(!app.navigate_to(ActiveTab::Treemap));
}

#[test]
fn test_03_analysis_completed_successfully() {
    let mut app = DiskLensApp::default();
    let root_path = PathBuf::from("/test/dataset");
    let mut fake_root = FileNode::new_dir("dataset".to_string(), root_path.clone(), None);
    fake_root.children.push(FileNode::new_file(
        "a.dat".to_string(),
        root_path.join("a.dat"),
        1024 * 1024,
        None,
    ));
    fake_root.size = 1024 * 1024;
    fake_root.file_count = 1;

    // Simulate completion
    let cache = disk_lens::models::AnalysisCache::build(&fake_root);
    app.session.analysis_state = AnalysisState::Ready {
        target_path: root_path.clone(),
    };
    app.session.scanned_root = Some(Box::new(fake_root));
    app.session.analysis_cache = Some(cache);

    // In State 2 (Analysis Ready with data), all analysis-dependent views are enabled
    assert!(app.is_view_available(ActiveTab::Overview));
    assert!(app.is_view_available(ActiveTab::Treemap));
    assert!(app.is_view_available(ActiveTab::DirectoryTree));
    assert!(app.is_view_available(ActiveTab::LargestFiles));
    assert!(app.is_view_available(ActiveTab::Categories));

    // Navigation now succeeds
    assert!(app.navigate_to(ActiveTab::Treemap));
    assert_eq!(app.selected_tab, ActiveTab::Treemap);

    assert!(app.navigate_to(ActiveTab::DirectoryTree));
    assert_eq!(app.selected_tab, ActiveTab::DirectoryTree);

    assert!(app.navigate_to(ActiveTab::LargestFiles));
    assert_eq!(app.selected_tab, ActiveTab::LargestFiles);

    assert!(app.navigate_to(ActiveTab::Categories));
    assert_eq!(app.selected_tab, ActiveTab::Categories);

    assert!(app.navigate_to(ActiveTab::Overview));
    assert_eq!(app.selected_tab, ActiveTab::Overview);
}

#[test]
fn test_04_scan_failure_target_validation() {
    let mut app = DiskLensApp::default();
    let nonexistent = PathBuf::from("/tmp/disklens_nonexistent_test_directory_987654");

    // Target validation must detect nonexistent path and transition to Failed
    app.start_scan(nonexistent.clone());

    assert!(!app.session.is_scanning);
    assert!(matches!(
        app.session.analysis_state,
        AnalysisState::Failed { .. }
    ));
    assert!(app.session.scan_error.is_some());
    assert!(app.session.scanned_root.is_none());
    assert!(app.session.analysis_cache.is_none());

    // Analysis-dependent views must remain strictly unavailable
    assert!(!app.is_view_available(ActiveTab::Overview));
    assert!(!app.is_view_available(ActiveTab::Treemap));
    assert!(!app.is_view_available(ActiveTab::DirectoryTree));
    assert!(!app.is_view_available(ActiveTab::LargestFiles));
    assert!(!app.is_view_available(ActiveTab::Categories));

    assert!(!app.navigate_to(ActiveTab::Overview));
    assert_eq!(app.selected_tab, ActiveTab::Storage);
}

#[test]
fn test_05_target_changed_invalidates_previous_state() {
    let mut app = DiskLensApp::default();

    // 1. Establish initial ready state on Target A
    let root_path_a = PathBuf::from("/test/target_a");
    let fake_root_a = FileNode::new_dir("target_a".to_string(), root_path_a.clone(), None);
    let cache_a = disk_lens::models::AnalysisCache::build(&fake_root_a);

    app.session.analysis_state = AnalysisState::Ready {
        target_path: root_path_a,
    };
    app.session.scanned_root = Some(Box::new(fake_root_a));
    app.session.analysis_cache = Some(cache_a);
    app.selected_tab = ActiveTab::Treemap;

    assert!(app.is_view_available(ActiveTab::Treemap));

    // 2. User selects Target B
    let temp_b = std::env::temp_dir().join(format!(
        "disklens_test_b_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&temp_b).expect("create tempdir b");
    app.start_scan(temp_b.clone());

    // Previous analysis data MUST be invalidated immediately
    assert!(
        app.session.scanned_root.is_none(),
        "Previous scanned_root must be invalidated"
    );
    assert!(
        app.session.analysis_cache.is_none(),
        "Previous analysis_cache must be invalidated"
    );
    assert!(app.session.analysis_state.is_scanning());

    // Treemap and other views MUST be disabled again
    assert!(!app.is_view_available(ActiveTab::Treemap));
    assert!(!app.is_view_available(ActiveTab::Overview));
    assert!(!app.is_view_available(ActiveTab::DirectoryTree));

    // Navigating to them is rejected
    assert!(!app.navigate_to(ActiveTab::Treemap));
}

#[test]
fn test_06_direct_navigation_attempt_blocked_across_entry_points() {
    let mut app = DiskLensApp::default();
    let ctx = egui::Context::default();
    let screen_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 720.0));

    // 1. Keyboard shortcuts 1 through 5 must NOT open analysis views when unavailable
    let shortcut_keys = [Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5];
    for key in shortcut_keys {
        let mut raw_input = egui::RawInput::default();
        raw_input.screen_rect = Some(screen_rect);
        raw_input.events.push(egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        });

        let _ = ctx.run(raw_input, |ctx| {
            app.render_app_ui(ctx);
        });

        assert_eq!(
            app.selected_tab,
            ActiveTab::Storage,
            "Shortcut {:?} must not activate view when analysis is unavailable",
            key
        );
    }

    // 2. Direct assignment bypass is caught and reverted by the render guardrail
    app.selected_tab = ActiveTab::Treemap; // Attempt direct assignment bypass
    let mut raw_input = egui::RawInput::default();
    raw_input.screen_rect = Some(screen_rect);

    let _ = ctx.run(raw_input, |ctx| {
        app.render_app_ui(ctx);
    });

    assert_eq!(
        app.selected_tab,
        ActiveTab::Storage,
        "Guardrail must revert direct assignment to Storage Home when analysis data is unavailable"
    );
}
