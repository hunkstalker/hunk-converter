# 2026-10-28 — Migración a Rust + egui

> Evidencia de sesión. No editar.

## Decisión

La aplicación principal pasa a ser el proyecto Rust + egui. El prototipo Godot queda archivado en `../Prototipo/`, con una nota `ARCHIVADO.md`. La carpeta activa es `/workspace/hunk-converter`.

La decisión se apoya en el benchmark comparativo: enorme reducción de tamaño y memoria, arranque más rápido, codificación más rápida y, sobre todo, una UI event-driven sin repintado continuo a 60 FPS.

## Cambios de producto

- Nombre y versión: **hunk-converter 0.2.0**.
- Paquete Cargo y binario renombrados a `hunk-converter`.
- Ventana y pie actualizados; eliminada la etiqueta de PoC.
- **Cuentagotas** reemplaza al antiguo selector de color de fondo:
  - activa el modo desde el panel inferior;
  - clic o arrastre sobre la imagen;
  - muestrea el píxel original;
  - muestra coordenadas;
  - códigos HEX, RGB, RGBA y HSL;
  - copia al portapapeles;
  - marca visual sobre la imagen;
  - se desactiva en ICO y al cambiar de imagen.
- **SVG** añadido a la carga mediante `resvg`.
- Filtro de archivos actualizado.
- Env de desarrollo sin prefijo PoC:
  - `HUNK_DEMO`
  - `HUNK_SCREENSHOT`
  - `HUNK_SCREENSHOT_DELAY_MS`
  - `HUNK_FORMAT`
  - `HUNK_SIZE`
  - `HUNK_METRICS`
- Script de build y nombres de artefactos actualizados a `hunk-converter-0.2.0-*`.

## Verificación

- 8 pruebas Rust superadas, incluida rasterización SVG.
- Cuentagotas probado con clic real bajo Xvfb: muestreó el píxel 1142, 325, mostró #FFF2C7 y no movió el divisor.
- `cargo clippy --all-targets -- -D warnings` limpio.
- `cargo fmt --all -- --check` limpio.
- Builds Linux y Windows generadas en `dist/`.
- Exportación Linux verificada en su carpeta `export/`.
- La build Windows no se ejecutó en Windows real (t-23).

## Documentación

- `proyecto.md`, `README.md`, `retomar.md`, `todo.md` y `t-completadas.md` reescritos para la app Rust.
- Notas históricas conservadas.
- Informe de rendimiento en `docs/benchmark-2026-10-28.md`.
- Prototipo Godot marcado como archivado.

## Tareas

- t-19: cuentagotas — completada.
- t-21: migración — completada.
- t-22, t-23 y t-24: accesibilidad, validación Windows e icono nativo del ejecutable.
- t-10 a t-18: pendientes ya existentes, actualizadas para el contexto Rust.

## Siguiente paso

Validar el `.exe` en Windows y después abordar EXIF/accesibilidad. No añadir nuevas funcionalidades de interfaz sin mantener el repintado bajo demanda.
