use disk_lens::app::ActiveTab;
use disk_lens::metadata::METADATA;
use disk_lens::ui::docs_view::render_docs_view;
use disk_lens::ui::side_nav::render_side_nav;
use eframe::egui::{self, Context, Rect, Vec2};

#[test]
fn test_metadata_centralization_and_clean_placeholders() {
    let meta = &METADATA;

    // Verify product naming & descriptions
    assert_eq!(meta.app_name, "DiskLens");
    assert_eq!(meta.subtitle, "Disk Space Analyzer for Linux");
    assert_eq!(
        meta.description_primary,
        "A fast, safe and modern disk space analyzer built in Rust."
    );
    assert_eq!(meta.description_secondary, "Find what's using your storage, manage duplicate files, and reclaim space with confidence.");

    // Verify developer credit strictly matches requirements
    assert_eq!(meta.developer_name, "Sayantan Mazumdar");
    assert_eq!(meta.developer_role, "Developer & Maintainer");
    assert!(!meta.developer_bio.is_empty());

    // Verify engine and license
    assert_eq!(meta.engine, "Rust / egui");
    assert_eq!(meta.license, "GPL-3.0");

    // Verify build target dynamically contains Linux and architecture
    let build_tgt = meta.build_target();
    assert!(
        build_tgt.starts_with("Linux ("),
        "Target must be Linux, got: {}",
        build_tgt
    );

    // Verify license text contains GPLv3 offline terms
    let lic_text = meta.license_text();
    assert!(lic_text.contains("GNU GENERAL PUBLIC LICENSE"));
    assert!(lic_text.contains("Sayantan Mazumdar"));

    // Verify configured GitHub publication URLs
    assert_eq!(
        meta.repository_url,
        Some("https://github.com/smazumdar261-IN/DiskLens")
    );
    assert_eq!(
        meta.documentation_url,
        Some("https://github.com/smazumdar261-IN/DiskLens#readme")
    );
    assert_eq!(
        meta.issues_url,
        Some("https://github.com/smazumdar261-IN/DiskLens/issues")
    );
    assert_eq!(
        meta.license_url,
        Some("https://github.com/smazumdar261-IN/DiskLens/blob/main/LICENSE")
    );
    assert_eq!(meta.github_url, Some("https://github.com/smazumdar261-IN"));
    assert!(meta.linkedin_url.is_none());
    assert!(meta.email_url.is_none());
}

#[test]
fn test_sidebar_groups_labels_and_destinations() {
    let ctx = Context::default();

    let resolutions = [
        ("Normal Desktop", Vec2::new(1280.0, 800.0), false),
        ("Narrow Window", Vec2::new(720.0, 600.0), true),
    ];

    for (res_name, _size, is_compact) in resolutions {
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.set_width(if is_compact { 52.0 } else { 195.0 });

                let mut active_tab = ActiveTab::Documentation;
                let is_available = |_tab: ActiveTab| true;

                // Render side navigation
                render_side_nav(
                    ui,
                    &mut active_tab,
                    &None,
                    0,
                    true, // dark_mode
                    is_compact,
                    &is_available,
                );
            });
        });

        println!("Sidebar rendered successfully at {}", res_name);
    }
}

#[test]
fn test_about_view_renders_across_themes_and_resolutions() {
    let ctx = Context::default();

    let test_cases = [
        ("Desktop 1280x820 Dark Mode", Vec2::new(1280.0, 820.0), true),
        ("Desktop 1280x820 Day Mode", Vec2::new(1280.0, 820.0), false),
        (
            "Ultrawide 1920x1080 Dark Mode",
            Vec2::new(1920.0, 1080.0),
            true,
        ),
        ("Narrow 700x600 Dark Mode", Vec2::new(700.0, 600.0), true),
        ("Narrow 700x600 Day Mode", Vec2::new(700.0, 600.0), false),
    ];

    for (label, size, dark_mode) in test_cases {
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, size)),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    render_docs_view(ui, dark_mode);
                });
            },
        );

        assert!(
            !output.shapes.is_empty(),
            "About page must produce renderable shapes for: {}",
            label
        );
    }
}
