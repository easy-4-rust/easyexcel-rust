/// 对应 Java：无直接对应对象；Rust XLSX 引擎扩展。模板超链接类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateHyperlinkType {
    /// 普通 URL。
    Url,
    /// 工作簿内部位置。
    Document,
    /// 邮件地址。
    Email,
    /// 外部文件。
    File,
}

impl TemplateHyperlinkType {
    /// 将调用方地址规范化为生成式 XLSX 后端接受的目标。
    ///
    /// 工作簿内部、邮件与外部文件链接分别使用 `internal:`、`mailto:` 与
    /// `external:` 前缀；已经带有前缀的地址保持不变。
    #[must_use]
    pub fn generation_target(self, address: &str) -> String {
        match self {
            Self::Url => address.to_owned(),
            Self::Document if address.starts_with("internal:") => address.to_owned(),
            Self::Document => format!("internal:{address}"),
            Self::Email if address.to_ascii_lowercase().starts_with("mailto:") => {
                address.to_owned()
            }
            Self::Email => format!("mailto:{address}"),
            Self::File if address.starts_with("external:") => address.to_owned(),
            Self::File => format!("external:{address}"),
        }
    }

    /// 将地址规范化为 OOXML relationship/location 中保存的目标。
    #[must_use]
    pub fn package_target(self, address: &str) -> String {
        match self {
            Self::Document => address
                .strip_prefix("internal:")
                .unwrap_or(address)
                .to_owned(),
            Self::Email if !address.to_ascii_lowercase().starts_with("mailto:") => {
                format!("mailto:{address}")
            }
            Self::File => address
                .strip_prefix("external:")
                .unwrap_or(address)
                .to_owned(),
            Self::Url | Self::Email => address.to_owned(),
        }
    }
}

/// 对应 Java：无直接对应对象；Rust XLSX 引擎扩展。绝对或相对坐标。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TemplateHyperlinkCoordinate {
    /// 大于零时优先使用的绝对零基坐标。
    pub absolute: Option<u32>,
    /// 相对当前填充单元格的偏移。
    pub relative: Option<i32>,
}

/// 对应 Java：无直接对应对象；Rust XLSX 引擎扩展。模板超链接元数据。
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
