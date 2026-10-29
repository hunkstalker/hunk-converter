use std::sync::Arc;

use egui::{
    Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Frame, Margin, Shadow,
    Stroke, TextStyle,
};

pub const BG: Color32 = Color32::from_rgb(10, 13, 16);
pub const BG_SOFT: Color32 = Color32::from_rgb(13, 17, 22);
pub const SURFACE: Color32 = Color32::from_rgb(17, 22, 28);
pub const SURFACE_2: Color32 = Color32::from_rgb(23, 30, 37);
pub const SURFACE_3: Color32 = Color32::from_rgb(30, 39, 48);
pub const BORDER: Color32 = Color32::from_rgb(39, 49, 59);
pub const BORDER_STRONG: Color32 = Color32::from_rgb(55, 68, 82);
pub const TEXT: Color32 = Color32::from_rgb(241, 245, 242);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(146, 158, 170);
pub const TEXT_FAINT: Color32 = Color32::from_rgb(103, 115, 127);
pub const ACCENT: Color32 = Color32::from_rgb(200, 244, 109);
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(216, 255, 144);
pub const ACCENT_PRESSED: Color32 = Color32::from_rgb(180, 226, 87);
pub const ACCENT_INK: Color32 = Color32::from_rgb(17, 23, 11);
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(27, 41, 24);
pub const WARNING: Color32 = Color32::from_rgb(255, 190, 122);
pub const DANGER: Color32 = Color32::from_rgb(255, 126, 126);
pub const CHECKER_A: Color32 = Color32::from_rgb(32, 40, 48);
pub const CHECKER_B: Color32 = Color32::from_rgb(39, 50, 60);

const MANROPE: &[u8] = include_bytes!("../assets/fonts/Manrope.ttf");

pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "manrope".to_owned(),
        Arc::new(FontData::from_static(MANROPE)),
    );
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "manrope".to_owned());
    ctx.set_fonts(fonts);

    ctx.set_theme(egui::Theme::Dark);
    let mut style = (*ctx.global_style()).clone();
    style
        .text_styles
        .insert(TextStyle::Body, FontId::new(13.5, FontFamily::Proportional));
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(13.5, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(11.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(17.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Monospace,
        FontId::new(11.0, FontFamily::Monospace),
    );
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(14.0, 8.0);
    style.spacing.slider_width = 230.0;
    style.spacing.interact_size.y = 32.0;
    style.spacing.scroll = egui::style::ScrollStyle::solid();
    style.spacing.scroll.bar_width = 8.0;

    let visuals = &mut style.visuals;
    *visuals = egui::Visuals::dark();
    visuals.dark_mode = true;
    visuals.panel_fill = BG;
    visuals.window_fill = SURFACE;
    visuals.extreme_bg_color = BG_SOFT;
    visuals.faint_bg_color = SURFACE_2;
    visuals.window_corner_radius = CornerRadius::same(16);
    visuals.menu_corner_radius = CornerRadius::same(10);
    visuals.window_stroke = Stroke::new(1.0, BORDER);
    visuals.window_shadow = Shadow {
        offset: [0, 14],
        blur: 30,
        spread: 0,
        color: Color32::from_black_alpha(110),
    };
    visuals.popup_shadow = Shadow {
        offset: [0, 8],
        blur: 18,
        spread: 0,
        color: Color32::from_black_alpha(90),
    };
    visuals.override_text_color = Some(TEXT);
    visuals.weak_text_alpha = 0.82;
    visuals.selection.bg_fill = ACCENT;
    visuals.selection.stroke = Stroke::new(1.0, ACCENT_INK);
    visuals.hyperlink_color = ACCENT;
    visuals.slider_trailing_fill = true;
    visuals.error_fg_color = DANGER;
    visuals.warn_fg_color = WARNING;

    let widgets = &mut visuals.widgets;
    widgets.noninteractive.bg_fill = SURFACE;
    widgets.noninteractive.weak_bg_fill = SURFACE;
    widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
    widgets.noninteractive.corner_radius = CornerRadius::same(10);

    widgets.inactive.bg_fill = SURFACE_3;
    widgets.inactive.weak_bg_fill = SURFACE_2;
    widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    widgets.inactive.corner_radius = CornerRadius::same(10);
    widgets.inactive.expansion = 0.0;

    widgets.hovered.bg_fill = SURFACE_3;
    widgets.hovered.weak_bg_fill = SURFACE_3;
    widgets.hovered.bg_stroke = Stroke::new(1.0, BORDER_STRONG);
    widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT);
    widgets.hovered.corner_radius = CornerRadius::same(10);
    widgets.hovered.expansion = 0.0;

    widgets.active.bg_fill = SURFACE_2;
    widgets.active.weak_bg_fill = SURFACE_2;
    widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
    widgets.active.fg_stroke = Stroke::new(1.0, TEXT);
    widgets.active.corner_radius = CornerRadius::same(10);
    widgets.active.expansion = 0.0;

    widgets.open.bg_fill = SURFACE_3;
    widgets.open.weak_bg_fill = SURFACE_3;
    widgets.open.bg_stroke = Stroke::new(1.0, BORDER_STRONG);
    widgets.open.fg_stroke = Stroke::new(1.0, TEXT);
    widgets.open.corner_radius = CornerRadius::same(10);

    ctx.set_style_of(egui::Theme::Dark, style);
}

pub fn sidebar_frame() -> Frame {
    Frame::new()
        .fill(SURFACE)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius {
            nw: 18,
            ne: 18,
            sw: 0,
            se: 0,
        })
        .inner_margin(Margin::symmetric(20, 14))
}

pub fn card_frame() -> Frame {
    Frame::new()
        .fill(SURFACE)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(20))
        .inner_margin(Margin::same(18))
}

pub fn soft_frame() -> Frame {
    Frame::new()
        .fill(Color32::from_rgb(16, 22, 28))
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::symmetric(12, 10))
}

pub fn muted(size: f32) -> egui::RichText {
    egui::RichText::new("").size(size).color(TEXT_MUTED)
}

pub fn eyebrow(text: &str) -> egui::RichText {
    egui::RichText::new(text)
        .size(9.5)
        .color(TEXT_FAINT)
        .strong()
}

pub fn section_title(text: &str) -> egui::RichText {
    egui::RichText::new(text).size(17.0).color(TEXT)
}

pub fn bullet_separator(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(1.0, 14.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, BORDER);
}
