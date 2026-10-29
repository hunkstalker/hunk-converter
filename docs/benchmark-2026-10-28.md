# Benchmark comparativo — Godot vs Rust/egui

> Fecha: 2026-10-28. Mediciones orientativas de una sola sesión.
> Referencia histórica: hunk-converter 0.1.0 (Godot 4.7.2).
> Aplicación actual: hunk-converter **0.2.0** (Rust + egui).

## Resumen

| Métrica | Godot 0.1.0 | Rust/egui 0.2.0 | Diferencia |
| --- | ---: | ---: | ---: |
| Binario Linux | 70,7 MiB | **16,1 MiB** | **−77,3 %** |
| Binario Windows | 104,7 MiB | **14,7 MiB** | **−86,0 %** |
| Arranque a ventana (mediana, 20 runs) | 239,8 ms | **200,2 ms** | **−16,5 %** |
| RAM en reposo (RSS) | 209,7 MiB | **102,0 MiB** | **−51,4 %** |
| RAM con imagen 12 MP | 316,9 MiB | **212,6 MiB** | **−32,9 %** |
| Pico con GUI e imagen 12 MP | 447,5 MiB | **336,8 MiB** | **−24,7 %** |
| WebP q82, 12 MP | 1121,0 ms | **760,9 ms** | **−32,1 %** |
| JPG q82, 12 MP | 1137,0 ms | **400,3 ms** | **−64,8 %** |
| PNG 12 MP | 4723,4 ms | **3221,4 ms** | **−31,8 %** |
| ICO 16–256, 12 MP | 723,9 ms | **468,2 ms** | **−35,3 %** |
| CPU en reposo, 60 FPS software | ~1130 % | **0 %** | actividad continua vs. bajo demanda |

La CPU se expresa por proceso: 100 % = un núcleo. Las cifras de CPU en reposo
están muy influidas por usar llvmpipe sin GPU ni VSync; deben leerse como una
diferencia de arquitectura de repintado, no como vatios reales en un equipo de
escritorio.

La aplicación 0.2.0 añade rasterización SVG, cuentagotas, historial de
colores, tres idiomas, zoom, paneo y los códecs AVIF y JPEG XL. El binario es bastante mayor que el de la prueba de
concepto inicial, sobre todo por los decodificadores y codificadores de AVIF y
JPEG XL. Las cifras de esta página corresponden al paquete final, no al piloto
de 0.1.0.

## Entorno

- CPU: Intel Core i7-14700K, 28 hilos lógicos disponibles.
- RAM del sistema: 32 GiB.
- Debian 12 sobre WSL2, kernel `6.18.33.2-microsoft-standard-WSL2`.
- Xvfb 1400×1000×24, sin gestor de ventanas.
- Mesa llvmpipe 22.3.6; OpenGL por software, VSync no disponible.
- Godot 4.7.2.
- Rust 1.99.1, eframe/egui 0.37, renderizador glow.
- Build Godot: release portable con PCK embebido.
- Build Rust: release con LTO, `codegen-units=1`, `panic=abort` y strip.

Los benchmarks de códecs de Godot se ejecutaron con el binario del editor en
modo headless, porque el script de medición no forma parte de la build exportada.
Las mediciones de interfaz se hicieron con las builds portables finales.

## Distribución

| Artefacto | Godot | Rust/egui 0.2.0 | Reducción |
| --- | ---: | ---: | ---: |
| Ejecutable Linux | 74.171.392 B (70,7 MiB) | 16.866.416 B (16,1 MiB) | 77,3 % |
| Carpeta Linux | 74.178.860 B | 16.874.395 B | 77,3 % |
| `.tar.gz` Linux | 27,7 MiB | 7,2 MiB | 74,1 % |
| Ejecutable Windows | 109.834.960 B (104,7 MiB) | 15.420.416 B (14,7 MiB) | 86,0 % |
| Carpeta Windows | 109.842.428 B | 15.428.395 B | 86,0 % |
| `.zip` Windows | 36,9 MiB | 6,7 MiB | 82,0 % |

Dependencias:

- Linux: ambos binarios enlazan solo bibliotecas base del sistema (libc/libm y
  runtime), y cargan X11/OpenGL dinámicamente.
- Windows: el `.exe` de Rust importa únicamente DLLs del sistema; no necesita
  DLLs de MinGW. El `.exe` de Godot no se ejecutó en Windows.
- La aplicación Rust no necesita el runtime de Godot ni archivos externos de
  assets: la fuente, los iconos y el SVG se procesan con código incluido en el
  binario.

