use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::{Duration, Instant};

use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, CursorIcon, FontId, Frame, Id, Layout, Margin,
    Rect, RichText, Sense, Stroke, StrokeKind, TextureHandle, TextureOptions, Vec2,
    ViewportCommand, pos2, vec2,
};
use image::{ImageFormat, RgbaImage, imageops};

use crate::config::{self, HISTORY_LIMIT, HistoryEntry};
use crate::core::codec::{self, EncodeOptions, EncodedOutput, ICO_SIZES, OutputFormat};
use crate::core::color;
use crate::i18n::{self, K, Lang};
use crate::theme as T;

const ENCODE_DELAY_MS: u64 = 160;
const UI_PREVIEW_MAX: u32 = 2048;
const MIN_ZOOM: f32 = 1.0;
const MAX_ZOOM: f32 = 8.0;
const HISTORY_COLUMNS: usize = 7;
const SWATCH_SIZE: f32 = 32.0;
const SWATCH_GAP: f32 = 4.0;

pub struct HunkApp {
    source: Option<Source>,
    result: Option<RenderedResult>,
    brand_icon: Option<TextureHandle>,
    language: Lang,
    history: Vec<HistoryEntry>,
    maximized: bool,
    native_maximized_seen: bool,
    sample_pending_history: bool,
    color_panel_height: f32,
    format: OutputFormat,
    quality: u8,
    ico_selected: [bool; 6],
    eyedropper_active: bool,
    sampled_color: Option<[u8; 4]>,
    sampled_position: Option<(u32, u32)>,
    split: f32,
    zoom: f32,
    pan: Vec2,
    revision: u64,
    pending_since: Option<Instant>,
    worker: Option<Receiver<JobMessage>>,
    toast: Option<Toast>,
    screenshot_path: Option<PathBuf>,
    screenshot_at: Option<Instant>,
    screenshot_requested: bool,
    screenshot_delay: Duration,
}

struct Source {
    name: String,
    stem: String,
    format: String,
    file_bytes: u64,
    width: u32,
    height: u32,
    pixels: Arc<RgbaImage>,
    texture: TextureHandle,
}

struct RenderedResult {
    texture: TextureHandle,
    bytes: Vec<u8>,
    size: u64,
    width: u32,
    height: u32,
    frame_count: usize,
    format: OutputFormat,
}

struct JobMessage {
    revision: u64,
    result: Result<EncodedOutput, String>,
    elapsed: Duration,
}

struct Toast {
    message: String,
    error: bool,
    until: Instant,
}

impl HunkApp {
    pub fn new(cc: &eframe::CreationContext<'_>, settings: config::Settings) -> Self {
        T::install(&cc.egui_ctx);

        let brand_icon = load_texture_from_png(
            &cc.egui_ctx,
            "brand-icon",
            include_bytes!("../assets/branding/icon_256.png"),
        );
        let screenshot_path = std::env::var_os("HUNK_SCREENSHOT").map(PathBuf::from);
        let screenshot_delay = std::env::var("HUNK_SCREENSHOT_DELAY_MS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .map(Duration::from_millis)
            .unwrap_or(Duration::from_millis(350));
        let demo_requested = std::env::var_os("HUNK_DEMO").is_some();
        let file_arg = std::env::args()
            .skip(1)
            .find(|argument| !argument.starts_with("--") && Path::new(argument).is_file());

        let mut app = Self {
            source: None,
            result: None,
            brand_icon,
            language: settings.language,
            history: settings.history,
            maximized: settings.maximized,
            native_maximized_seen: false,
            sample_pending_history: false,
            color_panel_height: 300.0,
            format: OutputFormat::Webp,
            quality: 82,
            ico_selected: [true; 6],
            eyedropper_active: false,
            sampled_color: None,
            sampled_position: None,
            split: 0.5,
            zoom: MIN_ZOOM,
            pan: Vec2::ZERO,
            revision: 0,
            pending_since: None,
            worker: None,
            toast: None,
            screenshot_path,
            screenshot_at: None,
            screenshot_requested: false,
            screenshot_delay,
        };

        if demo_requested {
            app.load_demo(&cc.egui_ctx);
        } else if let Some(path) = file_arg {
            app.load_path(Path::new(&path), &cc.egui_ctx);
        }

        if let Ok(format) = std::env::var("HUNK_FORMAT") {
            let format = match format.to_lowercase().as_str() {
                "jpg" | "jpeg" => OutputFormat::Jpg,
                "png" => OutputFormat::Png,
                "ico" => OutputFormat::Ico,
                "avif" => OutputFormat::Avif,
                "jxl" | "jpegxl" => OutputFormat::Jxl,
                _ => OutputFormat::Webp,
            };
            app.set_format(format, &cc.egui_ctx);
        }

        app
    }

    fn t(&self, key: K) -> &'static str {
        i18n::t(self.language, key)
    }

    fn save_settings(&self) {
        config::Settings {
            language: self.language,
            history: self.history.clone(),
            maximized: self.maximized,
        }
        .save();
    }

    fn sync_native_maximized(&mut self, ctx: &egui::Context) {
        let Some(maximized) = ctx.input(|input| input.viewport().maximized) else {
            return;
        };
        if !self.native_maximized_seen {
            self.maximized = maximized;
            self.native_maximized_seen = true;
        } else if maximized != self.maximized {
            self.maximized = maximized;
            self.save_settings();
            ctx.request_repaint();
        }
    }

    fn remember_color(&mut self, rgba: [u8; 4]) -> bool {
        if config::remember_color(&mut self.history, rgba) {
            self.save_settings();
            true
        } else {
            self.toast = Some(Toast {
                message: self.t(K::HistoryFull).to_owned(),
                error: true,
                until: Instant::now() + Duration::from_secs(4),
            });
            false
        }
    }

    fn selected_ico_sizes(&self) -> Vec<u32> {
        ICO_SIZES
            .iter()
            .zip(self.ico_selected)
            .filter_map(|(size, selected)| selected.then_some(*size))
            .collect()
    }

    fn busy(&self) -> bool {
        self.worker.is_some() || self.pending_since.is_some()
    }

    fn can_export(&self) -> bool {
        !self.busy()
            && self
                .result
                .as_ref()
                .is_some_and(|result| result.format == self.format)
    }

    fn view_rect(&self, fit_rect: Rect, area: Rect) -> Rect {
        Rect::from_center_size(area.center() + self.pan, fit_rect.size() * self.zoom)
    }

    fn clamp_pan(&mut self, fit_rect: Rect, area: Rect) {
        if self.zoom <= MIN_ZOOM + f32::EPSILON {
            self.zoom = MIN_ZOOM;
            self.pan = Vec2::ZERO;
            return;
        }
        let size = fit_rect.size() * self.zoom;
        let max_x = ((size.x - area.width()) * 0.5).max(0.0);
        let max_y = ((size.y - area.height()) * 0.5).max(0.0);
        self.pan.x = self.pan.x.clamp(-max_x, max_x);
        self.pan.y = self.pan.y.clamp(-max_y, max_y);
    }

    fn zoom_at(&mut self, scroll_y: f32, pointer: egui::Pos2, fit_rect: Rect, area: Rect) {
        let factor = (scroll_y * 0.0025).exp();
        let new_zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        if (new_zoom - self.zoom).abs() < f32::EPSILON {
            return;
        }

        let old_rect = self.view_rect(fit_rect, area);
        if old_rect.width() <= 0.0 || old_rect.height() <= 0.0 {
            return;
        }
        let normalized = vec2(
            ((pointer.x - old_rect.left()) / old_rect.width()).clamp(0.0, 1.0),
            ((pointer.y - old_rect.top()) / old_rect.height()).clamp(0.0, 1.0),
        );
        let new_size = fit_rect.size() * new_zoom;
        let new_center = pointer - normalized * new_size + new_size * 0.5;

        self.zoom = new_zoom;
        self.pan = new_center - area.center();
        self.clamp_pan(fit_rect, area);
    }

    fn request_encode(&mut self, ctx: &egui::Context, delay: Option<Duration>) {
        self.revision = self.revision.wrapping_add(1);
        self.pending_since = Some(Instant::now() + delay.unwrap_or(Duration::ZERO));
        self.screenshot_at = None;
        self.screenshot_requested = false;
        ctx.request_repaint();
    }

