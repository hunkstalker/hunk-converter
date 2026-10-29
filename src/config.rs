use std::path::PathBuf;

use crate::i18n::Lang;

pub const HISTORY_LIMIT: usize = 21;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HistoryEntry {
    pub rgba: [u8; 4],
    pub locked: bool,
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub language: Lang,
    pub history: Vec<HistoryEntry>,
    pub fullscreen: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: Lang::detect(),
            history: Vec::new(),
            fullscreen: true,
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
            let mut settings = Self::read().unwrap_or_default();
            settings.language = language;
            return settings;
        }
        match Self::read() {
            Some(settings) => settings,
            None => {
                let settings = Self::default();
                settings.save();
                settings
            }
        }
    }

    fn read() -> Option<Self> {
        let text = std::fs::read_to_string(config_path()).ok()?;
        let mut settings = Self {
            language: Lang::detect(),
            history: Vec::new(),
            fullscreen: true,
        };
        let mut section = String::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                section = line[1..line.len() - 1].trim().to_ascii_lowercase();
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let key = key.trim().to_ascii_lowercase();
            let value = value.trim();
            if section == "general" && key == "language" {
                if let Some(language) = Lang::from_code(value) {
                    settings.language = language;
                }
            } else if section == "general" && key == "fullscreen" {
                settings.fullscreen =
                    !matches!(value.to_ascii_lowercase().as_str(), "0" | "false" | "no");
            } else if section == "history"
                && let Some(index) = key
                    .strip_prefix("color")
                    .and_then(|index| index.parse::<usize>().ok())
                && let Some(entry) = parse_history_entry(value)
                && settings.history.len() < HISTORY_LIMIT
                && !settings
                    .history
                    .iter()
                    .any(|existing| existing.rgba == entry.rgba)
            {
                let _ = index;
                settings.history.push(entry);
            }
        }
        Some(settings)
    }

    pub fn save(&self) {
        let mut text = String::from("[general]\n");
        text.push_str(&format!(
            "language={}\nfullscreen={}\n\n[history]\n",
            self.language.code(),
            u8::from(self.fullscreen)
        ));
        text.push_str(&format!("count={}\n", self.history.len()));
        for (index, entry) in self.history.iter().take(HISTORY_LIMIT).enumerate() {
            text.push_str(&format!(
                "color{}={}|{}\n",
                index + 1,
                hex_rgba(entry.rgba),
                u8::from(entry.locked)
            ));
        }
        if let Err(error) = std::fs::write(config_path(), text) {
            eprintln!("[config] no se pudo guardar config.ini: {error}");
        }
    }
}

fn parse_history_entry(value: &str) -> Option<HistoryEntry> {
    let (color, locked) = value.split_once('|').unwrap_or((value, "0"));
    let rgba = parse_hex_rgba(color)?;
    Some(HistoryEntry {
        rgba,
        locked: locked.trim() == "1",
    })
}

pub fn parse_hex_rgba(value: &str) -> Option<[u8; 4]> {
    let value = value.trim().trim_start_matches('#');
    if !value.chars().all(|character| character.is_ascii_hexdigit()) {
        return None;
    }
    match value.len() {
        6 => Some([
            u8::from_str_radix(&value[0..2], 16).ok()?,
            u8::from_str_radix(&value[2..4], 16).ok()?,
            u8::from_str_radix(&value[4..6], 16).ok()?,
            255,
        ]),
        8 => Some([
            u8::from_str_radix(&value[0..2], 16).ok()?,
            u8::from_str_radix(&value[2..4], 16).ok()?,
            u8::from_str_radix(&value[4..6], 16).ok()?,
            u8::from_str_radix(&value[6..8], 16).ok()?,
        ]),
        _ => None,
    }
}

pub fn hex_rgba(rgba: [u8; 4]) -> String {
    format!(
        "#{:02X}{:02X}{:02X}{:02X}",
        rgba[0], rgba[1], rgba[2], rgba[3]
    )
}

pub fn remember_color(history: &mut Vec<HistoryEntry>, rgba: [u8; 4]) -> bool {
    if let Some(index) = history.iter().position(|entry| entry.rgba == rgba) {
        let entry = history.remove(index);
        history.insert(0, entry);
        return true;
    }
    if history.len() >= HISTORY_LIMIT {
        let Some(index) = history.iter().rposition(|entry| !entry.locked) else {
            return false;
        };
        history.remove(index);
    }
    history.insert(
        0,
        HistoryEntry {
            rgba,
            locked: false,
        },
    );
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(value: u8, locked: bool) -> HistoryEntry {
        HistoryEntry {
            rgba: [value, 0, 0, 255],
            locked,
        }
    }

    #[test]
    fn remembers_unique_colors_at_the_front() {
        let mut history = vec![entry(1, false), entry(2, false)];
        assert!(remember_color(&mut history, [3, 0, 0, 255]));
        assert_eq!(history[0].rgba[0], 3);
        assert!(remember_color(&mut history, [2, 0, 0, 255]));
        assert_eq!(history[0].rgba[0], 2);
        assert_eq!(history.len(), 3);
    }

    #[test]
    fn replaces_the_oldest_unlocked_color() {
        let mut history = (0..HISTORY_LIMIT)
            .map(|index| entry(index as u8, false))
            .collect::<Vec<_>>();
        history[HISTORY_LIMIT - 1].locked = true;
        assert!(remember_color(&mut history, [99, 0, 0, 255]));
        assert_eq!(history[0].rgba[0], 99);
        assert_eq!(history.len(), HISTORY_LIMIT);
        assert_eq!(
            history[HISTORY_LIMIT - 1].rgba[0],
            (HISTORY_LIMIT - 1) as u8
        );
        assert!(history[HISTORY_LIMIT - 1].locked);
    }

    #[test]
    fn rejects_new_colors_when_everything_is_locked() {
        let mut history = (0..HISTORY_LIMIT)
            .map(|index| entry(index as u8, true))
            .collect::<Vec<_>>();
        assert!(!remember_color(&mut history, [99, 0, 0, 255]));
        assert_eq!(history.len(), HISTORY_LIMIT);
    }
}
