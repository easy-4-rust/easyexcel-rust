use super::Biff8Record;

/// Workbook Globals 子流的可变记录集合。
///
/// 对应 Java：POI `InternalWorkbook`。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Biff8Globals {
    records: Vec<Biff8Record>,
}

impl Biff8Globals {
    /// 从原始顺序记录创建 globals。
    #[must_use]
    pub fn new(records: Vec<Biff8Record>) -> Self {
        Self { records }
    }

    /// 返回保持原始相对顺序的记录。
    #[must_use]
    pub fn records(&self) -> &[Biff8Record] {
        &self.records
    }

    /// 返回可变记录。
    #[must_use]
    pub fn records_mut(&mut self) -> &mut Vec<Biff8Record> {
        &mut self.records
    }
}
