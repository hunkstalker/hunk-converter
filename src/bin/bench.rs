use std::path::PathBuf;
use std::time::Instant;

use hunk_converter::codec::{self, EncodeOptions, OutputFormat, format_bytes};

fn main() {
    let path = std::env::args()
        .skip(1)
        .find(|argument| !argument.starts_with("--"))
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            eprintln!("Uso: cargo run --release --bin bench -- /ruta/a/imagen.jpg");
            std::process::exit(2);
        });

    let load_started = Instant::now();
    let loaded = match codec::load_image_file(&path) {
        Ok(loaded) => loaded,
        Err(error) => {
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
    };
    let load_ms = load_started.elapsed().as_secs_f64() * 1000.0;
    println!(
        "Archivo: {} · {} {}×{} · {} · carga {:.1} ms",
        path.display(),
        loaded.format,
        loaded.pixels.width(),
        loaded.pixels.height(),
        format_bytes(loaded.file_bytes),
        load_ms
    );

    let cases = [
        ("WebP q82", OutputFormat::Webp, 82, Vec::new()),
        ("JPG q82", OutputFormat::Jpg, 82, Vec::new()),
        ("PNG", OutputFormat::Png, 82, Vec::new()),
        (
            "ICO 16–256",
            OutputFormat::Ico,
            82,
            codec::ICO_SIZES.to_vec(),
        ),
    ];

    for (label, format, quality, ico_sizes) in cases {
        let options = EncodeOptions {
            format,
            quality,
            ico_sizes,
            background: [255, 255, 255, 255],
        };
        let started = Instant::now();
        match codec::encode(&loaded.pixels, &options) {
            Ok(output) => {
                let elapsed = started.elapsed().as_secs_f64() * 1000.0;
                println!(
                    "{label:<11} {:>10}  {:.1} ms  preview {}×{}",
                    format_bytes(output.bytes.len() as u64),
                    elapsed,
                    output.preview.width(),
                    output.preview.height()
                );
            }
            Err(error) => println!("{label:<11} ERROR: {error}"),
        }
    }

    #[cfg(target_os = "linux")]
    if let Some(peak_kb) = peak_rss_kb() {
        println!(
            "Memoria pico del proceso: {:.1} MB",
            peak_kb as f64 / 1024.0
        );
    }
}

#[cfg(target_os = "linux")]
fn peak_rss_kb() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    status.lines().find_map(|line| {
        line.strip_prefix("VmHWM:")
            .and_then(|value| value.split_whitespace().next())
            .and_then(|value| value.parse().ok())
    })
}
