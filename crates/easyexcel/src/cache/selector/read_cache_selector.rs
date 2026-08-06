//! 对应 Java：`com.alibaba.excel.cache.selector.ReadCacheSelector`.

use crate::read::read_cache::{ReadCacheMode, SharedStringCache};

/// Selects the shared-string cache backend for an XLSX workbook.
///
/// 对应 Java：`com.alibaba.excel.cache.selector.ReadCacheSelector`.
///
/// Java receives the `sharedStrings.xml` package part size in bytes. Rust passes
/// the same measurement into [`select_mode`](Self::select_mode). Use
/// [`SimpleReadCacheSelector`] for the default 5 MB Auto boundary, or
/// [`EternalReadCacheSelector`] to pin Memory/Moka/File regardless of size.
pub trait ReadCacheSelector: Send + Sync {
    /// Selects a cache mode for the given `sharedStrings.xml` size.
    ///
    /// 对应 Java：`readCache(PackagePart sharedStringsTablePackagePart)`.
    fn select_mode(&self, shared_strings_xml_size: u64) -> ReadCacheMode;

    /// 按选择结果创建实际共享字符串缓存后端。
    ///
    /// 默认实现按 [`Self::select_mode`] 创建标准引擎缓存；需要保留容量参数的
    /// selector 可覆盖该方法。对应 Java：`ReadCacheSelector#readCache`。
    ///
    /// # Errors
    ///
    /// 选择文件缓存且临时文件无法创建时返回错误。
    fn create_cache(
        &self,
        shared_strings_xml_size: u64,
    ) -> easyexcel_io::Result<Box<dyn SharedStringCache>> {
        easyexcel_cache::create_cache(
            self.select_mode(shared_strings_xml_size),
            shared_strings_xml_size,
        )
    }
}
