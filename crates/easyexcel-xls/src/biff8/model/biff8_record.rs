/// 单个带帧 BIFF8 记录。
///
/// 对应 Java：`org.apache.poi.hssf.record.RecordBase`。未知 SID 也按原始
/// payload 保留，从而允许模板无损往返。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Biff8Record {
    /// 记录类型编号。
    pub(crate) typ: u16,
    /// 不含四字节 record header 的 payload。
    pub(crate) data: Vec<u8>,
}

impl Biff8Record {
    /// 创建一个记录。
    #[must_use]
    pub fn new(sid: u16, payload: Vec<u8>) -> Self {
        Self {
            typ: sid,
            data: payload,
        }
    }

    /// 返回记录 SID。
    #[must_use]
    pub const fn sid(&self) -> u16 {
        self.typ
    }

    /// 返回记录 payload。
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.data
    }

    /// 返回可变 payload，供 typed transform 修改已知字段。
    #[must_use]
    pub fn payload_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }
}
