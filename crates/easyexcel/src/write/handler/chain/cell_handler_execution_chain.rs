//! 对应 Java：`com.alibaba.excel.write.handler.chain.CellHandlerExecutionChain`.

use crate::core::WriteCellContext;

/// 对应 Java：`CellHandlerExecutionChain` (a single linked-list node).
pub struct CellHandlerExecutionChain {
    pub(crate) handler: Option<Box<dyn crate::core::WriteHandler>>,
    pub(crate) next: Option<Box<CellHandlerExecutionChain>>,
}

impl CellHandlerExecutionChain {
    /// Creates the head of an empty chain.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.handler.chain.CellHandlerExecutionChain。
    pub const fn new() -> Self {
        Self {
            handler: None,
            next: None,
        }
    }

    /// 对应 Java：com.alibaba.excel.write.handler.chain.CellHandlerExecutionChain。 Creates a chain whose head contains `handler`. (Java constructor)
    #[must_use]
    pub fn with_handler(handler: Box<dyn crate::core::WriteHandler>) -> Self {
        Self {
            handler: Some(handler),
            next: None,
        }
    }

    /// 对应 Java：com.alibaba.excel.write.handler.chain.CellHandlerExecutionChain。 Appends a handler. (Java `addLast(WriteHandler)`)
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

    /// 对应 Java：com.alibaba.excel.write.handler.chain.CellHandlerExecutionChain。 Runs the chain's cell lifecycle. (Java `beforeCellCreate`)
    /// # Errors
    ///
    /// Propagates errors from the registered handlers (chain stops at the
    /// first failing handler).
    pub fn before_cell_create(
        &mut self,
        context: &mut WriteCellContext,
    ) -> crate::core::Result<()> {
        if let Some(handler) = self.handler.as_mut() {
            handler.before_cell_create(context)?;
        }
        if let Some(next) = self.next.as_mut() {
            next.before_cell_create(context)?;
        }
        Ok(())
    }

    /// 对应 Java：com.alibaba.excel.write.handler.chain.CellHandlerExecutionChain。 Runs Java `afterCellCreate` in chain order.
    /// # Errors
    ///
    /// Propagates errors from the registered handlers (chain stops at the
    /// first failing handler).
    pub fn after_cell_create(&mut self, context: &WriteCellContext) -> crate::core::Result<()> {
        if let Some(handler) = self.handler.as_mut() {
            handler.after_cell_create(context)?;
        }
        if let Some(next) = self.next.as_mut() {
            next.after_cell_create(context)?;
        }
        Ok(())
    }

    /// 对应 Java：com.alibaba.excel.write.handler.chain.CellHandlerExecutionChain。 Runs Java `afterCellDataConverted` in chain order.
    /// # Errors
    ///
    /// Propagates errors from the registered handlers (chain stops at the
    /// first failing handler).
    pub fn after_cell_data_converted(
        &mut self,
        context: &WriteCellContext,
    ) -> crate::core::Result<()> {
        if let Some(handler) = self.handler.as_mut() {
            handler.after_cell_data_converted(context)?;
        }
        if let Some(next) = self.next.as_mut() {
            next.after_cell_data_converted(context)?;
        }
        Ok(())
    }

    /// 对应 Java：com.alibaba.excel.write.handler.chain.CellHandlerExecutionChain。 Runs Java `afterCellDispose` in chain order.
    /// # Errors
    ///
    /// Propagates errors from the registered handlers (chain stops at the
    /// first failing handler).
    pub fn after_cell_dispose(&mut self, context: &WriteCellContext) -> crate::core::Result<()> {
        if let Some(handler) = self.handler.as_mut() {
            handler.after_cell_dispose(context)?;
        }
        if let Some(next) = self.next.as_mut() {
            next.after_cell_dispose(context)?;
        }
        Ok(())
    }
}

impl Default for CellHandlerExecutionChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{CellValue, WriteCellContext, WriteHandler};

    struct NoopHandler;
    impl WriteHandler for NoopHandler {}

    #[test]
    fn cell_chain_default_runs_lifecycle_across_nodes() {
        let mut chain = CellHandlerExecutionChain::default();
        chain.add_last(Box::new(NoopHandler));
        chain.add_last(Box::new(NoopHandler));
        let mut context = WriteCellContext::new("S", 0, 0, CellValue::Empty);
        chain.before_cell_create(&mut context).unwrap();
        chain.after_cell_create(&context).unwrap();
        chain.after_cell_data_converted(&context).unwrap();
        chain.after_cell_dispose(&context).unwrap();
    }

    #[test]
    fn cell_chain_with_handler_head_works() {
        let chain = CellHandlerExecutionChain::with_handler(Box::new(NoopHandler));
        let _ = chain;
    }
}
