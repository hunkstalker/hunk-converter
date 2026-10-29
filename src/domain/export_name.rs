pub fn safe_stem(value: &str) -> String {
    let mut clean = value
        .replace(&['<', '>', ':', '"', '/', '\\', '|', '?', '*'][..], "-")
        .trim()
        .trim_end_matches(&['.', ' '][..])
        .to_owned();
    if clean.len() > 120 {
        clean.truncate(120);
        clean = clean.trim_end().to_owned();
    }
    let reserved = [
        "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
        "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
    ];
    if clean.is_empty() || reserved.contains(&clean.to_lowercase().as_str()) {
        "imagen".to_owned()
    } else {
        clean
    }
}

pub fn export_file_name(stem: &str, extension: &str, suffix: Option<u32>) -> String {
    match suffix {
        Some(suffix) => format!("{stem}-{suffix}.{extension}"),
        None => format!("{stem}.{extension}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_reserved_and_invalid_names() {
        assert_eq!(safe_stem("foto:final"), "foto-final");
        assert_eq!(safe_stem("CON"), "imagen");
        assert_eq!(safe_stem("   "), "imagen");
        assert_eq!(safe_stem("foto."), "foto");
    }

    #[test]
    fn numbers_duplicate_exports_without_overwriting() {
        assert_eq!(export_file_name("foto", "webp", None), "foto.webp");
        assert_eq!(export_file_name("foto", "webp", Some(2)), "foto-2.webp");
    }
}
