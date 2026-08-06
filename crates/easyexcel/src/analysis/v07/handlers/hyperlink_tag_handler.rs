//! 对应 Java：`com.alibaba.excel.analysis.v07.handlers.HyperlinkTagHandler`.

use std::collections::HashMap;

use crate::constant::excel_xml_constants::{ATTRIBUTE_LOCATION, ATTRIBUTE_REF, ATTRIBUTE_RID};
use crate::core::{CellExtra, CellExtraType, ExcelError, Result};

use super::merge_cell_tag_handler::cell_extra_from_ref;
use super::xlsx_tag_handler::XlsxTagHandler;

/// 对应 Java：`HyperlinkTagHandler`.
#[derive(Debug, Default)]
pub struct HyperlinkTagHandler {
    /// Whether hyperlink extras are enabled. (Java `support`)
    pub enabled: bool,
    /// Last parsed hyperlink extra.
    pub last_extra: Option<CellExtra>,
}

impl HyperlinkTagHandler {
    /// 对应 Java：com.alibaba.excel.analysis.v07.handlers.HyperlinkTagHandler。 Creates a handler; `enabled` mirrors Java `support(XlsxReadContext)`.
    #[must_use]
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            last_extra: None,
        }
    }

    /// 对应 Java：com.alibaba.excel.analysis.v07.handlers.HyperlinkTagHandler。 Java `HyperlinkTagHandler.startElement`.
    ///
    /// `resolve_r_id` maps `r:id` → target URI (Java `PackageRelationshipCollection`).
    ///
    /// # Errors
    ///
    /// 当 `ref` 单元格区域解析失败时返回 [`ExcelError::Format`]。
    pub fn start_hyperlink(
        &mut self,
        attrs: &HashMap<String, String>,
        resolve_r_id: &dyn Fn(&str) -> Option<String>,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        let Some(reference) = attrs.get(ATTRIBUTE_REF) else {
            return Ok(());
        };
        if reference.is_empty() {
            return Ok(());
        }
        if let Some(location) = attrs.get(ATTRIBUTE_LOCATION) {
            self.last_extra = Some(cell_extra_from_ref(
                CellExtraType::Hyperlink,
                Some(location.clone()),
                reference,
            )?);
            return Ok(());
        }
        // Java `Attributes.get("r:id")`; `quick_xml` local-name strips the
        // prefix so worksheet SAX attrs arrive as plain `"id"`.
        let r_id = attrs
            .get(ATTRIBUTE_RID)
            .or_else(|| attrs.get("id"))
            .map(String::as_str);
        if let Some(r_id) = r_id
            && let Some(uri) = resolve_r_id(r_id)
        {
            self.last_extra = Some(cell_extra_from_ref(
                CellExtraType::Hyperlink,
                Some(uri),
                reference,
            )?);
        }
        Ok(())
    }

    /// 对应 Java：com.alibaba.excel.analysis.v07.handlers.HyperlinkTagHandler。 Strict variant used by `xlsx_rows::parse_worksheet_extras`.
    ///
    /// Missing / empty `ref`, missing `id`/`location`, and unresolved
    /// relationships all return [`ExcelError`] (historical Rust reader behaviour).
    ///
    /// # Errors
    ///
    /// 当 `ref` 缺失/为空、`id`/`location` 缺失、关系解析失败，或 `ref` 区域
    /// 解析失败时返回 [`ExcelError::Format`]。
    pub fn start_hyperlink_required(
        &mut self,
        attrs: &HashMap<String, String>,
        resolve_r_id: &dyn Fn(&str) -> Result<String>,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        let reference = attrs
            .get(ATTRIBUTE_REF)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| ExcelError::Format("hyperlink ref is missing".to_owned()))?;
        if let Some(location) = attrs.get(ATTRIBUTE_LOCATION) {
            self.last_extra = Some(cell_extra_from_ref(
                CellExtraType::Hyperlink,
                Some(location.clone()),
                reference,
            )?);
            return Ok(());
        }
        let r_id = attrs
            .get(ATTRIBUTE_RID)
            .or_else(|| attrs.get("id"))
            .ok_or_else(|| ExcelError::Format("hyperlink id is missing".to_owned()))?;
        let uri = resolve_r_id(r_id)?;
        self.last_extra = Some(cell_extra_from_ref(
            CellExtraType::Hyperlink,
            Some(uri),
            reference,
        )?);
        Ok(())
    }
}

