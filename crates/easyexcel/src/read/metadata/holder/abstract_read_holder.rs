//! 对应 Java：`com.alibaba.excel.read.metadata.holder.AbstractReadHolder`.

use std::ops::{Deref, DerefMut};

use crate::metadata::{AbstractHolder, ConfigurationHolder, MetadataHolder};
use crate::read::metadata::ReadBasicParameter;
use crate::read::metadata::holder::read_holder::ReadHolder;
use crate::{ExcelReadHeadProperty, HolderEnum};

/// 对应 Java：`AbstractReadHolder extends AbstractHolder implements ReadHolder`.
#[derive(Debug, Clone)]
pub struct AbstractReadHolder {
    abstract_holder: AbstractHolder,
    head_row_number: u32,
    excel_read_head_property: ExcelReadHeadProperty,
    /// Java listener 对象在 Rust 中由 builder 持有，这里保存有序注册标识。
    read_listener_list: Vec<String>,
}

impl AbstractReadHolder {
    /// 从读取参数和可选父 Holder 解析继承状态。
    #[must_use]
    pub fn from_parameter(
        parameter: &ReadBasicParameter,
        parent: Option<&AbstractReadHolder>,
        holder_type: HolderEnum,
    ) -> Self {
        let abstract_holder = AbstractHolder::from_parameter(
            &parameter.basic_parameter,
            parent.map(|value| &value.abstract_holder),
            holder_type,
        );
        let excel_read_head_property = ExcelReadHeadProperty::new(
            Some(&abstract_holder),
            abstract_holder.clazz.clone(),
            abstract_holder.head.clone(),
        );
        let head_row_number = if parameter.head_row_number == 0 {
            parent.map_or_else(
                || excel_read_head_property.head_row_number().max(1) as u32,
                |value| value.head_row_number,
            )
        } else {
            parameter.head_row_number
        };
        let mut read_listener_list = parent
            .map(|value| value.read_listener_list.clone())
            .unwrap_or_default();
        read_listener_list.extend(parameter.custom_read_listener_list.iter().cloned());
        Self { abstract_holder, head_row_number, excel_read_head_property, read_listener_list }
    }

    /// Java `getHeadRowNumber`。
    #[must_use] pub const fn get_head_row_number(&self) -> u32 { self.head_row_number }
    /// Java `setHeadRowNumber`。
    pub const fn set_head_row_number(&mut self, value: u32) { self.head_row_number = value; }
    /// Java `getExcelReadHeadProperty`。
    #[must_use] pub const fn get_excel_read_head_property(&self) -> &ExcelReadHeadProperty { &self.excel_read_head_property }
    /// Java `setExcelReadHeadProperty`。
    pub fn set_excel_read_head_property(&mut self, value: ExcelReadHeadProperty) { self.excel_read_head_property = value; }
    /// Java `getReadListenerList` 的后端中立视图。
    #[must_use] pub fn get_read_listener_list(&self) -> &[String] { &self.read_listener_list }
    /// Java `setReadListenerList` 的后端中立映射。
    pub fn set_read_listener_list(&mut self, value: Vec<String>) { self.read_listener_list = value; }
    /// Java `readListenerList()`。
    #[must_use] pub fn read_listener_list(&self) -> &[String] { &self.read_listener_list }
    /// Java `excelReadHeadProperty()`。
    #[must_use] pub const fn excel_read_head_property(&self) -> &ExcelReadHeadProperty { &self.excel_read_head_property }
    /// 返回父类 Holder 状态。
    #[must_use] pub const fn abstract_holder(&self) -> &AbstractHolder { &self.abstract_holder }
}

impl Default for AbstractReadHolder {
    fn default() -> Self {
        Self::from_parameter(&ReadBasicParameter::default(), None, HolderEnum::Workbook)
    }
}

impl MetadataHolder for AbstractReadHolder {
    fn holder_type(&self) -> HolderEnum {
        self.abstract_holder.holder_type
    }
}

impl ConfigurationHolder for AbstractReadHolder {
    fn is_new(&self) -> bool {
        self.abstract_holder.is_new()
    }

    fn global_configuration(&self) -> &crate::GlobalConfiguration {
        self.abstract_holder.global_configuration()
    }

    fn converter_map(&self) -> &crate::ConverterRegistry {
        self.abstract_holder.converter_map()
    }
}

impl ReadHolder for AbstractReadHolder {
    fn read_listener_list(&self) -> &[String] {
        &self.read_listener_list
    }

    fn excel_read_head_property(&self) -> &ExcelReadHeadProperty {
        &self.excel_read_head_property
    }
}

impl Deref for AbstractReadHolder {
    type Target = AbstractHolder;
    fn deref(&self) -> &Self::Target { &self.abstract_holder }
}

impl DerefMut for AbstractReadHolder {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.abstract_holder }
}
