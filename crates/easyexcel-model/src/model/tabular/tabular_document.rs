use std::collections::HashSet;

use crate::styles::CellStyle;
use crate::{Cell, CellValue, Workbook};

use super::{TabularCell, TabularTable};

/// 对应 Java：无直接对应对象；Rust 架构扩展。可包含多个表格的中立文档。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TabularDocument {
    tables: Vec<TabularTable>,
}

impl TabularDocument {
    /// 创建空文档。
    #[must_use]
    pub const fn new() -> Self {
        Self { tables: Vec::new() }
    }

    /// 从表格集合创建文档。
    #[must_use]
    pub fn from_tables(tables: Vec<TabularTable>) -> Self {
        Self { tables }
    }

    /// 返回全部表格。
    #[must_use]
    pub fn tables(&self) -> &[TabularTable] {
        &self.tables
    }

    /// 追加一个表格。
    pub fn push_table(&mut self, table: TabularTable) {
        self.tables.push(table);
    }

    /// 将所有表格映射为工作簿中的独立工作表。
    #[must_use]
    pub fn to_workbook(&self) -> Workbook {
        self.to_workbook_with_header_style(true)
    }

    /// 将所有表格映射为工作簿，并按需应用统一粗体表头样式。
    #[must_use]
    pub fn to_workbook_with_header_style(&self, apply_header_style: bool) -> Workbook {
        let mut workbook = Workbook::empty();
        let mut header_style = CellStyle::default();
        header_style.font.bold = true;
        let header_style_index = workbook.styles.intern(header_style);
        let mut names = HashSet::new();

        for (index, table) in self.tables.iter().enumerate() {
            let name = unique_sheet_name(table.name(), index, &mut names);
            let sheet_index = workbook.add_sheet(name);
            if let Some(sheet) = workbook.sheet_mut(sheet_index) {
                for (row_index, row) in table.rows().iter().enumerate() {
                    let Ok(row_index) = u32::try_from(row_index) else {
                        break;
                    };
                    for (column_index, cell) in row.iter().enumerate() {
                        let Ok(column_index) = u32::try_from(column_index) else {
                            break;
                        };
                        let value = cell.value().clone();
                        if !matches!(value, CellValue::Empty) {
                            sheet.set(row_index, column_index, Cell::from_value(value));
                        }
                        if apply_header_style && cell.is_header() {
                            sheet.set_style(row_index, column_index, header_style_index);
                        }
                    }
                }
                sheet.merged.extend_from_slice(table.merges());
            }
        }
        if workbook.sheets.is_empty() {
            workbook.add_sheet("Sheet1");
        }
        workbook
    }

    /// 从工作簿构造中立表格文档；公式只投影缓存值，样式不进入中立模型。
    #[must_use]
    pub fn from_workbook(workbook: &Workbook) -> Self {
        let tables = workbook
            .sheets
            .iter()
            .map(|sheet| {
                let mut table = TabularTable::new(&sheet.name);
                let (row_count, column_count) = sheet.dimensions();
                for row_index in 0..row_count {
                    let row = (0..column_count)
                        .map(|column_index| TabularCell::new(sheet.value(row_index, column_index)))
                        .collect();
                    table.push_row(row);
                }
                for range in &sheet.merged {
                    table.push_merge(*range);
                }
                table
            })
            .collect();
        Self { tables }
    }
}

