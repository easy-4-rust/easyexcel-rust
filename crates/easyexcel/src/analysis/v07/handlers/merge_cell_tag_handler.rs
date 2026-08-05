//! 对应 Java：`com.alibaba.excel.analysis.v07.handlers.MergeCellTagHandler`.

use std::collections::HashMap;

use crate::constant::excel_xml_constants::ATTRIBUTE_REF;
use crate::core::{CellExtra, CellExtraType, ExcelError, Result};

use super::xlsx_tag_handler::XlsxTagHandler;

/// 对应 Java：`MergeCellTagHandler`.
#[derive(Debug, Default)]
pub struct MergeCellTagHandler {
    /// Whether merge extras are enabled. (Java `support` / `extraReadSet`)
    pub enabled: bool,
    /// Last parsed merge extra (Java `setCellExtra` + `extra(...)`).
    pub last_extra: Option<CellExtra>,
}

impl MergeCellTagHandler {
    /// Creates a handler; `enabled` mirrors Java `support(XlsxReadContext)`.
    #[must_use]
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            last_extra: None,
        }
    }

    /// Java `MergeCellTagHandler.startElement`.
    ///
    /// # Errors
    ///
    /// 当 `ref` 单元格区域解析失败时返回 [`ExcelError::Format`]。
    pub fn start_merge(&mut self, attrs: &HashMap<String, String>) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        let Some(reference) = attrs.get(ATTRIBUTE_REF) else {
            return Ok(());
        };
        if reference.is_empty() {
            return Ok(());
        }
        self.last_extra = Some(cell_extra_from_ref(CellExtraType::Merge, None, reference)?);
        Ok(())
    }

    /// Same as [`Self::start_merge`], but missing / empty `ref` is an error
    /// (matches historical `xlsx_rows::required_attribute` behaviour).
    ///
    /// # Errors
    ///
    /// 当 `ref` 缺失/为空或单元格区域解析失败时返回 [`ExcelError::Format`]。
    pub fn start_merge_required(&mut self, attrs: &HashMap<String, String>) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        let reference = attrs
            .get(ATTRIBUTE_REF)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| ExcelError::Format("merge cell ref is missing".to_owned()))?;
        self.last_extra = Some(cell_extra_from_ref(CellExtraType::Merge, None, reference)?);
        Ok(())
    }
}

impl XlsxTagHandler for MergeCellTagHandler {
    fn support(&self) -> bool {
        self.enabled
    }

    /// Java `MergeCellTagHandler.startElement`.
    fn start_element(&mut self, name: &str, attrs: &str) {
        let local = name.rsplit(':').next().unwrap_or(name);
        if local != "mergeCell" {
            return;
        }
        let mut map = HashMap::new();
        for token in attrs.split_whitespace() {
            if let Some((key, value)) = token.split_once('=') {
                map.insert(key.to_owned(), value.to_owned());
            }
        }
        let _ = self.start_merge(&map);
    }
}

/// Builds a [`CellExtra`] from an A1 / A1:B2 reference (Java `new CellExtra(type, text, ref)`).
///
/// Also enforces first≤last ordering used by `xlsx_rows::parse_cell_range`.
pub(crate) fn cell_extra_from_ref(
    extra_type: CellExtraType,
    text: Option<String>,
    reference: &str,
) -> Result<CellExtra> {
    let (first, last) = match reference.split_once(':') {
        Some((first, last)) => (first, last),
        None => (reference, reference),
    };
    let (first_row, first_column) = parse_a1(first)?;
    let (last_row, last_column) = parse_a1(last)?;
    if first_row > last_row || first_column > last_column {
        return Err(ExcelError::Format(format!(
            "invalid cell range ordering: {reference}"
        )));
    }
    Ok(CellExtra::new(
        extra_type,
        text,
        first_row,
        last_row,
        first_column,
        last_column,
    ))
}

