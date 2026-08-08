// 将 handler 的后端中立修改计划应用到 XLSX 工作簿。

use crate::context::write_mutation::WriteMutation;
use crate::context::write_mutation_plan::WriteMutationPlan;

/// 编译模板集合填充所需的注解和 handler 样式增量。
pub(crate) fn compile_template_fill_styles<T>(
    options: &WriteOptions,
    handlers: &mut [Box<dyn WriteHandler>],
) -> Result<Option<crate::template::CompiledTemplateFillStyles>>
where
    T: ExcelRow,
{
    let schema_columns = selected_columns(T::schema(), options)?;
    let mut physical_columns = template_fill_columns(options)?;
    physical_columns.extend(schema_columns.iter().map(|(index, _, _)| *index));
    physical_columns.sort_unstable();
    physical_columns.dedup();
    if physical_columns.is_empty() {
        return Ok(None);
    }
    sort_handlers(handlers);
    let metadata = T::write_metadata();
    let global = WriteGlobalFlags::from(options);
    let mut formats = Vec::new();
    let mut styled_columns = Vec::new();
    for physical_index in physical_columns {
        let column = schema_columns
            .iter()
            .find(|(index, _, _)| *index == physical_index)
            .map(|(_, _, column)| *column);
        let annotation_cell = column
            .and_then(|column| column.content_style)
            .or(metadata.content_style);
        let annotation_font = column
            .and_then(|column| column.content_font_style)
            .or(metadata.content_font_style);
        let converted_data_format = column.and_then(template_fill_data_format);
        let mut context = WriteCellContext::new(
            &options.sheet_name,
            0,
            to_column(physical_index)?,
            CellValue::Empty,
        );
        if let Some(column) = column {
            context = context.with_column(column);
        }
        let handler_cell = collect_handler_cell_style(handlers, &context);
        if annotation_cell.is_none()
            && annotation_font.is_none()
            && handler_cell.is_none()
            && converted_data_format.is_none()
        {
            continue;
        }
        formats.push(cell_format(CellFormatContext {
            explicit: None,
            cell: annotation_cell,
            font: annotation_font,
            handler_cell,
            converted_cell: None,
            converted_data_format,
            ignore_fill_style: false,
            global,
        }));
        styled_columns.push(physical_index);
    }
    if formats.is_empty() {
        return Ok(None);
    }
    let mut compiler = generation::new_workbook();
    let worksheet = compiler.add_worksheet();
    for (index, format) in formats.iter().enumerate() {
        let row = u32::try_from(index)
            .map_err(|_| ExcelError::Format("too many template fill styles".to_owned()))?;
        generation::write_blank(worksheet, row, 0, format).map_err(format_error)?;
    }
    let workbook = generation::serialize_workbook(&mut compiler).map_err(ExcelError::from)?;
    Ok(Some(crate::template::CompiledTemplateFillStyles {
        workbook,
        columns: styled_columns,
    }))
}

fn template_fill_data_format(column: &ExcelColumn) -> Option<&'static str> {
    if let Some(format) = column.effective_date_time_format() {
        return Some(format);
    }
    let field_type = column.field_type?;
    if field_type.contains("NaiveDateTime") {
        Some(crate::converters::date_support::DEFAULT_DATETIME_FORMAT)
    } else if field_type.contains("NaiveDate") {
        Some(crate::converters::date_support::DEFAULT_DATE_FORMAT)
    } else {
        column.effective_number_format()
    }
}

fn template_fill_columns(options: &WriteOptions) -> Result<Vec<usize>> {
    let bytes = if let Some(bytes) = &options.template_bytes {
        bytes.clone()
    } else if let Some(path) = &options.template_file {
        std::fs::read(path)?
    } else {
        return Ok(Vec::new());
    };
    let package = easyexcel_xlsx::OoxmlTemplatePackage::from_bytes(&bytes)
        .map_err(ExcelError::from)?;
    let worksheet = if let Some(index) = options.sheet_index {
        package
            .worksheet_path_by_index(index)
            .map(|(_, path)| path)
            .map_err(ExcelError::from)?
    } else {
        package
            .worksheet_path_by_name(&options.sheet_name)
            .or_else(|_| package.worksheet_path_by_index(0).map(|(_, path)| path))
            .map_err(ExcelError::from)?
    };
    let entries = package.into_package().into_entries();
    Ok(easyexcel_xlsx::collection_column_style_indexes(
        &entries,
        &worksheet,
    )
    .into_keys()
    .collect())
}

