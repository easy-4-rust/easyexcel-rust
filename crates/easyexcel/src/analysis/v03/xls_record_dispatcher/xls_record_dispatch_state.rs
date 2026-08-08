/// 对应 Java：无直接对应对象；Rust 架构扩展。 Observable result of running Java-compatible BIFF handler dispatch.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct XlsRecordDispatchState {
    last_dummy_event: Option<DummyRecordEvent>,
    total_record_count: usize,
    handled_record_count: usize,
    unknown_record_count: usize,
    skipped_record_count: usize,
    workbook_bof_count: usize,
    worksheet_bof_count: usize,
    eof_count: usize,
    bound_sheets: Vec<BoundSheetEntry>,
    unique_string_count: Option<u32>,
    approximate_total_row_number: Option<u32>,
    last_blank_cell: Option<BlankCell>,
    last_boolean_cell: Option<BoolCell>,
    last_number_cell: Option<NumberCell>,
    last_rk_cell: Option<BlankCell>,
    last_label_cell: Option<LabelCell>,
    shared_strings: Vec<String>,
    rich_shared_strings: Vec<easyexcel_xls::Biff8SstString>,
    last_label_sst_cell: Option<LabelSstCell>,
    last_formula_cell: Option<FormulaCell>,
    extras: Vec<(usize, crate::core::CellExtra)>,
}

impl XlsRecordDispatchState {
    /// Returns the latest POI-compatible missing-cell or end-row event.
    #[must_use]
    pub const fn last_dummy_event(&self) -> Option<&DummyRecordEvent> {
        self.last_dummy_event.as_ref()
    }
    /// Number of physical BIFF records presented to the dispatcher.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn total_record_count(&self) -> usize {
        self.total_record_count
    }

    /// Number of records routed to a registered handler.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn handled_record_count(&self) -> usize {
        self.handled_record_count
    }

    /// Number of records ignored because Java has no registered handler SID.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn unknown_record_count(&self) -> usize {
        self.unknown_record_count
    }

    /// Number of known records skipped by a disabled `support()` predicate.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn skipped_record_count(&self) -> usize {
        self.skipped_record_count
    }

    /// Number of workbook-global BOF records.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn workbook_bof_count(&self) -> usize {
        self.workbook_bof_count
    }

    /// Number of worksheet BOF records.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn worksheet_bof_count(&self) -> usize {
        self.worksheet_bof_count
    }

    /// Number of EOF records.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn eof_count(&self) -> usize {
        self.eof_count
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Bound-sheet metadata decoded by `BoundSheetRecordHandler`.
    #[must_use]
    pub fn bound_sheets(&self) -> &[BoundSheetEntry] {
        &self.bound_sheets
    }

    /// Unique string count announced by the SST record.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn unique_string_count(&self) -> Option<u32> {
        self.unique_string_count
    }

    /// Approximate row count announced by the last Index record.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn approximate_total_row_number(&self) -> Option<u32> {
        self.approximate_total_row_number
    }

    /// Last blank cell decoded by the registered handler.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn last_blank_cell(&self) -> Option<BlankCell> {
        self.last_blank_cell
    }

    /// Last boolean cell decoded by the registered handler.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn last_boolean_cell(&self) -> Option<BoolCell> {
        self.last_boolean_cell
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Last number cell decoded by the registered handler.
    #[must_use]
    pub fn last_number_cell(&self) -> Option<&NumberCell> {
        self.last_number_cell.as_ref()
    }

    /// Last RK placement decoded with `EasyExcel`'s historical empty-cell rule.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn last_rk_cell(&self) -> Option<BlankCell> {
        self.last_rk_cell
    }

    /// 最近一次由 `LabelRecordHandler` 解码的内联字符串单元格。
    #[must_use]
    pub const fn last_label_cell(&self) -> Option<&LabelCell> {
        self.last_label_cell.as_ref()
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Fully decoded shared-string table in BIFF index order.
    #[must_use]
    pub fn shared_strings(&self) -> &[String] {
        &self.shared_strings
    }

    /// 保留 UTF-16 run 与 BIFF8 FONT 索引的共享字符串表。
    #[must_use]
    pub fn rich_shared_strings(&self) -> &[easyexcel_xls::Biff8SstString] {
        &self.rich_shared_strings
    }

    /// Last `LabelSST` cell resolved through the decoded shared-string table.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn last_label_sst_cell(&self) -> Option<&LabelSstCell> {
        self.last_label_sst_cell.as_ref()
    }

    /// Last completed cached formula result.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn last_formula_cell(&self) -> Option<&FormulaCell> {
        self.last_formula_cell.as_ref()
    }

    /// Extra metadata in physical BIFF record order, paired with sheet index.
    #[must_use]
    /// 对应 Java：`AnalysisEventProcessor.extra` 的工作表上下文。
    pub fn extras(&self) -> &[(usize, crate::core::CellExtra)] {
        &self.extras
    }
}
