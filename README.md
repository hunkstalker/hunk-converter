# hunk-converter — PoC Rust/egui

Prueba de concepto **independiente** del prototipo Godot. Su objetivo es medir si
Rust + egui permite conservar la experiencia y reducir de forma clara el consumo
de recursos antes de decidir una migración.

![Vista de la PoC](../docs/hunk-converter-rust-poc.png)

## Qué valida

- Ventana nativa con egui/eframe y renderizador OpenGL (glow).
- Arrastrar y soltar imágenes sobre la ventana.
- Carga de PNG, JPG, WebP, BMP y TGA.
- Codificación WebP, JPG, PNG e ICO multirresolución.
- Comparador deslizante antes/después con damero para transparencia.
- Procesamiento de imagen fuera del hilo de interfaz.
- Selector de color de fondo con códigos **HEX, RGB, RGBA y HSL** y copia al
  portapapeles. Se aplica al aplanar transparencias en JPG.
- Exportación directa a `export/`, junto al ejecutable, con nombres únicos.
- Atajos: `Ctrl+O`, `Ctrl+E` y `Ctrl+R`.

## Medidas de la prueba

Entorno de la medición: Debian 12, Rust 1.99.1, egui/eframe 0.37, Godot 4.7.2,
renderizado por software con llvmpipe bajo Xvfb. Imagen sintética de
**4000 × 3000 px** (12 MP) y PNG de 19,08 MB. Una sola ejecución; son cifras de
referencia, no un benchmark estadístico.

| Métrica | Godot 0.1.0 | PoC Rust/egui | Diferencia |
| --- | ---: | ---: | ---: |
| Binario Linux | 70,7 MiB | 7,5 MiB | **≈ 90 % menor** |
| Carga + decode | 200,4 ms | 88,2 ms | **≈ 56 % menos** |
| WebP q82 | 1169,3 ms / 88,9 KB | 754,5 ms / 78,7 KB | **35 % menos tiempo** |
| JPG q82 | 1183,8 ms / 604,7 KB | 398,7 ms / 700,9 KB | **66 % menos tiempo** |
| PNG | 4713,2 ms / 12,24 MB | 3255,0 ms / 12,36 MB | **31 % menos tiempo** |
| ICO 16–256 | 735,5 ms / 43,2 KB | 496,0 ms / 43,1 KB | **33 % menos tiempo** |
| Pico de memoria (benchmark) | ≈ 318 MB | ≈ 230 MB | **28 % menos** |
| RAM en reposo (RSS) | ≈ 277 MB | ≈ 102 MB | **63 % menos** |
| RAM con imagen cargada (RSS) | ≈ 303 MB | ≈ 130 MB | **57 % menos** |

La RAM en reposo se midió con las dos aplicaciones abiertas y sin imagen; la
cargada, con la misma imagen de 1600 × 1000. En ambos casos se usó el mismo
servidor Xvfb y llvmpipe, por lo que el consumo absoluto es mayor que con una GPU
real; la comparación relativa es la que importa.

Conclusiones provisionales:

- La mejora de CPU en códecs va de uno a dos dígitos porcentuales o más según
  formato, pero no es un orden de magnitud: libwebp ya es nativo en ambos casos.
- La diferencia estructural está en **tamaño del ejecutable, arranque y memoria**.
- egui cubre sin problemas este tipo de interfaz: lienzo propio, arrastre, slider,
  drag & drop, selector de color y paneles.
- La migración solo tiene sentido si esos objetivos de huella son prioritarios.
  Si el objetivo principal fuera únicamente convertir más rápido, habría que
  optimizar códecs y paralelismo antes que cambiar de toolkit.

## Ejecutar

Requisitos: Rust 1.99.1 (fijado en `rust-toolchain.toml`), un compilador de C para `libwebp-sys` y, en
Linux, las bibliotecas gráficas de X11/Wayland habituales. En Windows hace falta
MSVC Build Tools o un toolchain GNU.

```bash
cargo run --release -- /ruta/a/imagen.jpg
# o abre la ventana y arrastra una imagen
cargo run --release
```

Para generar las capturas de documentación:

```bash
HUNK_POC_DEMO=1 HUNK_POC_SCREENSHOT=../docs/hunk-converter-rust-poc.png \
  cargo run --release
```

Para probar la vista ICO:

```bash
HUNK_POC_DEMO=1 HUNK_POC_FORMAT=ico \
  HUNK_POC_SCREENSHOT=../docs/hunk-converter-rust-poc-ico.png \
  cargo run --release
```

## Benchmark

```bash
cargo run --release --bin make_demo -- /tmp/muestra.png 4000 3000
cargo run --release --bin bench -- /tmp/muestra.png
```

El benchmark imprime el tiempo de cada formato, el tamaño resultante y el pico
de memoria (`VmHWM`) del proceso.

## Pruebas

```bash
cargo test
```

Cubre codificación de los cuatro formatos, calidad frente a tamaño,
transparencia PNG, aplanado JPG, estructura y frames ICO, y conversiones de
color.

## Limitaciones de esta PoC

- No incluye SVG.
- No respeta orientación EXIF.
- No conserva metadatos ni perfiles ICC.
- No incluye AVIF, HEIC ni JPEG XL; están registrados en `todo.md`.
- No tiene accesibilidad nativa completa.
- La lectura de la imagen inicial ocurre en el hilo de interfaz.
- La ventana mínima es 1024 × 800.
- Aún no hay build Windows de esta PoC ni empaquetado final.
- El tema y la composición buscan validar la viabilidad, no copiar píxel a píxel
  el prototipo Godot.

El prototipo Godot sigue siendo la versión de referencia hasta que se decida la
migración definitiva.
