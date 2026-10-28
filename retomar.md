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

## Medidas recientes

- Arranque a ventana: 173,1 ms frente a 239,8 ms de Godot.
- RAM en reposo: 100,9 MiB frente a 209,7 MiB.
- RAM con imagen 12 MP: 211,3 MiB frente a 316,9 MiB.
- WebP q82 12 MP: 743,9 ms frente a 1121 ms.
- JPG q82 12 MP: 380,8 ms frente a 1137 ms.
- Informe completo: `docs/benchmark-2026-10-28.md`.

## Decisiones y límites

- Se mantiene separado del prototipo Godot en `../Prototipo/`.
- La app principal sigue siendo la versión de referencia hasta decidir la migración.
- No incluye SVG, EXIF, accesibilidad completa, AVIF, HEIC ni JPEG XL.
- La ventana mínima es 1024 × 800.
- La exportación se escribe junto al ejecutable (`target/release/export/` en desarrollo).
- La build Windows se generó por compilación cruzada, pero no se ejecutó en un Windows real.

## Documentos

- `README.md`: alcance, medidas comparativas y funcionamiento.
- `docs/benchmark-2026-10-28.md`: mediciones completas de arranque, memoria, CPU, códecs y distribución.
- `../Prototipo/todo.md`: tareas del producto principal.
- `../Prototipo/retomar.md`: contexto del prototipo Godot.

## Builds

```bash
./scripts/build_dist.sh linux
./scripts/build_dist.sh windows
./scripts/build_dist.sh all
```

Los artefactos quedan en `dist/`. La build Linux fue validada; la Windows se
generó por compilación cruzada y no se ejecutó en Windows en esta sesión.