    fn maybe_start_worker(&mut self, ctx: &egui::Context) {
        if self.worker.is_some() || self.source.is_none() {
            return;
        }
        let Some(due) = self.pending_since else {
            return;
        };
        let now = Instant::now();
        if now < due {
            ctx.request_repaint_after(due - now);
            return;
        }
        let Some(source) = self.source.as_ref() else {
            return;
        };
        if self.format == OutputFormat::Ico && self.selected_ico_sizes().is_empty() {
            self.toast = Some(Toast {
                message: self.t(K::SelectIcoSizeToast).to_owned(),
                error: true,
                until: now + Duration::from_secs(3),
            });
            self.pending_since = None;
            return;
        }

        let revision = self.revision;
        let pixels = Arc::clone(&source.pixels);
        let options = EncodeOptions {
            format: self.format,
            quality: self.quality,
            ico_sizes: self.selected_ico_sizes(),
            background: [255, 255, 255, 255],
        };
        let (sender, receiver) = mpsc::channel();
        let thread_ctx = ctx.clone();
        std::thread::spawn(move || {
            let started = Instant::now();
            let result = codec::encode(&pixels, &options);
            let elapsed = started.elapsed();
            let _ = sender.send(JobMessage {
                revision,
                result,
                elapsed,
            });
            thread_ctx.request_repaint();
        });
        self.worker = Some(receiver);
        self.pending_since = None;
        ctx.request_repaint();
    }

    fn poll_worker(&mut self, ctx: &egui::Context) {
        let Some(receiver) = self.worker.as_ref() else {
            return;
        };
        match receiver.try_recv() {
            Ok(message) => {
                self.worker = None;
                if message.revision == self.revision && self.pending_since.is_none() {
                    match message.result {
                        Ok(output) => self.apply_output(ctx, output, message.elapsed),
                        Err(error) => {
                            self.toast = Some(Toast {
                                message: i18n::localize_error(self.language, &error),
                                error: true,
                                until: Instant::now() + Duration::from_secs(4),
                            });
                        }
                    }
                }
                ctx.request_repaint();
            }
            Err(TryRecvError::Disconnected) => {
                self.worker = None;
                self.toast = Some(Toast {
                    message: self.t(K::WorkerInterrupted).to_owned(),
                    error: true,
                    until: Instant::now() + Duration::from_secs(4),
                });
                ctx.request_repaint();
            }
            Err(TryRecvError::Empty) => {
                ctx.request_repaint_after(Duration::from_millis(16));
            }
        }
    }

    fn apply_output(&mut self, ctx: &egui::Context, output: EncodedOutput, elapsed: Duration) {
        let preview = upload_texture(ctx, &format!("result-{}", self.revision), &output.preview);
        let format = self.format;
        if std::env::var_os("HUNK_METRICS").is_some() {
            eprintln!(
                "[metrics] format={format:?} bytes={} encode_ms={:.1} preview={}x{}",
                output.bytes.len(),
                elapsed.as_secs_f64() * 1000.0,
                output.preview.width(),
                output.preview.height()
            );
        }
        self.result = Some(RenderedResult {
            texture: preview,
            size: output.bytes.len() as u64,
            bytes: output.bytes,
            width: output.width,
            height: output.height,
            frame_count: output.frame_count,
            format,
        });
        self.screenshot_at = None;
        ctx.request_repaint();
    }

    fn set_source(
        &mut self,
        pixels: RgbaImage,
        name: String,
        stem: String,
        format: String,
        file_bytes: u64,
        ctx: &egui::Context,
    ) {
        let (width, height) = pixels.dimensions();
        let preview = resized_for_ui(&pixels, UI_PREVIEW_MAX);
        let texture = upload_texture(ctx, &format!("source-{}", self.revision), &preview);
        self.source = Some(Source {
            name,
            stem,
            format,
            file_bytes,
            width,
            height,
            pixels: Arc::new(pixels),
            texture,
        });
        self.result = None;
        self.eyedropper_active = false;
        self.sampled_color = None;
        self.sampled_position = None;
        self.sample_pending_history = false;
        self.split = 0.5;
        self.zoom = MIN_ZOOM;
        self.pan = Vec2::ZERO;
        self.screenshot_at = None;
        self.screenshot_requested = false;
        self.request_encode(ctx, None);
    }

