# Benchmark comparativo — Godot vs Rust/egui

> Fecha: 2026-10-28. Mediciones orientativas de una sola sesión.
> Prototipo de referencia: hunk-converter 0.1.0 (Godot 4.7.2) contra el piloto
> Rust/egui 0.1.0.

## Resumen

| Métrica | Godot 0.1.0 | Piloto Rust/egui | Diferencia |
| --- | ---: | ---: | ---: |
| Binario Linux | 70,7 MiB | **7,5 MiB** | **−89,4 %** |
| Binario Windows | 104,7 MiB | **6,4 MiB** | **−93,9 %** |
| Arranque a ventana (mediana, 10 runs) | 239,8 ms | **173,1 ms** | **−27,8 %** |
| RAM en reposo (RSS) | 209,7 MiB | **100,9 MiB** | **−51,9 %** |
| RAM con imagen 12 MP | 316,9 MiB | **211,3 MiB** | **−33,3 %** |
| Pico con GUI e imagen 12 MP | 447,5 MiB | **336,3 MiB** | **−24,8 %** |
| WebP q82, 12 MP | 1121,0 ms | **743,9 ms** | **−33,6 %** |
| JPG q82, 12 MP | 1137,0 ms | **380,8 ms** | **−66,5 %** |
| PNG 12 MP | 4723,4 ms | **3225,0 ms** | **−31,7 %** |
| ICO 16–256, 12 MP | 723,9 ms | **448,8 ms** | **−38,0 %** |
| CPU en reposo, 60 FPS software | ~1130 % | **0 %** | actividad continua vs. bajo demanda |

La CPU se expresa por proceso: 100 % = un núcleo. Las cifras de CPU en reposo
están muy influidas por usar llvmpipe sin GPU ni VSync; deben leerse como una
diferencia de arquitectura de repintado, no como vatios reales en un equipo de
escritorio.

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

| Artefacto | Godot | Rust/egui | Reducción |
| --- | ---: | ---: | ---: |
| Ejecutable Linux | 74.171.392 B (70,7 MiB) | 7.828.752 B (7,5 MiB) | 89,4 % |
| Carpeta Linux | 74.178.860 B | 7.836.653 B | 89,4 % |
| `.tar.gz` Linux | 27,7 MiB | 3,4 MiB | 87,6 % |
| Ejecutable Windows | 109.834.960 B (104,7 MiB) | 6.688.256 B (6,4 MiB) | 93,9 % |
| Carpeta Windows | 109.842.428 B | 6.696.157 B | 93,9 % |
| `.zip` Windows | 36,9 MiB | 3,0 MiB | 91,8 % |

Dependencias:

- Linux: ambos binarios enlazan solo bibliotecas base del sistema (libc/libm y
  runtime), y cargan X11/OpenGL dinámicamente.
- Windows: el `.exe` del piloto importa únicamente DLLs del sistema; no necesita
  DLLs de MinGW. El `.exe` de Godot no se ejecutó en Windows.
- El piloto no necesita el runtime de Godot ni archivos externos de assets: la
  fuente y el icono están embebidos.

## Arranque

Tiempo desde `spawn` hasta que la ventana aparece en el árbol X11. Medición
externa con `xwininfo` y sondeo aproximado de 5 ms. No es tiempo hasta primer
frame pintado y no es un arranque en frío real.

| Serie | Godot | Rust/egui | Diferencia |
| --- | ---: | ---: | ---: |
| Mediana, 10 arranques en caliente | 239,8 ms | **173,1 ms** | −27,8 % |
| Media | 243,5 ms | **180,2 ms** | −26,0 % |
| Desviación estándar | 8,6 ms | 17,4 ms | — |
| P95 | 262,1 ms | 227,5 ms | −13,2 % |
| Rango | 235,1–262,1 ms | 166,9–227,5 ms | — |
| Mediana tras expulsar el ejecutable de caché (8 runs) | 245,8 ms | **175,0 ms** | −28,8 % |

Muestras en caliente:

