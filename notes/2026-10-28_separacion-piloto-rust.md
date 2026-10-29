# 2026-10-28 — Separación del piloto Rust/egui

> Evidencia de sesión. No editar.

## 1. Decisión

El piloto de Rust/egui se separó del repositorio Godot para que ambos proyectos
tengan carpetas y repositorios independientes:

```text
/workspace/Prototipo/          hunk-converter (Godot)
/workspace/Piloto-Rust-egui/   piloto Rust/egui
```

La separación conserva el historial del piloto mediante un split de Git y le da
un ciclo de vida propio.

## 2. Cambios en el repositorio Godot

- Eliminado `poc-rust-egui/` del árbol y del seguimiento de Git.
- Eliminado el job de CI de Rust; vive ahora en el repo del piloto.
- Movidas las capturas del piloto a su repositorio.
- Actualizados README, proyecto, retomar, tareas completadas y notas para
  referirse al repositorio hermano.
- Las builds portables de Godot no incluyen el piloto.

## 3. Cambios en el repositorio del piloto

- Historia filtrada del proyecto Rust conservada.
- Assets propios: Manrope, licencia OFL, iconos SVG/PNG/ICO.
- Licencia MIT propia.
- README, capturas, CI, `.editorconfig` y `retomar.md` propios.
- Nota de inicio en `notes/2026-10-28_inicio-piloto.md`.
- Verificado que compila y ejecuta sin depender del repositorio Godot.

## 4. Verificación

- `cargo test`: 7 pruebas superadas.
- `cargo clippy --all-targets -- -D warnings`: limpio.
- `cargo fmt --all -- --check`: limpio.
- Captura renderizada correctamente desde el nuevo repositorio.
- El ejecutable Linux conserva su ruta `export/` junto al binario.

## 5. Contexto

El prototipo Godot sigue siendo la aplicación de referencia. El piloto se evaluará
por separado para decidir si se acomete la migración completa. Las ideas de AVIF,
HEIC, JPEG XL y selector de color con múltiples códigos siguen registradas en el
`todo.md` del proyecto principal.
