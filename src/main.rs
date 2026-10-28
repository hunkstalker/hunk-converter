#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use hunk_converter_poc::app::HunkApp;

fn main() -> eframe::Result {
    let icon = window_icon();
    let inner_size = std::env::var("HUNK_POC_SIZE")
        .ok()
        .and_then(|value| {
            let (width, height) = value.split_once('x')?;
            Some([
                width.trim().parse::<f32>().ok()?,
                height.trim().parse::<f32>().ok()?,
            ])
        })
        .unwrap_or([1280.0, 800.0]);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("hunk-converter · PoC Rust/egui")
            .with_inner_size(inner_size)
            .with_min_inner_size([1024.0, 800.0])
            .with_icon(icon),
        renderer: eframe::Renderer::Glow,
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "hunk-converter",
        options,
        Box::new(|cc| Ok(Box::new(HunkApp::new(cc)))),
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