```text
Godot: 236,8 235,1 239,9 239,7 238,9 262,1 236,6 249,8 241,2 255,0 ms
Rust:  166,9 227,5 176,9 170,0 168,4 170,1 185,7 190,4 171,0 175,1 ms
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
| Vacío | 209,7 MiB | **100,9 MiB** | 209,7 MiB | **100,9 MiB** | 91 / 59 |
| 1600×1000 | 252,6 MiB | **129,6 MiB** | 252,6 MiB | **134,6 MiB** | 91 / 59 |
| 4000×3000 | 316,9 MiB | **211,3 MiB** | 447,5 MiB | **336,3 MiB** | 91 / 59 |

CPU en reposo, ventana visible sin interacción, 100 % = un núcleo:

| Escenario | Godot | Rust/egui |
| --- | ---: | ---: |
| Sin límite de FPS, llvmpipe | 1570,9 % (≈15,7 núcleos) | 0,0 % |
| Limitado a 60 FPS, llvmpipe | 1130 % (≈11,3 núcleos) | 0,0 % |

Godot repinta de forma continua. El piloto egui es *event-driven* y no pide
frames cuando está quieto. Con una GPU real, el coste absoluto de Godot sería
mucho menor; la diferencia de comportamiento en reposo seguiría dependiendo del
renderizador. Los hilos incluyen los workers de llvmpipe, no solo la lógica de
la app.

## Codificación headless y escalado

Una ejecución por tamaño. Tiempos en milisegundos, incluida la generación de la
vista previa. La memoria es el RSS máximo de `/usr/bin/time`.

| Entrada | App | Decode | WebP | JPG | PNG | ICO | Total | Pico |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1200×800 (1 MP) | Godot | 15,9 | 88,7 | 143,6 | 539,6 | 89,1 | 1,11 s | 111,6 MiB |
| | Rust | 10,0 | **54,2** | **21,6** | **254,9** | **41,8** | **0,40 s** | **17,2 MiB** |
| 2400×1800 (4 MP) | Godot | 71,0 | 488,4 | 684,5 | 2278,3 | 297,9 | 3,95 s | 206,4 MiB |
| | Rust | 33,9 | **315,5** | **189,3** | **1215,5** | **155,4** | **1,93 s** | **117,8 MiB** |
| 4000×3000 (12 MP) | Godot | 198,3 | 1121,0 | 1137,0 | 4723,4 | 723,9 | 8,04 s | 317,1 MiB |
| | Rust | 91,2 | **743,9** | **380,8** | **3225,0** | **448,8** | **4,91 s** | **230,2 MiB** |
| 6000×4000 (24 MP) | Godot | 396,4 | 2100,1 | 1616,6 | 8216,1 | 1438,6 | 13,99 s | 540,6 MiB |
| | Rust | 172,3 | **1381,6** | **686,9** | **6406,7** | **923,4** | **9,59 s** | **351,8 MiB** |

Tiempo total del proceso (incluye arranque del motor/benchmark):

| Tamaño | Godot | Rust | Mejora |
| --- | ---: | ---: | ---: |
| 1 MP | 1,11 s | 0,40 s | 64,0 % |
| 4 MP | 3,95 s | 1,93 s | 51,1 % |
| 12 MP | 8,04 s | 4,91 s | 38,9 % |
| 24 MP | 13,99 s | 9,59 s | 31,5 % |

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
implementación de calidad distintos. El JPG del piloto produce archivos mayores
y el WebP menores con el mismo valor nominal 82.

## Interpretación

1. **Tamaño:** la diferencia es estructural y enorme: 7,5 MiB frente a 70,7 MiB
   en Linux y 6,4 MiB frente a 104,7 MiB en Windows.
2. **Arranque:** el piloto abre la ventana alrededor de un 28 % antes. Es una
   mejora real, aunque menor que la diferencia de tamaño.
3. **Memoria:** el piloto usa la mitad de RAM en reposo y entre un 25 % y un 34 %
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
- El piloto aún no tiene paridad funcional con Godot (SVG, EXIF, accesibilidad,
  AVIF/HEIC/JPEG XL).

## Conclusión

Si el objetivo del producto es reducir tamaño, consumo de memoria y tiempo de
arranque, la migración a Rust/egui está justificada por estos datos. La mejora
de velocidad de codificación es clara pero menor que la mejora de huella, y no
hay ganancia de paralelismo: ambos son esencialmente mononúcleo en los códecs
medidos.

La decisión de migrar debe incluir el coste de completar paridad y de integrar
AVIF/JPEG XL, que pueden alterar tanto el rendimiento como el tamaño final.
Mientras tanto, el prototipo Godot sigue siendo la referencia funcional.
