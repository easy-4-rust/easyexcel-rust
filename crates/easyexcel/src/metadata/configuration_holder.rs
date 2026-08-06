//! 对应 Java：`com.alibaba.excel.metadata.ConfigurationHolder` and `Holder`.

use crate::ConverterRegistry;
use crate::Holder as HolderEnum;

use super::global_configuration::GlobalConfiguration;

include!("configuration_holder/metadata_holder.rs");

/// 对应 Java：com.alibaba.excel.metadata.ConfigurationHolder。 Read/write holder configuration contract.
///
/// Rust port of Java `ConfigurationHolder extends Holder`.
pub trait ConfigurationHolder: MetadataHolder {
    /// Returns whether the holder was freshly initialized. (Java `isNew()`)
    fn is_new(&self) -> bool;

    /// Returns the global configuration. (Java `globalConfiguration()`)
    fn global_configuration(&self) -> &GlobalConfiguration;

    /// Returns the active converter registry. (Java `converterMap()`)
    fn converter_map(&self) -> &ConverterRegistry;
}
