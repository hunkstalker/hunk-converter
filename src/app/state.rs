use std::sync::Arc;
use std::time::{Duration, Instant};

use eframe::egui::TextureHandle;
use image::RgbaImage;

use crate::codec::{EncodedOutput, OutputFormat};

pub(super) struct SourceImage {
    pub name: String,
    pub stem: String,
    pub format: String,
    pub file_bytes: u64,
    pub width: u32,
    pub height: u32,
    pub pixels: Arc<RgbaImage>,
    pub texture: TextureHandle,
}

pub(super) struct RenderedResult {
    pub texture: TextureHandle,
    pub bytes: Vec<u8>,
    pub size: u64,
    pub width: u32,
    pub height: u32,
    pub frame_count: usize,
    pub format: OutputFormat,
}

pub(crate) struct SourceInput {
    pub pixels: RgbaImage,
    pub name: String,
    pub stem: String,
    pub format: String,
    pub file_bytes: u64,
}

pub(super) struct EncodeJobResult {
    pub revision: u64,
    pub result: Result<EncodedOutput, crate::domain::EncodeError>,
    pub elapsed: Duration,
}

pub(super) struct EncodeCompletion {
    pub output: EncodedOutput,
    pub elapsed: Duration,
}

pub(super) struct Toast {
    pub message: String,
    pub error: bool,
    pub until: Instant,
}

impl Toast {
    pub fn new(message: impl Into<String>, error: bool, ttl: Duration) -> Self {
        Self {
            message: message.into(),
            error,
            until: Instant::now() + ttl,
        }
    }
}
