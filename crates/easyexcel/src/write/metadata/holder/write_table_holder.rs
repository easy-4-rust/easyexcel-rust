//! 对应 Java：`com.alibaba.excel.write.metadata.holder.WriteTableHolder`.

use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use crate::write::holder::abstract_write_holder::AbstractWriteHolder;
use crate::write::metadata::holder::write_holder::delegate_write_holder_contract;
use crate::write::metadata::WriteBasicParameter;
use crate::{HolderEnum, WriteTable};

/// 对应 Java：`WriteTableHolder extends AbstractWriteHolder`.
///
/// Java's holder carries a POI `Sheet` plus a `tableNo` field. The Rust port
/// mirrors the type so `ExcelWriterTableBuilder` can return a
/// `WriteTableHolder` for parity. Runtime callbacks expose the active table
/// through [`crate::core::WriteTableHolderView`].
pub struct WriteTableHolder<'a> {
    abstract_holder: AbstractWriteHolder,
    table_no: i32,
    parent_sheet: Option<String>,
    parent_write_sheet_holder_id: Option<usize>,
    write_table: WriteTable,
    last_row_index: i32,
    /// 保留既有公开类型参数，但不借用父 Holder，避免 Table 无法插回父 Sheet。
    lifetime_marker: PhantomData<&'a ()>,
}

impl<'a> WriteTableHolder<'a> {
    /// 对应 Java：com.alibaba.excel.write.metadata.holder.WriteTableHolder。 Creates a table holder matching the Java `WriteTableHolder(WriteTable, WriteSheetHolder)` initialiser.
    #[must_use]
    pub fn new(table_no: i32) -> Self {
        let mut abstract_holder = AbstractWriteHolder::default();
        abstract_holder.abstract_holder_mut().holder_type = HolderEnum::Table;
        Self {
            abstract_holder,
            table_no,
            parent_sheet: None,
            parent_write_sheet_holder_id: None,
            write_table: WriteTable::with_table_no(table_no),
            last_row_index: 0,
            lifetime_marker: PhantomData,
        }
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.holder.WriteTableHolder。 Creates a table holder and resolves nullable values against its sheet.
    #[must_use]
    pub fn from_parameter(
        table_no: i32,
        parameter: &WriteBasicParameter,
        parent: &AbstractWriteHolder,
    ) -> Self {
        let mut holder = Self::new(table_no);
        holder.abstract_holder = AbstractWriteHolder::from_parameter(parameter, Some(parent));
        holder.abstract_holder.abstract_holder_mut().holder_type = HolderEnum::Table;
        holder
    }

    /// Returns the inherited write-holder state.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.holder.WriteTableHolder。
    pub const fn abstract_holder(&self) -> &AbstractWriteHolder {
        &self.abstract_holder
    }

    /// Returns mutable inherited write-holder state.
    /// 对应 Java：com.alibaba.excel.write.metadata.holder.WriteTableHolder。
    pub const fn abstract_holder_mut(&mut self) -> &mut AbstractWriteHolder {
        &mut self.abstract_holder
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.holder.WriteTableHolder。 Returns the parent sheet name, if any. (Java `getParentWriteSheetHolder().getSheetName()`)
    #[must_use]
    pub fn parent_sheet(&self) -> Option<&str> {
        self.parent_sheet.as_deref()
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.holder.WriteTableHolder。 Sets the parent sheet name.
    pub fn set_parent_sheet(&mut self, parent: impl Into<String>) {
        self.parent_sheet = Some(parent.into());
    }

    /// Returns the zero-based table index. (Java `getTableNo()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.holder.WriteTableHolder。
    pub const fn table_no(&self) -> i32 {
        self.table_no
    }

    /// Returns the last row index. (Java `getLastRowIndex()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.holder.WriteTableHolder。
    pub const fn last_row_index(&self) -> i32 {
        self.last_row_index
    }

    /// 使用完整 `WriteTable` 和父 Sheet Holder 创建 Holder。
    ///
    /// 对应 Java：`WriteTableHolder(WriteTable, WriteSheetHolder)`。
    #[must_use]
    pub fn from_write_table(
        write_table: WriteTable,
        parent: &super::write_sheet_holder::WriteSheetHolder<'_>,
    ) -> Self {
        let mut holder = Self::from_parameter(
            write_table.table_no,
            &write_table.parameter,
            parent.abstract_holder(),
        );
        holder.parent_sheet = Some(parent.sheet_name().to_owned());
        holder.parent_write_sheet_holder_id = Some(std::ptr::from_ref(parent).addr());
        holder.write_table = write_table;
        holder
    }

    /// 由拆分后的父状态创建 Holder，供不持有具体 Sheet Holder 的 Rust 内部路径使用。
    #[must_use]
    pub fn from_write_table_parts(
        write_table: WriteTable,
        parent_sheet: Option<&str>,
        parent: &AbstractWriteHolder,
    ) -> Self {
        let mut holder = Self::from_parameter(write_table.table_no, &write_table.parameter, parent);
        holder.parent_sheet = parent_sheet.map(str::to_owned);
        holder.write_table = write_table;
        holder
    }
    /// Java `getTableNo`。
    #[must_use] pub const fn get_table_no(&self) -> i32 { self.table_no }
    /// Java `setTableNo`。
    pub const fn set_table_no(&mut self, value: i32) { self.table_no = value; }
    /// Java `getWriteTable`。
    #[must_use] pub const fn get_write_table(&self) -> &WriteTable { &self.write_table }
    /// Java `setWriteTable`。
    pub fn set_write_table(&mut self, value: WriteTable) { self.table_no = value.table_no; self.write_table = value; }
    /// Java `getParentWriteSheetHolder` 的构造时身份映射。
    #[must_use] pub const fn get_parent_write_sheet_holder_id(&self) -> Option<usize> { self.parent_write_sheet_holder_id }
    /// Java 命名兼容入口；Rust 使用构造时身份令牌而不是自引用。
    #[must_use] pub const fn get_parent_write_sheet_holder(&self) -> Option<usize> { self.parent_write_sheet_holder_id }
    /// Java `setParentWriteSheetHolder` 的构造时身份映射。
    pub const fn set_parent_write_sheet_holder_id(&mut self, value: Option<usize>) { self.parent_write_sheet_holder_id = value; }
    /// Java 命名兼容入口；Rust 使用构造时身份令牌而不是自引用。
    pub const fn set_parent_write_sheet_holder(&mut self, value: Option<usize>) { self.parent_write_sheet_holder_id = value; }
    /// Java `holderType`。
    #[must_use] pub const fn holder_type(&self) -> HolderEnum { HolderEnum::Table }
}

impl Deref for WriteTableHolder<'_> {
    type Target = AbstractWriteHolder;

    fn deref(&self) -> &Self::Target {
        &self.abstract_holder
    }
}

impl DerefMut for WriteTableHolder<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.abstract_holder
    }
}

