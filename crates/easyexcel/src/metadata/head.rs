//! 对应 Java：`com.alibaba.excel.metadata.Head`.

use crate::core::excel_error::ExcelError;
use crate::metadata::property::{
    ColumnWidthProperty, FontProperty, LoopMergeProperty, StyleProperty,
};

/// 对应 Java：com.alibaba.excel.metadata.Head。 Excel header metadata for one column.
///
/// Rust port of Java `Head`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Head {
    /// Column index. (Java `columnIndex`)
    pub column_index: Option<i32>,
    /// Java 反射 `Field` 的后端中立字段键。
    pub field_key: Option<String>,
    /// Rust field name when bound to a model class. (Java `fieldName`)
    pub field_name: Option<String>,
    /// Header labels from the top row down. (Java `headNameList`)
    pub head_name_list: Vec<String>,
    /// Whether `@ExcelProperty.index` forced the column index. (Java `forceIndex`)
    pub force_index: Option<bool>,
    /// Whether `@ExcelProperty.value` forced the header name. (Java `forceName`)
    pub force_name: Option<bool>,
    /// Column width annotation. (Java `columnWidthProperty`)
    pub column_width_property: Option<ColumnWidthProperty>,
    /// Loop merge annotation. (Java `loopMergeProperty`)
    pub loop_merge_property: Option<LoopMergeProperty>,
    /// Header style annotation. (Java `headStyleProperty`)
    pub head_style_property: Option<StyleProperty>,
    /// Header font annotation. (Java `headFontProperty`)
    pub head_font_property: Option<FontProperty>,
}

impl Head {
    /// 对应 Java：com.alibaba.excel.metadata.Head。 Creates a head definition. (Java constructor)
    ///
    /// # Errors
    ///
    /// Rust strings cannot be null, so every supplied label is valid,
    /// including the empty string. This matches Java `Head`, which rejects
    /// null labels but permits empty labels.
    pub fn new(
        column_index: i32,
        field_name: Option<String>,
        head_name_list: Vec<String>,
        force_index: bool,
        force_name: bool,
    ) -> Result<Self, ExcelError> {
        let field_key = field_name.clone();
        Ok(Self {
            column_index: Some(column_index),
            field_key,
            field_name,
            head_name_list,
            force_index: Some(force_index),
            force_name: Some(force_name),
            column_width_property: None,
            loop_merge_property: None,
            head_style_property: None,
            head_font_property: None,
        })
    }

    /// 使用 Java 六参数构造器的后端中立形状创建表头。
    ///
    /// `field_key` 保存 Java 反射 `Field` 的稳定字段名，`field_name` 独立保存 Java 的
    /// `fieldName`。Rust 无 null 字符串元素，因此 `None` 的 head list 规范化为空集合。
    ///
    /// # Errors
    ///
    /// Rust 字符串无法表示 Java 集合中的 null 元素，所以已构造的输入总是有效。
    pub fn from_java_fields(
        column_index: Option<i32>,
        field_key: Option<String>,
        field_name: Option<String>,
        head_name_list: Option<Vec<String>>,
        force_index: Option<bool>,
        force_name: Option<bool>,
    ) -> Result<Self, ExcelError> {
        Ok(Self {
            column_index,
            field_key,
            field_name,
            head_name_list: head_name_list.unwrap_or_default(),
            force_index,
            force_name,
            column_width_property: None,
            loop_merge_property: None,
            head_style_property: None,
            head_font_property: None,
        })
    }

    /// 对应 Java：com.alibaba.excel.metadata.Head。 Returns the column index. (Java `getColumnIndex()`)
    #[must_use]
    pub fn column_index(&self) -> Option<i32> {
        self.column_index
    }

    /// 对应 Java：com.alibaba.excel.metadata.Head。 Returns the field name. (Java `getFieldName()`)
    #[must_use]
    pub fn field_name(&self) -> Option<&str> {
        self.field_name.as_deref()
    }

    /// 对应 Java：com.alibaba.excel.metadata.Head。 Returns the header labels. (Java `getHeadNameList()`)
    #[must_use]
    pub fn head_name_list(&self) -> &[String] {
        &self.head_name_list
    }

    /// Returns whether the column index was forced. (Java `getForceIndex()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.Head。
    pub fn force_index(&self) -> bool {
        self.force_index.unwrap_or(false)
    }