/// 在序列化之前执行 Java handler 请求的工作簿修改。
pub(crate) fn apply_xlsx_mutations(
    workbook: &mut Workbook,
    plan: &WriteMutationPlan,
) -> Result<()> {
    for mutation in plan.snapshot()? {
        match mutation {
            WriteMutation::SetCell {
                sheet_name,
                row_index,
                column_index,
                value,
            } => {
                let worksheet = workbook
                    .worksheet_from_name(&sheet_name)
                    .map_err(format_error)?;
                write_mutation_cell(worksheet, row_index, column_index, &value)?;
            }
            WriteMutation::ProtectSheet {
                sheet_name,
                password,
            } => {
                let worksheet = workbook
                    .worksheet_from_name(&sheet_name)
                    .map_err(format_error)?;
                worksheet.protect_with_password(&password);
            }
            WriteMutation::AddChart(chart) => add_chart(workbook, &chart)?,
            // `rust_xlsxwriter::merge_range` 会重写左上角单元格；Java
            // `Sheet.addMergedRegion` 不会。合并区域因此在工作簿序列化后由
            // OOXML package 层只修改 `<mergeCells>` 元数据。
            WriteMutation::AddMerge { .. } => {}
        }
    }
    Ok(())
}

fn add_chart(workbook: &mut Workbook, mutation: &crate::ChartMutation) -> Result<()> {
    if mutation.series.is_empty() {
        return Err(ExcelError::Format(
            "chart mutation requires at least one data series".to_owned(),
        ));
    }
    if mutation.last_row < mutation.first_row || mutation.last_column < mutation.first_column {
        return Err(ExcelError::Format(
            "chart mutation anchor end must not precede its start".to_owned(),
        ));
    }
    let chart_type = match mutation.chart_type {
        crate::ChartType::Bar => generation::ChartType::Bar,
        crate::ChartType::Line => generation::ChartType::Line,
        crate::ChartType::Pie => generation::ChartType::Pie,
    };
    let mut chart = generation::Chart::new(chart_type);
    if let Some(title) = &mutation.title {
        chart.title().set_name(title);
    }
    for source in &mutation.series {
        validate_chart_range(&source.values)?;
        let series = chart.add_series().set_values((
            source.values.sheet_name.as_str(),
            source.values.first_row,
            source.values.first_column,
            source.values.last_row,
            source.values.last_column,
        ));
        if let Some(categories) = &source.categories {
            validate_chart_range(categories)?;
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
        .map_err(format_error)?;
    let width = u32::from(mutation.last_column - mutation.first_column + 1).saturating_mul(64);
    let height = (mutation.last_row - mutation.first_row + 1).saturating_mul(20);
    chart.set_width(width).set_height(height);
    worksheet
        .insert_chart(mutation.first_row, mutation.first_column, &chart)
        .map(|_| ())
        .map_err(format_error)
}

fn validate_chart_range(range: &crate::ChartRange) -> Result<()> {
    if range.last_row < range.first_row || range.last_column < range.first_column {
        return Err(ExcelError::Format(format!(
            "chart range on sheet '{}' has an end before its start",
            range.sheet_name
        )));
    }
    Ok(())
}

fn write_mutation_cell(
    worksheet: &mut Worksheet,
    row_index: u32,
    column_index: u16,
    value: &CellValue,
) -> Result<()> {
    match value {
        CellValue::Empty => generation::write_blank(
            worksheet,
            row_index,
            column_index,
            &generation::new_format(),
        )
        .map_err(format_error),
        CellValue::String(value) | CellValue::Error(value) => {
            generation::write_string(worksheet, row_index, column_index, value)
                .map_err(format_error)
        }
        CellValue::Bool(value) => {
            generation::write_boolean(worksheet, row_index, column_index, *value)
                .map_err(format_error)
        }
        CellValue::Int(value) => {
            generation::write_integer(worksheet, row_index, column_index, *value, None)
                .map_err(format_error)
        }
        CellValue::Float(value) => {
            generation::write_number(worksheet, row_index, column_index, *value)
                .map_err(format_error)
        }
        CellValue::Decimal(value) => {
            let number = value.to_string().parse::<f64>().map_err(|error| {
                ExcelError::Format(format!("cannot write decimal mutation {value}: {error}"))
            })?;
            generation::write_number(worksheet, row_index, column_index, number)
                .map_err(format_error)
        }
        CellValue::Formula(value) => {
            generation::write_formula(worksheet, row_index, column_index, value)
                .map_err(format_error)
        }
        CellValue::Date(value) => generation::write_date_with_format(
            worksheet,
            row_index,
            column_index,
            *value,
            &generation::new_format(),
        )
        .map_err(format_error),
        CellValue::DateTime(value) => generation::write_datetime_with_format(
            worksheet,
            row_index,
            column_index,
            *value,
            &generation::new_format(),
        )
        .map_err(format_error),
        CellValue::Hyperlink { text, .. } | CellValue::HyperlinkWithMetadata { text, .. } => {
            generation::write_string(worksheet, row_index, column_index, text)
                .map_err(format_error)
        }
        CellValue::Comment { value, .. } | CellValue::Images { value, .. } => {
            write_mutation_cell(worksheet, row_index, column_index, value)
        }
        CellValue::RichText(value) => generation::write_string(
            worksheet,
            row_index,
            column_index,
            value.text_string(),
        )
        .map_err(format_error),
        CellValue::Image(_) => Err(ExcelError::Unsupported(
            "workbook handler image mutations require an explicit image anchor".to_owned(),
        )),
    }
}

#[cfg(test)]
mod chart_mutation_tests {
    use std::io::{Cursor, Read};

    use zip::ZipArchive;

    use super::*;
    use crate::{ChartMutation, ChartRange, ChartSeries, ChartType};

    #[test]
    fn applies_backend_neutral_bar_chart_to_xlsx() {
        let mut workbook = generation::new_workbook();
        let worksheet = workbook.add_worksheet();
        worksheet.set_name("Data").expect("sheet name");
        for row in 0..3 {
            generation::write_string(worksheet, row, 0, format!("C{row}"))
                .expect("category");
            generation::write_number(worksheet, row, 1, f64::from(row + 1)).expect("value");
        }

        let plan = WriteMutationPlan::default();
        plan.add_chart(
            ChartMutation::new("Data", ChartType::Bar, 0, 3, 14, 10)
                .with_title("Sales")
                .with_series(
                    ChartSeries::new(ChartRange::new("Data", 0, 1, 2, 1))
                        .with_name("Amount")
                        .with_categories(ChartRange::new("Data", 0, 0, 2, 0)),
                ),
        )
        .expect("queue chart");
        apply_xlsx_mutations(&mut workbook, &plan).expect("apply chart");

        let bytes = generation::serialize_workbook(&mut workbook).expect("serialize");
        let mut archive = ZipArchive::new(Cursor::new(bytes)).expect("xlsx zip");
        let mut xml = String::new();
        archive
            .by_name("xl/charts/chart1.xml")
            .expect("chart part")
            .read_to_string(&mut xml)
            .expect("chart xml");
        assert!(xml.contains("<c:barChart>"));
        assert!(xml.contains("Sales"));
        assert!(xml.contains("Data!$A$1:$A$3"));
        assert!(xml.contains("Data!$B$1:$B$3"));
    }

    #[test]
    fn rejects_empty_chart_series() {
        let mut workbook = generation::new_workbook();
        workbook
            .add_worksheet()
            .set_name("Data")
            .expect("sheet name");
        let plan = WriteMutationPlan::default();
        plan.add_chart(ChartMutation::new("Data", ChartType::Line, 0, 0, 10, 8))
            .expect("queue chart");
        assert!(matches!(
            apply_xlsx_mutations(&mut workbook, &plan),
            Err(ExcelError::Format(message)) if message.contains("at least one")
        ));
    }
}
