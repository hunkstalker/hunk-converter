use std::path::{Path, PathBuf};

use crate::domain::export_name::{export_file_name, safe_stem};
use crate::domain::{AppError, ExportError};
use crate::i18n;

use super::HunkApp;

impl HunkApp {
    pub(crate) fn export(&mut self) {
        if !self.can_export() {
            return;
        }
        let Some(result) = self.result.as_ref() else {
            return;
        };
        let Some(source) = self.source.as_ref() else {
            return;
        };

        let directory = match export_directory() {
            Ok(directory) => directory,
            Err(error) => {
                self.toast_error(&AppError::Export(error), 4);
                return;
            }
        };
        if let Err(source) = std::fs::create_dir_all(&directory) {
            self.toast_error(
                &AppError::Export(ExportError::CreateDir {
                    path: directory,
                    source,
                }),
                4,
            );
            return;
        }

        let stem = safe_stem(&source.stem);
        let extension = self.format.extension();
        let path = unique_path(&directory, &stem, extension);
        match std::fs::write(&path, &result.bytes) {
            Ok(()) => {
                let file_name = path
                    .file_name()
                    .map(|value| value.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path.display().to_string());
                self.toast(i18n::saved_export(self.language, &file_name), false, 4);
            }
            Err(source) => {
                self.toast_error(&AppError::Export(ExportError::Write { path, source }), 4);
            }
        }
    }
}

fn export_directory() -> Result<PathBuf, ExportError> {
    let root = crate::settings::app_root().map_err(|_| ExportError::MissingExecutableDirectory)?;
    Ok(root.join("export"))
}

fn unique_path(directory: &Path, stem: &str, extension: &str) -> PathBuf {
    let mut suffix = None;
    loop {
        let candidate = directory.join(export_file_name(stem, extension, suffix));
        if !candidate.exists() {
            return candidate;
        }
        suffix = Some(suffix.map_or(2, |value| value + 1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_paths_do_not_overwrite() {
        let directory =
            std::env::temp_dir().join(format!("hunk-converter-export-test-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let first = unique_path(&directory, "foto", "webp");
        std::fs::write(&first, b"x").unwrap();
        let second = unique_path(&directory, "foto", "webp");
        assert_eq!(second.file_name().unwrap(), "foto-2.webp");
        let _ = std::fs::remove_dir_all(&directory);
    }
}
