use eframe::egui::{self, Color32, Margin, Rounding, Stroke, Style, Vec2, Visuals};

pub struct AppTheme;

/// Tokyo Night & Modern Slate Semantic Tokens
#[derive(Clone, Copy)]
pub struct ThemeTokens {
    pub bg_base: Color32,
    pub bg_surface: Color32,
    pub bg_elevated: Color32,
    pub bg_container: Color32,
    pub bg_input: Color32,
    pub border: Color32,
    pub border_subtle: Color32,
    pub text: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub accent: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub danger: Color32,
    pub purple: Color32,
    pub cyan: Color32,
}

impl AppTheme {
    pub fn setup_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        let font_paths = [
            "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
            "/usr/share/fonts/truetype/ubuntu/UbuntuSans[wdth,wght].ttf",
            "/usr/share/fonts/truetype/ubuntu/Ubuntu[wdth,wght].ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        ];

        for path in font_paths {
            if let Ok(font_data) = std::fs::read(path) {
                fonts.font_data.insert(
                    "system_sans".to_owned(),
                    std::sync::Arc::new(egui::FontData::from_owned(font_data)),
                );
                if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                    family.insert(0, "system_sans".to_owned());
                }
                break;
            }
        }

        ctx.set_fonts(fonts);
    }

    pub fn tokens(dark_mode: bool) -> ThemeTokens {
        if dark_mode {
            // Tokyo Night Palette
            ThemeTokens {
                bg_base: Color32::from_rgb(26, 27, 38),       // #1A1B26
                bg_surface: Color32::from_rgb(20, 19, 20),    // #141314
                bg_elevated: Color32::from_rgb(36, 40, 59),   // #24283B
                bg_container: Color32::from_rgb(32, 31, 32),  // #201F20
                bg_input: Color32::from_rgb(20, 20, 26),      // #14141A
                border: Color32::from_rgb(41, 46, 66),        // #292E42
                border_subtle: Color32::from_rgb(31, 35, 53), // #1F2335
                text: Color32::from_rgb(192, 202, 245),       // #C0CAF5
                text_secondary: Color32::from_rgb(154, 165, 206), // #9AA5CE
                text_muted: Color32::from_rgb(86, 95, 137),   // #565F89
                accent: Color32::from_rgb(122, 162, 247),     // #7AA2F7
                success: Color32::from_rgb(158, 206, 106),    // #9ECE6A
                warning: Color32::from_rgb(224, 175, 104),    // #E0AF68
                danger: Color32::from_rgb(247, 118, 142),     // #F7768E
                purple: Color32::from_rgb(187, 154, 247),     // #BB9AF7
                cyan: Color32::from_rgb(115, 218, 202),       // #73DACA
            }
        } else {
            // Modern Slate Light Palette
            ThemeTokens {
                bg_base: Color32::from_rgb(248, 250, 252),    // #F8FAFC
                bg_surface: Color32::from_rgb(255, 255, 255), // #FFFFFF
                bg_elevated: Color32::from_rgb(255, 255, 255),// #FFFFFF
                bg_container: Color32::from_rgb(241, 245, 249),// #F1F5F9
                bg_input: Color32::from_rgb(248, 250, 252),   // #F8FAFC
                border: Color32::from_rgb(226, 232, 240),     // #E2E8F0
                border_subtle: Color32::from_rgb(203, 213, 225),// #CBD5E1
                text: Color32::from_rgb(15, 23, 42),          // #0F172A
                text_secondary: Color32::from_rgb(71, 85, 105), // #475569
                text_muted: Color32::from_rgb(100, 116, 139), // #64748B
                accent: Color32::from_rgb(37, 99, 235),       // #2563EB
                success: Color32::from_rgb(22, 163, 74),      // #16A34A
                warning: Color32::from_rgb(217, 119, 6),      // #D97706
                danger: Color32::from_rgb(220, 38, 38),       // #DC2626
                purple: Color32::from_rgb(147, 51, 234),      // #9333EA
                cyan: Color32::from_rgb(13, 148, 136),        // #0D9488
            }
        }
    }

    pub fn apply_dark(ctx: &egui::Context) {
        let t = Self::tokens(true);
        let mut visuals = Visuals::dark();

        visuals.dark_mode = true;
        visuals.override_text_color = Some(t.text);
        visuals.window_fill = t.bg_base;
        visuals.panel_fill = t.bg_surface;
        visuals.faint_bg_color = t.bg_elevated;
        visuals.extreme_bg_color = t.bg_input;

        visuals.widgets.noninteractive.bg_fill = t.bg_elevated;
        visuals.widgets.noninteractive.rounding = Rounding::same(6.0_f32);
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, t.border);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, t.text);

        visuals.widgets.inactive.bg_fill = t.bg_elevated;
        visuals.widgets.inactive.rounding = Rounding::same(6.0_f32);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, t.border);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, t.text);

        visuals.widgets.hovered.bg_fill = Color32::from_rgb(62, 69, 101); // #3E4565
        visuals.widgets.hovered.rounding = Rounding::same(6.0_f32);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, t.accent);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);

        visuals.widgets.active.bg_fill = Color32::from_rgb(65, 72, 104); // #414868
        visuals.widgets.active.rounding = Rounding::same(6.0_f32);
        visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, t.accent);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);

        visuals.selection.bg_fill = Color32::from_rgba_premultiplied(122, 162, 247, 60);
        visuals.selection.stroke = Stroke::new(1.0_f32, t.accent);
        visuals.window_stroke = Stroke::new(1.0_f32, t.border);

        let mut style = Style::default();
        style.visuals = visuals;
        style.spacing.item_spacing = Vec2::new(8.0, 6.0);
        style.spacing.button_padding = Vec2::new(10.0, 5.0);
        style.spacing.window_margin = Margin::same(12.0);

        ctx.set_style(style);
    }

    pub fn apply_light(ctx: &egui::Context) {
        let t = Self::tokens(false);
        let mut visuals = Visuals::light();

        visuals.dark_mode = false;
        visuals.override_text_color = Some(t.text);
        visuals.window_fill = t.bg_base;
        visuals.panel_fill = t.bg_surface;
        visuals.faint_bg_color = t.bg_elevated;
        visuals.extreme_bg_color = t.bg_container;

        visuals.widgets.noninteractive.bg_fill = t.bg_elevated;
        visuals.widgets.noninteractive.rounding = Rounding::same(6.0_f32);
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, t.border);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, t.text);

        visuals.widgets.inactive.bg_fill = t.bg_container;
        visuals.widgets.inactive.rounding = Rounding::same(6.0_f32);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, t.border);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, t.text);

        visuals.widgets.hovered.bg_fill = Color32::from_rgb(226, 232, 240);
        visuals.widgets.hovered.rounding = Rounding::same(6.0_f32);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, t.accent);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, Color32::BLACK);

        visuals.widgets.active.bg_fill = t.accent;
        visuals.widgets.active.rounding = Rounding::same(6.0_f32);
        visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, t.accent);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);

        visuals.selection.bg_fill = Color32::from_rgba_premultiplied(37, 99, 235, 40);
        visuals.selection.stroke = Stroke::new(1.0_f32, t.accent);
        visuals.window_stroke = Stroke::new(1.0_f32, t.border);

        let mut style = Style::default();
        style.visuals = visuals;
        style.spacing.item_spacing = Vec2::new(8.0, 6.0);
        style.spacing.button_padding = Vec2::new(10.0, 5.0);
        style.spacing.window_margin = Margin::same(12.0);

        ctx.set_style(style);
    }

    pub fn accent_color() -> Color32 {
        Color32::from_rgb(122, 162, 247) // #7AA2F7
    }

    pub fn text_muted(dark_mode: bool) -> Color32 {
        Self::tokens(dark_mode).text_muted
    }

    pub fn text_secondary(dark_mode: bool) -> Color32 {
        Self::tokens(dark_mode).text_secondary
    }

    pub fn text_subtle(dark_mode: bool) -> Color32 {
        if dark_mode {
            Color32::from_rgb(86, 95, 137)
        } else {
            Color32::from_rgb(148, 163, 184)
        }
    }

    pub fn success_color() -> Color32 {
        Color32::from_rgb(158, 206, 106) // #9ECE6A
    }

    pub fn warning_color() -> Color32 {
        Color32::from_rgb(224, 175, 104) // #E0AF68
    }

    pub fn danger_color() -> Color32 {
        Color32::from_rgb(247, 118, 142) // #F7768E
    }

    pub fn purple_color() -> Color32 {
        Color32::from_rgb(187, 154, 247) // #BB9AF7
    }

    pub fn cyan_color() -> Color32 {
        Color32::from_rgb(115, 218, 202) // #73DACA
    }
}

pub fn truncate_middle(text: &str, max_len: usize) -> String {
    let char_count = text.chars().count();
    if char_count <= max_len {
        text.to_string()
    } else {
        let half = (max_len.saturating_sub(3)) / 2;
        let prefix: String = text.chars().take(half).collect();
        let suffix: String = text
            .chars()
            .rev()
            .take(half)
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        format!("{}...{}", prefix, suffix)
    }
}
