//! 从可寻址的 `BufRead + Seek` 源按需读取 BIFF8 record 的流式迭代器。
//!
//! 相比旧的 `Records<'a>`（需要整个子流的 `&[u8]` 切片），`StreamingRecordIter`
//! 每次 `next()` 只从底层 reader 读取一条 record 的 header + payload，并自动合并
//! 紧随其后的 `CONTINUE` 链。适用于 `cfb::Stream` 等支持 seek 的流式源。
//!
//! 对应 Java：无直接对应对象；Rust 架构扩展（Phase 1 of xls-streaming RFC）。

use std::io::{BufRead, Seek, SeekFrom};

use easyexcel_io::{Error as XlsError, Result};

/// BIFF8 `CONTINUE` record 的 SID（0x003C）。
const CONTINUE_SID: u16 = 0x003C;

/// BIFF record header 固定字节数（2 bytes SID + 2 bytes payload length）。
const HEADER_LEN: usize = 4;

/// 从支持 `BufRead + Seek` 的源按需读取 BIFF8 record 的流式迭代器。
///
/// 每次调用 [`next()`](Iterator::next) 会从底层 reader 读取一条 record 的
/// header（4 字节：SID + payload 长度）和 payload，并自动合并紧随其后的所有
/// `CONTINUE` 记录（SID = 0x003C），返回一个完整的逻辑记录 `(sid, payload)`。
///
/// # 设计说明
///
/// 该迭代器对应 RFC Phase 1 的核心基础设施，目标是替代需要整个子流 `&[u8]`
/// 切片的旧 `Records<'a>`。使用 feature flag `xls-streaming-iter` 控制编译，
/// 不会影响现有代码路径。
///
/// # 类型参数
///
/// - `R`：底层 reader，必须实现 `BufRead`（用于高效缓冲读取）和 `Seek`
///   （用于在 CONTINUE 链合并失败时定位到下一条 record）。
///
/// # CONTINUE 合并
///
/// BIFF8 的逻辑记录可能跨越多个物理 record：一个主 record 后跟零或多个
/// `CONTINUE` record，每个 CONTINUE 的 payload 追加到主 record 的 payload。
/// 该迭代器在 `next()` 内自动处理此合并，调用方无需关心 CONTINUE 边界。
///
/// # EOF 行为
///
/// 当 reader 位置达到 `end` 偏移或读取到 EOF 时，`next()` 返回 `None`。
/// 如果读取过程中遇到 I/O 错误或 BIFF record 损坏，返回 `Some(Err(...))`。
///
/// # 示例
///
/// ```ignore
/// use std::io::Cursor;
/// use easyexcel_xls::biff8::streaming_record_iter::StreamingRecordIter;
///
/// // 一条 NUMBER record：SID=0x0203, payload=[0xAA, 0xBB]
/// let data = [0x03, 0x02, 0x02, 0x00, 0xAA, 0xBB];
/// let cursor = Cursor::new(data.to_vec());
/// let mut iter = StreamingRecordIter::new(cursor, 0, 6).unwrap();
///
/// let (sid, payload) = iter.next().unwrap().unwrap();
/// assert_eq!(sid, 0x0203);
/// assert_eq!(payload, vec![0xAA, 0xBB]);
/// assert!(iter.next().is_none());
/// ```
///
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub struct StreamingRecordIter<R: BufRead + Seek> {
    /// 底层可缓冲读取 + 可寻址的 reader。
    reader: R,
    /// 当前读取位置（字节偏移），用于检测何时到达 `end`。
    pos: u64,
    /// 流的结束偏移（不含），超过此位置后 `next()` 返回 `None`。
    end: u64,
}

impl<R: BufRead + Seek> StreamingRecordIter<R> {
    /// 使用给定的 reader 和字节范围创建新的流式 record 迭代器。
    ///
    /// `reader` 的初始位置应该是 BIFF record 流的起始位置（即 `start`）。
    /// 迭代器会从当前位置开始读取，直到达到 `end` 偏移。
    ///
    /// # 参数
    ///
    /// - `reader`：底层 `BufRead + Seek` 源（如 `cfb::Stream`）。
    /// - `start`：BIFF record 流的起始偏移（字节）。
    /// - `end`：BIFF record 流的结束偏移（不含）。
    ///
    /// # Errors
    ///
    /// 将 reader 定位到 `start` 失败时返回 I/O 错误。
    ///
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn new(mut reader: R, start: u64, end: u64) -> Result<Self> {
        reader.seek(SeekFrom::Start(start))?;
        Ok(Self {
            reader,
            pos: start,
            end,
        })
    }

