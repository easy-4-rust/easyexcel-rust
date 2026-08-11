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

#[cfg(test)]
mod template_hyperlink_tests {
    use super::*;

    /// 验证 `new` 创建默认坐标均为零值的超链接。
    #[test]
    fn new_creates_hyperlink_with_default_coordinates() {
        let link = TemplateHyperlink::new("https://example.com", TemplateHyperlinkType::Url);
        assert_eq!(link.address, "https://example.com");
        assert_eq!(link.hyperlink_type, TemplateHyperlinkType::Url);
        assert_eq!(link.first_row, TemplateHyperlinkCoordinate::default());
        assert_eq!(link.first_column, TemplateHyperlinkCoordinate::default());
        assert_eq!(link.last_row, TemplateHyperlinkCoordinate::default());
        assert_eq!(link.last_column, TemplateHyperlinkCoordinate::default());
    }

    /// 验证默认坐标解析为单个单元格引用。
    #[test]
    fn resolve_reference_single_cell_with_default_coordinates() {
        let link = TemplateHyperlink::new("https://example.com", TemplateHyperlinkType::Url);
        let result = link.resolve_reference(5, 3).unwrap();
        // row=5, col=3 → D6
        assert_eq!(result, "D6");
    }

    /// 验证绝对坐标优先于当前行/列。
    #[test]
    fn resolve_reference_absolute_coordinates_override_current() {
        let link = TemplateHyperlink {
            address: "https://example.com".to_owned(),
            hyperlink_type: TemplateHyperlinkType::Url,
            first_row: TemplateHyperlinkCoordinate {
                absolute: Some(1),
                relative: None,
            },
            first_column: TemplateHyperlinkCoordinate {
                absolute: Some(1),
                relative: None,
            },
            last_row: TemplateHyperlinkCoordinate {
                absolute: Some(3),
                relative: None,
            },
            last_column: TemplateHyperlinkCoordinate {
                absolute: Some(4),
                relative: None,
            },
        };
        let result = link.resolve_reference(99, 99).unwrap();
        // 绝对坐标 (1,1)→B2 到 (3,4)→E4
        assert_eq!(result, "B2:E4");
    }

    /// 验证相对偏移正确累加到当前行/列。
    #[test]
    fn resolve_reference_relative_offset_adds_to_current() {
        let link = TemplateHyperlink {
            address: "test".to_owned(),
            hyperlink_type: TemplateHyperlinkType::Url,
            first_row: TemplateHyperlinkCoordinate {
                absolute: None,
                relative: Some(1),
            },
            first_column: TemplateHyperlinkCoordinate {
                absolute: None,
                relative: Some(2),
            },
            last_row: TemplateHyperlinkCoordinate {
                absolute: None,
                relative: Some(1),
            },
            last_column: TemplateHyperlinkCoordinate {
                absolute: None,
                relative: Some(2),
            },
        };
        // current row=10, col=5 → 加偏移 (1,2) → row=11, col=7 → H12
        let result = link.resolve_reference(10, 5).unwrap();
        assert_eq!(result, "H12");
    }

