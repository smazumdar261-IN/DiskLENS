use crate::metadata::METADATA;
use crate::theme::AppTheme;
use eframe::egui::{
    self, Color32, CursorIcon, Margin, Pos2, Rect, RichText, Sense, Stroke, Ui, Vec2, Window,
};

pub fn render_docs_view(ui: &mut Ui, dark_mode: bool) {
    let t = AppTheme::tokens(dark_mode);

    // Persistent state in egui memory
    let modal_id = ui.id().with("license_modal_open");
    let mut is_license_open = ui
        .ctx()
        .data_mut(|d| d.get_temp::<bool>(modal_id).unwrap_or(false));

    let toast_id = ui.id().with("docs_toast_msg");
    let mut toast_text: Option<String> = ui.ctx().data_mut(|d| d.get_temp(toast_id));

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let scrollbar_margin = ui.spacing().scroll.bar_width + 16.0_f32;
            let available_w = (ui.available_width() - scrollbar_margin).max(360.0_f32);

            ui.vertical(|ui| {
                ui.set_max_width(available_w);
                ui.add_space(8.0_f32);

                // Section Title: "ABOUT"
                ui.label(
                    RichText::new("ABOUT")
                        .size(11.0_f32)
                        .strong()
                        .color(if dark_mode { Color32::from_rgb(148, 163, 184) } else { Color32::from_rgb(100, 116, 139) }),
                );
                ui.add_space(8.0_f32);

                // ==========================================
                // 1. PRODUCT INTRODUCTION HERO CARD
                // ==========================================
                let hero_bg = if dark_mode { Color32::from_rgb(15, 23, 42) } else { Color32::from_rgb(241, 245, 249) };
                let hero_border = if dark_mode { Color32::from_rgb(30, 41, 59) } else { Color32::from_rgb(226, 232, 240) };

                let hero_frame = egui::Frame::none()
                    .fill(hero_bg)
                    .rounding(10.0_f32)
                    .stroke(Stroke::new(1.0_f32, hero_border))
                    .inner_margin(Margin::same(18.0_f32));

                let hero_response = hero_frame.show(ui, |ui| {
                    ui.set_width(available_w - 36.0_f32);

                    let card_rect = ui.max_rect();

                    // Background Night Landscape (dark mode only, rendered on right side)
                    if dark_mode && card_rect.width() > 540.0_f32 {
                        let painter = ui.painter().with_clip_rect(card_rect);

                        // Soft Glowing Moon
                        let moon_center = Pos2::new(card_rect.max.x - 65.0_f32, card_rect.min.y + 45.0_f32);
                        let moon_r = 20.0_f32;

                        // Radial halos
                        painter.circle_filled(moon_center, 44.0_f32, Color32::from_rgba_premultiplied(129, 140, 248, 14));
                        painter.circle_filled(moon_center, 32.0_f32, Color32::from_rgba_premultiplied(129, 140, 248, 28));
                        painter.circle_filled(moon_center, 24.0_f32, Color32::from_rgba_premultiplied(129, 140, 248, 50));
                        painter.circle_filled(moon_center, moon_r, Color32::from_rgb(147, 158, 255));

                        // Twinkling Star Specks
                        let stars = [
                            (Pos2::new(card_rect.max.x - 170.0_f32, card_rect.min.y + 32.0_f32), 1.2_f32),
                            (Pos2::new(card_rect.max.x - 240.0_f32, card_rect.min.y + 44.0_f32), 0.9_f32),
                            (Pos2::new(card_rect.max.x - 110.0_f32, card_rect.min.y + 68.0_f32), 1.1_f32),
                        ];
                        for (sp, sr) in stars {
                            painter.circle_filled(sp, sr, Color32::from_rgba_premultiplied(224, 231, 255, 120));
                        }

                        // Layered Silhouetted Mountains
                        let ridge_start_x = (card_rect.max.x - 320.0_f32).max(card_rect.min.x + 360.0_f32);
                        let ridge_end_x = card_rect.max.x;
                        let bottom_y = card_rect.max.y;

                        // Ridge 1 (Deep background silhouette)
                        let mut pts1 = vec![Pos2::new(ridge_start_x, bottom_y)];
                        let steps = 20;
                        for i in 0..=steps {
                            let frac = i as f32 / steps as f32;
                            let x = ridge_start_x + (ridge_end_x - ridge_start_x) * frac;
                            let wave = (frac * 3.14159_f32 * 2.2_f32).sin() * 18.0_f32 + (frac * 3.14159_f32 * 4.0_f32).cos() * 8.0_f32;
                            let y = card_rect.min.y + 110.0_f32 - wave;
                            pts1.push(Pos2::new(x, y));
                        }
                        pts1.push(Pos2::new(ridge_end_x, bottom_y));
                        painter.add(egui::Shape::convex_polygon(pts1, Color32::from_rgba_premultiplied(30, 27, 75, 140), Stroke::NONE));

                        // Ridge 2 (Mid ridge)
                        let mut pts2 = vec![Pos2::new(ridge_start_x + 30.0_f32, bottom_y)];
                        for i in 0..=steps {
                            let frac = i as f32 / steps as f32;
                            let x = (ridge_start_x + 30.0_f32) + (ridge_end_x - (ridge_start_x + 30.0_f32)) * frac;
                            let wave = ((frac + 0.3_f32) * 3.14159_f32 * 2.0_f32).sin() * 22.0_f32;
                            let y = card_rect.min.y + 132.0_f32 - wave;
                            pts2.push(Pos2::new(x, y));
                        }
                        pts2.push(Pos2::new(ridge_end_x, bottom_y));
                        painter.add(egui::Shape::convex_polygon(pts2, Color32::from_rgba_premultiplied(15, 23, 42, 180), Stroke::NONE));
                    }

                    // Content layout
                    let text_limit = if card_rect.width() > 700.0_f32 { card_rect.width() - 200.0_f32 } else { card_rect.width() };
                    ui.allocate_ui_with_layout(Vec2::new(text_limit, 0.0_f32), egui::Layout::top_down(egui::Align::Min), |ui| {
                        ui.set_max_width(text_limit);

                        ui.horizontal(|ui| {
                            // App Icon Squircle
                            let sq_size = 50.0_f32;
                            let (sq_rect, _) = ui.allocate_exact_size(Vec2::new(sq_size, sq_size), Sense::hover());
                            let brand_blue = Color32::from_rgb(37, 99, 235);
                            ui.painter().rect_filled(sq_rect, 10.0_f32, brand_blue);

                            // Hard drive glyph inside squircle
                            let bc = sq_rect.center();
                            let drive_rect = Rect::from_center_size(bc, Vec2::new(sq_size * 0.58_f32, sq_size * 0.54_f32));
                            ui.painter().rect_stroke(drive_rect, 2.5_f32, Stroke::new(1.6_f32, Color32::WHITE));
                            ui.painter().circle_stroke(Pos2::new(bc.x, bc.y - 1.5_f32), sq_size * 0.16_f32, Stroke::new(1.3_f32, Color32::WHITE));
                            ui.painter().circle_filled(Pos2::new(drive_rect.max.x - 3.5_f32, drive_rect.max.y - 3.5_f32), 1.1_f32, Color32::WHITE);

                            ui.add_space(12.0_f32);

                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(METADATA.app_name)
                                            .size(22.0_f32)
                                            .strong()
                                            .color(if dark_mode { Color32::WHITE } else { Color32::from_rgb(15, 23, 42) }),
                                    );

                                    ui.add_space(8.0_f32);

                                    // Version Badge Pill
                                    let ver_bg = if dark_mode { Color32::from_rgb(30, 58, 138) } else { Color32::from_rgb(224, 231, 255) };
                                    let ver_fg = if dark_mode { Color32::from_rgb(96, 165, 250) } else { Color32::from_rgb(29, 78, 216) };
                                    let (badge_rect, _) = ui.allocate_exact_size(Vec2::new(66.0_f32, 20.0_f32), Sense::hover());
                                    ui.painter().rect_filled(badge_rect, 4.0_f32, ver_bg);
                                    ui.painter().rect_stroke(badge_rect, 4.0_f32, Stroke::new(1.0_f32, ver_fg.gamma_multiply(0.4_f32)));
                                    ui.painter().text(
                                        badge_rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        METADATA.version,
                                        egui::FontId::monospace(10.5_f32),
                                        ver_fg,
                                    );
                                });

                                ui.add_space(2.0_f32);
                                ui.label(
                                    RichText::new(METADATA.subtitle)
                                        .size(13.0_f32)
                                        .color(if dark_mode { Color32::from_rgb(148, 163, 184) } else { Color32::from_rgb(100, 116, 139) }),
                                );
                            });
                        });

                        ui.add_space(12.0_f32);

                        // Descriptions
                        ui.label(
                            RichText::new(METADATA.description_primary)
                                .size(12.5_f32)
                                .color(if dark_mode { Color32::from_rgb(226, 232, 240) } else { Color32::from_rgb(30, 41, 59) }),
                        );
                        ui.add_space(2.0_f32);
                        ui.label(
                            RichText::new(METADATA.description_secondary)
                                .size(12.0_f32)
                                .color(if dark_mode { Color32::from_rgb(148, 163, 184) } else { Color32::from_rgb(100, 116, 139) }),
                        );

                        ui.add_space(14.0_f32);

                        // Action Buttons Row: View on GitHub, Documentation, License (GPL-3.0)
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = Vec2::new(10.0_f32, 6.0_f32);

                            // 1. View on GitHub
                            let gh_url = METADATA.repository_url;
                            let gh_btn = egui::Button::new(
                                RichText::new("  View on GitHub")
                                    .size(12.0_f32)
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(Color32::from_rgb(37, 99, 235))
                            .rounding(5.0_f32)
                            .min_size(Vec2::new(136.0_f32, 30.0_f32));

                            let resp1 = ui.add(gh_btn);
                            if let Some(url) = gh_url {
                                let resp1 = resp1.on_hover_cursor(CursorIcon::PointingHand).on_hover_text(url);
                                if resp1.clicked() {
                                    let _ = open::that(url);
                                }
                            } else {
                                let resp1 = resp1.on_hover_cursor(CursorIcon::NotAllowed).on_hover_text("Not published yet — repository will be made available upon public release");
                                if resp1.clicked() {
                                    toast_text = Some("Repository has not been published yet. Check back soon!".to_string());
                                }
                            }

                            // 2. Documentation
                            let docs_url = METADATA.documentation_url;
                            let doc_btn = egui::Button::new(
                                RichText::new("📖  Documentation")
                                    .size(12.0_f32)
                                    .color(if dark_mode { Color32::from_rgb(226, 232, 240) } else { Color32::from_rgb(30, 41, 59) }),
                            )
                            .fill(if dark_mode { Color32::from_rgb(30, 41, 59) } else { Color32::from_rgb(226, 232, 240) })
                            .stroke(Stroke::new(1.0_f32, if dark_mode { Color32::from_rgb(51, 65, 85) } else { Color32::from_rgb(203, 213, 225) }))
                            .rounding(5.0_f32)
                            .min_size(Vec2::new(124.0_f32, 30.0_f32));

                            let resp2 = ui.add(doc_btn);
                            if let Some(url) = docs_url {
                                let resp2 = resp2.on_hover_cursor(CursorIcon::PointingHand).on_hover_text(url);
                                if resp2.clicked() {
                                    let _ = open::that(url);
                                }
                            } else {
                                let resp2 = resp2.on_hover_cursor(CursorIcon::NotAllowed).on_hover_text("Documentation will be published alongside the public release");
                                if resp2.clicked() {
                                    toast_text = Some("Documentation is bundled in this release. Online docs coming soon.".to_string());
                                }
                            }

                            // 3. License (GPL-3.0)
                            let lic_btn = egui::Button::new(
                                RichText::new("</>  License (GPL-3.0)")
                                    .size(12.0_f32)
                                    .color(if dark_mode { Color32::from_rgb(226, 232, 240) } else { Color32::from_rgb(30, 41, 59) }),
                            )
                            .fill(if dark_mode { Color32::from_rgb(30, 41, 59) } else { Color32::from_rgb(226, 232, 240) })
                            .stroke(Stroke::new(1.0_f32, if dark_mode { Color32::from_rgb(51, 65, 85) } else { Color32::from_rgb(203, 213, 225) }))
                            .rounding(5.0_f32)
                            .min_size(Vec2::new(138.0_f32, 30.0_f32));

                            if ui.add(lic_btn)
                                .on_hover_cursor(CursorIcon::PointingHand)
                                .on_hover_text("View open-source GNU General Public License terms")
                                .clicked()
                            {
                                is_license_open = true;
                            }
                        });
                    });
                });

                let _ = hero_response;

                ui.add_space(20.0_f32);

                // ==========================================
                // 2. CAPABILITY HIGHLIGHTS ROW (Open layout)
                // ==========================================
                let num_caps = if available_w >= 700.0_f32 { 3 } else { 1 };
                let cap_col_w = if num_caps == 3 {
                    ((available_w - 24.0_f32) / 3.0_f32).floor()
                } else {
                    available_w
                };

                let capabilities: [(&str, Color32, &str, &str); 3] = [
                    (
                        "lightning",
                        Color32::from_rgb(168, 85, 247), // #A855F7 Purple
                        "Fast Analysis",
                        "Multi-core scanning with near-instant directory indexing.",
                    ),
                    (
                        "shield",
                        Color32::from_rgb(16, 185, 129), // #10B981 Emerald
                        "Safe by Design",
                        "Non-destructive workflows where supported.",
                    ),
                    (
                        "monitor",
                        Color32::from_rgb(56, 189, 248), // #38BDF8 Sky Blue
                        "Native Linux",
                        "Built in Rust with egui for a smooth, lightweight experience.",
                    ),
                ];

                if num_caps == 3 {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 12.0_f32;
                        for (icon_id, color, title, desc) in capabilities {
                            ui.allocate_ui_with_layout(
                                Vec2::new(cap_col_w, 56.0_f32),
                                egui::Layout::left_to_right(egui::Align::Min),
                                |ui| {
                                    render_capability_summary(ui, icon_id, color, title, desc, cap_col_w, dark_mode);
                                },
                            );
                        }
                    });
                } else {
                    for (icon_id, color, title, desc) in capabilities {
                        render_capability_summary(ui, icon_id, color, title, desc, cap_col_w, dark_mode);
                        ui.add_space(8.0_f32);
                    }
                }

                ui.add_space(20.0_f32);

                // ==========================================
                // 3. DEVELOPER SECTION
                // ==========================================
                ui.label(
                    RichText::new("Developer")
                        .size(13.5_f32)
                        .strong()
                        .color(if dark_mode { Color32::WHITE } else { Color32::from_rgb(15, 23, 42) }),
                );
                ui.add_space(8.0_f32);

                let is_wide = available_w >= 740.0_f32;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(14.0_f32, 0.0_f32);

                    // Avatar Circle with letter "S"
                    let av_size = 44.0_f32;
                    let (av_rect, _) = ui.allocate_exact_size(Vec2::new(av_size, av_size), Sense::hover());
                    ui.painter().circle_filled(av_rect.center(), av_size / 2.0_f32, Color32::from_rgb(37, 99, 235));
                    ui.painter().text(
                        av_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "S",
                        egui::FontId::proportional(19.0_f32),
                        Color32::WHITE,
                    );

                    // Developer Name & Role
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(METADATA.developer_name)
                                .size(13.5_f32)
                                .strong()
                                .color(if dark_mode { Color32::WHITE } else { Color32::from_rgb(15, 23, 42) }),
                        );
                        ui.add_space(1.0_f32);
                        ui.label(
                            RichText::new(METADATA.developer_role)
                                .size(11.5_f32)
                                .color(if dark_mode { Color32::from_rgb(148, 163, 184) } else { Color32::from_rgb(100, 116, 139) }),
                        );
                    });

                    // Divider & Bio (on wider screens)
                    if is_wide {
                        ui.add_space(6.0_f32);
                        let div_x = ui.cursor().min.x;
                        let cy = av_rect.center().y;
                        ui.painter().line_segment(
                            [Pos2::new(div_x, cy - 18.0_f32), Pos2::new(div_x, cy + 18.0_f32)],
                            Stroke::new(1.0_f32, if dark_mode { Color32::from_rgb(30, 41, 59) } else { Color32::from_rgb(226, 232, 240) }),
                        );
                        ui.add_space(14.0_f32);

                        let bio_w = (available_w - 420.0_f32).max(180.0_f32);
                        ui.allocate_ui_with_layout(Vec2::new(bio_w, 0.0_f32), egui::Layout::top_down(egui::Align::Min), |ui| {
                            ui.set_max_width(bio_w);
                            ui.label(
                                RichText::new(METADATA.developer_bio)
                                    .size(11.5_f32)
                                    .color(if dark_mode { Color32::from_rgb(148, 163, 184) } else { Color32::from_rgb(100, 116, 139) }),
                            );
                        });
                    }

                    // Social Actions (Right aligned)
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(8.0_f32, 0.0_f32);

                        // Email
                        render_social_btn(
                            ui,
                            "✉",
                            "Email",
                            METADATA.email_url,
                            dark_mode,
                            &mut toast_text,
                        );

                        // LinkedIn
                        render_social_btn(
                            ui,
                            "in",
                            "LinkedIn",
                            METADATA.linkedin_url,
                            dark_mode,
                            &mut toast_text,
                        );

                        // GitHub
                        render_social_btn(
                            ui,
                            "",
                            "GitHub",
                            METADATA.github_url,
                            dark_mode,
                            &mut toast_text,
                        );
                    });
                });

                if !is_wide {
                    ui.add_space(6.0_f32);
                    ui.label(
                        RichText::new(METADATA.developer_bio)
                            .size(11.5_f32)
                            .color(if dark_mode { Color32::from_rgb(148, 163, 184) } else { Color32::from_rgb(100, 116, 139) }),
                    );
                }

                ui.add_space(20.0_f32);

                // ==========================================
                // 4. APPLICATION INFORMATION (Single 4-col card)
                // ==========================================
                ui.label(
                    RichText::new("Application Information")
                        .size(13.5_f32)
                        .strong()
                        .color(if dark_mode { Color32::WHITE } else { Color32::from_rgb(15, 23, 42) }),
                );
                ui.add_space(8.0_f32);

                let info_bg = if dark_mode { Color32::from_rgb(15, 23, 42) } else { Color32::from_rgb(248, 250, 252) };
                let info_border = if dark_mode { Color32::from_rgb(30, 41, 59) } else { Color32::from_rgb(226, 232, 240) };
                let info_h = 58.0_f32;

                let (card_rect, _) = ui.allocate_exact_size(Vec2::new(available_w, info_h), Sense::hover());
                ui.painter().rect_filled(card_rect, 8.0_f32, info_bg);
                ui.painter().rect_stroke(card_rect, 8.0_f32, Stroke::new(1.0_f32, info_border));

                let build_tgt = METADATA.build_target();
                let columns = [
                    ("Version", METADATA.version),
                    ("Build Target", build_tgt.as_str()),
                    ("Engine", METADATA.engine),
                    ("License", METADATA.license),
                ];

                let col_count = if available_w >= 640.0_f32 { 4 } else { 2 };
                let col_w = (card_rect.width() / col_count as f32).floor();

                for (idx, (label, val)) in columns.iter().enumerate() {
                    let col_left = card_rect.min.x + (idx as f32 * col_w);

                    if idx > 0 && col_count == 4 {
                        // Subtle vertical separator line
                        ui.painter().line_segment(
                            [Pos2::new(col_left, card_rect.min.y + 12.0_f32), Pos2::new(col_left, card_rect.max.y - 12.0_f32)],
                            Stroke::new(1.0_f32, info_border),
                        );
                    }

                    let text_x = col_left + 16.0_f32;
                    ui.painter().text(
                        Pos2::new(text_x, card_rect.min.y + 13.0_f32),
                        egui::Align2::LEFT_TOP,
                        *label,
                        egui::FontId::proportional(11.0_f32),
                        if dark_mode { Color32::from_rgb(148, 163, 184) } else { Color32::from_rgb(100, 116, 139) },
                    );

                    ui.painter().text(
                        Pos2::new(text_x, card_rect.min.y + 30.0_f32),
                        egui::Align2::LEFT_TOP,
                        *val,
                        egui::FontId::proportional(12.5_f32),
                        if dark_mode { Color32::WHITE } else { Color32::from_rgb(15, 23, 42) },
                    );
                }

                ui.add_space(16.0_f32);

                // Inline toast notification banner if present
                if let Some(msg) = &toast_text {
                    let (toast_rect, _) = ui.allocate_exact_size(Vec2::new(available_w, 32.0_f32), Sense::hover());
                    ui.painter().rect_filled(toast_rect, 6.0_f32, Color32::from_rgb(30, 41, 59));
                    ui.painter().rect_stroke(toast_rect, 6.0_f32, Stroke::new(1.0_f32, Color32::from_rgb(37, 99, 235)));
                    ui.painter().text(
                        toast_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        msg,
                        egui::FontId::proportional(11.5_f32),
                        Color32::from_rgb(224, 231, 255),
                    );
                    ui.add_space(8.0_f32);
                }
            });
        });

    // Update state in egui memory
    ui.ctx().data_mut(|d| {
        d.insert_temp(modal_id, is_license_open);
        d.insert_temp(toast_id, toast_text);
    });

    // ==========================================
    // LICENSE MODAL DIALOG
    // ==========================================
    if is_license_open {
        Window::new("DiskLens License — GNU General Public License v3.0")
            .collapsible(false)
            .resizable(false)
            .min_width(520.0_f32)
            .anchor(egui::Align2::CENTER_CENTER, [0.0_f32, 0.0_f32])
            .show(ui.ctx(), |ui| {
                ui.add_space(6.0_f32);
                ui.label(
                    RichText::new(
                        "DiskLens is free and open-source software licensed under GPL-3.0.",
                    )
                    .size(12.5_f32)
                    .strong()
                    .color(t.text),
                );
                ui.add_space(8.0_f32);

                egui::Frame::none()
                    .fill(if dark_mode {
                        Color32::from_rgb(15, 23, 42)
                    } else {
                        Color32::from_rgb(241, 245, 249)
                    })
                    .rounding(6.0_f32)
                    .stroke(Stroke::new(1.0_f32, t.border))
                    .inner_margin(Margin::same(12.0_f32))
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(260.0_f32)
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(METADATA.license_text())
                                        .size(11.0_f32)
                                        .monospace()
                                        .color(t.text_secondary),
                                );
                            });
                    });

                ui.add_space(10.0_f32);
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(RichText::new("Close").size(12.0_f32)).clicked() {
                            ui.ctx().data_mut(|d| d.insert_temp(modal_id, false));
                        }
                    });
                });
                ui.add_space(4.0_f32);
            });
    }
}