## Arranque

Tiempo desde `spawn` hasta que la ventana aparece en el árbol X11. Medición
externa con `xwininfo` y sondeo aproximado de 5 ms. No es tiempo hasta primer
frame pintado y no es un arranque en frío real.

| Serie | Godot | Rust/egui 0.2.0 | Diferencia |
| --- | ---: | ---: | ---: |
| Mediana, 20 arranques en caliente | 239,8 ms | **200,2 ms** | −16,5 % |
| Media | 243,5 ms | **202,5 ms** | −16,8 % |
| Desviación estándar | 8,6 ms | 9,9 ms | — |
| P95 | 262,1 ms | 217,9 ms | −16,9 % |
| Rango | 235,1–262,1 ms | 190,4–232,1 ms | — |
| Mediana tras expulsar el ejecutable de caché (10 runs) | 245,8 ms | **192,1 ms** | −21,8 % |

Muestras en caliente:

```text
Godot: 236,8 235,1 239,9 239,7 238,9 262,1 236,6 249,8 241,2 255,0 ms
Rust:  202,3 207,1 210,7 200,8 206,1 207,3 201,8 194,0 196,5 196,9 191,2 212,0 199,7 217,9 193,5 196,3 195,6 232,1 197,3 190,4 ms
```

Se hizo una segunda tanda expulsando solo el ejecutable con
`posix_fadvise(POSIX_FADV_DONTNEED)`. No equivale a un arranque en frío de todo
el sistema: las bibliotecas compartidas y el resto de la caché siguieron
calientes.

## Memoria y CPU con interfaz

Mediana de 3 ejecuciones. El escenario `12 MP` usa una imagen de 4000×3000; la
memoria se leyó tras 5 s, con la codificación ya terminada.

| Estado | Godot RSS | Rust RSS | Godot pico | Rust pico | Hilos G/R |
| --- | ---: | ---: | ---: | ---: | ---: |
| Vacío | 209,7 MiB | **102,0 MiB** | 209,7 MiB | **102,0 MiB** | 91 / 59 |
| 1600×1000 | 252,6 MiB | **132,2 MiB** | 252,6 MiB | **138,0 MiB** | 91 / 59 |
| 4000×3000 | 316,9 MiB | **212,6 MiB** | 447,5 MiB | **336,8 MiB** | 91 / 59 |

CPU en reposo, ventana visible sin interacción, 100 % = un núcleo:

| Escenario | Godot | Rust/egui |
| --- | ---: | ---: |
| Sin límite de FPS, llvmpipe | 1570,9 % (≈15,7 núcleos) | 0,0 % |
| Limitado a 60 FPS, llvmpipe | 1130 % (≈11,3 núcleos) | 0,0 % |

Godot repinta de forma continua. La aplicación egui es *event-driven* y no pide
frames cuando está quieta. Con una GPU real, el coste absoluto de Godot sería
mucho menor; la diferencia de comportamiento en reposo seguiría dependiendo del
renderizador. Los hilos incluyen los workers de llvmpipe, no solo la lógica de
la app.

## Codificación headless y escalado

Una ejecución por tamaño. Tiempos en milisegundos, incluida la generación de la
vista previa. La memoria es el RSS máximo de `/usr/bin/time`.

| Entrada | App | Decode | WebP | JPG | PNG | ICO | Total | Pico |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1200×800 (1 MP) | Godot | 15,9 | 88,7 | 143,6 | 539,6 | 89,1 | 1,11 s | 111,6 MiB |
| | Rust 0.2.0 | 11,2 | **54,9** | **21,1** | **246,5** | **39,3** | **0,40 s** | **18,9 MiB** |
| 2400×1800 (4 MP) | Godot | 71,0 | 488,4 | 684,5 | 2278,3 | 297,9 | 3,95 s | 206,4 MiB |
| | Rust 0.2.0 | 38,1 | **325,7** | **196,2** | **1226,9** | **153,4** | **1,97 s** | **119,5 MiB** |
| 4000×3000 (12 MP) | Godot | 198,3 | 1121,0 | 1137,0 | 4723,4 | 723,9 | 8,04 s | 317,1 MiB |
| | Rust 0.2.0 | 88,1 | **760,9** | **400,3** | **3221,4** | **468,2** | **4,97 s** | **231,8 MiB** |
| 6000×4000 (24 MP) | Godot | 396,4 | 2100,1 | 1616,6 | 8216,1 | 1438,6 | 13,99 s | 540,6 MiB |
| | Rust 0.2.0 | 195,1 | **1412,1** | **684,9** | **6360,5** | **956,7** | **9,65 s** | **353,4 MiB** |

