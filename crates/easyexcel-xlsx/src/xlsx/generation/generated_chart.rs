use easyexcel_io::{Error, Result};
use easyexcel_model::{ChartMutation, ChartRange, ChartType as ModelChartType};

use super::{Chart, ChartType, Workbook};

/// 将后端中立图表请求应用到生成式 XLSX 工作簿。
///
/// # Errors
///
/// 系列为空、锚点/区域倒置、目标工作表不存在或后端拒绝图表时返回错误。
pub fn add_chart(workbook: &mut Workbook, mutation: &ChartMutation) -> Result<()> {
    if mutation.series.is_empty() {
        return Err(Error::Xlsx(
            "chart mutation requires at least one data series".to_owned(),
        ));
    }
    if mutation.last_row < mutation.first_row || mutation.last_column < mutation.first_column {
        return Err(Error::Xlsx(
            "chart mutation anchor end must not precede its start".to_owned(),
        ));
    }
    let chart_type = match mutation.chart_type {
        ModelChartType::Bar => ChartType::Bar,
        ModelChartType::Line => ChartType::Line,
        ModelChartType::Pie => ChartType::Pie,
    };
    let mut chart = Chart::new(chart_type);
    if let Some(title) = &mutation.title {
        chart.title().set_name(title);
    }
    for source in &mutation.series {
        validate_range(&source.values)?;
        let series = chart.add_series().set_values((
            source.values.sheet_name.as_str(),
            source.values.first_row,
            source.values.first_column,
            source.values.last_row,
            source.values.last_column,
        ));
        if let Some(categories) = &source.categories {
            validate_range(categories)?;
            series.set_categories((
                categories.sheet_name.as_str(),
                categories.first_row,
                categories.first_column,
                categories.last_row,
                categories.last_column,
            ));
        }
        if let Some(name) = &source.name {
            series.set_name(name.as_str());
        }
    }

    let worksheet = workbook
        .worksheet_from_name(&mutation.sheet_name)
        .map_err(|error| Error::Xlsx(error.to_string()))?;
    let width = u32::from(mutation.last_column - mutation.first_column + 1).saturating_mul(64);
    let height = (mutation.last_row - mutation.first_row + 1).saturating_mul(20);
    chart.set_width(width).set_height(height);
    worksheet
        .insert_chart(mutation.first_row, mutation.first_column, &chart)
        .map(|_| ())
        .map_err(|error| Error::Xlsx(error.to_string()))
}

fn validate_range(range: &ChartRange) -> Result<()> {
    if range.last_row < range.first_row || range.last_column < range.first_column {
        return Err(Error::Xlsx(format!(
            "chart range on sheet '{}' has an end before its start",
            range.sheet_name
        )));
    }
    Ok(())
}
