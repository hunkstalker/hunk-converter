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

pub fn app_root() -> Result<PathBuf, std::io::Error> {
    std::env::current_exe().map(|path| portable_root_from(&path))
}

pub fn config_path() -> PathBuf {
    app_root()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("config.ini")
}

fn portable_root_from(executable: &Path) -> PathBuf {
    let Some(macos) = executable.parent() else {
        return PathBuf::from(".");
    };
    let Some(contents) = macos.parent() else {
        return macos.to_path_buf();
    };
    let Some(app) = contents.parent() else {
        return macos.to_path_buf();
    };
    let is_macos_bundle = macos.file_name().is_some_and(|name| name == "MacOS")
        && contents.file_name().is_some_and(|name| name == "Contents")
        && app.extension().is_some_and(|extension| extension == "app");
    if is_macos_bundle {
        app.parent().unwrap_or(app).to_path_buf()
    } else {
        macos.to_path_buf()
    }
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

#[cfg(test)]
mod path_tests {
    use super::*;

    #[test]
    fn normal_binary_uses_its_parent_directory() {
        let executable = Path::new("/opt/hunk-converter/hunk-converter");
        assert_eq!(
            portable_root_from(executable),
            PathBuf::from("/opt/hunk-converter")
        );
    }

    #[test]
    fn macos_bundle_uses_the_folder_that_contains_the_app() {
        let executable = Path::new(
            "/Users/ana/Downloads/hunk-converter-1.0.0/HUNK Converter.app/Contents/MacOS/hunk-converter",
        );
        assert_eq!(
            portable_root_from(executable),
            PathBuf::from("/Users/ana/Downloads/hunk-converter-1.0.0")
        );
    }
}
