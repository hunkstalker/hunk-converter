# Arquitectura de Hunk Converter

> Estado: aceptada · Fecha: 2026-10-29 · Versión: 0.2.0

## 1. Contexto

Hunk Converter es una aplicación de escritorio terminada: convierte, comprime,
compara y muestrea imágenes. Tiene unos 4.300 líneas de Rust, seis formatos de
salida y no se espera que crezca de forma significativa.

El objetivo de esta arquitectura no es preparar la aplicación para una escala
que no va a tener. Es conseguir que el código sea fácil de leer, probar y
modificar sin miedo, eliminando el antiguo `app.rs` monolítico de más de 2.300
líneas y las cadenas de error sin tipo.

## 2. Decisión

Se adopta un **monolito modular por features, con núcleo funcional y shell
imperativo**.

```text
main.rs
  └── app/                  shell: composición, estado y eframe::App
        ├── actions.rs      efectos explícitos que devuelve la UI
        ├── encoding.rs     worker de codificación y revisiones
        ├── export.rs       escritura portable y nombres únicos
        ├── session.rs      carga, pegado, reinicio y formato
        ├── state.rs        DTOs de estado de la aplicación
        └── ui/             componentes egui
              ├── header.rs
              ├── footer.rs
              ├── sidebar.rs
              ├── format_picker.rs
              ├── quality.rs
              ├── ico_sizes.rs
              ├── color_card.rs
              ├── preview.rs
              ├── canvas.rs
              ├── empty.rs
              ├── toast.rs
              ├── history.rs
              ├── icons.rs
              └── widgets.rs

domain/                     lógica pura y tipos del negocio
  ├── color.rs
  ├── error.rs
  ├── export_name.rs
  ├── format.rs
  └── history.rs

codec/                      implementaciones concretas de códecs
  ├── mod.rs
  ├── encoders.rs
  ├── ico.rs
  └── svg.rs

settings/                   persistencia de config.ini
  ├── mod.rs
  └── ini.rs

i18n.rs                     textos es/ca/en y presentación de errores
theme.rs                    paleta, tipografía y estilos
demo.rs                     imagen sintética para desarrollo
```

### Regla de dependencias

```text
ui ──► app ──► codec / settings
        │
        └──► domain ◄── codec / settings

domain no conoce egui, ni ficheros, ni hilos.
ui no escribe ficheros, no lanza workers y no guarda configuración.
app es el único sitio que conecta estado, UI y efectos.
codec y settings contienen implementaciones concretas.
```

No se crea una capa de puertos y adaptadores para cada operación. La app está
terminada; añadir un trait por cada función o un contenedor de dependencias
sería ceremonia sin retorno. La inversión de dependencias se aplica donde
aporta valor: el dominio no depende de la UI y los errores de infraestructura
se traducen a errores de dominio antes de llegar a la interfaz.

## 3. Aplicación de SOLID

| Principio | Decisión concreta |
| --- | --- |
| **SRP** | `app.rs` se divide por responsabilidad: sesión, codificación, exportación, estado y componentes de UI. `settings` separa modelo y parser INI. `codec` separa carga, codificadores, ICO y SVG. |
| **OCP** | La política de formatos vive en `domain/format.rs` (extensión, pérdida, badge). Añadir un formato toca su política, su codificador y su traducción; no obliga a cambiar toda la UI. |
| **LSP** | Todos los codificadores cumplen el mismo contrato: reciben `&RgbaImage` y `EncodeOptions`, devuelven `EncodedOutput` o `EncodeError`. No hay `panic!` ni ramas con un tipo de retorno distinto. |
| **ISP** | No existe un servicio genérico de imagen. Las funciones y DTOs son pequeños: `EncodeOptions`, `SourceInput`, `FileCard`, `FormatCard`, `EyedropperButton`, `HistorySwatch`, `Badge`, `CanvasContext`, `ZoomInput`. |
| **DIP** | `domain` no importa `egui` ni el sistema de ficheros. `ui` no importa `std::fs`. Los errores concretos de `image`, `png`, `avif-decode`, `jxl-oxide`, `arboard` o `std::io` se convierten en `LoadError`, `EncodeError`, `ClipboardError`, `ExportError` o `ConfigError`. |

## 4. Reglas de código

### Funciones y parámetros

- Máximo **3 parámetros además de `self`**.
- Si hacen falta más, se agrupan en un DTO: `FormatCard`, `HistorySwatch`,
  `Badge`, `EyedropperButton`, `SourceInput`, `CanvasContext`, `ZoomInput`.
- Funciones de dominio cortas y sin efectos: historial, formatos, nombres de
  exportación, color.
- Funciones de UI orientadas a una sola responsabilidad. Las que superan un
  tamaño razonable se dividen por sección visual.
- Sin `utils.rs` ni `helpers.rs` genéricos.

### Efectos secundarios

Las funciones de UI pueden mutar estado visual local (calidad, zoom, paneo,
muestra, formato) porque reciben `&mut self`. Los efectos de aplicación se
devuelven como acciones explícitas:

