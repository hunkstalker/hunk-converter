# Hunk Converter

**Convierte, comprime, compara, muestrea colores y exporta imágenes sin salir de tu equipo.**

Hunk Converter es una aplicación de escritorio local-first escrita en **Rust + egui**. Arrastra una imagen, elige el formato de salida, ajusta la calidad y comprueba el resultado real con un comparador deslizante antes de guardarlo. También genera iconos `.ico` multirresolución.

La interfaz solo repinta cuando hay actividad. No mantiene un bucle de render continuo mientras miras una imagen estática.

Todo el procesamiento ocurre en tu ordenador: no hay cuentas, subidas ni conexión a internet.

![Editor de Hunk Converter](docs/hunk-converter-editor.png)

## Características

- **Arrastrar y soltar** en cualquier parte de la ventana, o selector nativo con `Ctrl+O`.
- **Pegar desde el portapapeles** con `Ctrl+V`, haya o no una imagen cargada.
- **Entrada:** PNG, JPG/JPEG, WebP, BMP, TGA y SVG.
- **Salida:** WebP, AVIF, JPG, PNG, JPEG XL e ICO.
- **Calidad ajustable** de 1 a 100 para WebP, JPG y AVIF (82 % por defecto).
- **Comparador visual** antes/después con divisor arrastrable y soporte de teclado.
- **Zoom y paneo:** rueda del ratón sobre la imagen y clic derecho para desplazarte.
- **Vista previa fiel al archivo final**: se decodifica el resultado codificado real.
- **Cuentagotas:** pulsa el icono y haz clic o arrastra sobre la imagen para capturar un color. La tarjeta mantiene una altura fija al mostrar coordenadas y códigos.
- **Códigos simultáneos:** HEX, RGB, RGBA y HSL, con botones de copia y confirmación visual.
- **Historial de colores:** 21 colores en tres filas de siete, con bloqueo y limpieza.
- **Idiomas:** español, catalán e inglés, con detección del sistema y preferencia guardada.
- **Ventana maximizada:** se abre maximizada; los controles nativos permiten maximizar o restaurar y el estado se guarda.
- **Exportación ICO** con selección de 16, 32, 48, 64, 128 y 256 px en un solo archivo.
- **PNG y JPEG XL sin pérdida**, con transparencia conservada en WebP/AVIF/PNG/JPEG XL/ICO.
- **JPG sobre fondo blanco** cuando la imagen tiene transparencia.
- **Guardado directo** en `export/`, junto al ejecutable, sin sobrescribir archivos.
- **Reinicio completo** con `Ctrl+R`.
- **Portable:** builds de Linux y Windows, sin instalador.
- **Local y privado:** cero llamadas de red.
- **Render bajo demanda:** sin CPU de UI en reposo.
- **Enlace al repositorio:** el pie abre `github.com/hunkstalker/hunk-converter` en el navegador.

## Cuentagotas

1. Carga una imagen.
2. Pulsa el **icono de cuentagotas** en la tarjeta inferior.
3. Haz clic sobre la imagen o arrastra desde el icono hasta el píxel que quieras.
4. Copia cualquiera de sus códigos con el icono de portapapeles.

El icono vuelve a pulsarse para desactivar el modo. También puedes cancelarlo con `Esc` o con clic derecho sobre la imagen. La marca de muestreo solo se muestra mientras el cuentagotas está activo.

La muestra se toma del píxel original, no del resultado recompimido, y conserva el canal alfa. En la vista ICO el cuentagotas se desactiva porque esa vista representa el icono generado, no la imagen de origen.

![Cuentagotas de Hunk Converter](docs/hunk-converter-cuentagotas.png)

## Historial de colores

