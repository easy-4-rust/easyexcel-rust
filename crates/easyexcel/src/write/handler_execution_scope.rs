//! WriteHolder 的 own/effective Handler 执行链作用域与注解样式 Handler。
//!
//! 对应 Java：`com.alibaba.excel.write.handler` 内部类型（Handler 链作用域 / 注解配置）。

use std::collections::HashMap;

use crate::core::{
    ExcelCellStyle, ExcelRow, ExcelWriteMetadata, Result, WriteCellContext, WriteHandler,
};

use crate::write::excel_writer_core::{selected_columns, to_column};
use crate::write::merge::loop_merge_strategy::LoopMergeStrategy as MirroredLoopMergeStrategy;
use crate::write::merge::once_absolute_merge_strategy::OnceAbsoluteMergeStrategy;
use crate::write::metadata::style::write_font::merge_excel_font_style as merge_handler_font_style;
use crate::write::shared_write_handler::{
    SharedWriteHandler, boxed_handlers, normalized_shared_handlers,
};
use crate::write::style::column::simple_column_width_style_strategy::SimpleColumnWidthStyleStrategy;
use crate::write::style::row::simple_row_height_style_strategy::SimpleRowHeightStyleStrategy;
use crate::write::write_options::WriteOptions;

/// 对应 Java：com.alibaba.excel.write.handler。 Java `AbstractWriteHolder`'s own/effective execution-chain pair.
///
/// Workbook and sheet callbacks can select `own`, while row/cell callbacks
/// always select `effective`. Child candidates are placed before the already
/// normalized parent chain so stable ordering and `NotRepeatExecutor`
/// de-duplication give the more specific holder precedence.
#[derive(Clone, Default)]
pub(crate) struct HandlerExecutionScope {
    pub(crate) own: Vec<SharedWriteHandler>,
    pub(crate) effective: Vec<SharedWriteHandler>,
}

impl HandlerExecutionScope {
    /// 对应 Java：com.alibaba.excel.write.handler。
    pub(crate) fn root(handlers: &[SharedWriteHandler]) -> Self {
        let own = normalized_shared_handlers(handlers.to_vec());
        Self {
            effective: own.clone(),
            own,
        }
    }
    /// 对应 Java：com.alibaba.excel.write.handler。
    pub(crate) fn child(own_handlers: &[SharedWriteHandler], parent: &Self) -> Self {
        let own_candidates = own_handlers.to_vec();
        let own = normalized_shared_handlers(own_candidates.clone());
        let mut effective_candidates = own_candidates;
        effective_candidates.extend(parent.effective.iter().cloned());
        Self {
            own,
            effective: normalized_shared_handlers(effective_candidates),
        }
    }
    /// 对应 Java：com.alibaba.excel.write.handler。
    pub(crate) fn own_boxed(&self) -> Vec<Box<dyn WriteHandler>> {
        boxed_handlers(&self.own)
    }
    /// 对应 Java：com.alibaba.excel.write.handler。
    pub(crate) fn effective_boxed(&self) -> Vec<Box<dyn WriteHandler>> {
        boxed_handlers(&self.effective)
    }
}

/// Java `initAnnotationConfig` style handler.
///
/// Column widths, row heights and merges use their concrete Java-compatible
/// strategy types. Cell style needs the current `Head`, so this handler keeps
/// class metadata and resolves the field-level override from the callback.
struct AnnotationCellStyleHandler {
    metadata: ExcelWriteMetadata,
    requires_cell_context: bool,
}

impl WriteHandler for AnnotationCellStyleHandler {
    fn backend_capability(&self) -> crate::WriteHandlerCapability {
        crate::WriteHandlerCapability::StreamingSafe
    }

    fn requires_row_context(&self) -> bool {
        false
    }

    fn requires_cell_context(&self) -> bool {
        self.requires_cell_context
    }

    fn order(&self) -> i32 {
        crate::constant::order_constant::ANNOTATION_DEFINE_STYLE
    }

