use std::io::Cursor;
use std::path::Path;

use image::codecs::avif::AvifEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::{ExtendedColorType, ImageEncoder, ImageFormat, RgbaImage, imageops};
use zune_core::bit_depth::BitDepth as ZuneBitDepth;
use zune_core::colorspace::ColorSpace;
use zune_core::options::EncoderOptions as ZuneEncoderOptions;
use zune_jpegxl::JxlSimpleEncoder;

use super::color::flatten_rgba;
use super::ico::{IcoOutput, encode_ico};

pub const PREVIEW_MAX_SIDE: u32 = 2048;
pub const MAX_SOURCE_SIDE: u32 = 16_384;
pub const MAX_SOURCE_PIXELS: u32 = 40_000_000;
pub const ICO_SIZES: [u32; 6] = [16, 32, 48, 64, 128, 256];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputFormat {
    Webp,
    Jpg,
    Png,
    Ico,
    Avif,
    Jxl,
}

impl OutputFormat {
    pub const ALL: [Self; 6] = [
        Self::Webp,
        Self::Jpg,
        Self::Png,
        Self::Avif,
        Self::Jxl,
        Self::Ico,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Webp => "WebP",
            Self::Jpg => "JPG",
            Self::Png => "PNG",
            Self::Ico => "ICO",
            Self::Avif => "AVIF",
            Self::Jxl => "JPEG XL",
        }
    }

    pub fn badge(self) -> &'static str {
        match self {
            Self::Webp => "WEBP",
            Self::Jpg => "JPG",
            Self::Png => "PNG",
            Self::Ico => "ICO",
            Self::Avif => "AVIF",
            Self::Jxl => "JXL",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            Self::Webp => "Equilibrio",
            Self::Jpg => "Compatible",
            Self::Png => "Sin pérdida",
            Self::Ico => "Windows",
            Self::Avif => "Moderno",
            Self::Jxl => "Sin pérdida",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::Webp => "webp",
            Self::Jpg => "jpg",
            Self::Png => "png",
            Self::Ico => "ico",
            Self::Avif => "avif",
            Self::Jxl => "jxl",
        }
    }

    pub fn is_lossy(self) -> bool {
        matches!(self, Self::Webp | Self::Jpg | Self::Avif)
    }
}

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