    fn load_path(&mut self, path: &Path, ctx: &egui::Context) {
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
                    loaded.pixels,
                    name,
                    stem,
                    loaded.format,
                    loaded.file_bytes,
                    ctx,
                );
            }
            Err(error) => {
                self.toast = Some(Toast {
                    message: i18n::localize_error(self.language, &error),
                    error: true,
                    until: Instant::now() + Duration::from_secs(4),
                });
            }
        }
    }

    fn load_demo(&mut self, ctx: &egui::Context) {
        let pixels = crate::demo::landscape(1600, 1000);
        let mut encoded = Vec::new();
        let _ = image::DynamicImage::ImageRgba8(pixels.clone())
            .write_to(&mut std::io::Cursor::new(&mut encoded), ImageFormat::Png);
        self.set_source(
            pixels,
            "demo.png".to_owned(),
            "demo".to_owned(),
            "PNG".to_owned(),
            encoded.len() as u64,
            ctx,
        );
    }

    fn paste_clipboard_image(&mut self, ctx: &egui::Context) {
        let mut clipboard = match arboard::Clipboard::new() {
            Ok(clipboard) => clipboard,
            Err(error) => {
                self.toast = Some(Toast {
                    message: format!("{} {error}", self.t(K::ClipboardOpenError)),
                    error: true,
                    until: Instant::now() + Duration::from_secs(4),
                });
                ctx.request_repaint();
                return;
            }
        };
        let image = match clipboard.get_image() {
            Ok(image) => image,
            Err(arboard::Error::ContentNotAvailable) => return,
            Err(error) => {
                self.toast = Some(Toast {
                    message: format!("{} {error}", self.t(K::ClipboardReadError)),
                    error: true,
                    until: Instant::now() + Duration::from_secs(4),
                });
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
            self.toast = Some(Toast {
                message: self.t(K::ClipboardInvalid).to_owned(),
                error: true,
                until: Instant::now() + Duration::from_secs(4),
            });
            ctx.request_repaint();
            return;
        }

        let Some(pixels) = RgbaImage::from_raw(width, height, image.bytes.into_owned()) else {
            return;
        };
        let file_bytes = clipboard_png_size(&pixels);
        self.set_source(
            pixels,
            i18n::clipboard_name(self.language).to_owned(),
            i18n::clipboard_name(self.language)
                .trim_end_matches(".png")
                .to_owned(),
            "PNG".to_owned(),
            file_bytes,
            ctx,
        );
        self.toast = Some(Toast {
            message: self.t(K::ClipboardPasted).to_owned(),
            error: false,
            until: Instant::now() + Duration::from_secs(3),
        });
        ctx.request_repaint();
    }

    fn open_file_dialog(&mut self, ctx: &egui::Context) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(
                self.t(K::ImagesFilter),
                &["png", "jpg", "jpeg", "webp", "bmp", "tga", "svg"],
            )
            .pick_file()
        {
            self.load_path(&path, ctx);
        }
    }

    fn reset(&mut self, ctx: &egui::Context) {
        self.source = None;
        self.result = None;
        self.format = OutputFormat::Webp;
        self.quality = 82;
        self.ico_selected = [true; 6];
        self.split = 0.5;
        self.zoom = MIN_ZOOM;
        self.pan = Vec2::ZERO;
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

    fn set_format(&mut self, format: OutputFormat, ctx: &egui::Context) {
        if self.format == format {
            return;
        }
        self.format = format;
        if format == OutputFormat::Ico {
            self.eyedropper_active = false;
        }
        self.result = None;
        self.request_encode(ctx, None);
    }

    fn export(&mut self) {
        if !self.can_export() {
            return;
        }
        let Some(result) = self.result.as_ref() else {
            return;
        };
        let Some(source) = self.source.as_ref() else {
            return;
        };

        let directory = match std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(Path::to_path_buf))
        {
            Some(path) => path.join("export"),
            None => {
                self.toast = Some(Toast {
                    message: self.t(K::ExportFolderError).to_owned(),
                    error: true,
                    until: Instant::now() + Duration::from_secs(4),
                });
                return;
            }
        };
        if let Err(error) = std::fs::create_dir_all(&directory) {
            self.toast = Some(Toast {
                message: format!("{} {error}", self.t(K::ExportCreateError)),
                error: true,
                until: Instant::now() + Duration::from_secs(4),
            });
            return;
        }

        let base = safe_stem(&source.stem);
        let extension = self.format.extension();
        let mut path = directory.join(format!("{base}.{extension}"));
        let mut suffix = 2;
        while path.exists() {
            path = directory.join(format!("{base}-{suffix}.{extension}"));
            suffix += 1;
        }

        match std::fs::write(&path, &result.bytes) {
            Ok(()) => {
                let file_name = path
                    .file_name()
                    .map(|value| value.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path.display().to_string());
                self.toast = Some(Toast {
                    message: i18n::saved_export(self.language, &file_name),
                    error: false,
                    until: Instant::now() + Duration::from_secs(4),
                });
            }
            Err(error) => {
                self.toast = Some(Toast {
                    message: format!("{} {error}", self.t(K::ExportWriteError)),
                    error: true,
                    until: Instant::now() + Duration::from_secs(4),
                });
            }
        }
    }

    fn show_header(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        egui::Panel::top("app_header")
            .frame(
                Frame::new()
                    .fill(T::BG)
                    .inner_margin(Margin::symmetric(30, 14)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if let Some(icon) = &self.brand_icon {
                        ui.add(
                            egui::Image::new((icon.id(), vec2(34.0, 34.0)))
                                .corner_radius(CornerRadius::same(8)),
                        );
                    }
                    ui.add_space(2.0);
                    ui.vertical(|ui| {
                        ui.add_space(1.0);
                        ui.label(RichText::new("Hunk Converter").size(21.0).color(T::TEXT));
                        ui.label(
                            RichText::new("IMAGE TOOLBOX")
                                .size(8.5)
                                .color(T::TEXT_FAINT),
                        );
                    });

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if self.source.is_some() {
                            let reset = egui::Button::new(
                                RichText::new(self.t(K::Reset))
                                    .size(11.5)
                                    .color(T::TEXT_MUTED),
                            )
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::new(1.0, T::BORDER))
                            .corner_radius(CornerRadius::same(10))
                            .min_size(vec2(0.0, 32.0));
                            if ui.add(reset).clicked() {
                                self.reset(&ctx);
                            }
                        }

                        let mut language = self.language;
                        egui::ComboBox::from_id_salt("language_selector")
                            .selected_text(
                                RichText::new(language.short())
                                    .size(11.0)
                                    .color(T::TEXT_MUTED),
                            )
                            .width(64.0)
                            .show_ui(ui, |ui| {
                                for option in Lang::ALL {
                                    ui.selectable_value(&mut language, option, option.name());
                                }
                            })
                            .response
                            .on_hover_text(self.t(K::Language));
                        if language != self.language {
                            self.language = language;
                            self.save_settings();
                            ctx.request_repaint();
                        }
                    });
                });
            });
    }

    fn show_footer(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("footer")
            .frame(
                Frame::new()
                    .fill(T::BG)
                    .inner_margin(Margin::symmetric(30, 9)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "Rust + egui · Open Source · v{}",
                            env!("CARGO_PKG_VERSION")
                        ))
                        .size(9.5)
                        .color(T::TEXT_FAINT),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.hyperlink_to(
                            RichText::new("github.com/hunkstalker/hunk-converter")
                                .size(9.5)
                                .color(T::TEXT_MUTED),
                            "https://github.com/hunkstalker/hunk-converter",
                        )
                        .on_hover_text(self.t(K::OpenRepository));
                    });
                });
            });
    }

    fn show_sidebar(&mut self, root: &mut egui::Ui) {
        let ctx = root.ctx().clone();
        let Some(source) = self.source.as_ref() else {
            return;
        };
        let source_texture = source.texture.id();
        let file_name = source.name.clone();
        let file_format = source.format.clone();
        let file_size = source.file_bytes;
        let dimensions = (source.width, source.height);

        egui::Panel::left("settings_panel")
            .resizable(false)
            .exact_size(390.0)
            .show_separator_line(false)
            .frame(Frame::new().fill(T::BG).inner_margin(Margin {
                left: 30,
                right: 20,
                top: 14,
                bottom: 14,
            }))
            .show(root, |ui| {
                T::sidebar_frame().show(ui, |ui| {
                    ui.set_min_height(ui.available_height());
                    ui.set_width(ui.available_width());

                    let export_height = 68.0;
                    let scroll_height =
                        (ui.available_height() - self.color_panel_height - export_height - 8.0)
                            .max(160.0);
                    let mut settings_scroll = egui::ScrollArea::vertical()
                        .id_salt("sidebar_scroll")
                        .auto_shrink([false, false])
                        .max_height(scroll_height);
                    if scroll_height < 300.0 {
                        settings_scroll = settings_scroll.scroll_bar_visibility(
                            egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                        );
                    }
                    settings_scroll.show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        self.show_sidebar_top(
                            ui,
                            &ctx,
                            source_texture,
                            &file_name,
                            &file_format,
                            file_size,
                            dimensions,
                        );
                    });

                    ui.add_space(4.0);
                    let color_panel = ui.scope(|ui| self.show_color_panel(ui, &ctx));
                    self.color_panel_height = color_panel.response.rect.height();
                    let remaining = (ui.available_height() - export_height).max(0.0);
                    ui.add_space(remaining);
                    self.show_export(ui);
                });
            });
    }

    #[allow(clippy::too_many_arguments)]
    fn show_sidebar_top(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        source_texture: egui::TextureId,
        file_name: &str,
        file_format: &str,
        file_size: u64,
        dimensions: (u32, u32),
    ) {
        let mut change_requested = false;
        ui.horizontal(|ui| {
            ui.label(T::eyebrow(self.t(K::Image)));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let response = small_text_button(ui, self.t(K::Change))
                    .on_hover_text(self.t(K::ChangeTooltip));
                if response.clicked() {
                    change_requested = true;
                }
            });
        });
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            ui.add(
                egui::Image::new((source_texture, vec2(48.0, 48.0)))
                    .corner_radius(CornerRadius::same(9)),
            );
            ui.add_space(4.0);
            ui.vertical(|ui| {
                ui.add_space(3.0);
                ui.label(RichText::new(file_name).size(12.5).color(T::TEXT).strong());
                ui.label(
                    RichText::new(format!(
                        "{} · {} × {} px · {}",
                        file_format,
                        dimensions.0,
                        dimensions.1,
                        i18n::localized_bytes(self.language, file_size)
                    ))
                    .size(9.5)
                    .color(T::TEXT_MUTED),
                );
            });
        });
        if change_requested {
            self.open_file_dialog(ctx);
        }

        ui.add_space(6.0);
        hline(ui);
        ui.add_space(6.0);
        ui.label(T::eyebrow(self.t(K::OutputFormat)));
        ui.add_space(6.0);

        let gap = 6.0;
        let card_width = (ui.available_width() - gap * 2.0) / 3.0;
        let mut requested_format = None;
        for (row, formats) in OutputFormat::ALL.chunks(3).enumerate() {
            ui.horizontal(|ui| {
                for (column, format) in formats.iter().enumerate() {
                    let hint = i18n::format_hint(self.language, *format);
                    if format_card(ui, *format, self.format, card_width, hint).clicked() {
                        requested_format = Some(*format);
                    }
                    if column + 1 < formats.len() {
                        ui.add_space(gap);
                    }
                }
            });
            if row + 1 < OutputFormat::ALL.len().div_ceil(3) {
                ui.add_space(gap);
            }
        }
        if let Some(format) = requested_format {
            self.set_format(format, ctx);
        }

        ui.add_space(10.0);
        if self.format == OutputFormat::Ico {
            self.show_ico_sizes(ui, ctx);
        } else {
            self.show_quality(ui, ctx);
        }
    }

    fn show_quality(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let lossy = self.format.is_lossy();
        let hint = if lossy {
            self.t(K::QualityHint)
        } else {
            self.t(K::LosslessHint)
        };
        let header = ui.horizontal(|ui| {
            ui.label(T::eyebrow(self.t(K::Quality)));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let value = if lossy {
                    format!("{} %", self.quality)
                } else {
                    self.t(K::Lossless).to_owned()
                };
                ui.label(RichText::new(value).size(12.0).color(T::TEXT));
            });
        });
        header.response.on_hover_text(hint);

        if lossy {
            ui.add_space(3.0);
            ui.spacing_mut().slider_width = (ui.available_width() - 8.0).max(120.0);
            let slider = egui::Slider::new(&mut self.quality, 1..=100)
                .show_value(false)
                .trailing_fill(true);
            let response = ui.add(slider);
            response.clone().on_hover_text(hint);
            if response.changed() {
                self.request_encode(ctx, Some(Duration::from_millis(ENCODE_DELAY_MS)));
            }
        }
    }

    fn show_ico_sizes(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.label(T::eyebrow(self.t(K::IconSizes)));
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for (index, size) in ICO_SIZES.iter().enumerate() {
                let selected = self.ico_selected[index];
                let button = egui::Button::new(
                    RichText::new(size.to_string())
                        .size(10.5)
                        .color(if selected { T::ACCENT } else { T::TEXT_MUTED }),
                )
                .fill(if selected {
                    T::ACCENT_SOFT
                } else {
                    T::SURFACE_2
                })
                .stroke(Stroke::new(
                    1.0,
                    if selected { T::ACCENT } else { T::BORDER },
                ))
                .corner_radius(CornerRadius::same(8))
                .min_size(vec2(42.0, 26.0));
                if ui.add(button).clicked() {
                    let selected_count = self.ico_selected.iter().filter(|value| **value).count();
                    if selected && selected_count == 1 {
                        self.toast = Some(Toast {
                            message: self.t(K::SelectIcoSize).to_owned(),
                            error: true,
                            until: Instant::now() + Duration::from_secs(3),
                        });
                    } else {
                        self.ico_selected[index] = !self.ico_selected[index];
                        self.request_encode(ctx, None);
                    }
                }
            }
        });
        ui.add_space(4.0);
        ui.label(
            RichText::new(self.t(K::IcoCentered))
                .size(9.5)
                .color(T::TEXT_FAINT),
        );
    }

    fn show_color_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let mut activate_eyedropper = false;
        let mut toggle_eyedropper = false;
        let mut copy_text: Option<(String, String)> = None;
        let mut select_history = None;
        let mut toggle_lock = None;
        let mut clear_history = false;
        let icon_mode = self.format == OutputFormat::Ico;
        let sample = self.sampled_color;
        let sampled_position = self.sampled_position;
        let history = self.history.clone();
        let has_unlocked = history.iter().any(|entry| !entry.locked);

        T::soft_frame().show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.horizontal(|ui| {
                ui.label(T::eyebrow(self.t(K::History)));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let response = icon_action_button(ui, IconAction::Trash, has_unlocked)
                        .on_hover_text(self.t(K::ClearHistory));
                    if has_unlocked && response.clicked() {
                        clear_history = true;
                    }
                });
            });
            ui.add_space(4.0);

            let content_width = ui.available_width();
            let rows = HISTORY_LIMIT.div_ceil(HISTORY_COLUMNS);
            ui.spacing_mut().item_spacing = vec2(SWATCH_GAP, SWATCH_GAP);
            for row in 0..rows {
                let count = (HISTORY_LIMIT - row * HISTORY_COLUMNS).min(HISTORY_COLUMNS);
                let row_width =
                    count as f32 * SWATCH_SIZE + count.saturating_sub(1) as f32 * SWATCH_GAP;
                let offset = ((content_width - row_width) * 0.5).max(0.0);
                ui.horizontal(|ui| {
                    ui.add_space(offset);
                    for column in 0..count {
                        let index = row * HISTORY_COLUMNS + column;
                        let entry = history.get(index);
                        let (rect, response) =
                            ui.allocate_exact_size(vec2(SWATCH_SIZE, SWATCH_SIZE), Sense::click());
                        paint_history_swatch(ui.painter(), rect, entry, sample);
                        if let Some(entry) = entry {
                            let response = response
                                .on_hover_cursor(CursorIcon::PointingHand)
                                .on_hover_text(i18n::history_tooltip(
                                    self.language,
                                    &config::hex_rgba(entry.rgba),
                                    entry.locked,
                                ));
                            if response.clicked() {
                                select_history = Some(entry.rgba);
                            } else if response.clicked_by(egui::PointerButton::Secondary) {
                                toggle_lock = Some(index);
                            }
                        }
                    }
                });
                if row + 1 < rows {
                    ui.add_space(SWATCH_GAP);
                }
            }
            ui.add_space(4.0);
            hline(ui);
            ui.add_space(4.0);

            ui.spacing_mut().interact_size.y = 20.0;
            ui.horizontal(|ui| {
                ui.label(T::eyebrow(self.t(K::Eyedropper)));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let response = eyedropper_button(
                        ui,
                        self.eyedropper_active,
                        !icon_mode,
                        self.t(K::Eyedropper),
                    );
                    let hint = if icon_mode {
                        self.t(K::EyedropperUnavailable)
                    } else if self.eyedropper_active {
                        self.t(K::DisableEyedropper)
                    } else {
                        self.t(K::EnableEyedropper)
                    };
                    let response = response.on_hover_text(hint);
                    if !icon_mode {
                        if response.drag_started() {
                            activate_eyedropper = true;
                        } else if response.clicked()
                            || (self.eyedropper_active
                                && response.clicked_by(egui::PointerButton::Secondary))
                        {
                            toggle_eyedropper = true;
                        }
                    }
                });
            });
            ui.add_space(3.0);

            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(36.0, 24.0), Sense::hover());
                let painter = ui.painter();
                paint_checker(painter, rect);
                if let Some(rgba) = sample {
                    painter.rect_filled(
                        rect,
                        CornerRadius::same(5),
                        Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3]),
                    );
                }
                painter.rect_stroke(
                    rect,
                    CornerRadius::same(5),
                    Stroke::new(1.0, T::BORDER_STRONG),
                    StrokeKind::Inside,
                );
                ui.add_space(2.0);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    let message = if icon_mode {
                        self.t(K::EyedropperIcoHint)
                    } else if self.eyedropper_active {
                        if sample.is_some() {
                            self.t(K::SampleCapturedChoose)
                        } else {
                            self.t(K::SampleClickOrDrag)
                        }
                    } else if sample.is_some() && sampled_position.is_none() {
                        self.t(K::SampleFromHistory)
                    } else if sample.is_some() {
                        self.t(K::SampleCaptured)
                    } else {
                        self.t(K::EyedropperIdle)
                    };
                    ui.add(
                        egui::Label::new(RichText::new(message).size(9.5).color(T::TEXT_MUTED))
                            .truncate(),
                    );
                    let coords = sampled_position
                        .map(|(x, y)| i18n::pixel_coords(self.language, x, y))
                        .unwrap_or_else(|| "\u{00A0}".to_owned());
                    ui.label(
                        RichText::new(coords)
                            .font(FontId::monospace(8.5))
                            .color(T::TEXT_FAINT),
                    );
                });
            });
            ui.add_space(2.0);

            ui.spacing_mut().interact_size.y = 14.0;
            ui.spacing_mut().item_spacing.y = 1.0;
            let codes = sample.map(color::color_codes).unwrap_or_else(|| {
                [
                    ("HEX", "—".to_owned()),
                    ("RGB", "—".to_owned()),
                    ("RGBA", "—".to_owned()),
                    ("HSL", "—".to_owned()),
                ]
            });
            for (label, value) in codes {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(label).size(9.0).color(T::TEXT_FAINT).strong());
                    ui.label(
                        RichText::new(&value)
                            .font(FontId::monospace(10.0))
                            .color(T::TEXT_MUTED),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let response = icon_action_button(ui, IconAction::Copy, sample.is_some())
                            .on_hover_text(i18n::copy_tooltip(self.language, label));
                        if sample.is_some() && response.clicked() {
                            copy_text = Some((label.to_owned(), value.clone()));
                        }
                    });
                });
            }
        });

        if let Some(rgba) = select_history {
            self.sampled_color = Some(rgba);
            self.sampled_position = None;
            self.eyedropper_active = false;
            self.sample_pending_history = false;
            ctx.request_repaint();
        }
        if let Some(index) = toggle_lock
            && let Some(entry) = self.history.get_mut(index)
        {
            entry.locked = !entry.locked;
            let key = if entry.locked {
                K::ColorPinned
            } else {
                K::ColorUnpinned
            };
            self.toast = Some(Toast {
                message: self.t(key).to_owned(),
                error: false,
                until: Instant::now() + Duration::from_secs(2),
            });
            self.save_settings();
            ctx.request_repaint();
        }
        if clear_history {
            self.history.retain(|entry| entry.locked);
            self.save_settings();
            self.toast = Some(Toast {
                message: self.t(K::HistoryCleared).to_owned(),
                error: false,
                until: Instant::now() + Duration::from_secs(2),
            });
            ctx.request_repaint();
        }
        if activate_eyedropper {
            self.eyedropper_active = true;
            ctx.request_repaint();
        } else if toggle_eyedropper {
            self.eyedropper_active = !self.eyedropper_active;
            ctx.request_repaint();
        }
        if let Some((label, text)) = copy_text {
            ctx.copy_text(text);
            self.toast = Some(Toast {
                message: i18n::copied_message(self.language, &label),
                error: false,
                until: Instant::now() + Duration::from_secs(2),
            });
            ctx.request_repaint();
        }
    }

    fn show_export(&mut self, ui: &mut egui::Ui) {
        let enabled = self.can_export();
        let label = i18n::export_label(self.language, self.format.extension());
        let button = egui::Button::new(
            RichText::new(&label)
                .size(13.0)
                .color(if enabled {
                    T::ACCENT_INK
                } else {
                    T::TEXT_FAINT
                })
                .strong(),
        )
        .fill(if enabled { T::ACCENT } else { T::SURFACE_3 })
        .stroke(Stroke::new(
            1.0,
            if enabled { T::ACCENT } else { T::BORDER },
        ))
        .corner_radius(CornerRadius::same(12))
        .min_size(vec2(ui.available_width(), 43.0));
        if ui.add(button).clicked() && enabled {
            self.export();
        }
        ui.add_space(4.0);
        let hint = if self.busy() {
            self.t(K::Calculating)
        } else {
            self.t(K::ExportHint)
        };
        ui.label(RichText::new(hint).size(9.0).color(T::TEXT_FAINT));
    }

    fn show_preview(&mut self, root: &mut egui::Ui) {
        let ctx = root.ctx().clone();
        egui::CentralPanel::default()
            .frame(Frame::new().fill(T::BG).inner_margin(Margin {
                left: 20,
                right: 30,
                top: 8,
                bottom: 12,
            }))
            .show(root, |ui| {
                egui::Panel::bottom("preview_footer_panel")
                    .resizable(false)
                    .show_separator_line(false)
                    .frame(Frame::new().inner_margin(Margin {
                        left: 0,
                        right: 0,
                        top: 8,
                        bottom: 0,
                    }))
                    .show(ui, |ui| {
                        self.show_preview_footer(ui);
                    });

                ui.horizontal(|ui| {
                    ui.label(RichText::new(self.t(K::Preview)).size(17.0).color(T::TEXT));
                    ui.add_space(4.0);
                    if self.format != OutputFormat::Ico {
                        ui.add_space(2.0);
                        let zoom_percent = (self.zoom * 100.0).round() as u32;
                        if self.zoom > MIN_ZOOM + 0.001 {
                            let reset = egui::Button::new(
                                RichText::new(format!("Zoom {zoom_percent} %"))
                                    .size(9.5)
                                    .color(T::ACCENT),
                            )
                            .frame(false);
                            if ui.add(reset).on_hover_text(self.t(K::ResetZoom)).clicked() {
                                self.zoom = MIN_ZOOM;
                                self.pan = Vec2::ZERO;
                                ctx.request_repaint();
                            }
                        } else {
                            ui.label(RichText::new("Zoom 100 %").size(9.5).color(T::TEXT_FAINT));
                        }
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if self.busy() {
                            ui.add(egui::Spinner::new().size(12.0).color(T::ACCENT));
                            ui.label(
                                RichText::new(self.t(K::Processing))
                                    .size(9.5)
                                    .color(T::TEXT_MUTED),
                            );
                        } else if self.eyedropper_active {
                            ui.label(
                                RichText::new(self.t(K::EyedropperActiveHint))
                                    .size(9.5)
                                    .color(T::ACCENT),
                            );
                        } else if self.format == OutputFormat::Ico {
                            ui.label(
                                RichText::new(self.t(K::IcoGeneratedView))
                                    .size(9.5)
                                    .color(T::TEXT_FAINT),
                            );
                        } else if self.zoom > MIN_ZOOM + 0.001 {
                            ui.label(
                                RichText::new(self.t(K::PanZoomHint))
                                    .size(9.5)
                                    .color(T::TEXT_FAINT),
                            );
                        } else {
                            ui.label(
                                RichText::new(self.t(K::DividerZoomHint))
                                    .size(9.5)
                                    .color(T::TEXT_FAINT),
                            );
                        }
                    });
                });
                ui.add_space(7.0);

                let canvas_height = (ui.available_height() - 20.0).max(240.0);
                let frame = Frame::new()
                    .fill(T::BG_SOFT)
                    .stroke(Stroke::new(1.0, T::BORDER))
                    .corner_radius(CornerRadius::same(20))
                    .inner_margin(Margin::same(10));
                frame.show(ui, |ui| {
                    ui.set_min_size(vec2(ui.available_width(), canvas_height));
                    let size = vec2(ui.available_width(), canvas_height);
                    let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());
                    self.paint_canvas(ui, rect, &response, &ctx);
                });
            });
    }

    fn paint_canvas(
        &mut self,
        ui: &egui::Ui,
        rect: Rect,
        response: &egui::Response,
        ctx: &egui::Context,
    ) {
        let painter = ui.painter();
        let area = rect.shrink(12.0);
        let Some(source) = self.source.as_ref() else {
            return;
        };
        let source_texture = source.texture.id();
        let source_width = source.width;
        let source_height = source.height;
        let source_pixels = Arc::clone(&source.pixels);
        let source_size = vec2(source_width as f32, source_height as f32);
        let icon_mode = self.format == OutputFormat::Ico;

        if icon_mode {
            if let Some(result) = self
                .result
                .as_ref()
                .filter(|result| result.format == self.format)
            {
                let icon_size = vec2(result.width as f32, result.height as f32);
                let icon_rect = fit_rect(icon_size, area);
                paint_checker(painter, icon_rect);
                painter.image(
                    result.texture.id(),
                    icon_rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
                paint_badge(
                    painter,
                    pos2(icon_rect.right() - 10.0, icon_rect.top() + 10.0),
                    Align2::RIGHT_TOP,
                    i18n::ico_badge(self.language, result.frame_count).as_str(),
                    T::ACCENT,
                    T::ACCENT_INK,
                );
            } else {
                painter.text(
                    area.center(),
                    Align2::CENTER_CENTER,
                    self.t(K::GeneratingIcon),
                    FontId::proportional(13.0),
                    T::TEXT_FAINT,
                );
            }
            return;
        }

        let fit_rect = fit_rect(source_size, area);
        self.clamp_pan(fit_rect, area);
        let mut image_rect = self.view_rect(fit_rect, area);

        let pointer = ctx.input(|input| input.pointer.hover_pos());
        let right_clicked =
            ctx.input(|input| input.pointer.button_pressed(egui::PointerButton::Secondary));
        let deactivated_eyedropper = self.eyedropper_active && right_clicked;
        if deactivated_eyedropper {
            self.eyedropper_active = false;
            self.sample_pending_history = false;
        }
        if response.hovered()
            && let Some(position) = pointer
            && image_rect.contains(position)
        {
            let scroll = ctx.input(|input| input.smooth_scroll_delta.y);
            if scroll.abs() > f32::EPSILON {
                self.zoom_at(scroll, position, fit_rect, area);
                image_rect = self.view_rect(fit_rect, area);
            }
        }
        if !deactivated_eyedropper
            && response.dragged_by(egui::PointerButton::Secondary)
            && self.zoom > MIN_ZOOM
        {
            let delta = ctx.input(|input| input.pointer.delta());
            self.pan += delta;
            self.clamp_pan(fit_rect, area);
            image_rect = self.view_rect(fit_rect, area);
        }

        let view_rect = image_rect.intersect(area);
        let mut canvas = painter.clone();
        canvas.set_clip_rect(area);

        paint_checker(&canvas, image_rect);
        canvas.image(
            source_texture,
            image_rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        let result_texture = self
            .result
            .as_ref()
            .filter(|result| result.format == self.format)
            .map(|result| result.texture.id());
        let split_x = image_rect.left() + image_rect.width() * self.split;
        let over_handle = result_texture.is_some()
            && pointer.is_some_and(|position| pointer_over_split(view_rect, split_x, position));

        if self.eyedropper_active {
            if response.hovered() {
                ctx.set_cursor_icon(CursorIcon::Crosshair);
            }
            if (ctx.input(|input| input.pointer.primary_down()) || response.clicked())
                && let Some(position) = pointer
                && image_rect.contains(position)
                && area.contains(position)
            {
                let nx = ((position.x - image_rect.left()) / image_rect.width()).clamp(0.0, 1.0);
                let ny = ((position.y - image_rect.top()) / image_rect.height()).clamp(0.0, 1.0);
                let x = (nx * source_width.saturating_sub(1) as f32).round() as u32;
                let y = (ny * source_height.saturating_sub(1) as f32).round() as u32;
                let pixel = source_pixels.get_pixel(x, y);
                self.sampled_color = Some(pixel.0);
                self.sampled_position = Some((x, y));
                self.sample_pending_history = true;
            }
        } else {
            if result_texture.is_some()
                && (response.dragged_by(egui::PointerButton::Primary) || response.clicked())
                && let Some(position) = ctx.input(|input| input.pointer.interact_pos())
            {
                self.split =
                    ((position.x - image_rect.left()) / image_rect.width()).clamp(0.015, 0.985);
            }
            if response.hovered() && over_handle {
                ctx.set_cursor_icon(CursorIcon::ResizeHorizontal);
            } else if self.zoom > MIN_ZOOM
                && (response.hovered() || response.dragged_by(egui::PointerButton::Secondary))
            {
                ctx.set_cursor_icon(if response.dragged_by(egui::PointerButton::Secondary) {
                    CursorIcon::Grabbing
                } else {
                    CursorIcon::Grab
                });
            }
        }

        if self.sample_pending_history && ctx.input(|input| input.pointer.primary_released()) {
            if let Some(rgba) = self.sampled_color {
                self.remember_color(rgba);
            }
            self.sample_pending_history = false;
        }

        if let Some(texture_id) = result_texture {
            let split_x = image_rect.left() + image_rect.width() * self.split;
            let split_clip_x = split_x.clamp(view_rect.left(), view_rect.right());
            let mut clipped = canvas.clone();
            clipped.set_clip_rect(Rect::from_min_max(
                view_rect.min,
                pos2(split_clip_x, view_rect.max.y),
            ));
            clipped.image(
                texture_id,
                image_rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );

            if split_x >= view_rect.left()
                && split_x <= view_rect.right()
                && view_rect.height() > 24.0
            {
                let line_color = Color32::from_rgba_unmultiplied(255, 255, 255, 210);
                canvas.line_segment(
                    [
                        pos2(split_x, view_rect.top() + 8.0),
                        pos2(split_x, view_rect.bottom() - 8.0),
                    ],
                    Stroke::new(3.0, Color32::from_black_alpha(80)),
                );
                canvas.line_segment(
                    [
                        pos2(split_x, view_rect.top() + 8.0),
                        pos2(split_x, view_rect.bottom() - 8.0),
                    ],
                    Stroke::new(1.2, line_color),
                );
                let center = pos2(split_x, view_rect.center().y);
                canvas.circle_filled(center, 18.0, Color32::from_black_alpha(100));
                canvas.circle_filled(center, 16.0, T::BG_SOFT);
                canvas.circle_stroke(
                    center,
                    16.0,
                    Stroke::new(1.2, Color32::from_white_alpha(150)),
                );
                for direction in [-1.0_f32, 1.0] {
                    let tip = center + vec2(direction * 9.5, 0.0);
                    canvas.line_segment(
                        [center + vec2(direction * 4.0, -4.5), tip],
                        Stroke::new(1.4, T::TEXT),
                    );
                    canvas.line_segment(
                        [tip, center + vec2(direction * 4.0, 4.5)],
                        Stroke::new(1.4, T::TEXT),
                    );
                }
            }
        } else if self.busy() {
            canvas.text(
                view_rect.center(),
                Align2::CENTER_CENTER,
                self.t(K::PreparingResult),
                FontId::proportional(13.0),
                T::TEXT_FAINT,
            );
        }

        if self.eyedropper_active
            && let (Some(rgba), Some((sample_x, sample_y))) =
                (self.sampled_color, self.sampled_position)
        {
            let ratio_x = sample_x as f32 / source_width.saturating_sub(1).max(1) as f32;
            let ratio_y = sample_y as f32 / source_height.saturating_sub(1).max(1) as f32;
            let marker =
                image_rect.min + vec2(ratio_x * image_rect.width(), ratio_y * image_rect.height());
            canvas.circle_stroke(
                marker,
                8.0,
                Stroke::new(3.0, Color32::from_black_alpha(180)),
            );
            canvas.circle_stroke(marker, 8.0, Stroke::new(1.2, Color32::WHITE));
            canvas.circle_filled(
                marker,
                3.0,
                Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3]),
            );
        }

        paint_badge(
            &canvas,
            view_rect.left_top() + vec2(10.0, 10.0),
            Align2::LEFT_TOP,
            self.format.badge(),
            T::ACCENT,
            T::ACCENT_INK,
        );
        paint_badge(
            &canvas,
            view_rect.right_top() + vec2(-10.0, 10.0),
            Align2::RIGHT_TOP,
            self.t(K::Original),
            Color32::from_rgba_unmultiplied(10, 13, 16, 225),
            T::TEXT,
        );
    }
    fn show_preview_footer(&mut self, ui: &mut egui::Ui) {
        let source_size = self.source.as_ref().map(|source| source.file_bytes);
        let output_size = self
            .result
            .as_ref()
            .filter(|result| result.format == self.format && !self.busy())
            .map(|result| result.size);

        ui.horizontal(|ui| {
            ui.label(T::eyebrow(self.t(K::Original)));
            ui.label(
                RichText::new(
                    source_size
                        .map(|bytes| i18n::localized_bytes(self.language, bytes))
                        .unwrap_or_else(|| "—".to_owned()),
                )
                .size(11.5)
                .color(T::TEXT),
            );
            T::bullet_separator(ui);
            ui.label(T::eyebrow(self.t(K::Result)));
            ui.label(
                RichText::new(
                    output_size
                        .map(|bytes| i18n::localized_bytes(self.language, bytes))
                        .unwrap_or_else(|| "—".to_owned()),
                )
                .size(11.5)
                .color(T::ACCENT),
            );
            if let (Some(original), Some(output)) = (source_size, output_size)
                && original > 0
            {
                let change = (1.0 - output as f64 / original as f64) * 100.0;
                let (text, color) = if change > 0.5 {
                    (format!("−{change:.0} %"), T::ACCENT)
                } else if change < -0.5 {
                    (format!("+{:.0} %", change.abs()), T::TEXT_MUTED)
                } else {
                    (self.t(K::EqualWeight).to_owned(), T::TEXT_MUTED)
                };
                ui.add_space(6.0);
                ui.label(RichText::new(text).size(9.5).color(color));
            }
        });
    }

    fn show_empty(&mut self, root: &mut egui::Ui) {
        let ctx = root.ctx().clone();
        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(T::BG)
                    .inner_margin(Margin::symmetric(30, 18)),
            )
            .show(root, |ui| {
                let available = ui.available_size();
                let card_width = 680.0_f32.min(available.x - 20.0);
                let card_height = 500.0_f32.min(available.y - 20.0);
                let top_gap = ((available.y - card_height) / 2.0).max(0.0);
                ui.add_space(top_gap);
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    T::card_frame().show(ui, |ui| {
                        ui.set_width(card_width - 36.0);
                        ui.set_min_height(card_height - 36.0);
                        ui.vertical_centered(|ui| {
                            ui.add_space(12.0);
                            if let Some(icon) = &self.brand_icon {
                                ui.add(
                                    egui::Image::new((icon.id(), vec2(104.0, 104.0)))
                                        .corner_radius(CornerRadius::same(24)),
                                );
                            }
                            ui.add_space(14.0);
                            ui.label(
                                RichText::new(self.t(K::EmptyTitle))
                                    .size(34.0)
                                    .color(T::TEXT),
                            );
                            ui.add_space(12.0);
                            ui.label(
                                RichText::new(self.t(K::EmptySubtitle))
                                    .size(12.5)
                                    .color(T::TEXT_MUTED),
                            );
                            ui.add_space(22.0);
                            let choose = egui::Button::new(
                                RichText::new(self.t(K::SelectImage))
                                    .size(13.5)
                                    .color(T::ACCENT_INK)
                                    .strong(),
                            )
                            .fill(T::ACCENT)
                            .stroke(Stroke::new(1.0, T::ACCENT))
                            .corner_radius(CornerRadius::same(12))
                            .min_size(vec2(230.0, 44.0));
                            if ui.add(choose).clicked() {
                                self.open_file_dialog(&ctx);
                            }
                            ui.add_space(16.0);
                            ui.label(
                                RichText::new("PNG · JPG · WEBP · BMP · TGA · SVG")
                                    .size(9.0)
                                    .color(T::TEXT_FAINT)
                                    .strong(),
                            );
                            ui.add_space(2.0);
                            ui.label(
                                RichText::new(self.t(K::PrivacyNote))
                                    .size(9.5)
                                    .color(T::TEXT_FAINT),
                            );
                        });
                    });
                });
            });
    }

    fn show_toast(&mut self, ctx: &egui::Context) {
        let Some(toast) = &self.toast else {
            return;
        };
        let now = Instant::now();
        if now >= toast.until {
            self.toast = None;
            return;
        }
        let remaining = toast.until - now;
        let text_color = if toast.error { T::DANGER } else { T::TEXT };
        let fill = if toast.error {
            Color32::from_rgb(42, 24, 26)
        } else {
            T::SURFACE_3
        };
        let stroke_color = if toast.error {
            Color32::from_rgba_unmultiplied(255, 126, 126, 140)
        } else {
            T::BORDER_STRONG
        };
        let message = toast.message.clone();

        egui::Area::new(Id::unique("toast"))
            .anchor(Align2::CENTER_TOP, vec2(0.0, 68.0))
            .order(egui::Order::Foreground)
            .interactable(false)
            .show(ctx, |ui| {
                let galley =
                    ui.painter()
                        .layout(message, FontId::proportional(11.0), text_color, 420.0);
                let padding = vec2(16.0, 9.0);
                let (rect, _) =
                    ui.allocate_exact_size(galley.size() + padding * 2.0, Sense::hover());
                let painter = ui.painter();
                painter.rect(
                    rect,
                    CornerRadius::same(10),
                    fill,
                    Stroke::new(1.0, stroke_color),
                    StrokeKind::Inside,
                );
                painter.galley(rect.min + padding, galley, text_color);
            });
        ctx.request_repaint_after(remaining.min(Duration::from_millis(100)));
    }

    fn maybe_screenshot(&mut self, ctx: &egui::Context) {
        let Some(path) = self.screenshot_path.clone() else {
            return;
        };
        if self.screenshot_requested {
            return;
        }
        if self.source.is_some() && (self.busy() || self.result.is_none()) {
            return;
        }
        let at = *self
            .screenshot_at
            .get_or_insert_with(|| Instant::now() + self.screenshot_delay);
        let now = Instant::now();
        if now < at {
            ctx.request_repaint_after(at - now);
            return;
        }

        self.screenshot_requested = true;
        let screenshot_ctx = ctx.clone();
        ctx.request_screenshot(move |image| {
            let width = image.width() as u32;
            let height = image.height() as u32;
            let mut rgba = Vec::with_capacity(image.pixels.len() * 4);
            for pixel in &image.pixels {
                rgba.extend_from_slice(&pixel.to_array());
            }
            let result = image::save_buffer(&path, &rgba, width, height, image::ColorType::Rgba8);
            match result {
                Ok(()) => eprintln!("[screenshot] {}", path.display()),
                Err(error) => eprintln!("[screenshot] error: {error}"),
            }
            screenshot_ctx.send_viewport_cmd(ViewportCommand::Close);
        });
    }
}

