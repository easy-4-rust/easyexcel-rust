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
        }
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
        CellValue::Hyperlink { text, .. } => {
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
