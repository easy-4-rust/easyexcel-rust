/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn validate_excel_row_schema<T>() -> Result<()>
where
    T: ExcelRow,
{
    validate_schema(T::schema())
}

fn validate_schema(schema: &'static [ExcelColumn]) -> Result<()> {
    let mut indexed_fields = HashMap::new();
    for column in schema {
        let Some(index) = column.index else {
            continue;
        };
        if let Some(previous_field) = indexed_fields.insert(index, column.field) {
            return Err(ExcelError::Format(format!(
                "The index of '{previous_field}' and '{}' must be inconsistent",
                column.field
            )));
        }
    }
    Ok(())
}

fn ordered_columns(
    schema: &'static [ExcelColumn],
) -> Result<Vec<(usize, usize, &'static ExcelColumn)>> {
    validate_schema(schema)?;
    // Java `ClassUtils.buildSortedAllFieldMap` first groups non-indexed fields
    // by `@ExcelProperty.order` (preserving declaration order inside a group),
    // then inserts them into the first free physical indexes while skipping
    // every forced `@ExcelProperty.index`. Forced indexes are anchors, not a
    // secondary sort key.
    let forced_indexes = schema
        .iter()
        .filter_map(|column| column.index)
        .collect::<HashSet<_>>();
    let mut automatic = schema
        .iter()
        .enumerate()
        .filter(|(_, column)| column.index.is_none())
        .collect::<Vec<_>>();
    automatic.sort_by_key(|(schema_index, column)| (column.order, *schema_index));

    let mut columns = schema
        .iter()
        .enumerate()
        .filter_map(|(schema_index, column)| {
            column
                .index
                .map(|physical_index| (physical_index, schema_index, column))
        })
        .collect::<Vec<_>>();
    let mut next_automatic_index = 0usize;
    for (schema_index, column) in automatic {
        while forced_indexes.contains(&next_automatic_index) {
            next_automatic_index = next_automatic_index.saturating_add(1);
        }
        columns.push((next_automatic_index, schema_index, column));
        next_automatic_index = next_automatic_index.saturating_add(1);
    }
    columns.sort_by_key(|(physical_index, _, _)| *physical_index);
    Ok(columns)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn apply_annotation_column_widths<T>(
    worksheet: &mut Worksheet,
    options: &WriteOptions,
) -> Result<()>
where
    T: ExcelRow,
{
    let type_width = T::write_metadata().column_width;
    for (physical_index, _, column) in selected_columns(T::schema(), options)? {
        if options
            .column_widths
            .iter()
            .any(|(explicit, _)| usize::from(*explicit) == physical_index)
        {
            continue;
        }
        if let Some(width) = column.column_width.or(type_width) {
            set_xlsx_column_width_chars(worksheet, to_column(physical_index)?, width)?;
        }
    }
    Ok(())
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn apply_template_holder_layout<T>(
    package: &mut crate::write::template_write::TemplatePackage,
    sheet_name: &str,
    options: &WriteOptions,
    handlers: &[Box<dyn WriteHandler>],
    excluded_merges: &[crate::core::OnceAbsoluteMergeProperty],
) -> Result<()>
where
    T: ExcelRow,
{
    let explicit_columns = options
        .column_widths
        .iter()
        .map(|(column, _)| *column)
        .collect::<HashSet<_>>();
    let mut widths = options
        .column_widths
        .iter()
        .copied()
        .collect::<HashMap<_, _>>();
    let type_width = T::write_metadata().column_width;
    for (physical_index, _, column) in selected_columns(T::schema(), options)? {
        let physical_index = to_column(physical_index)?;
        if !explicit_columns.contains(&physical_index) {
            if let Some(width) = column.column_width.or(type_width) {
                widths.entry(physical_index).or_insert(width);
            }
            for handler in handlers {
                if let Some(width) = handler.style_column_width(usize::from(physical_index)) {
                    widths.insert(physical_index, width);
                }
            }
        }
    }
    let mut widths = widths.into_iter().collect::<Vec<_>>();
    widths.sort_unstable_by_key(|(column, _)| *column);

    let mut merges = Vec::new();
    if let Some(merge) = T::write_metadata().once_absolute_merge
        && !excluded_merges.contains(&merge)
        && let Some(range) = absolute_merge_range(merge)
    {
        merges.push(range);
    }
    for handler in handlers {
        if let Some(merge) = handler.style_once_absolute_merge()
            && !excluded_merges.contains(&merge)
            && let Some(range) = absolute_merge_range(merge)
            && !merges.contains(&range)
        {
            merges.push(range);
        }
    }
    package.apply_sheet_layout(sheet_name, &widths, &merges)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn collect_once_absolute_merges<T>(
    handlers: &[Box<dyn WriteHandler>],
) -> Vec<crate::core::OnceAbsoluteMergeProperty>
where
    T: ExcelRow,
{
    let mut merges = Vec::new();
    if let Some(merge) = T::write_metadata().once_absolute_merge {
        merges.push(merge);
    }
    for merge in collect_handler_once_absolute_merges(handlers) {
        if !merges.contains(&merge) {
            merges.push(merge);
        }
    }
    merges
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn collect_handler_once_absolute_merges(
    handlers: &[Box<dyn WriteHandler>],
) -> Vec<crate::core::OnceAbsoluteMergeProperty> {
    let mut merges = Vec::new();
    for handler in handlers {
        if let Some(merge) = handler.style_once_absolute_merge()
            && !merges.contains(&merge)
        {
            merges.push(merge);
        }
    }
    merges
}

fn absolute_merge_range(merge: crate::core::OnceAbsoluteMergeProperty) -> Option<MergeRange> {
    if merge.first_row_index < 0
        || merge.last_row_index < 0
        || merge.first_column_index < 0
        || merge.last_column_index < 0
    {
        return None;
    }
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    Some(MergeRange::new(
        merge.first_row_index as u32,
        merge.last_row_index as u32,
        merge.first_column_index as u16,
        merge.last_column_index as u16,
    ))
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Applies column widths from registered strategies
/// (Java `SimpleColumnWidthStyleStrategy` / `AbstractColumnWidthStyleStrategy`).
pub(crate) fn apply_handler_column_widths<T>(
    worksheet: &mut Worksheet,
    options: &WriteOptions,
    handlers: &[Box<dyn WriteHandler>],
) -> Result<()>
where
    T: ExcelRow,
{
    for (physical_index, _, _) in selected_columns(T::schema(), options)? {
        let column = to_column(physical_index)?;
        // Explicit `WriteOptions::column_widths` wins over strategies.
        if options
            .column_widths
            .iter()
            .any(|(explicit, _)| *explicit == column)
        {
            continue;
        }
        for handler in handlers {
            if let Some(width) = handler.style_column_width(physical_index) {
                set_xlsx_column_width_chars(worksheet, column, width)?;
            }
        }
    }
    Ok(())
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Collects head row height from registered strategies
/// (Java `SimpleRowHeightStyleStrategy`).
pub(crate) fn collect_handler_head_row_height(handlers: &[Box<dyn WriteHandler>]) -> Option<u16> {
    handlers
        .iter()
        .rev()
        .find_map(|handler| handler.style_head_row_height())
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Collects content row height from registered strategies
/// (Java `SimpleRowHeightStyleStrategy`).
pub(crate) fn collect_handler_content_row_height(
    handlers: &[Box<dyn WriteHandler>],
) -> Option<u16> {
    handlers
        .iter()
        .rev()
        .find_map(|handler| handler.style_content_row_height())
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Whether any handler requests longest-match autofit
/// (Java `LongestMatchColumnWidthStyleStrategy`).
pub(crate) fn handlers_request_auto_width(handlers: &[Box<dyn WriteHandler>]) -> bool {
    handlers
        .iter()
        .any(|handler| handler.style_auto_column_width())
}

/// Merges cell styles from registered style strategies in handler order
/// (Java `AbstractCellStyleStrategy.afterCellDispose` + `WriteCellStyle.merge`).
fn collect_handler_cell_style(
    handlers: &[Box<dyn WriteHandler>],
    context: &WriteCellContext,
) -> Option<ExcelCellStyle> {
    let mut merged: Option<ExcelCellStyle> = None;
    for handler in handlers {
        if let Some(style) = handler.style_cell_style(context) {
            merged = Some(match merged {
                Some(target) => merge_write_cell_style(&style, target),
                None => style,
            });
        }
    }
    merged
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Combines registered strategy styles with a mutation requested through the
/// logical POI-equivalent cell handle. The explicit handle request runs last,
/// matching a custom Java handler that mutates the POI cell in
/// `afterCellDispose`.
pub(crate) fn effective_handler_cell_style(
    handlers: &[Box<dyn WriteHandler>],
    context: &WriteCellContext,
) -> Option<ExcelCellStyle> {
    let merged = collect_handler_cell_style(handlers, context);
    context
        .cell()
        .requested_style()
        .map_or(merged, |requested| {
            Some(match merged {
                Some(current) => merge_write_cell_style(&requested, current),
                None => requested,
            })
        })
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Applies type-level `@OnceAbsoluteMerge` metadata when all indexes are non-negative.
pub(crate) fn apply_annotation_once_absolute_merge<T>(
    worksheet: &mut Worksheet,
    handlers: &[Box<dyn WriteHandler>],
) -> Result<()>
where
    T: ExcelRow,
{
    let Some(merge) = T::write_metadata().once_absolute_merge else {
        return Ok(());
    };
    if handlers
        .iter()
        .any(|handler| handler.style_once_absolute_merge() == Some(merge))
    {
        return Ok(());
    }
    apply_once_absolute_merge_property(worksheet, merge)
}

/// Applies registered [`OnceAbsoluteMergeStrategy`] regions
/// (Java `OnceAbsoluteMergeStrategy.afterSheetCreate` → `addMergedRegionUnsafe`).
fn apply_handler_once_absolute_merge(
    worksheet: &mut Worksheet,
    handlers: &[Box<dyn WriteHandler>],
) -> Result<()> {
    for handler in handlers {
        if let Some(merge) = handler.style_once_absolute_merge() {
            apply_once_absolute_merge_property(worksheet, merge)?;
        }
    }
    Ok(())
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Shared absolute-merge apply used by annotation and registered strategy paths.
pub(crate) fn apply_once_absolute_merge_property(
    worksheet: &mut Worksheet,
    merge: crate::core::OnceAbsoluteMergeProperty,
) -> Result<()> {
    if merge.first_row_index < 0
        || merge.last_row_index < 0
        || merge.first_column_index < 0
        || merge.last_column_index < 0
    {
        return Ok(());
    }
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    generation::merge_range(
        worksheet,
        merge.first_row_index as u32,
        merge.first_column_index as u16,
        merge.last_row_index as u32,
        merge.last_column_index as u16,
        "",
        &generation::new_format(),
    )
    .map_err(format_error)?;
    Ok(())
}

/// Builds loop-merge strategies from field-level `@ContentLoopMerge` metadata.
fn annotation_loop_merges_from_columns(
    columns: &[(usize, usize, &'static ExcelColumn)],
) -> Result<Vec<MirroredLoopMergeStrategy>> {
    let mut strategies = Vec::new();
    for (physical_index, _, column) in columns {
        let Some(property) = column.loop_merge else {
            continue;
        };
        strategies.push(MirroredLoopMergeStrategy::new(
            property.each_row,
            property.column_extend,
            to_column(*physical_index)?,
        )?);
    }
    Ok(strategies)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn effective_loop_merges(
    columns: &[(usize, usize, &'static ExcelColumn)],
    options: &WriteOptions,
    handlers: &[Box<dyn WriteHandler>],
) -> Result<Vec<MirroredLoopMergeStrategy>> {
    let mut strategies = options.loop_merges.clone();
    for strategy in annotation_loop_merges_from_columns(columns)? {
        if !strategies.contains(&strategy) {
            strategies.push(strategy);
        }
    }
    for handler in handlers {
        let Some((property, column_index)) = handler.style_loop_merge() else {
            continue;
        };
        let strategy = MirroredLoopMergeStrategy::new(
            property.each_row,
            property.column_extend,
            to_column(column_index)?,
        )?;
        if !strategies.contains(&strategy) {
            strategies.push(strategy);
        }
    }
    Ok(strategies)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn selected_columns(
    schema: &'static [ExcelColumn],
    options: &WriteOptions,
) -> Result<Vec<(usize, usize, &'static ExcelColumn)>> {
    if schema.is_empty()
        && let Some(head) = &options.dynamic_head
    {
        return Ok(selected_dynamic_columns(head.len(), options));
    }
    let mut columns = ordered_columns(schema)?
        .into_iter()
        .filter(|(physical_index, _, column)| {
            let included_by_index = options
                .include_column_indexes
                .as_ref()
                .is_some_and(|indexes| indexes.contains(physical_index));
            let included_by_name = options
                .include_column_field_names
                .as_ref()
                .is_some_and(|names| names.iter().any(|name| name == column.field));
            let has_includes = options.include_column_indexes.is_some()
                || options.include_column_field_names.is_some();
            let excluded = options.exclude_column_indexes.contains(physical_index)
                || options
                    .exclude_column_field_names
                    .iter()
                    .any(|name| name == column.field);
            (!has_includes || included_by_index || included_by_name) && !excluded
        })
        .collect::<Vec<_>>();

    if options.order_by_include_column {
        columns.sort_by_key(|(physical_index, _, column)| {
            options
                .include_column_indexes
                .as_ref()
                .and_then(|indexes| indexes.iter().position(|index| index == physical_index))
                .or_else(|| {
                    options
                        .include_column_field_names
                        .as_ref()
                        .and_then(|names| names.iter().position(|name| name == column.field))
                })
                .unwrap_or(usize::MAX)
        });
        for (output_index, (physical_index, _, _)) in columns.iter_mut().enumerate() {
            *physical_index = output_index;
        }
    }
    Ok(columns)
}

const DYNAMIC_COLUMN: ExcelColumn = ExcelColumn::new("", "", None, i32::MAX, None);

#[inline(never)]
fn selected_dynamic_columns(
    column_count: usize,
    options: &WriteOptions,
) -> Vec<(usize, usize, &'static ExcelColumn)> {
    let mut columns = Vec::with_capacity(column_count);
    for index in 0..column_count {
        let included_by_index = match &options.include_column_indexes {
            Some(indexes) => indexes.contains(&index),
            None => false,
        };
        let has_includes = options.include_column_indexes.is_some()
            || options.include_column_field_names.is_some();
        let excluded = options.exclude_column_indexes.contains(&index);
        if (!has_includes || included_by_index) && !excluded {
            columns.push((index, index, &DYNAMIC_COLUMN));
        }
    }

    if options.order_by_include_column {
        if let Some(indexes) = &options.include_column_indexes {
            let mut ordered = Vec::with_capacity(columns.len());
            for requested in indexes {
                for column in &columns {
                    if column.1 == *requested {
                        ordered.push(*column);
                        break;
                    }
                }
            }
            columns = ordered;
        }
        for (output_index, (physical_index, _, _)) in columns.iter_mut().enumerate() {
            *physical_index = output_index;
        }
    }
    columns
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn dynamic_columns_for_row(
    schema_is_empty: bool,
    column_count: usize,
    options: &WriteOptions,
) -> Option<Vec<(usize, usize, &'static ExcelColumn)>> {
    if !schema_is_empty {
        return None;
    }
    let Some(head) = &options.dynamic_head else {
        return Some(selected_dynamic_columns(column_count, options));
    };

    // Java `ExcelWriteAddExecutor.addBasicTypeToExcel` consumes basic row
    // values sequentially while iterating the effective head map. When the
    // row is shorter it stops creating cells; when it is longer it appends the
    // remaining values after the greatest head column (issue #1702).
    let mut columns = selected_dynamic_columns(head.len(), options);
    columns.truncate(column_count);
    for (data_index, (_, schema_index, _)) in columns.iter_mut().enumerate() {
        *schema_index = data_index;
    }

    let mut next_physical_index = columns
        .iter()
        .map(|(physical_index, _, _)| *physical_index)
        .max()
        .map_or(0, |index| index.saturating_add(1));
    for data_index in columns.len()..column_count {
        columns.push((next_physical_index, data_index, &DYNAMIC_COLUMN));
        next_physical_index = next_physical_index.saturating_add(1);
    }
    Some(columns)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn head_rows_for_schema(schema: &[ExcelColumn], options: &WriteOptions) -> Result<u32> {
    if schema.is_empty() || options.dynamic_head.is_some() {
        return head_rows_for_schema_state(schema.is_empty(), options);
    }
    if !options.need_head {
        return Ok(0);
    }
    let levels = schema
        .iter()
        .map(|column| column.head_names.map_or(1, <[_]>::len))
        .max()
        .unwrap_or(0);
    head_level_to_row(levels)
}

fn head_rows_for_columns(
    columns: &[(usize, usize, &'static ExcelColumn)],
    schema_is_empty: bool,
    options: &WriteOptions,
) -> Result<u32> {
    if schema_is_empty || options.dynamic_head.is_some() {
        return head_rows_for_schema_state(schema_is_empty, options);
    }
    if !options.need_head {
        return Ok(0);
    }
    let levels = columns
        .iter()
        .map(|(_, _, column)| column.head_names.map_or(1, <[_]>::len))
        .max()
        .unwrap_or(0);
    head_level_to_row(levels)
}

fn head_rows_for_schema_state(schema_is_empty: bool, options: &WriteOptions) -> Result<u32> {
    if schema_is_empty && options.dynamic_head.is_none() {
        return Ok(0);
    }
    dynamic_head_rows(options)
}

fn dynamic_head_rows(options: &WriteOptions) -> Result<u32> {
    if !options.need_head {
        return Ok(0);
    }
    let Some(head) = &options.dynamic_head else {
        return Ok(1);
    };
    if head.is_empty() || head.iter().any(Vec::is_empty) {
        return Err(ExcelError::Format(
            "dynamic head must contain at least one non-empty path".to_owned(),
        ));
    }
    let levels = head.iter().map(Vec::len).max().unwrap_or(0);
    head_level_to_row(levels)
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn selected_dynamic_head_paths(
    columns: &[(usize, usize, &'static ExcelColumn)],
    head: &[Vec<String>],
) -> Result<Vec<Vec<String>>> {
    columns
        .iter()
        .map(|(_, source_index, _)| {
            head.get(*source_index).cloned().ok_or_else(|| {
                ExcelError::Format(format!(
                    "dynamic head source column {source_index} is absent"
                ))
            })
        })
        .collect()
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 返回最终写入表头：动态表头优先，否则使用派生宏生成的 `ExcelProperty.value()`。
pub(crate) fn selected_head_paths(
    columns: &[(usize, usize, &'static ExcelColumn)],
    options: &WriteOptions,
) -> Result<Vec<Vec<String>>> {
    options.dynamic_head.as_deref().map_or_else(
        || {
            Ok(columns
                .iter()
                .map(|(_, _, column)| column.head_path())
                .collect())
        },
        |head| selected_dynamic_head_paths(columns, head),
    )
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn resolved_write_context_holder_state<T>(
    options: &WriteOptions,
    table_no: Option<i32>,
) -> Result<WriteContextHolderState>
where
    T: ExcelRow,
{
    let selected_columns = selected_columns(T::schema(), options)?;
    let selected_head = options
        .dynamic_head
        .as_deref()
        .map(|head| selected_dynamic_head_paths(&selected_columns, head))
        .transpose()?;
    let indexed_columns = selected_columns
        .iter()
        .map(|(column_index, _, column)| (*column_index, *column))
        .collect::<Vec<_>>();
    let head_property = ExcelWriteHeadProperty::from_columns(
        (!T::schema().is_empty()).then(|| type_name::<T>().to_owned()),
        &indexed_columns,
        selected_head.as_deref(),
        *T::write_metadata(),
    )?;

    Ok(WriteContextHolderState {
        holder_type: if table_no.is_some() {
            Holder::Table
        } else {
            Holder::Sheet
        },
        excel_write_head_property: head_property,
        converter_map: crate::converters::default_converter_loader::load_default_write_converter()
            .merged_with(&options.converters),
        need_head: options.need_head,
        automatic_merge_head: options.automatic_merge_head,
        relative_head_row_index: options.relative_head_row_index,
        order_by_include_column: options.order_by_include_column,
        include_column_indexes: options.include_column_indexes.clone(),
        include_column_field_names: options.include_column_field_names.clone(),
        exclude_column_indexes: options.exclude_column_indexes.clone(),
        exclude_column_field_names: options.exclude_column_field_names.clone(),
    })
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn normalized_head_label(path: &[String], level: usize) -> &str {
    path.get(level)
        .or_else(|| path.last())
        .map_or("", String::as_str)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Exact Rust port of Java `ExcelWriteHeadProperty.headCellRangeList()`.
///
/// Each unclaimed cell greedily expands right across equal labels, then down
/// only while the complete rectangle remains equal and unclaimed. Short paths
/// repeat their final label, matching Java `ExcelHeadProperty.initHeadRowNumber`.
pub(crate) fn dynamic_head_merge_ranges(
    columns: &[(usize, usize, &'static ExcelColumn)],
    head: &[Vec<String>],
    start_row: u32,
) -> Result<Vec<MergeRange>> {
    if columns.len() != head.len() {
        return Err(ExcelError::Format(format!(
            "dynamic head column count {} does not match selected column count {}",
            head.len(),
            columns.len()
        )));
    }
    let indexed_columns = columns
        .iter()
        .map(|(column_index, _, column)| (*column_index, *column))
        .collect::<Vec<_>>();
    let property = ExcelWriteHeadProperty::from_columns(
        None,
        &indexed_columns,
        Some(head),
        ExcelWriteMetadata::default(),
    )?;
    property
        .head_cell_range_list()
        .into_iter()
        .map(|range| {
            let first_row = start_row
                .checked_add(u32::try_from(range.first_row).map_err(|_| {
                    ExcelError::Format("dynamic head row can not be negative".to_owned())
                })?)
                .ok_or_else(|| ExcelError::Format("dynamic head row overflow".to_owned()))?;
            let last_row = start_row
                .checked_add(u32::try_from(range.last_row).map_err(|_| {
                    ExcelError::Format("dynamic head row can not be negative".to_owned())
                })?)
                .ok_or_else(|| ExcelError::Format("dynamic head row overflow".to_owned()))?;
            let first_col = usize::try_from(range.first_col).map_err(|_| {
                ExcelError::Format("dynamic head column can not be negative".to_owned())
            })?;
            let last_col = usize::try_from(range.last_col).map_err(|_| {
                ExcelError::Format("dynamic head column can not be negative".to_owned())
            })?;
            Ok(MergeRange::new(
                first_row,
                last_row,
                to_column(first_col)?,
                to_column(last_col)?,
            ))
        })
        .collect()
}
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) fn automatic_dynamic_head_merge_ranges<T>(
    options: &WriteOptions,
    start_row: u32,
    write_head: bool,
) -> Result<Vec<MergeRange>>
where
    T: ExcelRow,
{
    if !write_head || !options.need_head || !options.automatic_merge_head {
        return Ok(Vec::new());
    }
    let columns = selected_columns(T::schema(), options)?;
    let head = selected_head_paths(&columns, options)?;
    dynamic_head_merge_ranges(
        &columns,
        &head,
        start_row.saturating_add(relative_head_start_row(options)),
    )
}

fn head_level_to_row(level: usize) -> Result<u32> {
    u32::try_from(level).map_err(|_| ExcelError::Format("dynamic head is too deep".to_owned()))
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Java `relativeHeadRowIndex` → zero-based start row for a new sheet write.
pub(crate) fn relative_head_start_row(options: &WriteOptions) -> u32 {
    if options.relative_head_row_index <= 0 {
        0
    } else {
        u32::try_from(options.relative_head_row_index).unwrap_or(0)
    }
}
