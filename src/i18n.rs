use crate::domain::error::{
    AppError, ClipboardError, ConfigError, EncodeError, ExportError, IcoError, LoadError,
};
use crate::domain::format::OutputFormat;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    Es,
    Ca,
    En,
}

impl Lang {
    pub const ALL: [Self; 3] = [Self::Es, Self::Ca, Self::En];

    pub fn code(self) -> &'static str {
        match self {
            Self::Es => "es",
            Self::Ca => "ca",
            Self::En => "en",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Es => "Español",
            Self::Ca => "Català",
            Self::En => "English",
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            Self::Es => "ES",
            Self::Ca => "CA",
            Self::En => "EN",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        let code = code
            .split(['-', '_'])
            .next()
            .unwrap_or(code)
            .to_ascii_lowercase();
        match code.as_str() {
            "es" | "spa" => Some(Self::Es),
            "ca" | "cat" => Some(Self::Ca),
            "en" | "eng" => Some(Self::En),
            _ => None,
        }
    }

    pub fn detect() -> Self {
        sys_locale::get_locale()
            .as_deref()
            .and_then(Self::from_code)
            .unwrap_or(Self::En)
    }
}

macro_rules! catalogue {
    ($( $key:ident => ($es:expr, $ca:expr, $en:expr) ),* $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum K {
            $( $key ),*
        }

        pub fn t(lang: Lang, key: K) -> &'static str {
            match key {
                $( K::$key => match lang {
                    Lang::Es => $es,
                    Lang::Ca => $ca,
                    Lang::En => $en,
                } ),*
            }
        }
    };
}

