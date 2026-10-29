pub mod color;
pub mod error;
pub mod export_name;
pub mod format;
pub mod history;

pub use color::{color_codes, flatten_rgba, hex_rgba, parse_hex_rgba};
pub use error::{
    AppError, ClipboardError, ConfigError, EncodeError, ExportError, IcoError, LoadError,
};
pub use format::{OutputFormat, format_bytes};
pub use history::{HISTORY_LIMIT, HistoryEntry, clear_unlocked, remember_color, toggle_lock};