impl eframe::App for HunkApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.sync_native_maximized(&ctx);

        let dropped = ctx.input(|input| {
            input
                .raw
                .dropped_files
                .first()
                .map(|file| file.path().to_path_buf())
        });
        if let Some(path) = dropped {
            self.load_path(&path, &ctx);
        }

        if ctx.input(|input| input.modifiers.command && input.key_released(egui::Key::V)) {
            self.paste_clipboard_image(&ctx);
        }

        if self.eyedropper_active && ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.eyedropper_active = false;
            ctx.request_repaint();
        }

        let (export_shortcut, reset_shortcut, open_shortcut) = ctx.input(|input| {
            let command = input.modifiers.command;
            (
                command && input.key_pressed(egui::Key::E),
                command && input.key_pressed(egui::Key::R),
                command && input.key_pressed(egui::Key::O),
            )
        });
        if reset_shortcut {
            self.reset(&ctx);
        }
        if export_shortcut {
            self.export();
        }
        if open_shortcut {
            self.open_file_dialog(&ctx);
        }

        self.poll_worker(&ctx);
        self.maybe_start_worker(&ctx);

        self.show_header(ui);
        self.show_footer(ui);
        if self.source.is_some() {
            self.show_sidebar(ui);
            self.show_preview(ui);
        } else {
            self.show_empty(ui);
        }
        self.show_toast(&ctx);
        self.maybe_screenshot(&ctx);
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        T::BG.to_normalized_gamma_f32()
    }
}

