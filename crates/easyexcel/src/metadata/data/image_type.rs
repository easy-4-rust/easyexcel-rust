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