Tiempo total del proceso (incluye arranque del motor/benchmark):

| Tamaño | Godot | Rust | Mejora |
| --- | ---: | ---: | ---: |
| 1 MP | 1,11 s | 0,40 s | 64,0 % |
| 4 MP | 3,95 s | 1,97 s | 50,1 % |
| 12 MP | 8,04 s | 4,97 s | 38,2 % |
| 24 MP | 13,99 s | 9,65 s | 31,0 % |

El uso de CPU de usuario+sistema fue aproximadamente el 99 % del tiempo de reloj
en 12 y 24 MP para ambos, es decir, los codificadores principales trabajan
efectivamente en un solo núcleo. La ventaja de Rust viene del códec y de menos
trabajo de copia/preview, no de paralelismo.

Archivos de salida a calidad 82, 12 MP (KB = 1024 B; MB = 1024² B):

| Formato | Godot | Rust/egui |
| --- | ---: | ---: |
| WebP | 88,9 KB | **78,7 KB** |
| JPG | **604,7 KB** | 700,9 KB |
| PNG | **12,24 MB** | 12,36 MB |
| ICO | 43,2 KB | **43,1 KB** |

Los tamaños no son directamente comparables: cada app usa un codificador y una
implementación de calidad distintos. El JPG de Rust produce archivos mayores y
el WebP menores con el mismo valor nominal 82.

## Interpretación

1. **Tamaño:** la diferencia sigue siendo enorme: 16,1 MiB frente a 70,7 MiB
   en Linux y 14,7 MiB frente a 104,7 MiB en Windows, incluso después de añadir
   SVG, cuentagotas, historial, idiomas, zoom, AVIF, JPEG XL y el enlace al
   repositorio.
2. **Arranque:** la aplicación Rust abre la ventana alrededor de un 17 % antes.
   Es una mejora real, aunque menor que la diferencia de tamaño.
3. **Memoria:** Rust usa la mitad de RAM en reposo y entre un 25 % y un 34 %
   menos en los escenarios de imagen medidos.
4. **Velocidad:** WebP, JPG e ICO son entre un 32 % y un 65 % más rápidos en
   12 MP; PNG sigue siendo lento en ambos, pero un 32 % más rápido en Rust.
5. **CPU:** la diferencia en reposo es cualitativa: Godot repinta continuamente
   y egui no. Bajo llvmpipe esto amplifica el consumo de CPU de Godot.

## Limitaciones de la medición

- El entorno no tiene GPU real: OpenGL usa llvmpipe y no hay VSync. Los valores
  absolutos de CPU y memoria no representan necesariamente un escritorio normal.
- No se pudo ejecutar la build Windows en Windows; solo se midió su tamaño y sus
  importaciones PE.
- El arranque se mide hasta la aparición de la ventana, no hasta el primer frame
  visible.
- No se hizo un arranque en frío completo: no hay permisos para vaciar toda la
  caché del sistema. La tanda con fadvise solo expulsa el ejecutable.
- El benchmark de códecs de Godot usó el binario del editor en headless, no la
  build portable. La lógica de imagen es la misma, pero el proceso base difiere.
- Las imágenes son sintéticas (paisaje con grano), no una muestra fotográfica
  diversa.
- Los benchmarks de códecs son una ejecución por tamaño; los de interfaz,
  tres repeticiones.
- No se midieron PSNR/SSIM ni calidad perceptual.
- La aplicación ya soporta AVIF y JPEG XL como salida; HEIC sigue sin soporte.
- AVIF y JPEG XL se añadieron después de la campaña comparativa: se verificó su
  funcionamiento, pero no se midió su velocidad frente a Godot.
- La aplicación Rust aún no interpreta EXIF ni ofrece accesibilidad nativa completa.

## Conclusión

Si el objetivo del producto es reducir tamaño, consumo de memoria y tiempo de
arranque, la migración a Rust/egui está justificada por estos datos. La mejora
de velocidad de codificación es clara pero menor que la mejora de huella, y no
hay ganancia de paralelismo: ambos son esencialmente mononúcleo en los códecs
medidos.

La aplicación 0.2.0 conserva esa ventaja después de incorporar SVG, el
cuentagotas, historial, idiomas, zoom, AVIF y JPEG XL. El siguiente coste relevante es completar accesibilidad, EXIF y
posibles códecs adicionales, que pueden alterar tanto el rendimiento como el
tamaño final.
