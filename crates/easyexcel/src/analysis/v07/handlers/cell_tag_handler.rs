//! 对应 Java：`com.alibaba.excel.analysis.v07.handlers.CellTagHandler`.
//!
//! `quick_xml` 事件循环、A1 引用和索引解析由 `easyexcel-xlsx` 负责；本处理器只
//! 保留 Java `CellTagHandler` 的临时状态与 `EasyExcel` `CellDataType` 映射。

use std::collections::HashMap;

use crate::constant::excel_xml_constants::{ATTRIBUTE_R, ATTRIBUTE_S, ATTRIBUTE_T};
use crate::core::{CellDataType, ExcelError, Result};

use super::xlsx_tag_handler::XlsxTagHandler;

/// Default style / format index when `c@s` is absent.
/// Java `CellTagHandler.DEFAULT_FORMAT_INDEX`.
const DEFAULT_FORMAT_INDEX: usize = 0;

include!("cell_tag_handler/cell_start_attrs.rs");

/// 对应 Java：`CellTagHandler`.
///
/// Holds the per-cell temp buffer that Java stores on `XlsxReadSheetHolder`
/// (`tempCellData` / `tempData`).
#[derive(Debug, Default)]
pub struct CellTagHandler {
    /// Current column cursor after the last `startElement`. (Java sheet holder)
    pub column_index: Option<usize>,
    /// Style index from the last `c@s`.
    pub style_index: usize,
    /// OOXML type code from the last `c@t`.
    pub cell_type: Option<String>,
    /// Logical type from [`CellDataType::build_from_cell_type`].
    pub data_type: CellDataType,
    /// Accumulated character data for `<v>` / inline text. (Java `tempData`)
    pub temp_data: String,
}

impl CellTagHandler {
    /// 对应 Java：com.alibaba.excel.analysis.v07.handlers.CellTagHandler。 Creates an idle handler.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 对应 Java：com.alibaba.excel.analysis.v07.handlers.CellTagHandler。 Java `CellTagHandler.startElement(XlsxReadContext, String, Attributes)`.
    ///
    /// Parses `r` / `t` / `s` and resets `temp_data`.
    ///
    /// # Errors
    ///
    /// 当 `r` 引用无法解析或 `t` 类型不受支持时返回 [`ExcelError::Format`]。
    pub fn start_cell(
        &mut self,
        attrs: &HashMap<String, String>,
        fallback_row: u32,
        fallback_column: usize,
    ) -> Result<CellStartAttrs> {
        let parsed = Self::parse_start(attrs, fallback_row, fallback_column)?;
        self.column_index = Some(parsed.position.1);
        self.style_index = parsed.style_index;
        self.cell_type.clone_from(&parsed.cell_type);
        self.data_type = parsed.data_type;
        self.temp_data.clear();
        Ok(parsed)
    }

    /// 对应 Java：com.alibaba.excel.analysis.v07.handlers.CellTagHandler。 不修改处理器状态地解析 Java handler 所需属性。
    ///
    /// Corresponds to the attribute-reading portion of Java `startElement`.
    ///
    /// # Errors
    ///
    /// 当 `r` 引用无法解析或 `t` 类型不受支持时返回 [`ExcelError::Format`]。
    pub fn parse_start(
        attrs: &HashMap<String, String>,
        fallback_row: u32,
        fallback_column: usize,
    ) -> Result<CellStartAttrs> {
        let position = match attrs.get(ATTRIBUTE_R) {
            Some(reference) => {
                easyexcel_xlsx::parse_a1_cell_reference(reference).map_err(ExcelError::from)?
            }
            None => (fallback_row, fallback_column),
        };
        let style_index = match attrs.get(ATTRIBUTE_S) {
            Some(value) if !value.is_empty() => {
                easyexcel_xlsx::parse_xlsx_index(value, "style").map_err(ExcelError::from)?
            }
            _ => DEFAULT_FORMAT_INDEX,
        };
        let cell_type = attrs.get(ATTRIBUTE_T).cloned();
        let data_type =
            CellDataType::build_from_cell_type(cell_type.as_deref()).ok_or_else(|| {
                ExcelError::Format(format!(
                    "unsupported XLSX cell type: {}",
                    cell_type.as_deref().unwrap_or_default()
                ))
            })?;
        Ok(CellStartAttrs {
            position,
            style_index,
            cell_type,
            data_type,
        })
    }

    /// 对应 Java：com.alibaba.excel.analysis.v07.handlers.CellTagHandler。 Java `AbstractCellValueTagHandler.characters` path when this handler
    /// owns the temp buffer (also used when `<v>` text arrives).
    pub fn append_characters(&mut self, ch: &str) {
        self.temp_data.push_str(ch);
    }

    /// 对应 Java：com.alibaba.excel.analysis.v07.handlers.CellTagHandler。 Clears per-cell state after `endElement`. (Java puts cell into `cellMap`)
    pub fn reset_temp(&mut self) {
        self.temp_data.clear();
        self.cell_type = None;
        self.data_type = CellDataType::Empty;
        self.style_index = DEFAULT_FORMAT_INDEX;
    }
}

impl XlsxTagHandler for CellTagHandler {
    /// Java `CellTagHandler.startElement` — `attrs` is `key=value` pairs separated by spaces.
    fn start_element(&mut self, name: &str, attrs: &str) {
        let local = easyexcel_xlsx::local_tag_name(name);
        if local != "c" {
            return;
        }
        let map = easyexcel_xlsx::parse_attribute_pairs(attrs);
        let _ = self.start_cell(&map, 0, self.column_index.unwrap_or(0));
    }

