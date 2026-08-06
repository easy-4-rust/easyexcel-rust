//! 与具体工作簿格式无关的工作表选择。

use easyexcel_utils::string_utils::equals_with_optional_java_trim;

use crate::{Error, Result};

/// 中立的工作表选择请求。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SheetSelection<'a> {
    /// 选择第一张工作表。
    #[default]
    First,
    /// 按零基下标选择工作表。
    Index(usize),
    /// 按名称选择工作表。
    Name(&'a str),
    /// 按工作簿顺序选择全部工作表。
    All,
}

impl SheetSelection<'_> {
    /// 判断工作簿顺序中的一张工作表是否命中当前选择请求。
    ///
    /// `index` 为零基下标；启用 `auto_trim` 时，名称匹配采用 Java
    /// `String#trim` 语义。该方法适用于无法预先收集全部工作表的事件读取器。
    #[must_use]
    pub fn matches(self, index: usize, name: Option<&str>, auto_trim: bool) -> bool {
        match self {
            Self::First => index == 0,
            Self::Index(selected) => index == selected,
            Self::Name(selected) => name.is_some_and(|candidate| {
                equals_with_optional_java_trim(candidate, selected, auto_trim)
            }),
            Self::All => true,
        }
    }
}

/// 从有序工作表名称中解析选择结果。
///
/// 返回值保留原始工作簿下标与名称。启用 `auto_trim` 时，名称匹配采用
/// Java `String#trim` 语义；索引越界或名称不存在时返回强类型错误。
pub fn select_sheet_names(
    names: Vec<String>,
    selection: SheetSelection<'_>,
    auto_trim: bool,
) -> Result<Vec<(usize, String)>> {
    match selection {
        SheetSelection::First => names
            .first()
            .cloned()
            .map(|name| vec![(0, name)])
            .ok_or_else(|| Error::SheetNotFound("0".to_owned())),
        SheetSelection::Index(index) => names
            .get(index)
            .cloned()
            .map(|name| vec![(index, name)])
            .ok_or_else(|| Error::SheetNotFound(index.to_string())),
        SheetSelection::Name(name) => names
            .iter()
            .enumerate()
            .find(|(_, candidate)| equals_with_optional_java_trim(candidate, name, auto_trim))
            .map(|(index, candidate)| vec![(index, candidate.clone())])
            .ok_or_else(|| Error::SheetNotFound(name.to_owned())),
        SheetSelection::All => Ok(names.into_iter().enumerate().collect()),
    }
}

#[cfg(test)]
mod tests {
    use super::SheetSelection;

    #[test]
    fn streaming_match_uses_index_name_and_java_trim_semantics() {
        assert!(SheetSelection::First.matches(0, Some("First"), false));
        assert!(!SheetSelection::First.matches(1, Some("Second"), false));
        assert!(SheetSelection::Index(2).matches(2, None, false));
        assert!(SheetSelection::Name("Data").matches(3, Some(" Data "), true));
        assert!(!SheetSelection::Name("Data").matches(3, Some(" Data "), false));
        assert!(!SheetSelection::Name("Data").matches(3, None, true));
        assert!(SheetSelection::All.matches(99, None, false));
    }
}
