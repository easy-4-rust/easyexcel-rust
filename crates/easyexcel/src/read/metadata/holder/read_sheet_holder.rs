//! 对应 Java：`com.alibaba.excel.read.metadata.holder.ReadSheetHolder`.

use std::collections::HashMap;

use crate::{CellExtra, CellValue, HolderEnum, ReadCellData, ReadSheet};

use super::read_workbook_holder::ReadWorkbookHolder;
use super::abstract_read_holder::AbstractReadHolder;
use std::ops::{Deref, DerefMut};

/// 对应 Java：`ReadSheetHolder extends AbstractReadHolder`.
#[derive(Debug, Clone)]
pub struct ReadSheetHolder {
    abstract_holder: AbstractReadHolder,
    /// Mirrors `ReadSheetHolder.sheetNo`.
    pub sheet_no: i32,
    /// Mirrors `ReadSheetHolder.sheetName`.
    pub sheet_name: String,
    /// Mirrors `ReadSheetHolder.rowIndex`.
    pub row_index: i32,
    /// Mirrors `ReadSheetHolder.ended`.
    pub ended: bool,
    approximate_total_row_number: Option<i32>,
    max_not_empty_data_head_size: Option<i32>,
    cell_map: HashMap<usize, CellValue>,
    cell_extra: Option<CellExtra>,
    temp_cell_data: Option<ReadCellData>,
    read_sheet: ReadSheet,
    parent_read_workbook_holder: Option<Box<ReadWorkbookHolder>>,
}

impl ReadSheetHolder {
    /// 对应 Java：`ReadSheetHolder(ReadSheet, ReadWorkbookHolder)`.
    pub fn new(sheet_no: i32, sheet_name: impl Into<String>) -> Self {
        let sheet_name = sheet_name.into();
        Self {
            abstract_holder: AbstractReadHolder::from_parameter(
                &crate::read::metadata::ReadBasicParameter::default(),
                None,
                HolderEnum::Sheet,
            ),
            sheet_no,
            read_sheet: if sheet_no >= 0 {
                ReadSheet::with_name(sheet_no as usize, &sheet_name)
            } else {
                ReadSheet::named(&sheet_name)
            },
            sheet_name,
            row_index: -1,
            ended: false,
            approximate_total_row_number: None,
            max_not_empty_data_head_size: None,
            cell_map: HashMap::new(),
            cell_extra: None,
            temp_cell_data: None,
            parent_read_workbook_holder: None,
        }
    }

    /// Java `ReadSheetHolder(ReadSheet, ReadWorkbookHolder)` 完整构造器。
    #[must_use]
    pub fn from_read_sheet(read_sheet: ReadSheet, read_workbook_holder: ReadWorkbookHolder) -> Self {
        let sheet_no = read_sheet.get_sheet_no().unwrap_or(-1);
        let sheet_name = read_sheet.get_sheet_name().to_owned();
        let abstract_holder = AbstractReadHolder::from_parameter(
            read_sheet.get_read_basic_parameter(),
            Some(read_workbook_holder.abstract_holder()),
            HolderEnum::Sheet,
        );
        Self {
            abstract_holder,
            sheet_no,
            sheet_name,
            row_index: -1,
            ended: false,
            approximate_total_row_number: None,
            max_not_empty_data_head_size: None,
            cell_map: HashMap::new(),
            cell_extra: None,
            temp_cell_data: None,
            read_sheet,
            parent_read_workbook_holder: Some(Box::new(read_workbook_holder)),
        }
    }

    /// Java 无参构造器。
    #[must_use]
    pub fn default_construction() -> Self { Self::new(-1, "") }

    #[must_use] pub const fn get_sheet_no(&self) -> i32 { self.sheet_no }
    pub fn set_sheet_no(&mut self, value: i32) { self.sheet_no = value; }
    #[must_use] pub fn get_sheet_name(&self) -> &str { &self.sheet_name }
    pub fn set_sheet_name(&mut self, value: impl Into<String>) { self.sheet_name = value.into(); }
    #[must_use] pub const fn get_row_index(&self) -> i32 { self.row_index }
    pub const fn set_row_index(&mut self, value: i32) { self.row_index = value; }
    #[must_use] pub const fn get_ended(&self) -> bool { self.ended }
    pub const fn set_ended(&mut self, value: bool) { self.ended = value; }
    #[must_use] pub const fn get_approximate_total_row_number(&self) -> Option<i32> {
        self.approximate_total_row_number
    }
    pub const fn set_approximate_total_row_number(&mut self, value: Option<i32>) {
        self.approximate_total_row_number = value;
    }
    #[must_use] pub const fn get_total(&self) -> Option<i32> { self.approximate_total_row_number }
    pub const fn set_total(&mut self, value: Option<i32>) { self.approximate_total_row_number = value; }
    #[must_use] pub const fn get_max_not_empty_data_head_size(&self) -> Option<i32> {
        self.max_not_empty_data_head_size
    }
    pub const fn set_max_not_empty_data_head_size(&mut self, value: Option<i32>) {
        self.max_not_empty_data_head_size = value;
    }
    #[must_use] pub const fn get_cell_map(&self) -> &HashMap<usize, CellValue> { &self.cell_map }
    pub fn set_cell_map(&mut self, value: HashMap<usize, CellValue>) { self.cell_map = value; }
    #[must_use] pub const fn get_cell_extra(&self) -> Option<&CellExtra> { self.cell_extra.as_ref() }
    pub fn set_cell_extra(&mut self, value: Option<CellExtra>) { self.cell_extra = value; }
    #[must_use] pub const fn get_temp_cell_data(&self) -> Option<&ReadCellData> {
        self.temp_cell_data.as_ref()
    }
    pub fn set_temp_cell_data(&mut self, value: Option<ReadCellData>) { self.temp_cell_data = value; }
    #[must_use] pub const fn get_read_sheet(&self) -> &ReadSheet { &self.read_sheet }
    pub fn set_read_sheet(&mut self, value: ReadSheet) { self.read_sheet = value; }
    #[must_use] pub const fn get_parent_read_workbook_holder(&self) -> Option<&ReadWorkbookHolder> {
        self.parent_read_workbook_holder.as_deref()
    }
    pub fn set_parent_read_workbook_holder(&mut self, value: Option<ReadWorkbookHolder>) {
        self.parent_read_workbook_holder = value.map(Box::new);
    }
    #[must_use] pub const fn holder_type(&self) -> HolderEnum { HolderEnum::Sheet }
    /// 返回父类读取 Holder。
    #[must_use] pub const fn abstract_holder(&self) -> &AbstractReadHolder { &self.abstract_holder }
    /// 返回可变父类读取 Holder。
    pub const fn abstract_holder_mut(&mut self) -> &mut AbstractReadHolder { &mut self.abstract_holder }
}

impl Deref for ReadSheetHolder {
    type Target = AbstractReadHolder;
    fn deref(&self) -> &Self::Target { &self.abstract_holder }
}

impl DerefMut for ReadSheetHolder {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.abstract_holder }
}
