use crate::theme::AppTheme;
use eframe::egui::{self, Margin, RichText, Ui};

pub fn render_docs_view(ui: &mut Ui, dark_mode: bool) {
    let t = AppTheme::tokens(dark_mode);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let scrollbar_margin = ui.spacing().scroll.bar_width + 14.0;
            let available_w = (ui.available_width() - scrollbar_margin).max(320.0);
            let content_w = available_w;

            ui.vertical(|ui| {
                ui.set_max_width(content_w);
                ui.add_space(8.0);

                // 1. Top Banner
                egui::Frame::none()
                    .fill(t.bg_elevated)
                    .rounding(6.0_f32)
                    .stroke(egui::Stroke::new(1.0_f32, t.accent))
                    .inner_margin(Margin::same(16.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("📖").size(28.0).color(t.accent));
                            ui.vertical(|ui| {
                                ui.label(RichText::new("DiskLens Architecture & User Manual").size(16.0).strong().color(t.text));
                                ui.add(egui::Label::new(RichText::new("Native Linux storage analysis, io_uring architecture, deduplication, and safe reclamation guide.").size(11.5).color(t.text_secondary)).wrap());
                            });
                        });
                    });

                ui.add_space(14.0);

                // 2. 4 Architecture Pillar Cards
                let is_single_col = content_w < 700.0;
                let card_inner_w = if is_single_col {
                    (content_w - 28.0).max(100.0)
                } else {
                    (((content_w - 12.0) / 2.0) - 28.0).floor().max(100.0)
                };

                let pillars = [
                    (
                        "⚡ High-Throughput Rust Engine",
                        "DiskLens employs a dedicated Rust daemon (disklens-core). On modern Linux kernels (5.1+), it leverages asynchronous io_uring system calls to batch directory traversal requests, indexing over 1,000,000 files in under 1.5s with negligible memory footprint.",
                        t.accent,
                    ),
                    (
                        "🛡 Safe Trash & Deletion Isolation",
                        "Unlike destructive CLI utilities, DiskLens adheres strictly to the FreeDesktop.org Trash Specification. Deleted files are safely staged in ~/.local/share/Trash with rollback metadata, preventing accidental data loss.",
                        t.success,
                    ),
                    (
                        "🖴 Btrfs Subvolumes & Reflink Deduplication",
                        "When operating on copy-on-write filesystems like Btrfs or ZFS, DiskLens can convert duplicate file instances into lightweight block reflinks, reclaiming 100% of redundant storage immediately without path fragmentation.",
                        t.purple,
                    ),
                    (
                        "🦀 Command-Line Integration (disklens-cli)",
                        "All capabilities exposed in the desktop GUI are accessible via scriptable CLI endpoints. Use disklens-cli --scan / --json for headless server monitoring, automated Prometheus exporters, or scheduled cron health checks.",
                        t.warning,
                    ),
                ];

                for chunk in pillars.chunks(if is_single_col { 1 } else { 2 }) {
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = 12.0;
                        for (title, text, color) in chunk {
                            egui::Frame::none()
                                .fill(t.bg_container)
                                .rounding(6.0_f32)
                                .stroke(egui::Stroke::new(1.0_f32, t.border))
                                .inner_margin(Margin::same(14.0))
                                .show(ui, |ui| {
                                    ui.set_width(card_inner_w);
                                    ui.label(RichText::new(*title).size(13.0).strong().color(*color));
                                    ui.add_space(6.0);
                                    ui.add(egui::Label::new(RichText::new(*text).size(11.5).color(t.text_secondary)).wrap());
                                });
                        }
                    });
                    ui.add_space(10.0);
                }

                ui.add_space(10.0);

                // 3. Keyboard Shortcuts Matrix
                egui::Frame::none()
                    .fill(t.bg_elevated)
                    .rounding(6.0_f32)
                    .stroke(egui::Stroke::new(1.0_f32, t.border))
                    .inner_margin(Margin::same(14.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new("GLOBAL KEYBOARD SHORTCUTS").size(11.0).strong().color(t.text_muted));
                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(6.0);

                        let shortcuts = [
                            ("Ctrl+K / Cmd+K", "Global Command Palette & Quick Search"),
                            ("1", "Overview Allocation Dashboard"),
                            ("2", "Visual Squarified Treemap"),
                            ("3", "Hierarchical Folder Tree Explorer"),
                            ("4", "Space Hogs Leaderboard"),
                            ("5", "File Types & MIME Analytics"),
                            ("6", "Duplicate Finder (SHA-256)"),
                            ("7", "Storage Growth Trends & History"),
                            ("8", "Rust Engine & Interactive CLI Shell"),
                            ("R", "Rescan Target Directory"),
                            ("T", "Toggle Dark / Tokyo Night / Light Theme"),
                            (", / S", "Application Preferences & Settings"),
                            ("Esc", "Dismiss Modals / Clear Search Focus"),
                        ];

                        let s_cols = if content_w > 720.0 { 2 } else { 1 };
                        egui::Grid::new("shortcuts_grid")
                            .num_columns(if s_cols == 2 { 4 } else { 2 })
                            .spacing([14.0, 6.0])
                            .show(ui, |ui| {
                                for chunk in shortcuts.chunks(s_cols) {
                                    for (key, desc) in chunk {
                                        ui.label(RichText::new(format!("[ {} ]", key)).size(10.5).monospace().strong().color(t.accent));
                                        ui.label(RichText::new(*desc).size(11.0).color(t.text));
                                    }
                                    ui.end_row();
                                }
                            });
                    });

                ui.add_space(20.0);
            });
        });
}
