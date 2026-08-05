//! Java `FileTypeUtils` 兼容入口。

/// 返回规范化图片扩展名。
#[must_use]
pub fn get_image_type_format(image_type: &str) -> String {
    easyexcel_io::io::media_type::normalize_image_type(image_type)
}

/// 根据文件头识别图片类型。
#[must_use]
pub fn get_image_type(image_header: &[u8]) -> Option<&'static str> {
    easyexcel_io::io::media_type::detect_image_type(image_header)
}