catalogue! {
    Reset => ("Reiniciar", "Reinicia", "Reset"),
    Language => ("Idioma", "Idioma", "Language"),
    OpenRepository => (
        "Abrir el repositorio en el navegador",
        "Obre el repositori al navegador",
        "Open the repository in your browser"
    ),
    EmptyTitle => ("Suelta una imagen.\nHazla tuya.", "Deixa anar una imatge.\nFes-la teva.", "Drop an image.\nMake it yours."),
    EmptySubtitle => (
        "Convierte, comprime y exporta sin salir de tu equipo.",
        "Converteix, comprimeix i exporta sense sortir del teu equip.",
        "Convert, compress and export without leaving your computer."
    ),
    SelectImage => ("Seleccionar una imagen", "Selecciona una imatge", "Select an image"),
    PrivacyNote => (
        "Nada se sube a internet. Nada sale de esta ventana.",
        "Res no es puja a internet. Res no surt d'aquesta finestra.",
        "Nothing is uploaded. Nothing leaves this window."
    ),
    Image => ("IMAGEN", "IMATGE", "IMAGE"),
    Change => ("Cambiar", "Canvia", "Change"),
    ChangeTooltip => ("Seleccionar otra imagen", "Selecciona una altra imatge", "Select another image"),
    OutputFormat => ("FORMATO DE SALIDA", "FORMAT DE SORTIDA", "OUTPUT FORMAT"),
    Quality => ("CALIDAD", "QUALITAT", "QUALITY"),
    Lossless => ("Sin pérdida", "Sense pèrdua", "Lossless"),
    QualityHint => (
        "Más calidad conserva más detalle y aumenta el peso.",
        "Més qualitat conserva més detall i augmenta el pes.",
        "Higher quality keeps more detail and increases file size."
    ),
    LosslessHint => (
        "Este formato conserva todos los píxeles; no necesita ajuste de calidad.",
        "Aquest format conserva tots els píxels; no necessita ajust de qualitat.",
        "This format preserves every pixel; no quality adjustment is needed."
    ),
    IconSizes => ("TAMAÑOS DEL ICONO", "MIDES DE LA ICONA", "ICON SIZES"),
    IcoCentered => (
        "Imagen centrada en un lienzo transparente, sin recortar.",
        "Imatge centrada en un llenç transparent, sense retallar.",
        "Image centered on a transparent canvas, without cropping."
    ),
    SelectIcoSize => (
        "Deja al menos un tamaño seleccionado.",
        "Deixa almenys una mida seleccionada.",
        "Leave at least one size selected."
    ),
    Eyedropper => ("CUENTAGOTAS", "COMPTAGOTES", "EYEDROPPER"),
    EyedropperUnavailable => (
        "El cuentagotas está disponible en la vista de imagen.",
        "El comptagotes està disponible a la vista d'imatge.",
        "The eyedropper is available in the image view."
    ),
    DisableEyedropper => ("Desactivar el cuentagotas", "Desactiva el comptagotes", "Disable eyedropper"),
    EnableEyedropper => (
        "Activar el cuentagotas · también puedes arrastrar hasta la imagen",
        "Activa el comptagotes · també pots arrossegar fins a la imatge",
        "Enable eyedropper · you can also drag onto the image"
    ),
    EyedropperIcoHint => (
        "Cambia a cualquier formato de imagen para muestrear.",
        "Canvia a qualsevol format d'imatge per mostrejar.",
        "Switch to any image format to sample."
    ),
    SampleCapturedChoose => (
        "Muestra capturada · elige otro píxel.",
        "Mostra capturada · tria un altre píxel.",
        "Sample captured · choose another pixel."
    ),
    SampleClickOrDrag => (
        "Haz clic o arrastra sobre la imagen.",
        "Fes clic o arrossega sobre la imatge.",
        "Click or drag over the image."
    ),
    SampleCaptured => ("Muestra capturada.", "Mostra capturada.", "Sample captured."),
    SampleFromHistory => ("Color del historial.", "Color de l'historial.", "Color from history."),
    EyedropperIdle => (
        "Pulsa el cuentagotas y elige un píxel.",
        "Prem el comptagotes i tria un píxel.",
        "Press the eyedropper and choose a pixel."
    ),
    History => ("HISTORIAL", "HISTORIAL", "HISTORY"),
    ClearHistory => ("Limpiar recientes", "Neteja els recents", "Clear recent"),
    HistoryFull => (
        "Historial lleno: libera un color para guardarlo.",
        "Historial ple: allibera un color per desar-lo.",
        "History full: unlock a color to save it."
    ),
    HistoryCleared => ("Historial limpiado.", "Historial netejat.", "History cleared."),
    ColorPinned => ("Color fijado.", "Color fixat.", "Color pinned."),
    ColorUnpinned => ("Color liberado.", "Color alliberat.", "Color unpinned."),
    ColorLockHover => ("clic derecho para fijar", "clic dret per fixar", "right-click to pin"),
    ColorUnlockHover => ("clic derecho para liberar", "clic dret per alliberar", "right-click to unpin"),
    ColorUseHover => ("clic para usar", "clic per usar", "click to use"),
    Copy => ("Copiar", "Copia", "Copy"),
    Copied => ("Copiado", "Copiat", "Copied"),
    Export => ("Exportar", "Exporta", "Export"),
    ExportHint => (
        "Se guarda en export/, junto a la app.  Ctrl+E",
        "Es desa a export/, al costat de l'app.  Ctrl+E",
        "Saved to export/, next to the app.  Ctrl+E"
    ),
    Calculating => ("Calculando el resultado…", "Calculant el resultat…", "Calculating result…"),
    Preview => ("Vista previa", "Vista prèvia", "Preview"),
    ResetZoom => ("Restablecer zoom", "Restableix el zoom", "Reset zoom"),
    DividerZoomHint => (
        "Arrastra el divisor · rueda para zoom",
        "Arrossega el divisor · roda per fer zoom",
        "Drag the divider · wheel to zoom"
    ),
    PanZoomHint => (
        "Rueda: zoom · clic derecho: mover",
        "Roda: zoom · clic dret: moure",
        "Wheel: zoom · right-click: pan"
    ),
    EyedropperActiveHint => (
        "Cuentagotas activo · haz clic o arrastra sobre la imagen",
        "Comptagotes actiu · fes clic o arrossega sobre la imatge",
        "Eyedropper active · click or drag over the image"
    ),
    IcoGeneratedView => ("Vista del icono generado", "Vista de la icona generada", "Generated icon view"),
    Processing => ("Procesando…", "Processant…", "Processing…"),
    PreparingResult => ("Preparando resultado…", "Preparant el resultat…", "Preparing result…"),
    GeneratingIcon => ("Generando icono…", "Generant la icona…", "Generating icon…"),
    Original => ("ORIGINAL", "ORIGINAL", "ORIGINAL"),
    Result => ("RESULTADO", "RESULTAT", "RESULT"),
    EqualWeight => ("= peso", "= pes", "= size"),
    SelectIcoSizeToast => (
        "Selecciona al menos un tamaño de icono.",
        "Selecciona almenys una mida d'icona.",
        "Select at least one icon size."
    ),
    WorkerInterrupted => (
        "El procesamiento en segundo plano se interrumpió.",
        "El processament en segon pla s'ha interromput.",
        "Background processing was interrupted."
    ),
    ClipboardOpenError => (
        "No se pudo abrir el portapapeles:",
        "No s'ha pogut obrir el porta-retalls:",
        "Could not open the clipboard:"
    ),
    ClipboardReadError => (
        "No se pudo leer la imagen del portapapeles:",
        "No s'ha pogut llegir la imatge del porta-retalls:",
        "Could not read the clipboard image:"
    ),
    ClipboardInvalid => (
        "La imagen del portapapeles no tiene un tamaño válido.",
        "La imatge del porta-retalls no té una mida vàlida.",
        "The clipboard image has an invalid size."
    ),
    ClipboardPasted => (
        "Imagen pegada desde el portapapeles.",
        "Imatge enganxada des del porta-retalls.",
        "Image pasted from the clipboard."
    ),
    ExportFolderError => (
        "No se pudo localizar la carpeta de la aplicación.",
        "No s'ha pogut localitzar la carpeta de l'aplicació.",
        "Could not locate the application folder."
    ),
    ExportCreateError => (
        "No se pudo crear export/:",
        "No s'ha pogut crear export/:",
        "Could not create export/:"
    ),
    ExportSaved => ("Imagen guardada en export/", "Imatge desada a export/", "Image saved to export/"),
    ExportWriteError => (
        "No se pudo guardar la imagen:",
        "No s'ha pogut desar la imatge:",
        "Could not save the image:"
    ),
    ImagesFilter => ("Imágenes", "Imatges", "Images"),
    FormatWebpHint => ("Equilibrio", "Equilibri", "Balanced"),
    FormatJpgHint => ("Compatible", "Compatible", "Compatible"),
    FormatPngHint => ("Sin pérdida", "Sense pèrdua", "Lossless"),
    FormatIcoHint => ("Windows", "Windows", "Windows"),
    FormatAvifHint => ("Moderno", "Modern", "Modern"),
    FormatJxlHint => ("Sin pérdida", "Sense pèrdua", "Lossless"),
    ErrReadFile => ("No se pudo leer el archivo:", "No s'ha pogut llegir el fitxer:", "Could not read the file:"),
    ErrTooLarge => ("La imagen supera el límite de 200 MB.", "La imatge supera el límit de 200 MB.", "The image exceeds the 200 MB limit."),
    ErrUnsupported => ("El archivo no parece una imagen compatible.", "El fitxer no sembla una imatge compatible.", "The file does not look like a supported image."),
    ErrDecode => ("No se pudo decodificar la imagen:", "No s'ha pogut descodificar la imatge:", "Could not decode the image:"),
    ErrConfig => ("No se pudo guardar la configuración:", "No s'ha pogut desar la configuració:", "Could not save the configuration:"),
    ErrSvgParse => ("No se pudo interpretar el SVG:", "No s'ha pogut interpretar l'SVG:", "Could not parse the SVG:"),
    ErrSvgCanvas => ("No se pudo crear el lienzo del SVG.", "No s'ha pogut crear el llenç de l'SVG.", "Could not create the SVG canvas."),
    ErrSvgInvalid => ("El SVG no produjo una imagen válida.", "L'SVG no ha produït una imatge vàlida.", "The SVG did not produce a valid image."),
    ErrInvalidDimensions => ("La imagen no tiene dimensiones válidas.", "La imatge no té dimensions vàlides.", "The image has invalid dimensions."),
    ErrDimensionsLimit => (
        "La imagen supera el límite de 40 MP o 16 384 px por lado.",
        "La imatge supera el límit de 40 MP o 16 384 px per costat.",
        "The image exceeds the 40 MP or 16,384 px per side limit."
    ),
    ErrProcessImage => ("No se pudo procesar la imagen:", "No s'ha pogut processar la imatge:", "Could not process the image:"),
    ErrEncodeJpg => ("No se pudo codificar el JPG:", "No s'ha pogut codificar el JPG:", "Could not encode the JPG:"),
    ErrEncodePng => ("No se pudo codificar el PNG:", "No s'ha pogut codificar el PNG:", "Could not encode the PNG:"),
    ErrEncodeAvif => ("No se pudo codificar el AVIF:", "No s'ha pogut codificar l'AVIF:", "Could not encode the AVIF:"),
    ErrEncodeJxl => ("No se pudo codificar el JPEG XL:", "No s'ha pogut codificar el JPEG XL:", "Could not encode the JPEG XL:"),
    ErrPreview => ("No se pudo decodificar la vista previa:", "No s'ha pogut descodificar la vista prèvia:", "Could not decode the preview:"),
    ErrPreviewAvif => ("No se pudo decodificar la vista previa AVIF:", "No s'ha pogut descodificar la vista prèvia AVIF:", "Could not decode the AVIF preview:"),
    ErrPreviewJxl => ("No se pudo decodificar la vista previa JPEG XL:", "No s'ha pogut descodificar la vista prèvia JPEG XL:", "Could not decode the JPEG XL preview:"),
    ErrAvifInvalid => ("El AVIF no produjo una imagen válida.", "L'AVIF no ha produït una imatge vàlida.", "The AVIF did not produce a valid image."),
    ErrPngStart => ("No se pudo iniciar el PNG:", "No s'ha pogut iniciar el PNG:", "Could not start the PNG:"),
}

