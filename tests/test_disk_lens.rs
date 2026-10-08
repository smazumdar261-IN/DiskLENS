use disk_lens::models::{FileCategory, FileNode};
use disk_lens::scanner::{ScanMessage, Scanner};
use disk_lens::treemap::TreemapLayout;
use eframe::egui::{Pos2, Rect};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use sysinfo::Disks;

#[test]
fn test_real_scanner_and_actual_data_accuracy() {
    let test_dir = std::env::temp_dir().join("disk_lens_accuracy_test");
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).unwrap();

    let sub_dir_a = test_dir.join("sub_images");
    let sub_dir_b = test_dir.join("sub_archives");
    fs::create_dir_all(&sub_dir_a).unwrap();
    fs::create_dir_all(&sub_dir_b).unwrap();

    // 1. Create real files with exact byte contents
    let file1_path = sub_dir_a.join("photo.jpg");
    let mut file1 = File::create(&file1_path).unwrap();
    file1.write_all(&vec![1u8; 50_000]).unwrap(); // 50,000 bytes

    let file2_path = sub_dir_a.join("document.pdf");
    let mut file2 = File::create(&file2_path).unwrap();
    file2.write_all(&vec![2u8; 100_000]).unwrap(); // 100,000 bytes

    let file3_path = sub_dir_b.join("backup.zip");
    let mut file3 = File::create(&file3_path).unwrap();
    file3.write_all(&vec![3u8; 200_000]).unwrap(); // 200,000 bytes

    let file4_path = test_dir.join("main.rs");
    let mut file4 = File::create(&file4_path).unwrap();
    file4.write_all(&vec![4u8; 15_000]).unwrap(); // 15,000 bytes

    let total_expected_bytes = 50_000 + 100_000 + 200_000 + 15_000; // 365,000 bytes

    // 2. Run Scanner on the real directory
    let (tx, rx) = crossbeam_channel::unbounded();
    let _handle = Scanner::start_scan(test_dir.clone(), tx);

    let mut scanned_root: Option<Box<FileNode>> = None;
    while let Ok(msg) = rx.recv() {
        match msg {
            ScanMessage::Complete(root) => {
                scanned_root = Some(root);
                break;
            }
            ScanMessage::Error(err) => panic!("Scan error: {}", err),
            _ => {}
        }
    }

    let root = scanned_root.expect("Scanner failed to produce root FileNode");

    // 3. Assert real sizes and item counts
    println!(
        "Scanned root size: {} bytes, expected: {} bytes",
        root.size, total_expected_bytes
    );
    assert_eq!(root.size, total_expected_bytes);
    assert_eq!(root.file_count, 4);
    assert_eq!(root.dir_count, 2);

    // 4. Test category mapping
    assert_eq!(FileCategory::from_extension("jpg"), FileCategory::Images);
    assert_eq!(FileCategory::from_extension("pdf"), FileCategory::Documents);
    assert_eq!(FileCategory::from_extension("zip"), FileCategory::Archives);
    assert_eq!(FileCategory::from_extension("rs"), FileCategory::Code);

    let mut cat_stats = HashMap::new();
    root.collect_category_stats(&mut cat_stats);

    let (img_count, img_size, img_alloc) = cat_stats.get(&FileCategory::Images).unwrap();
    assert_eq!((*img_count, *img_size), (1, 50_000));
    assert!(*img_alloc >= 50_000);

    let (doc_count, doc_size, doc_alloc) = cat_stats.get(&FileCategory::Documents).unwrap();
    assert_eq!((*doc_count, *doc_size), (1, 100_000));
    assert!(*doc_alloc >= 100_000);

    let (arc_count, arc_size, arc_alloc) = cat_stats.get(&FileCategory::Archives).unwrap();
    assert_eq!((*arc_count, *arc_size), (1, 200_000));
    assert!(*arc_alloc >= 200_000);

    let (code_count, code_size, code_alloc) = cat_stats.get(&FileCategory::Code).unwrap();
    assert_eq!((*code_count, *code_size), (1, 15_000));
    assert!(*code_alloc >= 15_000);

    // 5. Test Largest Files aggregation
    let mut largest = Vec::new();
    root.collect_largest_files(&mut largest);
    largest.sort_by(|a, b| b.size.cmp(&a.size));

    assert_eq!(largest.len(), 4);
    assert_eq!(largest[0].name, "backup.zip");
    assert_eq!(largest[0].size, 200_000);
    assert_eq!(largest[1].name, "document.pdf");
    assert_eq!(largest[1].size, 100_000);
    assert_eq!(largest[2].name, "photo.jpg");
    assert_eq!(largest[2].size, 50_000);
    assert_eq!(largest[3].name, "main.rs");
    assert_eq!(largest[3].size, 15_000);

    // 6. Test Treemap Layout geometry
    let bounds = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1000.0, 800.0));
    let treemap = TreemapLayout::build(&root, bounds, 3);
    assert!(!treemap.items.is_empty());
    for item in &treemap.items {
        assert!(item.rect.width() >= 0.0);
        assert!(item.rect.height() >= 0.0);
        assert!(bounds.contains(item.rect.center()));
    }

    // 7. Verify real system drives detection from Linux kernel
    let mut disks = Disks::new();
    disks.refresh(true);
    assert!(
        !disks.list().is_empty(),
        "Real system disks should be detected"
    );
    for disk in disks.list() {
        println!(
            "Detected disk mount: {:?}, Total: {} bytes, Available: {} bytes",
            disk.mount_point(),
            disk.total_space(),
            disk.available_space()
        );
        assert!(disk.total_space() > 0);
    }

    // Clean up test directory
    let _ = fs::remove_dir_all(&test_dir);
}

#[test]
fn test_storage_insights_explain_unclassified_data() {
    let root_path = std::env::temp_dir().join("disk_lens_insight_test");
    let mut root = FileNode::new_dir(
        "disk_lens_insight_test".to_string(),
        root_path.clone(),
        None,
    );
    let unknown = FileNode::new_file(
        "application-data".to_string(),
        root_path.join("application-data"),
        900,
        None,
    );
    let document = FileNode::new_file(
        "notes.txt".to_string(),
        root_path.join("notes.txt"),
        100,
        None,
    );
    root.children = vec![unknown, document];
    root.size = 1_000;
    root.file_count = 2;

    let insights = root.generate_storage_insights();
    assert!(insights.iter().any(|insight| {
        insight.action_label == "Investigate types" && insight.title.contains("Other / Unknown")
    }));
}
