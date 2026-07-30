//! 工作表元数据类型。
//!
//! 对应 Java：`com.alibaba.excel.write.metadata.WriteSheet`。
//! 原文件：easyexcel-core/src/main/java/com/alibaba/excel/write/metadata/WriteSheet.java

use std::marker::PhantomData;

use easyexcel_core::{
    Converter, ExcelColumn, ExcelRow, NullableObjectConverter, Result,
};

use crate::cell_style::CellStyle;
use crate::merge_range::MergeRange;
use crate::merge::loop_merge_strategy::LoopMergeStrategy;
use crate::write_options::WriteOptions;

/// 用于 [`ExcelWriter`] 的工作表元数据。
///
/// 对应 Java：`com.alibaba.excel.write.metadata.WriteSheet`。
/// 通过 `WriteSheet::new("name")` 或 `WriteSheet::new_index(index)` 创建。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteSheet<T> {
    pub(crate) options: WriteOptions,
    marker: PhantomData<T>,
}

impl<T> WriteSheet<T> {
    /// 从完整选项集创建工作表元数据。
    #[must_use]
    pub fn from_options(options: WriteOptions) -> Self {
        Self {
            options,
            marker: PhantomData,
        }
    }

    /// 使用提供的名称创建工作表元数据。
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            options: WriteOptions {
                sheet_name: name.into(),
                ..WriteOptions::default()
            },
            marker: PhantomData,
        }
    }

    /// 使用 Java 风格的零基工作表编号创建工作表元数据。
    #[must_use]
    pub fn new_index(index: usize) -> Self {
        Self {
            options: WriteOptions {
                sheet_name: index.to_string(),
                sheet_index: Some(index),
                ..WriteOptions::default()
            },
            marker: PhantomData,
        }
    }

    /// 返回有效的写入选项。
    #[must_use]
    pub const fn options(&self) -> &WriteOptions {
        &self.options
    }

    /// 注册覆盖工作簿注册的工作表级转换器。
    #[must_use]
    pub fn register_converter<V, C>(mut self, converter: C) -> Self
    where
        V: 'static,
        C: Converter<V> + Send + Sync + 'static,
    {
        self.options.converters.register::<V, C>(converter);
        self
    }

    /// 注册可以接收缺失的 `Option<T>` 值的工作表级转换器。
    #[must_use]
    pub fn register_nullable_converter<V, C>(mut self, converter: C) -> Self
    where
        V: 'static,
        C: NullableObjectConverter<V> + Send + Sync + 'static,
    {
        self.options.converters.register_nullable::<V, C>(converter);
        self
    }

    /// 添加 Java 风格的零基逻辑工作表编号。
    #[must_use]
    pub const fn sheet_index(mut self, index: usize) -> Self {
        self.options.sheet_index = Some(index);
        self
    }

    /// 启用或禁用此工作表的表头。
    #[must_use]
    pub const fn need_head(mut self, enabled: bool) -> Self {
        self.options.need_head = enabled;
        self
    }

    /// 启用或禁用此工作表的常量内存输出。
    #[must_use]
    pub const fn constant_memory(mut self, enabled: bool) -> Self {
        self.options.constant_memory = enabled;
        self
    }

    /// 启用压缩/磁盘溢出临时文件用于批量写入。
    ///
    /// Java: `SXSSFWorkbook.setCompressTempFiles(bool)`。
    /// 同时启用 [`Self::constant_memory`] 以便行刷新到磁盘而不是在 RAM 中增长。
    #[must_use]
    pub const fn compress_temp_files(mut self, enabled: bool) -> Self {
        self.options.compress_temp_files = enabled;
        if enabled {
            self.options.constant_memory = true;
        }
        self
    }

    /// 冻结此工作表的表头行。
    #[must_use]
    pub const fn freeze_head(mut self, enabled: bool) -> Self {
        self.options.freeze_head = enabled;
        self
    }

    /// 添加绝对合并单元格范围。
    #[must_use]
    pub fn merge_cells(mut self, range: MergeRange) -> Self {
        self.options.merge_ranges.push(range);
        self
    }

    /// 启用或禁用自动宽度计算。
    #[must_use]
    pub const fn auto_width(mut self, enabled: bool) -> Self {
        self.options.auto_width = enabled;
        self
    }

    /// 为零基物理列设置显式宽度。
    #[must_use]
    pub fn column_width(mut self, column: u16, width: u16) -> Self {
        self.options.column_widths.push((column, width));
        self
    }

    /// 替换表头样式。
    #[must_use]
    pub fn head_style(mut self, style: CellStyle) -> Self {
        self.options.head_style = style;
        self
    }

    /// 为所有内容行使用一种样式。
    #[must_use]
    pub fn content_style(mut self, style: CellStyle) -> Self {
        self.options.content_styles = vec![style];
        self
    }

    /// 循环使用提供的样式。
    #[must_use]
    pub fn content_styles(mut self, styles: impl IntoIterator<Item = CellStyle>) -> Self {
        self.options.content_styles = styles.into_iter().collect();
        self
    }

    /// 注册重复数据行合并策略。
    #[must_use]
    pub fn loop_merge(mut self, strategy: LoopMergeStrategy) -> Self {
        self.options.loop_merges.push(strategy);
        self
    }

    /// 使用动态多级表头路径替换派生的表头。
    #[must_use]
    pub fn head<S, P>(mut self, paths: impl IntoIterator<Item = P>) -> Self
    where
        S: Into<String>,
        P: IntoIterator<Item = S>,
    {
        self.options.dynamic_head = Some(
            paths
                .into_iter()
                .map(|path| path.into_iter().map(Into::into).collect())
                .collect(),
        );
        self
    }
}
