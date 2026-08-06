impl Workbook {
    /// An empty workbook with a single blank sheet named `Sheet1`.
    #[must_use]
    pub fn new() -> Self {
        Workbook {
            sheets: vec![Sheet::new("Sheet1")],
            styles: StyleTable::default(),
            defined_names: Vec::new(),
            date_system: DateSystem::Date1900,
            metadata: Metadata::default(),
            opaque: Vec::new(),
            active_sheet: 0,
        }
    }

    /// A completely empty workbook with no sheets.
    #[must_use]
    pub fn empty() -> Self {
        Workbook {
            sheets: Vec::new(),
            styles: StyleTable::default(),
            defined_names: Vec::new(),
            date_system: DateSystem::Date1900,
            metadata: Metadata::default(),
            opaque: Vec::new(),
            active_sheet: 0,
        }
    }

    pub fn add_sheet(&mut self, name: impl Into<String>) -> usize {
        self.sheets.push(Sheet::new(name));
        self.sheets.len() - 1
    }

    #[must_use]
    pub fn sheet_by_name(&self, name: &str) -> Option<&Sheet> {
        self.sheets
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case(name))
    }

    #[must_use]
    pub fn sheet_index(&self, name: &str) -> Option<usize> {
        self.sheets
            .iter()
            .position(|s| s.name.eq_ignore_ascii_case(name))
    }

    /// Resolve a structured table reference to its `(sheet index, range)`.
    ///
    /// Supports the common forms: a bare `Table` (→ data body), `Table[Column]`,
    /// `Table[#All]`/`[#Data]`/`[#Headers]`/`[#Totals]`, and the combined
    /// `Table[[#Data],[Column]]`. The `[@...]` this-row form is not supported.
    #[must_use]
    pub fn resolve_structured(&self, raw: &str) -> Option<(usize, CellRange)> {
        let raw = raw.trim();
        let (name, spec) = match raw.find('[') {
            Some(open) => {
                let close = raw.rfind(']')?;
                (raw[..open].trim(), Some(raw[open + 1..close].trim()))
            }
            None => (raw, None),
        };
        let (sidx, table) = self.table_by_name(name)?;

        let Some(spec) = spec else {
            return table.data_range().map(|r| (sidx, r));
        };

        // Collect selectors (an `#`-area and/or a column name).
        let mut selectors: Vec<&str> = Vec::new();
        if spec.contains('[') {
            // Multiple bracketed selectors, e.g. `[#Data],[Amount]`.
            let mut rest = spec;
            while let Some(o) = rest.find('[') {
                let c = rest[o..].find(']')? + o;
                selectors.push(rest[o + 1..c].trim());
                rest = &rest[c + 1..];
            }
        } else {
            selectors.push(spec);
        }

        let mut area = TableArea::Data;
        let mut column: Option<&str> = None;
        for sel in selectors {
            if let Some(a) = parse_table_area(sel) {
                area = a;
            } else if !sel.is_empty() {
                column = Some(sel);
            }
        }

        let base = match area {
            TableArea::All => table.range,
            TableArea::Data => table.data_range()?,
            TableArea::Headers => table_header_range(table)?,
            TableArea::Totals => table_totals_range(table)?,
        };
        let range = match column {
            Some(col) => {
                let off = table.column_index(col)?;
                let c = table.range.start.col + off;
                CellRange::new(
                    CellAddress::new(base.start.row, c),
                    CellAddress::new(base.end.row, c),
                )
            }
            None => base,
        };
        Some((sidx, range))
    }

    /// Find a table by name (or display name), case-insensitive, across all
    /// sheets. Returns the owning sheet index and the table.
    #[must_use]
    pub fn table_by_name(&self, name: &str) -> Option<(usize, &Table)> {
        for (i, s) in self.sheets.iter().enumerate() {
            if let Some(t) = s.tables.iter().find(|t| {
                t.name.eq_ignore_ascii_case(name) || t.display_name.eq_ignore_ascii_case(name)
            }) {
                return Some((i, t));
            }
        }
        None
    }

    pub fn sheet_mut(&mut self, idx: usize) -> Option<&mut Sheet> {
        self.sheets.get_mut(idx)
    }

    /// Display text for a cell, applying its number format (date-aware).
    #[must_use]
    pub fn display_cell(&self, sheet_idx: usize, row: u32, col: u32) -> String {
        let Some(sheet) = self.sheets.get(sheet_idx) else {
            return String::new();
        };
        let value = sheet.value(row, col);
        match value {
            CellValue::Number(n) => {
                if let Some(style_idx) = sheet.style_at(row, col)
                    && let Some(style) = self.styles.get(style_idx)
                    && !style.number_format.is_empty()
                    && !style.number_format.eq_ignore_ascii_case("general")
                {
                    return numfmt::format_value(n, &style.number_format, self.date_system);
                }
                super::value::format_number_general(n)
            }
            other => other.to_display_string(),
        }
    }
}

#[cfg(test)]
#[path = "../workbook_tests/tests.rs"]
mod tests;
