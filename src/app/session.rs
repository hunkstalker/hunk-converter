use std::path::Path;

use eframe::egui;
use image::{ImageFormat, RgbaImage};

use crate::codec;
use crate::domain::{AppError, ClipboardError, OutputFormat};
use crate::i18n;

use super::HunkApp;
use super::state::{SourceImage, SourceInput};
use super::ui::widgets::{resized_for_ui, upload_texture};
use super::{MIN_ZOOM, UI_PREVIEW_MAX};

impl HunkApp {
    pub(crate) fn set_source(&mut self, input: SourceInput, ctx: &egui::Context) {
        let (width, height) = input.pixels.dimensions();
        let preview = resized_for_ui(&input.pixels, UI_PREVIEW_MAX);
        let texture = upload_texture(ctx, &format!("source-{}", self.revision), &preview);
        self.source = Some(SourceImage {
            name: input.name,
            stem: input.stem,
            format: input.format,
            file_bytes: input.file_bytes,
            width,
            height,
            pixels: std::sync::Arc::new(input.pixels),
            texture,
        });
        self.result = None;
        self.eyedropper_active = false;
        self.sampled_color = None;
        self.sampled_position = None;
        self.sample_pending_history = false;
        self.split = 0.5;
        self.zoom = MIN_ZOOM;
        self.pan = egui::Vec2::ZERO;
        self.screenshot_at = None;
        self.screenshot_requested = false;
        self.request_encode(ctx, None);
    }

    pub(crate) fn load_path(&mut self, path: &Path, ctx: &egui::Context) {
        match codec::load_image_file(path) {
            Ok(loaded) => {
                let name = path
                    .file_name()
                    .map(|value| value.to_string_lossy().into_owned())
                    .unwrap_or_else(|| i18n::fallback_name(self.language).to_owned());
                let stem = path
                    .file_stem()
                    .map(|value| value.to_string_lossy().into_owned())
                    .unwrap_or_else(|| i18n::fallback_name(self.language).to_owned());
                self.set_source(
                    SourceInput {
                        pixels: loaded.pixels,
                        name,
                        stem,
                        format: loaded.format,
                        file_bytes: loaded.file_bytes,
                    },
                    ctx,
                );
            }
            Err(error) => self.toast_error(&AppError::Load(error), 4),
        }
    }

    pub(crate) fn load_demo(&mut self, ctx: &egui::Context) {
        let pixels = crate::demo::landscape(1600, 1000);
        let mut encoded = Vec::new();
        let _ = image::DynamicImage::ImageRgba8(pixels.clone())
            .write_to(&mut std::io::Cursor::new(&mut encoded), ImageFormat::Png);
        self.set_source(
            SourceInput {
                pixels,
                name: "demo.png".to_owned(),
                stem: "demo".to_owned(),
                format: "PNG".to_owned(),
                file_bytes: encoded.len() as u64,
            },
            ctx,
        );
    }

    pub(crate) fn paste_clipboard_image(&mut self, ctx: &egui::Context) {
        let mut clipboard = match arboard::Clipboard::new() {
            Ok(clipboard) => clipboard,
            Err(error) => {
                self.toast_error(
                    &AppError::Clipboard(ClipboardError::Open {
                        source: Box::new(error),
                    }),
                    4,
                );
                ctx.request_repaint();
                return;
            }
        };
        let image = match clipboard.get_image() {
            Ok(image) => image,
            Err(arboard::Error::ContentNotAvailable) => return,
            Err(error) => {
                self.toast_error(
                    &AppError::Clipboard(ClipboardError::Read {
                        source: Box::new(error),
                    }),
                    4,
                );
                ctx.request_repaint();
                return;
            }
        };

        let width = image.width as u32;
        let height = image.height as u32;
        let expected = u64::from(width) * u64::from(height) * 4;
        if width == 0
            || height == 0
            || image.bytes.len() as u64 != expected
            || width > codec::MAX_SOURCE_SIDE
            || height > codec::MAX_SOURCE_SIDE
            || u64::from(width) * u64::from(height) > u64::from(codec::MAX_SOURCE_PIXELS)
        {
            self.toast_error(&AppError::Clipboard(ClipboardError::InvalidImage), 4);
            ctx.request_repaint();
            return;
        }

        let Some(pixels) = RgbaImage::from_raw(width, height, image.bytes.into_owned()) else {
            self.toast_error(&AppError::Clipboard(ClipboardError::InvalidImage), 4);
            return;
        };
        let file_bytes = super::ui::widgets::clipboard_png_size(&pixels);
        self.set_source(
            SourceInput {
                pixels,
                name: i18n::clipboard_name(self.language).to_owned(),
                stem: i18n::clipboard_name(self.language)
                    .trim_end_matches(".png")
                    .to_owned(),
                format: "PNG".to_owned(),
                file_bytes,
            },
            ctx,
        );
        self.toast(self.t(crate::i18n::K::ClipboardPasted).to_owned(), false, 3);
        ctx.request_repaint();
    }

    pub(crate) fn open_file_dialog(&mut self, ctx: &egui::Context) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(
                self.t(crate::i18n::K::ImagesFilter),
                &["png", "jpg", "jpeg", "webp", "bmp", "tga", "svg"],
            )
            .pick_file()
        {
            self.load_path(&path, ctx);
        }
    }

    pub(crate) fn reset(&mut self, ctx: &egui::Context) {
        self.source = None;
        self.result = None;
        self.format = OutputFormat::Webp;
        self.quality = 82;
        self.ico_selected = [true; 6];
        self.split = 0.5;
        self.zoom = MIN_ZOOM;
        self.pan = egui::Vec2::ZERO;
        self.eyedropper_active = false;
        self.sampled_color = None;
        self.sampled_position = None;
        self.sample_pending_history = false;
        self.pending_since = None;
        self.revision = self.revision.wrapping_add(1);
        self.screenshot_at = None;
        self.screenshot_requested = false;
        self.toast = None;
        ctx.request_repaint();
    }

    pub(crate) fn set_format(&mut self, format: OutputFormat) -> bool {
        if self.format == format {
            return false;
        }
        self.format = format;
        if format == OutputFormat::Ico {
            self.eyedropper_active = false;
        }
        self.result = None;
        true
    }
}
