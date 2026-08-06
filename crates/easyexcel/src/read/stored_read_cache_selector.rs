//! Java-compatible stored cache selector wiring.

use crate::cache::{EternalReadCacheSelector, ReadCacheSelector, SimpleReadCacheSelector};
use crate::read::read_cache::{ReadCacheMode, SharedStringCache};

/// 对应 Java：SimpleReadCacheSelector。 Stored cache selector matching Java `ReadCacheSelector` wiring.
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

    fn create_cache(
        &self,
        shared_strings_xml_size: u64,
    ) -> easyexcel_io::Result<Box<dyn SharedStringCache>> {
        match self {
            Self::Simple(selector) => selector.create_cache(shared_strings_xml_size),
            Self::Eternal(selector) => selector.create_cache(shared_strings_xml_size),
        }
    }
}
