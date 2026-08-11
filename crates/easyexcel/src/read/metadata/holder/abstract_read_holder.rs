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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_creates_workbook_holder() {
        // 对应 Java：AbstractReadHolder 默认构造器
        let holder = AbstractReadHolder::default();
        assert_eq!(holder.holder_type(), HolderEnum::Workbook);
    }

    #[test]
    fn from_parameter_creates_sheet_holder() {
        // 对应 Java：fromParameter 指定 HolderType
        let holder = AbstractReadHolder::from_parameter(
            &ReadBasicParameter::default(),
            None,
            HolderEnum::Sheet,
        );
        assert_eq!(holder.holder_type(), HolderEnum::Sheet);
    }

    #[test]
    fn head_row_number_accessor() {
        // 对应 Java：headRowNumber getter/setter
        let mut holder = AbstractReadHolder::default();
        holder.set_head_row_number(3);
        assert_eq!(holder.get_head_row_number(), 3);
    }

    #[test]
    fn excel_read_head_property_accessor() {
        // 对应 Java：excelReadHeadProperty getter/setter
        let holder = AbstractReadHolder::default();
        let _prop: &ExcelReadHeadProperty = holder.get_excel_read_head_property();
        let _prop: &ExcelReadHeadProperty = holder.excel_read_head_property();
    }

    #[test]
    fn read_listener_list_accessor() {
        // 对应 Java：readListenerList getter/setter
        let mut holder = AbstractReadHolder::default();
        assert!(holder.get_read_listener_list().is_empty());
        assert!(holder.read_listener_list().is_empty());
        holder.set_read_listener_list(vec!["listener1".to_owned()]);
        assert_eq!(holder.get_read_listener_list().len(), 1);
    }

    #[test]
    fn abstract_holder_accessor() {
        // 对应 Java：abstractHolder 访问器
        let holder = AbstractReadHolder::default();
        let _ah: &AbstractHolder = holder.abstract_holder();
    }

    #[test]
    fn is_new_from_abstract_holder() {
        // 对应 Java：isNew 委托
        let holder = AbstractReadHolder::default();
        let _is_new: bool = ConfigurationHolder::is_new(&holder);
    }

    #[test]
    fn global_configuration_from_abstract_holder() {
        // 对应 Java：globalConfiguration 委托
        let holder = AbstractReadHolder::default();
        let _gc: &crate::GlobalConfiguration = ConfigurationHolder::global_configuration(&holder);
    }

    #[test]
    fn converter_map_from_abstract_holder() {
        // 对应 Java：converterMap 委托
        let holder = AbstractReadHolder::default();
        let _cm: &crate::ConverterRegistry = ConfigurationHolder::converter_map(&holder);
    }

    #[test]
    fn clone_produces_equal() {
        // 对应 Java：clone
        let holder = AbstractReadHolder::default();
        let cloned = holder.clone();
        assert_eq!(holder.holder_type(), cloned.holder_type());
    }

    #[test]
    fn debug_format_does_not_panic() {
        // 对应 Java：toString
        let holder = AbstractReadHolder::default();
        let _debug = format!("{holder:?}");
    }

    #[test]
    fn parent_inherits_listener_list() {
        // 对应 Java：子 Holder 继承父监听器列表
        let mut parent = AbstractReadHolder::default();
        parent.set_read_listener_list(vec!["p1".to_owned()]);
        let child = AbstractReadHolder::from_parameter(
            &ReadBasicParameter::default(),
            Some(&parent),
            HolderEnum::Sheet,
        );
        assert!(child.read_listener_list().contains(&"p1".to_owned()));
    }
}
