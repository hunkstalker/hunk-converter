#!/usr/bin/env bash
# Empaqueta hunk-converter en carpetas portables.
#
# Linux:
#   ./scripts/build_dist.sh linux
#
# Windows:
#   ./scripts/build_dist.sh windows
#
# macOS universal:
#   ./scripts/build_dist.sh macos
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
HUNK Converter
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
- En Linux, hunk-converter.desktop y hunk-converter.png permiten integrar la app en el menú.
- La versión Windows se distribuye como ejecutable portable.
- En macOS, abre "HUNK Converter.app" desde la carpeta extraída.
TXT
}

install_linux_launcher() {
	local folder="$1"
	cp "$ROOT/assets/branding/icon_256.png" "$folder/hunk-converter.png"
	cat >"$folder/hunk-converter.desktop" <<'DESKTOP'
[Desktop Entry]
Type=Application
Name=HUNK Converter
GenericName=Image Converter
Comment=Convert, compress, compare and sample images
Exec=hunk-converter
TryExec=hunk-converter
Icon=hunk-converter
Terminal=false
Categories=Graphics;Photography;
StartupWMClass=hunk-converter
DESKTOP
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
	install_linux_launcher "$folder"
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

build_macos() {
	local folder="$DIST/hunk-converter-$VERSION-macos-universal"
	local app="$folder/HUNK Converter.app"
	local arm="${MACOS_ARM_TARGET:-aarch64-apple-darwin}"
	local intel="${MACOS_X86_TARGET:-x86_64-apple-darwin}"
	echo "→ Compilando macOS universal ($arm + $intel)"
	cargo build --release --target "$arm" --bin "$BIN_NAME"
	cargo build --release --target "$intel" --bin "$BIN_NAME"
	rm -rf "$folder"
	mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
	lipo -create \
		"$ROOT/target/$arm/release/$BIN_NAME" \
		"$ROOT/target/$intel/release/$BIN_NAME" \
		-output "$app/Contents/MacOS/hunk-converter"
	chmod +x "$app/Contents/MacOS/hunk-converter"
	cp "$ROOT/assets/branding/icon.icns" "$app/Contents/Resources/icon.icns"
	cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleDevelopmentRegion</key><string>es</string>
	<key>CFBundleDisplayName</key><string>HUNK Converter</string>
	<key>CFBundleExecutable</key><string>hunk-converter</string>
	<key>CFBundleIconFile</key><string>icon.icns</string>
	<key>CFBundleIdentifier</key><string>com.hunkstalker.hunk-converter</string>
	<key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
	<key>CFBundleName</key><string>HUNK Converter</string>
	<key>CFBundlePackageType</key><string>APPL</string>
	<key>CFBundleShortVersionString</key><string>${VERSION}</string>
	<key>CFBundleVersion</key><string>${VERSION}</string>
	<key>LSApplicationCategoryType</key><string>public.app-category.graphics-design</string>
	<key>LSMinimumSystemVersion</key><string>11.0</string>
	<key>NSHighResolutionCapable</key><true/>
	<key>NSPrincipalClass</key><string>NSApplication</string>
</dict>
</plist>
PLIST
	ln -s "HUNK Converter.app/Contents/MacOS/hunk-converter" "$folder/hunk-converter"
	common_files "$folder"
	tar -czf "$DIST/hunk-converter-$VERSION-macos-universal.tar.gz" \
		-C "$DIST" "hunk-converter-$VERSION-macos-universal"
	echo "✓ $app"
	echo "✓ $DIST/hunk-converter-$VERSION-macos-universal.tar.gz"
}

mkdir -p "$DIST"
case "$TARGET" in
linux) build_linux ;;
windows) build_windows ;;
all)
	build_linux
	build_windows
	;;
macos) build_macos ;;
*)
	echo "Uso: $0 [linux|windows|macos|all]" >&2
	exit 2
	;;
esac
