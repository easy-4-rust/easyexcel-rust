//! SST 延迟解码容器——Phase 2 性能优化核心。
//!
//! 对应 Java：无直接对应对象；Rust 架构扩展。保留 SST 原始字节，仅扫描
//! 每个字符串的头部建立字节偏移索引，首次按索引访问时才解码该字符串
//! （UTF-16 -> UTF-8），避免一次性解码全部字符串带来的 CPU 与内存压力。
//!
//! 当 `xls-lazy-sst` feature 启用时，SST 解码路径使用此类型替代
//! 一次性解码为 `Vec<Biff8SstString>` 的旧路径。

use easyexcel_io::{Error as ExcelError, Result};

use crate::xls::Biff8SstString;

// ---------------------------------------------------------------------------
// 公共 API
// ---------------------------------------------------------------------------

/// SST 延迟解码容器。
///
/// 保留 SST + CONTINUE 原始字节，构造时仅扫描字符串头部建立偏移索引，
/// 不解码任何字符数据。`get(idx)` 按需解码第 `idx` 个字符串。
///
/// 对应 Java：`HSSFWorkbook` / `SharedStringsTable` 的延迟加载概念。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LazySst {
    /// 合并后的 SST + CONTINUE 原始字节。
    data: Vec<u8>,
    /// CONTINUE 段在 `data` 中的起始字节偏移（该位置的第一个字节为
    /// CONTINUE 后的 fresh grbit）。
    breaks: Vec<usize>,
    /// 每个字符串在 `data` 中的起始偏移（指向 string header 的首字节）。
    offsets: Vec<usize>,
    /// SST 声明的唯一字符串数量。
    unique_count: usize,
}

impl LazySst {
    /// 从 SST record segments 构造延迟解码容器。
    ///
    /// 仅扫描字符串头部建立偏移索引，不解码字符数据。
    ///
    /// # Errors
    ///
    /// SST 元数据或分段字符串头部损坏时返回错误。
    pub fn new(segments: &[Vec<u8>]) -> Result<Self> {
        // 合并所有 segment 为连续字节，记录 CONTINUE 断点
        let mut data = Vec::with_capacity(segments.iter().map(Vec::len).sum());
        let mut breaks = Vec::new();
        for (i, seg) in segments.iter().enumerate() {
            if i > 0 {
                breaks.push(data.len());
            }
            data.extend_from_slice(seg);
        }

        if data.len() < 8 {
            return Err(ExcelError::Xls(
                "SST record too short for header".to_owned(),
            ));
        }
        let _total = read_u32(&data, 0);
        let unique = read_u32(&data, 4) as usize;

        let available = data.len();
        if unique > available.saturating_div(3).saturating_add(1) {
            return Err(ExcelError::Xls(format!(
                "SST declares {unique} unique strings in only {available} bytes"
            )));
        }

        // 扫描每个 string header，记录字节偏移但不解码字符
        let mut cursor = ScanCursor::new(&data, &breaks, 8);
        let mut offsets = Vec::with_capacity(unique.min(16_384));

        for _ in 0..unique {
            offsets.push(cursor.pos());
            cursor.skip_string()?;
        }

        Ok(Self {
            data,
            breaks,
            offsets,
            unique_count: unique,
        })
    }

    /// 返回 SST 中唯一字符串的数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.unique_count
    }

    /// SST 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.unique_count == 0
    }

    /// 按需解码第 `idx` 个共享字符串。
    ///
    /// # Errors
    ///
    /// `idx` 越界或该字符串的字节数据损坏时返回错误。
    pub fn get(&self, idx: usize) -> Result<Biff8SstString> {
        if idx >= self.unique_count {
            return Err(ExcelError::Xls(format!(
                "SST index {idx} out of range ({} unique strings)",
                self.unique_count
            )));
        }
        let offset = self.offsets[idx];
        let mut cursor = ScanCursor::new(&self.data, &self.breaks, offset);
        cursor.read_string(idx)
    }

    /// 批量解码全部字符串（用于需要 `Vec<Biff8SstString>` 的场景）。
    ///
    /// # Errors
    ///
    /// 任一字符串损坏时返回错误。
    pub fn to_vec(&self) -> Result<Vec<Biff8SstString>> {
        let mut out = Vec::with_capacity(self.unique_count);
        for idx in 0..self.unique_count {
            out.push(self.get(idx)?);
        }
        Ok(out)
    }
}

// ---------------------------------------------------------------------------
// 内部扫描游标
// ---------------------------------------------------------------------------

/// 在合并字节流上扫描 SST 字符串的游标，理解 CONTINUE 断点处的 fresh grbit。
struct ScanCursor<'a> {
    data: &'a [u8],
    /// 排序后的 CONTINUE 断点偏移
    breaks: &'a [usize],
    pos: usize,
}