pub fn pixel_coords(lang: Lang, x: u32, y: u32) -> String {
    match lang {
        Lang::Es => format!("píxel {x} · {y}"),
        Lang::Ca => format!("píxel {x} · {y}"),
        Lang::En => format!("pixel {x} · {y}"),
    }
}

pub fn copy_tooltip(lang: Lang, label: &str) -> String {
    match lang {
        Lang::Es => format!("Copiar {label}"),
        Lang::Ca => format!("Copia {label}"),
        Lang::En => format!("Copy {label}"),
    }
}

pub fn copied_message(lang: Lang, label: &str) -> String {
    match lang {
        Lang::Es => format!("{label} copiado."),
        Lang::Ca => format!("{label} copiat."),
        Lang::En => format!("{label} copied."),
    }
}

pub fn export_label(lang: Lang, extension: &str) -> String {
    format!("{} .{extension}", t(lang, K::Export))
}

pub fn saved_export(lang: Lang, name: &str) -> String {
    format!("{}{name}", t(lang, K::ExportSaved))
}

pub fn history_tooltip(lang: Lang, hex: &str, locked: bool) -> String {
    let action = if locked {
        t(lang, K::ColorUnlockHover)
    } else {
        t(lang, K::ColorLockHover)
    };
    format!("{hex}\n{} · {action}", t(lang, K::ColorUseHover))
}

