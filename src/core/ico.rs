use std::io::Cursor;

use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{ExtendedColorType, ImageEncoder, RgbaImage, imageops};

pub struct IcoOutput {
    pub bytes: Vec<u8>,
    pub preview: RgbaImage,
    pub sizes: Vec<u32>,
    pub width: u32,
    pub height: u32,
}

/// Builds an ICO container using PNG-compressed frames.
pub fn encode_ico(source: &RgbaImage, requested_sizes: &[u32]) -> Result<IcoOutput, String> {
    let mut sizes = requested_sizes
        .iter()
        .copied()
        .filter(|size| (16..=256).contains(size))
        .collect::<Vec<_>>();
    sizes.sort_unstable();
    sizes.dedup();

    if sizes.is_empty() {
        return Err("Selecciona al menos un tamaño de icono.".to_owned());
    }

    let mut frames: Vec<(u32, Vec<u8>)> = Vec::with_capacity(sizes.len());
    let mut largest_frame: Option<RgbaImage> = None;
    let mut largest_size = 0;

    for size in &sizes {
        let size = *size;
        let scale = (size as f32 / source.width() as f32).min(size as f32 / source.height() as f32);
        let target_width = ((source.width() as f32 * scale).round() as u32).max(1);
        let target_height = ((source.height() as f32 * scale).round() as u32).max(1);
        let resized = imageops::resize(
            source,
            target_width,
            target_height,
            imageops::FilterType::Lanczos3,
        );

        let mut frame = RgbaImage::new(size, size);
        let offset_x = ((size - target_width) / 2) as i64;
        let offset_y = ((size - target_height) / 2) as i64;
        imageops::overlay(&mut frame, &resized, offset_x, offset_y);

        let mut png = Vec::new();
        PngEncoder::new_with_quality(
            Cursor::new(&mut png),
            CompressionType::Default,
            FilterType::Adaptive,
        )
        .write_image(frame.as_raw(), size, size, ExtendedColorType::Rgba8)
        .map_err(|error| format!("No se pudo codificar el icono de {size} px: {error}"))?;

        frames.push((size, png));
        if size >= largest_size {
            largest_size = size;
            largest_frame = Some(frame);
        }
    }

    let header_size = 6 + 16 * frames.len();
    let mut bytes =
        Vec::with_capacity(header_size + frames.iter().map(|(_, png)| png.len()).sum::<usize>());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&(frames.len() as u16).to_le_bytes());

    let mut payload_offset = header_size as u32;
    for (size, png) in &frames {
        let dimension = if *size >= 256 { 0 } else { *size as u8 };
        bytes.push(dimension);
        bytes.push(dimension);
        bytes.push(0); // palette size
        bytes.push(0); // reserved
        bytes.extend_from_slice(&1u16.to_le_bytes()); // color planes
        bytes.extend_from_slice(&32u16.to_le_bytes()); // bits per pixel
        bytes.extend_from_slice(&(png.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&payload_offset.to_le_bytes());
        payload_offset += png.len() as u32;
    }
    for (_, png) in &frames {
        bytes.extend_from_slice(png);
    }

    let preview = largest_frame.ok_or_else(|| "No se pudo generar ningún icono.".to_owned())?;
    Ok(IcoOutput {
        bytes,
        preview,
        sizes,
        width: largest_size,
        height: largest_size,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    #[test]
    fn writes_valid_multisize_ico() {
        let mut source = RgbaImage::new(120, 64);
        image::imageops::overlay(
            &mut source,
            &RgbaImage::from_pixel(120, 64, Rgba([99, 200, 80, 255])),
            0,
            0,
        );
        let result = encode_ico(&source, &[256, 16, 32]).unwrap();
        assert_eq!(result.sizes, vec![16, 32, 256]);
        assert_eq!(result.width, 256);
        assert_eq!(result.bytes[0..4], [0, 0, 1, 0]);
        let count = u16::from_le_bytes([result.bytes[4], result.bytes[5]]) as usize;
        assert_eq!(count, 3);

        for index in 0..count {
            let entry = 6 + index * 16;
            let dimension = result.bytes[entry];
            let size = if dimension == 0 {
                256
            } else {
                dimension as usize
            };
            let length = u32::from_le_bytes([
                result.bytes[entry + 8],
                result.bytes[entry + 9],
                result.bytes[entry + 10],
                result.bytes[entry + 11],
            ]) as usize;
            let offset = u32::from_le_bytes([
                result.bytes[entry + 12],
                result.bytes[entry + 13],
                result.bytes[entry + 14],
                result.bytes[entry + 15],
            ]) as usize;
            assert_eq!(&result.bytes[offset..offset + 4], b"\x89PNG");
            let decoded = image::load_from_memory(&result.bytes[offset..offset + length])
                .unwrap()
                .to_rgba8();
            assert_eq!(decoded.dimensions(), (size as u32, size as u32));
        }
    }

    #[test]
    fn rejects_empty_sizes() {
        let source = RgbaImage::new(16, 16);
        assert!(encode_ico(&source, &[]).is_err());
    }
}
