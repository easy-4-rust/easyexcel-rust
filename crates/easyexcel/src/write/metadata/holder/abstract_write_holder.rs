//! 对应 Java：`com.alibaba.excel.write.metadata.holder.AbstractWriteHolder`.

use std::collections::HashSet;

use crate::converters::default_converter_loader::load_default_write_converter;
use crate::core::ConverterRegistry;
use crate::core::ExcelCellStyle;
use crate::core::ExcelFontStyle;
use crate::core::ExcelWriteMetadata;

use crate::core::ExcelWriteHeadProperty;
use crate::metadata::AbstractHolder;
use crate::write::WriteHolder;
use crate::write::metadata::WriteBasicParameter;
use crate::write::handler::chain::cell_handler_execution_chain::CellHandlerExecutionChain;
use crate::write::handler::chain::row_handler_execution_chain::RowHandlerExecutionChain;
use crate::write::handler::chain::sheet_handler_execution_chain::SheetHandlerExecutionChain;
use crate::write::handler::chain::workbook_handler_execution_chain::WorkbookHandlerExecutionChain;

/// 对应 Java：com.alibaba.excel.write.metadata.holder.AbstractWriteHolder。 对应 Java：`AbstractWriteHolder extends AbstractHolder implements WriteHolder`.
///
/// The Java side carries resolved nullable parameters inherited from the
/// parent holder. Rust keeps the same resolved state here; builders use
/// [`crate::WriteOptions`] for the live backend while handler-facing
/// compatibility APIs use this holder.
// 语义敏感：needHead/useDefaultStyle/automaticMergeHead 等布尔字段与 Java
// `AbstractWriteHolder` 一一对应，合并会破坏 1:1 可追溯性。
#[allow(clippy::struct_excessive_bools)]
/// 对应 Java：com.alibaba.excel.write.metadata.holder.AbstractWriteHolder。
pub struct AbstractWriteHolder {
    abstract_holder: AbstractHolder,
    /// Java 继承后的 workbook handler 链。
    pub workbook_handler_execution_chain: WorkbookHandlerExecutionChain,
    /// Java 当前 Holder 自有 workbook handler 链。
    pub own_workbook_handler_execution_chain: WorkbookHandlerExecutionChain,
    /// Java 继承后的 sheet handler 链。
    pub sheet_handler_execution_chain: SheetHandlerExecutionChain,
    /// Java 当前 Holder 自有 sheet handler 链。
    pub own_sheet_handler_execution_chain: SheetHandlerExecutionChain,
    /// Java row handler 链。
    pub row_handler_execution_chain: RowHandlerExecutionChain,
    /// Java cell handler 链。
    pub cell_handler_execution_chain: CellHandlerExecutionChain,
    /// Java Holder 原始 handler 注册表。
    pub write_handler_list: Vec<Box<dyn crate::WriteHandler>>,
    /// Mirrors `AbstractWriteHolder.needHead`.
    pub need_head: bool,
    /// Mirrors `AbstractWriteHolder.relativeHeadRowIndex`.
    pub relative_head_row_index: i32,
    /// Mirrors `AbstractWriteHolder.useDefaultStyle`.
    pub use_default_style: bool,
    /// Mirrors `AbstractWriteHolder.automaticMergeHead`.
    pub automatic_merge_head: bool,
    /// Mirrors `AbstractWriteHolder.excelWriteHeadProperty`.
    pub excel_write_head_property: ExcelWriteHeadProperty,
    /// Mirrors `AbstractWriteHolder.headStyle`.
    pub head_style: Option<ExcelCellStyle>,
    /// Mirrors `AbstractWriteHolder.contentStyle`.
    pub content_style: Option<ExcelCellStyle>,
    /// Mirrors `AbstractWriteHolder.headFontStyle`.
    pub head_font_style: Option<ExcelFontStyle>,
    /// Mirrors `AbstractWriteHolder.contentFontStyle`.
    pub content_font_style: Option<ExcelFontStyle>,
    /// Mirrors `AbstractWriteHolder.excludeColumnIndexes`.
    pub exclude_column_indexes: Option<HashSet<usize>>,
    /// Mirrors `AbstractWriteHolder.excludeColumnFieldNames`.
    pub exclude_column_field_names: Option<HashSet<String>>,
    /// Mirrors `AbstractWriteHolder.includeColumnIndexes`.
    pub include_column_indexes: Option<HashSet<usize>>,
    /// Mirrors `AbstractWriteHolder.includeColumnFieldNames`.
    pub include_column_field_names: Option<HashSet<String>>,
    /// Mirrors `AbstractWriteHolder.orderByIncludeColumn`.
    pub order_by_include_column: bool,
    /// Mirrors `AbstractHolder.converterMap`.
    pub converter_map: ConverterRegistry,
}