- Se guardan automáticamente los últimos **21 colores** muestreados.
- **Clic izquierdo** en una muestra: usa ese color y actualiza los códigos.
- **Clic derecho** en una muestra: la fija o la libera. Las muestras fijadas llevan un candado y no se reemplazan.
- Cuando el historial está lleno, se sustituye el color no fijado más antiguo.
- Si todos los colores están fijados, el color nuevo se muestra pero no entra al historial; aparece un aviso.
- El botón de papelera limpia los colores no fijados.
- El historial y el estado de cada candado se conservan en `config.ini`, junto al ejecutable. Al reiniciar, los colores fijados siguen fijados.

## Idiomas y configuración

- Selector **ES / CA / EN** en la cabecera.
- En el primer arranque se detecta el idioma del sistema.
- El idioma elegido, el historial, los bloqueos y el estado maximizado se guardan en `config.ini`, junto al ejecutable. La ventana usa los controles nativos del sistema.
- Si no se puede escribir el archivo, la aplicación sigue funcionando sin persistencia.

## Zoom y navegación

- **Rueda del ratón** sobre la imagen: acerca y aleja manteniendo el punto bajo el cursor.
- **Clic derecho y arrastra**: desplaza la imagen cuando está ampliada.
- El porcentaje de zoom aparece en la cabecera de la vista previa; pulsa sobre él para volver al ajuste inicial.
- El divisor y el cuentagotas siguen funcionando con cualquier zoom.

![Zoom y paneo de Hunk Converter](docs/hunk-converter-zoom.png)

## Capturas

| Inicio | Vista ICO |
| --- | --- |
| ![Pantalla de inicio](docs/hunk-converter-inicio.png) | ![Vista de icono](docs/hunk-converter-ico.png) |

Las capturas usan imágenes de muestra generadas por el propio proyecto.

## Ejecutar desde el código

Requisitos:

- **Rust 1.99.1** (fijado en `rust-toolchain.toml`).
- Un compilador de C para `libwebp-sys`.
- **NASM** en x86/x86_64 para el decodificador AVIF (`rav1d`).
- En Linux, las bibliotecas gráficas habituales de X11/OpenGL.
- En Windows, MSVC Build Tools o un toolchain GNU.

```bash
cargo run --release -- /ruta/a/imagen.jpg
# o abre la ventana y arrastra una imagen
cargo run --release
```

## Uso

1. Arrastra una imagen a la ventana, pulsa **Seleccionar una imagen** o usa `Ctrl+V`.
2. Elige **WebP**, **JPG**, **PNG** o **ICO**.
3. Ajusta la calidad o los tamaños del icono.
4. Arrastra el divisor para comparar. Usa la rueda para acercar y el clic derecho para moverte por la imagen.
5. Activa el cuentagotas si necesitas extraer colores.
6. Pulsa **Exportar** o `Ctrl+E`.

El archivo aparece en `export/`, junto a la aplicación. Si el nombre existe, se añade `-2`, `-3`, etc.

### Atajos

| Atajo | Acción |
| --- | --- |
| `Ctrl+O` | Abrir imagen |
| `Ctrl+V` | Pegar una imagen del portapapeles |
| `Ctrl+E` | Exportar |
| `Ctrl+R` | Reiniciar la aplicación |
| `←` / `→` | Mover el divisor del comparador |
| `Inicio` / `Fin` | Llevar el divisor a un extremo |

## Formatos

| Formato | Entrada | Salida | Transparencia | Calidad |
| --- | :---: | :---: | :---: | :---: |
| WebP | ✓ | ✓ | Sí | 1–100 |
| AVIF | — | ✓ | Sí | 1–100 |
| JPG / JPEG | ✓ | ✓ | No (fondo blanco) | 1–100 |
| PNG | ✓ | ✓ | Sí | Sin pérdida |
| JPEG XL | — | ✓ | Sí | Sin pérdida |
| ICO | — | ✓ | Sí | Tamaños fijos |
| BMP | ✓ | — | Según archivo | — |
| TGA | ✓ | — | Según archivo | — |
| SVG | ✓ | — | Rasterizado | — |

Los SVG se rasterizan con `resvg` al tamaño declarado. AVIF y JPEG XL se añaden como formatos de salida; JPEG XL se codifica sin pérdida. PNG y JPEG XL no muestran deslizador porque no modifican la calidad de la imagen. El límite de entrada es de 200 MB y 40 megapíxeles (máximo 16 384 px por lado).