/// Helper function to render a capability summary column in the open layout.
fn render_capability_summary(
    ui: &mut Ui,
    icon_type: &str,
    color: Color32,
    title: &str,
    desc: &str,
    width: f32,
    dark_mode: bool,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(10.0_f32, 0.0_f32);

        // Vector icon glyph
        let (icon_rect, _) = ui.allocate_exact_size(Vec2::new(28.0_f32, 28.0_f32), Sense::hover());
        let ic = icon_rect.center();

        match icon_type {
            "lightning" => {
                // Purple lightning bolt
                let stroke = Stroke::new(1.6_f32, color);
                ui.painter().line_segment(
                    [
                        Pos2::new(ic.x + 2.0_f32, ic.y - 8.0_f32),
                        Pos2::new(ic.x - 4.0_f32, ic.y + 0.5_f32),
                    ],
                    stroke,
                );
                ui.painter().line_segment(
                    [
                        Pos2::new(ic.x - 4.0_f32, ic.y + 0.5_f32),
                        Pos2::new(ic.x + 1.0_f32, ic.y + 0.5_f32),
                    ],
                    stroke,
                );
                ui.painter().line_segment(
                    [
                        Pos2::new(ic.x + 1.0_f32, ic.y + 0.5_f32),
                        Pos2::new(ic.x - 2.0_f32, ic.y + 8.0_f32),
                    ],
                    stroke,
                );
            }
            "shield" => {
                // Emerald shield check
                let stroke = Stroke::new(1.6_f32, color);
                // Shield outline
                ui.painter().line_segment(
                    [
                        Pos2::new(ic.x - 6.0_f32, ic.y - 6.0_f32),
                        Pos2::new(ic.x + 6.0_f32, ic.y - 6.0_f32),
                    ],
                    stroke,
                );
                ui.painter().line_segment(
                    [
                        Pos2::new(ic.x + 6.0_f32, ic.y - 6.0_f32),
                        Pos2::new(ic.x + 6.0_f32, ic.y + 1.0_f32),
                    ],
                    stroke,
                );
                ui.painter().line_segment(
                    [
                        Pos2::new(ic.x + 6.0_f32, ic.y + 1.0_f32),
                        Pos2::new(ic.x, ic.y + 7.5_f32),
                    ],
                    stroke,
                );
                ui.painter().line_segment(
                    [
                        Pos2::new(ic.x, ic.y + 7.5_f32),
                        Pos2::new(ic.x - 6.0_f32, ic.y + 1.0_f32),
                    ],
                    stroke,
                );
                ui.painter().line_segment(
                    [
                        Pos2::new(ic.x - 6.0_f32, ic.y + 1.0_f32),
                        Pos2::new(ic.x - 6.0_f32, ic.y - 6.0_f32),
                    ],
                    stroke,
                );
                // Checkmark inside
                ui.painter().line_segment(
                    [
                        Pos2::new(ic.x - 2.5_f32, ic.y + 0.5_f32),
                        Pos2::new(ic.x - 0.5_f32, ic.y + 2.5_f32),
                    ],
                    stroke,
                );
                ui.painter().line_segment(
                    [
                        Pos2::new(ic.x - 0.5_f32, ic.y + 2.5_f32),
                        Pos2::new(ic.x + 3.0_f32, ic.y - 1.5_f32),
                    ],
                    stroke,
                );
            }
            "monitor" => {
                // Desktop monitor
                let stroke = Stroke::new(1.6_f32, color);
                let screen = Rect::from_center_size(
                    Pos2::new(ic.x, ic.y - 2.0_f32),
                    Vec2::new(15.0_f32, 10.0_f32),
                );
                ui.painter().rect_stroke(screen, 1.5_f32, stroke);
                // Stand stem and base
                ui.painter().line_segment(
                    [
                        Pos2::new(ic.x, screen.max.y),
                        Pos2::new(ic.x, screen.max.y + 3.5_f32),
                    ],
                    stroke,
                );
                ui.painter().line_segment(
                    [
                        Pos2::new(ic.x - 4.5_f32, screen.max.y + 3.5_f32),
                        Pos2::new(ic.x + 4.5_f32, screen.max.y + 3.5_f32),
                    ],
                    stroke,
                );
            }
            _ => {}
        }

        let text_w = (width - 42.0_f32).max(120.0_f32);
        ui.allocate_ui_with_layout(
            Vec2::new(text_w, 0.0_f32),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.set_max_width(text_w);
                ui.label(
                    RichText::new(title)
                        .size(13.0_f32)
                        .strong()
                        .color(if dark_mode {
                            Color32::WHITE
                        } else {
                            Color32::from_rgb(15, 23, 42)
                        }),
                );
                ui.add_space(2.0_f32);
                ui.label(RichText::new(desc).size(11.5_f32).color(if dark_mode {
                    Color32::from_rgb(148, 163, 184)
                } else {
                    Color32::from_rgb(100, 116, 139)
                }));
            },
        );
    });
}

