# Tareas pendientes

> Convención: cada tarea usa un identificador `t-N`, un título y una descripción.
> Al completarse se retira de este archivo y se registra en `t-completadas.md`.
> Las tareas se listan en orden ascendente y los identificadores no se reutilizan.

## t-11 — Redimensionado y encuadre opcionales

**Descripción:** Añadir controles para cambiar el tamaño de salida por píxeles o porcentaje, con opción de mantener proporción y de elegir un método de interpolación. Para ICO, permitir alternar entre ajuste centrado y recorte cuadrado. La exportación sin tocar estos ajustes debe conservar el comportamiento actual.

## t-12 — Conversión por lotes

**Descripción:** Permitir arrastrar varias imágenes y procesarlas con un mismo formato y calidad, mostrando progreso y resultados individuales. Los archivos deben guardarse en `export/` sin sobrescribir y con nombres únicos, igual que en el flujo de una sola imagen.

## t-13 — Validar CI y publicación de la release

**Descripción:** El workflow `.github/workflows/release.yml` ya compila Linux Debian x86_64, Windows 10/11 x86_64 y macOS universal, genera checksums y publica la GitHub Release al empujar un tag. Falta validar el primer release real y cerrar la tarea.

## t-15 — Entrada HEIC y códecs opcionales

**Descripción:** Evaluar HEIC como formato de entrada manteniendo la portabilidad y sin introducir llamadas de red. AVIF y JPEG XL ya están integrados como salida (`t-26`); queda valorar si los códecs pesados deben poder excluirse mediante features de Cargo y documentar el impacto en tamaño y compilación.

## t-16 — Navegación avanzada del lienzo

**Descripción:** Completar la navegación con un modo 1:1, zoom mediante botones o gestos además de la rueda y atajos de encuadre (ajustar, restablecer). El divisor, su círculo y el cuentagotas ya funcionan a cualquier zoom; el encuadre debe conservarse al cambiar de formato.

## t-18 — Selector avanzado de carpeta de salida

**Descripción:** Mantener `export/` junto al ejecutable como destino predeterminado y añadir una opción avanzada para elegir otra carpeta de forma puntual o recordada. Debe seguir evitando sobrescrituras accidentales y avisar si el destino no es escribible.

## t-22 — Accesibilidad y navegación completa por teclado

**Descripción:** Incorporar soporte de accesibilidad nativa de egui/AccessKit, orden de foco explícito, nombres y descripciones para lector de pantalla, navegación completa sin ratón y pruebas de contraste y escalado de fuente. El cuentagotas y el comparador deben poder usarse con teclado.

## t-23 — Validar la build Windows 10/11 de la release

**Descripción:** Ejecutar el `.zip` portable de la release en Windows 10 y 11 y comprobar arranque, high DPI, carga por arrastre, diálogo nativo, exportación a `export/`, cierre limpio e imports del sistema. Incluye la verificación visual del icono en Explorer, barra de tareas y accesos directos (absorbe la antigua `t-24`).
