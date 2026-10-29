# Proyecto — hunk-converter

> Documento vivo. Última actualización: 2026-10-29.
> Versión actual: **0.2.0** · Estado: **aplicación principal Rust + egui, portable y funcional**.

## 1. Visión

hunk-converter es una aplicación de escritorio local-first para convertir, comprimir, comparar y exportar imágenes. También genera iconos `.ico` multirresolución para Windows.

El producto está diseñado para hacer tres operaciones sin fricción:

1. Convertir una imagen a otro formato.
2. Reducir su peso ajustando la calidad.
3. Comprobar visualmente el resultado antes de guardarlo.

La interacción principal es arrastrar, ajustar y comparar en una sola ventana. No hay cuentas, subidas, telemetría ni pasos de configuración inicial.

La versión actual está construida en **Rust + egui**. El prototipo original de Godot se conserva como material histórico en `../Prototipo`.

## 2. Principios de producto

- **Local-first:** todo el procesamiento ocurre en el equipo del usuario. La aplicación no realiza peticiones de red.
- **Portable:** no requiere instalación. Los resultados se guardan en `export/`, junto al ejecutable.
- **Repintado bajo demanda:** la interfaz solo solicita frames cuando hay actividad. Una ventana estática no consume un bucle de render a 60 FPS.
- **Transparencia:** la comparación usa la imagen codificada y decodificada, no una estimación.
- **No sobrescribir:** cada exportación crea un nombre único.
- **Código abierto:** licencia MIT, dependencias razonables y pruebas ejecutables sin servicios externos.
- **Interfaz tranquila:** una tarea principal, jerarquía clara y espacio para la imagen.

## 3. Funcionalidad actual

### 3.1 Entrada

- Arrastrar y soltar un archivo en cualquier parte de la ventana.
- Selector nativo de archivos con el botón **Seleccionar una imagen** o `Ctrl+O`.
- Botón **Cambiar** para reemplazar la imagen conservando los ajustes.
- Formatos: **PNG, JPG/JPEG, WebP, BMP, TGA y SVG**.
- SVG rasterizado con `resvg`, manteniendo proporción y transparencia.
- Detección por contenido con respaldo por extensión.
- Límites: **200 MB**, **40 megapíxeles** y **16 384 px por lado**.
- Los errores se muestran como avisos y no sustituyen la sesión válida anterior.

### 3.2 Formatos de salida

| Formato | Extensión | Compresión | Transparencia | Uso |
| --- | --- | --- | --- | --- |
| WebP | `.webp` | Con pérdida, calidad 1–100 | Sí | Opción recomendada |
| JPG | `.jpg` | Con pérdida, calidad 1–100 | No; fondo blanco | Compatibilidad |
| PNG | `.png` | Sin pérdida | Sí | Calidad pixel a pixel |
| ICO | `.ico` | PNG por tamaño | Sí | Iconos de Windows |

- Calidad inicial: **82 %**.
- WebP y JPG muestran deslizador; PNG conserva todos los píxeles; ICO usa selección de tamaños.
- JPG aplana la transparencia sobre blanco de forma predecible.
- ICO incluye 16, 32, 48, 64, 128 y 256 px, ajustando la imagen centrada en un lienzo transparente sin recortar.
- Se exige al menos un tamaño ICO seleccionado.

### 3.3 Comparador y vista previa

- Vista dividida entre **Original** y **Resultado**.
- Divisor arrastrable con ratón; el cursor de redimensionado solo aparece sobre la línea o el círculo.
- Respuesta a `←`/`→`, `Inicio` y `Fin`.
- La imagen resultante se obtiene decodificando el archivo codificado real.
- Damero para transparencia.
- Modo ICO: se oculta el divisor y se muestra el icono generado, con recuento de tamaños.
- El lienzo se adapta al tamaño de la ventana conservando la proporción.
- La vista previa se limita a 2048 px de lado; la exportación conserva la resolución completa.

### 3.4 Cuentagotas

- Icono de **cuentagotas** en la cabecera de la tarjeta inferior.
- Un clic activa o desactiva el modo; después basta con hacer clic sobre la imagen.
- También se puede pulsar el icono y arrastrar hasta el píxel: se muestrea mientras se arrastra y se conserva el último color al soltar.
- `Esc` desactiva el modo sin alterar la muestra.
- Se muestran las coordenadas del píxel muestreado.
- La marca visual de muestreo solo se dibuja mientras el cuentagotas está activo.
- Códigos simultáneos: **HEX, RGB, RGBA y HSL**, cada uno con botón de copia.
- El muestreo no mueve el divisor del comparador.
- En modo ICO se desactiva, porque la vista no representa la imagen original.
- La muestra se limpia al cargar otra imagen o reiniciar.

### 3.5 Exportación

