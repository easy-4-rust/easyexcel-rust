//! XLSX 图片锚点坐标与像素几何计算。

use easyexcel_io::{Error, Result};

use super::{MAX_XLSX_COLUMN_NUMBER, MAX_XLSX_ROW_NUMBER};

/// 相对当前单元格或绝对指定的锚点坐标。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AnchorCoordinate {
    /// 非零绝对坐标；零值按未指定处理，以匹配 EasyExcel/POI 语义。
    pub absolute: Option<u32>,
    /// 相对当前坐标的有符号偏移。
    pub relative: Option<i32>,
}

/// 图片锚点输入参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageAnchorSpec {
    /// 当前单元格行号。
    pub current_row: u32,
    /// 当前单元格列号。
    pub current_column: u16,
    /// 首行坐标。
    pub first_row: AnchorCoordinate,
    /// 首列坐标。
    pub first_column: AnchorCoordinate,
    /// 尾行坐标。
    pub last_row: AnchorCoordinate,
    /// 尾列坐标。
    pub last_column: AnchorCoordinate,
    /// 左侧像素边距。
    pub left: u32,
    /// 右侧像素边距。
    pub right: u32,
    /// 顶部像素边距。
    pub top: u32,
    /// 底部像素边距。
    pub bottom: u32,
}

/// 已校验、可直接交给 XLSX 写入后端的图片锚点。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedImageAnchor {
    /// 首行坐标。
    pub first_row: u32,
    /// 首列坐标。
    pub first_column: u16,
    /// 扣除左右边距后的图片宽度。
    pub width: u32,
    /// 扣除上下边距后的图片高度。
    pub height: u32,
    /// 左侧像素偏移。
    pub left: u32,
    /// 顶部像素偏移。
    pub top: u32,
}

/// 解析图片锚点并按行列尺寸计算可用像素区域。
///
/// # Errors
///
/// 坐标越界、起点晚于终点、像素求和溢出或边距耗尽锚点时返回 XLSX
/// 格式错误。
pub fn resolve_image_anchor<ColumnWidth, RowHeight>(
    spec: ImageAnchorSpec,
    mut column_width: ColumnWidth,
    mut row_height: RowHeight,
) -> Result<ResolvedImageAnchor>
where
    ColumnWidth: FnMut(u16) -> u32,
    RowHeight: FnMut(u32) -> u32,
{
    let first_row = resolve_coordinate(spec.current_row, spec.first_row, "first row")?;
    let first_column = resolve_coordinate(
        u32::from(spec.current_column),
        spec.first_column,
        "first column",
    )?;
    let last_row = resolve_coordinate(spec.current_row, spec.last_row, "last row")?;
    let last_column = resolve_coordinate(
        u32::from(spec.current_column),
        spec.last_column,
        "last column",
    )?;
    if first_row > last_row || first_column > last_column {
        return Err(Error::Xlsx(
            "image anchor start must not follow its end".to_owned(),
        ));
    }
    if last_row >= MAX_XLSX_ROW_NUMBER
        || usize::try_from(last_column).unwrap_or(usize::MAX) >= MAX_XLSX_COLUMN_NUMBER
    {
        return Err(Error::Xlsx(
            "image anchor exceeds XLSX worksheet limits".to_owned(),
        ));
    }
    let first_column = u16::try_from(first_column)
        .map_err(|_| Error::Xlsx("image anchor column exceeds XLSX limit".to_owned()))?;
    let last_column = u16::try_from(last_column)
        .map_err(|_| Error::Xlsx("image anchor column exceeds XLSX limit".to_owned()))?;

    let total_width = (first_column..=last_column).try_fold(0_u32, |width, column| {
        width
            .checked_add(column_width(column))
            .ok_or_else(|| Error::Xlsx("image anchor width overflow".to_owned()))
    })?;
    let total_height = (first_row..=last_row).try_fold(0_u32, |height, row| {
        height
            .checked_add(row_height(row))
            .ok_or_else(|| Error::Xlsx("image anchor height overflow".to_owned()))
    })?;
    let width = total_width
        .checked_sub(spec.left)
        .and_then(|value| value.checked_sub(spec.right))
        .filter(|value| *value > 0)
        .ok_or_else(|| Error::Xlsx("image horizontal margins consume its anchor".to_owned()))?;
    let height = total_height
        .checked_sub(spec.top)
        .and_then(|value| value.checked_sub(spec.bottom))
        .filter(|value| *value > 0)
        .ok_or_else(|| Error::Xlsx("image vertical margins consume its anchor".to_owned()))?;

    Ok(ResolvedImageAnchor {
        first_row,
        first_column,
        width,
        height,
        left: spec.left,
        top: spec.top,
    })
}

fn resolve_coordinate(current: u32, coordinate: AnchorCoordinate, label: &str) -> Result<u32> {
    if let Some(absolute) = coordinate.absolute.filter(|value| *value > 0) {
        return Ok(absolute);
    }
    let Some(relative) = coordinate.relative else {
        return Ok(current);
    };
    current.checked_add_signed(relative).ok_or_else(|| {
        Error::Xlsx(format!(
            "image anchor {label} is outside the worksheet"
        ))
    })
}
