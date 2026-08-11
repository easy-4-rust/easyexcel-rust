//! 对应 Java：`com.alibaba.excel.write.metadata.holder.WriteSheetHolder`.

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use crate::write::holder::abstract_write_holder::AbstractWriteHolder;
use crate::write::metadata::holder::write_holder::delegate_write_holder_contract;
use crate::write::holder::write_table_holder::WriteTableHolder;
use crate::write::metadata::WriteBasicParameter;
use crate::write::metadata::WriteSheet;
use crate::{HolderEnum, WriteLastRowTypeEnum};

/// 对应 Java：`WriteSheetHolder extends AbstractWriteHolder`.
///
/// Java's holder stores a POI `Sheet` instance plus the in-flight row
/// cursors. The Rust port reuses [`crate::ExcelWriter`] for the live
/// `rust_xlsxwriter::Worksheet`; this owned builder-side mirror remains for
/// Java package/API parity. Runtime callbacks use
/// [`crate::core::WriteSheetHolderView`] instead of a fake POI sheet.
pub struct WriteSheetHolder<'a> {
    abstract_holder: AbstractWriteHolder,
    write_sheet: WriteSheet,
    /// 实际后端 Sheet 由 writer 所有；此处保存当前与缓存 Sheet 的末行状态。
    sheet_last_row_index: i32,
    cached_sheet_last_row_index: i32,
    cached_sheet_has_row_zero: bool,
    sheet_name: String,
    sheet_no: i32,
    parent_write_workbook_holder_id: Option<usize>,
    tables: HashMap<i32, WriteTableHolder<'a>>,
    write_last_row_type_enum: WriteLastRowTypeEnum,
    last_row_index: i32,
    has_data: bool,
}