    /// 从 reader 读取并返回下一条完整的逻辑 BIFF record（含 CONTINUE 合并）。
    ///
    /// 返回 `Some(Ok((sid, payload)))` 表示成功读取一条 record；
    /// `Some(Err(...))` 表示读取过程中出错；`None` 表示已到达流末尾。
    fn next_inner(&mut self) -> Option<Result<(u16, Vec<u8>)>> {
        if self.pos >= self.end {
            return None;
        }

        // --- 读取主 record header ---
        let (sid, payload_len) = match self.read_header() {
            Ok(header) => header,
            Err(e) => return Some(Err(e)),
        };

        // --- 读取主 record payload ---
        let mut payload = match self.read_payload(payload_len) {
            Ok(p) => p,
            Err(e) => return Some(Err(e)),
        };

        // --- 合并后续 CONTINUE 链 ---
        loop {
            match self.peek_sid() {
                Ok(CONTINUE_SID) => {
                    // 消费 CONTINUE header
                    let (_cont_sid, cont_len) = match self.read_header() {
                        Ok(header) => header,
                        Err(e) => return Some(Err(e)),
                    };
                    match self.read_payload(cont_len) {
                        Ok(cont_payload) => payload.extend_from_slice(&cont_payload),
                        Err(e) => return Some(Err(e)),
                    }
                }
                Ok(_other_sid) => {
                    // 下一条 record 不是 CONTINUE，当前逻辑记录结束
                    break;
                }
                Err(e) => {
                    // peek 失败（I/O 或截断 header）——向上层报告错误
                    return Some(Err(e));
                }
            }
        }

        Some(Ok((sid, payload)))
    }

    /// 从 reader 读取 4 字节 BIFF record header，返回 `(sid, payload_length)`。
    ///
    /// # Errors
    ///
    /// header 不足 4 字节时返回错误。
    fn read_header(&mut self) -> Result<(u16, usize)> {
        let mut header = [0u8; HEADER_LEN];
        let bytes_read = self.reader.read(&mut header).map_err(XlsError::Io)?;
        if bytes_read < HEADER_LEN {
            return Err(XlsError::Xls(format!(
                "truncated BIFF record header at byte {}: expected 4 bytes, got {}",
                self.pos, bytes_read,
            )));
        }
        let sid = u16::from_le_bytes([header[0], header[1]]);
        let payload_len = u16::from_le_bytes([header[2], header[3]]) as usize;
        self.pos += HEADER_LEN as u64;
        Ok((sid, payload_len))
    }

    /// 从 reader 读取指定长度的 payload 字节。
    ///
    /// # Errors
    ///
    /// payload 不足预期长度时返回错误。
    fn read_payload(&mut self, len: usize) -> Result<Vec<u8>> {
        if len == 0 {
            return Ok(Vec::new());
        }
        let mut payload = vec![0u8; len];
        self.reader.read_exact(&mut payload).map_err(XlsError::Io)?;
        self.pos += len as u64;
        Ok(payload)
    }

    /// 偷看下一条 record 的 SID，但不消费 header 字节。
    ///
    /// 如果已到达流末尾，返回一个非 CONTINUE 的哨兵 SID（0xFFFF）。
    ///
    /// # Errors
    ///
    /// header 不足 4 字节时返回错误。
    fn peek_sid(&mut self) -> Result<u16> {
        if self.pos >= self.end {
            // 流已结束，返回哨兵值
            return Ok(0xFFFF);
        }

        let mut header = [0u8; HEADER_LEN];
        let bytes_read = self.reader.read(&mut header).map_err(XlsError::Io)?;
        if bytes_read < HEADER_LEN {
            // 不足 4 字节不能构成有效 header，视为流结束
            // 但需要将已读的字节回退
            self.reader
                .seek(SeekFrom::Current(-(bytes_read as i64)))
                .map_err(XlsError::Io)?;
            return Ok(0xFFFF);
        }

        let sid = u16::from_le_bytes([header[0], header[1]]);

        // 回退，不消费 header
        self.reader
            .seek(SeekFrom::Current(-(HEADER_LEN as i64)))
            .map_err(XlsError::Io)?;

        Ok(sid)
    }
}

impl<R: BufRead + Seek> Iterator for StreamingRecordIter<R> {
    type Item = Result<(u16, Vec<u8>)>;

