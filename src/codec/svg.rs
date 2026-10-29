use std::path::Path;

use image::RgbaImage;

use crate::domain::error::LoadError;

use super::validate_dimensions;

pub(super) fn decode_svg(bytes: &[u8], path: &Path) -> Result<RgbaImage, LoadError> {
    let mut options = resvg::usvg::Options {
        resources_dir: path.parent().map(Path::to_path_buf),
        ..Default::default()
    };
    options.fontdb_mut().load_system_fonts();
    options
        .fontdb_mut()
        .load_font_data(include_bytes!("../../assets/fonts/Manrope.ttf").to_vec());

    let tree =
        resvg::usvg::Tree::from_data(bytes, &options).map_err(|source| LoadError::SvgParse {
            source: Box::new(source),
        })?;
    let size = tree.size().to_int_size();
    let width = size.width();
    let height = size.height();
    validate_dimensions(width, height)?;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height).ok_or(LoadError::SvgCanvas)?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );
    RgbaImage::from_raw(width, height, pixmap.take()).ok_or(LoadError::SvgInvalid)
}
