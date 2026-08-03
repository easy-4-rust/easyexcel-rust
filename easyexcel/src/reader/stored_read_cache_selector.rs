//! Java-compatible stored cache selector wiring.

use crate::reader::cache::{
    EternalReadCacheSelector, ReadCacheSelector, SimpleReadCacheSelector,
};
use crate::reader::read_cache::ReadCacheMode;

/// Stored cache selector matching Java `ReadCacheSelector` wiring.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoredReadCacheSelector {
    /// 对应 Java：`SimpleReadCacheSelector`.
    Simple(SimpleReadCacheSelector),
    /// 对应 Java：`EternalReadCacheSelector`.
    Eternal(EternalReadCacheSelector),
}

impl ReadCacheSelector for StoredReadCacheSelector {
    fn select_mode(&self, shared_strings_xml_size: u64) -> ReadCacheMode {
        match self {
            Self::Simple(selector) => selector.select_mode(shared_strings_xml_size),
            Self::Eternal(selector) => selector.select_mode(shared_strings_xml_size),
        }
    }
}
