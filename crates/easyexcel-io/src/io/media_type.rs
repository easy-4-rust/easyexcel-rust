//! 二进制媒体文件头识别。

/// 对应 Java：无直接对应对象；Rust 架构扩展。 规范化常见图片类型扩展名。
#[must_use]
pub fn normalize_image_type(image_type: &str) -> String {
    match image_type.to_ascii_lowercase().as_str() {
        "jpeg" | "jpg" => "jpg".to_owned(),
        "png" => "png".to_owned(),
        "gif" => "gif".to_owned(),
        "bmp" => "bmp".to_owned(),
        other => other.to_owned(),
    }
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 根据文件头识别常见图片类型。
#[must_use]
pub fn detect_image_type(header: &[u8]) -> Option<&'static str> {
    if header.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("jpg")
    } else if header.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        Some("png")
    } else if header.starts_with(b"GIF") {
        Some("gif")
    } else if header.starts_with(b"BM") {
        Some("bmp")
    } else {
        None
    }
}
