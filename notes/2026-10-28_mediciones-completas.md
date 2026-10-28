# 2026-10-28 — Mediciones completas

> Evidencia de sesión. No editar.

## Alcance

Se midieron las builds portables de Godot y del piloto Rust/egui:

- Tamaño de binarios y paquetes (Linux y Windows).
- Dependencias dinámicas.
- Arranque a ventana: 10 arranques en caliente + 8 tras expulsar el ejecutable.
- RSS, pico de memoria, hilos y CPU en reposo con GUI.
- Codificación headless a 1, 4, 12 y 24 MP.
- Tiempo, CPU de usuario/sistema, pico de memoria y tamaño de salida por formato.
- Limitaciones de medición en Windows y con GPU real.

## Resultado abreviado

| Métrica | Godot | Rust/egui |
| --- | ---: | ---: |
| Ejecutable Linux | 70,7 MiB | 7,5 MiB |
| Ejecutable Windows | 104,7 MiB | 6,4 MiB |
| Arranque a ventana (mediana) | 239,8 ms | 173,1 ms |
| RAM en reposo | 209,7 MiB | 100,9 MiB |
| RAM con 12 MP | 316,9 MiB | 211,3 MiB |
| WebP q82, 12 MP | 1121 ms | 743,9 ms |
| JPG q82, 12 MP | 1137 ms | 380,8 ms |
| PNG 12 MP | 4723,4 ms | 3225,0 ms |
| ICO 12 MP | 723,9 ms | 448,8 ms |
| Pico headless 12 MP | 317,1 MiB | 230,2 MiB |

El informe completo, con metodología, tablas por tamaño y muestras, está en
`docs/benchmark-2026-10-28.md`.

## Condiciones y límites

- Xvfb + llvmpipe, sin GPU ni VSync. La CPU en reposo de Godot (~1130 % a 60 FPS)
  está inflada por el renderizado software.
- El arranque mide aparición de ventana, no primer frame.
- No hubo arranque en frío completo.
- La build Windows no pudo ejecutarse aquí.
- Los benchmarks de códecs no miden PSNR/SSIM.
