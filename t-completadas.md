# Tareas completadas

> Registro histórico. Las tareas cerradas se retiran de `todo.md` y se conservan aquí.
> Los identificadores no se reutilizan.

## t-1 — Fundaciones del producto y marca

**Descripción:** Crear la estructura base, la identidad visual de hunk-converter, el tema oscuro con acento lima, la tipografía incluida, el icono de aplicación y los estados vacío/editor.

**Completada:** 2026-10-28
**Resultado:** Identidad y base del producto, inicialmente materializadas en Godot 4.7.2 con renderizador GL Compatibility, tema generado en código, pantalla de bienvenida ilustrada y estructura de escenas/scripts.

## t-2 — Carga local de imágenes

**Descripción:** Implementar arrastrar y soltar, selector de archivos, validación de formatos y límites, decodificación de PNG/JPG/WebP/BMP/TGA/SVG y preparación de la vista previa.

**Completada:** 2026-10-28
**Resultado:** Carga desde cualquier parte de la ventana, selector nativo y validaciones de archivo, tamaño y dimensiones. La versión Rust conserva esta funcionalidad y añade rasterización SVG con `resvg`.

## t-3 — Motor de conversión y compresión

**Descripción:** Codificar WebP, JPG y PNG con control de calidad para los formatos con pérdida, conservar transparencia cuando el formato lo permite y aplanar JPG sobre blanco de forma predecible.

**Completada:** 2026-10-28
**Resultado:** Motor de codificación separado de la interfaz, con calidad 1–100, WebP/JPG con pérdida, PNG sin pérdida y aplanado JPG sobre blanco. Reimplementado en Rust en la migración.

## t-4 — Comparador visual antes/después

**Descripción:** Construir una superficie de comparación con divisor arrastrable, etiquetas, damero para transparencia y visualización fiel del resultado codificado. Añadir un modo específico para previsualizar iconos ICO.

**Completada:** 2026-10-28
**Resultado:** Comparador con divisor, damero y modo ICO sin divisor. La vista previa se obtiene decodificando la salida real. La versión Rust añade muestreo de color desde el propio lienzo.

## t-5 — Exportación ICO multirresolución

**Descripción:** Implementar el contenedor ICO con frames PNG, selección de tamaños 16/32/48/64/128/256, ajuste centrado sin recorte y validación de que al menos haya un tamaño.

**Completada:** 2026-10-28
**Resultado:** Exportador ICO con cabecera/directorio estándar, frames RGBA validados y prueba de estructura completa, reimplementado en Rust durante la migración.

## t-6 — Panel de ajustes y metadatos

**Descripción:** Diseñar el panel de archivo, formato de salida, calidad/tamaños ICO, pesos finales y porcentaje de ahorro, con exportación fija al pie.

**Completada:** 2026-10-28
**Resultado:** Panel lateral con formatos, calidad, tamaños ICO, metadatos, comparación de pesos y exportación siempre accesible. La versión Rust lo adapta a 390 px.

## t-7 — Flujo portable y carpeta export/

**Descripción:** Guardar los resultados en una carpeta `export/` junto al ejecutable, evitar sobrescrituras, sanear nombres, abrir la carpeta desde la interfaz y generar builds portables.

**Completada:** 2026-10-28
**Resultado:** Guardado directo en `export/` junto al ejecutable, nombres únicos, saneado de nombres, apertura de carpeta y builds Linux/Windows. Validado en el prototipo y de nuevo en la aplicación Rust.

## t-8 — Robustez, accesibilidad y reinicio

**Descripción:** Añadir procesamiento en segundo plano con descarte de resultados obsoletos, manejo de errores, avisos temporales, reinicio completo y atajos de teclado.

**Completada:** 2026-10-28
**Resultado:** Procesamiento con hilos, agrupación de cambios, descarte de resultados obsoletos, avisos y reinicio. La accesibilidad completa sigue pendiente en t-22 para la versión Rust.

## t-9 — Pruebas, documentación y builds iniciales

**Descripción:** Construir una suite headless, documentar el proyecto, generar capturas, preparar integración continua y producir builds portables de Linux y Windows para el prototipo.

**Completada:** 2026-10-28
**Resultado:** Suite Godot de 71 comprobaciones, capturas, documentación viva, CI, y builds portables Linux/Windows. Esta base sirvió para comparar y decidir la migración.

## t-19 — Cuentagotas de color con códigos múltiples

**Descripción:** Añadir un cuentagotas que permita seleccionar píxeles de la imagen cargada desde la vista previa y mostrar sus códigos HEX, RGB, RGBA y HSL, con copia al portapapeles. El modo debe poder activarse y desactivarse, indicar las coordenadas del píxel, dibujar una marca de muestreo y desactivarse cuando no tenga sentido (vista ICO).

**Completada:** 2026-10-28
**Resultado:** Cuentagotas implementado en el panel inferior de la versión Rust con un icono dedicado. Se activa con un clic o arrastrando desde el icono hasta la imagen, muestrea el píxel original, conserva la posición del divisor, muestra coordenadas y los cuatro códigos con copia, dibuja la marca solo mientras está activo y se desactiva en la vista ICO. También se cancela con `Esc`. Verificado con interacción real bajo Xvfb.

