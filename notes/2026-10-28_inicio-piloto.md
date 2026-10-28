# 2026-10-28 — Inicio del piloto Rust/egui

> Evidencia de sesión. No editar.

## Separación de repositorios

El piloto se extrajo del repositorio principal de hunk-converter a una carpeta
hermana independiente:

```text
/workspace/Prototipo/          app Godot
/workspace/Piloto-Rust-egui/   piloto Rust/egui
```

El nuevo repositorio conserva el historial filtrado del piloto, sus pruebas y
capturas. Se copiaron los assets necesarios (Manrope y el icono) para que compile
sin depender del repositorio Godot.

## Validación

- `cargo test`: 7 pruebas superadas.
- `cargo clippy --all-targets -- -D warnings`: limpio.
- `cargo fmt --all -- --check`: limpio.
- Exportación verificada con `Ctrl+E` en `target/release/export/`.
- Capturas: `docs/hunk-converter-rust-poc.png` y
  `docs/hunk-converter-rust-poc-ico.png`.

## Medidas de referencia

Con una imagen de 12 MP y renderizado por software bajo Xvfb:

- Binario Linux: 7,5 MiB.
- WebP q82: 754 ms.
- JPG q82: 399 ms.
- ICO 16–256: 496 ms.
- Pico de memoria: ~230 MB.
- RAM con imagen cargada: ~130 MB.

## Pendientes

- Integrar AVIF, HEIC y JPEG XL (t-15 del proyecto principal).
- Portar EXIF, SVG y accesibilidad si se decide continuar.
- Empaquetado Windows y distribución portable.
- Decidir la migración completa desde Godot.
