use super::Biff8Record;

/// 工作簿 drawing/comment/chart 对象记录视图。
///
/// 对应 Java：POI `HSSFPatriarch` 与 `EscherAggregate`。该模型不重排对象
/// 记录，只集中分配对象 ID 并保存其原始 record group。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Biff8ObjectModel {
    records: Vec<Biff8Record>,
    next_object_id: u16,
}

impl Biff8ObjectModel {
    /// 创建对象记录模型。
    #[must_use]
    pub fn new(records: Vec<Biff8Record>, next_object_id: u16) -> Self {
        Self {
            records,
            next_object_id: next_object_id.max(1),
        }
    }

    /// 返回对象相关记录。
    #[must_use]
    pub fn records(&self) -> &[Biff8Record] {
        &self.records
    }

    /// 分配工作簿内唯一对象 ID。
    pub fn allocate_object_id(&mut self) -> u16 {
        let current = self.next_object_id;
        self.next_object_id = self.next_object_id.saturating_add(1);
        current
    }
}