impl<'a> WriteSheetHolder<'a> {
    /// Java `getSheetName`。
    #[must_use] pub fn get_sheet_name(&self) -> &str { &self.sheet_name }
    /// Java `setSheetName`。
    pub fn set_sheet_name(&mut self, value: impl Into<String>) { self.sheet_name = value.into(); }
    /// Java `getSheetNo`。
    #[must_use] pub const fn get_sheet_no(&self) -> i32 { self.sheet_no }
    /// Java `setSheetNo`。
    pub const fn set_sheet_no(&mut self, value: i32) { self.sheet_no = value; }
    /// Java `getHasBeenInitializedTable`。
    #[must_use] pub fn get_has_been_initialized_table(&self) -> &HashMap<i32, WriteTableHolder<'a>> { &self.tables }
    /// Java `setHasBeenInitializedTable`。
    pub fn set_has_been_initialized_table(&mut self, value: HashMap<i32, WriteTableHolder<'a>>) { self.tables = value; }
    /// Java `getLastRowIndex`。
    #[must_use] pub const fn get_last_row_index(&self) -> i32 { self.last_row_index }
    /// Java `setLastRowIndex`。
    pub const fn set_last_row_index(&mut self, value: i32) { self.last_row_index = value; }
    /// Java `getHasData`。
    #[must_use] pub const fn get_has_data(&self) -> bool { self.has_data }
    /// Java `setHasData`。
    pub const fn set_has_data(&mut self, value: bool) { self.has_data = value; }

    /// 对应 Java：com.alibaba.excel.write.metadata.holder.WriteSheetHolder。 Creates a sheet holder matching the Java `WriteSheetHolder(WriteSheet, WriteWorkbookHolder)` initialiser.
    #[must_use]
    pub fn new(sheet_name: impl Into<String>, sheet_no: i32) -> Self {
        let mut abstract_holder = AbstractWriteHolder::default();
        abstract_holder.abstract_holder_mut().holder_type = HolderEnum::Sheet;
        Self {
            abstract_holder,
            write_sheet: WriteSheet::with_sheet(sheet_no, sheet_name.into()),
            sheet_last_row_index: 0,
            cached_sheet_last_row_index: 0,
            cached_sheet_has_row_zero: false,
            sheet_name: String::new(),
            sheet_no,
            parent_write_workbook_holder_id: None,
            tables: HashMap::new(),
            write_last_row_type_enum: WriteLastRowTypeEnum::CommonEmpty,
            last_row_index: 0,
            has_data: false,
        }.synchronize_sheet_name()
    }

    /// Java 无参构造器；真实 `WriteSheet` 可在 Holder 初始化阶段再设置。
    #[must_use]
    pub fn default_construction() -> Self {
        Self::new("", 0)
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.holder.WriteSheetHolder。 Creates a sheet holder and resolves nullable values against its parent.
    #[must_use]
    pub fn from_parameter(
        sheet_name: impl Into<String>,
        sheet_no: i32,
        parameter: &WriteBasicParameter,
        parent: &AbstractWriteHolder,
    ) -> Self {
        let mut holder = Self::new(sheet_name, sheet_no);
        holder.abstract_holder = AbstractWriteHolder::from_parameter(parameter, Some(parent));
        holder.abstract_holder.abstract_holder_mut().holder_type = HolderEnum::Sheet;
        holder
    }

    /// Returns the inherited write-holder state.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.holder.WriteSheetHolder。
    pub const fn abstract_holder(&self) -> &AbstractWriteHolder {
        &self.abstract_holder
    }

    /// Returns mutable inherited write-holder state.
    /// 对应 Java：com.alibaba.excel.write.metadata.holder.WriteSheetHolder。
    pub const fn abstract_holder_mut(&mut self) -> &mut AbstractWriteHolder {
        &mut self.abstract_holder
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.holder.WriteSheetHolder。 Returns the sheet name. (Java `getSheetName()`)
    #[must_use]
    pub fn sheet_name(&self) -> &str {
        &self.sheet_name
    }

    /// Returns the zero-based sheet index. (Java `getSheetNo()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.holder.WriteSheetHolder。
    pub const fn sheet_no(&self) -> i32 {
        self.sheet_no
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.holder.WriteSheetHolder。 Returns the per-table holders. (Java `getHasBeenInitializedTable()`)
    #[must_use]
    pub fn tables(&self) -> &HashMap<i32, WriteTableHolder<'a>> {
        &self.tables
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.holder.WriteSheetHolder。 Returns a mutable handle on the per-table holders.
    pub fn tables_mut(&mut self) -> &mut HashMap<i32, WriteTableHolder<'a>> {
        &mut self.tables
    }

    /// Returns the last row index. (Java `getLastRowIndex()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.holder.WriteSheetHolder。
    pub const fn last_row_index(&self) -> i32 {
        self.last_row_index
    }

    /// Returns whether at least one row has been written. (Java `getHasData()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.holder.WriteSheetHolder。
    pub const fn has_data(&self) -> bool {
        self.has_data
    }

    /// 对应 Java：com.alibaba.excel.write.metadata.holder.WriteSheetHolder。 Records the next row index. (Java `getNewRowIndexAndStartDoWrite()` step)
    pub fn advance_row(&mut self) -> i32 {
        self.get_new_row_index_and_start_do_write()
    }

    fn synchronize_sheet_name(mut self) -> Self {
        self.sheet_name = self.write_sheet.sheet_name.clone();
        self
    }

    /// 使用完整 `WriteSheet` 和父工作簿 Holder 创建 Holder。
    ///
    /// 对应 Java：`WriteSheetHolder(WriteSheet, WriteWorkbookHolder)`。Sheet
    /// 参数继承父 Holder 的 converter、Handler 与列过滤配置，模板状态由父
    /// Holder 的临时模板输入决定；父对象以构造时身份令牌记录，避免 Rust 自引用。
    #[must_use]
    pub fn from_write_sheet(
        write_sheet: WriteSheet,
        parent: &super::write_workbook_holder::WriteWorkbookHolder<'_>,
    ) -> Self {
        let template_present = parent.get_temp_template_input_stream().is_some();
        let mut holder = Self::new(write_sheet.sheet_name.clone(), write_sheet.sheet_no);
        holder.abstract_holder = AbstractWriteHolder::from_parameter(
            &write_sheet.parameter,
            Some(parent.abstract_holder()),
        );
        holder.abstract_holder.abstract_holder_mut().holder_type = HolderEnum::Sheet;
        holder.parent_write_workbook_holder_id = Some(std::ptr::from_ref(parent).addr());
        holder.write_sheet = write_sheet;
        holder.write_last_row_type_enum = if template_present {
            WriteLastRowTypeEnum::TemplateEmpty
        } else {
            WriteLastRowTypeEnum::CommonEmpty
        };
        holder
    }

    /// 无父 Holder 时按显式模板状态创建 Sheet Holder，供 Rust 内部预构建使用。
    #[must_use]
    pub fn from_write_sheet_with_template_state(
        write_sheet: WriteSheet,
        template_present: bool,
    ) -> Self {
        let mut holder = Self::new(write_sheet.sheet_name.clone(), write_sheet.sheet_no);
        holder.abstract_holder = AbstractWriteHolder::from_parameter(&write_sheet.parameter, None);
        holder.abstract_holder.abstract_holder_mut().holder_type = HolderEnum::Sheet;
        holder.write_sheet = write_sheet;
        holder.write_last_row_type_enum = if template_present {
            WriteLastRowTypeEnum::TemplateEmpty
        } else {
            WriteLastRowTypeEnum::CommonEmpty
        };
        holder
    }

    /// Java `getWriteSheet`。
    #[must_use] pub const fn get_write_sheet(&self) -> &WriteSheet { &self.write_sheet }
    /// Java `setWriteSheet`。
    pub fn set_write_sheet(&mut self, value: WriteSheet) {
        self.sheet_no = value.sheet_no;
        self.sheet_name.clone_from(&value.sheet_name);
        self.write_sheet = value;
    }
    /// Java `getParentWriteWorkbookHolder` 的构造时身份映射。
    #[must_use] pub const fn get_parent_write_workbook_holder_id(&self) -> Option<usize> { self.parent_write_workbook_holder_id }
    /// Java 命名兼容入口；Rust 使用构造时身份令牌而不是自引用。
    #[must_use] pub const fn get_parent_write_workbook_holder(&self) -> Option<usize> { self.parent_write_workbook_holder_id }
    /// Java `setParentWriteWorkbookHolder` 的构造时身份映射。
    pub const fn set_parent_write_workbook_holder_id(&mut self, value: Option<usize>) { self.parent_write_workbook_holder_id = value; }
    /// Java 命名兼容入口；Rust 使用构造时身份令牌而不是自引用。
    pub const fn set_parent_write_workbook_holder(&mut self, value: Option<usize>) { self.parent_write_workbook_holder_id = value; }
    /// Java `getWriteLastRowTypeEnum`。
    #[must_use] pub const fn get_write_last_row_type_enum(&self) -> WriteLastRowTypeEnum { self.write_last_row_type_enum }
    /// Java `setWriteLastRowTypeEnum`。
    pub const fn set_write_last_row_type_enum(&mut self, value: WriteLastRowTypeEnum) { self.write_last_row_type_enum = value; }
    /// 更新实际 Sheet 与缓存 Sheet 的末行状态，供模板追加算法使用。
    pub const fn set_backend_row_state(&mut self, sheet_last_row_index: i32, cached_sheet_last_row_index: i32, cached_sheet_has_row_zero: bool) {
        self.sheet_last_row_index = sheet_last_row_index;
        self.cached_sheet_last_row_index = cached_sheet_last_row_index;
        self.cached_sheet_has_row_zero = cached_sheet_has_row_zero;
    }
    /// 返回当前 Sheet 元数据。对应 Java Lombok `getSheet()`。
    #[must_use] pub const fn get_sheet(&self) -> &WriteSheet { &self.write_sheet }
    /// 替换当前 Sheet 元数据。对应 Java Lombok `setSheet()`。
    pub fn set_sheet(&mut self, value: WriteSheet) { self.set_write_sheet(value); }
    /// 返回缓存 Sheet 的最后行状态。对应 Java `getCachedSheet()` 的后端中立映射。
    #[must_use] pub const fn get_cached_sheet(&self) -> (i32, bool) {
        (self.cached_sheet_last_row_index, self.cached_sheet_has_row_zero)
    }
    /// 替换缓存 Sheet 的最后行状态。对应 Java `setCachedSheet()`。
    pub const fn set_cached_sheet(&mut self, last_row_index: i32, has_row_zero: bool) {
        self.cached_sheet_last_row_index = last_row_index;
        self.cached_sheet_has_row_zero = has_row_zero;
    }
    /// Java `getNewRowIndexAndStartDoWrite`，完整复现三态游标推进。
    pub fn get_new_row_index_and_start_do_write(&mut self) -> i32 {
        let new_row_index = match self.write_last_row_type_enum {
            WriteLastRowTypeEnum::TemplateEmpty => {
                let last = self.sheet_last_row_index.max(self.cached_sheet_last_row_index);
                if last != 0 || self.cached_sheet_has_row_zero { last + 1 } else { 0 }
            }
            WriteLastRowTypeEnum::HasData => self.sheet_last_row_index.max(self.cached_sheet_last_row_index) + 1,
            WriteLastRowTypeEnum::CommonEmpty => 0,
        };
        self.write_last_row_type_enum = WriteLastRowTypeEnum::HasData;
        self.has_data = true;
        self.last_row_index = new_row_index;
        self.sheet_last_row_index = new_row_index;
        new_row_index
    }
    /// Java `holderType`。
    #[must_use] pub const fn holder_type(&self) -> HolderEnum { HolderEnum::Sheet }
}

impl Deref for WriteSheetHolder<'_> {
    type Target = AbstractWriteHolder;

    fn deref(&self) -> &Self::Target {
        &self.abstract_holder
    }
}

impl DerefMut for WriteSheetHolder<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.abstract_holder
    }
}

