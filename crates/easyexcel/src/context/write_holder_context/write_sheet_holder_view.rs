/// 对应 Java：无直接对应对象；Rust 架构扩展。 Read-only runtime view of Java `WriteSheetHolder`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteSheetHolderView {
    sheet_name: String,
    sheet_no: Option<i32>,
    last_row_index: Option<u32>,
    has_data: bool,
}

impl WriteSheetHolderView {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Creates a view for an active worksheet.
    #[must_use]
    pub fn new(sheet_name: impl Into<String>) -> Self {
        Self {
            sheet_name: sheet_name.into(),
            sheet_no: None,
            last_row_index: None,
            has_data: false,
        }
    }

    /// Records the resolved zero-based sheet number.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn with_sheet_no(mut self, sheet_no: i32) -> Self {
        self.sheet_no = Some(sheet_no);
        self
    }

    /// Records the latest physical row visible at this callback stage.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn with_last_row_index(mut self, last_row_index: u32) -> Self {
        self.last_row_index = Some(last_row_index);
        self.has_data = true;
        self
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Returns the resolved worksheet name.
    #[must_use]
    pub fn sheet_name(&self) -> &str {
        &self.sheet_name
    }

    /// Returns the resolved zero-based sheet number, when known.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn sheet_no(&self) -> Option<i32> {
        self.sheet_no
    }

    /// Returns the latest physical row visible at this callback stage.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn last_row_index(&self) -> Option<u32> {
        self.last_row_index
    }

    /// Returns whether a physical row is visible at this callback stage.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn has_data(&self) -> bool {
        self.has_data
    }
}

