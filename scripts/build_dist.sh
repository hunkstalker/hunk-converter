#!/usr/bin/env bash
# Empaqueta hunk-converter en carpetas portables.
#
# Linux:
#   ./scripts/build_dist.sh linux
#
# Windows:
#   ./scripts/build_dist.sh windows
#
# Requisitos:
#   - Rust 1.99.1
#   - Compilador de C y linker para el target
#   - NASM en x86/x86_64 para el decodificador AVIF (rav1d)
#   - Para Windows desde Linux, un toolchain mingw-w64 funcional y `zip`

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -n 1)"
TARGET="${1:-linux}"
DIST="$ROOT/dist"
BIN_NAME="hunk-converter"

common_files() {
	local folder="$1"
	mkdir -p "$folder/export" "$folder/LICENCIAS"
	cp "$ROOT/LICENSE" "$folder/LICENSE.txt"
	cp "$ROOT/assets/fonts/OFL.txt" "$folder/LICENCIAS/Manrope-OFL.txt"
	cat >"$folder/LEEME.txt" <<'TXT'
HUNK-CONVERTER
==============
Convierte, comprime, compara y exporta imágenes sin salir de tu equipo.

CÓMO USAR
- Arrastra una imagen a la ventana, pulsa "Seleccionar una imagen" o pega con Ctrl+V.
- Elige WebP, AVIF, JPG, PNG, JPEG XL o ICO.
- Ajusta la calidad o los tamaños del icono.
- Activa el cuentagotas y haz clic sobre la imagen para obtener sus códigos de color.
- Pulsa "Exportar" o Ctrl+E.

Las imágenes se guardan en la carpeta "export/" que está junto a este
programa. No se sobrescriben: si el nombre ya existe se añade -2, -3, etc.

El idioma, el historial de colores, sus bloqueos y el estado de la ventana
maximizada se guardan en config.ini, también junto al programa. Puedes cambiar
entre español, catalán e inglés desde la cabecera.

- La aplicación no necesita instalación ni conexión a internet.
- En Linux se requieren las bibliotecas gráficas habituales (X11/OpenGL).
- La versión Windows se distribuye como ejecutable portable.
TXT
}

build_linux() {
	local folder="$DIST/hunk-converter-$VERSION-linux-x86_64"
	echo "→ Compilando Linux"
	cargo build --release --bin "$BIN_NAME"
	rm -rf "$folder"
	mkdir -p "$folder"
	cp "$ROOT/target/release/$BIN_NAME" "$folder/hunk-converter"
	chmod +x "$folder/hunk-converter"
	common_files "$folder"
	tar -czf "$DIST/hunk-converter-$VERSION-linux-x86_64.tar.gz" \
		-C "$DIST" "hunk-converter-$VERSION-linux-x86_64"
	echo "✓ $folder"
	echo "✓ $DIST/hunk-converter-$VERSION-linux-x86_64.tar.gz"
}

build_windows() {
	local folder="$DIST/hunk-converter-$VERSION-windows-x86_64"
	local target="${WINDOWS_TARGET:-x86_64-pc-windows-gnu}"
	echo "→ Compilando Windows ($target)"
	cargo build --release --target "$target" --bin "$BIN_NAME"
	rm -rf "$folder"
	mkdir -p "$folder"
	cp "$ROOT/target/$target/release/$BIN_NAME.exe" "$folder/hunk-converter.exe"
	common_files "$folder"
	if command -v zip >/dev/null 2>&1; then
		(cd "$DIST" && zip -qr "hunk-converter-$VERSION-windows-x86_64.zip" \
			"hunk-converter-$VERSION-windows-x86_64")
		echo "✓ $DIST/hunk-converter-$VERSION-windows-x86_64.zip"
	else
		echo "Aviso: instala 'zip' para generar también el archivo comprimido."
	fi
	echo "✓ $folder"
}

mkdir -p "$DIST"
case "$TARGET" in
linux) build_linux ;;
windows) build_windows ;;
all)
	build_linux
	build_windows
	;;
*)
	echo "Uso: $0 [linux|windows|all]" >&2
	exit 2
	;;
esac
