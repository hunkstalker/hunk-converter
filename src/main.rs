#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use hunk_converter::app::HunkApp;
use hunk_converter::settings;

fn main() -> eframe::Result {
    let icon = window_icon();
    let inner_size = std::env::var("HUNK_SIZE")
        .ok()
        .and_then(|value| {
            let (width, height) = value.split_once('x')?;
            Some([
                width.trim().parse::<f32>().ok()?,
                height.trim().parse::<f32>().ok()?,
            ])
        })
        .unwrap_or([1280.0, 800.0]);
    let mut settings = settings::Settings::load();
    let windowed = std::env::var_os("HUNK_WINDOWED").is_some();
    let maximized = settings.maximized && !windowed;
    settings.maximized = maximized;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Hunk Converter · Image Toolbox")
            .with_inner_size(inner_size)
            .with_min_inner_size([1024.0, 800.0])
            .with_maximized(maximized)
            .with_icon(icon),
        renderer: eframe::Renderer::Glow,
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "hunk-converter",
        options,
        Box::new(move |cc| Ok(Box::new(HunkApp::new(cc, settings)))),
    )
}

fn window_icon() -> egui::IconData {
    let image = image::load_from_memory(include_bytes!("../assets/branding/icon_256.png"))
        .expect("icono de aplicación inválido")
        .to_rgba8();
    egui::IconData {
        rgba: image.into_raw(),
        width: 256,
        height: 256,
    }
}