fn eyedropper_button(
    ui: &mut egui::Ui,
    active: bool,
    enabled: bool,
    label: &str,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(32.0, 28.0), Sense::click_and_drag());
    let hovered = enabled && response.hovered();
    let fill = if active && enabled {
        T::ACCENT
    } else if hovered {
        T::SURFACE_3
    } else {
        T::SURFACE_2
    };
    let border = if active && enabled {
        T::ACCENT
    } else if hovered {
        T::BORDER_STRONG
    } else {
        T::BORDER
    };
    let icon = if !enabled {
        T::TEXT_FAINT
    } else if active {
        T::ACCENT_INK
    } else {
        T::TEXT_MUTED
    };

    let painter = ui.painter();
    painter.rect(
        rect,
        CornerRadius::same(8),
        fill,
        Stroke::new(1.0, border),
        StrokeKind::Inside,
    );
    paint_eyedropper(painter, rect.shrink2(vec2(8.0, 6.0)), icon);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, label));

    if enabled {
        response.on_hover_cursor(CursorIcon::PointingHand)
    } else {
        response
    }
}

fn paint_eyedropper(painter: &egui::Painter, rect: Rect, color: Color32) {
    let size = rect.width().min(rect.height());
    let center = rect.center();
    let tip = pos2(center.x - size * 0.40, center.y + size * 0.40);
    let shoulder = pos2(center.x - size * 0.08, center.y + size * 0.08);
    let bulb_center = pos2(center.x + size * 0.28, center.y - size * 0.28);
    let direction = (bulb_center - shoulder).normalized();
    let perpendicular = vec2(-direction.y, direction.x);

    painter.line_segment(
        [tip + direction * size * 0.05, bulb_center],
        Stroke::new(size * 0.11, color),
    );
    painter.circle_filled(bulb_center, size * 0.22, color);
    painter.line_segment(
        [
            shoulder - perpendicular * size * 0.18,
            shoulder + perpendicular * size * 0.18,
        ],
        Stroke::new(size * 0.08, color),
    );
    painter.line_segment(
        [
            bulb_center + direction * size * 0.14,
            bulb_center + direction * size * 0.26,
        ],
        Stroke::new(size * 0.08, color),
    );

    let tip_base = tip + direction * size * 0.18;
    painter.add(egui::Shape::convex_polygon(
        vec![
            tip,
            tip_base + perpendicular * size * 0.12,
            tip_base - perpendicular * size * 0.12,
        ],
        color,
        Stroke::NONE,
    ));
}

