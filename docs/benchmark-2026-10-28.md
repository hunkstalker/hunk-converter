# Benchmark comparativo — Godot vs Rust/egui

> Fecha: 2026-10-28. Mediciones orientativas de una sola sesión.
> Referencia histórica: hunk-converter 0.1.0 (Godot 4.7.2).
> Aplicación actual: hunk-converter **0.2.0** (Rust + egui).

## Resumen

| Métrica | Godot 0.1.0 | Rust/egui 0.2.0 | Diferencia |
| --- | ---: | ---: | ---: |
| Binario Linux | 70,7 MiB | **9,1 MiB** | **−87,2 %** |
| Binario Windows | 104,7 MiB | **7,8 MiB** | **−92,6 %** |
| Arranque a ventana (mediana, 20 runs) | 239,8 ms | **177,8 ms** | **−25,9 %** |
| RAM en reposo (RSS) | 209,7 MiB | **101,4 MiB** | **−51,6 %** |
| RAM con imagen 12 MP | 316,9 MiB | **211,6 MiB** | **−33,2 %** |
| Pico con GUI e imagen 12 MP | 447,5 MiB | **335,4 MiB** | **−25,1 %** |
| WebP q82, 12 MP | 1121,0 ms | **741,1 ms** | **−33,9 %** |
| JPG q82, 12 MP | 1137,0 ms | **390,3 ms** | **−65,7 %** |
| PNG 12 MP | 4723,4 ms | **3204,6 ms** | **−32,2 %** |
| ICO 16–256, 12 MP | 723,9 ms | **456,2 ms** | **−37,0 %** |
| CPU en reposo, 60 FPS software | ~1130 % | **0 %** | actividad continua vs. bajo demanda |

La CPU se expresa por proceso: 100 % = un núcleo. Las cifras de CPU en reposo
están muy influidas por usar llvmpipe sin GPU ni VSync; deben leerse como una
diferencia de arquitectura de repintado, no como vatios reales en un equipo de
escritorio.

La aplicación 0.2.0 añade rasterización SVG con `resvg` y el cuentagotas, de
modo que su binario es algo mayor que el de la prueba de concepto inicial. Las
cifras de esta página corresponden al paquete final, no al piloto de 0.1.0.

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
| Ejecutable Linux | 74.171.392 B (70,7 MiB) | 9.498.816 B (9,1 MiB) | 87,2 % |
| Carpeta Linux | 74.178.860 B | 9.506.553 B | 87,2 % |
| `.tar.gz` Linux | 27,7 MiB | 4,2 MiB | 84,9 % |
| Ejecutable Windows | 109.834.960 B (104,7 MiB) | 8.180.736 B (7,8 MiB) | 92,6 % |
| Carpeta Windows | 109.842.428 B | 8.188.473 B | 92,5 % |
| `.zip` Windows | 36,9 MiB | 3,7 MiB | 89,9 % |

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
| Mediana, 20 arranques en caliente | 239,8 ms | **177,8 ms** | −25,9 % |
| Media | 243,5 ms | **179,9 ms** | −26,1 % |
| Desviación estándar | 8,6 ms | 7,8 ms | — |
| P95 | 262,1 ms | 192,9 ms | −26,4 % |
| Rango | 235,1–262,1 ms | 170,2–201,9 ms | — |
| Mediana tras expulsar el ejecutable de caché (10 runs) | 245,8 ms | **185,0 ms** | −24,7 % |

Muestras en caliente:

```text
Godot: 236,8 235,1 239,9 239,7 238,9 262,1 236,6 249,8 241,2 255,0 ms
Rust:  170,2 174,8 179,9 177,5 172,0 178,1 173,9 176,7 201,9 177,2 185,9 184,0 170,4 182,1 174,4 172,2 189,3 181,6 182,4 192,9 ms
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
| Vacío | 209,7 MiB | **101,4 MiB** | 209,7 MiB | **101,4 MiB** | 91 / 59 |
| 1600×1000 | 252,6 MiB | **132,8 MiB** | 252,6 MiB | **137,9 MiB** | 91 / 59 |
| 4000×3000 | 316,9 MiB | **211,6 MiB** | 447,5 MiB | **335,4 MiB** | 91 / 59 |

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
| | Rust 0.2.0 | 10,6 | **53,7** | **25,0** | **249,5** | **39,1** | **0,40 s** | **18,0 MiB** |
| 2400×1800 (4 MP) | Godot | 71,0 | 488,4 | 684,5 | 2278,3 | 297,9 | 3,95 s | 206,4 MiB |
| | Rust 0.2.0 | 36,3 | **321,3** | **197,1** | **1225,8** | **151,3** | **1,95 s** | **118,2 MiB** |
| 4000×3000 (12 MP) | Godot | 198,3 | 1121,0 | 1137,0 | 4723,4 | 723,9 | 8,04 s | 317,1 MiB |
| | Rust 0.2.0 | 89,3 | **741,1** | **390,3** | **3204,6** | **456,2** | **4,90 s** | **230,9 MiB** |
| 6000×4000 (24 MP) | Godot | 396,4 | 2100,1 | 1616,6 | 8216,1 | 1438,6 | 13,99 s | 540,6 MiB |
| | Rust 0.2.0 | 183,0 | **1380,8** | **682,3** | **6370,7** | **934,0** | **9,53 s** | **352,5 MiB** |

Tiempo total del proceso (incluye arranque del motor/benchmark):

| Tamaño | Godot | Rust | Mejora |
| --- | ---: | ---: | ---: |
| 1 MP | 1,11 s | 0,40 s | 64,0 % |
| 4 MP | 3,95 s | 1,95 s | 50,6 % |
| 12 MP | 8,04 s | 4,90 s | 39,1 % |
| 24 MP | 13,99 s | 9,53 s | 31,9 % |

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

1. **Tamaño:** la diferencia es estructural y enorme: 9,1 MiB frente a 70,7 MiB
   en Linux y 7,8 MiB frente a 104,7 MiB en Windows, incluso después de añadir
   SVG y el cuentagotas.
2. **Arranque:** la aplicación Rust abre la ventana alrededor de un 26 % antes.
   Es una mejora real, aunque menor que la diferencia de tamaño.
3. **Memoria:** Rust usa la mitad de RAM en reposo y entre un 25 % y un 34 %
   menos en los escenarios de imagen medidos.
4. **Velocidad:** WebP, JPG e ICO son entre un 34 % y un 66 % más rápidos en
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
- La aplicación Rust aún no interpreta EXIF, no ofrece accesibilidad nativa
  completa y no soporta AVIF/HEIC/JPEG XL.

## Conclusión

Si el objetivo del producto es reducir tamaño, consumo de memoria y tiempo de
arranque, la migración a Rust/egui está justificada por estos datos. La mejora
de velocidad de codificación es clara pero menor que la mejora de huella, y no
hay ganancia de paralelismo: ambos son esencialmente mononúcleo en los códecs
medidos.

La aplicación 0.2.0 conserva esa ventaja después de incorporar SVG y el
cuentagotas. El siguiente coste relevante es completar accesibilidad, EXIF y
posibles códecs adicionales, que pueden alterar tanto el rendimiento como el
tamaño final.
