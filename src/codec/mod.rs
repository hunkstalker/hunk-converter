use std::path::Path;

use image::{ImageFormat, RgbaImage, imageops};

use crate::domain::error::{EncodeError, LoadError};

mod encoders;
mod ico;
mod svg;

pub use crate::domain::format::{OutputFormat, format_bytes};

pub const PREVIEW_MAX_SIDE: u32 = 2048;
pub const MAX_SOURCE_SIDE: u32 = 16_384;
pub const MAX_SOURCE_PIXELS: u32 = 40_000_000;
pub const ICO_SIZES: [u32; 6] = [16, 32, 48, 64, 128, 256];
const MAX_SOURCE_BYTES: u64 = 200 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct EncodeOptions {
    pub format: OutputFormat,
    pub quality: u8,
    pub ico_sizes: Vec<u32>,
    pub background: [u8; 4],
}

pub struct EncodedOutput {
    pub bytes: Vec<u8>,
    pub preview: RgbaImage,
    pub width: u32,
    pub height: u32,
    pub frame_count: usize,
    pub ico_sizes: Vec<u32>,
}

pub fn encode(source: &RgbaImage, options: &EncodeOptions) -> Result<EncodedOutput, EncodeError> {
    if source.width() == 0 || source.height() == 0 {
        return Err(EncodeError::InvalidDimensions);
    }
    let quality = options.quality.clamp(1, 100);

    let mut result = match options.format {
        OutputFormat::Webp => encoders::encode_webp(source, quality)?,
        OutputFormat::Jpg => encoders::encode_jpg(source, quality, options.background)?,
        OutputFormat::Png => encoders::encode_png(source)?,
        OutputFormat::Avif => encoders::encode_avif(source, quality)?,
        OutputFormat::Jxl => encoders::encode_jxl(source)?,
        OutputFormat::Ico => {
            let ico = ico::encode_ico(source, &options.ico_sizes)?;
            EncodedOutput {
                bytes: ico.bytes,
                preview: ico.preview,
                width: ico.width,
                height: ico.height,
                frame_count: ico.sizes.len(),
                ico_sizes: ico.sizes,
            }
        }
    };

    result.preview = bounded_preview(&result.preview);
    Ok(result)
}

pub struct LoadedImage {
    pub pixels: RgbaImage,
    pub format: String,
    pub file_bytes: u64,
}

pub fn load_image_file(path: &Path) -> Result<LoadedImage, LoadError> {
    let metadata = std::fs::metadata(path).map_err(|source| LoadError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    if metadata.len() > MAX_SOURCE_BYTES {
        return Err(LoadError::TooLarge {
            bytes: metadata.len(),
            limit: MAX_SOURCE_BYTES,
        });
    }

    let bytes = std::fs::read(path).map_err(|source| LoadError::Read {
        path: path.to_path_buf(),
        source,
    })?;

    if path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("svg"))
    {
        let pixels = svg::decode_svg(&bytes, path)?;
        return Ok(LoadedImage {
            pixels,
            format: "SVG".to_owned(),
            file_bytes: bytes.len() as u64,
        });
    }

    let guessed = image::guess_format(&bytes)
        .or_else(|_| ImageFormat::from_path(path))
        .map_err(|_| LoadError::Unsupported)?;
    let decoded = image::load_from_memory_with_format(&bytes, guessed).map_err(|source| {
        LoadError::Decode {
            source: Box::new(source),
        }
    })?;
    let pixels = decoded.to_rgba8();
    validate_dimensions(pixels.width(), pixels.height())?;

    Ok(LoadedImage {
        pixels,
        format: format_name(guessed).to_owned(),
        file_bytes: bytes.len() as u64,
    })
}

fn validate_dimensions(width: u32, height: u32) -> Result<(), LoadError> {
    if width == 0 || height == 0 {
        return Err(LoadError::InvalidDimensions);
    }
    if width > MAX_SOURCE_SIDE
        || height > MAX_SOURCE_SIDE
        || u64::from(width) * u64::from(height) > u64::from(MAX_SOURCE_PIXELS)
    {
        return Err(LoadError::DimensionsLimit);
    }
    Ok(())
}

