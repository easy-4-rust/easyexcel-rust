//! 对应 Java：`com.alibaba.excel.metadata.CellExtra`.

pub use crate::enums::enum_cell_extra_type::CellExtraType;

/// 对应 Java：com.alibaba.excel.metadata.CellExtra。 Extra worksheet information equivalent to Java `EasyExcel`'s `CellExtra`.
///
/// Java carries `rowIndex / columnIndex` plus the interval bounds. Rust keeps
/// the interval bounds as `first_row_index` / `last_row_index` /
/// `first_column_index` / `last_column_index`, while `AnalysisContext` carries
/// the singular cell coordinates, matching how the Java readers forward the
/// event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellExtra {
    extra_type: CellExtraType,
    text: Option<String>,
    row_index: Option<u32>,
    column_index: Option<usize>,
    first_row_index: u32,
    last_row_index: u32,
    first_column_index: usize,
    last_column_index: usize,
}

impl CellExtra {
    /// Creates a cell or range extra event. (Java `CellExtra(type, text, firstRowIndex, lastRowIndex, firstColumnIndex, lastColumnIndex)`)
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.CellExtra。
    pub const fn new(
        extra_type: CellExtraType,
        text: Option<String>,
        first_row_index: u32,
        last_row_index: u32,
        first_column_index: usize,
        last_column_index: usize,
    ) -> Self {
        Self {
            extra_type,
            text,
            row_index: Some(first_row_index),
            column_index: Some(first_column_index),
            first_row_index,
            last_row_index,
            first_column_index,
            last_column_index,
        }
    }

    /// 创建单单元格额外信息。对应 Java 四参数构造器。
    #[must_use]
    pub const fn for_cell(
        extra_type: CellExtraType,
        text: Option<String>,
        row_index: u32,
        column_index: usize,
    ) -> Self {
        Self::new(
            extra_type,
            text,
            row_index,
            row_index,
            column_index,
            column_index,
        )
    }

    /// 从 `A1` 或 `A1:B2` 范围创建额外信息。对应 Java 字符串范围构造器。
    pub fn from_range(
        extra_type: CellExtraType,
        text: Option<String>,
        range: &str,
    ) -> Result<Self, String> {
        let mut ranges = range.split(':');
        let first = ranges
            .next()
            .ok_or_else(|| "cell range is empty".to_owned())?;
        let last = ranges.next().unwrap_or(first);
        if ranges.next().is_some() {
            return Err(format!("invalid cell range: {range}"));
        }
        let (first_row, first_column) = parse_cell_reference(first)?;
        let (last_row, last_column) = parse_cell_reference(last)?;
        Ok(Self::new(
            extra_type,
            text,
            first_row,
            last_row,
            first_column,
            last_column,
        ))
    }

    /// Returns the extra-data kind. (Java `getType()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.CellExtra。
    pub const fn extra_type(&self) -> CellExtraType {
        self.extra_type
    }

    /// 对应 Java：com.alibaba.excel.metadata.CellExtra。 Returns comment text or hyperlink target; merge events have no text. (Java `getText()`)
    #[must_use]
    pub fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }

    /// Returns the first zero-based row index. (Java `getFirstRowIndex()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.CellExtra。
    pub const fn first_row_index(&self) -> u32 {
        self.first_row_index
    }

    /// Returns the last zero-based row index. (Java `getLastRowIndex()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.CellExtra。
    pub const fn last_row_index(&self) -> u32 {
        self.last_row_index
    }

    /// Returns the first zero-based column index. (Java `getFirstColumnIndex()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.CellExtra。
    pub const fn first_column_index(&self) -> usize {
        self.first_column_index
    }

    /// Returns the last zero-based column index. (Java `getLastColumnIndex()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.CellExtra。
    pub const fn last_column_index(&self) -> usize {
        self.last_column_index
    }

    /// Java `getType` 别名。
    #[must_use]
    pub const fn get_type(&self) -> CellExtraType {
        self.extra_type
    }
    /// Java `setType`。
    pub const fn set_type(&mut self, value: CellExtraType) {
        self.extra_type = value;
    }
    /// Java `getText` 别名。
    #[must_use]
    pub fn get_text(&self) -> Option<&str> {
        self.text.as_deref()
    }
    /// Java `setText`。
    pub fn set_text(&mut self, value: Option<String>) {
        self.text = value;
    }
    /// Java `AbstractCell#getRowIndex`。
    #[must_use]
    pub const fn get_row_index(&self) -> Option<u32> {
        self.row_index
    }
    /// Java `AbstractCell#setRowIndex`。
    pub const fn set_row_index(&mut self, value: Option<u32>) {
        self.row_index = value;
    }
    /// Java `AbstractCell#getColumnIndex`。
    #[must_use]
    pub const fn get_column_index(&self) -> Option<usize> {
        self.column_index
    }
    /// Java `AbstractCell#setColumnIndex`。
    pub const fn set_column_index(&mut self, value: Option<usize>) {
        self.column_index = value;
    }
    /// Java `getFirstRowIndex` 别名。
    #[must_use]
    pub const fn get_first_row_index(&self) -> u32 {
        self.first_row_index
    }
    /// Java `setFirstRowIndex`。
    pub const fn set_first_row_index(&mut self, value: u32) {
        self.first_row_index = value;
    }
    /// Java `getLastRowIndex` 别名。
    #[must_use]
    pub const fn get_last_row_index(&self) -> u32 {
        self.last_row_index
    }
    /// Java `setLastRowIndex`。
    pub const fn set_last_row_index(&mut self, value: u32) {
        self.last_row_index = value;
    }
    /// Java `getFirstColumnIndex` 别名。
    #[must_use]
    pub const fn get_first_column_index(&self) -> usize {
        self.first_column_index
    }
    /// Java `setFirstColumnIndex`。
    pub const fn set_first_column_index(&mut self, value: usize) {
        self.first_column_index = value;
    }
    /// Java `getLastColumnIndex` 别名。
    #[must_use]
    pub const fn get_last_column_index(&self) -> usize {
        self.last_column_index
    }
    /// Java `setLastColumnIndex`。
    pub const fn set_last_column_index(&mut self, value: usize) {
        self.last_column_index = value;
    }
}

