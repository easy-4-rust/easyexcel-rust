/// 对应 Java：无直接对应对象；Rust 架构扩展。 智能体与命令行执行时的资源限制。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
// `max_` 前缀是公共安全契约的一部分，可直接表达每个维度的硬上限。
#[allow(clippy::struct_field_names)]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub struct ResourceLimits {
    max_file_bytes: u64,
    max_sheets: usize,
    max_rows: u64,
    max_formula_cells: u64,
    max_output_bytes: u64,
    max_cell_chars: usize,
    max_columns: usize,
}

impl ResourceLimits {
    /// 创建资源限制。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn new(
        max_file_bytes: u64,
        max_sheets: usize,
        max_rows: u64,
        max_formula_cells: u64,
    ) -> Self {
        Self {
            max_file_bytes,
            max_sheets,
            max_rows,
            max_formula_cells,
            max_output_bytes: 256 * 1024 * 1024,
            max_cell_chars: 1024 * 1024,
            max_columns: 16_384,
        }
    }

    /// 返回最大输入文件字节数。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn max_file_bytes(&self) -> u64 {
        self.max_file_bytes
    }

    /// 返回最大工作表数量。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn max_sheets(&self) -> usize {
        self.max_sheets
    }

    /// 返回所有工作表允许的最大总行数。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn max_rows(&self) -> u64 {
        self.max_rows
    }

    /// 返回允许计算的最大公式单元格数。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn max_formula_cells(&self) -> u64 {
        self.max_formula_cells
    }

    /// 返回最大输出字节数。
    #[must_use]
    pub const fn max_output_bytes(&self) -> u64 {
        self.max_output_bytes
    }

    /// 返回单个文本单元格允许的最大字符数。
    #[must_use]
    pub const fn max_cell_chars(&self) -> usize {
        self.max_cell_chars
    }

    /// 返回单张表允许的最大列数。
    #[must_use]
    pub const fn max_columns(&self) -> usize {
        self.max_columns
    }

    /// 设置最大输出字节数。
    #[must_use]
    pub const fn with_max_output_bytes(mut self, value: u64) -> Self {
        self.max_output_bytes = value;
        self
    }

    /// 设置单个文本单元格允许的最大字符数。
    #[must_use]
    pub const fn with_max_cell_chars(mut self, value: usize) -> Self {
        self.max_cell_chars = value;
        self
    }

    /// 设置单张表允许的最大列数。
    #[must_use]
    pub const fn with_max_columns(mut self, value: usize) -> Self {
        self.max_columns = value;
        self
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self::new(256 * 1024 * 1024, 256, 2_000_000, 500_000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_core_limits_and_defaults_for_extended() {
        let rl = ResourceLimits::new(100, 10, 5000, 2000);
        assert_eq!(rl.max_file_bytes(), 100);
        assert_eq!(rl.max_sheets(), 10);
        assert_eq!(rl.max_rows(), 5000);
        assert_eq!(rl.max_formula_cells(), 2000);
        assert_eq!(rl.max_output_bytes(), 256 * 1024 * 1024);
        assert_eq!(rl.max_cell_chars(), 1024 * 1024);
        assert_eq!(rl.max_columns(), 16_384);
    }

    #[test]
    fn default_has_java_compatible_values() {
        let rl = ResourceLimits::default();
        assert_eq!(rl.max_file_bytes(), 256 * 1024 * 1024);
        assert_eq!(rl.max_sheets(), 256);
        assert_eq!(rl.max_rows(), 2_000_000);
        assert_eq!(rl.max_formula_cells(), 500_000);
    }

    #[test]
    fn builder_methods_override_defaults() {
        let rl = ResourceLimits::default()
            .with_max_output_bytes(1024)
            .with_max_cell_chars(512)
            .with_max_columns(100);
        assert_eq!(rl.max_output_bytes(), 1024);
        assert_eq!(rl.max_cell_chars(), 512);
        assert_eq!(rl.max_columns(), 100);
    }

    #[test]
    fn copy_and_clone_work() {
        let rl = ResourceLimits::new(100, 10, 5000, 2000);
        let rl2 = rl;
        assert_eq!(rl, rl2);
        let rl3 = rl.clone();
        assert_eq!(rl, rl3);
    }

    #[test]
    fn debug_format() {
        let rl = ResourceLimits::new(100, 10, 5000, 2000);
        let dbg = format!("{rl:?}");
        assert!(dbg.contains("ResourceLimits"));
    }
}
