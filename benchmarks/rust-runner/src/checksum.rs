//! 跨语言稳定的行级校验和。

use sha2::{Digest, Sha256};

use crate::benchmark_row::BenchmarkRow;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 统一表格行校验和累加器。
#[derive(Debug, Default)]
pub(crate) struct RowChecksum(Sha256);

impl RowChecksum {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 按契约的规范字节序列加入一行。
    pub(crate) fn update(&mut self, row: &BenchmarkRow) {
        let canonical = format!(
            "{}\t{}\t{}\t{:016x}\n",
            row.id,
            row.name,
            row.date.format("%Y-%m-%d"),
            row.score.to_bits()
        );
        self.0.update(canonical.as_bytes());
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回小写十六进制 SHA-256。
    pub(crate) fn finish(self) -> String {
        format!("{:x}", self.0.finalize())
    }
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 计算给定规模的期望校验和，不计入被测操作耗时。
pub(crate) fn expected_checksum(rows: u64) -> Result<String, String> {
    let mut checksum = RowChecksum::default();
    for id in 0..rows {
        let id = i64::try_from(id).map_err(|error| error.to_string())?;
        checksum.update(&BenchmarkRow::from_id(id));
    }
    Ok(checksum.finish())
}