impl<'a> ScanCursor<'a> {
    fn new(data: &'a [u8], breaks: &'a [usize], pos: usize) -> Self {
        Self { data, breaks, pos }
    }

    fn pos(&self) -> usize {
        self.pos
    }

    fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    /// 当前位置是否恰好在 CONTINUE 断点上
    fn at_break(&self) -> bool {
        self.breaks.contains(&self.pos)
    }

    fn read_u8(&mut self) -> Result<u8> {
        self.data
            .get(self.pos)
            .copied()
            .ok_or_else(|| ExcelError::Xls("truncated SST data".to_owned()))
            .map(|v| {
                self.pos += 1;
                v
            })
    }

    fn read_u16(&mut self) -> Result<u16> {
        if self.pos + 2 > self.data.len() {
            return Err(ExcelError::Xls("truncated SST u16".to_owned()));
        }
        let v = u16::from_le_bytes([self.data[self.pos], self.data[self.pos + 1]]);
        self.pos += 2;
        Ok(v)
    }

    fn read_u32(&mut self) -> Result<u32> {
        if self.pos + 4 > self.data.len() {
            return Err(ExcelError::Xls("truncated SST u32".to_owned()));
        }
        let v = u32::from_le_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        self.pos += 4;
        Ok(v)
    }

    /// 跳过 `cch` 个字符（含 CONTINUE 边界 fresh grbit 处理），不解码。
    fn skip_chars(&mut self, cch: usize, mut compressed: bool) -> Result<()> {
        let mut read = 0usize;
        while read < cch {
            if self.pos >= self.data.len() {
                return Err(ExcelError::Xls(format!(
                    "truncated SST character data: expected {cch} chars, got {read}"
                )));
            }
            if self.at_break() {
                let grbit = self.read_u8()?;
                compressed = grbit & 0x01 == 0;
            }
            if compressed {
                self.pos += 1;
            } else {
                if self.pos + 2 > self.data.len() {
                    return Err(ExcelError::Xls(
                        "UTF-16 code unit split at SST boundary".to_owned(),
                    ));
                }
                self.pos += 2;
            }
            read += 1;
        }
        Ok(())
    }

    /// 跳过一个完整 string entry（header + 字符 + rich runs + extension）。
    fn skip_string(&mut self) -> Result<()> {
        let _cch = self.read_u16()? as usize;
        let flags = self.read_u8()?;
        let compressed = flags & 0x01 == 0;
        let rich = flags & 0x08 != 0;
        let ext = flags & 0x04 != 0;

        let c_run = if rich {
            self.read_u16()? as usize
        } else {
            0
        };
        let cch_ext = if ext {
            self.read_u32()? as usize
        } else {
            0
        };

        self.skip_chars(_cch, compressed)?;

        // 跳过 rich runs (每 run 4 字节)
        let run_bytes = c_run.saturating_mul(4);
        if self.pos + run_bytes > self.data.len() {
            return Err(ExcelError::Xls(
                "truncated SST rich-run data".to_owned(),
            ));
        }
        self.pos += run_bytes;

        // 跳过 extension
        if self.pos + cch_ext > self.data.len() {
            return Err(ExcelError::Xls(
                "truncated SST extension data".to_owned(),
            ));
        }
        self.pos += cch_ext;

        Ok(())
    }

    /// 解码当前位置的单个字符串。
    fn read_string(&mut self, index: usize) -> Result<Biff8SstString> {
        let cch = self.read_u16()? as usize;
        let flags = self.read_u8()?;
        let compressed = flags & 0x01 == 0;
        let rich = flags & 0x08 != 0;
        let ext = flags & 0x04 != 0;

        let c_run = if rich {
            self.read_u16()? as usize
        } else {
            0
        };
        let cch_ext = if ext {
            self.read_u32()? as usize
        } else {
            0
        };

        let text = self.read_chars(cch, compressed, index)?;

        let mut formatting_runs = Vec::with_capacity(c_run);
        for _ in 0..c_run {
            if self.pos + 4 > self.data.len() {
                return Err(ExcelError::Xls(format!(
                    "SST string {index} truncated rich-run data"
                )));
            }
            let char_idx = self.read_u16()?;
            let font_idx = self.read_u16()?;
            formatting_runs.push((char_idx, font_idx));
        }

        // 跳过 extension
        if self.pos + cch_ext > self.data.len() {
            return Err(ExcelError::Xls(format!(
                "SST string {index} truncated extension data"
            )));
        }
        self.pos += cch_ext;

        Ok(Biff8SstString::new(text, formatting_runs))
    }

