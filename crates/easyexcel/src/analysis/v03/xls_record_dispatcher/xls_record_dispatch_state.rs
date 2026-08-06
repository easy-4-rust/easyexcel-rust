/// 对应 Java：无直接对应对象；Rust 架构扩展。 Observable result of running Java-compatible BIFF handler dispatch.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct XlsRecordDispatchState {
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
    shared_strings: Vec<String>,
    last_label_sst_cell: Option<LabelSstCell>,
    last_formula_cell: Option<FormulaCell>,
}

impl XlsRecordDispatchState {
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

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Fully decoded shared-string table in BIFF index order.
    #[must_use]
    pub fn shared_strings(&self) -> &[String] {
        &self.shared_strings
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
}

