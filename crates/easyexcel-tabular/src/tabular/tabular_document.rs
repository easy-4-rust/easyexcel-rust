use std::collections::HashSet;

use easyexcel_model::{Cell, CellValue, Workbook};

use super::{TabularCell, TabularTable};

/// 可包含多个表格的中立文档。
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
        let mut workbook = Workbook::empty();
        let mut names = HashSet::new();
        for (index, table) in self.tables.iter().enumerate() {
            let name = unique_sheet_name(table.name(), index, &mut names);
            let sheet_index = workbook.add_sheet(name);
            if let Some(sheet) = workbook.sheet_mut(sheet_index) {
                for (row_index, row) in table.rows().iter().enumerate() {
                    for (column_index, cell) in row.iter().enumerate() {
                        let value = cell.value().clone();
                        if !matches!(value, CellValue::Empty) {
                            sheet.set(
                                row_index as u32,
                                column_index as u32,
                                Cell::from_value(value),
                            );
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

    /// 从工作簿构造中立表格文档。
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

    #[test]
    fn creates_unique_valid_sheet_names() {
        let document =
            TabularDocument::from_tables(vec![TabularTable::new("A/B"), TabularTable::new("AB")]);
        let workbook = document.to_workbook();
        assert_eq!(workbook.sheets[0].name, "AB");
        assert_eq!(workbook.sheets[1].name, "AB-2");
    }
}
