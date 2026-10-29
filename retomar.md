# Retomar hunk-converter

> Punto de entrada para nuevas sesiones. Última actualización: **2026-10-29**.
> Versión: **0.2.0** · Rama: **main** · Estado: **aplicación principal Rust + egui, portable**.

## 1. Qué es este proyecto

**hunk-converter** es una aplicación de escritorio local-first para convertir, comprimir, comparar y exportar imágenes. Está hecha en **Rust + egui**, usa el renderizador **glow** y no requiere instalador.

Puede exportar WebP, AVIF, JPG, PNG, JPEG XL e ICO. La vista previa compara el original con el archivo codificado real. Incluye un cuentagotas que obtiene colores de la imagen y muestra sus códigos HEX/RGB/RGBA/HSL. Los resultados se guardan en `export/`, junto al ejecutable.

Ruta principal:

```text
/workspace/hunk-converter/
```

El prototipo Godot original está archivado en `/workspace/Prototipo/`. No se debe seguir desarrollando allí.

## 2. Estado actual

- Interfaz completa en español y tema oscuro propio.
- Carga por arrastre, selector nativo o argumento de línea de comandos.
- Entrada: PNG, JPG, WebP, BMP, TGA y SVG.
- Salida: WebP, AVIF, JPG, PNG, JPEG XL e ICO.
- Calidad 1–100 para WebP/JPG/AVIF; compresión 0–100 para PNG y JPEG XL sin pérdida; JPG aplanado sobre blanco.
- Comparador visual con divisor, damero, zoom con rueda y paneo con clic derecho.
- Zoom de 100 % a 800 % anclado al cursor; clic en el porcentaje para volver al ajuste.
- Cuentagotas con muestra del píxel original, coordenadas y códigos HEX/RGB/RGBA/HSL; copia al portapapeles.
- Exportación directa a `export/`, sin sobrescribir y con nombres saneados.
- Procesamiento en hilo aparte con debounce y descarte de resultados obsoletos.
- Reinicio, atajos y avisos de error.
- **Render bajo demanda:** sin repintado de UI cuando la ventana está quieta.
- Builds portables de Linux y Windows; assets y fuente embebidos.
- **10 pruebas Rust**, clippy y formato limpios.
- CI de Rust en GitHub Actions.
- Benchmark comparativo completo en `docs/benchmark-2026-10-28.md`.

## 3. Comandos

```bash
cargo run --release -- /ruta/a/imagen.jpg
cargo run --release

cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check

# Demo y capturas
cargo run --release --bin make_demo -- /tmp/muestra.png 4000 3000
cargo run --release --bin bench -- /tmp/muestra.png
HUNK_DEMO=1 HUNK_SCREENSHOT=docs/hunk-converter-editor.png \
  cargo run --release
```

Builds portables:

```bash
./scripts/build_dist.sh linux
./scripts/build_dist.sh windows
./scripts/build_dist.sh all
```

- Linux usa las bibliotecas gráficas del sistema (X11/OpenGL).
- Windows se compila por defecto con `x86_64-pc-windows-gnu`; requiere target de Rust y toolchain/linker de Windows. El script genera zip si `zip` está disponible.
- La build Windows se genera, pero no se ha ejecutado en un Windows real (`t-23`).

## 4. Decisiones que no se deben romper

- **Salida portable:** en una build, `<carpeta del ejecutable>/export/`; en desarrollo, `target/release/export/`.
- **No sobrescribir:** `foto.webp`, `foto-2.webp`, etc.
- **Vista previa fiel:** decodificar el buffer final, no estimar.
- **Cuentagotas:** icono de pipeta; un clic activa o desactiva el modo y también se puede arrastrar desde el icono hasta la imagen; muestrea el píxel original; no mueve el divisor; la marca solo se ve mientras está activo; se cancela con `Esc`; se desactiva en ICO; limpia la muestra al cambiar de imagen o reiniciar.
- **Comparador:** el cursor de redimensionado solo aparece sobre la línea o el círculo del divisor. La rueda hace zoom (100–800 %) y el clic derecho desplaza la imagen cuando está ampliada.
- **Vista:** el zoom se reinicia al cargar otra imagen o reiniciar; el encuadre se conserva al cambiar de formato.
- **JPG:** transparencia compuesta sobre blanco.
- **ICO:** frames PNG, contenedor manual, ajuste centrado sin recorte, al menos un tamaño.
- **Calidad inicial:** 82 %; compresión inicial: 70 %.
- **PNG y JPEG XL:** sin pérdida; sus deslizadores controlan compresión/esfuerzo, no calidad.
- **AVIF:** salida con pérdida y control de calidad; vista previa decodificada con `avif-decode`.
- **Badges del comparador:** el formato de salida va a la izquierda y `ORIGINAL` a la derecha, igual que las mitades visibles.
- **Reinicio:** restablece formato, calidad, tamaños y muestra de color.
- **Privacidad:** sin red, cuentas ni telemetría.
- **Render:** no añadir animaciones permanentes ni bucles de repintado innecesarios. Cualquier animación debe justificar sus frames.
- **Tema:** todo el estilo vive en `src/theme.rs`; mantener la paleta y jerarquía.

