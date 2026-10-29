use std::path::{Path, PathBuf};

use crate::domain::ConfigError;
use crate::domain::history::HistoryEntry;
use crate::i18n::Lang;

mod ini;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub language: Lang,
    pub history: Vec<HistoryEntry>,
    pub maximized: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: Lang::detect(),
            history: Vec::new(),
            maximized: true,
        }
    }
}

#[derive(Debug)]
pub struct ConfigLoad {
    pub settings: Settings,
    pub warnings: Vec<ConfigWarning>,
}

#[derive(Debug)]
pub struct ConfigWarning {
    pub line: usize,
    pub message: String,
}

impl ConfigWarning {
    pub fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

pub fn config_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join("config.ini")))
        .unwrap_or_else(|| PathBuf::from("config.ini"))
}

impl Settings {
    pub fn load() -> Self {
        if let Ok(code) = std::env::var("HUNK_LANG")
            && let Some(language) = Lang::from_code(&code)
        {
            let mut settings = match ini::read(&config_path()) {
                Ok(outcome) => {
                    log_warnings(&outcome.warnings);
                    outcome.settings
                }
                Err(ConfigError::Read { source, .. })
                    if source.kind() == std::io::ErrorKind::NotFound =>
                {
                    Self::default()
                }
                Err(error) => {
                    log_error(&error);
                    Self::default()
                }
            };
            settings.language = language;
            return settings;
        }

        match ini::read(&config_path()) {
            Ok(outcome) => {
                log_warnings(&outcome.warnings);
                outcome.settings
            }
            Err(ConfigError::Read { source, .. })
                if source.kind() == std::io::ErrorKind::NotFound =>
            {
                let settings = Self::default();
                if let Err(error) = settings.save() {
                    log_error(&error);
                }
                settings
            }
            Err(error) => {
                log_error(&error);
                Self::default()
            }
        }
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let path = config_path();
        std::fs::write(&path, ini::serialize(self))
            .map_err(|source| ConfigError::Write { path, source })
    }
}

pub fn read_from(path: &Path) -> Result<ConfigLoad, ConfigError> {
    ini::read(path)
}

fn log_warnings(warnings: &[ConfigWarning]) {
    for warning in warnings {
        eprintln!(
            "[config] aviso en la línea {}: {}",
            warning.line, warning.message
        );
    }
}

fn log_error(error: &ConfigError) {
    eprintln!("[config] {error}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip_keeps_locked_history() {
        let settings = Settings {
            language: Lang::Es,
            history: vec![
                HistoryEntry::new([124, 126, 120, 255], true),
                HistoryEntry::new([179, 142, 117, 255], false),
            ],
            maximized: false,
        };
        let outcome = ini::parse(&ini::serialize(&settings));
        assert_eq!(outcome.settings, settings);
        assert!(outcome.warnings.is_empty());
    }
}