fn unique_sheet_name(requested: &str, index: usize, used: &mut HashSet<String>) -> String {
    let sanitized: String = requested
        .chars()
        .filter(|character| !matches!(character, ':' | '\\' | '/' | '?' | '*' | '[' | ']'))
        .take(31)
        .collect();
    let base = if sanitized.trim().is_empty() {
        format!("Table{}", index + 1)
    } else {
        sanitized
    };
    let mut candidate = base.clone();
    let mut suffix = 2usize;
    while !used.insert(candidate.to_ascii_lowercase()) {
        let marker = format!("-{suffix}");
        let keep = 31usize.saturating_sub(marker.len());
        candidate = format!("{}{}", base.chars().take(keep).collect::<String>(), marker);
        suffix += 1;
    }
    candidate
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CellValue;

    // --- TabularDocument 基本测试 --------------------------------------

    #[test]
    fn new_creates_empty_document() {
        let doc = TabularDocument::new();
        assert!(doc.tables().is_empty());
    }

    #[test]
    fn from_tables_creates_document() {
        let table = TabularTable::new("Test");
        let doc = TabularDocument::from_tables(vec![table]);
        assert_eq!(doc.tables().len(), 1);
        assert_eq!(doc.tables()[0].name(), "Test");
    }

    #[test]
    fn push_table_adds_table() {
        let mut doc = TabularDocument::new();
        doc.push_table(TabularTable::new("Sheet1"));
        doc.push_table(TabularTable::new("Sheet2"));
        assert_eq!(doc.tables().len(), 2);
    }

    // --- to_workbook 测试 ----------------------------------------------

    #[test]
    fn to_workbook_empty_document_has_one_sheet() {
        let doc = TabularDocument::new();
        let wb = doc.to_workbook();
        // 空文档应该有一个默认 Sheet1
        assert_eq!(wb.sheets.len(), 1);
    }

    #[test]
    fn to_workbook_with_data() {
        let mut table = TabularTable::new("Data");
        table.push_row(vec![
            TabularCell::header(CellValue::Text("Name".into())),
            TabularCell::header(CellValue::Text("Age".into())),
        ]);
        table.push_row(vec![
            TabularCell::new(CellValue::Text("Alice".into())),
            TabularCell::new(CellValue::Number(25.0)),
        ]);
        let doc = TabularDocument::from_tables(vec![table]);
        let wb = doc.to_workbook();
        assert_eq!(wb.sheets.len(), 1);
    }

    #[test]
    fn to_workbook_with_header_style() {
        let mut table = TabularTable::new("Styled");
        table.push_row(vec![
            TabularCell::header(CellValue::Text("Col1".into())),
            TabularCell::new(CellValue::Number(1.0)),
        ]);
        let doc = TabularDocument::from_tables(vec![table]);
        let wb = doc.to_workbook_with_header_style(true);
        assert_eq!(wb.sheets.len(), 1);
    }

    #[test]
    fn to_workbook_without_header_style() {
        let mut table = TabularTable::new("NoStyle");
        table.push_row(vec![
            TabularCell::header(CellValue::Text("Col1".into())),
            TabularCell::new(CellValue::Number(1.0)),
        ]);
        let doc = TabularDocument::from_tables(vec![table]);
        let wb = doc.to_workbook_with_header_style(false);
        assert_eq!(wb.sheets.len(), 1);
    }

    #[test]
    fn to_workbook_empty_value_skipped() {
        let mut table = TabularTable::new("Skip");
        table.push_row(vec![
            TabularCell::new(CellValue::Empty),
            TabularCell::new(CellValue::Number(42.0)),
        ]);
        let doc = TabularDocument::from_tables(vec![table]);
        let wb = doc.to_workbook();
        assert_eq!(wb.sheets.len(), 1);
    }

    #[test]
    fn to_workbook_multiple_tables() {
        let t1 = TabularTable::new("Table1");
        let t2 = TabularTable::new("Table2");
        let doc = TabularDocument::from_tables(vec![t1, t2]);
        let wb = doc.to_workbook();
        assert_eq!(wb.sheets.len(), 2);
    }

    // --- from_workbook 测试 --------------------------------------------

    #[test]
    fn from_workbook_roundtrip() {
        let mut table = TabularTable::new("RT");
        table.push_row(vec![
            TabularCell::header(CellValue::Text("X".into())),
            TabularCell::new(CellValue::Number(99.0)),
        ]);
        let doc = TabularDocument::from_tables(vec![table]);
        let wb = doc.to_workbook();
        let doc2 = TabularDocument::from_workbook(&wb);
        assert_eq!(doc2.tables().len(), 1);
    }

    // --- unique_sheet_name 测试 ----------------------------------------

    #[test]
    fn unique_sheet_name_basic() {
        let mut used = HashSet::new();
        let name = unique_sheet_name("Test", 0, &mut used);
        assert_eq!(name, "Test");
    }

    #[test]
    fn unique_sheet_name_sanitizes_special_chars() {
        let mut used = HashSet::new();
        // ":" "\" "/" "?" "*" "[" "]" are removed; "H" remains → "ABCDEFGH"
        let name = unique_sheet_name("A:B\\C/D?E*F[G]H", 0, &mut used);
        assert_eq!(name, "ABCDEFGH");
    }

    #[test]
    fn unique_sheet_name_empty_fallback() {
        let mut used = HashSet::new();
        let name = unique_sheet_name("", 0, &mut used);
        assert_eq!(name, "Table1");
    }

    #[test]
    fn unique_sheet_name_whitespace_fallback() {
        let mut used = HashSet::new();
        let name = unique_sheet_name("   ", 0, &mut used);
        assert_eq!(name, "Table1");
    }

    #[test]
    fn unique_sheet_name_deduplicates() {
        let mut used = HashSet::new();
        let n1 = unique_sheet_name("Sheet", 0, &mut used);
        let n2 = unique_sheet_name("Sheet", 0, &mut used);
        assert_eq!(n1, "Sheet");
        assert_eq!(n2, "Sheet-2");
    }

    #[test]
    fn unique_sheet_name_deduplicates_case_insensitive() {
        let mut used = HashSet::new();
        let n1 = unique_sheet_name("Sheet", 0, &mut used);
        let n2 = unique_sheet_name("SHEET", 0, &mut used);
        assert_eq!(n1, "Sheet");
        assert_eq!(n2, "SHEET-2");
    }

    #[test]
    fn unique_sheet_name_truncates_long_name() {
        let mut used = HashSet::new();
        let long = "A".repeat(50);
        let name = unique_sheet_name(&long, 0, &mut used);
        assert!(name.len() <= 31);
    }

    #[test]
    fn unique_sheet_name_truncates_with_suffix() {
        let mut used = HashSet::new();
        let long = "B".repeat(50);
        let n1 = unique_sheet_name(&long, 0, &mut used);
        let n2 = unique_sheet_name(&long, 0, &mut used);
        assert!(n1.len() <= 31);
        assert!(n2.len() <= 31);
        assert_ne!(n1, n2);
    }

    // --- PartialEq / Clone / Debug 测试 --------------------------------

    #[test]
    fn document_equality() {
        let doc1 = TabularDocument::new();
        let doc2 = TabularDocument::new();
        assert_eq!(doc1, doc2);
    }

    #[test]
    fn document_clone() {
        let doc = TabularDocument::new();
        let cloned = doc.clone();
        assert_eq!(doc, cloned);
    }

    #[test]
    fn document_debug() {
        let doc = TabularDocument::new();
        let dbg = format!("{:?}", doc);
        assert!(dbg.contains("TabularDocument"));
    }
}
