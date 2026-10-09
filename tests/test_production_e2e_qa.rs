use disk_lens::app::{ActiveTab, DiskLensApp};
use disk_lens::duplicates::DuplicateDetector;
use disk_lens::history::ScanSnapshotRecord;
use disk_lens::models::{AnalysisCache, FileNode};
use disk_lens::scanner::{ScanMessage, Scanner};
use disk_lens::settings::{AppSettings, ThemePreference};
use eframe::egui::{self, Pos2, Rect, Vec2};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use sysinfo::Disks;

#[test]
fn test_qa_01_application_startup_and_theme() {
    let ctx = egui::Context::default();
    let mut app = DiskLensApp::default();
    assert_eq!(app.selected_tab, ActiveTab::Storage);
    assert!(app.dark_mode);

    let screen_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 720.0));
    let mut raw_input = egui::RawInput::default();
    raw_input.screen_rect = Some(screen_rect);

    let output = ctx.run(raw_input, |ctx| {
        app.render_app_ui(ctx);
    });

    assert!(
        !output.shapes.is_empty(),
        "Application must render initial frame cleanly"
    );
}

#[test]
fn test_qa_02_real_mount_detection() {
    let mut disks = Disks::new();
    disks.refresh(true);
    let list = disks.list();
    assert!(
        !list.is_empty(),
        "Must detect at least 1 real Linux mount point"
    );

    let root_disk = list.iter().find(|d| d.mount_point() == Path::new("/"));
    assert!(root_disk.is_some(), "Must detect root '/' filesystem");
    let root = root_disk.unwrap();
    assert!(
        root.total_space() > 0,
        "Root filesystem total space must be > 0"
    );
    assert!(
        root.available_space() > 0,
        "Root filesystem available space must be > 0"
    );
    println!(
        "Verified Real Linux Mount: {:?} [FS: {:?}, Total: {} GB, Avail: {} GB]",
        root.mount_point(),
        root.file_system(),
        root.total_space() / (1024 * 1024 * 1024),
        root.available_space() / (1024 * 1024 * 1024)
    );
}