delegate_write_holder_contract!(WriteSheetHolder<'a>, abstract_holder);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::write::metadata::write_basic_parameter::WriteBasicParameter;

    #[test]
    fn write_sheet_holder_new() {
        let holder = WriteSheetHolder::new("Sheet1", 0);
        assert_eq!(holder.sheet_name(), "Sheet1");
        assert_eq!(holder.sheet_no(), 0);
        assert_eq!(holder.last_row_index(), 0);
        assert!(!holder.has_data());
        assert!(holder.tables().is_empty());
    }

    #[test]
    fn write_sheet_holder_from_parameter() {
        let parent = AbstractWriteHolder::default();
        let param = WriteBasicParameter::default();
        let holder = WriteSheetHolder::from_parameter("S1", 1, &param, &parent);
        assert_eq!(holder.sheet_name(), "S1");
        assert_eq!(holder.sheet_no(), 1);
    }

    #[test]
    fn write_sheet_holder_abstract_holder_accessors() {
        let mut holder = WriteSheetHolder::new("Sheet", 0);
        let _ = holder.abstract_holder();
        let _ = holder.abstract_holder_mut();
    }

    #[test]
    fn write_sheet_holder_tables_mut() {
        let mut holder = WriteSheetHolder::new("S", 0);
        let _ = holder.tables_mut();
    }

    #[test]
    fn write_sheet_holder_advance_row() {
        let mut holder = WriteSheetHolder::new("S", 0);
        // CommonEmpty 初始状态返回 0（第一个可写行），第二次返回 1
        assert_eq!(holder.advance_row(), 0);
        assert!(holder.has_data());
        assert_eq!(holder.last_row_index(), 0);
        assert_eq!(holder.advance_row(), 1);
    }

    #[test]
    fn write_sheet_holder_deref() {
        let holder = WriteSheetHolder::new("S", 0);
        let _ = holder.abstract_holder();
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    use crate::core::ExcelWriteHeadProperty;

    #[test]
    fn sheet_holder_deref_mut_reaches_abstract_holder() {
        let mut holder = WriteSheetHolder::new("Sheet1", 0);
        holder.set_excel_write_head_property(ExcelWriteHeadProperty::new());
        holder.advance_row();
        assert!(holder.has_data());
    }
}
