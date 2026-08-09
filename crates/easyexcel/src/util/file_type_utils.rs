//! Java `FileTypeUtils` 兼容入口。

use std::sync::RwLock;

use crate::ImageType;

/// Java 可变 `defaultImageType` 字段的线程安全 Rust 载体。
///
/// 对应 Java：`FileTypeUtils.defaultImageType`。
pub static DEFAULT_IMAGE_TYPE: RwLock<ImageType> = RwLock::new(ImageType::Png);

/// 返回图片的 POI 数字类型；未知格式使用当前默认类型。
///
/// 对应 Java：`FileTypeUtils#getImageTypeFormat(byte[])`。
#[must_use]
pub fn get_image_type_format(image: &[u8]) -> i32 {
    get_image_type(image).unwrap_or_else(default_image_type).get_value()
}

/// 按 Java 4.0.3 的公开规则识别 JPEG/PNG 图片类型。
///
/// Java 源码只在输入超过 28 字节时检查头部；Rust 保留该边界，但将其短 key
/// 比较修正为前缀比较，实际图片写入仍由 `easyexcel-io` 的完整探测器负责。
#[must_use]
pub fn get_image_type(image: &[u8]) -> Option<ImageType> {
    if image.len() <= 28 {
        return None;
    }
    match easyexcel_io::io::media_type::detect_image_type(image) {
        Some("jpg") => Some(ImageType::Jpeg),
        Some("png") => Some(ImageType::Png),
        _ => None,
    }
}

/// 返回 Java 可变默认图片类型的当前值。
#[must_use]
pub fn default_image_type() -> ImageType {
    *DEFAULT_IMAGE_TYPE
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// 更新 Java 可变默认图片类型的线程安全 Rust 载体。
///
/// # 参数
///
/// - `image_type`：未知图片格式回退使用的类型。
pub fn set_default_image_type(image_type: ImageType) {
    *DEFAULT_IMAGE_TYPE
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = image_type;
}