## Builds portables

Cada build es una carpeta autocontenida. La salida queda en `export/`, junto al ejecutable.

```text
hunk-converter-0.2.0-windows-x86_64/
├── hunk-converter.exe
├── config.ini          (se crea al primer arranque)
├── export/
├── LEEME.txt
├── LICENSE.txt
└── LICENCIAS/
    └── Manrope-OFL.txt
```

```bash
./scripts/build_dist.sh linux
./scripts/build_dist.sh windows
./scripts/build_dist.sh all
```

Los resultados se generan en `dist/`:

- Linux: carpeta y `.tar.gz`.
- Windows: carpeta y `.zip` si `zip` está instalado.

La build Windows se ha generado, pero todavía no se ha ejecutado en un Windows real. La build Linux se validó bajo X11, incluyendo la exportación a su carpeta `export/`.

## Pruebas

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
```

La suite cubre 23 pruebas: codificación WebP/JPG/PNG/AVIF/JPEG XL, historial y bloqueos, configuración, calidad, transparencia, aplanado JPG, estructura ICO, códigos de color, rasterización SVG y nombres de exportación. También se ejecuta en GitHub Actions.

## Rendimiento

La versión Rust/egui se comparó con el prototipo Godot antes de migrar. En el mismo entorno Linux y con la misma imagen, el binario, la memoria, el arranque y los tiempos de codificación son claramente menores. El informe completo, la metodología y las limitaciones están en:

[`docs/benchmark-2026-10-28.md`](docs/benchmark-2026-10-28.md)

## Estructura

```text
.
├── assets/                 Iconos, tipografía y licencias
├── docs/                   Capturas, benchmark y arquitectura
├── notes/                  Evidencia fechada de las sesiones
├── scripts/build_dist.sh   Empaquetado portable
├── src/
│   ├── app/                Shell, estado, acciones y componentes egui
│   │   ├── actions.rs      Efectos explícitos devueltos por la UI
│   │   ├── encoding.rs     Worker de codificación y revisiones
│   │   ├── export.rs       Exportación portable sin sobrescritura
│   │   ├── session.rs      Carga, pegado, reinicio y formato
│   │   ├── state.rs        SourceImage, RenderedResult, Toast
│   │   └── ui/             Cabecera, sidebar, color, preview y widgets
│   ├── codec/              WebP, JPG, PNG, AVIF, JPEG XL, ICO y SVG
│   ├── domain/             Formatos, color, historial y errores tipados
│   ├── settings/           config.ini, idioma, historial y maximizado
│   ├── bin/                Benchmark, muestra y generador de icono
│   ├── demo.rs             Imagen sintética para capturas
│   ├── i18n.rs             Textos en español, catalán e inglés
│   └── theme.rs            Tema visual
├── proyecto.md             Documento vivo
├── todo.md                 Tareas pendientes
├── t-completadas.md        Tareas cerradas
└── retomar.md              Punto de entrada para nuevas sesiones
```

## Documentación

- [proyecto.md](proyecto.md) — producto, funcionalidad y arquitectura.
- [todo.md](todo.md) — tareas pendientes.
- [t-completadas.md](t-completadas.md) — tareas cerradas.
- [retomar.md](retomar.md) — estado y contexto para continuar.
- [notes/](notes/) — evidencia fechada.
- [docs/benchmark-2026-10-28.md](docs/benchmark-2026-10-28.md) — benchmark comparativo.
- [docs/arquitectura.md](docs/arquitectura.md) — decisiones de arquitectura y reglas de código.

El prototipo Godot original se conserva como archivo histórico en `../Prototipo`.

## Licencia

Código bajo [MIT](LICENSE). La tipografía Manrope se distribuye bajo la SIL Open Font License 1.1; su licencia está incluida en [assets/fonts/OFL.txt](assets/fonts/OFL.txt).
