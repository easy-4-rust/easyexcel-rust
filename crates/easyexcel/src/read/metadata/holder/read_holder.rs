//! 对应 Java：`com.alibaba.excel.read.metadata.holder.ReadHolder` (interface).

use crate::{ConfigurationHolder, ExcelReadHeadProperty};

/// 对应 Java：`ReadHolder extends ConfigurationHolder`.
pub trait ReadHolder: ConfigurationHolder {
    /// 返回当前 Holder 按注册顺序解析出的读取监听器标识。
    ///
    /// Java 保存 `List<ReadListener<?>>`；Rust 的实际监听器所有权留在 builder，
    /// Holder 保存稳定的注册标识，避免复制 trait object 生命周期。
    fn read_listener_list(&self) -> &[String];

    /// 返回当前读取表头属性。对应 Java `excelReadHeadProperty()`。
    fn excel_read_head_property(&self) -> &ExcelReadHeadProperty;
}

/// 为 Java 中继承 `AbstractReadHolder` 的具体 Holder 生成显式接口委托。
///
/// Rust 的 `Deref` 只提供方法查找，不会让具体类型自动满足
/// `ReadHolder`/`ConfigurationHolder` trait bound；因此每个 Java 具体类型仍需在其
/// 自己的文件中显式调用本宏。宏只复用委托逻辑，不创建兼容层对象。
macro_rules! delegate_read_holder_contract {
    ($holder:ty, $parent:ident) => {
        impl $crate::metadata::MetadataHolder for $holder {
            fn holder_type(&self) -> $crate::HolderEnum {
                $crate::metadata::MetadataHolder::holder_type(self.$parent())
            }
        }

        impl $crate::metadata::ConfigurationHolder for $holder {
            fn is_new(&self) -> bool {
                $crate::metadata::ConfigurationHolder::is_new(self.$parent())
            }

            fn global_configuration(&self) -> &$crate::GlobalConfiguration {
                $crate::metadata::ConfigurationHolder::global_configuration(self.$parent())
            }

            fn converter_map(&self) -> &$crate::ConverterRegistry {
                $crate::metadata::ConfigurationHolder::converter_map(self.$parent())
            }
        }

        impl $crate::read::metadata::holder::read_holder::ReadHolder for $holder {
            fn read_listener_list(&self) -> &[String] {
                $crate::read::metadata::holder::read_holder::ReadHolder::read_listener_list(
                    self.$parent(),
                )
            }

            fn excel_read_head_property(&self) -> &$crate::ExcelReadHeadProperty {
                $crate::read::metadata::holder::read_holder::ReadHolder::excel_read_head_property(
                    self.$parent(),
                )
            }
        }
    };
}

pub(crate) use delegate_read_holder_contract;