    fn style_cell_style(&self, context: &WriteCellContext) -> Option<ExcelCellStyle> {
        let column = context.column?;
        let (cell, font) = if context.is_head {
            (
                column.head_style.or(self.metadata.head_style),
                column.head_font_style.or(self.metadata.head_font_style),
            )
        } else {
            (
                column.content_style.or(self.metadata.content_style),
                column
                    .content_font_style
                    .or(self.metadata.content_font_style),
            )
        };
        let mut cell = cell.unwrap_or_default();
        if let Some(font) = font {
            cell.font = Some(match cell.font {
                Some(existing) => merge_handler_font_style(&font, existing),
                None => font,
            });
        }
        (cell != ExcelCellStyle::default()).then_some(cell)
    }
}
/// 对应 Java：com.alibaba.excel.write.handler。
pub(crate) fn load_annotation_handlers<T>(
    options: &WriteOptions,
) -> Result<Vec<Box<dyn WriteHandler>>>
where
    T: ExcelRow,
{
    if T::schema().is_empty() {
        return Ok(Vec::new());
    }
    let metadata = T::write_metadata();
    let columns = selected_columns(T::schema(), options)?;
    let mut handlers: Vec<Box<dyn WriteHandler>> = Vec::new();

    for (physical_index, _, column) in &columns {
        if let Some(property) = column.loop_merge {
            handlers.push(Box::new(MirroredLoopMergeStrategy::new(
                property.each_row,
                property.column_extend,
                to_column(*physical_index)?,
            )?));
        }
    }

    let mut widths = SimpleColumnWidthStyleStrategy::new();
    let mut has_width = false;
    for (physical_index, _, column) in &columns {
        if let Some(width) = column.column_width.or(metadata.column_width) {
            widths.set_column_width(*physical_index, width);
            has_width = true;
        }
    }
    if has_width {
        handlers.push(Box::new(widths));
    }

    let has_cell_style = metadata.head_style.is_some()
        || metadata.head_font_style.is_some()
        || metadata.content_style.is_some()
        || metadata.content_font_style.is_some()
        || columns.iter().any(|(_, _, column)| {
            column.head_style.is_some()
                || column.head_font_style.is_some()
                || column.content_style.is_some()
                || column.content_font_style.is_some()
        });
    // Java 始终注册注解样式处理器，因此即使当前类型没有单元格样式，也必须
    // 保留其数量、顺序及后续覆盖语义。能力标记仅允许写入热路径跳过空回调。
    handlers.push(Box::new(AnnotationCellStyleHandler {
        metadata: *metadata,
        requires_cell_context: has_cell_style,
    }));

    if metadata.head_row_height.is_some() || metadata.content_row_height.is_some() {
        handlers.push(Box::new(SimpleRowHeightStyleStrategy::new(
            metadata.head_row_height,
            metadata.content_row_height,
        )));
    }

    if let Some(property) = metadata.once_absolute_merge {
        handlers.push(Box::new(OnceAbsoluteMergeStrategy::from_property(
            property,
        )?));
    }
    Ok(handlers)
}

/// 对应 Java：com.alibaba.excel.write.handler。 Ensures a gzip spill writer exists for `sheet_name` when compress is on.
pub(crate) fn ensure_gzip_spill<'a>(
    spills: &'a mut HashMap<String, crate::write::gzip_spill::GzipSheetDataWriter>,
    sheet_name: &str,
    compress: bool,
) -> Result<Option<&'a mut crate::write::gzip_spill::GzipSheetDataWriter>> {
    if !compress {
        return Ok(None);
    }
    if !spills.contains_key(sheet_name) {
        spills.insert(
            sheet_name.to_owned(),
            crate::write::gzip_spill::GzipSheetDataWriter::create_owned(sheet_name)?,
        );
    }
    Ok(spills.get_mut(sheet_name))
}
