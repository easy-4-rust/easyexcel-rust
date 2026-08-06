/// 对应 Java：无直接对应对象；Rust 架构扩展。 数字格式解析和渲染错误。
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{message}")]
pub struct NumberFormatError {
    message: String,
}

impl NumberFormatError {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 创建带诊断信息的格式错误。
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回稳定的人类可读诊断信息。
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