fn pointer_over_split(image_rect: Rect, split_x: f32, pointer: egui::Pos2) -> bool {
    let over_line = (pointer.x - split_x).abs() <= 10.0
        && pointer.y >= image_rect.top()
        && pointer.y <= image_rect.bottom();
    let over_circle = pointer.distance(pos2(split_x, image_rect.center().y)) <= 20.0;
    over_line || over_circle
}

#[derive(Clone, Copy)]
enum IconAction {
    Copy,
    Trash,
}

fn icon_action_button(ui: &mut egui::Ui, action: IconAction, enabled: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(22.0, 20.0), Sense::click());
    let hovered = enabled && response.hovered();
    if hovered {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(5), T::SURFACE_3);
    }
    let color = if !enabled {
        T::TEXT_FAINT
    } else if hovered {
        T::TEXT
    } else {
        T::TEXT_MUTED
    };
    match action {
        IconAction::Copy => paint_copy_icon(ui.painter(), rect, color),
        IconAction::Trash => paint_trash_icon(ui.painter(), rect, color),
    }
    if enabled {
        response.on_hover_cursor(CursorIcon::PointingHand)
    } else {
        response
    }
}

fn paint_copy_icon(painter: &egui::Painter, rect: Rect, color: Color32) {
    let body = Rect::from_center_size(rect.center() + vec2(0.0, 1.0), vec2(10.0, 12.0));
    painter.rect_stroke(
        body,
        CornerRadius::same(2),
        Stroke::new(1.2, color),
        StrokeKind::Inside,
    );
    let clip = Rect::from_center_size(body.center_top() + vec2(0.0, 1.0), vec2(5.0, 3.0));
    painter.rect_filled(clip, CornerRadius::same(1), color);
    for offset in [-2.0_f32, 2.0] {
        painter.line_segment(
            [
                pos2(body.left() + 2.5, body.center().y + offset),
                pos2(body.right() - 2.5, body.center().y + offset),
            ],
            Stroke::new(1.0, color),
        );
    }
}

