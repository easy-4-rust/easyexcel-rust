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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_model(name: &str, index: usize, is_ws: bool) -> Biff8WorksheetModel {
        Biff8WorksheetModel::new(
            name.to_owned(),
            index,
            is_ws,
            vec![Biff8Record::new(0x0009, vec![1, 2, 3])],
            Biff8ObjectModel::default(),
        )
    }

    /// 验证 `new` 设置所有字段。
    #[test]
    fn new_sets_all_fields() {
        let m = make_model("Sheet1", 2, true);
        assert_eq!(m.name(), "Sheet1");
        assert_eq!(m.bound_sheet_index(), 2);
        assert!(m.is_worksheet());
        assert_eq!(m.records().len(), 1);
        assert_eq!(m.records()[0].sid(), 0x0009);
    }

    /// 非 worksheet 子流（如 chart）。
    #[test]
    fn non_worksheet() {
        let m = make_model("Chart1", 1, false);
        assert!(!m.is_worksheet());
        assert_eq!(m.name(), "Chart1");
    }

    /// 空记录和对象。
    #[test]
    fn empty_records_and_objects() {
        let m = Biff8WorksheetModel::new(
            "Empty".to_owned(),
            0,
            true,
            vec![],
            Biff8ObjectModel::default(),
        );
        assert!(m.records().is_empty());
        assert!(m.objects().records().is_empty());
    }

    /// `records_mut` 返回可变引用。
    #[test]
    fn records_mut_allows_push() {
        let mut m = make_model("S", 0, true);
        m.records_mut().push(Biff8Record::new(0x000A, vec![4, 5]));
        assert_eq!(m.records().len(), 2);
    }

    /// `objects_mut` 返回可变引用。
    #[test]
    fn objects_mut_allows_allocate_id() {
        let mut m = make_model("S", 0, true);
        let id = m.objects_mut().allocate_object_id();
        assert_eq!(id, 0); // default next_object_id is 0
    }

    /// Clone 和 PartialEq。
    #[test]
    fn clone_and_eq() {
        let m1 = make_model("S", 0, true);
        let m2 = m1.clone();
        assert_eq!(m1, m2);
    }

    /// Debug 不 panic。
    #[test]
    fn debug_does_not_panic() {
        let m = make_model("S", 0, true);
        let _ = format!("{m:?}");
    }
}
