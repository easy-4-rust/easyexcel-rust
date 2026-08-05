/// 智能体与命令行执行时的资源限制。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceLimits {
    max_file_bytes: u64,
    max_sheets: usize,
    max_rows: u64,
    max_formula_cells: u64,
}

impl ResourceLimits {
    /// 创建资源限制。
    #[must_use]
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
        }
    }

    /// 返回最大输入文件字节数。
    #[must_use]
    pub const fn max_file_bytes(&self) -> u64 {
        self.max_file_bytes
    }

    /// 返回最大工作表数量。
    #[must_use]
    pub const fn max_sheets(&self) -> usize {
        self.max_sheets
    }

    /// 返回所有工作表允许的最大总行数。
    #[must_use]
    pub const fn max_rows(&self) -> u64 {
        self.max_rows
    }

    /// 返回允许计算的最大公式单元格数。
    #[must_use]
    pub const fn max_formula_cells(&self) -> u64 {
        self.max_formula_cells
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self::new(256 * 1024 * 1024, 256, 2_000_000, 500_000)
    }
}
