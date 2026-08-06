/// 对应 Java：无直接对应对象；Rust 架构扩展。 Fully resolved Java `WriteHolder` state independent of a concrete backend.
#[derive(Debug, Clone, PartialEq)]
pub struct WriteContextHolderState {
    /// Active holder level.
    pub holder_type: Holder,
    /// Resolved header metadata.
    pub excel_write_head_property: ExcelWriteHeadProperty,
    /// Effective workbook/sheet/table converter map.
    pub converter_map: ConverterRegistry,
    /// Whether a header is written.
    pub need_head: bool,
    /// Whether automatic header merging is enabled.
    pub automatic_merge_head: bool,
    /// Relative header row offset.
    pub relative_head_row_index: i32,
    /// Whether include-list order controls output.
    pub order_by_include_column: bool,
    /// Included physical columns.
    pub include_column_indexes: Option<Vec<usize>>,
    /// Included field names.
    pub include_column_field_names: Option<Vec<String>>,
    /// Excluded physical columns.
    pub exclude_column_indexes: Vec<usize>,
    /// Excluded field names.
    pub exclude_column_field_names: Vec<String>,
}

impl Default for WriteContextHolderState {
    fn default() -> Self {
        Self {
            holder_type: Holder::Workbook,
            excel_write_head_property: ExcelWriteHeadProperty::new(),
            converter_map: ConverterRegistry::default(),
            need_head: true,
            automatic_merge_head: true,
            relative_head_row_index: 0,
            order_by_include_column: false,
            include_column_indexes: None,
            include_column_field_names: None,
            exclude_column_indexes: Vec::new(),
            exclude_column_field_names: Vec::new(),
        }
    }
}

impl WriteContextHolderState {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Clones the backend-neutral state exposed by a live Java-style holder.
    #[must_use]
    pub fn from_holder(holder: &dyn WriteContextHolder) -> Self {
        Self {
            holder_type: holder.holder_type(),
            excel_write_head_property: holder.excel_write_head_property().clone(),
            converter_map: holder.converter_map().clone(),
            need_head: holder.need_head(),
            automatic_merge_head: holder.automatic_merge_head(),
            relative_head_row_index: holder.relative_head_row_index(),
            order_by_include_column: holder.order_by_include_column(),
            include_column_indexes: holder.include_column_indexes().map(<[usize]>::to_vec),
            include_column_field_names: holder.include_column_field_names().map(<[String]>::to_vec),
            exclude_column_indexes: holder.exclude_column_indexes().to_vec(),
            exclude_column_field_names: holder.exclude_column_field_names().to_vec(),
        }
    }
}