    /// Returns whether the header name was forced. (Java `getForceName()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.Head。
    pub fn force_name(&self) -> bool {
        self.force_name.unwrap_or(false)
    }

    /// Java `getColumnIndex` 别名。
    #[must_use]
    pub fn get_column_index(&self) -> Option<i32> { self.column_index }
    /// Java `setColumnIndex`。
    pub fn set_column_index(&mut self, value: Option<i32>) { self.column_index = value; }
    /// Java `getField` 的后端中立映射；Rust 以静态字段名替代反射 `Field`。
    #[must_use]
    pub fn get_field(&self) -> Option<&str> { self.field_key.as_deref() }
    /// Java `setField` 的后端中立映射。
    pub fn set_field(&mut self, value: Option<String>) { self.field_key = value; }
    /// Java `getFieldName` 别名。
    #[must_use]
    pub fn get_field_name(&self) -> Option<&str> { self.field_name.as_deref() }
    /// Java `setFieldName`。
    pub fn set_field_name(&mut self, value: Option<String>) { self.field_name = value; }
    /// Java `getHeadNameList` 别名。
    #[must_use]
    pub fn get_head_name_list(&self) -> &[String] { &self.head_name_list }
    /// Java `setHeadNameList`。
    pub fn set_head_name_list(&mut self, value: Vec<String>) { self.head_name_list = value; }
    /// Java `getForceIndex` 别名。
    #[must_use]
    pub const fn get_force_index(&self) -> Option<bool> { self.force_index }
    /// Java `setForceIndex`。
    pub const fn set_force_index(&mut self, value: Option<bool>) { self.force_index = value; }
    /// Java `getForceName` 别名。
    #[must_use]
    pub const fn get_force_name(&self) -> Option<bool> { self.force_name }
    /// Java `setForceName`。
    pub const fn set_force_name(&mut self, value: Option<bool>) { self.force_name = value; }
    /// Java `getColumnWidthProperty`。
    #[must_use]
    pub const fn get_column_width_property(&self) -> Option<&ColumnWidthProperty> {
        self.column_width_property.as_ref()
    }
    /// Java `setColumnWidthProperty`。
    pub fn set_column_width_property(&mut self, value: Option<ColumnWidthProperty>) {
        self.column_width_property = value;
    }
    /// Java `getLoopMergeProperty`。
    #[must_use]
    pub const fn get_loop_merge_property(&self) -> Option<&LoopMergeProperty> {
        self.loop_merge_property.as_ref()
    }
    /// Java `setLoopMergeProperty`。
    pub fn set_loop_merge_property(&mut self, value: Option<LoopMergeProperty>) {
        self.loop_merge_property = value;
    }
    /// Java `getHeadStyleProperty`。
    #[must_use]
    pub const fn get_head_style_property(&self) -> Option<&StyleProperty> {
        self.head_style_property.as_ref()
    }
    /// Java `setHeadStyleProperty`。
    pub fn set_head_style_property(&mut self, value: Option<StyleProperty>) {
        self.head_style_property = value;
    }
    /// Java `getHeadFontProperty`。
    #[must_use]
    pub const fn get_head_font_property(&self) -> Option<&FontProperty> {
        self.head_font_property.as_ref()
    }
    /// Java `setHeadFontProperty`。
    pub fn set_head_font_property(&mut self, value: Option<FontProperty>) {
        self.head_font_property = value;
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn force_index_and_force_name_accessors() {
        // 对应 Java：Head.getForceIndex / getForceName
        let head = Head::new(
            2,
            Some("field".to_owned()),
            vec!["Name".to_owned()],
            true,
            true,
        )
        .expect("head");
        assert!(head.force_index());
        assert!(head.force_name());
        assert_eq!(head.column_index(), Some(2));
        assert_eq!(head.field_name(), Some("field"));
        assert_eq!(head.head_name_list(), &["Name".to_owned()][..]);

        let plain = Head::new(0, None, Vec::new(), false, false).expect("head");
        assert!(!plain.force_index());
        assert!(!plain.force_name());
        assert_eq!(plain.column_index(), Some(0));
    }

    #[test]
    fn from_java_fields_all_none() {
        // 对应 Java：Head 全 None 构造
        let head = Head::from_java_fields(None, None, None, None, None, None).expect("head");
        assert!(head.column_index().is_none());
        assert!(head.get_field().is_none());
        assert!(head.field_name().is_none());
        assert!(head.head_name_list().is_empty());
        assert!(!head.force_index());
        assert!(!head.force_name());
    }

    #[test]
    fn from_java_fields_with_values() {
        // 对应 Java：Head 全参数构造
        let head = Head::from_java_fields(
            Some(5),
            Some("key".to_owned()),
            Some("name".to_owned()),
            Some(vec!["A".to_owned(), "B".to_owned()]),
            Some(true),
            Some(false),
        )
        .expect("head");
        assert_eq!(head.column_index(), Some(5));
        assert_eq!(head.get_field(), Some("key"));
        assert_eq!(head.field_name(), Some("name"));
        assert_eq!(head.head_name_list().len(), 2);
        assert!(head.force_index());
        assert!(!head.force_name());
    }

    #[test]
    fn set_column_index_and_get() {
        // 对应 Java：setColumnIndex / getColumnIndex
        let mut head = Head::new(0, None, Vec::new(), false, false).expect("head");
        head.set_column_index(Some(10));
        assert_eq!(head.get_column_index(), Some(10));
    }

    #[test]
    fn set_field_and_get() {
        // 对应 Java：setField / getField
        let mut head = Head::new(0, None, Vec::new(), false, false).expect("head");
        head.set_field(Some("myKey".to_owned()));
        assert_eq!(head.get_field(), Some("myKey"));
    }

    #[test]
    fn set_field_name_and_get() {
        // 对应 Java：setFieldName / getFieldName
        let mut head = Head::new(0, None, Vec::new(), false, false).expect("head");
        head.set_field_name(Some("myName".to_owned()));
        assert_eq!(head.get_field_name(), Some("myName"));
    }

    #[test]
    fn set_head_name_list_and_get() {
        // 对应 Java：setHeadNameList / getHeadNameList
        let mut head = Head::new(0, None, Vec::new(), false, false).expect("head");
        head.set_head_name_list(vec!["Col1".to_owned(), "Col2".to_owned()]);
        assert_eq!(head.get_head_name_list().len(), 2);
    }

    #[test]
    fn set_force_index_and_get() {
        // 对应 Java：setForceIndex / getForceIndex
        let mut head = Head::new(0, None, Vec::new(), false, false).expect("head");
        head.set_force_index(Some(true));
        assert_eq!(head.get_force_index(), Some(true));
    }

    #[test]
    fn set_force_name_and_get() {
        // 对应 Java：setForceName / getForceName
        let mut head = Head::new(0, None, Vec::new(), false, false).expect("head");
        head.set_force_name(Some(true));
        assert_eq!(head.get_force_name(), Some(true));
    }

    #[test]
    fn column_width_property_getter_setter() {
        // 对应 Java：columnWidthProperty getter/setter
        let mut head = Head::new(0, None, Vec::new(), false, false).expect("head");
        assert!(head.get_column_width_property().is_none());
        head.set_column_width_property(None);
        assert!(head.get_column_width_property().is_none());
    }

    #[test]
    fn loop_merge_property_getter_setter() {
        // 对应 Java：loopMergeProperty getter/setter
        let mut head = Head::new(0, None, Vec::new(), false, false).expect("head");
        assert!(head.get_loop_merge_property().is_none());
        head.set_loop_merge_property(None);
        assert!(head.get_loop_merge_property().is_none());
    }

    #[test]
    fn head_style_property_getter_setter() {
        // 对应 Java：headStyleProperty getter/setter
        let mut head = Head::new(0, None, Vec::new(), false, false).expect("head");
        assert!(head.get_head_style_property().is_none());
        head.set_head_style_property(None);
        assert!(head.get_head_style_property().is_none());
    }

    #[test]
    fn head_font_property_getter_setter() {
        // 对应 Java：headFontProperty getter/setter
        let mut head = Head::new(0, None, Vec::new(), false, false).expect("head");
        assert!(head.get_head_font_property().is_none());
        head.set_head_font_property(None);
        assert!(head.get_head_font_property().is_none());
    }

    #[test]
    fn clone_produces_equal() {
        // 对应 Java：clone
        let head = Head::new(1, Some("f".to_owned()), vec!["N".to_owned()], true, false)
            .expect("head");
        let cloned = head.clone();
        assert_eq!(head, cloned);
    }

    #[test]
    fn debug_format_does_not_panic() {
        // 对应 Java：toString
        let head = Head::new(0, None, Vec::new(), false, false).expect("head");
        let _debug = format!("{head:?}");
    }
}