    /// 返回下一条完整的逻辑 BIFF record（已合并 CONTINUE 链）。
    ///
    /// - `Some(Ok((sid, payload)))`：成功读取一条逻辑 record。
    /// - `Some(Err(...))`：读取过程中遇到 I/O 错误或 record 损坏。
    /// - `None`：已到达流末尾（pos >= end）。
    fn next(&mut self) -> Option<Self::Item> {
        self.next_inner()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// 构造一条 BIFF record 的字节序列（header + payload）。
    fn make_record(sid: u16, payload: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(HEADER_LEN + payload.len());
        bytes.extend_from_slice(&sid.to_le_bytes());
        bytes.extend_from_slice(&(payload.len() as u16).to_le_bytes());
        bytes.extend_from_slice(payload);
        bytes
    }

    /// 创建测试用的 `StreamingRecordIter`（Cursor 包装）。
    fn make_iter(data: &[u8]) -> StreamingRecordIter<Cursor<Vec<u8>>> {
        let len = data.len() as u64;
        StreamingRecordIter::new(Cursor::new(data.to_vec()), 0, len).unwrap()
    }

    #[test]
    fn reads_single_record() {
        // 一条 NUMBER record：SID=0x0203, payload=[0xAA, 0xBB]
        let data = make_record(0x0203, &[0xAA, 0xBB]);
        let mut iter = make_iter(&data);

        let (sid, payload) = iter.next().unwrap().unwrap();
        assert_eq!(sid, 0x0203);
        assert_eq!(payload, vec![0xAA, 0xBB]);

        // 流结束
        assert!(iter.next().is_none());
    }

    #[test]
    fn reads_multiple_records() {
        // 三条连续 record
        let mut data = Vec::new();
        data.extend_from_slice(&make_record(0x000A, &[])); // EOF (empty)
        data.extend_from_slice(&make_record(0x0203, &[1, 2, 3])); // NUMBER
        data.extend_from_slice(&make_record(0x00FC, &[4, 5])); // SST

        let mut iter = make_iter(&data);

        let (sid, p) = iter.next().unwrap().unwrap();
        assert_eq!(sid, 0x000A);
        assert!(p.is_empty());

        let (sid, p) = iter.next().unwrap().unwrap();
        assert_eq!(sid, 0x0203);
        assert_eq!(p, vec![1, 2, 3]);

        let (sid, p) = iter.next().unwrap().unwrap();
        assert_eq!(sid, 0x00FC);
        assert_eq!(p, vec![4, 5]);

        assert!(iter.next().is_none());
    }

    #[test]
    fn merges_continue_chain() {
        // 主 record（SST） + 2 个 CONTINUE
        let mut data = Vec::new();
        data.extend_from_slice(&make_record(0x00FC, &[0x01, 0x02])); // SST, payload=[1,2]
        data.extend_from_slice(&make_record(CONTINUE_SID, &[0x03, 0x04])); // CONTINUE, payload=[3,4]
        data.extend_from_slice(&make_record(CONTINUE_SID, &[0x05])); // CONTINUE, payload=[5]
        // 下一条非 CONTINUE
        data.extend_from_slice(&make_record(0x000A, &[])); // EOF

        let mut iter = make_iter(&data);

        // SST 合并后 payload = [1,2,3,4,5]
        let (sid, payload) = iter.next().unwrap().unwrap();
        assert_eq!(sid, 0x00FC);
        assert_eq!(payload, vec![0x01, 0x02, 0x03, 0x04, 0x05]);

        // EOF
        let (sid, payload) = iter.next().unwrap().unwrap();
        assert_eq!(sid, 0x000A);
        assert!(payload.is_empty());

        assert!(iter.next().is_none());
    }

    #[test]
    fn handles_empty_payload() {
        let data = make_record(0x000A, &[]);
        let mut iter = make_iter(&data);

        let (sid, payload) = iter.next().unwrap().unwrap();
        assert_eq!(sid, 0x000A);
        assert!(payload.is_empty());
        assert!(iter.next().is_none());
    }

    #[test]
    fn handles_empty_stream() {
        let data: Vec<u8> = Vec::new();
        let mut iter = make_iter(&data);
        assert!(iter.next().is_none());
    }

    #[test]
    fn rejects_truncated_header() {
        // 只有 2 字节 header（不足 4 字节）
        let data = [0x03, 0x02];
        let mut iter = make_iter(&data);

        let err = iter.next().unwrap().unwrap_err();
        assert!(
            err.to_string().contains("truncated BIFF record header"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn rejects_truncated_payload() {
        // header 声明 payload 4 字节，但只有 1 字节
        let data = [0x03, 0x02, 0x04, 0x00, 0xAA];
        let mut iter = make_iter(&data);

        let err = iter.next().unwrap().unwrap_err();
        assert!(
            err.to_string().contains("truncated BIFF record header")
                || err.to_string().contains("failed to fill whole buffer"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn read_from_offset() {
        // 前 4 字节是垃圾，有效数据从 offset 4 开始
        let mut data = vec![0xFF, 0xFF, 0xFF, 0xFF];
        data.extend_from_slice(&make_record(0x0203, &[0xAA]));

        let len = data.len() as u64;
        let mut iter = StreamingRecordIter::new(Cursor::new(data), 4, len).unwrap();

        let (sid, payload) = iter.next().unwrap().unwrap();
        assert_eq!(sid, 0x0203);
        assert_eq!(payload, vec![0xAA]);
        assert!(iter.next().is_none());
    }

    #[test]
    fn eof_at_exact_boundary() {
        // end 刚好在最后一条 record 之后
        let data = make_record(0x000A, &[1, 2, 3]);
        let mut iter = make_iter(&data);

        let (sid, payload) = iter.next().unwrap().unwrap();
        assert_eq!(sid, 0x000A);
        assert_eq!(payload, vec![1, 2, 3]);
        assert!(iter.next().is_none());
    }

    #[test]
    fn continues_merge_with_large_payload() {
        // 模拟一个较大的 CONTINUE 链
        let mut data = Vec::new();
        let main_payload: Vec<u8> = (0..100).collect();
        data.extend_from_slice(&make_record(0x00FC, &main_payload));

        for i in 0..5u8 {
            let cont_payload: Vec<u8> = (0..50).map(|j| i * 50 + j).collect();
            data.extend_from_slice(&make_record(CONTINUE_SID, &cont_payload));
        }

        // 终止记录
        data.extend_from_slice(&make_record(0x000A, &[]));

        let mut iter = make_iter(&data);

        let (sid, payload) = iter.next().unwrap().unwrap();
        assert_eq!(sid, 0x00FC);
        // 主 payload 100 bytes + 5 * 50 bytes = 350 bytes
        assert_eq!(payload.len(), 350);
        // 验证前 100 字节是主 payload
        assert_eq!(&payload[..100], &main_payload[..]);

        // 第二条是 EOF
        let (sid, _) = iter.next().unwrap().unwrap();
        assert_eq!(sid, 0x000A);

        assert!(iter.next().is_none());
    }

    #[test]
    fn no_continue_after_non_continue_record() {
        // 确保只有 CONTINUE SID 的 record 才被合并
        let mut data = Vec::new();
        data.extend_from_slice(&make_record(0x00FC, &[1, 2])); // SST
        data.extend_from_slice(&make_record(0x0203, &[3, 4])); // NUMBER (非 CONTINUE)
        data.extend_from_slice(&make_record(0x000A, &[])); // EOF

        let mut iter = make_iter(&data);

        // SST 不应该合并 NUMBER
        let (sid, payload) = iter.next().unwrap().unwrap();
        assert_eq!(sid, 0x00FC);
        assert_eq!(payload, vec![1, 2]);

        let (sid, payload) = iter.next().unwrap().unwrap();
        assert_eq!(sid, 0x0203);
        assert_eq!(payload, vec![3, 4]);

        let (sid, _) = iter.next().unwrap().unwrap();
        assert_eq!(sid, 0x000A);

        assert!(iter.next().is_none());
    }

    #[test]
    fn multiple_records_with_zero_padding() {
        // 数据后面有零填充（常见于实际 XLS 文件）
        let mut data = make_record(0x0203, &[0xAA]);
        data.extend_from_slice(&[0; 16]); // 零填充

        // end 只覆盖有效 record 部分
        let record_len = data.len() - 16;
        let mut iter =
            StreamingRecordIter::new(Cursor::new(data), 0, record_len as u64).unwrap();

        let (sid, payload) = iter.next().unwrap().unwrap();
        assert_eq!(sid, 0x0203);
        assert_eq!(payload, vec![0xAA]);
        // end 已达到，不应读到零填充
        assert!(iter.next().is_none());
    }
}
