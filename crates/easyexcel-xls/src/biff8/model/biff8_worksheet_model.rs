use super::{Biff8ObjectModel, Biff8Record};

/// 单个 BIFF8 worksheet/chart/macro sheet 子流。
///
/// 对应 Java：POI `InternalSheet`。worksheet 以外的 BoundSheet 子流也保留，
/// 但 `is_worksheet` 为 false，调用方不得把它当作单元格网格修改。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Biff8WorksheetModel {
    name: String,
    bound_sheet_index: usize,
    is_worksheet: bool,
    records: Vec<Biff8Record>,
    objects: Biff8ObjectModel,
}

impl Biff8WorksheetModel {
    /// 创建一个 sheet 子流模型。
    #[must_use]
    pub fn new(
        name: String,
        bound_sheet_index: usize,
        is_worksheet: bool,
        records: Vec<Biff8Record>,
        objects: Biff8ObjectModel,
    ) -> Self {
        Self {
            name,
            bound_sheet_index,
            is_worksheet,
            records,
            objects,
        }
    }

    /// 返回 BoundSheet 名称。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 返回该子流在全部 BoundSheet 中的索引。
    #[must_use]
    pub const fn bound_sheet_index(&self) -> usize {
        self.bound_sheet_index
    }

    /// 返回是否为 worksheet 单元格子流。
    #[must_use]
    pub const fn is_worksheet(&self) -> bool {
        self.is_worksheet
    }

    /// 返回原始顺序记录。
    #[must_use]
    pub fn records(&self) -> &[Biff8Record] {
        &self.records
    }

    /// 返回可变记录。
    #[must_use]
    pub fn records_mut(&mut self) -> &mut Vec<Biff8Record> {
        &mut self.records
    }

    /// 返回对象模型。
    #[must_use]
    pub const fn objects(&self) -> &Biff8ObjectModel {
        &self.objects
    }

    /// 返回可变对象模型。
    #[must_use]
    pub fn objects_mut(&mut self) -> &mut Biff8ObjectModel {
        &mut self.objects
    }
}