    /// 验证范围不合法（start > end）时返回错误。
    #[test]
    fn resolve_reference_start_after_end_returns_error() {
        let link = TemplateHyperlink {
            address: "test".to_owned(),
            hyperlink_type: TemplateHyperlinkType::Url,
            first_row: TemplateHyperlinkCoordinate {
                absolute: Some(5),
                relative: None,
            },
            first_column: TemplateHyperlinkCoordinate {
                absolute: Some(5),
                relative: None,
            },
            last_row: TemplateHyperlinkCoordinate {
                absolute: Some(3),
                relative: None,
            },
            last_column: TemplateHyperlinkCoordinate {
                absolute: Some(3),
                relative: None,
            },
        };
        let result = link.resolve_reference(0, 0);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("start must not follow its end")
        );
    }

    /// 验证坐标超出 XLSX 行数上限时返回错误。
    #[test]
    fn resolve_reference_exceeds_row_limit_returns_error() {
        let max_row = 1_048_576u32; // MAX_XLSX_ROW_NUMBER
        let link = TemplateHyperlink {
            address: "test".to_owned(),
            hyperlink_type: TemplateHyperlinkType::Url,
            first_row: TemplateHyperlinkCoordinate {
                absolute: Some(max_row),
                relative: None,
            },
            first_column: TemplateHyperlinkCoordinate::default(),
            last_row: TemplateHyperlinkCoordinate {
                absolute: Some(max_row),
                relative: None,
            },
            last_column: TemplateHyperlinkCoordinate::default(),
        };
        let result = link.resolve_reference(0, 0);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("exceeds XLSX worksheet limits")
        );
    }

    /// 验证坐标超出 XLSX 列数上限时返回错误。
    #[test]
    fn resolve_reference_exceeds_column_limit_returns_error() {
        let max_col = 16_384u32; // MAX_XLSX_COLUMN_NUMBER
        let link = TemplateHyperlink {
            address: "test".to_owned(),
            hyperlink_type: TemplateHyperlinkType::Url,
            first_row: TemplateHyperlinkCoordinate::default(),
            first_column: TemplateHyperlinkCoordinate {
                absolute: Some(max_col),
                relative: None,
            },
            last_row: TemplateHyperlinkCoordinate::default(),
            last_column: TemplateHyperlinkCoordinate {
                absolute: Some(max_col),
                relative: None,
            },
        };
        let result = link.resolve_reference(0, 0);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("exceeds XLSX worksheet limits")
        );
    }

    /// 验证相对偏移导致溢出时返回错误。
    #[test]
    fn resolve_reference_relative_overflow_returns_error() {
        let link = TemplateHyperlink {
            address: "test".to_owned(),
            hyperlink_type: TemplateHyperlinkType::Url,
            first_row: TemplateHyperlinkCoordinate {
                absolute: None,
                relative: Some(-200),
            },
            first_column: TemplateHyperlinkCoordinate::default(),
            last_row: TemplateHyperlinkCoordinate::default(),
            last_column: TemplateHyperlinkCoordinate::default(),
        };
        // current row=10, -200 溢出
        let result = link.resolve_reference(10, 0);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("outside the worksheet")
        );
    }

    /// 验证 `resolve_coordinate` 的绝对值零回退到相对/当前。
    #[test]
    fn resolve_coordinate_zero_absolute_falls_back_to_relative() {
        // absolute=0 被 filter 掉（>0 条件不满足），回退到 relative 或 current
        let coord = TemplateHyperlinkCoordinate {
            absolute: Some(0),
            relative: Some(5),
        };
        let result = resolve_coordinate(10, coord, "test").unwrap();
        // relative=5 → current(10) + 5 = 15
        assert_eq!(result, 15);
    }

    /// 验证 `resolve_coordinate` 在无 absolute、无 relative 时返回 current。
    #[test]
    fn resolve_coordinate_default_returns_current() {
        let coord = TemplateHyperlinkCoordinate::default();
        let result = resolve_coordinate(42, coord, "test").unwrap();
        assert_eq!(result, 42);
    }

    /// 验证两坐标相同时解析为单单元格。
    #[test]
    fn resolve_reference_same_first_and_last_returns_single_cell() {
        let link = TemplateHyperlink {
            address: "test".to_owned(),
            hyperlink_type: TemplateHyperlinkType::Url,
            first_row: TemplateHyperlinkCoordinate {
                absolute: Some(1),
                relative: None,
            },
            first_column: TemplateHyperlinkCoordinate {
                absolute: Some(1),
                relative: None,
            },
            last_row: TemplateHyperlinkCoordinate {
                absolute: Some(1),
                relative: None,
            },
            last_column: TemplateHyperlinkCoordinate {
                absolute: Some(1),
                relative: None,
            },
        };
        let result = link.resolve_reference(0, 0).unwrap();
        assert_eq!(result, "B2");
    }
}