    /// Java `CellTagHandler.endElement` — clears temp buffers after the cell closes.
    fn end_element(&mut self, name: &str) {
        let local = easyexcel_xlsx::local_tag_name(name);
        if local == "c" {
            self.reset_temp();
        }
    }

    /// Java path routes characters through value handlers; we also accept them here
    /// when the handler is used stand-alone.
    fn characters(&mut self, ch: &str) {
        self.append_characters(ch);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_start_reads_r_t_s() {
        let mut attrs = HashMap::new();
        attrs.insert("r".into(), "B2".into());
        attrs.insert("t".into(), "s".into());
        attrs.insert("s".into(), "3".into());
        let parsed = CellTagHandler::parse_start(&attrs, 0, 0).unwrap();
        assert_eq!(parsed.position, (1, 1));
        assert_eq!(parsed.style_index, 3);
        assert_eq!(parsed.cell_type.as_deref(), Some("s"));
        assert_eq!(parsed.data_type, CellDataType::String);
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn start_cell_parses_and_resets_temp_buffer() {
        // 对应 Java：CellTagHandler.startElement 解析 r/t/s 并清空 tempData
        let mut handler = CellTagHandler::new();
        handler.append_characters("old");
        let mut attrs = HashMap::new();
        attrs.insert(ATTRIBUTE_R.to_owned(), "C5".to_owned());
        attrs.insert(ATTRIBUTE_T.to_owned(), "b".to_owned());
        attrs.insert(ATTRIBUTE_S.to_owned(), "7".to_owned());
        let parsed = handler.start_cell(&attrs, 0, 0).unwrap();
        assert_eq!(parsed.position, (4, 2));
        assert_eq!(handler.column_index, Some(2));
        assert_eq!(handler.style_index, 7);
        assert_eq!(handler.cell_type.as_deref(), Some("b"));
        assert_eq!(handler.data_type, CellDataType::Boolean);
        assert!(handler.temp_data.is_empty());
    }

    #[test]
    fn start_cell_uses_fallback_cursor_without_r() {
        // 对应 Java：无 r 属性时沿用游标位置
        let mut handler = CellTagHandler::new();
        let parsed = handler.start_cell(&HashMap::new(), 3, 5).unwrap();
        assert_eq!(parsed.position, (3, 5));
        assert_eq!(handler.column_index, Some(5));
        // 无 s 属性时默认样式 0（对应 Java：DEFAULT_FORMAT_INDEX）
        assert_eq!(handler.style_index, 0);
    }

    #[test]
    fn start_cell_rejects_unknown_type_and_bad_style() {
        // 对应 Java：未知 t 类型 / 非数字 s 报错
        let mut attrs = HashMap::new();
        attrs.insert(ATTRIBUTE_T.to_owned(), "zzz".to_owned());
        assert!(CellTagHandler::new().start_cell(&attrs, 0, 0).is_err());

        let mut attrs = HashMap::new();
        attrs.insert(ATTRIBUTE_S.to_owned(), "nan".to_owned());
        assert!(CellTagHandler::new().start_cell(&attrs, 0, 0).is_err());

        // 无 t 属性时默认数值类型
        let parsed = CellTagHandler::parse_start(&HashMap::new(), 0, 0).unwrap();
        assert_eq!(parsed.cell_type, None);
    }

    #[test]
    fn tag_events_update_cell_state() {
        // 对应 Java：CellTagHandler 的 SAX 事件入口（含前缀标签名）
        let mut handler = CellTagHandler::new();
        handler.characters("accumulate");
        assert_eq!(handler.temp_data, "accumulate");
        handler.start_element("x:c", "r=A1 t=s");
        assert!(handler.temp_data.is_empty());
        assert_eq!(handler.data_type, CellDataType::String);
        // 非 c 标签不重置状态
        handler.start_element("row", "r=1");
        handler.end_element("row");
        assert_eq!(handler.data_type, CellDataType::String);
        handler.end_element("c");
        assert_eq!(handler.data_type, CellDataType::Empty);
        assert_eq!(handler.style_index, 0);
        assert_eq!(handler.cell_type, None);
    }

    #[test]
    fn parse_cell_reference_rejects_invalid_and_out_of_range() {
        // 对应 Java：非法单元格引用 / 越界行列报错
        let mut attrs = HashMap::new();
        attrs.insert(ATTRIBUTE_R.to_owned(), "A".to_owned());
        assert!(CellTagHandler::parse_start(&attrs, 0, 0).is_err());

        let mut attrs = HashMap::new();
        attrs.insert(ATTRIBUTE_R.to_owned(), "XFE1".to_owned()); // 列 16385 越界
        assert!(CellTagHandler::parse_start(&attrs, 0, 0).is_err());

        let mut attrs = HashMap::new();
        attrs.insert(ATTRIBUTE_R.to_owned(), "A1048577".to_owned()); // 行越界
        assert!(CellTagHandler::parse_start(&attrs, 0, 0).is_err());

        // $ 前缀与最大合法坐标
        let mut attrs = HashMap::new();
        attrs.insert(ATTRIBUTE_R.to_owned(), "$XFD$1048576".to_owned());
        let parsed = CellTagHandler::parse_start(&attrs, 0, 0).unwrap();
        assert_eq!(parsed.position, (1_048_575, 16_383));
    }
}
