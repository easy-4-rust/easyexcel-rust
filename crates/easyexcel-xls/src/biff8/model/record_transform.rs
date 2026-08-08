use easyexcel_io::Result;

use super::Biff8Record;

/// BIFF8 record 变换器。
///
/// 对应 Java：POI record visitor/filter。返回 `None` 表示明确删除记录；未知
/// 记录默认应原样返回，禁止静默丢弃。
pub trait RecordTransform {
    /// 变换单个记录。
    fn transform(&mut self, record: Biff8Record) -> Result<Option<Biff8Record>>;
}