pub fn ico_badge(lang: Lang, count: usize) -> String {
    let sizes = match lang {
        Lang::Es => "TAMAÑOS",
        Lang::Ca => "MIDES",
        Lang::En => "SIZES",
    };
    format!("ICO · {count} {sizes}")
}

pub fn clipboard_name(lang: Lang) -> &'static str {
    match lang {
        Lang::Es => "portapapeles.png",
        Lang::Ca => "porta-retalls.png",
        Lang::En => "clipboard.png",
    }
}

pub fn fallback_name(lang: Lang) -> &'static str {
    match lang {
        Lang::Es => "imagen",
        Lang::Ca => "imatge",
        Lang::En => "image",
    }
}

pub fn format_hint(lang: Lang, format: OutputFormat) -> &'static str {
    use OutputFormat as F;
    let key = match format {
        F::Webp => K::FormatWebpHint,
        F::Jpg => K::FormatJpgHint,
        F::Png => K::FormatPngHint,
        F::Ico => K::FormatIcoHint,
        F::Avif => K::FormatAvifHint,
        F::Jxl => K::FormatJxlHint,
    };
    t(lang, key)
}

pub fn localized_bytes(lang: Lang, bytes: u64) -> String {
    let value = crate::domain::format_bytes(bytes);
    if lang == Lang::En {
        value.replace(',', ".")
    } else {
        value
    }
}

