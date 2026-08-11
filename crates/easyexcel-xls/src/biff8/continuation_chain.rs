//! BIFF8 逻辑记录的物理 CONTINUE 分段。

use easyexcel_io::Result;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 保存一个 BIFF8 逻辑记录及其后续 `CONTINUE` 记录体。
///
/// 该类型只负责二进制分段的所有权与解码入口；上层事件分派器决定链属于
/// SST、公式字符串还是其他逻辑记录。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Biff8ContinuationChain {
    segments: Vec<Vec<u8>>,
}

impl Biff8ContinuationChain {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 使用逻辑记录的第一个物理记录体创建分段链。
    #[must_use]
    pub fn new(first_segment: &[u8]) -> Self {
        Self {
            segments: vec![first_segment.to_vec()],
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 追加一个 `CONTINUE` 记录体。
    pub fn push(&mut self, continuation: &[u8]) {
        self.segments.push(continuation.to_vec());
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回物理记录体，供特定 BIFF 解码器读取。
    #[must_use]
    pub fn segments(&self) -> &[Vec<u8>] {
        &self.segments
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 将当前分段链解码为共享字符串表。
    ///
    /// 当 `xls-lazy-sst` feature 启用时返回延迟解码容器 `LazySst`，
    /// 否则返回立即解码的 `Vec<Biff8SstString>`。
    ///
    /// # Errors
    ///
    /// SST 元数据、字符串标志或 CONTINUE 边界损坏时返回错误。
    #[cfg(feature = "xls-lazy-sst")]
    pub fn decode_sst(&self) -> Result<super::lazy_sst::LazySst> {
        super::lazy_sst::LazySst::new(self.segments())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 将当前分段链解码为共享字符串表。
    ///
    /// # Errors
    ///
    /// SST 元数据、字符串标志或 CONTINUE 边界损坏时返回错误。
    #[cfg(not(feature = "xls-lazy-sst"))]
    pub fn decode_sst(&self) -> Result<Vec<crate::xls::Biff8SstString>> {
        super::string::decode_sst_segments(self.segments())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 将当前分段链解码为一个 BIFF8 Unicode 字符串。
    ///
    /// # Errors
    ///
    /// 字符计数、编码标志或 CONTINUE 边界损坏时返回错误。
    pub fn decode_unicode_string(&self) -> Result<String> {
        super::string::decode_unicode_string_segments(self.segments())
    }
}