fn parse_cell_reference(reference: &str) -> Result<(u32, usize), String> {
    let reference = reference.trim().trim_start_matches('$');
    let mut column = 0usize;
    let mut letters = 0usize;
    let bytes = reference.as_bytes();
    while letters < bytes.len() && bytes[letters].is_ascii_alphabetic() {
        column = column
            .checked_mul(26)
            .and_then(|value| {
                value.checked_add(usize::from(bytes[letters].to_ascii_uppercase() - b'A' + 1))
            })
            .ok_or_else(|| format!("cell column overflows: {reference}"))?;
        letters += 1;
        if letters < bytes.len() && bytes[letters] == b'$' {
            letters += 1;
        }
    }
    if column == 0 || letters == bytes.len() {
        return Err(format!("invalid cell reference: {reference}"));
    }
    let row = reference[letters..]
        .parse::<u32>()
        .map_err(|_| format!("invalid cell row: {reference}"))?;
    if row == 0 {
        return Err(format!("cell row must start at 1: {reference}"));
    }
    Ok((row - 1, column - 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_extra_with_bounds() {
        // 对应 Java：CellExtra 构造器
        let extra = CellExtra::new(CellExtraType::Comment, Some("note".to_owned()), 0, 5, 1, 3);
        assert_eq!(extra.extra_type(), CellExtraType::Comment);
        assert_eq!(extra.text(), Some("note"));
        assert_eq!(extra.first_row_index(), 0);
        assert_eq!(extra.last_row_index(), 5);
        assert_eq!(extra.first_column_index(), 1);
        assert_eq!(extra.last_column_index(), 3);
    }

    #[test]
    fn for_cell_creates_single_cell_extra() {
        // 对应 Java：CellExtra 单单元格构造器
        let extra = CellExtra::for_cell(
            CellExtraType::Hyperlink,
            Some("https://example.com".to_owned()),
            2,
            4,
        );
        assert_eq!(extra.first_row_index(), 2);
        assert_eq!(extra.last_row_index(), 2);
        assert_eq!(extra.first_column_index(), 4);
        assert_eq!(extra.last_column_index(), 4);
    }

    #[test]
    fn from_range_single_cell() {
        // 对应 Java：fromRange 单单元格 "A1"
        let extra =
            CellExtra::from_range(CellExtraType::Comment, Some("text".to_owned()), "A1").unwrap();
        assert_eq!(extra.first_row_index(), 0);
        assert_eq!(extra.first_column_index(), 0);
    }

    #[test]
    fn from_range_cell_range() {
        // 对应 Java：fromRange 范围 "A1:B2"
        let extra = CellExtra::from_range(CellExtraType::Merge, None, "A1:B2").unwrap();
        assert_eq!(extra.first_row_index(), 0);
        assert_eq!(extra.last_row_index(), 1);
        assert_eq!(extra.first_column_index(), 0);
        assert_eq!(extra.last_column_index(), 1);
    }

    #[test]
    fn from_range_rejects_empty() {
        // 对应 Java：空范围报错
        let result = CellExtra::from_range(CellExtraType::Comment, None, "");
        assert!(result.is_err());
    }

    #[test]
    fn from_range_rejects_triple_colon() {
        // 对应 Java：三个冒号报错
        let result = CellExtra::from_range(CellExtraType::Comment, None, "A1:B2:C3");
        assert!(result.is_err());
    }

    #[test]
    fn get_type_and_set_type() {
        // 对应 Java：getType / setType
        let mut extra = CellExtra::new(CellExtraType::Comment, None, 0, 0, 0, 0);
        assert_eq!(extra.get_type(), CellExtraType::Comment);
        extra.set_type(CellExtraType::Hyperlink);
        assert_eq!(extra.extra_type(), CellExtraType::Hyperlink);
    }

    #[test]
    fn get_text_and_set_text() {
        // 对应 Java：getText / setText
        let mut extra = CellExtra::new(CellExtraType::Comment, None, 0, 0, 0, 0);
        assert!(extra.get_text().is_none());
        extra.set_text(Some("hello".to_owned()));
        assert_eq!(extra.get_text(), Some("hello"));
        extra.set_text(None);
        assert!(extra.text().is_none());
    }

    #[test]
    fn row_index_accessor() {
        // 对应 Java：rowIndex getter/setter
        let mut extra = CellExtra::new(CellExtraType::Comment, None, 0, 0, 0, 0);
        assert!(extra.get_row_index().is_some());
        extra.set_row_index(Some(5));
        assert_eq!(extra.get_row_index(), Some(5));
        extra.set_row_index(None);
        assert!(extra.get_row_index().is_none());
    }

    #[test]
    fn column_index_accessor() {
        // 对应 Java：columnIndex getter/setter
        let mut extra = CellExtra::new(CellExtraType::Comment, None, 0, 0, 0, 0);
        assert!(extra.get_column_index().is_some());
        extra.set_column_index(Some(3));
        assert_eq!(extra.get_column_index(), Some(3));
        extra.set_column_index(None);
        assert!(extra.get_column_index().is_none());
    }

    #[test]
    fn first_row_index_setter() {
        // 对应 Java：firstRowIndex setter
        let mut extra = CellExtra::new(CellExtraType::Comment, None, 0, 0, 0, 0);
        extra.set_first_row_index(10);
        assert_eq!(extra.get_first_row_index(), 10);
    }

    #[test]
    fn last_row_index_setter() {
        // 对应 Java：lastRowIndex setter
        let mut extra = CellExtra::new(CellExtraType::Comment, None, 0, 0, 0, 0);
        extra.set_last_row_index(20);
        assert_eq!(extra.get_last_row_index(), 20);
    }

    #[test]
    fn first_column_index_setter() {
        // 对应 Java：firstColumnIndex setter
        let mut extra = CellExtra::new(CellExtraType::Comment, None, 0, 0, 0, 0);
        extra.set_first_column_index(2);
        assert_eq!(extra.get_first_column_index(), 2);
    }

    #[test]
    fn last_column_index_setter() {
        // 对应 Java：lastColumnIndex setter
        let mut extra = CellExtra::new(CellExtraType::Comment, None, 0, 0, 0, 0);
        extra.set_last_column_index(8);
        assert_eq!(extra.get_last_column_index(), 8);
    }

    #[test]
    fn clone_produces_equal() {
        // 对应 Java：clone
        let extra = CellExtra::new(CellExtraType::Comment, Some("t".to_owned()), 1, 2, 3, 4);
        let cloned = extra.clone();
        assert_eq!(extra, cloned);
    }

    #[test]
    fn debug_format_does_not_panic() {
        // 对应 Java：toString
        let extra = CellExtra::new(CellExtraType::Comment, None, 0, 0, 0, 0);
        let _debug = format!("{extra:?}");
    }

    #[test]
    fn parse_cell_reference_a1() {
        // 内部函数：A1 解析
        let (row, col) = parse_cell_reference("A1").unwrap();
        assert_eq!(row, 0);
        assert_eq!(col, 0);
    }

    #[test]
    fn parse_cell_reference_with_dollar() {
        // 内部函数：$A$1 解析
        let (row, col) = parse_cell_reference("$A$1").unwrap();
        assert_eq!(row, 0);
        assert_eq!(col, 0);
    }

    #[test]
    fn parse_cell_reference_rejects_zero_row() {
        // 内部函数：行号 0 报错
        assert!(parse_cell_reference("A0").is_err());
    }

    #[test]
    fn parse_cell_reference_rejects_no_column() {
        // 内部函数：无列号报错
        assert!(parse_cell_reference("1").is_err());
    }
}