pub fn encode(source: &RgbaImage, options: &EncodeOptions) -> Result<EncodedOutput, String> {
    if source.width() == 0 || source.height() == 0 {
        return Err("La imagen no tiene dimensiones válidas.".to_owned());
    }
    let quality = options.quality.clamp(1, 100);

    let mut result = match options.format {
        OutputFormat::Webp => {
            let memory = webp::Encoder::from_rgba(source.as_raw(), source.width(), source.height())
                .encode(quality as f32);
            let bytes = memory.to_vec();
            let preview = decode_preview(ImageFormat::WebP, &bytes)?;
            EncodedOutput {
                bytes,
                preview,
                width: source.width(),
                height: source.height(),
                frame_count: 1,
                ico_sizes: Vec::new(),
            }
        }
        OutputFormat::Jpg => {
            let flattened = flatten_rgba(source, options.background);
            let mut bytes = Vec::new();
            JpegEncoder::new_with_quality(Cursor::new(&mut bytes), quality)
                .write_image(
                    flattened.as_raw(),
                    flattened.width(),
                    flattened.height(),
                    ExtendedColorType::Rgb8,
                )
                .map_err(|error| format!("No se pudo codificar el JPG: {error}"))?;
            let preview = decode_preview(ImageFormat::Jpeg, &bytes)?;
            EncodedOutput {
                bytes,
                preview,
                width: source.width(),
                height: source.height(),
                frame_count: 1,
                ico_sizes: Vec::new(),
            }
        }
        OutputFormat::Png => {
            let mut bytes = Vec::new();
            {
                let mut encoder =
                    png::Encoder::new(Cursor::new(&mut bytes), source.width(), source.height());
                encoder.set_color(png::ColorType::Rgba);
                encoder.set_depth(png::BitDepth::Eight);
                encoder.set_compression(png::Compression::Balanced);
                let mut writer = encoder
                    .write_header()
                    .map_err(|error| format!("No se pudo iniciar el PNG: {error}"))?;
                writer
                    .write_image_data(source.as_raw())
                    .map_err(|error| format!("No se pudo codificar el PNG: {error}"))?;
            }
            let preview = decode_preview(ImageFormat::Png, &bytes)?;
            EncodedOutput {
                bytes,
                preview,
                width: source.width(),
                height: source.height(),
                frame_count: 1,
                ico_sizes: Vec::new(),
            }
        }
        OutputFormat::Avif => {
            let mut bytes = Vec::new();
            AvifEncoder::new_with_speed_quality(Cursor::new(&mut bytes), 4, quality)
                .write_image(
                    source.as_raw(),
                    source.width(),
                    source.height(),
                    ExtendedColorType::Rgba8,
                )
                .map_err(|error| format!("No se pudo codificar el AVIF: {error}"))?;
            let preview = decode_avif_preview(&bytes)?;
            EncodedOutput {
                bytes,
                preview,
                width: source.width(),
                height: source.height(),
                frame_count: 1,
                ico_sizes: Vec::new(),
            }
        }
        OutputFormat::Jxl => {
            let jxl_options = ZuneEncoderOptions::new(
                source.width() as usize,
                source.height() as usize,
                ColorSpace::RGBA,
                ZuneBitDepth::Eight,
            );
            let mut bytes = Vec::new();
            JxlSimpleEncoder::new(source.as_raw(), jxl_options)
                .encode(&mut bytes)
                .map_err(|error| format!("No se pudo codificar el JPEG XL: {error}"))?;
            let preview = decode_jxl_preview(&bytes)?;
            EncodedOutput {
                bytes,
                preview,
                width: source.width(),
                height: source.height(),
                frame_count: 1,
                ico_sizes: Vec::new(),
            }
        }
        OutputFormat::Ico => {
            let IcoOutput {
                bytes,
                preview,
                sizes,
                width,
                height,
            } = encode_ico(source, &options.ico_sizes)?;
            EncodedOutput {
                bytes,
                preview,
                width,
                height,
                frame_count: sizes.len(),
                ico_sizes: sizes,
            }
        }
    };

    result.preview = bounded_preview(&result.preview);
    Ok(result)
}

fn decode_preview(format: ImageFormat, bytes: &[u8]) -> Result<RgbaImage, String> {
    image::load_from_memory_with_format(bytes, format)
        .map(|image| image.to_rgba8())
        .map_err(|error| format!("No se pudo decodificar la vista previa: {error}"))
}

fn u16_to_u8(value: u16) -> u8 {
    ((u32::from(value) * 255 + 32_767) / 65_535) as u8
}

fn decode_avif_preview(bytes: &[u8]) -> Result<RgbaImage, String> {
    let image = avif_decode::Decoder::from_avif(bytes)
        .and_then(|decoder| decoder.to_image())
        .map_err(|error| format!("No se pudo decodificar la vista previa AVIF: {error}"))?;

    let (width, height, pixels) = match image {
        avif_decode::Image::Rgba8(image) => {
            let pixels = image
                .buf()
                .iter()
                .flat_map(|pixel| [pixel.r, pixel.g, pixel.b, pixel.a])
                .collect();
            (image.width() as u32, image.height() as u32, pixels)
        }
        avif_decode::Image::Rgb8(image) => {
            let pixels = image
                .buf()
                .iter()
                .flat_map(|pixel| [pixel.r, pixel.g, pixel.b, 255])
                .collect();
            (image.width() as u32, image.height() as u32, pixels)
        }
        avif_decode::Image::Gray8(image) => {
            let pixels = image
                .buf()
                .iter()
                .flat_map(|pixel| {
                    let value = pixel.value();
                    [value, value, value, 255]
                })
                .collect();
            (image.width() as u32, image.height() as u32, pixels)
        }
        avif_decode::Image::Rgba16(image) => {
            let pixels = image
                .buf()
                .iter()
                .flat_map(|pixel| {
                    [
                        u16_to_u8(pixel.r),
                        u16_to_u8(pixel.g),
                        u16_to_u8(pixel.b),
                        u16_to_u8(pixel.a),
                    ]
                })
                .collect();
            (image.width() as u32, image.height() as u32, pixels)
        }
        avif_decode::Image::Rgb16(image) => {
            let pixels = image
                .buf()
                .iter()
                .flat_map(|pixel| {
                    [
                        u16_to_u8(pixel.r),
                        u16_to_u8(pixel.g),
                        u16_to_u8(pixel.b),
                        255,
                    ]
                })
                .collect();
            (image.width() as u32, image.height() as u32, pixels)
        }
        avif_decode::Image::Gray16(image) => {
            let pixels = image
                .buf()
                .iter()
                .flat_map(|pixel| {
                    let value = u16_to_u8(pixel.value());
                    [value, value, value, 255]
                })
                .collect();
            (image.width() as u32, image.height() as u32, pixels)
        }
    };

    RgbaImage::from_raw(width, height, pixels)
        .ok_or_else(|| "El AVIF no produjo una imagen válida.".to_owned())
}

