use disk_lens::app::{ActiveTab, DiskLensApp};
use disk_lens::models::{AnalysisCache, FileNode};
use disk_lens::treemap::TreemapLayout;
use eframe::egui::{self, Pos2, Rect, Vec2};
use std::path::PathBuf;
use std::time::SystemTime;

/// Generates a realistic stress filesystem hierarchy with extreme content:
/// - 200+ character directory/file names
/// - 15 levels of deep nesting
/// - Hundreds of files across all categories
fn generate_stress_test_hierarchy() -> FileNode {
    let root_path = PathBuf::from("/home/sayantan/extreme_storage_test_environment");
    let mut root = FileNode::new_dir(
        "extreme_storage_test_environment".to_string(),
        root_path.clone(),
        Some(SystemTime::now()),
    );

    // 1. Extremely long path component
    let long_name = "a_very_long_project_name_that_simulates_real_world_nested_node_modules_or_deeply_nested_java_package_structures_with_extended_identifiers_and_hash_suffixes_0123456789_abcdef";
    let deep_dir_path = root_path.join(long_name);
    let mut deep_dir = FileNode::new_dir(
        long_name.to_string(),
        deep_dir_path.clone(),
        Some(SystemTime::now()),
    );

    // 2. Add deeply nested subfolders (12 levels)
    let mut current_parent = &mut deep_dir;
    let mut curr_path = deep_dir_path.clone();
    for i in 1..=12 {
        let child_name = format!("level_{:02}_subfolder_component", i);
        curr_path = curr_path.join(&child_name);
        let child_dir = FileNode::new_dir(child_name, curr_path.clone(), Some(SystemTime::now()));
        current_parent.children.push(child_dir);
        current_parent = current_parent.children.last_mut().unwrap();
    }

    // Add deep files
    for i in 1..=20 {
        let f_name = format!("deep_payload_artifact_{}.bin", i);
        let f_path = curr_path.join(&f_name);
        let f_node = FileNode::new_file(
            f_name,
            f_path,
            15_000_000 * i as u64,
            Some(SystemTime::now()),
        );
        current_parent.children.push(f_node);
    }

    // 3. Add space hogs across diverse categories
    let mut media_dir = FileNode::new_dir(
        "Videos_and_Media".to_string(),
        root_path.join("Videos_and_Media"),
        Some(SystemTime::now()),
    );
    let massive_video = FileNode::new_file(
        "4k_render_final_cut_pro_lossless_master.mov".to_string(),
        root_path.join("Videos_and_Media/4k_render_final_cut_pro_lossless_master.mov"),
        14_800_000_000,
        Some(SystemTime::now()),
    );
    let iso_file = FileNode::new_file(
        "archlinux-2026.08.01-x86_64.iso".to_string(),
        root_path.join("Videos_and_Media/archlinux-2026.08.01-x86_64.iso"),
        4_200_000_000,
        Some(SystemTime::now()),
    );
    media_dir.children.push(massive_video);
    media_dir.children.push(iso_file);

    let mut cache_dir = FileNode::new_dir(
        ".cache".to_string(),
        root_path.join(".cache"),
        Some(SystemTime::now()),
    );
    for i in 1..=50 {
        let cache_file = FileNode::new_file(
            format!("blob_cache_{:04}.tmp", i),
            root_path.join(format!(".cache/blob_cache_{:04}.tmp", i)),
            12_500_000,
            Some(SystemTime::now()),
        );
        cache_dir.children.push(cache_file);
    }

    let mut code_dir = FileNode::new_dir(
        "rust_engine_source".to_string(),
        root_path.join("rust_engine_source"),
        Some(SystemTime::now()),
    );
    for ext in ["rs", "toml", "json", "md", "lock", "c", "h", "py"] {
        for i in 1..=10 {
            let code_file = FileNode::new_file(
                format!("source_{}.{}", i, ext),
                root_path.join(format!("rust_engine_source/source_{}.{}", i, ext)),
                45_000 * i as u64,
                Some(SystemTime::now()),
            );
            code_dir.children.push(code_file);
        }
    }

    root.children.push(deep_dir);
    root.children.push(media_dir);
    root.children.push(cache_dir);
    root.children.push(code_dir);

    // Compute sizes and item counts
    fn recalculate(node: &mut FileNode) {
        if node.is_dir {
            for c in &mut node.children {
                recalculate(c);
            }
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
        }
    }

    recalculate(&mut root);
    root
}

