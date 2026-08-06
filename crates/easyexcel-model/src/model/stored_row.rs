//! 工作表持久化范围的只读行视图。

use crate::model::{Cell, Sheet};

/// 工作表持久化范围中的一行。
///
/// 该视图只遍历显式单元格，同时保留行号和从第零列起算的物理宽度，供事件读取、
/// 查询和导出等上层消费者复用。持久化范围内没有显式单元格的行也会产生一个空视图。
#[derive(Debug, Clone, Copy)]
pub struct StoredRow<'a> {
    sheet: &'a Sheet,
    index: u32,
    first_column: u32,
    last_column: u32,
}

impl<'a> StoredRow<'a> {
    pub(crate) const fn new(
        sheet: &'a Sheet,
        index: u32,
        first_column: u32,
        last_column: u32,
    ) -> Self {
        Self {
            sheet,
            index,
            first_column,
            last_column,
        }
    }

    /// 返回零基行号。
    #[must_use]
    pub const fn index(self) -> u32 {
        self.index
    }

    /// 返回该工作表持久化范围的最小列号。
    #[must_use]
    pub const fn first_column(self) -> u32 {
        self.first_column
    }

    /// 返回该工作表持久化范围的最大列号。
    #[must_use]
    pub const fn last_column(self) -> u32 {
        self.last_column
    }

    /// 返回从第零列起覆盖最大持久化列所需的物理宽度。
    #[must_use]
    pub const fn physical_width(self) -> u32 {
        self.last_column.saturating_add(1)
    }

    /// 按列号升序遍历本行显式持久化的单元格。
    pub fn cells(self) -> impl Iterator<Item = (u32, &'a Cell)> {
        self.sheet
            .cells
            .range((self.index, self.first_column)..=(self.index, self.last_column))
            .map(|(&(_, column), cell)| (column, cell))
    }
}

impl Sheet {
    /// 按行号升序遍历持久化范围中的全部物理行。
    ///
    /// 空工作表返回空迭代器；范围内的缺失行仍返回空 [`StoredRow`]，以便事件读取器
    /// 保留原工作簿行号。
    pub fn stored_rows(&self) -> impl Iterator<Item = StoredRow<'_>> {
        self.stored_range().into_iter().flat_map(move |range| {
            (range.start.row..=range.end.row)
                .map(move |index| StoredRow::new(self, index, range.start.col, range.end.col))
        })
    }
}
