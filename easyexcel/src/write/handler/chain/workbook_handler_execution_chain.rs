//! 对应 Java：`com.alibaba.excel.write.handler.chain.WorkbookHandlerExecutionChain`.

use crate::core::WriteWorkbookContext;

/// 对应 Java：`WorkbookHandlerExecutionChain`.
pub struct WorkbookHandlerExecutionChain {
    pub(crate) handler: Option<Box<dyn crate::core::WriteHandler>>,
    pub(crate) next: Option<Box<WorkbookHandlerExecutionChain>>,
}

impl WorkbookHandlerExecutionChain {
    /// Creates an empty chain head.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            handler: None,
            next: None,
        }
    }

    /// Creates a chain whose head contains `handler`. (Java constructor)
    #[must_use]
    pub fn with_handler(handler: Box<dyn crate::core::WriteHandler>) -> Self {
        Self {
            handler: Some(handler),
            next: None,
        }
    }

    /// Appends a handler. (Java `addLast`)
    pub fn add_last(&mut self, handler: Box<dyn crate::core::WriteHandler>) {
        match self.next.as_mut() {
            Some(next) => next.add_last(handler),
            None => {
                self.next = Some(Box::new(Self {
                    handler: Some(handler),
                    next: None,
                }));
            }
        }
    }

    /// Runs Java `beforeWorkbookCreate` in chain order.
    /// # Errors
    ///
    /// Propagates errors from the registered handlers (chain stops at the
    /// first failing handler).
    pub fn before_workbook_create(
        &mut self,
        context: &WriteWorkbookContext,
    ) -> crate::core::Result<()> {
        if let Some(handler) = self.handler.as_mut() {
            handler.before_workbook_create(context)?;
        }
        if let Some(next) = self.next.as_mut() {
            next.before_workbook_create(context)?;
        }
        Ok(())
    }

    /// Runs Java `afterWorkbookCreate` in chain order.
    /// # Errors
    ///
    /// Propagates errors from the registered handlers (chain stops at the
    /// first failing handler).
    pub fn after_workbook_create(
        &mut self,
        context: &WriteWorkbookContext,
    ) -> crate::core::Result<()> {
        if let Some(handler) = self.handler.as_mut() {
            handler.after_workbook_create(context)?;
        }
        if let Some(next) = self.next.as_mut() {
            next.after_workbook_create(context)?;
        }
        Ok(())
    }

    /// Runs Java `afterWorkbookDispose` in chain order.
    /// # Errors
    ///
    /// Propagates errors from the registered handlers (chain stops at the
    /// first failing handler).
    pub fn after_workbook_dispose(
        &mut self,
        context: &WriteWorkbookContext,
    ) -> crate::core::Result<()> {
        if let Some(handler) = self.handler.as_mut() {
            handler.after_workbook_dispose(context)?;
        }
        if let Some(next) = self.next.as_mut() {
            next.after_workbook_dispose(context)?;
        }
        Ok(())
    }
}

impl Default for WorkbookHandlerExecutionChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{WriteHandler, WriteWorkbookContext};

    struct NoopHandler;
    impl WriteHandler for NoopHandler {}

    #[test]
    fn workbook_chain_default_runs_lifecycle_across_nodes() {
        let mut chain = WorkbookHandlerExecutionChain::default();
        chain.add_last(Box::new(NoopHandler));
        chain.add_last(Box::new(NoopHandler));
        let context = WriteWorkbookContext::new("out.xlsx");
        chain.before_workbook_create(&context).unwrap();
        chain.after_workbook_create(&context).unwrap();
        chain.after_workbook_dispose(&context).unwrap();
    }

    #[test]
    fn workbook_chain_with_handler_head_works() {
        let chain = WorkbookHandlerExecutionChain::with_handler(Box::new(NoopHandler));
        let _ = chain;
    }
}
