use std::path::Path;

fn main() {
    let svg = std::fs::read("assets/branding/icon.svg").expect("icon.svg");
    let tree = resvg::usvg::Tree::from_data(&svg, &resvg::usvg::Options::default())
        .expect("parse icon.svg");
    let size = tree.size().to_int_size();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width(), size.height()).expect("pixmap");
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );
    let out = Path::new("assets/branding/icon_256.png");
    pixmap.save_png(out).expect("save icon");
    println!("{}", out.display());
}
