#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputFormat {
    Webp,
    Jpg,
    Png,
    Ico,
    Avif,
    Jxl,
}

impl OutputFormat {
    pub const ALL: [Self; 6] = [
        Self::Webp,
        Self::Jpg,
        Self::Png,
        Self::Avif,
        Self::Jxl,
        Self::Ico,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Webp => "WebP",
            Self::Jpg => "JPG",
            Self::Png => "PNG",
            Self::Ico => "ICO",
            Self::Avif => "AVIF",
            Self::Jxl => "JPEG XL",
        }
    }

    pub fn badge(self) -> &'static str {
        match self {
            Self::Webp => "WEBP",
            Self::Jpg => "JPG",
            Self::Png => "PNG",
            Self::Ico => "ICO",
            Self::Avif => "AVIF",
            Self::Jxl => "JXL",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::Webp => "webp",
            Self::Jpg => "jpg",
            Self::Png => "png",
            Self::Ico => "ico",
            Self::Avif => "avif",
            Self::Jxl => "jxl",
        }
    }

    pub fn is_lossy(self) -> bool {
        matches!(self, Self::Webp | Self::Jpg | Self::Avif)
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / KB).replace('.', ",")
    } else {
        format!("{:.2} MB", bytes as f64 / MB).replace('.', ",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_expose_a_single_policy_source() {
        assert_eq!(OutputFormat::Webp.extension(), "webp");
        assert!(OutputFormat::Webp.is_lossy());
        assert!(!OutputFormat::Png.is_lossy());
        assert!(!OutputFormat::Jxl.is_lossy());
        assert_eq!(OutputFormat::Jxl.badge(), "JXL");
    }

    #[test]
    fn formats_human_readable_sizes() {
        assert_eq!(format_bytes(900), "900 B");
        assert_eq!(format_bytes(1536), "1,5 KB");
        assert_eq!(format_bytes(2 * 1024 * 1024), "2,00 MB");
    }
}
