use std::path::Path;

use crate::domain::color::parse_hex_rgba;
use crate::domain::history::{HISTORY_LIMIT, HistoryEntry};
use crate::i18n::Lang;

use super::{ConfigLoad, ConfigWarning, Settings};

pub(super) fn read(path: &Path) -> Result<ConfigLoad, crate::domain::ConfigError> {
    let text =
        std::fs::read_to_string(path).map_err(|source| crate::domain::ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;
    Ok(parse(&text))
}

pub(super) fn parse(text: &str) -> ConfigLoad {
    let mut settings = Settings {
        language: Lang::detect(),
        history: Vec::new(),
        maximized: true,
    };
    let mut warnings = Vec::new();
    let mut section = String::new();
    let mut maximized_seen = false;

    for (index, raw_line) in text.lines().enumerate() {
        let line_number = index + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_ascii_lowercase();
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            warnings.push(ConfigWarning::new(line_number, "línea sin '='; se ignora"));
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();

        if section == "general" && key == "language" {
            match Lang::from_code(value) {
                Some(language) => settings.language = language,
                None => warnings.push(ConfigWarning::new(
                    line_number,
                    format!("idioma desconocido: {value}"),
                )),
            }
        } else if section == "general" && key == "maximized" {
            settings.maximized = parse_bool(value);
            maximized_seen = true;
        } else if section == "general" && key == "fullscreen" && !maximized_seen {
            settings.maximized = parse_bool(value);
        } else if section == "history" && key.starts_with("color") {
            let Some(index) = key
                .strip_prefix("color")
                .and_then(|index| index.parse::<usize>().ok())
            else {
                warnings.push(ConfigWarning::new(
                    line_number,
                    format!("clave de color no válida: {key}"),
                ));
                continue;
            };
            let Some(entry) = parse_history_entry(value) else {
                warnings.push(ConfigWarning::new(
                    line_number,
                    format!("color no válido: {value}"),
                ));
                continue;
            };
            if index == 0 {
                warnings.push(ConfigWarning::new(
                    line_number,
                    "los colores empiezan en color1",
                ));
                continue;
            }
            if settings.history.len() >= HISTORY_LIMIT {
                warnings.push(ConfigWarning::new(
                    line_number,
                    "hay más colores que el límite del historial",
                ));
                continue;
            }
            if settings
                .history
                .iter()
                .any(|existing| existing.rgba == entry.rgba)
            {
                warnings.push(ConfigWarning::new(
                    line_number,
                    "color duplicado; se ignora",
                ));
                continue;
            }
            settings.history.push(entry);
        }
    }

    ConfigLoad { settings, warnings }
}

pub(super) fn serialize(settings: &Settings) -> String {
    let mut text = String::from("[general]\n");
    text.push_str(&format!(
        "language={}\nmaximized={}\n\n[history]\ncount={}\n",
        settings.language.code(),
        u8::from(settings.maximized),
        settings.history.len()
    ));
    for (index, entry) in settings.history.iter().take(HISTORY_LIMIT).enumerate() {
        text.push_str(&format!(
            "color{}={}|{}\n",
            index + 1,
            crate::domain::hex_rgba(entry.rgba),
            u8::from(entry.locked)
        ));
    }
    text
}

fn parse_bool(value: &str) -> bool {
    !matches!(value.to_ascii_lowercase().as_str(), "0" | "false" | "no")
}

fn parse_history_entry(value: &str) -> Option<HistoryEntry> {
    let (color, locked) = value.split_once('|').unwrap_or((value, "0"));
    let rgba = parse_hex_rgba(color)?;
    Some(HistoryEntry::new(rgba, locked.trim() == "1"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locked_colors_survive_a_round_trip() {
        let text = "[general]\nlanguage=es\nmaximized=0\n\n[history]\ncount=2\ncolor1=#7C7E78FF|1\ncolor2=#B38E75FF|0\n";
        let loaded = parse(text);
        assert_eq!(loaded.settings.language, Lang::Es);
        assert!(!loaded.settings.maximized);
        assert_eq!(loaded.settings.history.len(), 2);
        assert!(loaded.settings.history[0].locked);
        assert!(!loaded.settings.history[1].locked);

        let round_trip = parse(&serialize(&loaded.settings));
        assert_eq!(round_trip.settings.history, loaded.settings.history);
    }

    #[test]
    fn old_fullscreen_key_is_still_accepted() {
        let loaded = parse("[general]\nfullscreen=0\n");
        assert!(!loaded.settings.maximized);

        let loaded = parse("[general]\nfullscreen=1\nmaximized=0\n");
        assert!(!loaded.settings.maximized);
    }

    #[test]
    fn malformed_history_entries_are_reported_not_silenced() {
        let loaded = parse("[history]\ncolor1=no-es-un-color|1\ncolor2=#112233FF|1\n");
        assert_eq!(loaded.settings.history.len(), 1);
        assert!(loaded.settings.history[0].locked);
        assert_eq!(loaded.warnings.len(), 1);
    }
}
