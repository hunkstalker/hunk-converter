# Tareas pendientes

> Convención: cada tarea usa un identificador `t-N`, un título y una descripción.
> Al completarse se retira de este archivo y se registra en `t-completadas.md`.
> Las tareas se listan en orden ascendente y los identificadores no se reutilizan.

## t-10 — Respetar la orientación EXIF

**Descripción:** Leer la etiqueta `Orientation` de las imágenes JPEG/WebP que la incluyan y aplicar la rotación o el volteo correspondiente antes de mostrar, comparar, muestrear y exportar. Añadir pruebas con imágenes de orientación 1, 3, 6 y 8 para evitar fotos de móvil giradas.

## t-11 — Redimensionado y encuadre opcionales

**Descripción:** Añadir controles para cambiar el tamaño de salida por píxeles o porcentaje, con opción de mantener proporción y de elegir un método de interpolación. Para ICO, permitir alternar entre ajuste centrado y recorte cuadrado. La exportación sin tocar estos ajustes debe conservar el comportamiento actual.

## t-12 — Conversión por lotes

**Descripción:** Permitir arrastrar varias imágenes y procesarlas con un mismo formato y calidad, mostrando progreso y resultados individuales. Los archivos deben guardarse en `export/` sin sobrescribir y con nombres únicos, igual que en el flujo de una sola imagen.

## t-13 — Automatizar builds y publicaciones

**Descripción:** Extender la integración continua para ejecutar la suite, compilar las builds portables de Linux y Windows y publicar artefactos con checksums al crear una etiqueta de versión. Mantener la build sin instalador y la carpeta `export/` junto al ejecutable.

## t-14 — Interfaz en inglés

**Descripción:** Internacionalizar todos los textos de interfaz, avisos, tooltips y documentación de usuario, con español como idioma predeterminado e inglés como alternativa. Incluir un selector de idioma y recordar la preferencia de forma local.

## t-15 — Entrada HEIC y códecs opcionales

**Descripción:** Evaluar HEIC como formato de entrada manteniendo la portabilidad y sin introducir llamadas de red. AVIF y JPEG XL ya están integrados como salida (`t-26`); queda valorar si los códecs pesados deben poder excluirse mediante features de Cargo y documentar el impacto en tamaño y compilación.

## t-16 — Navegación avanzada del lienzo

**Descripción:** Completar la navegación con un modo 1:1, zoom mediante botones o gestos además de la rueda y atajos de encuadre (ajustar, restablecer). El divisor, su círculo y el cuentagotas ya funcionan a cualquier zoom; el encuadre debe conservarse al cambiar de formato.

## t-17 — Build de macOS

**Descripción:** Añadir un preset de exportación para macOS, documentar la ubicación de la carpeta `export/` dentro de la estructura de la aplicación y preparar firma/notarización opcional para distribución fuera de la App Store.

## t-18 — Selector avanzado de carpeta de salida

**Descripción:** Mantener `export/` junto al ejecutable como destino predeterminado y añadir una opción avanzada para elegir otra carpeta de forma puntual o recordada. Debe seguir evitando sobrescrituras accidentales y avisar si el destino no es escribible.

## t-22 — Accesibilidad y navegación completa por teclado

**Descripción:** Incorporar soporte de accesibilidad nativa de egui/AccessKit, orden de foco explícito, nombres y descripciones para lector de pantalla, navegación completa sin ratón y pruebas de contraste y escalado de fuente. El cuentagotas y el comparador deben poder usarse con teclado.

## t-23 — Validar la build Windows en un equipo real

**Descripción:** Ejecutar el `.exe` portable en Windows 10 y 11, comprobar arranque, high DPI, carga por arrastre, diálogo nativo, exportación a `export/` junto al ejecutable y cierre limpio. Revisar también que el icono del ejecutable y los imports del sistema sean correctos.

## t-24 — Icono nativo del ejecutable Windows

**Descripción:** Incrustar el icono de hunk-converter en los metadatos del `.exe` de Windows, además del icono de ventana que ya se carga en tiempo de ejecución. Documentar el proceso de build para que siga siendo reproducible en Linux y Windows.
