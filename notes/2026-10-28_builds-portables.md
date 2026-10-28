# 2026-10-28 — Builds portables del piloto

> Evidencia de sesión. No editar.

## Artefactos

```text
dist/hunk-converter-piloto-0.1.0-linux-x86_64/
dist/hunk-converter-piloto-0.1.0-linux-x86_64.tar.gz
dist/hunk-converter-piloto-0.1.0-windows-x86_64/
dist/hunk-converter-piloto-0.1.0-windows-x86_64.zip
```

Cada carpeta contiene el ejecutable, `export/`, `LEEME.txt`, licencia MIT y la
licencia OFL de Manrope.

## Verificación

- Linux: ejecutado bajo Xvfb + llvmpipe; `Ctrl+E` guardó un WebP en
  `export/` junto al binario.
- Windows: compilado de forma cruzada para `x86_64-pc-windows-gnu`; solo importa
  DLLs del sistema de Windows y no requiere DLLs adicionales de MinGW según la
  tabla de importaciones del PE.
- No se pudo ejecutar el binario Windows en este entorno Linux.

## Tamaños

- Linux: 7.828.752 bytes.
- Windows: 6.688.256 bytes.
- Tar Linux: ~3,6 MB.
- Zip Windows: ~3,2 MB.

## Script

`scripts/build_dist.sh` empaqueta ambos targets. El modo Windows necesita el
target de Rust y un compilador/linker para Windows. La build de esta sesión usó
Zig como C compiler/linker, pero esa configuración es del entorno y no forma
parte del repositorio.