    /// 解码 `cch` 个字符（含 CONTINUE 边界 fresh grbit 处理）。
    fn read_chars(
        &mut self,
        cch: usize,
        mut compressed: bool,
        index: usize,
    ) -> Result<String> {
        let mut units = Vec::with_capacity(cch.min(16_384));
        let mut read = 0usize;
        while read < cch {
            if self.pos >= self.data.len() {
                return Err(ExcelError::Xls(format!(
                    "SST string {index} truncated: expected {cch} chars, got {read}"
                )));
            }
            if self.at_break() {
                let grbit = self.read_u8()?;
                compressed = grbit & 0x01 == 0;
            }
            if compressed {
                units.push(u16::from(
                    *self
                        .data
                        .get(self.pos)
                        .ok_or_else(|| {
                            ExcelError::Xls(format!("SST string {index} truncated char"))
                        })?,
                ));
                self.pos += 1;
            } else {
                if self.pos + 2 > self.data.len() {
                    return Err(ExcelError::Xls(format!(
                        "SST string {index} UTF-16 code unit split at boundary"
                    )));
                }
                units.push(u16::from_le_bytes([
                    self.data[self.pos],
                    self.data[self.pos + 1],
                ]));
                self.pos += 2;
            }
            read += 1;
        }
        Ok(String::from_utf16_lossy(&units))
    }
}

// ---------------------------------------------------------------------------
// 辅助函数
// ---------------------------------------------------------------------------

#[inline]
fn read_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

