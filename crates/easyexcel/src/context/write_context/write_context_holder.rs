/// 对应 Java：无直接对应对象；Rust 架构扩展。 Holder surface exposed through [`WriteContext`].
pub trait WriteContextHolder {
    /// Returns the output path. (Java `WriteWorkbookHolder.getFile()`)
    fn path(&self) -> &Path;

    /// Returns the workbook-level handler context when available.
    /// (Java `WriteWorkbookHolder` via `WriteContextImpl.writeWorkbookHolder`)
    fn workbook_context(&self) -> Option<&WriteWorkbookContext> {
        None
    }

    /// Returns the active sheet handler context when available.
    /// (Java `WriteSheetHolder` via `WriteContextImpl.writeSheetHolder`)
    fn sheet_context(&self) -> Option<&WriteSheetContext> {
        None
    }

    /// Returns the zero-based table index when writing table content.
    /// (Java `WriteTableHolder.getTableNo()`)
    fn table_no(&self) -> Option<i32> {
        None
    }

    /// Returns the active sheet name when a sheet holder exists.
    fn sheet_name(&self) -> Option<&str> {
        self.sheet_context().map(WriteSheetContext::sheet_name)
    }

    /// Returns the resolved zero-based sheet number when known.
    fn sheet_no(&self) -> Option<i32> {
        self.sheet_context()
            .and_then(|context| context.write_sheet_holder().sheet_no())
    }

    /// Returns the latest physical row visible to the holder.
    fn last_row_index(&self) -> Option<u32> {
        self.sheet_context()
            .and_then(|context| context.write_sheet_holder().last_row_index())
    }

    /// Returns whether the active sheet has visible row data.
    fn has_data(&self) -> bool {
        self.sheet_context()
            .is_some_and(|context| context.write_sheet_holder().has_data())
    }

    /// Returns the active holder level. (Java `HolderEnum`)
    fn holder_type(&self) -> Holder;

    /// Returns the fully resolved header property.
    /// (Java `WriteHolder.excelWriteHeadProperty()`)
    fn excel_write_head_property(&self) -> &ExcelWriteHeadProperty;

    /// Returns the effective converter map for the active holder.
    /// (Java `ConfigurationHolder.converterMap()`)
    fn converter_map(&self) -> &ConverterRegistry;

    /// Returns whether this holder writes a header. (Java `needHead()`)
    fn need_head(&self) -> bool;

    /// Returns whether automatic header merging is enabled.
    fn automatic_merge_head(&self) -> bool;

    /// Returns the relative header row offset.
    fn relative_head_row_index(&self) -> i32;

    /// Returns whether include-list order controls output order.
    fn order_by_include_column(&self) -> bool;

    /// Returns included physical column indexes.
    fn include_column_indexes(&self) -> Option<&[usize]>;

    /// Returns included field names.
    fn include_column_field_names(&self) -> Option<&[String]>;

    /// Returns excluded physical column indexes.
    fn exclude_column_indexes(&self) -> &[usize];

    /// Returns excluded field names.
    fn exclude_column_field_names(&self) -> &[String];
}

