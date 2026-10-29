# 2026-10-28 — Mediciones comparativas

> Evidencia de sesión. No editar.

Se completó la comparación entre la build Godot y el piloto Rust/egui.

## Resultados principales

| Métrica | Godot | Rust/egui |
| --- | ---: | ---: |
| Arranque a ventana (mediana, 10 runs) | 239,8 ms | 173,1 ms |
| Ejecutable Linux | 70,7 MiB | 7,5 MiB |
| Ejecutable Windows | 104,7 MiB | 6,4 MiB |
| RAM en reposo | 209,7 MiB | 100,9 MiB |
| RAM con 12 MP | 316,9 MiB | 211,3 MiB |
| WebP q82, 12 MP | 1121 ms | 743,9 ms |
| JPG q82, 12 MP | 1137 ms | 380,8 ms |
| PNG, 12 MP | 4723,4 ms | 3225,0 ms |
| ICO, 12 MP | 723,9 ms | 448,8 ms |

## Cobertura

- Tamaños de binarios y paquetes Linux/Windows.
- Arranque en caliente y con el ejecutable expulsado de caché.
- RSS, pico de memoria, hilos y CPU en reposo.
- Codificación headless a 1, 4, 12 y 24 MP.
- Tiempo y pico de memoria por formato.
- Tamaños de salida a calidad 82.

El informe completo, la metodología y las limitaciones están en el repositorio del
piloto: `../Piloto-Rust-egui/docs/benchmark-2026-10-28.md`.

## Límites

- Xvfb + llvmpipe, sin GPU ni VSync; la CPU absoluta está inflada.
- Windows generado pero no ejecutado.
- Sin arranque en frío completo.
- Sin PSNR/SSIM.
- El benchmark de códecs de Godot usó el binario del editor en headless.
