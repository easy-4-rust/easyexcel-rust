//! 对应 Java：`com.alibaba.excel.metadata.property.ExcelHeadProperty`.

use std::collections::BTreeMap;

use crate::HeadKind;

use super::super::configuration_holder::ConfigurationHolder;
use super::super::head::Head;

/// 对应 Java：com.alibaba.excel.metadata.property.ExcelHeadProperty。 Header metadata for one sheet or table.
///
/// Rust port of Java `ExcelHeadProperty`.
#[derive(Debug, Clone, PartialEq)]
pub struct ExcelHeadProperty {
    /// Model type name. (Java `headClazz`)
    pub head_clazz: Option<String>,
    /// Header source kind. (Java `headKind`)
    pub head_kind: HeadKind,
    /// Maximum header row count. (Java `headRowNumber`)
    pub head_row_number: i32,
    /// Column index to header metadata. (Java `headMap`)
    pub head_map: BTreeMap<i32, Head>,
}

impl Default for ExcelHeadProperty {
    fn default() -> Self {
        Self::new(None, None)
    }
}

impl ExcelHeadProperty {
    /// 对应 Java：com.alibaba.excel.metadata.property.ExcelHeadProperty。 Initializes header metadata from holder configuration. (Java constructor)
    ///
    /// # Panics
    ///
    /// 不会 panic（表头行数不可能超过 `i32` 上限，`expect` 仅为静态证明）。
    #[must_use]
    pub fn new(
        _configuration_holder: Option<&dyn ConfigurationHolder>,
        head: Option<Vec<Vec<String>>>,
    ) -> Self {
        let mut property = Self {
            head_clazz: None,
            head_kind: HeadKind::None,
            head_row_number: 0,
            head_map: BTreeMap::new(),
        };

        if let Some(head_rows) = head.filter(|rows| !rows.is_empty()) {
            for (head_index, row) in head_rows.into_iter().enumerate() {
                // 表头行数受内存中字符串表限制，不可能超过 i32::MAX
                let head_index = i32::try_from(head_index).expect("表头行数不可能超过 i32::MAX");
                if let Ok(head) = Head::new(head_index, None, row, false, true) {
                    property.head_map.insert(head_index, head);
                }
            }
            property.head_kind = HeadKind::String;
            property.init_head_row_number();
        }

        property
    }

    /// 对应 Java：com.alibaba.excel.metadata.property.ExcelHeadProperty。 Initializes header metadata for a typed model class. (Java `initColumnProperties`)
    #[must_use]
    pub fn for_class(
        configuration_holder: Option<&dyn ConfigurationHolder>,
        head_clazz: impl Into<String>,
        head: Option<Vec<Vec<String>>>,
    ) -> Self {
        let mut property = Self::new(configuration_holder, head);
        property.head_clazz = Some(head_clazz.into());
        // Java always changes the kind to CLASS after class-field metadata is
        // applied, even when an explicit string head was also supplied.
        property.head_kind = HeadKind::Class;
        property
    }

    /// 对应 Java：com.alibaba.excel.metadata.property.ExcelHeadProperty。 Creates a fully resolved property from an indexed head map.
    ///
    /// This is the Rust equivalent of Java constructing the inherited
    /// `headMap` through `initColumnProperties`. Uneven paths are normalized by
    /// repeating their final label, exactly like `initHeadRowNumber()`.
    #[must_use]
    pub fn from_head_map(
        head_clazz: Option<String>,
        head_kind: HeadKind,
        head_map: BTreeMap<i32, Head>,
    ) -> Self {
        let mut property = Self {
            head_clazz,
            head_kind,
            head_row_number: 0,
            head_map,
        };
        property.init_head_row_number();
        property
    }

    /// Normalizes uneven header row counts. (Java `initHeadRowNumber`)
    fn init_head_row_number(&mut self) {
        self.head_row_number = self
            .head_map
            .values()
            .map(|head| {
                // 表头行数受内存中字符串表限制，不可能超过 i32::MAX
                i32::try_from(head.head_name_list.len()).expect("表头行数不可能超过 i32::MAX")
            })
            .max()
            .unwrap_or(0);

        for head in self.head_map.values_mut() {
            if head.head_name_list.is_empty() {
                continue;
            }
            let last = head.head_name_list.len() - 1;
            // head_row_number 由各表头行数取 max 而来，恒为非负
            let target = usize::try_from(self.head_row_number).expect("head_row_number 恒非负");
            while head.head_name_list.len() < target {
                head.head_name_list.push(head.head_name_list[last].clone());
            }
        }
    }

    /// 对应 Java：com.alibaba.excel.metadata.property.ExcelHeadProperty。 Returns whether any header is configured. (Java `hasHead()`)
    #[must_use]
    pub fn has_head(&self) -> bool {
        self.head_kind != HeadKind::None
    }

    /// 对应 Java：com.alibaba.excel.metadata.property.ExcelHeadProperty。 Returns the model type name. (Java `getHeadClazz()`)
    #[must_use]
    pub fn head_clazz(&self) -> Option<&str> {
        self.head_clazz.as_deref()
    }
    #[must_use] pub fn get_head_clazz(&self) -> Option<&str> { self.head_clazz() }
    pub fn set_head_clazz(&mut self, value: Option<String>) { self.head_clazz = value; }

    /// Returns the header kind. (Java `getHeadKind()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.property.ExcelHeadProperty。
    pub const fn head_kind(&self) -> HeadKind {
        self.head_kind
    }
    #[must_use] pub const fn get_head_kind(&self) -> HeadKind { self.head_kind() }
    pub const fn set_head_kind(&mut self, value: HeadKind) { self.head_kind = value; }

    /// Returns the header row count. (Java `getHeadRowNumber()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.property.ExcelHeadProperty。
    pub const fn head_row_number(&self) -> i32 {
        self.head_row_number
    }

    /// 对应 Java：com.alibaba.excel.metadata.property.ExcelHeadProperty。 Returns the header map. (Java `getHeadMap()`)
    #[must_use]
    pub fn head_map(&self) -> &BTreeMap<i32, Head> {
        &self.head_map
    }
    #[must_use] pub fn get_head_map(&self) -> &BTreeMap<i32, Head> { self.head_map() }
    pub fn set_head_map(&mut self, value: BTreeMap<i32, Head>) {
        self.head_map = value;
        self.init_head_row_number();
    }
}
