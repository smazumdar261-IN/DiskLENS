mod actions;
mod app;
mod diagnostics;
mod duplicates;
mod export;
mod history;
mod manage;
mod models;
mod scanner;
mod settings;
mod theme;
mod treemap;
mod ui;

use app::DiskLensApp;
use eframe::egui::{IconData, Vec2, ViewportBuilder};

const ICON_BYTES: &[u8] = include_bytes!("../assets/icon.png");

fn load_app_icon() -> Option<IconData> {
    if let Ok(image) = image::load_from_memory(ICON_BYTES) {
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();
        Some(IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        })
    } else {
        None
    }
}

fn main() -> eframe::Result<()> {
    let mut viewport = ViewportBuilder::default()
        .with_title("DiskLens - Disk Space Analyzer")
        .with_app_id("com.disklens.analyzer")
        .with_inner_size(Vec2::new(1280.0, 820.0))
        .with_min_inner_size(Vec2::new(900.0, 600.0))
        .with_active(true);

    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "DiskLens",
        native_options,
        Box::new(|cc| Ok(Box::new(DiskLensApp::new(cc)))),
    )
}
