use std::path::PathBuf;

use hunk_converter::demo;

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("hunk-converter-demo.png"));
    let width = args
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(1600);
    let height = args
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(1000);
    let image = demo::landscape(width, height);
    if let Err(error) = image.save(&path) {
        eprintln!("No se pudo guardar {}: {error}", path.display());
        std::process::exit(1);
    }
    println!(
        "Muestra {}×{} guardada en {}",
        width,
        height,
        path.display()
    );
}
