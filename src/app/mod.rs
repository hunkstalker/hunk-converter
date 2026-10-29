use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use eframe::egui::{self, TextureHandle};

use crate::codec::{ICO_SIZES, OutputFormat};
use crate::domain::{AppError, HistoryEntry, clear_unlocked, remember_color, toggle_lock};
use crate::i18n::{self, K, Lang};
use crate::settings;
use crate::theme as T;

mod actions;
mod encoding;
mod export;
mod session;
mod state;
mod ui;

use actions::AppAction;
use state::{EncodeJobResult, RenderedResult, SourceImage, Toast};

pub const ENCODE_DELAY_MS: u64 = 160;
pub const UI_PREVIEW_MAX: u32 = 2048;
pub const MIN_ZOOM: f32 = 1.0;
pub const MAX_ZOOM: f32 = 8.0;

pub struct HunkApp {
    source: Option<SourceImage>,
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
    pan: egui::Vec2,
    revision: u64,
    pending_since: Option<Instant>,
    worker: Option<Receiver<EncodeJobResult>>,
    toast: Option<Toast>,
    screenshot_path: Option<PathBuf>,
    screenshot_at: Option<Instant>,
    screenshot_requested: bool,
    screenshot_delay: Duration,
}

impl HunkApp {
    pub fn new(cc: &eframe::CreationContext<'_>, settings: settings::Settings) -> Self {
        T::install(&cc.egui_ctx);

        let brand_icon = ui::widgets::load_texture_from_png(
            &cc.egui_ctx,
            "brand-icon",
            include_bytes!("../../assets/branding/icon_256.png"),
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
            pan: egui::Vec2::ZERO,
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
            if app.set_format(format) {
                app.request_encode(&cc.egui_ctx, None);
            }
        }

        app
    }

    fn t(&self, key: K) -> &'static str {
        i18n::t(self.language, key)
    }

    fn save_settings(&self) {
        if let Err(error) = (settings::Settings {
            language: self.language,
            history: self.history.clone(),
            maximized: self.maximized,
        })
        .save()
        {
            eprintln!("[config] {error}");
        }
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
        if remember_color(&mut self.history, rgba) {
            self.save_settings();
            true
        } else {
            self.toast(self.t(K::HistoryFull).to_owned(), true, 4);
            false
        }
    }

    fn toggle_history_lock(&mut self, index: usize) {
        let Some(locked) = toggle_lock(&mut self.history, index) else {
            return;
        };
        let key = if locked {
            K::ColorPinned
        } else {
            K::ColorUnpinned
        };
        self.toast(self.t(key).to_owned(), false, 2);
        self.save_settings();
    }

    fn clear_history(&mut self) {
        clear_unlocked(&mut self.history);
        self.save_settings();
        self.toast(self.t(K::HistoryCleared).to_owned(), false, 2);
    }

    fn selected_ico_sizes(&self) -> Vec<u32> {
        ICO_SIZES
            .iter()
            .zip(self.ico_selected)
            .filter_map(|(size, selected)| selected.then_some(*size))
            .collect()
    }

    fn can_export(&self) -> bool {
        !self.busy()
            && self
                .result
                .as_ref()
                .is_some_and(|result| result.format == self.format)
    }

    fn toast(&mut self, message: impl Into<String>, error: bool, seconds: u64) {
        self.toast = Some(Toast::new(message, error, Duration::from_secs(seconds)));
    }

    fn toast_error(&mut self, error: &AppError, seconds: u64) {
        let message = i18n::present_error(self.language, error);
        self.toast(message, true, seconds);
    }

    fn apply_actions(&mut self, ctx: &egui::Context, actions: Vec<AppAction>) {
        for action in actions {
            match action {
                AppAction::RequestEncode(delay) => self.request_encode(ctx, delay),
                AppAction::PersistSettings => self.save_settings(),
                AppAction::Export => self.export(),
                AppAction::OpenFileDialog => self.open_file_dialog(ctx),
                AppAction::Reset => self.reset(ctx),
                AppAction::RememberColor(rgba) => {
                    self.remember_color(rgba);
                }
                AppAction::ToggleHistoryLock(index) => self.toggle_history_lock(index),
                AppAction::ClearHistory => self.clear_history(),
                AppAction::CopyColor { label, text } => {
                    ctx.copy_text(text);
                    self.toast(i18n::copied_message(self.language, &label), false, 2);
                }
                AppAction::RequestRepaint => ctx.request_repaint(),
            }
        }
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
            screenshot_ctx.send_viewport_cmd(egui::ViewportCommand::Close);
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

        let mut actions = Vec::new();
        actions.extend(self.show_header(ui));
        self.show_footer(ui);
        if self.source.is_some() {
            actions.extend(self.show_sidebar(ui));
            actions.extend(self.show_preview(ui));
        } else {
            actions.extend(self.show_empty(ui));
        }
        self.apply_actions(&ctx, actions);
        self.show_toast(&ctx);
        self.maybe_screenshot(&ctx);
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        T::BG.to_normalized_gamma_f32()
    }
}
