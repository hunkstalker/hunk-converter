# Pulido de cabecera, tarjeta de color y pie

Fecha: 2026-10-29

## Cambios

- Eliminado el botón propio de maximizar/restaurar de la cabecera. La ventana
  usa exclusivamente los controles nativos del sistema.
- El estado maximizado se sincroniza con `config.ini` cuando cambia desde la
  barra de título. En el primer frame solo se toma la referencia para no
  sobrescribir la preferencia guardada durante el arranque.
- La tarjeta del cuentagotas reserva siempre la línea de coordenadas, aunque no
  haya una muestra. El mensaje se limita a una línea y el bloque de códigos
  mantiene filas de altura fija.
- El pie muestra `github.com/hunkstalker/hunk-converter` a la derecha de la
  versión. El enlace usa `OpenUrl` de egui y abre el navegador del sistema.

## Verificación

- Capturas comparadas a 1280 × 1000: el borde superior de la tarjeta queda en
  `y=513` tanto sin muestra como con un color seleccionado; el inferior en
  `y=855` en ambos casos.
- A 1280 × 800 la tarjeta también conserva sus bordes en `y=399` y `y=655`
  con y sin muestra.
- El enlace se probó con un navegador simulado vía `BROWSER`; recibió
  `https://github.com/hunkstalker/hunk-converter` al hacer clic.
- `cargo fmt --check`, `cargo test` (13 pruebas) y
  `cargo clippy --all-targets -- -D warnings` limpios.
- Paquetes regenerados: Linux 16.847.272 B; Windows 15.398.912 B. El enlace
  añade `webbrowser` y deja los imports PE limitados a DLLs del sistema.