pub fn present_error(lang: Lang, error: &AppError) -> String {
    match error {
        AppError::Load(error) => present_load_error(lang, error),
        AppError::Encode(error) => present_encode_error(lang, error),
        AppError::Export(error) => present_export_error(lang, error),
        AppError::Clipboard(error) => present_clipboard_error(lang, error),
        AppError::Config(error) => present_config_error(lang, error),
    }
}

fn present_load_error(lang: Lang, error: &LoadError) -> String {
    match error {
        LoadError::Read { source, .. } => detail(lang, K::ErrReadFile, source),
        LoadError::TooLarge { .. } => t(lang, K::ErrTooLarge).to_owned(),
        LoadError::Unsupported => t(lang, K::ErrUnsupported).to_owned(),
        LoadError::Decode { source } => detail(lang, K::ErrDecode, source),
        LoadError::SvgParse { source } => detail(lang, K::ErrSvgParse, source),
        LoadError::SvgCanvas => t(lang, K::ErrSvgCanvas).to_owned(),
        LoadError::SvgInvalid => t(lang, K::ErrSvgInvalid).to_owned(),
        LoadError::InvalidDimensions => t(lang, K::ErrInvalidDimensions).to_owned(),
        LoadError::DimensionsLimit => t(lang, K::ErrDimensionsLimit).to_owned(),
    }
}

fn present_encode_error(lang: Lang, error: &EncodeError) -> String {
    match error {
        EncodeError::InvalidDimensions => t(lang, K::ErrInvalidDimensions).to_owned(),
        EncodeError::Ico(error) => present_ico_error(lang, error),
        EncodeError::Jpg { source } => detail(lang, K::ErrEncodeJpg, source),
        EncodeError::PngStart { source } => detail(lang, K::ErrPngStart, source),
        EncodeError::Png { source } => detail(lang, K::ErrEncodePng, source),
        EncodeError::Avif { source } => detail(lang, K::ErrEncodeAvif, source),
        EncodeError::Jxl { source } => detail(lang, K::ErrEncodeJxl, source),
        EncodeError::Preview { format, source } => {
            let key = match format {
                OutputFormat::Avif => K::ErrPreviewAvif,
                OutputFormat::Jxl => K::ErrPreviewJxl,
                _ => K::ErrPreview,
            };
            detail(lang, key, source)
        }
        EncodeError::AvifPreviewInvalid => t(lang, K::ErrAvifInvalid).to_owned(),
    }
}

fn present_ico_error(lang: Lang, error: &IcoError) -> String {
    match error {
        IcoError::NoSizes => t(lang, K::SelectIcoSize).to_owned(),
        IcoError::Encode { size, source } => {
            detail(lang, K::ErrProcessImage, format!("{size} px: {source}"))
        }
        IcoError::Empty => t(lang, K::ErrProcessImage).to_owned(),
    }
}

fn present_export_error(lang: Lang, error: &ExportError) -> String {
    match error {
        ExportError::MissingExecutableDirectory => t(lang, K::ExportFolderError).to_owned(),
        ExportError::CreateDir { source, .. } => detail(lang, K::ExportCreateError, source),
        ExportError::Write { source, .. } => detail(lang, K::ExportWriteError, source),
    }
}

fn present_clipboard_error(lang: Lang, error: &ClipboardError) -> String {
    match error {
        ClipboardError::Open { source } => detail(lang, K::ClipboardOpenError, source),
        ClipboardError::Read { source } => detail(lang, K::ClipboardReadError, source),
        ClipboardError::InvalidImage => t(lang, K::ClipboardInvalid).to_owned(),
    }
}

fn present_config_error(lang: Lang, error: &ConfigError) -> String {
    match error {
        ConfigError::Read { source, .. } | ConfigError::Write { source, .. } => {
            detail(lang, K::ErrConfig, source)
        }
    }
}

fn detail(lang: Lang, key: K, detail: impl std::fmt::Display) -> String {
    format!("{} {detail}", t(lang, key))
}
