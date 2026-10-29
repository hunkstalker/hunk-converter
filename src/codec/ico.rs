use std::io::Cursor;

use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{ExtendedColorType, ImageEncoder, RgbaImage, imageops};

use crate::domain::error::IcoError;

pub struct IcoOutput {
    pub bytes: Vec<u8>,
    pub preview: RgbaImage,
    pub sizes: Vec<u32>,
    pub width: u32,
    pub height: u32,
}

pub fn encode_ico(source: &RgbaImage, requested_sizes: &[u32]) -> Result<IcoOutput, IcoError> {
    let mut sizes = requested_sizes
        .iter()
        .copied()
        .filter(|size| (16..=256).contains(size))
        .collect::<Vec<_>>();
    sizes.sort_unstable();
    sizes.dedup();

    if sizes.is_empty() {
        return Err(IcoError::NoSizes);
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
        .map_err(|source| IcoError::Encode {
            size,
            source: Box::new(source),
        })?;

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
        bytes.push(0);
        bytes.push(0);
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&32u16.to_le_bytes());
        bytes.extend_from_slice(&(png.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&payload_offset.to_le_bytes());
        payload_offset += png.len() as u32;
    }
    for (_, png) in &frames {
        bytes.extend_from_slice(png);
    }

    let preview = largest_frame.ok_or(IcoError::Empty)?;
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

    #[test]
    fn writes_valid_multisize_ico() {
        let mut source = RgbaImage::new(120, 64);
        for (x, y, pixel) in source.enumerate_pixels_mut() {
            *pixel = image::Rgba([x as u8, y as u8, 128, 255]);
        }
        let output = encode_ico(&source, &[16, 32, 48]).unwrap();
        assert_eq!(output.frame_count_marker(), 3);
        assert_eq!(output.bytes[0..4], [0, 0, 1, 0]);
        assert_eq!(output.bytes[4], 3);
        assert_eq!(output.sizes, vec![16, 32, 48]);
        assert_eq!(output.preview.dimensions(), (48, 48));
    }

    #[test]
    fn rejects_empty_sizes() {
        let source = RgbaImage::new(16, 16);
        assert!(matches!(encode_ico(&source, &[]), Err(IcoError::NoSizes)));
        assert!(matches!(
            encode_ico(&source, &[8, 512]),
            Err(IcoError::NoSizes)
        ));
    }

    impl IcoOutput {
        fn frame_count_marker(&self) -> u16 {
            u16::from_le_bytes([self.bytes[4], self.bytes[5]])
        }
    }
}