impl Default for AbstractWriteHolder {
    fn default() -> Self {
        Self {
            abstract_holder: AbstractHolder::default(),
            workbook_handler_execution_chain: WorkbookHandlerExecutionChain::new(),
            own_workbook_handler_execution_chain: WorkbookHandlerExecutionChain::new(),
            sheet_handler_execution_chain: SheetHandlerExecutionChain::new(),
            own_sheet_handler_execution_chain: SheetHandlerExecutionChain::new(),
            row_handler_execution_chain: RowHandlerExecutionChain::new(),
            cell_handler_execution_chain: CellHandlerExecutionChain::new(),
            write_handler_list: Vec::new(),
            need_head: true,
            relative_head_row_index: 0,
            use_default_style: true,
            automatic_merge_head: true,
            excel_write_head_property: ExcelWriteHeadProperty::new(),
            head_style: None,
            content_style: None,
            head_font_style: None,
            content_font_style: None,
            exclude_column_indexes: None,
            exclude_column_field_names: None,
            include_column_indexes: None,
            include_column_field_names: None,
            order_by_include_column: false,
            converter_map: load_default_write_converter(),
        }
    }
}

impl AbstractWriteHolder {
    /// 返回 Java 父类 Holder。
    #[must_use] pub const fn abstract_holder(&self) -> &AbstractHolder { &self.abstract_holder }
    /// 返回可变 Java 父类 Holder。
    pub const fn abstract_holder_mut(&mut self) -> &mut AbstractHolder { &mut self.abstract_holder }
    /// 返回是否写表头。
    #[must_use]
    pub const fn get_need_head(&self) -> bool { self.need_head }
    /// 设置是否写表头。
    pub const fn set_need_head(&mut self, value: bool) { self.need_head = value; }
    /// 返回表头相对起始行。
    #[must_use]
    pub const fn get_relative_head_row_index(&self) -> i32 { self.relative_head_row_index }
    /// 设置表头相对起始行。
    pub const fn set_relative_head_row_index(&mut self, value: i32) { self.relative_head_row_index = value; }
    /// 返回解析后的写表头属性。
    #[must_use]
    pub const fn get_excel_write_head_property(&self) -> &ExcelWriteHeadProperty { &self.excel_write_head_property }
    /// 返回是否使用默认样式。
    #[must_use]
    pub const fn get_use_default_style(&self) -> bool { self.use_default_style }
    /// 设置是否使用默认样式。
    pub const fn set_use_default_style(&mut self, value: bool) { self.use_default_style = value; }
    /// 返回是否自动合并表头。
    #[must_use]
    pub const fn get_automatic_merge_head(&self) -> bool { self.automatic_merge_head }
    /// 设置是否自动合并表头。
    pub const fn set_automatic_merge_head(&mut self, value: bool) { self.automatic_merge_head = value; }
    /// 返回排除列索引。
    #[must_use]
    pub const fn get_exclude_column_indexes(&self) -> Option<&HashSet<usize>> { self.exclude_column_indexes.as_ref() }
    /// 设置排除列索引；`None` 保留 Java null 语义。
    pub fn set_exclude_column_indexes(&mut self, value: Option<HashSet<usize>>) { self.exclude_column_indexes = value; }
    /// 返回排除字段名。
    #[must_use]
    pub const fn get_exclude_column_field_names(&self) -> Option<&HashSet<String>> { self.exclude_column_field_names.as_ref() }
    /// 设置排除字段名。
    pub fn set_exclude_column_field_names(&mut self, value: Option<HashSet<String>>) { self.exclude_column_field_names = value; }
    /// 返回包含列索引。
    #[must_use]
    pub const fn get_include_column_indexes(&self) -> Option<&HashSet<usize>> { self.include_column_indexes.as_ref() }
    /// 设置包含列索引。
    pub fn set_include_column_indexes(&mut self, value: Option<HashSet<usize>>) { self.include_column_indexes = value; }
    /// 返回包含字段名。
    #[must_use]
    pub const fn get_include_column_field_names(&self) -> Option<&HashSet<String>> { self.include_column_field_names.as_ref() }
    /// 设置包含字段名。
    pub fn set_include_column_field_names(&mut self, value: Option<HashSet<String>>) { self.include_column_field_names = value; }
    /// 返回是否按 include 顺序输出。
    #[must_use]
    pub const fn get_order_by_include_column(&self) -> bool { self.order_by_include_column }
    /// 设置是否按 include 顺序输出。
    pub const fn set_order_by_include_column(&mut self, value: bool) { self.order_by_include_column = value; }
    /// 替换 converter 注册表。
    pub fn set_converter_map(&mut self, value: ConverterRegistry) { self.converter_map = value; }
    /// Java `getHeadStyle`。
    #[must_use] pub const fn get_head_style(&self) -> Option<ExcelCellStyle> { self.head_style }
    /// Java `setHeadStyle`。
    pub const fn set_head_style(&mut self, value: Option<ExcelCellStyle>) { self.head_style = value; }
    /// Java `getContentStyle`。
    #[must_use] pub const fn get_content_style(&self) -> Option<ExcelCellStyle> { self.content_style }
    /// Java `setContentStyle`。
    pub const fn set_content_style(&mut self, value: Option<ExcelCellStyle>) { self.content_style = value; }
    /// Java `getHeadFontStyle`。
    #[must_use] pub const fn get_head_font_style(&self) -> Option<ExcelFontStyle> { self.head_font_style }
    /// Java `setHeadFontStyle`。
    pub const fn set_head_font_style(&mut self, value: Option<ExcelFontStyle>) { self.head_font_style = value; }
    /// Java `getContentFontStyle`。
    #[must_use] pub const fn get_content_font_style(&self) -> Option<ExcelFontStyle> { self.content_font_style }
    /// Java `setContentFontStyle`。
    pub const fn set_content_font_style(&mut self, value: Option<ExcelFontStyle>) { self.content_font_style = value; }