fn paint_trash_icon(painter: &egui::Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    painter.line_segment(
        [
            pos2(center.x - 6.0, center.y - 5.0),
            pos2(center.x + 6.0, center.y - 5.0),
        ],
        Stroke::new(1.3, color),
    );
    painter.line_segment(
        [
            pos2(center.x - 2.5, center.y - 7.0),
            pos2(center.x + 2.5, center.y - 7.0),
        ],
        Stroke::new(1.3, color),
    );
    painter.line_segment(
        [
            pos2(center.x - 4.5, center.y - 3.0),
            pos2(center.x - 3.5, center.y + 7.0),
        ],
        Stroke::new(1.2, color),
    );
    painter.line_segment(
        [
            pos2(center.x + 4.5, center.y - 3.0),
            pos2(center.x + 3.5, center.y + 7.0),
        ],
        Stroke::new(1.2, color),
    );
    painter.line_segment(
        [
            pos2(center.x - 3.5, center.y + 7.0),
            pos2(center.x + 3.5, center.y + 7.0),
        ],
        Stroke::new(1.2, color),
    );
    painter.line_segment(
        [
            pos2(center.x - 1.0, center.y - 1.0),
            pos2(center.x - 0.5, center.y + 4.0),
        ],
        Stroke::new(1.0, color),
    );
    painter.line_segment(
        [
            pos2(center.x + 1.0, center.y - 1.0),
            pos2(center.x + 0.5, center.y + 4.0),
        ],
        Stroke::new(1.0, color),
    );
}

