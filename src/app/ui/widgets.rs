use std::io::Cursor;

use eframe::egui::{
    self, Align2, Color32, CornerRadius, CursorIcon, FontId, Rect, Sense, TextureHandle,
    TextureOptions, Vec2, vec2,
};
use image::{ImageFormat, RgbaImage, imageops};

use crate::theme as T;

pub(crate) const HISTORY_COLUMNS: usize = 7;
pub(crate) const SWATCH_SIZE: f32 = 32.0;
pub(crate) const SWATCH_GAP: f32 = 4.0;

pub(crate) fn small_text_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let font = FontId::proportional(10.5);
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font, T::TEXT_MUTED);
    let size = vec2(galley.size().x + 18.0, 26.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let hovered = response.hovered();
    if hovered {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(8), T::SURFACE_3);
    }
    ui.painter().galley(
        rect.center() - galley.size() * 0.5,
        galley,
        if hovered { T::TEXT } else { T::TEXT_MUTED },
    );
    if hovered {
        response.on_hover_cursor(CursorIcon::PointingHand)
    } else {
        response
    }
}

pub(crate) fn hline(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, T::BORDER);
}

pub(crate) fn clipboard_png_size(image: &RgbaImage) -> u64 {
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(Cursor::new(&mut bytes), image.width(), image.height());
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::Fast);
    if let Ok(mut writer) = encoder.write_header() {
        let _ = writer.write_image_data(image.as_raw());
    }
    bytes.len() as u64
}

pub(crate) fn upload_texture(ctx: &egui::Context, name: &str, image: &RgbaImage) -> TextureHandle {
    let color_image = egui::ColorImage::from_rgba_unmultiplied(
        [image.width() as usize, image.height() as usize],
        image.as_raw(),
    );
    ctx.load_texture(name.to_owned(), color_image, TextureOptions::LINEAR)
}

pub(crate) fn load_texture_from_png(
    ctx: &egui::Context,
    name: &str,
    bytes: &[u8],
) -> Option<TextureHandle> {
    let image = image::load_from_memory_with_format(bytes, ImageFormat::Png)
        .ok()?
        .to_rgba8();
    Some(upload_texture(ctx, name, &image))
}

pub(crate) fn resized_for_ui(image: &RgbaImage, max_side: u32) -> RgbaImage {
    let longest = image.width().max(image.height());
    if longest <= max_side {
        return image.clone();
    }
    let scale = max_side as f32 / longest as f32;
    imageops::resize(
        image,
        ((image.width() as f32 * scale).round() as u32).max(1),
        ((image.height() as f32 * scale).round() as u32).max(1),
        imageops::FilterType::Triangle,
    )
}

pub(crate) fn fit_rect(image_size: Vec2, area: Rect) -> Rect {
    if image_size.x <= 0.0 || image_size.y <= 0.0 {
        return area;
    }
    let scale = (area.width() / image_size.x).min(area.height() / image_size.y);
    Rect::from_center_size(area.center(), image_size * scale)
}

pub(crate) fn paint_checker(painter: &egui::Painter, rect: Rect) {
    let cell = 14.0;
    let clipped = painter.with_clip_rect(rect);
    let columns = (rect.width() / cell).ceil() as usize + 1;
    let rows = (rect.height() / cell).ceil() as usize + 1;
    for row in 0..rows {
        for column in 0..columns {
            let min = rect.min + vec2(column as f32 * cell, row as f32 * cell);
            let color = if (row + column) % 2 == 0 {
                T::CHECKER_A
            } else {
                T::CHECKER_B
            };
            clipped.rect_filled(Rect::from_min_size(min, vec2(cell, cell)), 0.0, color);
        }
    }
}

pub(crate) struct Badge<'a> {
    pub anchor: egui::Pos2,
    pub align: Align2,
    pub text: &'a str,
    pub background: Color32,
    pub foreground: Color32,
}

pub(crate) fn paint_badge(painter: &egui::Painter, badge: Badge<'_>) {
    let Badge {
        anchor,
        align,
        text,
        background,
        foreground,
    } = badge;
    let galley = painter.layout_no_wrap(text.to_owned(), FontId::proportional(9.0), foreground);
    let size = galley.size() + vec2(16.0, 9.0);
    let rect = align.anchor_size(anchor, size);
    painter.rect_filled(rect, CornerRadius::same(7), background);
    painter.galley(rect.min + vec2(8.0, 4.5), galley, foreground);
}
