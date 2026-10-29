# CI y releases

## Workflow único

`.github/workflows/ci.yml` concentra todo:

- **Pull requests:** solo formato, clippy y pruebas.
- **Push a `main`:** pruebas y builds de Linux, Windows y macOS, sin release.
- **Tags de versión:** pruebas, builds, checksums y GitHub Release.
- **Ejecución manual:** builds sin release.

## Plataformas

| Job | Runner | Artefacto |
| --- | --- | --- |
| Linux Debian x86_64 | `ubuntu-22.04` | `hunk-converter-<versión>-linux-x86_64.tar.gz` |
| Windows 10/11 x86_64 | `windows-latest` | `hunk-converter-<versión>-windows-x86_64.zip` |
| macOS universal | `macos-14` | `hunk-converter-<versión>-macos-universal.tar.gz` |

La build Linux se compila en Ubuntu 22.04 y requiere glibc 2.35 o superior
(Debian 12+, Ubuntu 22.04+). La build de macOS contiene un binario universal
`arm64 + x86_64`; no está firmada ni notarizada. Windows se compila con MSVC y
se distribuye como carpeta portable comprimida. La entrega Linux incluye un lanzador `.desktop` y el icono PNG; la entrega macOS es un `.app` con `icon.icns`.

## Release

El workflow se dispara con tags que encajen con `*.*.*` o `v*.*.*`. Para publicar:

```bash
git tag v1.0.0
git push origin main
git push origin v1.0.0
```

El job `release` solo se ejecuta cuando el evento es un tag. Descarga los tres
artefactos, genera `SHA256SUMS.txt` y crea la GitHub Release con las builds y las
notas automáticas.

También se puede lanzar manualmente desde **Actions → CI y release → Run
workflow**. En ese caso se generan los artefactos, pero no se publica ninguna
release porque no hay tag.

## Requisitos de build

- Rust 1.99.1.
- NASM para el decodificador AVIF.
- `lipo` en macOS (incluido en Xcode Command Line Tools).
- PowerShell y `Compress-Archive` en Windows para el `.zip` portable.
- `lipo` y los targets universales de Rust en macOS.
- `winresource` incrusta `icon.ico` en el `.exe` de Windows.

El script local equivalente es:

```bash
./scripts/build_dist.sh linux
./scripts/build_dist.sh windows
./scripts/build_dist.sh macos
```