fn parse_a1(reference: &str) -> Result<(u32, usize)> {
    const MAX_ROW: u32 = 1_048_576;
    const MAX_COL: usize = 16_384;
    let reference = reference.strip_prefix('$').unwrap_or(reference);
    let column_end = reference
        .find(|character: char| !character.is_ascii_alphabetic())
        .unwrap_or(reference.len());
    let (column, row) = reference.split_at(column_end);
    let row = row.strip_prefix('$').unwrap_or(row);
    if column.is_empty() || row.is_empty() || !row.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(ExcelError::Format(format!(
            "invalid cell reference: {reference}"
        )));
    }
    let mut one_based_column = 0_usize;
    for letter in column.bytes() {
        one_based_column = one_based_column
            .checked_mul(26)
            .and_then(|value| {
                value.checked_add(usize::from(letter.to_ascii_uppercase() - b'A' + 1))
            })
            .ok_or_else(|| ExcelError::Format(format!("invalid cell reference: {reference}")))?;
    }
    if !(1..=MAX_COL).contains(&one_based_column) {
        return Err(ExcelError::Format(format!(
            "column index exceeds XLSX limits: {reference}"
        )));
    }
    let one_based_row: u32 = row
        .parse()
        .map_err(|error| ExcelError::Format(format!("{error}")))?;
    if !(1..=MAX_ROW).contains(&one_based_row) {
        return Err(ExcelError::Format(format!(
            "row index exceeds XLSX limits: {reference}"
        )));
    }
    Ok((one_based_row - 1, one_based_column - 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_merge_parses_single_and_ranged_refs() -> Result<()> {
        // 对应 Java：MergeCellTagHandler.startElement 解析 ref
        let mut handler = MergeCellTagHandler::new(true);
        let mut attrs = HashMap::new();
        attrs.insert(ATTRIBUTE_REF.to_owned(), "B2:C3".to_owned());
        handler.start_merge(&attrs)?;
        let extra = handler.last_extra.clone().expect("merge extra");
        assert_eq!(extra.extra_type(), CellExtraType::Merge);
        assert_eq!(extra.first_row_index(), 1);
        assert_eq!(extra.last_row_index(), 2);
        assert_eq!(extra.first_column_index(), 1);
        assert_eq!(extra.last_column_index(), 2);
        assert_eq!(extra.text(), None);

        let mut attrs = HashMap::new();
        attrs.insert(ATTRIBUTE_REF.to_owned(), "A1".to_owned());
        handler.start_merge(&attrs)?;
        let extra = handler.last_extra.as_ref().expect("merge extra");
        assert_eq!((extra.first_row_index(), extra.last_row_index()), (0, 0));
        assert_eq!(
            (extra.first_column_index(), extra.last_column_index()),
            (0, 0)
        );
        Ok(())
    }

    #[test]
    fn start_merge_ignores_disabled_missing_or_empty_refs() -> Result<()> {
        // 对应 Java：support()=false / 缺 ref / 空 ref 均跳过
        let mut disabled = MergeCellTagHandler::new(false);
        let mut attrs = HashMap::new();
        attrs.insert(ATTRIBUTE_REF.to_owned(), "A1:B2".to_owned());
        disabled.start_merge(&attrs)?;
        assert!(disabled.last_extra.is_none());

        let mut handler = MergeCellTagHandler::new(true);
        handler.start_merge(&HashMap::new())?;
        assert!(handler.last_extra.is_none());
        let mut attrs = HashMap::new();
        attrs.insert(ATTRIBUTE_REF.to_owned(), String::new());
        handler.start_merge(&attrs)?;
        assert!(handler.last_extra.is_none());
        Ok(())
    }

    #[test]
    fn start_merge_required_errors_on_missing_ref() {
        // 对应 Java：严格模式缺 ref 报错
        let mut handler = MergeCellTagHandler::new(true);
        assert!(handler.start_merge_required(&HashMap::new()).is_err());
        let mut attrs = HashMap::new();
        attrs.insert(ATTRIBUTE_REF.to_owned(), String::new());
        assert!(handler.start_merge_required(&attrs).is_err());
        let mut attrs = HashMap::new();
        attrs.insert(ATTRIBUTE_REF.to_owned(), "A1:B2".to_owned());
        assert!(handler.start_merge_required(&attrs).is_ok());
    }

    #[test]
    fn cell_extra_from_ref_validates_range_ordering_and_bounds() {
        // 对应 Java：首尾行/列乱序与越界引用报错
        assert!(cell_extra_from_ref(CellExtraType::Merge, None, "B2:A1").is_err());
        assert!(cell_extra_from_ref(CellExtraType::Merge, None, "A1:A1048577").is_err());
        assert!(cell_extra_from_ref(CellExtraType::Merge, None, "XFE1").is_err());
        assert!(cell_extra_from_ref(CellExtraType::Merge, None, "1A").is_err());
        // $ 前缀与合法最大范围
        let extra = cell_extra_from_ref(CellExtraType::Merge, None, "$A$1:$XFD$1048576").unwrap();
        assert_eq!(extra.first_row_index(), 0);
        assert_eq!(extra.last_row_index(), 1_048_575);
        assert_eq!(extra.last_column_index(), 16_383);
    }

    #[test]
    fn tag_events_dispatch_only_for_merge_cell() {
        // 对应 Java：SAX startElement 仅处理 mergeCell
        let mut handler = MergeCellTagHandler::new(true);
        handler.start_element("mergeCell", "ref=A1:B2");
        assert!(handler.last_extra.is_some());
        handler.start_element("x:mergeCell", "ref=C1:D1");
        assert!(handler.last_extra.is_some());
        handler.start_element("row", "ref=E1:F1");
        let before = handler.last_extra.clone();
        handler.start_element("row", "ref=Z9:Z10");
        assert_eq!(handler.last_extra, before);
        // support() 与 enabled 一致
        assert!(handler.support());
        assert!(!MergeCellTagHandler::new(false).support());
    }
}
