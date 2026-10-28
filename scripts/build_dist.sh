#!/usr/bin/env bash
# Empaqueta el piloto Rust/egui en carpetas portables.
#
# Linux:
#   ./scripts/build_dist.sh linux
#
# Windows (cross-compilación):
#   ./scripts/build_dist.sh windows
#
# Requisitos:
#   - Rust 1.99.1
#   - Compilador de C y linker para el target
#   - Para Windows desde Linux, un toolchain mingw-w64 funcional
#   - `zip` para el archivo comprimido de Windows

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -n 1)"
TARGET="${1:-linux}"
DIST="$ROOT/dist"
BIN_NAME="hunk-converter-poc"

common_files() {
	local folder="$1"
	mkdir -p "$folder/export" "$folder/LICENCIAS"
	cp "$ROOT/LICENSE" "$folder/LICENSE.txt"
	cp "$ROOT/assets/fonts/OFL.txt" "$folder/LICENCIAS/Manrope-OFL.txt"
	cat >"$folder/LEEME.txt" <<'TXT'
HUNK-CONVERTER — PILOTO RUST/EGUI
================================
Esta es la versión piloto de hunk-converter, hecha con Rust y egui.
No sustituye todavía a la aplicación principal Godot; sirve para comparar
experiencia, rendimiento y consumo de recursos.

CÓMO USAR
- Arrastra una imagen a la ventana o abre una desde el botón de selección.
- Elige WebP, JPG, PNG o ICO.
- Ajusta la calidad y el color de fondo cuando corresponda.
- Pulsa Exportar o Ctrl+E.

Las imágenes se guardan en la carpeta "export/" que está junto a este
programa. Si el nombre existe se añade -2, -3, etc. No se sobrescribe nada.

LIMITACIONES DEL PILOTO
- No incluye SVG, EXIF, AVIF, HEIC ni JPEG XL.
- No tiene accesibilidad completa.
- En Linux necesita las bibliotecas gráficas del sistema (X11/OpenGL).
- La ventana mínima es 1024 × 800.

El código es MIT. Manrope se distribuye bajo SIL OFL 1.1.
TXT
}

build_linux() {
	local folder="$DIST/hunk-converter-piloto-$VERSION-linux-x86_64"
	echo "→ Compilando piloto Linux"
	cargo build --release --bin "$BIN_NAME"
	rm -rf "$folder"
	mkdir -p "$folder"
	cp "$ROOT/target/release/$BIN_NAME" "$folder/hunk-converter-piloto"
	chmod +x "$folder/hunk-converter-piloto"
	common_files "$folder"
	tar -czf "$DIST/hunk-converter-piloto-$VERSION-linux-x86_64.tar.gz" \
		-C "$DIST" "hunk-converter-piloto-$VERSION-linux-x86_64"
	echo "✓ $folder"
	echo "✓ $DIST/hunk-converter-piloto-$VERSION-linux-x86_64.tar.gz"
}

build_windows() {
	local folder="$DIST/hunk-converter-piloto-$VERSION-windows-x86_64"
	local target="${WINDOWS_TARGET:-x86_64-pc-windows-gnu}"
	echo "→ Compilando piloto Windows ($target)"
	cargo build --release --target "$target" --bin "$BIN_NAME"
	rm -rf "$folder"
	mkdir -p "$folder"
	cp "$ROOT/target/$target/release/$BIN_NAME.exe" "$folder/hunk-converter-piloto.exe"
	common_files "$folder"
	if command -v zip >/dev/null 2>&1; then
		(cd "$DIST" && zip -qr "hunk-converter-piloto-$VERSION-windows-x86_64.zip" \
			"hunk-converter-piloto-$VERSION-windows-x86_64")
		echo "✓ $DIST/hunk-converter-piloto-$VERSION-windows-x86_64.zip"
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