delegate_write_holder_contract!(WriteTableHolder<'a>, abstract_holder);

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::write::WriteHolder;
    use crate::write::holder::write_sheet_holder::WriteSheetHolder;
    use crate::write::holder::write_workbook_holder::WriteWorkbookHolder;

    #[test]
    fn workbook_sheet_table_holders_resolve_java_parent_chain() {
        let workbook = WriteWorkbookHolder::from_parameter(
            "out.xlsx",
            &WriteBasicParameter {
                need_head: Some(false),
                include_column_indexes: Some(vec![1, 3]),
                ..WriteBasicParameter::default()
            },
        );
        let sheet = WriteSheetHolder::from_parameter(
            "Data",
            0,
            &WriteBasicParameter {
                need_head: Some(true),
                exclude_column_field_names: Some(vec!["secret".to_owned()]),
                ..WriteBasicParameter::default()
            },
            workbook.abstract_holder(),
        );
        let table = WriteTableHolder::from_parameter(
            2,
            &WriteBasicParameter {
                include_column_indexes: Some(Vec::new()),
                order_by_include_column: Some(true),
                ..WriteBasicParameter::default()
            },
            sheet.abstract_holder(),
        );

        assert!(!workbook.need_head());
        assert!(sheet.need_head());
        assert!(table.need_head());
        assert_eq!(sheet.include_column_indexes, Some(HashSet::from([1, 3])));
        assert_eq!(table.include_column_indexes, Some(HashSet::new()));
        assert_eq!(
            table.exclude_column_field_names,
            Some(HashSet::from(["secret".to_owned()]))
        );
        assert!(table.order_by_include_column());
    }

    #[test]
    fn write_table_holder_new() {
        let holder = WriteTableHolder::new(0);
        assert_eq!(holder.table_no(), 0);
        assert_eq!(holder.last_row_index(), 0);
        assert!(holder.parent_sheet().is_none());
    }

    #[test]
    fn write_table_holder_from_parameter() {
        let parent = AbstractWriteHolder::default();
        let param = WriteBasicParameter::default();
        let holder = WriteTableHolder::from_parameter(3, &param, &parent);
        assert_eq!(holder.table_no(), 3);
    }

    #[test]
    fn write_table_holder_abstract_holder_accessors() {
        let mut holder = WriteTableHolder::new(0);
        let _ = holder.abstract_holder();
        let _ = holder.abstract_holder_mut();
    }

    #[test]
    fn write_table_holder_set_parent_sheet() {
        let mut holder = WriteTableHolder::new(0);
        holder.set_parent_sheet("Sheet1");
        assert_eq!(holder.parent_sheet(), Some("Sheet1"));
    }

    #[test]
    fn write_table_holder_deref() {
        let holder = WriteTableHolder::new(0);
        let _ = holder.abstract_holder();
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    use crate::core::ExcelWriteHeadProperty;

    #[test]
    fn table_holder_deref_mut_reaches_abstract_holder() {
        use crate::write::holder::write_holder::WriteHolder;
        let mut holder = WriteTableHolder::new(0);
        holder.set_excel_write_head_property(ExcelWriteHeadProperty::new());
        let target: &mut crate::write::holder::abstract_write_holder::AbstractWriteHolder =
            &mut holder;
        assert!(target.need_head());
    }
}