fn decode_jxl_preview(bytes: &[u8]) -> Result<RgbaImage, String> {
    let decoder = jxl_oxide::integration::JxlDecoder::new(Cursor::new(bytes))
        .map_err(|error| format!("No se pudo decodificar la vista previa JPEG XL: {error}"))?;
    let image = image::DynamicImage::from_decoder(decoder)
        .map_err(|error| format!("No se pudo decodificar la vista previa JPEG XL: {error}"))?;
    Ok(image.to_rgba8())
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

pub struct LoadedImage {
    pub pixels: RgbaImage,
    pub format: String,
    pub file_bytes: u64,
}

pub fn load_image_file(path: &Path) -> Result<LoadedImage, String> {
    let metadata =
        std::fs::metadata(path).map_err(|error| format!("No se pudo leer el archivo: {error}"))?;
    if metadata.len() > 200 * 1024 * 1024 {
        return Err("La imagen supera el límite de 200 MB.".to_owned());
    }

    let bytes =
        std::fs::read(path).map_err(|error| format!("No se pudo leer el archivo: {error}"))?;

    if path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("svg"))
    {
        let pixels = decode_svg(&bytes, path)?;
        return Ok(LoadedImage {
            pixels,
            format: "SVG".to_owned(),
            file_bytes: bytes.len() as u64,
        });
    }

    let guessed = image::guess_format(&bytes)
        .or_else(|_| ImageFormat::from_path(path))
        .map_err(|_| "El archivo no parece una imagen compatible.".to_owned())?;
    let decoded = image::load_from_memory_with_format(&bytes, guessed)
        .map_err(|error| format!("No se pudo decodificar la imagen: {error}"))?;
    let pixels = decoded.to_rgba8();
    validate_dimensions(pixels.width(), pixels.height())?;

    Ok(LoadedImage {
        pixels,
        format: format_name(guessed).to_owned(),
        file_bytes: bytes.len() as u64,
    })
}

fn decode_svg(bytes: &[u8], path: &Path) -> Result<RgbaImage, String> {
    let mut options = resvg::usvg::Options {
        resources_dir: path.parent().map(Path::to_path_buf),
        ..Default::default()
    };
    options.fontdb_mut().load_system_fonts();
    options
        .fontdb_mut()
        .load_font_data(include_bytes!("../../assets/fonts/Manrope.ttf").to_vec());

    let tree = resvg::usvg::Tree::from_data(bytes, &options)
        .map_err(|error| format!("No se pudo interpretar el SVG: {error}"))?;
    let size = tree.size().to_int_size();
    let width = size.width();
    let height = size.height();
    validate_dimensions(width, height)?;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| "No se pudo crear el lienzo del SVG.".to_owned())?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );
    RgbaImage::from_raw(width, height, pixmap.take())
        .ok_or_else(|| "El SVG no produjo una imagen válida.".to_owned())
}

fn validate_dimensions(width: u32, height: u32) -> Result<(), String> {
    if width == 0 || height == 0 {
        return Err("La imagen no tiene dimensiones válidas.".to_owned());
    }
    if width > MAX_SOURCE_SIDE
        || height > MAX_SOURCE_SIDE
        || u64::from(width) * u64::from(height) > u64::from(MAX_SOURCE_PIXELS)
    {
        return Err("La imagen supera el límite de 40 MP o 16 384 px por lado.".to_owned());
    }
    Ok(())
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

pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / KB).replace('.', ",")
    } else {
        format!("{:.2} MB", bytes as f64 / MB).replace('.', ",")
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
        let image = decode_svg(svg, Path::new("test.svg")).unwrap();
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