impl XlsxTagHandler for HyperlinkTagHandler {
    fn support(&self) -> bool {
        self.enabled
    }

    /// Java `HyperlinkTagHandler.startElement` (location-only; `r:id` needs a resolver).
    fn start_element(&mut self, name: &str, attrs: &str) {
        let local = easyexcel_xlsx::local_tag_name(name);
        if local != "hyperlink" {
            return;
        }
        let map = easyexcel_xlsx::parse_attribute_pairs(attrs);
        let _ = self.start_hyperlink(&map, &|_| None);
    }
}

// 共享测试解析器：在 location 分支/disabled 分支中解析器不会被调用，
// 与真正调用解析器的用例共享同一函数体，避免产生永不执行的闭包行。
#[cfg(test)]
fn resolve_none(_: &str) -> Option<String> {
    None
}

#[cfg(test)]
// 对应 Java：测试辅助函数须与 `&dyn Fn(&str) -> Result<String>` 解析器签名一致，
// 保留 Result 返回类型以满足调用点类型约束。
#[allow(clippy::unnecessary_wraps)]
fn resolve_ok(_: &str) -> Result<String> {
    Ok("https://example.com".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attrs(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect()
    }

    #[test]
    fn start_hyperlink_resolves_location_and_r_id() -> Result<()> {
        // 对应 Java：location 直接作为文本；r:id 通过关系解析器
        let mut handler = HyperlinkTagHandler::new(true);
        handler.start_hyperlink(
            &attrs(&[("ref", "A1"), ("location", "sheet2!B2")]),
            &resolve_none,
        )?;
        let extra = handler.last_extra.as_ref().expect("hyperlink extra");
        assert_eq!(extra.extra_type(), CellExtraType::Hyperlink);
        assert_eq!(extra.text(), Some("sheet2!B2"));
        assert_eq!(extra.first_row_index(), 0);

        handler.start_hyperlink(&attrs(&[("ref", "B2:C3"), ("id", "rId1")]), &|id| {
            assert_eq!(id, "rId1");
            Some("https://example.com".to_owned())
        })?;
        let extra = handler.last_extra.as_ref().expect("hyperlink extra");
        assert_eq!(extra.text(), Some("https://example.com"));
        assert_eq!((extra.first_row_index(), extra.last_row_index()), (1, 2));

        // rId 无法解析时保持上一次结果
        handler.start_hyperlink(&attrs(&[("ref", "D1"), ("r:id", "rId2")]), &resolve_none)?;
        let extra = handler.last_extra.as_ref().expect("hyperlink extra");
        assert_eq!(extra.text(), Some("https://example.com"));
        Ok(())
    }

    #[test]
    fn start_hyperlink_skips_disabled_missing_or_empty_refs() -> Result<()> {
        // 对应 Java：support()=false / 缺 ref / 空 ref 均跳过
        let mut disabled = HyperlinkTagHandler::new(false);
        disabled.start_hyperlink(&attrs(&[("ref", "A1")]), &|_| None)?;
        assert!(disabled.last_extra.is_none());
        assert!(!disabled.support());

        let mut handler = HyperlinkTagHandler::new(true);
        handler.start_hyperlink(&HashMap::new(), &|_| None)?;
        assert!(handler.last_extra.is_none());
        handler.start_hyperlink(&attrs(&[("ref", "")]), &|_| None)?;
        assert!(handler.last_extra.is_none());
        Ok(())
    }

    #[test]
    fn start_hyperlink_required_reports_each_missing_part() -> Result<()> {
        // 对应 Java：严格模式缺 ref / 缺 id 报错
        let mut handler = HyperlinkTagHandler::new(true);
        assert!(
            handler
                .start_hyperlink_required(&HashMap::new(), &|_| Ok("x".to_owned()))
                .is_err()
        );
        assert!(
            handler
                .start_hyperlink_required(&attrs(&[("ref", "")]), &|_| Ok("x".to_owned()))
                .is_err()
        );
        assert!(
            handler
                .start_hyperlink_required(&attrs(&[("ref", "A1")]), &|_| Ok("x".to_owned()))
                .is_err()
        );
        assert!(
            handler
                .start_hyperlink_required(&attrs(&[("ref", "A1"), ("id", "rId1")]), &|_| {
                    Err(ExcelError::Format("broken relationship".to_owned()))
                })
                .is_err()
        );
        // location 优先于 id
        handler.start_hyperlink_required(
            &attrs(&[("ref", "A1"), ("location", "local!A1"), ("id", "rId1")]),
            &resolve_ok,
        )?;
        let extra = handler.last_extra.as_ref().expect("hyperlink extra");
        assert_eq!(extra.text(), Some("local!A1"));
        Ok(())
    }

    #[test]
    fn tag_events_dispatch_only_for_hyperlink() {
        // 对应 Java：SAX startElement 仅处理 hyperlink
        let mut handler = HyperlinkTagHandler::new(true);
        handler.start_element("hyperlink", "ref=A1 location=example.com");
        assert!(handler.last_extra.is_some());
        handler.start_element("x:hyperlink", "ref=B2 location=example.org");
        assert!(handler.last_extra.is_some());
        handler.start_element("row", "ref=A1");
        let before = handler.last_extra.clone();
        handler.start_element("row", "ref=Z9 location=never");
        assert_eq!(handler.last_extra, before);
    }
}

#[cfg(test)]
mod tests_extra2 {
    use super::*;

    fn attrs(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect()
    }

    #[test]
    fn start_hyperlink_propagates_invalid_reference_errors() {
        // 对应 Java：超链接 ref 区域乱序（首>尾）时报错
        let mut handler = HyperlinkTagHandler::new(true);

        // location 分支的 cell_extra_from_ref 失败路径
        let error = handler
            .start_hyperlink(
                &attrs(&[("ref", "B2:A1"), ("location", "sheet2!B2")]),
                &|_| None,
            )
            .expect_err("reversed ref with location must fail");
        assert!(error.to_string().contains("invalid cell range ordering"));

        // r:id 分支的 cell_extra_from_ref 失败路径
        let error = handler
            .start_hyperlink(&attrs(&[("ref", "1A"), ("id", "rId1")]), &|_| {
                Some("https://example.com".to_owned())
            })
            .expect_err("invalid ref with r:id must fail");
        assert!(error.to_string().contains("invalid cell reference"));
    }

    #[test]
    fn start_hyperlink_required_propagates_invalid_reference_errors() {
        // 对应 Java：严格模式 ref 区域乱序同样报错
        let mut handler = HyperlinkTagHandler::new(true);
        let error = handler
            .start_hyperlink_required(&attrs(&[("ref", "B2:A1"), ("id", "rId1")]), &resolve_ok)
            .expect_err("reversed ref must fail in required mode");
        assert!(error.to_string().contains("invalid cell range ordering"));
    }

    #[test]
    fn disabled_handler_skips_every_required_branch() -> Result<()> {
        // 对应 Java：support()=false 时严格模式同样直接返回
        let mut handler = HyperlinkTagHandler::new(false);
        handler.start_hyperlink_required(
            &attrs(&[("ref", "A1"), ("location", "local!A1")]),
            &resolve_ok,
        )?;
        assert!(handler.last_extra.is_none());
        Ok(())
    }
}