## t-20 — Prueba de concepto Rust + egui

**Descripción:** Crear una PoC separada con Rust y egui que reproduzca el flujo principal (carga, formatos, calidad, comparador, ICO y exportación) y permita comparar consumo y rendimiento con el prototipo Godot.

**Completada:** 2026-10-28
**Resultado:** PoC funcional con 7 pruebas, capturas y benchmark. El ejecutable Linux midió 7,5 MiB frente a 70,7 MiB de Godot, abrió la ventana alrededor de un 28 % antes y redujo la RAM en reposo a la mitad. La comparación motivó la migración definitiva.

## t-21 — Migración a Rust + egui como aplicación principal

**Descripción:** Convertir el piloto en la aplicación principal, completar la paridad imprescindible del producto, reemplazar el selector de color de fondo por el cuentagotas, añadir soporte SVG, renombrar la aplicación a hunk-converter 0.2.0, empaquetar Linux y Windows, actualizar documentación y archivar el prototipo Godot.

**Completada:** 2026-10-28
**Resultado:** Aplicación principal en Rust con egui; UI event-driven sin repintado continuo, comparador, ICO, exportación portable, cuentagotas y SVG. Suite de 13 pruebas Rust, clippy y formato limpios. Builds portables Linux y Windows generadas. El prototipo Godot queda archivado como referencia. La build Windows está generada, pero su ejecución en Windows sigue pendiente en t-23.

## t-25 — Zoom con rueda y paneo del lienzo

**Descripción:** Añadir zoom con la rueda del ratón sobre la imagen, anclado al punto bajo el cursor, y desplazamiento con clic derecho cuando la imagen está ampliada. El divisor, su zona de agarre y el cuentagotas deben seguir funcionando correctamente, y el porcentaje de zoom debe poder restablecerse.

**Completada:** 2026-10-29
**Resultado:** Zoom de 100 % a 800 % con la rueda, anclado al cursor y limitado para no perder la imagen. Paneo con clic derecho y cursor de agarre. El porcentaje aparece en la cabecera y se puede pulsar para volver al ajuste inicial. El divisor, el cuentagotas y la marca de muestreo se adaptan al zoom y al desplazamiento. Verificado con rueda y arrastre reales bajo Xvfb.

## t-26 — Salida AVIF y JPEG XL

**Descripción:** Añadir AVIF y JPEG XL como formatos de salida, con vista previa fiel del archivo codificado, control de calidad cuando corresponda, exportación portable y soporte en las builds Linux y Windows.

**Completada:** 2026-10-29
**Resultado:** AVIF con calidad 1–100 y JPEG XL sin pérdida. PNG y JPEG XL no muestran deslizador porque no modifican la calidad. Las vistas previas se decodifican con `avif-decode` y `jxl-oxide`. Se reorganizó la rejilla de formatos y se corrigieron los badges del comparador. Verificado con pruebas y exportación real AVIF/JPEG XL.

## t-14 — Interfaz en inglés

**Descripción:** Internacionalizar los textos de interfaz, avisos y tooltips, con español como idioma predeterminado e inglés como alternativa. Incluir un selector de idioma y recordar la preferencia.

**Completada:** 2026-10-29
**Resultado:** Interfaz traducida a español, catalán e inglés. Selector en la cabecera, detección del idioma del sistema en el primer arranque y persistencia en `config.ini`.

## t-27 — Historial de colores del cuentagotas

**Descripción:** Añadir un historial de 21 colores integrado en la tarjeta del cuentagotas, con selección por clic izquierdo, bloqueo por clic derecho, limpieza de colores no fijados y persistencia en `config.ini`. Al llenarse, sustituir el color no fijado más antiguo y avisar si todos están fijados.

**Completada:** 2026-10-29
**Resultado:** Historial de 21 colores en tres filas, con candado visible, anillo de selección, papelera para limpiar no fijados y avisos de historial lleno. Verificado con muestreo real, bloqueo, selección y persistencia en config.ini.

## t-28 — Marca Hunk Converter e icono de máscara de gas

**Descripción:** Cambiar el nombre visible a Hunk Converter, el subtítulo a IMAGE TOOLBOX, actualizar el pie de versión y crear un icono original de máscara de gas.

**Completada:** 2026-10-29
**Resultado:** Cabecera «Hunk Converter / IMAGE TOOLBOX», pie «Rust + egui · Open Source · v0.2.0», icono SVG original de máscara de gas rasterizado a PNG y usado en la ventana y la cabecera.

## t-29 — Ventana maximizada persistente

**Descripción:** Abrir la aplicación con la ventana maximizada por defecto, permitir maximizar o restaurar con un botón y guardar el estado en `config.ini`.

**Completada:** 2026-10-29
**Resultado:** La app arranca con la ventana maximizada salvo que `config.ini` indique lo contrario. El icono de la cabecera maximiza o restaura y guarda el estado. En desarrollo, `HUNK_WINDOWED=1` fuerza una ventana normal.