```rust
enum AppAction {
    RequestEncode(Option<Duration>),
    PersistSettings,
    Export,
    OpenFileDialog,
    Reset,
    RememberColor([u8; 4]),
    ToggleHistoryLock(usize),
    ClearHistory,
    CopyColor { label: String, text: String },
    RequestRepaint,
}
```

Flujo:

```text
estado + entrada → componente UI → Vec<AppAction>
                                      ↓
                           HunkApp::apply_actions
                                      ↓
                 worker / disco / portapapeles / configuración
```

Esto elimina los efectos escondidos: una función de pintado ya no escribe
`config.ini`, no copia al portapapeles y no lanza hilos.

### Errores

No se usa `Result<_, String>` ni se hace *matching* contra mensajes en español.

```rust
enum LoadError { ... }
enum EncodeError { ... }
enum ExportError { ... }
enum ClipboardError { ... }
enum ConfigError { ... }

enum AppError {
    Load(LoadError),
    Encode(EncodeError),
    Export(ExportError),
    Clipboard(ClipboardError),
    Config(ConfigError),
}
```

`i18n::present_error` traduce variantes tipadas, no cadenas. Los errores de
infraestructura conservan su `source` y se muestran con detalle. No hay
`catch` vacíos: los avisos de configuración se registran y los fallos de
guardado se registran en `stderr`.

### Nombres

- Identificadores de código en inglés; textos de interfaz en `i18n`.
- Nombres de dominio explícitos: `SourceImage`, `RenderedResult`,
  `EncodeCompletion`, `HistoryEntry`, `OutputFormat`.
- Nada de nombres genéricos como `data`, `manager`, `helper` o `process`.

### Estado

- `SourceImage`: imagen original, metadatos y textura.
- `RenderedResult`: resultado codificado y textura de preview.
- `EncodeJobResult` / `EncodeCompletion`: mensajes del worker.
- `Toast`: aviso temporal.
- La revisión (`revision`) descarta resultados obsoletos cuando el usuario
  cambia de formato o calidad mientras se codifica.

## 5. Persistencia y bloqueos

`config.ini` se guarda junto al ejecutable:

```ini
[general]
language=es
maximized=1

[history]
count=2
color1=#7C7E78FF|1
color2=#B38E75FF|0
```

Decisiones:

- El último campo de cada color es el bloqueo (`1` fijado, `0` libre).
- Los colores fijados se conservan entre sesiones y al limpiar el historial.
- Al llenarse el historial se sustituye el color libre más antiguo.
- `maximized` sustituye al antiguo `fullscreen`; el valor antiguo se acepta
  para no romper configuraciones existentes.
- El parser devuelve advertencias con número de línea para entradas inválidas,
  en lugar de ignorarlas en silencio.
- La persistencia de bloqueos está cubierta por pruebas de ida y vuelta.

## 6. Pruebas

La suite cubre:

- reglas del historial y bloqueos;
- ida y vuelta de `config.ini`, incluidos colores fijados;
- compatibilidad con `fullscreen`;
- advertencias de configuración inválida;
- política de formatos;
- nombres de exportación y no sobrescritura;
- códecs WebP/JPG/PNG/AVIF/JPEG XL;
- transparencia PNG/JPEG XL y aplanado JPG;
- rasterización SVG;
- estructura ICO;
- códigos de color.

Estado actual: **23 pruebas**, `cargo fmt --check` y
`cargo clippy --all-targets -- -D warnings` limpios.

## 7. Decisiones deliberadamente descartadas

- **Clean Architecture completa / Hexagonal estricta:** demasiados traits,
  DTOs y adaptadores para una app cerrada.
- **Un trait por códec:** los códecs son estables; se prueban directamente y
  comparten el contrato de `EncodeError`.
- **Contenedor de dependencias:** el grafo es pequeño y se construye en
  `main.rs`/`HunkApp::new`.
- **Múltiples crates:** no aporta aislamiento real y complica el empaquetado
  portable.
- **`anyhow` en dominio:** difumina los errores que la UI necesita traducir.
- **`thiserror`:** no se añade como dependencia; los enums implementan
  `Display` y `Error` a mano para mantener el árbol de dependencias mínimo.
- **Mover `i18n` y `theme` a `ui/`:** se mantienen como módulos compartidos de
  presentación porque los usan varios componentes y no contienen lógica de
  negocio.
- **Reescribir la UI desde cero:** se conservó el comportamiento y se dividió
  por componentes.

## 8. Mapa de migración

| Antes | Ahora |
| --- | --- |
| `src/app.rs` | `src/app/mod.rs` + `src/app/ui/*` + `src/app/{encoding,session,export,state,actions}.rs` |
| `src/config.rs` | `src/settings/mod.rs` + `src/settings/ini.rs` |
| `src/core/codec.rs` | `src/codec/mod.rs` + `src/codec/encoders.rs` + `src/codec/svg.rs` |
| `src/core/ico.rs` | `src/codec/ico.rs` |
| `src/core/color.rs` | `src/domain/color.rs` |
| Reglas del historial en `config.rs` | `src/domain/history.rs` |
| `safe_stem` en `app.rs` | `src/domain/export_name.rs` |
| `OutputFormat` en `codec.rs` | `src/domain/format.rs` |
| `localize_error(String)` | `i18n::present_error(&AppError)` |
