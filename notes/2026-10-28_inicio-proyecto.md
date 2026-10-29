# 2026-10-28 — Inicio de Lienzo

> Evidencia de sesión. No editar.

## 1. Objetivo y restricciones

Se inició **Lienzo**, una aplicación de escritorio con Godot 4 para convertir, comprimir, comparar y exportar imágenes, incluyendo iconos `.ico`.

Requisitos fijados durante la sesión:

- Arrastrar y soltar imágenes de forma inmediata.
- Comparador visual con divisor arrastrable.
- Formatos de salida WebP, JPG, PNG e ICO.
- Exportación ICO.
- Botón de reinicio completo.
- Proyecto open source con git desde el inicio.
- Builds portables; las imágenes generadas deben guardarse en `export/`, junto al ejecutable.
- Documentación viva en `proyecto.md`, tareas en `todo.md`/`t-completadas.md`, notas fechadas y `retomar.md`.

## 2. Decisiones de implementación

- **Nombre:** Lienzo. Interfaz en español.
- **Motor:** Godot 4.7.2 estándar, GDScript y renderizador GL Compatibility.
- **Arquitectura:** lógica de codificación pura en `ImageProcessor`, sin nodos, para poder ejecutarla en un hilo de trabajo.
- **Entrada:** PNG, JPG, WebP, BMP, TGA y SVG; límites de 200 MB, 40 MP y 16 384 px por lado.
- **Salida:** WebP/JPG con pérdida (calidad 1–100), PNG sin pérdida e ICO con frames PNG.
- **JPG:** aplanado sobre blanco para que la transparencia no genere resultados impredecibles.
- **ICO:** contenedor escrito a mano, tamaños 16/32/48/64/128/256, ajuste centrado con lienzo transparente y sin recorte.
- **Comparador:** shader con damero, ajuste independiente de ambas imágenes y modo ICO que oculta el divisor.
- **Vista previa:** se decodifica el archivo codificado real; no se simula la compresión.
- **Procesamiento:** `WorkerThreadPool`, una tarea activa, agrupación de cambios de calidad a 160 ms y contador de generación para descartar resultados obsoletos.
- **Persistencia:** sin preferencias ni base de datos. La salida va a `<ejecutable>/export/`; en el editor, a la raíz del proyecto.
- **Nombres:** nunca se sobrescribe; se añaden `-2`, `-3`, etc.
- **Build:** PCK embebido, presets Linux/Windows y script `scripts/build_portable.sh`.
- **Tema:** paleta carbón/lima, Manrope variable incluida, ilustración y fondos dibujados por código.
- **Marca:** icono SVG rasterizado a PNG y `.ico` con la propia utilidad ICO del proyecto.

## 3. Implementación completada

- Proyecto, escena principal, tema, marca, capturas y utilidades de desarrollo.
- Interfaz completa: bienvenida, editor, panel lateral, comparador, metadatos, avisos y estados.
- Carga por arrastre, selector nativo, formatos adicionales y validaciones.
- Codificación WebP/JPG/PNG, aplanado JPG y conservación de alfa.
- Comparador con foco, teclado, damero y modo ICO.
- Exportación directa a `export/`, nombres únicos, saneado y apertura de carpeta.
- Reinicio y atajos `Ctrl+O`, `Ctrl+E`, `Ctrl+R`.
- Accesibilidad: nombres/descripciones, soporte del proyecto y foco visible.
- Suite headless sin dependencias.
- CI de pruebas en GitHub Actions.
- Builds portables Linux y Windows.
- Documentación: README, proyecto, tareas, completadas, retomar y esta nota.

Tareas cerradas: **t-1 a t-9** (ver `t-completadas.md`).

## 4. Verificación realizada

- **71 comprobaciones, 0 fallos** con:
  `godot --headless --path . --script tests/run_tests.gd`
- Casos cubiertos:
  - codificación WebP/JPG/PNG y decodificación;
  - calidad baja vs. alta;
  - transparencia PNG;
  - aplanado JPG sobre blanco;
  - estructura ICO, cabecera, offsets, frames PNG y tamaños;
  - rechazo de ICO sin tamaños y de formatos no soportados;
  - carga de PNG, BMP generado, TGA generado y SVG;
  - flujo de interfaz completo, modo ICO, reinicio, arrastre y exportación;
  - nombres únicos en una segunda exportación.
- Capturas renderizadas bajo X11 con llvmpipe:
  - `docs/lienzo-inicio.png`
  - `docs/lienzo-editor.png`
  - `docs/lienzo-ico.png`
- Build portable Linux ejecutada desde `dist/Lienzo-0.1.0-linux-x86_64/`.
  - Se abrió con una imagen por argumento.
  - Se envió `Ctrl+E`.
  - El archivo apareció en `dist/.../export/atardecer.webp`, junto al ejecutable.
- Build portable Windows generada (`Lienzo.exe`, PCK embebido, icono de aplicación). No se ejecutó por estar en un entorno Linux.

## 5. Incidencias y aprendizajes

- `LinkButton` ya existe como clase nativa de Godot; no puede usarse como nombre de variación de tema. Se renombró la variación a `TextButton`.
- El entorno de pruebas no tenía fuentes de sistema ni bibliotecas gráficas. La app usa Manrope empaquetada; para capturar fue necesario montar Xvfb, GLX de Mesa y xkb de forma local sin root.
- La ilustración del estado vacío dibujaba la señal de subida bajo una transformación de canvas pendiente; se corrigió reseteando la transformación antes de dibujarla.
- El previsualizador de ICO necesitaba un modo dedicado: comparar una imagen rectangular con un icono cuadrado dejaba el icono recortado por el divisor. Ahora el modo ICO muestra solo el icono centrado.
- A 720 px de alto el panel encajaba con un texto de consejo de una línea; se recortó el copy para evitar contenido cortado.
- El guardado directo en `export/` se validó específicamente sobre la build exportada, no solo en el editor.

## 6. Estado de git

- Repositorio inicializado en la raíz.
- Sin archivos de build ni `export/` de usuario en el seguimiento.
- `.godot/`, `dist/`, `builds/` y el contenido de `export/` están ignorados.

## 7. Próximos pasos

1. **t-10:** orientación EXIF.
2. t-11: redimensionado y encuadre.
3. t-12: procesamiento por lotes.
4. t-13: CI de builds y releases.
5. Resto de tareas en `todo.md`.

No quedan dudas bloqueantes para continuar.
