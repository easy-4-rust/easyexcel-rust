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

#[cfg(test)]
mod tests {
    use super::*;

    /// 正常记录写入正确的 header + payload。
    #[test]
    fn write_record_normal() {
        let mut buf = Vec::new();
        let record = Biff8Record::new(0x0009, vec![0xAA, 0xBB]);
        buf.write_record(&record).unwrap();
        // SID (LE) + length (LE) + payload
        assert_eq!(buf[0..2], [0x09, 0x00]); // SID = 0x0009
        assert_eq!(buf[2..4], [0x02, 0x00]); // length = 2
        assert_eq!(buf[4..6], [0xAA, 0xBB]); // payload
        assert_eq!(buf.len(), 6);
    }

    /// 空 payload 写入 4 字节 header。
    #[test]
    fn write_record_empty_payload() {
        let mut buf = Vec::new();
        let record = Biff8Record::new(0x000A, vec![]);
        buf.write_record(&record).unwrap();
        assert_eq!(buf[2..4], [0x00, 0x00]); // length = 0
        assert_eq!(buf.len(), 4);
    }

    /// 多条记录依次追加。
    #[test]
    fn write_multiple_records() {
        let mut buf = Vec::new();
        buf.write_record(&Biff8Record::new(0x0001, vec![1])).unwrap();
        buf.write_record(&Biff8Record::new(0x0002, vec![2, 3])).unwrap();
        // 第一条：4+1=5 bytes，第二条：4+2=6 bytes，共 11
        assert_eq!(buf.len(), 11);
        // 第二条的 SID 在 offset 5
        assert_eq!(buf[5..7], [0x02, 0x00]);
    }

    /// payload 超过 MAX_RECORD_DATA 时返回错误。
    #[test]
    fn write_record_payload_too_large() {
        let mut buf = Vec::new();
        let record = Biff8Record::new(0x0001, vec![0u8; 8225]); // MAX_RECORD_DATA = 8224
        let result = buf.write_record(&record);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("payload exceeds")
        );
    }

    /// payload 恰好等于 MAX_RECORD_DATA 时成功。
    #[test]
    fn write_record_payload_at_max() {
        let mut buf = Vec::new();
        let record = Biff8Record::new(0x0001, vec![0u8; 8224]);
        buf.write_record(&record).unwrap();
        // 4 (header) + 8224 (payload) = 8228
        assert_eq!(buf.len(), 8228);
    }
}
