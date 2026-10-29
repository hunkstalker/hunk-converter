# 2026-10-29 — Marca, idiomas e historial de colores

> Evidencia de sesión. No editar.

## Marca e icono

- Nombre visible: **Hunk Converter**.
- Subtítulo: **IMAGE TOOLBOX**.
- Pie: `Rust + egui · Open Source · v0.2.0`; la versión sale de `CARGO_PKG_VERSION`.
- Nuevo icono original de máscara de gas en `assets/branding/icon.svg`,
  rasterizado a `icon_256.png` con `src/bin/make_icon.rs`.
- El icono se usa en la ventana y en la cabecera.

## Idiomas y configuración

- Selector ES / CA / EN en la cabecera.
- Primer arranque: se detecta el idioma con `sys-locale` y se guarda en
  `config.ini` junto al ejecutable.
- El cambio de idioma se guarda inmediatamente.
- La interfaz, los avisos, los tooltips y los textos de error habituales se
  traducen a los tres idiomas.
- `config.ini` guarda:
  - `[general] language`;
  - `[history] colorN=#RRGGBBAA|locked`.
- Si no se puede escribir el archivo, la aplicación sigue funcionando.

## Historial de colores

- Integrado en la tarjeta del cuentagotas, no es una tarjeta independiente.
- 21 colores en tres filas de 7 muestras de 32 px. La tarjeta queda anclada
  abajo, con los ajustes en un área con scroll por encima y Exportar fijo.
- Clic izquierdo: selecciona el color, actualiza los códigos y desactiva el
  cuentagotas. No muestra coordenadas ni marca de píxel.
- Clic derecho: fija o libera el color. Las muestras fijadas llevan candado y no
  se reemplazan.
- Si el historial está lleno, se sustituye el color no fijado más antiguo. Si
  todos están fijados, se avisa y el color no entra.
- El botón de papelera elimina los colores no fijados.
- El historial y los bloqueos persisten en `config.ini`.
- Se añadieron pruebas para inserción, duplicados, reemplazo y historial lleno.

## Otros ajustes

- El clic derecho sobre la imagen desactiva el cuentagotas.
- El botón **Cambiar** tiene fondo y cursor de mano al pasar por encima, además
  de tooltip.
- Los códigos de color usan botones de icono de portapapeles y muestran un
  aviso temporal al copiar.
- Se eliminó el pill «100 % LOCAL» y el texto de resolución junto a «Vista
  previa».
- La reducción de peso aparece como texto junto al resultado, sin pill.
- La barra lateral mantiene los ajustes con scroll y ancla la tarjeta del color
  y el botón de exportar abajo.
- Los toasts se miden con el texto real, con ancho máximo y ajuste de varias
  líneas.
- La aplicación arranca con la ventana maximizada. Los controles nativos
  maximizan o restauran, y el estado se guarda en `config.ini`.

## Verificación

- `cargo test`: **13 pruebas superadas**.
- `cargo clippy --all-targets -- -D warnings`: limpio.
- `cargo fmt --all -- --check`: limpio.
- Detección inicial catalana probada con `LANG=ca_ES.UTF-8`: `config.ini`
  guardó `language=ca`.
- Cambio a inglés probado desde la interfaz: `config.ini` guardó `language=en`.
- Muestreo, bloqueo y persistencia del historial verificados con Xvfb.
- Aviso «HEX copiado» verificado con clic real.
- Clic derecho sobre la imagen verificado: el cuentagotas se desactiva y la
  marca desaparece.

## Medidas finales

- Ejecutable Linux: 16.847.272 B (16,1 MiB).
- Ejecutable Windows: 15.398.912 B (14,7 MiB).
- Arranque a ventana: 200,2 ms de mediana (20 arranques).
- RSS en reposo: 102,0 MiB.
- RSS con imagen 12 MP: 212,6 MiB.
- El paquete Linux creó `config.ini` junto al ejecutable en el primer
  arranque y detectó el idioma catalán.
