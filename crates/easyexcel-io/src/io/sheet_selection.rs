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
            .find(|(_, candidate)| {
                equals_with_optional_java_trim(candidate, name, auto_trim)
            })
            .map(|(index, candidate)| vec![(index, candidate.clone())])
            .ok_or_else(|| Error::SheetNotFound(name.to_owned())),
        SheetSelection::All => Ok(names.into_iter().enumerate().collect()),
    }
}