- Botón principal **Exportar .ext** y atajo `Ctrl+E`.
- Guardado directo en la carpeta **`export/`** situada junto al ejecutable.
- La carpeta se crea si no existe.
- El nombre base se conserva; si el destino ya existe se añaden `-2`, `-3`, etc.
- El nombre se sanea para Windows.
- Los errores de permisos o escritura se comunican mediante aviso.
- La build portable incluye una carpeta `export/` vacía junto al ejecutable.

### 3.6 Reinicio y atajos

| Atajo | Acción |
| --- | --- |
| `Ctrl+O` | Abrir imagen |
| `Ctrl+E` | Exportar |
| `Ctrl+R` | Reiniciar la sesión |
| `←` / `→` | Mover el divisor |
| `Inicio` / `Fin` | Extremos del divisor |

- **Reiniciar** elimina la imagen, el resultado, el muestreo y el estado de exportación.
- Restablece formato WebP, calidad 82 y todos los tamaños ICO.
- No borra archivos ya exportados.

## 4. Experiencia de uso

### Estados

1. **Vacío:** tarjeta central con ilustración, acción principal y formatos admitidos.
2. **Editando:** panel lateral de ajustes + comparador + metadatos.
3. **Procesando:** se mantiene la última vista válida, aparece un indicador y se bloquea la exportación.
4. **Listo:** comparador, peso y ahorro actualizados; exportación disponible.
5. **Exportado:** confirmación con la ruta relativa.
6. **Error:** aviso temporal; la sesión existente se conserva cuando es posible.

### Ventana y distribución

- Tamaño inicial 1280 × 800; mínimo 1024 × 800.
- Cabecera con marca, estado local y reinicio.
- Panel lateral de 390 px con archivo, formato, calidad o tamaños ICO, cuentagotas y exportación.
- Zona principal con cabecera de vista previa, lienzo y pie de pesos.
- Pie global con estado del proyecto y versión.
- El cuentagotas permanece visible y se sitúa por encima del botón de exportar.

### Render bajo demanda

- La interfaz no mantiene una animación de UI en reposo.
- egui solo pide repintado cuando hay eventos o mientras se está procesando una imagen.
- En reposo, el proceso no consume CPU de render.
- Es una diferencia deliberada frente al prototipo Godot, que mantenía el bucle de render continuo.

## 5. Arquitectura

### Stack

- **Rust 1.99.1**, fijado en `rust-toolchain.toml`.
- **eframe/egui 0.37** con renderizador **glow** (OpenGL).
- `image` 0.25 para decodificación PNG/JPG/WebP/BMP/TGA.
- `webp`/libwebp para codificación WebP.
- `resvg` 0.49 para SVG.
- `rfd` para diálogo de archivos nativo.
- Procesamiento en un hilo con canal `std::sync::mpsc`.
- Sin runtime de Python, Node, Godot ni ImageMagick.

### Mapa de archivos

| Ruta | Responsabilidad |
| --- | --- |
| `src/main.rs` | Arranque de eframe, ventana, icono e inicialización |
| `src/app.rs` | Estado, UI, interacción, cuentagotas, hilos y exportación |
| `src/core/codec.rs` | Carga, decodificación, codificación y previews |
| `src/core/color.rs` | Códigos de color y aplanado RGBA |
| `src/core/ico.rs` | Contenedor ICO y frames PNG |
| `src/demo.rs` | Imagen sintética para demos y capturas |
| `src/theme.rs` | Paleta, tipografía, estilos y frames |
| `src/bin/bench.rs` | Benchmark de códecs y memoria |
| `src/bin/make_demo.rs` | Generador de imágenes de prueba |
| `scripts/build_dist.sh` | Empaquetado portable Linux/Windows |
| `docs/benchmark-2026-10-28.md` | Comparación de rendimiento y recursos |
| `tests` integrados en los módulos | Pruebas unitarias Rust |

### Flujo de datos

```text
Archivo local
  → lectura y decodificación (image / resvg)
  → RGBA8
  → parámetros de salida (formato, calidad, tamaños ICO)
  → hilo de trabajo
      → codificación real
      → decodificación para preview
      → PNG de preview + buffer final
  → hilo principal
      → texturas de egui
      → pesos, ahorro, comparador, cuentagotas
  → export/archivo.ext
```

### Procesamiento y concurrencia

- Solo hay una codificación activa a la vez.
- Los cambios de calidad se agrupan con un debounce de 160 ms.
- Si cambian los ajustes mientras se procesa, se guarda la última petición.
- Un número de generación descarta resultados obsoletos.
- Las texturas se crean en el hilo de UI; las imágenes se codifican en un worker.
- El cierre espera a que termine la tarea activa.

### Cuentagotas: implementación

- La vista previa guarda la geometría del rectángulo ajustado de la imagen.
- La posición del puntero se convierte a coordenadas del píxel original completo.
- El color se lee de la imagen RGBA original, no del resultado recomprimido.
- Se conservan los cuatro canales, de modo que RGBA muestra la transparencia real.
- El icono alterna el modo al hacer clic; si se arrastra desde él, se activa y muestrea bajo el puntero.
- El modo tiene prioridad sobre el arrastre del divisor.
- La marca de muestreo se pinta con contraste claro/oscuro.
- `Esc` cancela el modo.
- En modo ICO el icono queda deshabilitado para no muestrear una representación engañosa.

