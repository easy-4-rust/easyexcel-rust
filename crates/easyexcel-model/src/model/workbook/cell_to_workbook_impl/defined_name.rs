/// 对应 Java：无直接对应对象；Rust 架构扩展。 A workbook-scoped or sheet-scoped defined name (named range / constant).
#[derive(Debug, Clone)]
pub struct DefinedName {
    pub name: String,
    /// The formula text the name refers to (e.g. `Sheet1!$A$1:$B$2`).
    pub refers_to: String,
    /// `None` for workbook scope, else the sheet index it is local to.
    pub scope: Option<usize>,
    pub hidden: bool,
}

