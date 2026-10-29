# 2026-10-28 — PoC Rust/egui, rendimiento y nuevo nombre

> Evidencia de sesión. No editar.

## 1. Nombre definitivo y peticiones nuevas

- La aplicación pasa a llamarse **hunk-converter**.
- Se actualizaron marca visible, títulos, documentación y nombres de builds.
- La carpeta local del repositorio es `Prototipo`.
- La nota anterior conserva el nombre provisional Lienzo como evidencia histórica.

Antes de iniciar la PoC se registraron dos peticiones:

- **t-15** se amplió a AVIF, HEIC y **JPEG XL**.
- **t-19** se añadió para el selector de color de fondo debajo de los formatos, anclado al pie, con códigos HEX, RGB, RGBA y HSL y copia al portapapeles.

## 2. PoC creada

Ruta: `poc-rust-egui/`.

Stack:

- Rust estable 1.99.1.
- egui/eframe 0.37 con renderizador **glow**.
- `image` 0.25.10 para decodificación y salida PNG/JPG.
- `webp` 0.3.1 / libwebp para WebP.
- ICO escrito a mano con frames PNG.
- `rfd` para diálogo nativo.
- `std::thread` + canal para sacar la codificación del hilo de interfaz.

Funciones validadas:

- Arrastrar y soltar y apertura por argumento.
- PNG/JPG/WebP/BMP/TGA de entrada.
- WebP/JPG/PNG/ICO de salida.
- Comparador antes/después con texturas y damero.
- Modo ICO.
- Selector de color con **HEX, RGB, RGBA y HSL**, copia al portapapeles y uso como fondo al aplanar JPG.
- Exportación directa a `export/` junto al binario, con nombres únicos.
- Atajos `Ctrl+O`, `Ctrl+E`, `Ctrl+R`.
- Captura de pantalla interna para documentación con `HUNK_POC_SCREENSHOT`.

## 3. Verificación

- `cargo test`: **7 pruebas, 0 fallos**.
- `cargo clippy --all-targets -- -D warnings`: limpio.
- `cargo fmt --all -- --check`: limpio.
- Ejecución con Xvfb + llvmpipe + OpenGL.
- Exportación mediante `Ctrl+E` verificada: el archivo apareció en `poc-rust-egui/target/release/export/`.
- Capturas:
  - `docs/hunk-converter-rust-poc.png`.
  - `docs/hunk-converter-rust-poc-ico.png`.

## 4. Benchmark comparativo

Entorno: Debian 12, Rust 1.99.1, egui 0.37, Godot 4.7.2, OpenGL por software (llvmpipe) bajo Xvfb. Imagen sintética de 4000×3000 (12 MP), PNG de 19,08 MB. Una ejecución orientativa.

| Métrica | Godot 0.1.0 | PoC Rust/egui |
| --- | ---: | ---: |
| Ejecutable Linux | 70,7 MiB | 7,5 MiB |
| Carga + decode | 200,4 ms | 88,2 ms |
| WebP q82 | 1169,3 ms / 88,9 KB | 754,5 ms / 78,7 KB |
| JPG q82 | 1183,8 ms / 604,7 KB | 398,7 ms / 700,9 KB |
| PNG | 4713,2 ms / 12,24 MB | 3255,0 ms / 12,36 MB |
| ICO 16–256 | 735,5 ms / 43,2 KB | 496,0 ms / 43,1 KB |
| Pico de memoria | ~318 MB | ~230 MB |
| RSS en reposo | ~277 MB | ~102 MB |
| RSS con imagen | ~303 MB | ~130 MB |

Notas:

- Los bytes de salida no son idénticos; la comparación mide orden de magnitud de recursos, no equivalencia exacta de archivos.
- La mejora de CPU es significativa, no de un orden de magnitud: ambos usan códecs nativos.
- El gran salto está en tamaño de binario, memoria y control del empaquetado.
- AVIF/JPEG XL no se han integrado en la PoC.

## 5. Incidencias y correcciones

- Se corrigió la composición del panel lateral de la PoC para que el selector de color y la exportación queden siempre visibles; en tamaños insuficientes, la zona superior tiene scroll.
- Se pintó el indicador local de la cabecera en lugar de usar un glifo, evitando problemas de fuente.
- La ventana mínima de la PoC se fijó en 1024×800 para que el panel completo sea legible.
- Godot y la PoC requirieron bibliotecas locales de X11/GL para las capturas; es una limitación del entorno de pruebas, no del proyecto.
- El color de fondo de JPG en Godot sigue siendo blanco; el selector existe solo en la PoC hasta cerrar t-19.

## 6. Estado Git

- Prototipo Godot renombrado y documentado.
- PoC añadida bajo `poc-rust-egui/`.
- `poc-rust-egui/target/` ignorado.
- `Cargo.lock` generado para la PoC.
- Artefactos `dist/` y el zip local del usuario no forman parte del código fuente.

## 7. Siguiente paso

Decidir si se continúa con Godot o se migra el producto completo a Rust/egui. La evidencia apoya la migración si la prioridad es reducir huella, pero la paridad funcional y el coste de AVIF/JPEG XL deben planificarse antes de retirar el prototipo actual.
