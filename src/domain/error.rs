use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use super::format::OutputFormat;

pub type BoxError = Box<dyn Error + Send + Sync>;

#[derive(Debug)]
pub enum LoadError {
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    TooLarge {
        bytes: u64,
        limit: u64,
    },
    Unsupported,
    Decode {
        source: BoxError,
    },
    SvgParse {
        source: BoxError,
    },
    SvgCanvas,
    SvgInvalid,
    InvalidDimensions,
    DimensionsLimit,
}

impl fmt::Display for LoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => {
                write!(formatter, "no se pudo leer {}: {source}", path.display())
            }
            Self::TooLarge { bytes, limit } => {
                write!(
                    formatter,
                    "la imagen pesa {bytes} B y supera el límite de {limit} B"
                )
            }
            Self::Unsupported => formatter.write_str("formato de imagen no compatible"),
            Self::Decode { source } => write!(formatter, "no se pudo decodificar: {source}"),
            Self::SvgParse { source } => {
                write!(formatter, "no se pudo interpretar el SVG: {source}")
            }
            Self::SvgCanvas => formatter.write_str("no se pudo crear el lienzo del SVG"),
            Self::SvgInvalid => formatter.write_str("el SVG no produjo una imagen válida"),
            Self::InvalidDimensions => formatter.write_str("dimensiones no válidas"),
            Self::DimensionsLimit => {
                formatter.write_str("la imagen supera el límite de 40 MP o 16 384 px por lado")
            }
        }
    }
}

impl Error for LoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read { source, .. } => Some(source),
            Self::Decode { source } | Self::SvgParse { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum IcoError {
    NoSizes,
    Encode { size: u32, source: BoxError },
    Empty,
}

impl fmt::Display for IcoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSizes => formatter.write_str("selecciona al menos un tamaño de icono"),
            Self::Encode { size, source } => {
                write!(
                    formatter,
                    "no se pudo codificar el icono de {size} px: {source}"
                )
            }
            Self::Empty => formatter.write_str("no se pudo generar ningún icono"),
        }
    }
}

impl Error for IcoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Encode { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum EncodeError {
    InvalidDimensions,
    Ico(IcoError),
    Jpg {
        source: BoxError,
    },
    PngStart {
        source: BoxError,
    },
    Png {
        source: BoxError,
    },
    Avif {
        source: BoxError,
    },
    Jxl {
        source: BoxError,
    },
    Preview {
        format: OutputFormat,
        source: BoxError,
    },
    AvifPreviewInvalid,
}

impl fmt::Display for EncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions => {
                formatter.write_str("la imagen no tiene dimensiones válidas")
            }
            Self::Ico(error) => write!(formatter, "{error}"),
            Self::Jpg { source } => write!(formatter, "no se pudo codificar el JPG: {source}"),
            Self::PngStart { source } => write!(formatter, "no se pudo iniciar el PNG: {source}"),
            Self::Png { source } => write!(formatter, "no se pudo codificar el PNG: {source}"),
            Self::Avif { source } => write!(formatter, "no se pudo codificar el AVIF: {source}"),
            Self::Jxl { source } => write!(formatter, "no se pudo codificar el JPEG XL: {source}"),
            Self::Preview { format, source } => {
                write!(
                    formatter,
                    "no se pudo decodificar la vista previa {format:?}: {source}"
                )
            }
            Self::AvifPreviewInvalid => formatter.write_str("el AVIF no produjo una imagen válida"),
        }
    }
}

impl Error for EncodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Ico(error) => Some(error),
            Self::Jpg { source }
            | Self::PngStart { source }
            | Self::Png { source }
            | Self::Avif { source }
            | Self::Jxl { source }
            | Self::Preview { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl From<IcoError> for EncodeError {
    fn from(error: IcoError) -> Self {
        Self::Ico(error)
    }
}

#[derive(Debug)]
pub enum ExportError {
    MissingExecutableDirectory,
    CreateDir {
        path: PathBuf,
        source: std::io::Error,
    },
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl fmt::Display for ExportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingExecutableDirectory => {
                formatter.write_str("no se pudo localizar la carpeta del ejecutable")
            }
            Self::CreateDir { path, source } => {
                write!(formatter, "no se pudo crear {}: {source}", path.display())
            }
            Self::Write { path, source } => {
                write!(
                    formatter,
                    "no se pudo escribir {}: {source}",
                    path.display()
                )
            }
        }
    }
}

impl Error for ExportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CreateDir { source, .. } | Self::Write { source, .. } => Some(source),
            Self::MissingExecutableDirectory => None,
        }
    }
}

#[derive(Debug)]
pub enum ClipboardError {
    Open { source: BoxError },
    Read { source: BoxError },
    InvalidImage,
}

impl fmt::Display for ClipboardError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Open { source } => {
                write!(formatter, "no se pudo abrir el portapapeles: {source}")
            }
            Self::Read { source } => write!(formatter, "no se pudo leer el portapapeles: {source}"),
            Self::InvalidImage => formatter.write_str("la imagen del portapapeles no es válida"),
        }
    }
}

impl Error for ClipboardError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Open { source } | Self::Read { source } => Some(source.as_ref()),
            Self::InvalidImage => None,
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => {
                write!(formatter, "no se pudo leer {}: {source}", path.display())
            }
            Self::Write { path, source } => {
                write!(
                    formatter,
                    "no se pudo escribir {}: {source}",
                    path.display()
                )
            }
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read { source, .. } | Self::Write { source, .. } => Some(source),
        }
    }
}

#[derive(Debug)]
pub enum AppError {
    Load(LoadError),
    Encode(EncodeError),
    Export(ExportError),
    Clipboard(ClipboardError),
    Config(ConfigError),
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load(error) => write!(formatter, "{error}"),
            Self::Encode(error) => write!(formatter, "{error}"),
            Self::Export(error) => write!(formatter, "{error}"),
            Self::Clipboard(error) => write!(formatter, "{error}"),
            Self::Config(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Load(error) => Some(error),
            Self::Encode(error) => Some(error),
            Self::Export(error) => Some(error),
            Self::Clipboard(error) => Some(error),
            Self::Config(error) => Some(error),
        }
    }
}

impl From<LoadError> for AppError {
    fn from(error: LoadError) -> Self {
        Self::Load(error)
    }
}

impl From<EncodeError> for AppError {
    fn from(error: EncodeError) -> Self {
        Self::Encode(error)
    }
}

impl From<ExportError> for AppError {
    fn from(error: ExportError) -> Self {
        Self::Export(error)
    }
}

impl From<ClipboardError> for AppError {
    fn from(error: ClipboardError) -> Self {
        Self::Clipboard(error)
    }
}

impl From<ConfigError> for AppError {
    fn from(error: ConfigError) -> Self {
        Self::Config(error)
    }
}
