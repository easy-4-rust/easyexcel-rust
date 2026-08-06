//! 与具体工作簿格式无关的绘图坐标换算。

/// Apache POI `Units.EMU_PER_POINT`。
pub const EMU_PER_POINT: i32 = 12_700;

/// 将可选点坐标转换为 EMU；缺失坐标按零处理。
#[must_use]
pub fn points_to_emu(coordinate: Option<i32>) -> i32 {
    coordinate.unwrap_or_default().saturating_mul(EMU_PER_POINT)
}

/// 按 Java EasyExcel 优先级解析单元格内坐标。
///
/// 正数绝对坐标优先；否则相对坐标叠加到当前坐标；两者均缺失时返回当前坐标。
#[must_use]
pub fn resolve_cell_coordinate(
    current_coordinate: i32,
    absolute_coordinate: Option<i32>,
    relative_coordinate: Option<i32>,
) -> i32 {
    if let Some(absolute_coordinate) = absolute_coordinate.filter(|coordinate| *coordinate > 0) {
        return absolute_coordinate;
    }
    relative_coordinate.map_or(current_coordinate, |relative_coordinate| {
        current_coordinate.saturating_add(relative_coordinate)
    })
}

#[cfg(test)]
mod tests {
    use super::{EMU_PER_POINT, points_to_emu, resolve_cell_coordinate};

    #[test]
    fn converts_points_and_resolves_coordinate_precedence() {
        assert_eq!(points_to_emu(None), 0);
        assert_eq!(points_to_emu(Some(2)), 2 * EMU_PER_POINT);
        assert_eq!(resolve_cell_coordinate(10, Some(20), Some(3)), 20);
        assert_eq!(resolve_cell_coordinate(10, Some(0), Some(3)), 13);
        assert_eq!(resolve_cell_coordinate(10, None, None), 10);
    }
}