/// Helper function to render developer social action button.
fn render_social_btn(
    ui: &mut Ui,
    icon: &str,
    platform: &str,
    url_opt: Option<&str>,
    dark_mode: bool,
    toast_text: &mut Option<String>,
) {
    let btn_bg = if dark_mode {
        Color32::from_rgb(30, 41, 59)
    } else {
        Color32::from_rgb(241, 245, 249)
    };
    let btn_border = if dark_mode {
        Color32::from_rgb(51, 65, 85)
    } else {
        Color32::from_rgb(203, 213, 225)
    };

    let btn = egui::Button::new(RichText::new(icon).size(12.0_f32).color(if dark_mode {
        Color32::from_rgb(226, 232, 240)
    } else {
        Color32::from_rgb(71, 85, 105)
    }))
    .fill(btn_bg)
    .stroke(Stroke::new(1.0_f32, btn_border))
    .rounding(5.0_f32)
    .min_size(Vec2::new(32.0_f32, 30.0_f32));

    let resp = ui.add(btn);

    if let Some(url) = url_opt {
        let resp = resp
            .on_hover_cursor(CursorIcon::PointingHand)
            .on_hover_text(format!("Open {} profile", platform));
        if resp.clicked() {
            let _ = open::that(url);
        }
    } else {
        let tooltip = format!("{} profile will be linked here once published", platform);
        let resp = resp
            .on_hover_cursor(CursorIcon::NotAllowed)
            .on_hover_text(tooltip);
        if resp.clicked() {
            *toast_text = Some(format!(
                "Sayantan Mazumdar's {} profile is not configured yet.",
                platform
            ));
        }
    }
}