### ICO

- Cabecera estándar ICO: `reserved=0`, `type=1`, `count=N`.
- Entradas de 16 bytes con ancho/alto (0 para 256), planos, bits, tamaño y offset.
- Frames PNG RGBA, centrados en un lienzo cuadrado transparente.
- Tamaños 16/32/48/64/128/256; preview del frame mayor.

### Tema visual

- Paleta carbón con acento lima.
- Manrope incluida con licencia OFL.
- Tema egui personalizado: tipografía, botones, paneles, sliders, scrollbars y tooltips.
- Icono de la aplicación embebido desde `assets/branding/icon_256.png`.
- Las capturas de documentación se generan con el modo de demo y el capturador interno.

## 6. Distribución portable

### Salida

- En una build: `<carpeta del ejecutable>/export/`.
- En desarrollo: `target/release/export/`.
- Los nombres no se sobrescriben.

### Estructura de la build

```text
hunk-converter-0.2.0-windows-x86_64/
├── hunk-converter.exe
├── export/
├── LEEME.txt
├── LICENSE.txt
└── LICENCIAS/
    └── Manrope-OFL.txt
```

- El modo Linux genera una carpeta y un `.tar.gz`.
- El modo Windows genera una carpeta y un `.zip` si `zip` está disponible.
- La build de Linux depende de las bibliotecas gráficas del sistema (X11/OpenGL), como cualquier app nativa.
- La build Windows importa únicamente DLLs del sistema; no necesita DLLs de MinGW.
- La build Windows no se ha ejecutado todavía en un Windows real (t-23).

## 7. Rendimiento y recursos

La comparación completa entre el prototipo Godot y la aplicación Rust está en [`docs/benchmark-2026-10-28.md`](docs/benchmark-2026-10-28.md).

Resumen del contexto de referencia:

- La aplicación Rust es drásticamente más pequeña y usa menos RAM.
- El arranque a ventana es más rápido.
- La interfaz no consume CPU de render en reposo: repinta bajo demanda.
- WebP, JPG, PNG e ICO son más rápidos en la versión Rust.
- Las cifras absolutas de CPU en reposo de Godot están infladas por usar llvmpipe sin VSync; no representan vatios reales en una GPU.
- Windows está pendiente de validación en hardware real.

## 8. Límites conocidos

- No se interpreta la orientación EXIF ni se conservan metadatos (`t-10`).
- No hay redimensionado, recorte, zoom ni procesamiento por lotes (`t-11`, `t-12`, `t-16`).
- No hay AVIF, HEIC ni JPEG XL (`t-15`).
- No se conservan perfiles ICC.
- La accesibilidad nativa completa y el uso con lector de pantalla están pendientes (`t-22`).
- La build Windows se genera, pero no se ha ejecutado en Windows (`t-23`).
- El icono nativo del `.exe` de Windows aún no está incrustado (`t-24`).
- No hay build macOS en esta versión (`t-17`).
- El resultado ICO usa PNG dentro del contenedor, compatible con Windows moderno.

## 9. Pruebas y verificación

- **9 pruebas Rust**: codificación, calidad, transparencia, aplanado JPG, ICO, códigos de color, carga SVG y zona de agarre del comparador.
- `cargo fmt --all -- --check` limpio.
- `cargo clippy --all-targets -- -D warnings` limpio.
- CI de Godot histórica en el repositorio archivado; CI de Rust en este repositorio.
- Exportación Linux verificada bajo Xvfb: el archivo aparece en `export/` junto al binario.
- Prueba de arranque, memoria, CPU y escalado de códecs documentada en el informe de benchmark.

Comandos:

```bash
cargo run --release -- /ruta/a/imagen.jpg
cargo test
cargo clippy --all-targets -- -D warnings
./scripts/build_dist.sh all
```

## 10. Tareas y documentación

- `todo.md`: tareas pendientes con identificador `t-N`, título y descripción.
- `t-completadas.md`: tareas cerradas.
- `retomar.md`: contexto para nuevas sesiones.
- `notes/AAAA-MM-DD_asunto.md`: evidencia fechada e inmutable.
- `docs/benchmark-2026-10-28.md`: resultados de rendimiento y metodología.

Al preguntar por las tareas, se debe devolver toda la información de cada tarea pendiente de `todo.md`, sin omitir descripciones.

## 11. Licencia y créditos

- Código: MIT (ver `LICENSE`).
- Tipografía Manrope: SIL Open Font License 1.1 (ver `assets/fonts/OFL.txt`).
- Prototipo Godot histórico: `../Prototipo`.
- Motores y bibliotecas: Godot (histórico), eframe/egui, image, webp, resvg.
