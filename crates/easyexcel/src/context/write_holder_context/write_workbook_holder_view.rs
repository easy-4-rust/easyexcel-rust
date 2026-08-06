/// 对应 Java：无直接对应对象；Rust 架构扩展。 Read-only runtime view of Java `WriteWorkbookHolder`.
///
/// The view deliberately exposes logical `EasyExcel` state rather than a fake
/// Apache POI workbook. Backend objects remain owned by the writer engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteWorkbookHolderView {
    path: PathBuf,
}

impl WriteWorkbookHolderView {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Creates a workbook holder view for the active output.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Returns the active output path. (Java `WriteWorkbookHolder.getFile()`)
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