// ---------------------------------------------------------------------------
// 测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// 构建最小 SST payload：2 个字符串，无 CONTINUE
    fn make_simple_sst() -> Vec<Vec<u8>> {
        let mut body = Vec::new();
        body.extend_from_slice(&2u32.to_le_bytes()); // total
        body.extend_from_slice(&2u32.to_le_bytes()); // unique
        // string 0: "abc" compressed
        body.extend_from_slice(&3u16.to_le_bytes()); // cch
        body.push(0x00); // flags: compressed
        body.extend_from_slice(b"abc");
        // string 1: UTF-16 "你好"
        body.extend_from_slice(&2u16.to_le_bytes()); // cch
        body.push(0x01); // flags: wide
        body.extend_from_slice(&[0x60, 0x4F, 0x7D, 0x59]); // 你好
        vec![body]
    }

    #[test]
    fn lazy_sst_len_and_get() -> Result<()> {
        let segments = make_simple_sst();
        let sst = LazySst::new(&segments)?;
        assert_eq!(sst.len(), 2);
        assert!(!sst.is_empty());

        let s0 = sst.get(0)?;
        assert_eq!(s0.text, "abc");
        assert!(s0.formatting_runs.is_empty());

        let s1 = sst.get(1)?;
        assert_eq!(s1.text, "你好");
        assert!(s1.formatting_runs.is_empty());
        Ok(())
    }

    #[test]
    fn lazy_sst_out_of_range() {
        let segments = make_simple_sst();
        let sst = LazySst::new(&segments).unwrap();
        assert!(sst.get(2).is_err());
    }

    #[test]
    fn lazy_sst_rich_runs() -> Result<()> {
        let mut body = Vec::new();
        body.extend_from_slice(&1u32.to_le_bytes()); // total
        body.extend_from_slice(&1u32.to_le_bytes()); // unique
        // string with rich text
        body.extend_from_slice(&4u16.to_le_bytes()); // cch
        body.push(0x09); // flags: wide + rich
        body.extend_from_slice(&2u16.to_le_bytes()); // cRun = 2
        body.extend_from_slice(&[0x60, 0x4F, 0x7D, 0x59]); // "你好" (wide, 2 chars x 2 bytes)
        body.extend_from_slice(&[0x60, 0x4F, 0x7D, 0x59]); // "你好" again
        // 2 rich runs
        body.extend_from_slice(&0u16.to_le_bytes()); // char index 0
        body.extend_from_slice(&5u16.to_le_bytes()); // font index 5
        body.extend_from_slice(&2u16.to_le_bytes()); // char index 2
        body.extend_from_slice(&6u16.to_le_bytes()); // font index 6

        let sst = LazySst::new(&[body])?;
        let s = sst.get(0)?;
        assert_eq!(s.text, "你好你好");
        assert_eq!(s.formatting_runs, vec![(0, 5), (2, 6)]);
        Ok(())
    }

    #[test]
    fn lazy_sst_continuation_boundary() -> Result<()> {
        // 字符跨越 CONTINUE 边界，带 fresh grbit 切换
        let mut first = Vec::new();
        first.extend_from_slice(&1u32.to_le_bytes()); // total
        first.extend_from_slice(&1u32.to_le_bytes()); // unique
        first.extend_from_slice(&4u16.to_le_bytes()); // cch = 4
        first.push(0x00); // compressed
        first.extend_from_slice(b"ab"); // 2 chars in first segment

        let second = vec![
            0x00, // fresh grbit: still compressed
            b'c', b'd',
        ];

        let sst = LazySst::new(&[first, second])?;
        let s = sst.get(0)?;
        assert_eq!(s.text, "abcd");
        Ok(())
    }

    #[test]
    fn lazy_sst_continuation_flips_encoding() -> Result<()> {
        // CONTINUE 边界切换编码方式
        let mut first = Vec::new();
        first.extend_from_slice(&1u32.to_le_bytes());
        first.extend_from_slice(&1u32.to_le_bytes());
        first.extend_from_slice(&4u16.to_le_bytes()); // cch = 4
        first.push(0x00); // compressed
        first.push(b'a');
        first.push(b'b');

        let second = vec![
            0x01, // fresh grbit -> wide
            0x60, 0x4F, // '你' UTF-16LE
            0x7D, 0x59, // '好' UTF-16LE
        ];

        let sst = LazySst::new(&[first, second])?;
        let s = sst.get(0)?;
        assert_eq!(s.text, "ab你好");
        Ok(())
    }

    #[test]
    fn lazy_sst_to_vec_matches_eager() -> Result<()> {
        let segments = make_simple_sst();
        let lazy = LazySst::new(&segments)?;
        let vec = lazy.to_vec()?;
        assert_eq!(vec.len(), 2);
        assert_eq!(vec[0].text, "abc");
        assert_eq!(vec[1].text, "你好");
        Ok(())
    }

    #[test]
    fn lazy_sst_rejects_implausible_count() {
        let mut body = Vec::new();
        body.extend_from_slice(&100u32.to_le_bytes());
        body.extend_from_slice(&100u32.to_le_bytes());
        let result = LazySst::new(&[body]);
        assert!(result.is_err());
    }

    #[test]
    fn lazy_sst_extension_data() -> Result<()> {
        let mut body = Vec::new();
        body.extend_from_slice(&1u32.to_le_bytes());
        body.extend_from_slice(&1u32.to_le_bytes());
        body.extend_from_slice(&1u16.to_le_bytes()); // cch = 1
        body.push(0x04); // flags: compressed + extension
        body.extend_from_slice(&2u32.to_le_bytes()); // extension size = 2
        body.push(b'x'); // character
        body.extend_from_slice(&[0xAA, 0xBB]); // extension data

        let sst = LazySst::new(&[body])?;
        let s = sst.get(0)?;
        assert_eq!(s.text, "x");
        Ok(())
    }

    /// 构建包含 N 个字符串的 SST payload（用于性能对比）。
    fn make_large_sst(n: usize) -> Vec<Vec<u8>> {
        let mut body = Vec::new();
        body.extend_from_slice(&(n as u32).to_le_bytes()); // total
        body.extend_from_slice(&(n as u32).to_le_bytes()); // unique
        for i in 0..n {
            let text = format!("string_{i:06}_padding_data");
            let chars: Vec<u16> = text.encode_utf16().collect();
            body.extend_from_slice(&(chars.len() as u16).to_le_bytes());
            body.push(0x00); // compressed
            for &c in &chars {
                body.push(c as u8);
            }
        }
        vec![body]
    }

    #[test]
    fn benchmark_lazy_vs_eager_construction() {
        use std::time::Instant;

        let n = 10_000;
        let segments = make_large_sst(n);

        // 预热
        let _ = super::LazySst::new(&segments);

        // LazySst 构造（仅扫描头部）
        let start = Instant::now();
        let lazy = super::LazySst::new(&segments).unwrap();
        let lazy_build = start.elapsed();

        // 旧路径立即解码（decode_sst_segments）
        let start = Instant::now();
        let eager = super::super::string::decode_sst_segments(&segments).unwrap();
        let eager_build = start.elapsed();

        assert_eq!(lazy.len(), eager.len());
        assert_eq!(lazy.len(), n);

        // LazySst 按需解码全部（最差情况）
        let start = Instant::now();
        for i in 0..n {
            let s = lazy.get(i).unwrap();
            assert_eq!(s.text, eager[i].text);
        }
        let lazy_full_access = start.elapsed();

        eprintln!(
            "SST {n} strings: lazy_build={lazy_build:?}, eager_build={eager_build:?}, \
             lazy_full_access={lazy_full_access:?}, speedup={:.1}x",
            eager_build.as_secs_f64() / lazy_build.as_secs_f64()
        );
    }
}
