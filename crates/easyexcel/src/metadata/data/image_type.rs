//! 对应 Java：`com.alibaba.excel.metadata.data.ImageData.ImageType`.

/// Java `ImageData.ImageType` equivalent metadata.
///
/// Java retains the POI numeric codes (2..=7). Rust drops them and maps to
/// `rust_xlsxwriter::Image` automatically; the enum is preserved for API
/// completeness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
/// 对应 Java：com.alibaba.excel.metadata.data.ImageData.ImageType。
pub enum ImageType {
    /// Extended Windows metafile.
    Emf,
    /// Windows metafile.
    Wmf,
    /// Macintosh PICT.
    Pict,
    /// JPEG.
    Jpeg,
    /// PNG.
    Png,
    /// Device-independent bitmap.
    Dib,
}

impl ImageType {
    /// Java `values()` 的声明顺序。
    pub const ALL: [Self; 6] = [
        Self::Emf,
        Self::Wmf,
        Self::Pict,
        Self::Jpeg,
        Self::Png,
        Self::Dib,
    ];
    /// Java 枚举常量名。
    #[must_use]
    pub const fn java_name(self) -> &'static str {
        match self {
            Self::Emf => "PICTURE_TYPE_EMF",
            Self::Wmf => "PICTURE_TYPE_WMF",
            Self::Pict => "PICTURE_TYPE_PICT",
            Self::Jpeg => "PICTURE_TYPE_JPEG",
            Self::Png => "PICTURE_TYPE_PNG",
            Self::Dib => "PICTURE_TYPE_DIB",
        }
    }
    /// Java `getValue()` 使用的 POI 图片类型编号。
    #[must_use]
    pub const fn get_value(self) -> i32 {
        match self {
            Self::Emf => 2,
            Self::Wmf => 3,
            Self::Pict => 4,
            Self::Jpeg => 5,
            Self::Png => 6,
            Self::Dib => 7,
        }
    }
}

impl std::str::FromStr for ImageType {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|item| item.java_name() == value)
            .ok_or_else(|| format!("unknown ImageData.ImageType value: {value}"))
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn java_names_match_java_constants() {
        assert_eq!(ImageType::Emf.java_name(), "PICTURE_TYPE_EMF");
        assert_eq!(ImageType::Wmf.java_name(), "PICTURE_TYPE_WMF");
        assert_eq!(ImageType::Pict.java_name(), "PICTURE_TYPE_PICT");
        assert_eq!(ImageType::Jpeg.java_name(), "PICTURE_TYPE_JPEG");
        assert_eq!(ImageType::Png.java_name(), "PICTURE_TYPE_PNG");
        assert_eq!(ImageType::Dib.java_name(), "PICTURE_TYPE_DIB");
    }

    #[test]
    fn get_value_matches_poi_codes() {
        assert_eq!(ImageType::Emf.get_value(), 2);
        assert_eq!(ImageType::Wmf.get_value(), 3);
        assert_eq!(ImageType::Pict.get_value(), 4);
        assert_eq!(ImageType::Jpeg.get_value(), 5);
        assert_eq!(ImageType::Png.get_value(), 6);
        assert_eq!(ImageType::Dib.get_value(), 7);
    }

    #[test]
    fn all_contains_six_variants() {
        assert_eq!(ImageType::ALL.len(), 6);
    }

    #[test]
    fn from_str_parses_all_variants() {
        for img in ImageType::ALL {
            let parsed: ImageType = img.java_name().parse().unwrap();
            assert_eq!(parsed, img);
        }
    }

    #[test]
    fn from_str_rejects_unknown() {
        let result: Result<ImageType, _> = "UNKNOWN".parse();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown"));
    }

    #[test]
    fn serde_roundtrip() {
        let img = ImageType::Png;
        let json = serde_json::to_string(&img).unwrap();
        let parsed: ImageType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ImageType::Png);
    }
}
