# Retomar Piloto Rust/egui

> Repositorio piloto independiente de **hunk-converter**. Última actualización: 2026-10-28.

## Estado

- Ventana nativa con egui/eframe + glow.
- Carga PNG/JPG/WebP/BMP/TGA por arrastre o argumento.
- Salida WebP/JPG/PNG/ICO.
- Comparador, procesamiento en hilo aparte y exportación portable a `export/`.
- Selector de color de fondo con HEX/RGB/RGBA/HSL y copia al portapapeles.
- 7 pruebas Rust, clippy y rustfmt limpios.
- Capturas en `docs/`.

## Comandos

```bash
cargo run --release -- /ruta/a/imagen.jpg
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check

# Demostración y capturas
cargo run --release --bin make_demo -- /tmp/muestra.png 4000 3000
cargo run --release --bin bench -- /tmp/muestra.png
HUNK_POC_DEMO=1 HUNK_POC_SCREENSHOT=docs/hunk-converter-rust-poc.png \
  cargo run --release
```

## Decisiones y límites

- Se mantiene separado del prototipo Godot en `../Prototipo/`.
- La app principal sigue siendo la versión de referencia hasta decidir la migración.
- No incluye SVG, EXIF, accesibilidad completa, AVIF, HEIC ni JPEG XL.
- La ventana mínima es 1024 × 800.
- La exportación se escribe junto al ejecutable (`target/release/export/` en desarrollo).
- No hay build Windows propia todavía.

## Documentos

- `README.md`: alcance, medidas comparativas y funcionamiento.
- `../Prototipo/todo.md`: tareas del producto principal.
- `../Prototipo/retomar.md`: contexto del prototipo Godot.
