# Refactorización de arquitectura y persistencia de bloqueos

Fecha: 2026-10-29

## Decisión

Se adopta un monolito modular por features con núcleo funcional y shell
imperativo. No se aplica Clean Architecture completa: la aplicación está
terminada y no se espera que escale. Las reglas completas están en
`docs/arquitectura.md`.

## Cambios estructurales

- `src/app.rs` (2.328 líneas) se divide en `src/app/`:
  - shell y composición en `app/mod.rs`;
  - estado en `app/state.rs`;
  - acciones de UI en `app/actions.rs`;
  - worker en `app/encoding.rs`;
  - sesión en `app/session.rs`;
  - exportación en `app/export.rs`;
  - componentes en `app/ui/`.
- `src/core/` desaparece:
  - `codec/` contiene carga, codificadores, ICO y SVG;
  - `domain/` contiene formatos, color, historial, nombres y errores.
- `src/config.rs` se divide en `settings/mod.rs` y `settings/ini.rs`.
- `OutputFormat` y su política quedan en `domain/format.rs`.
- `Result<_, String>` se sustituye por `LoadError`, `EncodeError`,
  `ExportError`, `ClipboardError` y `ConfigError`.
- `i18n::localize_error(&str)` se sustituye por `i18n::present_error(&AppError)`.
- La UI devuelve `AppAction` para workers, configuración, exportación,
  portapapeles, historial y reinicio.

## Persistencia de bloqueos

Se verificó el flujo real:

1. Se creó `config.ini` con historial vacío.
2. Se muestreó un píxel.
3. Se fijó el color con clic derecho.
4. El archivo quedó así:

```ini
[history]
count=1
color1=#7C7E78FF|1
```

5. Se cerró la aplicación y se volvió a abrir.
6. El color siguió fijado y el candado visible.

Además, `settings/ini.rs` incluye pruebas de ida y vuelta que comparan el
historial completo, incluido el campo `locked`.

## Verificación funcional

- Muestreo y bloqueo con clic derecho: correcto.
- Exportación WebP a `export/demo.webp`: correcta.
- Cambio a ICO con tamaños y preview: correcto.
- Deslizador de calidad y recodificación: correcto (82 % → 1 %, resultado
  actualizado).
- Reinicio desde cabecera: vuelve al estado vacío.
- Cambio de idioma a catalán: persistido en `config.ini`.
- Archivo inválido: aviso localizado y tipado, sin panic.

## Pruebas y calidad

- `cargo test`: **23 pruebas superadas**.
- `cargo fmt --all -- --check`: limpio.
- `cargo clippy --all-targets -- -D warnings`: limpio.

## Interacciones verificadas bajo Xvfb

- Captura del editor con demo.
- Captura del estado vacío tras reiniciar.
- Captura del error de archivo no compatible.
- Captura de ICO.
- Captura de calidad al 1 %.
- Reinicio y cierre limpio.

## Paquetes finales

- Linux: 16.866.416 B.
- Windows: 15.420.416 B.
- `.tar.gz`: 7.536.790 B.
- `.zip`: 6.989.429 B.
- La refactorización no cambia el flujo de codificación ni el uso de memoria
  medido; los valores de referencia siguen en `docs/benchmark-2026-10-28.md`.
