use super::{LexTok, tokenize};

/// BIFF8 工作簿内部 3D 引用的 `SUPBOOK`/`EXTERNSHEET` 分配表。
///
/// 对应 Java：POI `LinkTable`、`InternalSheet` 与 `ExternalSheetRecord`。
#[derive(Debug, Clone, Default)]
pub(crate) struct Biff8LinkTable {
    sheet_count: u16,
    entries: Vec<(String, String, u16, u16)>,
    ixti_base: u16,
    supbook_index: u16,
}

impl Biff8LinkTable {
    #[cfg(test)]
    pub(crate) fn from_formulas(sheet_names: &[String], formulas: &[&str]) -> Self {
        Self::from_formulas_and_references(sheet_names, formulas, &[])
    }

    pub(crate) fn from_formulas_and_references(
        sheet_names: &[String],
        formulas: &[&str],
        references: &[(&str, &str)],
    ) -> Self {
        let sheet_count = u16::try_from(sheet_names.len()).unwrap_or(u16::MAX);
        let mut table = Self {
            sheet_count,
            entries: Vec::new(),
            ixti_base: 0,
            supbook_index: 0,
        };
        for formula in formulas {
            let expr = formula.strip_prefix('=').unwrap_or(formula);
            let Ok(tokens) = tokenize(expr) else {
                continue;
            };
            for token in tokens {
                if let LexTok::Ref3d {
                    first_sheet,
                    last_sheet,
                    ..
                } = token
                {
                    table.register(sheet_names, &first_sheet, &last_sheet);
                }
            }
        }
        for &(first_sheet, last_sheet) in references {
            table.register(sheet_names, first_sheet, last_sheet);
        }
        table
    }

    /// 将新 LinkTable 接到模板已有 SUPBOOK/EXTERNSHEET 表之后。
    #[must_use]
    pub(crate) const fn with_template_offsets(
        mut self,
        ixti_base: u16,
        supbook_index: u16,
    ) -> Self {
        self.ixti_base = ixti_base;
        self.supbook_index = supbook_index;
        self
    }

    fn register(&mut self, sheet_names: &[String], first_sheet: &str, last_sheet: &str) {
        if self.ixti(first_sheet, last_sheet).is_some() {
            return;
        }
        // 只为当前工作簿中真实存在的 Sheet 建立内部引用。未知名称不能用
        // 0xFFFF 伪装成有效 internal SUPBOOK 索引，否则公式编码会成功，最终
        // Excel/POI 才在打开文件时发现损坏。保持未登记可让调用方现有的
        // `ixti(...).ok_or_else(...)` 路径立即 fail-closed。
        let (Some(first), Some(last)) = (
            sheet_index(sheet_names, first_sheet),
            sheet_index(sheet_names, last_sheet),
        ) else {
            return;
        };
        self.entries
            .push((first_sheet.to_owned(), last_sheet.to_owned(), first, last));
    }

    pub(crate) fn ixti(&self, first_sheet: &str, last_sheet: &str) -> Option<u16> {
        self.entries
            .iter()
            .position(|(first, last, _, _)| {
                first.eq_ignore_ascii_case(first_sheet) && last.eq_ignore_ascii_case(last_sheet)
            })
            .and_then(|index| u16::try_from(index).ok())
            .and_then(|index| self.ixti_base.checked_add(index))
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub(crate) fn supbook_payload(&self) -> [u8; 4] {
        let count = self.sheet_count.to_le_bytes();
        [count[0], count[1], 0x01, 0x04]
    }

    pub(crate) fn externsheet_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(2 + self.entries.len() * 6);
        payload.extend_from_slice(
            &u16::try_from(self.entries.len())
                .unwrap_or(u16::MAX)
                .to_le_bytes(),
        );
        for (_, _, first, last) in &self.entries {
            payload.extend_from_slice(&self.supbook_index.to_le_bytes());
            payload.extend_from_slice(&first.to_le_bytes());
            payload.extend_from_slice(&last.to_le_bytes());
        }
        payload
    }
}

fn sheet_index(sheet_names: &[String], name: &str) -> Option<u16> {
    sheet_names
        .iter()
        .position(|sheet| sheet.eq_ignore_ascii_case(name))
        .and_then(|index| u16::try_from(index).ok())
}
