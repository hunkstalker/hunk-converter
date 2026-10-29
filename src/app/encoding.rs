use std::sync::Arc;
use std::sync::mpsc::{self, TryRecvError};
use std::time::{Duration, Instant};

use eframe::egui;

use crate::codec::{self, EncodeOptions};
use crate::domain::{AppError, OutputFormat};
use crate::i18n::K;

use super::HunkApp;
use super::state::{EncodeCompletion, EncodeJobResult, RenderedResult};

impl HunkApp {
    pub(crate) fn busy(&self) -> bool {
        self.worker.is_some() || self.pending_since.is_some()
    }

    pub(crate) fn request_encode(&mut self, ctx: &egui::Context, delay: Option<Duration>) {
        self.revision = self.revision.wrapping_add(1);
        self.pending_since = Some(Instant::now() + delay.unwrap_or(Duration::ZERO));
        self.screenshot_at = None;
        self.screenshot_requested = false;
        ctx.request_repaint();
    }

    pub(crate) fn maybe_start_worker(&mut self, ctx: &egui::Context) {
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
            self.pending_since = None;
            self.toast(self.t(K::SelectIcoSizeToast).to_owned(), true, 3);
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
            let _ = sender.send(EncodeJobResult {
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

    pub(crate) fn poll_worker(&mut self, ctx: &egui::Context) {
        let Some(receiver) = self.worker.as_ref() else {
            return;
        };
        match receiver.try_recv() {
            Ok(message) => {
                self.worker = None;
                if message.revision == self.revision && self.pending_since.is_none() {
                    match message.result {
                        Ok(output) => self.apply_output(
                            ctx,
                            EncodeCompletion {
                                output,
                                elapsed: message.elapsed,
                            },
                        ),
                        Err(error) => {
                            self.toast_error(&AppError::Encode(error), 4);
                        }
                    }
                }
                ctx.request_repaint();
            }
            Err(TryRecvError::Disconnected) => {
                self.worker = None;
                self.toast(self.t(K::WorkerInterrupted).to_owned(), true, 4);
                ctx.request_repaint();
            }
            Err(TryRecvError::Empty) => {
                ctx.request_repaint_after(Duration::from_millis(16));
            }
        }
    }

    fn apply_output(&mut self, ctx: &egui::Context, completion: EncodeCompletion) {
        let EncodeCompletion { output, elapsed } = completion;
        let preview = super::ui::widgets::upload_texture(
            ctx,
            &format!("result-{}", self.revision),
            &output.preview,
        );
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
}