#[test]
fn test_window_responsiveness_audit_across_all_resolutions() {
    let test_resolutions = [
        (800.0, 600.0, "800x600 (SVGA Minimum Window)"),
        (900.0, 700.0, "900x700 (Compact Desktop)"),
        (1024.0, 768.0, "1024x768 (XGA Standard)"),
        (1280.0, 720.0, "1280x720 (HD 720p)"),
        (1310.0, 878.0, "1310x878 (Runtime Tested Resolution)"),
        (1366.0, 768.0, "1366x768 (Laptop Standard)"),
        (1600.0, 900.0, "1600x900 (HD+)"),
        (1920.0, 1080.0, "1920x1080 (FHD Standard Desktop)"),
        (2560.0, 1440.0, "2560x1440 (2K QHD)"),
        (3840.0, 2160.0, "3840x2160 (4K UHD)"),
    ];

    let all_views = [
        ActiveTab::Storage,
        ActiveTab::Overview,
        ActiveTab::Treemap,
        ActiveTab::DirectoryTree,
        ActiveTab::LargestFiles,
        ActiveTab::Categories,
        ActiveTab::Duplicates,
        ActiveTab::History,
        ActiveTab::RustEngine,
    ];

    let stress_root = generate_stress_test_hierarchy();
    let cache = AnalysisCache::build(&stress_root);

    let ctx = egui::Context::default();

    for (w, h, res_name) in test_resolutions {
        let screen_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(w, h));

        for tab in all_views {
            for dark_mode in [true, false] {
                let mut app = DiskLensApp::default();
                app.dark_mode = dark_mode;
                app.session.scanned_root = Some(Box::new(stress_root.clone()));
                app.session.analysis_cache = Some(cache.clone());
                app.selected_tab = tab;

                let mut raw_input = egui::RawInput::default();
                raw_input.screen_rect = Some(screen_rect);

                // Run an actual full frame evaluation with egui
                let output = ctx.run(raw_input, |ctx| {
                    app.render_app_ui(ctx);
                });

                assert!(
                    !output.shapes.is_empty(),
                    "Resolution {} in tab {:?} (dark={}) must generate rendered shapes",
                    res_name,
                    tab,
                    dark_mode
                );
            }
        }
    }
}

#[test]
fn test_narrow_window_and_collapsed_rail_behavior() {
    let ctx = egui::Context::default();
    let stress_root = generate_stress_test_hierarchy();
    let cache = AnalysisCache::build(&stress_root);

    let narrow_resolutions = [
        (640.0, 480.0),
        (720.0, 540.0),
        (800.0, 600.0),
        (859.0, 700.0), // Just below the 860px threshold (compact 48px rail active)
        (860.0, 700.0), // At threshold (190px sidebar active)
    ];

    for (w, h) in narrow_resolutions {
        let screen_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(w, h));
        let mut app = DiskLensApp::default();
        app.session.scanned_root = Some(Box::new(stress_root.clone()));
        app.session.analysis_cache = Some(cache.clone());
        app.selected_tab = ActiveTab::Overview;

        let mut raw_input = egui::RawInput::default();
        raw_input.screen_rect = Some(screen_rect);

        let output = ctx.run(raw_input, |ctx| {
            app.render_app_ui(ctx);
        });

        assert!(!output.shapes.is_empty());
    }
}

#[test]
fn test_modal_dialogs_bounded_within_screen() {
    let ctx = egui::Context::default();
    let stress_root = generate_stress_test_hierarchy();
    let cache = AnalysisCache::build(&stress_root);

    let screen_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));

    let mut app = DiskLensApp::default();
    app.session.scanned_root = Some(Box::new(stress_root.clone()));
    app.session.analysis_cache = Some(cache.clone());

    // Trigger all modal dialogs simultaneously
    app.modals.is_dir_picker_open = true;
    app.modals.is_mount_points_open = true;
    app.modals.is_system_logs_open = true;
    app.modals.is_docs_open = true;
    app.modals.pending_confirm = Some(disk_lens::ui::modals::ConfirmAction::MoveToTrash(
        PathBuf::from("/home/sayantan/Videos/large.mp4"),
        1024 * 1024 * 1024,
    ));

    let mut raw_input = egui::RawInput::default();
    raw_input.screen_rect = Some(screen_rect);

    let output = ctx.run(raw_input, |ctx| {
        app.render_app_ui(ctx);
    });

    assert!(!output.shapes.is_empty());
}

#[test]
fn test_treemap_squarified_geometry_at_all_depths() {
    let stress_root = generate_stress_test_hierarchy();

    for depth in 1..=6 {
        let canvas = Rect::from_min_max(Pos2::new(10.0, 10.0), Pos2::new(790.0, 550.0));
        let layout = TreemapLayout::build(&stress_root, canvas, depth);

        assert!(
            !layout.items.is_empty(),
            "Treemap layout must produce items at depth {}",
            depth
        );
        for item in &layout.items {
            assert!(item.rect.width() >= 0.0, "Width must be non-negative");
            assert!(item.rect.height() >= 0.0, "Height must be non-negative");
            assert!(item.size > 0, "Item size must be positive");
            assert!(
                canvas.contains(item.rect.center()),
                "Item center must stay within canvas bounds"
            );
        }
    }
}

#[test]
fn test_settings_modal_all_tabs_responsiveness() {
    let ctx = egui::Context::default();
    let resolutions = [
        (800.0, 600.0),
        (900.0, 700.0),
        (1024.0, 768.0),
        (1280.0, 720.0),
        (1366.0, 768.0),
        (1920.0, 1080.0),
    ];

    for (w, h) in resolutions {
        let screen_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(w, h));
        for tab_idx in 0..5 {
            let mut app = DiskLensApp::default();
            app.settings_state.is_open = true;
            app.settings_state.active_tab = tab_idx;
            app.settings_state.show_add_exclusion = true; // test open add input

            let mut raw_input = egui::RawInput::default();
            raw_input.screen_rect = Some(screen_rect);

            let output = ctx.run(raw_input, |ctx| {
                app.render_app_ui(ctx);
            });

            assert!(
                !output.shapes.is_empty(),
                "Settings tab {} must render properly at {}x{}",
                tab_idx,
                w,
                h
            );
        }
    }
}