## 5. Estructura

```text
src/main.rs                  eframe, ventana e icono
src/app.rs                   UI, estado, cuentagotas, hilos y exportación
src/core/codec.rs            Carga/decodificación/codificación
src/core/color.rs            Códigos de color y aplanado
src/core/ico.rs              Contenedor ICO
src/demo.rs                  Imagen sintética para demos/capturas
src/theme.rs                 Tema egui
src/bin/bench.rs             Benchmark de códecs
src/bin/make_demo.rs         Generador de muestra
scripts/build_dist.sh        Empaquetado portable
assets/                      Manrope, iconos y licencias
docs/                        Capturas e informe de benchmark
notes/                       Evidencia fechada
proyecto.md                 Documento de producto
todo.md                     Tareas pendientes
t-completadas.md            Tareas cerradas
```

## 6. Tareas

- Pendientes: `todo.md` (t-10 a t-18, t-22 a t-24).
- Cerradas: `t-completadas.md` (t-1 a t-9, t-19, t-20, t-21, t-25 y t-26).
- Evidencia: todas las notas de `notes/`, incluida la migración.
- **Regla:** si el usuario pregunta qué tareas hay, devolver toda la información de cada tarea pendiente de `todo.md`: identificador `t-N`, título y descripción completa. Al cerrar una tarea, moverla a `t-completadas.md` y actualizar este archivo.

Siguiente tarea recomendada: **t-23 — validar la build Windows en un equipo real**, porque ya existe el ejecutable y es la principal incógnita de distribución. Después, t-10 (EXIF) y t-22 (accesibilidad).

## 7. Benchmarks y límites

El informe completo está en `docs/benchmark-2026-10-28.md`.

- La versión Rust es mucho más pequeña que el prototipo Godot, arranca antes, usa menos RAM y es más rápida codificando.
- La CPU en reposo del prototipo Godot medida bajo llvmpipe no representa un equipo con GPU, pero sí evidencia el repintado continuo.
- La ventaja de velocidad no viene de paralelismo: los códecs principales trabajan en un solo núcleo.
- AVIF y JPEG XL no se incluyeron en la comparación de velocidad; solo WebP/JPG/PNG/ICO.
- No se midieron PSNR/SSIM.

Pendientes conocidos:

- EXIF y metadatos (`t-10`).
- Redimensión y lotes (`t-11`, `t-12`).
- Modo 1:1, botones/gestos de zoom y atajos de encuadre (`t-16`).
- HEIC (`t-15`); AVIF y JPEG XL ya están integrados como salida.
- Accesibilidad completa (`t-22`).
- Validación Windows (`t-23`) e icono del ejecutable (`t-24`).
- Build macOS (`t-17`).

## 8. Flujo de trabajo para la próxima sesión

1. Leer `retomar.md`, `proyecto.md` y `todo.md`.
2. Revisar `git status`, `notes/` y `docs/benchmark-2026-10-28.md`.
3. Ejecutar `cargo test` antes de tocar código.
4. Implementar una tarea pendiente.
5. Añadir pruebas y actualizar capturas si cambia la interfaz.
6. Mover la tarea a `t-completadas.md` y actualizar proyecto/retomar.
7. Crear una nota fechada en `notes/` con decisiones y verificación.
8. Hacer commit con un mensaje claro.
