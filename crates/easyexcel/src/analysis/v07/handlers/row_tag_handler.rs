//! 对应 Java：`com.alibaba.excel.analysis.v07.handlers.RowTagHandler`.
//!
//! Row-index resolution is shared with `xlsx_rows::XlsxDisplayCellReader::next_cell`
//! via [`RowTagHandler::resolve_row_index`]. Empty-row synthesis from Java
//! `startElement` (emitting `RowTypeEnum.EMPTY` for gaps) remains the
//! responsibility of higher-level read dispatchers.

use std::collections::HashMap;

use crate::constant::excel_xml_constants::ATTRIBUTE_R;
use crate::core::{ExcelError, Result};

use super::xlsx_tag_handler::XlsxTagHandler;

/// 对应 Java：`RowTagHandler`.
#[derive(Debug, Default)]
pub struct RowTagHandler {
    /// Zero-based current row index. (Java `XlsxReadSheetHolder.rowIndex`)
    pub row_index: Option<u32>,
    /// Whether the open row accumulated any non-empty cells. (Java `RowTypeEnum`)
    pub has_data: bool,
}

impl RowTagHandler {
    /// Creates an idle handler.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Java `RowTagHandler.startElement` — resolve `r` via
    /// `PositionUtils.getRowByRowTagt(rowTagt, before)`.
    ///
    /// Returns the zero-based row index for the opened `<row>`.
    ///
    /// # Errors
    ///
    /// 当 `r` 非空但无法解析为数字，或超出 XLSX 行上限（`1..=1_048_576`）时
    /// 返回 [`ExcelError::Format`]。
    pub fn resolve_row_index(row_attr: Option<&str>, before: u32) -> Result<u32> {
        match row_attr {
            Some(value) if !value.is_empty() => {
                let one_based: u32 = value
                    .parse()
                    .map_err(|error| ExcelError::Format(format!("{error}")))?;
                if !(1..=1_048_576).contains(&one_based) {
                    return Err(ExcelError::Format(format!(
                        "row index exceeds XLSX limits: {value}"
                    )));
                }
                Ok(one_based - 1)
            }
            _ => Ok(before),
        }
    }

    /// Java `RowTagHandler.startElement` body (without empty-row gap fill).
    ///
    /// # Errors
    ///
    /// 当 `r` 非空但无法解析为数字，或超出 XLSX 行上限时返回 [`ExcelError::Format`]。
    pub fn start_row(&mut self, attrs: &HashMap<String, String>) -> Result<u32> {
        let before = self.row_index.unwrap_or(0);
        let row = Self::resolve_row_index(attrs.get(ATTRIBUTE_R).map(String::as_str), before)?;
        self.row_index = Some(row);
        self.has_data = false;
        Ok(row)
    }

    /// Java `RowTagHandler.endElement` — advances the cursor and reports whether
    /// the row looked like `DATA` vs `EMPTY`.
    pub fn end_row(&mut self) -> (u32, bool) {
        let row = self.row_index.unwrap_or(0);
        let has_data = self.has_data;
        self.row_index = Some(row.saturating_add(1));
        self.has_data = false;
        (row, has_data)
    }

    /// Marks that at least one non-empty cell was seen in the open row.
    pub fn mark_data(&mut self) {
        self.has_data = true;
    }
}

impl XlsxTagHandler for RowTagHandler {
    /// Java `RowTagHandler.startElement`.
    fn start_element(&mut self, name: &str, attrs: &str) {
        let local = name.rsplit(':').next().unwrap_or(name);
        if local != "row" {
            return;
        }
        let mut map = HashMap::new();
        for token in attrs.split_whitespace() {
            if let Some((key, value)) = token.split_once('=') {
                map.insert(key.to_owned(), value.to_owned());
            }
        }
        let _ = self.start_row(&map);
    }

    /// Java `RowTagHandler.endElement`.
    fn end_element(&mut self, name: &str) {
        let local = name.rsplit(':').next().unwrap_or(name);
        if local == "row" {
            let _ = self.end_row();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_row_index_from_one_based_attr() {
        assert_eq!(RowTagHandler::resolve_row_index(Some("3"), 0).unwrap(), 2);
        assert_eq!(RowTagHandler::resolve_row_index(None, 4).unwrap(), 4);
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn resolve_row_index_rejects_invalid_values() {
        // 对应 Java：PositionUtils 对越界/非数字行号报错
        assert!(RowTagHandler::resolve_row_index(Some("0"), 0).is_err());
        assert!(RowTagHandler::resolve_row_index(Some("1048577"), 0).is_err());
        assert!(RowTagHandler::resolve_row_index(Some("abc"), 0).is_err());
        // 空字符串回退到游标（对应 Java：rowTagt 为空使用 before）
        assert_eq!(RowTagHandler::resolve_row_index(Some(""), 7).unwrap(), 7);
    }

    #[test]
    fn start_and_end_row_track_index_and_data() {
        // 对应 Java：RowTagHandler.startElement/endElement 维护 rowIndex 与 RowType
        let mut handler = RowTagHandler::new();
        let mut attrs = HashMap::new();
        attrs.insert(ATTRIBUTE_R.to_owned(), "5".to_owned());
        assert_eq!(handler.start_row(&attrs).unwrap(), 4);
        assert_eq!(handler.row_index, Some(4));
        assert!(!handler.has_data);
        handler.mark_data();
        assert!(handler.has_data);
        let (row, has_data) = handler.end_row();
        assert_eq!((row, has_data), (4, true));
        assert_eq!(handler.row_index, Some(5));
        // 无 r 属性时沿用游标（对应 Java：rowTagt 为空使用 before）
        let empty = HashMap::new();
        assert_eq!(handler.start_row(&empty).unwrap(), 5);
        assert_eq!(handler.end_row(), (5, false));
    }

    #[test]
    fn tag_events_dispatch_only_for_row() {
        // 对应 Java：SAX 事件仅路由 row 标签
        let mut handler = RowTagHandler::new();
        handler.start_element("x:row", "r=3");
        assert_eq!(handler.row_index, Some(2));
        handler.start_element("other", "r=9");
        assert_eq!(handler.row_index, Some(2));
        handler.characters("x");
        handler.end_element("other");
        assert_eq!(handler.row_index, Some(2));
        handler.end_element("row");
        assert_eq!(handler.row_index, Some(3));
        assert!(!handler.has_data);
    }
}
