use std::io::Cursor;
use std::path::Path;

use image::{RgbaImage, imageops};

const SOURCE: &str = "assets/branding/hunk-converter.png";
const BRANDING: &str = "assets/branding";

fn main() {
    let source = image::open(SOURCE).expect("hunk-converter.png").to_rgba8();

    let icon_256 = resize(&source, 256);
    std::fs::write(
        Path::new(BRANDING).join("icon_256.png"),
        encode_png(&icon_256),
    )
    .expect("icon_256.png");

    let ico = hunk_converter::codec::encode_ico(&source, &[16, 24, 32, 48, 64, 128, 256])
        .expect("icon.ico");
    std::fs::write(Path::new(BRANDING).join("icon.ico"), ico.bytes).expect("icon.ico");

    let entries = [
        (*b"ic07", 128_u32),
        (*b"ic08", 256_u32),
        (*b"ic09", 512_u32),
        (*b"ic10", 1024_u32),
    ];
    let mut icns = Vec::new();
    icns.extend_from_slice(b"icns");
    icns.extend_from_slice(&0u32.to_be_bytes());
    for (kind, size) in entries {
        let png = encode_png(&resize(&source, size));
        icns.extend_from_slice(&kind);
        icns.extend_from_slice(&((png.len() + 8) as u32).to_be_bytes());
        icns.extend_from_slice(&png);
    }
    let length = icns.len() as u32;
    icns[4..8].copy_from_slice(&length.to_be_bytes());
    std::fs::write(Path::new(BRANDING).join("icon.icns"), icns).expect("icon.icns");

    println!("icon_256.png, icon.ico e icon.icns generados desde {SOURCE}");
}

fn resize(source: &RgbaImage, size: u32) -> RgbaImage {
    imageops::resize(source, size, size, imageops::FilterType::Lanczos3)
}

fn encode_png(image: &RgbaImage) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(Cursor::new(&mut bytes), image.width(), image.height());
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::High);
    let mut writer = encoder.write_header().expect("PNG header");
    writer.write_image_data(image.as_raw()).expect("PNG data");
    drop(writer);
    bytes
}
