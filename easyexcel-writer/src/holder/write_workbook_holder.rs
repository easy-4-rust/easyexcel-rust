//! 对应 Java：`com.alibaba.excel.write.metadata.holder.WriteWorkbookHolder`.

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use crate::MirroredWriteSheetHolder as WriteSheetHolder;
use crate::holder::abstract_write_holder::AbstractWriteHolder;
use crate::metadata::WriteBasicParameter;
use easyexcel_core::WriteHandler;

/// 对应 Java：`WriteWorkbookHolder extends AbstractWriteHolder`.
///
/// The Java side aggregates the `rust_xlsxwriter::Workbook` POI handle, the
/// in-progress sheet holders, and the writer's handler list. Rust holds the
/// same data inside [`crate::ExcelWriter`]; this owned builder-side mirror is
/// retained for Java package/API parity. Runtime callbacks expose the actual
/// logical state through [`easyexcel_core::WriteWorkbookHolderView`].
pub struct WriteWorkbookHolder<'a> {
    abstract_holder: AbstractWriteHolder,
    path: String,
    sheets: HashMap<String, WriteSheetHolder<'a>>,
    handlers: Vec<Box<dyn WriteHandler>>,
}

impl<'a> WriteWorkbookHolder<'a> {
    /// Creates a holder matching the Java `WriteWorkbookHolder(WriteWorkbook)`
    /// initialiser.
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            abstract_holder: AbstractWriteHolder::default(),
            path: path.into(),
            sheets: HashMap::new(),
            handlers: Vec::new(),
        }
    }

    /// Creates a workbook holder from nullable write parameters.
    #[must_use]
    pub fn from_parameter(path: impl Into<String>, parameter: &WriteBasicParameter) -> Self {
        let mut holder = Self::new(path);
        holder.abstract_holder = AbstractWriteHolder::from_parameter(parameter, None);
        holder
    }

    /// Returns the inherited write-holder state.
    #[must_use]
    pub const fn abstract_holder(&self) -> &AbstractWriteHolder {
        &self.abstract_holder
    }

    /// Returns mutable inherited write-holder state.
    pub const fn abstract_holder_mut(&mut self) -> &mut AbstractWriteHolder {
        &mut self.abstract_holder
    }

    /// Returns the output path. (Java `getFile()`)
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns the in-progress sheet holders. (Java `getHasBeenInitializedSheetNameMap()`)
    #[must_use]
    pub fn sheets(&self) -> &HashMap<String, WriteSheetHolder<'a>> {
        &self.sheets
    }

    /// Returns a mutable handle on the in-progress sheet holders.
    pub fn sheets_mut(&mut self) -> &mut HashMap<String, WriteSheetHolder<'a>> {
        &mut self.sheets
    }

    /// Returns the ordered write handler list. (Java `getWriteHandlerList()`)
    #[must_use]
    pub fn handlers(&self) -> &[Box<dyn WriteHandler>] {
        &self.handlers
    }

    /// Appends a handler. (Java `setWriteHandlerList` step)
    pub fn push_handler(&mut self, handler: Box<dyn WriteHandler>) {
        self.handlers.push(handler);
    }
}

impl Deref for WriteWorkbookHolder<'_> {
    type Target = AbstractWriteHolder;

    fn deref(&self) -> &Self::Target {
        &self.abstract_holder
    }
}

impl DerefMut for WriteWorkbookHolder<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.abstract_holder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::write_basic_parameter::WriteBasicParameter;

    #[test]
    fn write_workbook_holder_new() {
        let holder = WriteWorkbookHolder::new("/tmp/out.xlsx");
        assert_eq!(holder.path(), "/tmp/out.xlsx");
        assert!(holder.sheets().is_empty());
        assert!(holder.handlers().is_empty());
    }

    #[test]
    fn write_workbook_holder_from_parameter() {
        let param = WriteBasicParameter::default();
        let holder = WriteWorkbookHolder::from_parameter("/tmp/p.xlsx", &param);
        assert_eq!(holder.path(), "/tmp/p.xlsx");
    }

    #[test]
    fn write_workbook_holder_abstract_holder_accessors() {
        let mut holder = WriteWorkbookHolder::new("/tmp/a.xlsx");
        let _ = holder.abstract_holder();
        let _ = holder.abstract_holder_mut();
    }

    #[test]
    fn write_workbook_holder_sheets_mut() {
        let mut holder = WriteWorkbookHolder::new("/tmp/b.xlsx");
        let _ = holder.sheets_mut();
    }

    #[test]
    fn write_workbook_holder_push_handler() {
        /// No-op `WriteHandler` for testing.
        struct NoopHandler;
        impl WriteHandler for NoopHandler {
            fn order(&self) -> i32 {
                0
            }
        }
        let mut holder = WriteWorkbookHolder::new("/tmp/c.xlsx");
        holder.push_handler(Box::new(NoopHandler));
        assert_eq!(holder.handlers().len(), 1);
        assert_eq!(NoopHandler.order(), 0);
    }

    #[test]
    fn write_workbook_holder_deref() {
        let holder = WriteWorkbookHolder::new("/tmp/d.xlsx");
        let _ = holder.abstract_holder();
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    use easyexcel_core::{ExcelWriteHeadProperty, WriteHandler};

    struct NoopHandler;
    impl WriteHandler for NoopHandler {}

    #[test]
    fn workbook_holder_deref_mut_reaches_abstract_holder() {
        let mut holder = WriteWorkbookHolder::new("out.xlsx");
        holder.set_excel_write_head_property(ExcelWriteHeadProperty::new());
        assert_eq!(NoopHandler.order(), 0);
    }
}