    /// Java `getWorkbookHandlerExecutionChain`。
    #[must_use]
    pub const fn get_workbook_handler_execution_chain(&self) -> &WorkbookHandlerExecutionChain {
        &self.workbook_handler_execution_chain
    }
    /// Java `setWorkbookHandlerExecutionChain`。
    pub fn set_workbook_handler_execution_chain(&mut self, value: WorkbookHandlerExecutionChain) {
        self.workbook_handler_execution_chain = value;
    }
    /// Java `getOwnWorkbookHandlerExecutionChain`。
    #[must_use]
    pub const fn get_own_workbook_handler_execution_chain(&self) -> &WorkbookHandlerExecutionChain {
        &self.own_workbook_handler_execution_chain
    }
    /// Java `setOwnWorkbookHandlerExecutionChain`。
    pub fn set_own_workbook_handler_execution_chain(&mut self, value: WorkbookHandlerExecutionChain) {
        self.own_workbook_handler_execution_chain = value;
    }
    /// Java `getSheetHandlerExecutionChain`。
    #[must_use]
    pub const fn get_sheet_handler_execution_chain(&self) -> &SheetHandlerExecutionChain {
        &self.sheet_handler_execution_chain
    }
    /// Java `setSheetHandlerExecutionChain`。
    pub fn set_sheet_handler_execution_chain(&mut self, value: SheetHandlerExecutionChain) {
        self.sheet_handler_execution_chain = value;
    }
    /// Java `getOwnSheetHandlerExecutionChain`。
    #[must_use]
    pub const fn get_own_sheet_handler_execution_chain(&self) -> &SheetHandlerExecutionChain {
        &self.own_sheet_handler_execution_chain
    }
    /// Java `setOwnSheetHandlerExecutionChain`。
    pub fn set_own_sheet_handler_execution_chain(&mut self, value: SheetHandlerExecutionChain) {
        self.own_sheet_handler_execution_chain = value;
    }
    /// Java `getRowHandlerExecutionChain`。
    #[must_use]
    pub const fn get_row_handler_execution_chain(&self) -> &RowHandlerExecutionChain {
        &self.row_handler_execution_chain
    }
    /// Java `setRowHandlerExecutionChain`。
    pub fn set_row_handler_execution_chain(&mut self, value: RowHandlerExecutionChain) {
        self.row_handler_execution_chain = value;
    }
    /// Java `getCellHandlerExecutionChain`。
    #[must_use]
    pub const fn get_cell_handler_execution_chain(&self) -> &CellHandlerExecutionChain {
        &self.cell_handler_execution_chain
    }
    /// Java `setCellHandlerExecutionChain`。
    pub fn set_cell_handler_execution_chain(&mut self, value: CellHandlerExecutionChain) {
        self.cell_handler_execution_chain = value;
    }
    /// Java `getWriteHandlerList`。
    #[must_use]
    pub fn get_write_handler_list(&self) -> &[Box<dyn crate::WriteHandler>] {
        &self.write_handler_list
    }
    /// Java `setWriteHandlerList`。
    pub fn set_write_handler_list(&mut self, value: Vec<Box<dyn crate::WriteHandler>>) {
        self.write_handler_list = value;
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.holder.AbstractWriteHolder。 Resolves Java nullable write parameters against an optional parent.
    ///
    /// A missing collection inherits the parent collection, while an explicit
    /// empty collection clears it. This distinction is required by Java
    /// `AbstractWriteHolder(WriteBasicParameter, parent)`.
    #[must_use]
    pub fn from_parameter(
        parameter: &WriteBasicParameter,
        parent: Option<&AbstractWriteHolder>,
    ) -> Self {
        let defaults = Self::default();
        Self {
            abstract_holder: AbstractHolder::from_parameter(
                &parameter.basic_parameter,
                parent.map(|holder| &holder.abstract_holder),
                parent.map_or(crate::HolderEnum::Workbook, |holder| holder.abstract_holder.holder_type),
            ),
            workbook_handler_execution_chain: WorkbookHandlerExecutionChain::new(),
            own_workbook_handler_execution_chain: WorkbookHandlerExecutionChain::new(),
            sheet_handler_execution_chain: SheetHandlerExecutionChain::new(),
            own_sheet_handler_execution_chain: SheetHandlerExecutionChain::new(),
            row_handler_execution_chain: RowHandlerExecutionChain::new(),
            cell_handler_execution_chain: CellHandlerExecutionChain::new(),
            write_handler_list: Vec::new(),
            need_head: parameter
                .need_head
                .or_else(|| parent.map(|holder| holder.need_head))
                .unwrap_or(defaults.need_head),
            relative_head_row_index: parameter
                .relative_head_row_index
                .or_else(|| parent.map(|holder| holder.relative_head_row_index))
                .unwrap_or(defaults.relative_head_row_index),
            use_default_style: parameter
                .use_default_style
                .or_else(|| parent.map(|holder| holder.use_default_style))
                .unwrap_or(defaults.use_default_style),
            automatic_merge_head: parameter
                .automatic_merge_head
                .or_else(|| parent.map(|holder| holder.automatic_merge_head))
                .unwrap_or(defaults.automatic_merge_head),
            exclude_column_indexes: resolve_set(
                parameter.exclude_column_indexes.as_ref(),
                parent.and_then(|holder| holder.exclude_column_indexes.as_ref()),
            ),
            exclude_column_field_names: resolve_set(
                parameter.exclude_column_field_names.as_ref(),
                parent.and_then(|holder| holder.exclude_column_field_names.as_ref()),
            ),
            include_column_indexes: resolve_set(
                parameter.include_column_indexes.as_ref(),
                parent.and_then(|holder| holder.include_column_indexes.as_ref()),
            ),
            include_column_field_names: resolve_set(
                parameter.include_column_field_names.as_ref(),
                parent.and_then(|holder| holder.include_column_field_names.as_ref()),
            ),
            order_by_include_column: parameter
                .order_by_include_column
                .or_else(|| parent.map(|holder| holder.order_by_include_column))
                .unwrap_or(defaults.order_by_include_column),
            excel_write_head_property: ExcelWriteHeadProperty::new(),
            head_style: parent.and_then(|holder| holder.head_style),
            content_style: parent.and_then(|holder| holder.content_style),
            head_font_style: parent.and_then(|holder| holder.head_font_style),
            content_font_style: parent.and_then(|holder| holder.content_font_style),
            converter_map: parent
                .map_or_else(load_default_write_converter, |holder| {
                    holder.converter_map.clone()
                })
                .merged_with(&parameter.converters),
        }
    }

    /// Returns the effective converter map inherited by this holder.
    /// (Java `ConfigurationHolder.converterMap()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.holder.AbstractWriteHolder。
    pub const fn converter_map(&self) -> &ConverterRegistry {
        &self.converter_map
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.holder.AbstractWriteHolder。 Replaces the resolved head property carried by this holder.
    ///
    /// Java creates this property during every holder constructor. Rust
    /// builders can resolve schema and dynamic-head information later, so the
    /// assignment is explicit rather than hidden behind a metadata placeholder.
    pub fn set_excel_write_head_property(&mut self, property: ExcelWriteHeadProperty) {
        self.excel_write_head_property = property;
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.holder.AbstractWriteHolder。 Resolves a raw dynamic/class head into this holder. (Java constructor)
    pub fn resolve_head(
        &mut self,
        head_clazz: Option<String>,
        head: Option<Vec<Vec<String>>>,
        metadata: ExcelWriteMetadata,
    ) {
        self.excel_write_head_property =
            ExcelWriteHeadProperty::from_head(None, head_clazz, head, metadata);
    }

    /// Java `needHead()`。
    #[must_use]
    pub const fn need_head(&self) -> bool { self.need_head }
    /// Java `relativeHeadRowIndex()`。
    #[must_use]
    pub const fn relative_head_row_index(&self) -> i32 { self.relative_head_row_index }
    /// Java `automaticMergeHead()`。
    #[must_use]
    pub const fn automatic_merge_head(&self) -> bool { self.automatic_merge_head }
    /// Java `orderByIncludeColumn()`。
    #[must_use]
    pub const fn order_by_include_column(&self) -> bool { self.order_by_include_column }
    /// Java `ignore(String, Integer)`。
    #[must_use]
    pub fn ignore(&self, field_name: Option<&str>, column_index: Option<usize>) -> bool {
        WriteHolder::ignore(self, field_name, column_index)
    }
    /// Java `excelWriteHeadProperty()`。
    #[must_use]
    pub const fn excel_write_head_property(&self) -> &ExcelWriteHeadProperty {
        &self.excel_write_head_property
    }
    /// Java `includeColumnIndexes()`。
    #[must_use]
    pub const fn include_column_indexes(&self) -> Option<&HashSet<usize>> {
        self.include_column_indexes.as_ref()
    }
    /// Java `includeColumnFieldNames()`。
    #[must_use]
    pub const fn include_column_field_names(&self) -> Option<&HashSet<String>> {
        self.include_column_field_names.as_ref()
    }
    /// Java `excludeColumnIndexes()`。
    #[must_use]
    pub const fn exclude_column_indexes(&self) -> Option<&HashSet<usize>> {
        self.exclude_column_indexes.as_ref()
    }
    /// Java `excludeColumnFieldNames()`。
    #[must_use]
    pub const fn exclude_column_field_names(&self) -> Option<&HashSet<String>> {
        self.exclude_column_field_names.as_ref()
    }
}

impl std::ops::Deref for AbstractWriteHolder {
    type Target = AbstractHolder;
    fn deref(&self) -> &Self::Target { &self.abstract_holder }
}

impl std::ops::DerefMut for AbstractWriteHolder {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.abstract_holder }
}

impl WriteHolder for AbstractWriteHolder {
    fn excel_write_head_property(&self) -> &ExcelWriteHeadProperty {
        &self.excel_write_head_property
    }

    fn ignore(&self, field_name: Option<&str>, column_index: Option<usize>) -> bool {
        if let Some(field_name) = field_name {
            if self
                .include_column_field_names
                .as_ref()
                .is_some_and(|names| !names.contains(field_name))
            {
                return true;
            }
            if self
                .exclude_column_field_names
                .as_ref()
                .is_some_and(|names| names.contains(field_name))
            {
                return true;
            }
        }
        if let Some(column_index) = column_index {
            if self
                .include_column_indexes
                .as_ref()
                .is_some_and(|indexes| !indexes.contains(&column_index))
            {
                return true;
            }
            if self
                .exclude_column_indexes
                .as_ref()
                .is_some_and(|indexes| indexes.contains(&column_index))
            {
                return true;
            }
        }
        false
    }

    fn need_head(&self) -> bool {
        self.need_head
    }

    fn relative_head_row_index(&self) -> i32 {
        self.relative_head_row_index
    }

    fn automatic_merge_head(&self) -> bool {
        self.automatic_merge_head
    }

    fn order_by_include_column(&self) -> bool {
        self.order_by_include_column
    }

    fn include_column_indexes(&self) -> Option<&HashSet<usize>> {
        self.include_column_indexes.as_ref()
    }

    fn include_column_field_names(&self) -> Option<&HashSet<String>> {
        self.include_column_field_names.as_ref()
    }

    fn exclude_column_indexes(&self) -> Option<&HashSet<usize>> {
        self.exclude_column_indexes.as_ref()
    }

    fn exclude_column_field_names(&self) -> Option<&HashSet<String>> {
        self.exclude_column_field_names.as_ref()
    }
}

fn resolve_set<T>(own: Option<&Vec<T>>, parent: Option<&HashSet<T>>) -> Option<HashSet<T>>
where
    T: Clone + Eq + std::hash::Hash,
{
    own.map(|values| values.iter().cloned().collect())
        .or_else(|| parent.cloned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    struct PrefixConverter(&'static str);

    impl crate::core::Converter<String> for PrefixConverter {
        fn convert_to_excel_data(
            &self,
            context: &crate::core::WriteConverterContext<'_, String>,
        ) -> crate::core::Result<crate::core::WriteCellData> {
            Ok(crate::core::WriteCellData::from_string(format!(
                "{}:{}",
                self.0,
                context.value()
            )))
        }
    }

    fn convert_string(holder: &AbstractWriteHolder, value: &str) -> String {
        holder
            .converter_map()
            .convert_to_excel_data(
                &value.to_owned(),
                &crate::core::ExcelColumn::new("value", "Value", Some(0), 0, None),
                &crate::core::ConvertContext {
                    sheet_name: "Data".to_owned(),
                    row_index: 1,
                    column_index: Some(0),
                    field: "value",
                    format: None,
                    date_time_format: None,
                    number_format: None,
                    use_1904_windowing: false,
                },
            )
            .expect("converter succeeds")
            .expect("converter registered")
            .value()
            .as_text()
    }

    #[test]
    fn java_root_defaults_and_parent_inheritance_are_resolved() {
        let root = AbstractWriteHolder::from_parameter(&WriteBasicParameter::default(), None);
        assert!(root.need_head);
        assert!(root.use_default_style);
        assert!(root.automatic_merge_head);
        assert_eq!(root.relative_head_row_index, 0);
        assert!(!root.order_by_include_column);

        let parent = AbstractWriteHolder::from_parameter(
            &WriteBasicParameter {
                need_head: Some(false),
                include_column_indexes: Some(vec![1, 3]),
                exclude_column_field_names: Some(vec!["secret".to_owned()]),
                order_by_include_column: Some(true),
                ..WriteBasicParameter::default()
            },
            None,
        );
        let child =
            AbstractWriteHolder::from_parameter(&WriteBasicParameter::default(), Some(&parent));
        assert!(!child.need_head);
        assert_eq!(child.include_column_indexes, parent.include_column_indexes);
        assert_eq!(
            child.exclude_column_field_names,
            parent.exclude_column_field_names
        );
        assert!(child.order_by_include_column);
    }

    #[test]
    fn explicit_empty_collection_clears_parent_and_ignore_matches_java() {
        let parent = AbstractWriteHolder::from_parameter(
            &WriteBasicParameter {
                include_column_indexes: Some(vec![1, 3]),
                include_column_field_names: Some(vec!["name".to_owned(), "age".to_owned()]),
                exclude_column_field_names: Some(vec!["age".to_owned()]),
                ..WriteBasicParameter::default()
            },
            None,
        );
        assert!(!parent.ignore(Some("name"), Some(1)));
        assert!(parent.ignore(Some("other"), Some(1)));
        assert!(parent.ignore(Some("age"), Some(1)));
        assert!(parent.ignore(Some("name"), Some(2)));

        let child = AbstractWriteHolder::from_parameter(
            &WriteBasicParameter {
                include_column_indexes: Some(Vec::new()),
                include_column_field_names: Some(Vec::new()),
                exclude_column_field_names: Some(Vec::new()),
                ..WriteBasicParameter::default()
            },
            Some(&parent),
        );
        assert!(child.ignore(Some("name"), Some(1)));
        assert_eq!(child.include_column_indexes, Some(HashSet::new()));
        assert_eq!(child.exclude_column_field_names, Some(HashSet::new()));
    }

    #[test]
    fn converter_map_clones_parent_and_applies_child_override() {
        let mut parent_parameter = WriteBasicParameter::default();
        parent_parameter
            .converters
            .register::<String, _>(PrefixConverter("parent"));
        let parent = AbstractWriteHolder::from_parameter(&parent_parameter, None);

        let inherited =
            AbstractWriteHolder::from_parameter(&WriteBasicParameter::default(), Some(&parent));
        assert_eq!(convert_string(&inherited, "value"), "parent:value");

        let mut child_parameter = WriteBasicParameter::default();
        child_parameter
            .converters
            .register::<String, _>(PrefixConverter("child"));
        let child = AbstractWriteHolder::from_parameter(&child_parameter, Some(&parent));
        assert_eq!(convert_string(&child, "value"), "child:value");
        assert_eq!(convert_string(&parent, "value"), "parent:value");
    }

    #[test]
    fn holder_exposes_real_head_property_and_complete_selection_surface() {
        let mut holder = AbstractWriteHolder::from_parameter(
            &WriteBasicParameter {
                include_column_indexes: Some(vec![2, 4]),
                include_column_field_names: Some(vec!["name".to_owned()]),
                exclude_column_indexes: Some(vec![7]),
                exclude_column_field_names: Some(vec!["secret".to_owned()]),
                order_by_include_column: Some(true),
                ..WriteBasicParameter::default()
            },
            None,
        );
        holder.resolve_head(
            Some("DemoData".to_owned()),
            Some(vec![vec!["用户".to_owned(), "姓名".to_owned()]]),
            ExcelWriteMetadata::new().head_row_height(26),
        );

        let contract: &dyn WriteHolder = &holder;
        assert_eq!(
            contract.excel_write_head_property().head_clazz(),
            Some("DemoData")
        );
        assert_eq!(contract.excel_write_head_property().head_row_number(), 2);
        assert_eq!(
            contract
                .excel_write_head_property()
                .head_row_height_property()
                .map(crate::metadata::RowHeightProperty::height),
            Some(26)
        );
        assert!(contract.order_by_include_column());
        assert_eq!(
            contract.include_column_indexes(),
            Some(&HashSet::from([2, 4]))
        );
        assert_eq!(
            contract.include_column_field_names(),
            Some(&HashSet::from(["name".to_owned()]))
        );
        assert_eq!(contract.exclude_column_indexes(), Some(&HashSet::from([7])));
        assert_eq!(
            contract.exclude_column_field_names(),
            Some(&HashSet::from(["secret".to_owned()]))
        );
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    use crate::core::ExcelWriteMetadata;
    use crate::write::holder::write_holder::WriteHolder;

    #[test]
    fn resolve_head_and_write_holder_getters_are_covered() {
        let mut holder = AbstractWriteHolder::default();
        holder.resolve_head(
            None,
            Some(vec![vec!["Name".to_owned()]]),
            ExcelWriteMetadata::default(),
        );
        assert!(holder.need_head());
        assert_eq!(holder.relative_head_row_index(), 0);
        assert!(holder.automatic_merge_head());
        assert!(!holder.order_by_include_column());
        // None field-name / None column-index arms of WriteHolder::ignore.
        assert!(!holder.ignore(None, None));
        let excluded = AbstractWriteHolder {
            exclude_column_indexes: Some(HashSet::from([7])),
            ..AbstractWriteHolder::default()
        };
        assert!(excluded.ignore(None, Some(7)));
    }
}
