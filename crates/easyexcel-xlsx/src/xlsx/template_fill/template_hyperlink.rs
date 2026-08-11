// 模板超链接元数据结构体。
// 对应 Java：无直接对应对象；Rust XLSX 引擎扩展。
// 拆分后仅保留 `TemplateHyperlink` 结构体及其私有辅助函数；
// `TemplateHyperlinkType` 和 `TemplateHyperlinkCoordinate` 分别位于
// 同级 `template_hyperlink_type.rs` 和 `template_hyperlink_coordinate.rs`。

/// 对应 Java：无直接对应对象；Rust XLSX 引擎扩展。模板超链接元数据。
///
/// 描述一个模板超链接的地址、类型以及在工作表中的覆盖范围。
/// 覆盖范围通过四个坐标（起始行/列、结束行/列）描述，支持绝对和相对模式。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateHyperlink {
    /// Java `HyperlinkData.address`。
    pub address: String,
    /// 超链接类别。
    pub hyperlink_type: TemplateHyperlinkType,
    /// 覆盖范围起始行。
    pub first_row: TemplateHyperlinkCoordinate,
    /// 覆盖范围起始列。
    pub first_column: TemplateHyperlinkCoordinate,
    /// 覆盖范围结束行。
    pub last_row: TemplateHyperlinkCoordinate,
    /// 覆盖范围结束列。
    pub last_column: TemplateHyperlinkCoordinate,
}

impl TemplateHyperlink {
    /// 创建仅覆盖当前单元格的超链接。
    ///
    /// # 参数
    /// - `address`: 超链接地址。
    /// - `hyperlink_type`: 超链接类别。
    ///
    /// # 返回
    /// 新建的模板超链接，默认坐标均为零值（覆盖当前单元格）。
    #[must_use]
    pub fn new(address: impl Into<String>, hyperlink_type: TemplateHyperlinkType) -> Self {
        Self {
            address: address.into(),
            hyperlink_type,
            first_row: TemplateHyperlinkCoordinate::default(),
            first_column: TemplateHyperlinkCoordinate::default(),
            last_row: TemplateHyperlinkCoordinate::default(),
            last_column: TemplateHyperlinkCoordinate::default(),
        }
    }

    /// 按 EasyExcel 绝对优先、零值回退相对坐标规则解析 A1 覆盖范围。
    ///
    /// # 参数
    /// - `row`: 当前行号（零基）。
    /// - `column`: 当前列号（零基）。
    ///
    /// # 返回
    /// A1 格式的范围字符串（如 `A1` 或 `A1:B2`）。
    ///
    /// # 错误
    /// 坐标超出 XLSX 工作表限制时返回错误。
    pub(crate) fn resolve_reference(&self, row: u32, column: u16) -> easyexcel_io::Result<String> {
        let first_row = resolve_coordinate(row, self.first_row, "first row")?;
        let first_column = resolve_coordinate(
            u32::from(column),
            self.first_column,
            "first column",
        )?;
        let last_row = resolve_coordinate(row, self.last_row, "last row")?;
        let last_column = resolve_coordinate(
            u32::from(column),
            self.last_column,
            "last column",
        )?;
        if first_row > last_row || first_column > last_column {
            return Err(easyexcel_io::Error::Xlsx(
                "template hyperlink start must not follow its end".to_owned(),
            ));
        }
        if first_row >= super::MAX_XLSX_ROW_NUMBER
            || last_row >= super::MAX_XLSX_ROW_NUMBER
            || usize::try_from(first_column).unwrap_or(usize::MAX)
                >= super::MAX_XLSX_COLUMN_NUMBER
            || usize::try_from(last_column).unwrap_or(usize::MAX)
                >= super::MAX_XLSX_COLUMN_NUMBER
        {
            return Err(easyexcel_io::Error::Xlsx(
                "template hyperlink range exceeds XLSX worksheet limits".to_owned(),
            ));
        }
        let first = format!(
            "{}{}",
            column_name(usize::try_from(first_column).unwrap_or(usize::MAX) + 1),
            first_row + 1
        );
        let last = format!(
            "{}{}",
            column_name(usize::try_from(last_column).unwrap_or(usize::MAX) + 1),
            last_row + 1
        );
        Ok(if first == last {
            first
        } else {
            format!("{first}:{last}")
        })
    }
}

fn resolve_coordinate(
    current: u32,
    coordinate: TemplateHyperlinkCoordinate,
    label: &str,
) -> easyexcel_io::Result<u32> {
    if let Some(absolute) = coordinate.absolute.filter(|value| *value > 0) {
        return Ok(absolute);
    }
    let Some(relative) = coordinate.relative else {
        return Ok(current);
    };
    current.checked_add_signed(relative).ok_or_else(|| {
        easyexcel_io::Error::Xlsx(format!(
            "template hyperlink {label} is outside the worksheet"
        ))
    })
}