fn bounded_preview(image: &RgbaImage) -> RgbaImage {
    let longest = image.width().max(image.height());
    if longest <= PREVIEW_MAX_SIDE {
        return image.clone();
    }
    let scale = PREVIEW_MAX_SIDE as f32 / longest as f32;
    imageops::resize(
        image,
        ((image.width() as f32 * scale).round() as u32).max(1),
        ((image.height() as f32 * scale).round() as u32).max(1),
        imageops::FilterType::Triangle,
    )
}

fn format_name(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Png => "PNG",
        ImageFormat::Jpeg => "JPG",
        ImageFormat::WebP => "WEBP",
        ImageFormat::Bmp => "BMP",
        ImageFormat::Tga => "TGA",
        _ => "IMG",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn noise(width: u32, height: u32) -> RgbaImage {
        let mut seed = 0xA11CE_u32;
        RgbaImage::from_fn(width, height, |_, _| {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            Rgba([
                (seed & 0xFF) as u8,
                ((seed >> 8) & 0xFF) as u8,
                ((seed >> 16) & 0xFF) as u8,
                255,
            ])
        })
    }

    #[test]
    fn renders_svg() {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="40"><rect width="64" height="40" fill="#ff0000"/></svg>"##;
        let image = svg::decode_svg(svg, Path::new("test.svg")).unwrap();
        assert_eq!(image.dimensions(), (64, 40));
        let pixel = image.get_pixel(32, 20);
        assert!(pixel[0] > 240 && pixel[1] < 10 && pixel[2] < 10);
    }

    #[test]
    fn encodes_supported_formats() {
        let source = noise(120, 80);
        for format in [
            OutputFormat::Webp,
            OutputFormat::Jpg,
            OutputFormat::Png,
            OutputFormat::Avif,
            OutputFormat::Jxl,
        ] {
            let output = encode(
                &source,
                &EncodeOptions {
                    format,
                    quality: 82,
                    ico_sizes: Vec::new(),
                    background: [255, 255, 255, 255],
                },
            )
            .unwrap();
            assert!(!output.bytes.is_empty(), "{format:?}");
            assert_eq!(output.width, 120);
            assert_eq!(output.height, 80);
        }
    }

    #[test]
    fn quality_affects_lossy_size() {
        let source = noise(256, 256);
        let low = encode(
            &source,
            &EncodeOptions {
                format: OutputFormat::Webp,
                quality: 20,
                ico_sizes: Vec::new(),
                background: [255; 4],
            },
        )
        .unwrap();
        let high = encode(
            &source,
            &EncodeOptions {
                format: OutputFormat::Webp,
                quality: 92,
                ico_sizes: Vec::new(),
                background: [255; 4],
            },
        )
        .unwrap();
        assert!(low.bytes.len() < high.bytes.len());
    }

    #[test]
    fn jxl_preserves_alpha() {
        let mut source = RgbaImage::new(8, 8);
        source.put_pixel(2, 2, Rgba([255, 0, 0, 128]));
        let output = encode(
            &source,
            &EncodeOptions {
                format: OutputFormat::Jxl,
                quality: 82,
                ico_sizes: Vec::new(),
                background: [255; 4],
            },
        )
        .unwrap();
        assert_eq!(output.preview.get_pixel(2, 2).0[3], 128);
    }

    #[test]
    fn png_preserves_alpha() {
        let mut source = RgbaImage::new(8, 8);
        source.put_pixel(2, 2, Rgba([255, 0, 0, 128]));
        let output = encode(
            &source,
            &EncodeOptions {
                format: OutputFormat::Png,
                quality: 82,
                ico_sizes: Vec::new(),
                background: [255; 4],
            },
        )
        .unwrap();
        let decoded = image::load_from_memory(&output.bytes).unwrap().to_rgba8();
        assert_eq!(decoded.get_pixel(2, 2).0[3], 128);
    }
}
