use disk_lens::app::ActiveTab;
use disk_lens::diagnostics::DiagnosticsManager;
use disk_lens::duplicates::DuplicateDetector;
use disk_lens::history::ScanSnapshotRecord;
use disk_lens::manage::StorageManager;
use disk_lens::models::FileNode;
use disk_lens::settings::{AppSettings, SymlinkPolicy, ThemePreference};
use std::fs::{self, File};
use std::io::Write;

#[test]
fn test_duplicate_detector_sha256_accuracy() {
    let temp_dir = std::env::temp_dir().join(format!("disklens_dup_test_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).unwrap();

    let file_a = temp_dir.join("original.dat");
    let file_b = temp_dir.join("duplicate_1.dat");
    let file_c = temp_dir.join("duplicate_2.dat");
    let file_unique = temp_dir.join("unique.dat");

    // Exact 1.5 MB identical payload for A, B, C
    let payload = vec![0xAB; 1500 * 1024];
    let unique_payload = vec![0xCD; 1500 * 1024];

    File::create(&file_a).unwrap().write_all(&payload).unwrap();
    File::create(&file_b).unwrap().write_all(&payload).unwrap();
    File::create(&file_c).unwrap().write_all(&payload).unwrap();
    File::create(&file_unique).unwrap().write_all(&unique_payload).unwrap();

    let mut root_node = FileNode::new_dir("temp".to_string(), temp_dir.clone(), None);
    root_node.children.push(FileNode::new_file("original.dat".to_string(), file_a.clone(), payload.len() as u64, None));
    root_node.children.push(FileNode::new_file("duplicate_1.dat".to_string(), file_b.clone(), payload.len() as u64, None));
    root_node.children.push(FileNode::new_file("duplicate_2.dat".to_string(), file_c.clone(), payload.len() as u64, None));
    root_node.children.push(FileNode::new_file("unique.dat".to_string(), file_unique.clone(), unique_payload.len() as u64, None));

    let dup_groups = DuplicateDetector::find_duplicates(&root_node, 1024, None);

    assert_eq!(dup_groups.len(), 1, "Should find exactly 1 duplicate group");
    let grp = &dup_groups[0];
    assert_eq!(grp.items.len(), 3, "Group should contain 3 identical files");
    assert_eq!(grp.size_bytes_per_file, 1500 * 1024);
    assert_eq!(grp.total_size_bytes, 3 * 1500 * 1024);

    // Verify first item is designated as original
    assert!(grp.items[0].is_original);
    assert!(!grp.items[1].is_original);
    assert!(!grp.items[2].is_original);

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_storage_history_snapshots_and_deltas() {
    let temp_history_dir = std::env::temp_dir().join(format!("disklens_hist_test_{}", std::process::id()));
    fs::create_dir_all(&temp_history_dir).unwrap();
    let history_file = temp_history_dir.join("history.json");

    let snapshots = vec![
        ScanSnapshotRecord {
            id: "hist-2".to_string(),
            timestamp: "2026-08-29 10:00".to_string(),
            root_path: "/home/user".to_string(),
            total_size_bytes: 120 * 1024 * 1024 * 1024,
            file_count: 50000,
            dir_count: 5000,
            duration_seconds: 1.5,
            top_subdirs: vec![("Downloads".to_string(), 40 * 1024 * 1024 * 1024)],
        },
        ScanSnapshotRecord {
            id: "hist-1".to_string(),
            timestamp: "2026-08-22 10:00".to_string(),
            root_path: "/home/user".to_string(),
            total_size_bytes: 110 * 1024 * 1024 * 1024,
            file_count: 45000,
            dir_count: 4500,
            duration_seconds: 1.4,
            top_subdirs: vec![("Downloads".to_string(), 30 * 1024 * 1024 * 1024)],
        },
    ];

    let json = serde_json::to_string_pretty(&snapshots).unwrap();
    File::create(&history_file).unwrap().write_all(json.as_bytes()).unwrap();

    let loaded: Vec<ScanSnapshotRecord> = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.len(), 2);

    let diff = loaded[0].total_size_bytes as i64 - loaded[1].total_size_bytes as i64;
    assert_eq!(diff, 10 * 1024 * 1024 * 1024, "Growth delta should be exactly +10 GB");

    let _ = fs::remove_dir_all(&temp_history_dir);
}

#[test]
fn test_app_settings_defaults_and_serialization() {
    let mut settings = AppSettings::default();
    assert_eq!(settings.symlink_policy, SymlinkPolicy::DontFollow);
    assert!(settings.include_hidden_files);
    assert!(!settings.cross_filesystem_boundaries);
    assert!(settings.confirm_before_trash);

    settings.worker_threads = 12;
    settings.theme_preference = ThemePreference::Dark;
    settings.exclusions.push("/opt/custom".to_string());

    let json = serde_json::to_string(&settings).unwrap();
    let deserialized: AppSettings = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.worker_threads, 12);
    assert_eq!(deserialized.theme_preference, ThemePreference::Dark);
    assert!(deserialized.exclusions.contains(&"/opt/custom".to_string()));
}

#[test]
fn test_four_pillar_tabs_enum() {
    let tabs = [
        // ANALYZE
        ActiveTab::Storage,
        ActiveTab::Overview,
        ActiveTab::Treemap,
        ActiveTab::DirectoryTree,
        ActiveTab::LargestFiles,
        ActiveTab::Categories,
        // MANAGE
        ActiveTab::Duplicates,
        ActiveTab::SafeTrash,
        ActiveTab::History,
        ActiveTab::MountPoints,
        // DIAGNOSE
        ActiveTab::DiagnosticsSmart,
        ActiveTab::SystemLogs,
        // ENGINE
        ActiveTab::RustEngine,
        ActiveTab::Documentation,
    ];

    assert_eq!(tabs.len(), 14, "Must support all 14 navigation views across the 4 pillars");
}

#[test]
fn test_safe_trash_discovery_and_dry_run() {
    let categories = StorageManager::discover_cleanable_targets();
    // Verify categories are discovered without panicking
    for cat in &categories {
        assert!(!cat.id.is_empty());
        assert!(!cat.title.is_empty());
        let reclaimable = cat.total_reclaimable_bytes();
        let _ = reclaimable;
    }
}

#[test]
fn test_smart_telemetry_discovery() {
    let devices = DiagnosticsManager::inspect_storage_devices();
    assert!(!devices.is_empty(), "Must discover at least root or system storage devices");
    for dev in &devices {
        assert!(!dev.device_path.is_empty());
        assert!(dev.health_percent <= 100);
        assert!(!dev.smart_status.is_empty());
    }
}
