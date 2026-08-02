//! Mirrors Java com.alibaba.excel.util.FileTypeUtils.

#![allow(dead_code)]

/// Mirrors `com.alibaba.excel.util.FileTypeUtils#getImageTypeFormat`.
///
/// Returns the canonical lowercase file extension (without leading dot)
/// for a given image type name. Java normalises to `jpg`, `png`, `gif`,
/// `bmp`; an unknown name is returned unchanged.
#[must_use]
pub fn get_image_type_format(image_type: &str) -> String {
    let lower = image_type.to_ascii_lowercase();
    match lower.as_str() {
        "jpeg" | "jpg" => "jpg".to_owned(),
        "png" => "png".to_owned(),
        "gif" => "gif".to_owned(),
        "bmp" => "bmp".to_owned(),
        other => other.to_owned(),
    }
}

/// Mirrors `com.alibaba.excel.util.FileTypeUtils#getImageType`.
///
/// Sniffs the image type from the magic bytes of a file header.
#[must_use]
pub fn get_image_type(image_header: &[u8]) -> Option<&'static str> {
    if image_header.len() >= 3 && image_header[0..3] == [0xFF, 0xD8, 0xFF] {
        return Some("jpg");
    }
    if image_header.len() >= 4 && image_header[0..4] == [0x89, 0x50, 0x4E, 0x47] {
        return Some("png");
    }
    if image_header.len() >= 3 && image_header[0..3] == [0x47, 0x49, 0x46] {
        return Some("gif");
    }
    if image_header.len() >= 2 && image_header[0..2] == [0x42, 0x4D] {
        return Some("bmp");
    }
    None
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn get_image_type_format_normalizes_names() {
        // 对应 Java：FileTypeUtils.getImageTypeFormat 规范化扩展名
        assert_eq!(get_image_type_format("jpeg"), "jpg");
        assert_eq!(get_image_type_format("JPG"), "jpg");
        assert_eq!(get_image_type_format("JPEG"), "jpg");
        assert_eq!(get_image_type_format("png"), "png");
        assert_eq!(get_image_type_format("PNG"), "png");
        assert_eq!(get_image_type_format("gif"), "gif");
        assert_eq!(get_image_type_format("bmp"), "bmp");
        // 未知类型原样返回（小写化）
        assert_eq!(get_image_type_format("webp"), "webp");
        assert_eq!(get_image_type_format(""), "");
    }

    #[test]
    fn get_image_type_sniffs_magic_bytes() {
        // 对应 Java：FileTypeUtils.getImageType 按魔数识别图片类型
        assert_eq!(get_image_type(&[0xFF, 0xD8, 0xFF, 0xE0]), Some("jpg"));
        assert_eq!(get_image_type(&[0x89, 0x50, 0x4E, 0x47]), Some("png"));
        assert_eq!(get_image_type(&[0x47, 0x49, 0x46, 0x38]), Some("gif"));
        assert_eq!(get_image_type(&[0x42, 0x4D, 0x00, 0x00]), Some("bmp"));
        // 过短或不匹配的头部返回 None
        assert_eq!(get_image_type(&[0xFF, 0xD8]), None);
        assert_eq!(get_image_type(&[0x89, 0x50]), None);
        assert_eq!(get_image_type(&[]), None);
        assert_eq!(get_image_type(&[0x00, 0x01, 0x02, 0x03]), None);
    }
}