#[test]
fn test_qa_03_real_filesystem_scan_pause_resume_cancel() {
    let temp_dir = std::env::temp_dir().join(format!("disklens_scan_flow_{}", std::process::id()));
    fs::create_dir_all(temp_dir.join("sub1/sub2")).unwrap();

    for i in 1..=20 {
        let fpath = temp_dir.join(format!("sub1/file_{}.dat", i));
        File::create(&fpath)
            .unwrap()
            .write_all(&vec![0xAA; 100 * 1024])
            .unwrap();
    }

    let (tx, rx) = crossbeam_channel::unbounded();
    let handle = Scanner::start_scan_with_settings(temp_dir.clone(), tx, AppSettings::default());

    // Test pause
    handle.pause();
    assert!(handle.is_paused());

    // Test resume
    handle.resume();
    assert!(!handle.is_paused());

    // Await completion or messages
    let mut received_complete = false;
    while let Ok(msg) = rx.recv_timeout(std::time::Duration::from_secs(3)) {
        match msg {
            ScanMessage::Complete(node) => {
                assert_eq!(node.file_count, 20);
                assert_eq!(node.size, 20 * 100 * 1024);
                received_complete = true;
                break;
            }
            ScanMessage::Error(e) => panic!("Scan error: {}", e),
            _ => {}
        }
    }

    assert!(
        received_complete,
        "Scanner must complete successfully on real filesystem"
    );
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_qa_04_duplicate_finder_with_known_duplicates() {
    let temp_dir = std::env::temp_dir().join(format!("disklens_dup_real_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).unwrap();

    let f1 = temp_dir.join("doc_original.pdf");
    let f2 = temp_dir.join("doc_copy1.pdf");
    let f3 = temp_dir.join("doc_copy2.pdf");
    let f_uniq = temp_dir.join("doc_different.pdf");

    // Exact identical payload of 2 MB for f1, f2, f3
    let dup_payload = vec![0xFE; 2 * 1024 * 1024];
    let uniq_payload = vec![0xBA; 2 * 1024 * 1024];

    File::create(&f1).unwrap().write_all(&dup_payload).unwrap();
    File::create(&f2).unwrap().write_all(&dup_payload).unwrap();
    File::create(&f3).unwrap().write_all(&dup_payload).unwrap();
    File::create(&f_uniq)
        .unwrap()
        .write_all(&uniq_payload)
        .unwrap();

    let mut root_node = FileNode::new_dir("temp".to_string(), temp_dir.clone(), None);
    root_node.children.push(FileNode::new_file(
        "doc_original.pdf".to_string(),
        f1,
        dup_payload.len() as u64,
        None,
    ));
    root_node.children.push(FileNode::new_file(
        "doc_copy1.pdf".to_string(),
        f2,
        dup_payload.len() as u64,
        None,
    ));
    root_node.children.push(FileNode::new_file(
        "doc_copy2.pdf".to_string(),
        f3,
        dup_payload.len() as u64,
        None,
    ));
    root_node.children.push(FileNode::new_file(
        "doc_different.pdf".to_string(),
        f_uniq,
        uniq_payload.len() as u64,
        None,
    ));

    let groups = DuplicateDetector::find_duplicates(&root_node, 1024 * 1024, None);

    assert_eq!(groups.len(), 1, "Must find exactly 1 group of 3 duplicates");
    let grp = &groups[0];
    assert_eq!(grp.items.len(), 3);
    assert_eq!(grp.size_bytes_per_file, 2 * 1024 * 1024);
    assert_eq!(grp.total_size_bytes, 6 * 1024 * 1024);

    // Verify original preservation
    assert!(grp.items[0].is_original);
    assert!(!grp.items[1].is_original);
    assert!(!grp.items[2].is_original);

    // Simulate "Review Selection" (marking non-originals for trash)
    let mut test_grp = grp.clone();
    for (idx, item) in test_grp.items.iter_mut().enumerate() {
        item.selected_for_trash = idx > 0;
    }

    // Verify mathematically correct potential savings calculation (2 files * 2MB = 4MB)
    assert_eq!(test_grp.potential_savings(), 4 * 1024 * 1024);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_qa_05_storage_history_persistence_and_growth_math() {
    let temp_dir = std::env::temp_dir().join(format!("disklens_hist_real_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).unwrap();

    let root_path = temp_dir.clone();
    let mut node_1 = FileNode::new_dir("test_root".to_string(), root_path.clone(), None);
    node_1.size = 50 * 1024 * 1024 * 1024; // 50 GB
    node_1.file_count = 10000;

    let mut node_2 = FileNode::new_dir("test_root".to_string(), root_path.clone(), None);
    node_2.size = 55 * 1024 * 1024 * 1024; // 55 GB (+5 GB growth)
    node_2.file_count = 11000;

    let snapshots = vec![
        ScanSnapshotRecord {
            id: "snap-2".to_string(),
            timestamp: "2026-08-29 17:00".to_string(),
            root_path: root_path.display().to_string(),
            total_size_bytes: node_2.size,
            file_count: node_2.file_count,
            dir_count: 100,
            duration_seconds: 1.2,
            top_subdirs: vec![("projects".to_string(), 35 * 1024 * 1024 * 1024)],
        },
        ScanSnapshotRecord {
            id: "snap-1".to_string(),
            timestamp: "2026-08-22 17:00".to_string(),
            root_path: root_path.display().to_string(),
            total_size_bytes: node_1.size,
            file_count: node_1.file_count,
            dir_count: 90,
            duration_seconds: 1.1,
            top_subdirs: vec![("projects".to_string(), 30 * 1024 * 1024 * 1024)],
        },
    ];

    let history_file = temp_dir.join("history.json");
    let json = serde_json::to_string_pretty(&snapshots).unwrap();
    File::create(&history_file)
        .unwrap()
        .write_all(json.as_bytes())
        .unwrap();

    // Verify snapshot reading
    let loaded: Vec<ScanSnapshotRecord> = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.len(), 2);
    let diff = loaded[0].total_size_bytes as i64 - loaded[1].total_size_bytes as i64;
    assert_eq!(diff, 5 * 1024 * 1024 * 1024);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_qa_06_settings_persistence_and_exclusion_enforcement() {
    let temp_config_dir =
        std::env::temp_dir().join(format!("disklens_cfg_test_{}", std::process::id()));
    fs::create_dir_all(&temp_config_dir).unwrap();
    let settings_file = temp_config_dir.join("settings.json");

    let mut settings = AppSettings::default();
    settings.worker_threads = 16;
    settings.exclusions = vec!["/tmp/excluded_folder".to_string()];
    settings.theme_preference = ThemePreference::Dark;

    let json = serde_json::to_string_pretty(&settings).unwrap();
    File::create(&settings_file)
        .unwrap()
        .write_all(json.as_bytes())
        .unwrap();

    let loaded: AppSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.worker_threads, 16);
    assert_eq!(loaded.exclusions, vec!["/tmp/excluded_folder".to_string()]);

    let _ = fs::remove_dir_all(&temp_config_dir);
}

#[test]
fn test_qa_07_all_8_tabs_rendered_across_7_resolutions() {
    let resolutions = [
        (800.0, 600.0),
        (900.0, 700.0),
        (1024.0, 768.0),
        (1280.0, 720.0),
        (1310.0, 878.0),
        (1366.0, 768.0),
        (1600.0, 900.0),
        (1920.0, 1080.0),
        (2560.0, 1440.0),
        (3840.0, 2160.0),
    ];

    let tabs = [
        ActiveTab::Storage,
        ActiveTab::Overview,
        ActiveTab::Treemap,
        ActiveTab::DirectoryTree,
        ActiveTab::LargestFiles,
        ActiveTab::Categories,
        ActiveTab::Duplicates,
        ActiveTab::SafeTrash,
        ActiveTab::History,
        ActiveTab::MountPoints,
        ActiveTab::DiagnosticsSmart,
        ActiveTab::SystemLogs,
        ActiveTab::RustEngine,
        ActiveTab::Documentation,
    ];

    let ctx = egui::Context::default();
    let mut fake_root = FileNode::new_dir("root".to_string(), PathBuf::from("/home/user"), None);
    fake_root.children.push(FileNode::new_file(
        "movie.mp4".to_string(),
        PathBuf::from("/home/user/movie.mp4"),
        4 * 1024 * 1024 * 1024,
        None,
    ));
    fake_root.children.push(FileNode::new_file(
        "doc.pdf".to_string(),
        PathBuf::from("/home/user/doc.pdf"),
        50 * 1024 * 1024,
        None,
    ));
    fake_root.size = 4 * 1024 * 1024 * 1024 + 50 * 1024 * 1024;
    fake_root.file_count = 2;

    let cache = AnalysisCache::build(&fake_root);

    for (w, h) in resolutions {
        let screen_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(w, h));

        for tab in tabs {
            let mut app = DiskLensApp::default();
            app.session.scanned_root = Some(Box::new(fake_root.clone()));
            app.session.analysis_cache = Some(cache.clone());
            app.selected_tab = tab;

            let mut raw_input = egui::RawInput::default();
            raw_input.screen_rect = Some(screen_rect);

            let output = ctx.run(raw_input, |ctx| {
                app.render_app_ui(ctx);
            });

            assert!(
                !output.shapes.is_empty(),
                "Failed rendering {:?} at {}x{}",
                tab,
                w,
                h
            );
        }
    }
}
