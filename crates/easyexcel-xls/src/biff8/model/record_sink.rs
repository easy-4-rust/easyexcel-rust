use easyexcel_io::Result;

use super::Biff8Record;

/// BIFF8 record 统一输出端。
///
/// 加密、golden dump 与普通序列化共享此接口，避免各自重新解释 record
/// header 和 payload 边界。
pub trait RecordSink {
    /// 接收一个完整逻辑记录。
    fn write_record(&mut self, record: &Biff8Record) -> Result<()>;
}

impl RecordSink for Vec<u8> {
    fn write_record(&mut self, record: &Biff8Record) -> Result<()> {
        if record.payload().len() > super::super::encode::MAX_RECORD_DATA {
            return Err(easyexcel_io::Error::Xls(format!(
                "BIFF record 0x{:04X} payload exceeds {} bytes",
                record.sid(),
                super::super::encode::MAX_RECORD_DATA
            )));
        }
        let length = u16::try_from(record.payload().len()).map_err(|_| {
            easyexcel_io::Error::Xls(format!(
                "BIFF record 0x{:04X} payload length overflow",
                record.sid()
            ))
        })?;
        self.extend_from_slice(&record.sid().to_le_bytes());
        self.extend_from_slice(&length.to_le_bytes());
        self.extend_from_slice(record.payload());
        Ok(())
    }
}