fn paint_history_swatch(
    painter: &egui::Painter,
    rect: Rect,
    entry: Option<&HistoryEntry>,
    selected: Option<[u8; 4]>,
) {
    let radius = CornerRadius::same(7);
    if let Some(entry) = entry {
        paint_checker(painter, rect);
        painter.rect_filled(
            rect,
            radius,
            Color32::from_rgba_unmultiplied(
                entry.rgba[0],
                entry.rgba[1],
                entry.rgba[2],
                entry.rgba[3],
            ),
        );
        painter.rect_stroke(
            rect,
            radius,
            Stroke::new(
                if entry.locked { 1.8 } else { 1.0 },
                if entry.locked {
                    T::ACCENT
                } else {
                    T::BORDER_STRONG
                },
            ),
            StrokeKind::Inside,
        );
        if selected == Some(entry.rgba) {
            painter.rect_stroke(
                rect.expand(2.0),
                CornerRadius::same(9),
                Stroke::new(2.0, T::ACCENT),
                StrokeKind::Outside,
            );
        }
        if entry.locked {
            paint_lock_badge(painter, rect);
        }
    } else {
        painter.rect(
            rect,
            radius,
            T::SURFACE_2,
            Stroke::new(1.0, T::BORDER),
            StrokeKind::Inside,
        );
    }
}

fn paint_lock_badge(painter: &egui::Painter, rect: Rect) {
    let center = rect.right_bottom() + vec2(-6.0, -6.0);
    painter.circle_filled(center, 6.0, Color32::from_black_alpha(190));
    let body = Rect::from_center_size(center + vec2(0.0, 1.0), vec2(5.5, 4.0));
    painter.rect_filled(body, CornerRadius::same(1), T::ACCENT);
    let shackle = Rect::from_center_size(center + vec2(0.0, -1.8), vec2(3.2, 3.2));
    painter.line_segment(
        [
            pos2(shackle.left(), shackle.center().y),
            pos2(shackle.left(), shackle.top()),
        ],
        Stroke::new(0.9, T::ACCENT),
    );
    painter.line_segment(
        [
            pos2(shackle.left(), shackle.top()),
            pos2(shackle.right(), shackle.top()),
        ],
        Stroke::new(0.9, T::ACCENT),
    );
    painter.line_segment(
        [
            pos2(shackle.right(), shackle.top()),
            pos2(shackle.right(), shackle.center().y),
        ],
        Stroke::new(0.9, T::ACCENT),
    );
}

fn format_card(
    ui: &mut egui::Ui,
    format: OutputFormat,
    current: OutputFormat,
    width: f32,
    hint: &str,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(width, 50.0), Sense::click());
    let selected = format == current;
    let hovered = response.hovered();
    let fill = if selected {
        T::ACCENT_SOFT
    } else if hovered {
        T::SURFACE_3
    } else {
        T::SURFACE_2
    };
    let stroke = Stroke::new(
        1.0,
        if selected {
            T::ACCENT
        } else if hovered {
            T::BORDER_STRONG
        } else {
            T::BORDER
        },
    );
    ui.painter().rect(
        rect,
        CornerRadius::same(12),
        fill,
        stroke,
        StrokeKind::Inside,
    );
    ui.painter().text(
        rect.left_top() + vec2(10.0, 10.0),
        Align2::LEFT_TOP,
        format.label(),
        FontId::proportional(13.5),
        T::TEXT,
    );
    ui.painter().text(
        rect.left_bottom() + vec2(10.0, -10.0),
        Align2::LEFT_BOTTOM,
        hint,
        FontId::proportional(9.5),
        if selected { T::ACCENT } else { T::TEXT_FAINT },
    );
    response.on_hover_cursor(CursorIcon::PointingHand)
}

fn small_text_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
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

fn hline(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, T::BORDER);
}

fn clipboard_png_size(image: &RgbaImage) -> u64 {
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

fn upload_texture(ctx: &egui::Context, name: &str, image: &RgbaImage) -> TextureHandle {
    let color_image = egui::ColorImage::from_rgba_unmultiplied(
        [image.width() as usize, image.height() as usize],
        image.as_raw(),
    );
    ctx.load_texture(name.to_owned(), color_image, TextureOptions::LINEAR)
}

fn load_texture_from_png(ctx: &egui::Context, name: &str, bytes: &[u8]) -> Option<TextureHandle> {
    let image = image::load_from_memory_with_format(bytes, ImageFormat::Png)
        .ok()?
        .to_rgba8();
    Some(upload_texture(ctx, name, &image))
}

fn resized_for_ui(image: &RgbaImage, max_side: u32) -> RgbaImage {
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

fn fit_rect(image_size: Vec2, area: Rect) -> Rect {
    if image_size.x <= 0.0 || image_size.y <= 0.0 {
        return area;
    }
    let scale = (area.width() / image_size.x).min(area.height() / image_size.y);
    Rect::from_center_size(area.center(), image_size * scale)
}

fn paint_checker(painter: &egui::Painter, rect: Rect) {
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

fn paint_badge(
    painter: &egui::Painter,
    anchor: egui::Pos2,
    align: Align2,
    text: &str,
    background: Color32,
    foreground: Color32,
) {
    let galley = painter.layout_no_wrap(text.to_owned(), FontId::proportional(9.0), foreground);
    let size = galley.size() + vec2(16.0, 9.0);
    let rect = align.anchor_size(anchor, size);
    painter.rect_filled(rect, CornerRadius::same(7), background);
    painter.galley(rect.min + vec2(8.0, 4.5), galley, foreground);
}

fn safe_stem(value: &str) -> String {
    let mut clean = value
        .replace(&['<', '>', ':', '"', '/', '\\', '|', '?', '*'][..], "-")
        .trim()
        .trim_end_matches(&['.', ' '][..])
        .to_owned();
    if clean.len() > 120 {
        clean.truncate(120);
        clean = clean.trim_end().to_owned();
    }
    let reserved = [
        "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
        "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
    ];
    if clean.is_empty() || reserved.contains(&clean.to_lowercase().as_str()) {
        "imagen".to_owned()
    } else {
        clean
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_cursor_is_limited_to_the_line_and_handle() {
        let image = Rect::from_min_max(pos2(0.0, 0.0), pos2(200.0, 100.0));
        assert!(pointer_over_split(image, 100.0, pos2(100.0, 10.0)));
        assert!(pointer_over_split(image, 100.0, pos2(108.0, 10.0)));
        assert!(pointer_over_split(image, 100.0, pos2(118.0, 50.0)));
        assert!(!pointer_over_split(image, 100.0, pos2(121.0, 50.0)));
        assert!(!pointer_over_split(image, 100.0, pos2(100.0, -1.0)));
        assert!(!pointer_over_split(image, 100.0, pos2(70.0, 50.0)));
    }
}
