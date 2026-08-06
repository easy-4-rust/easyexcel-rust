//! 对应 Java：`com.alibaba.excel.write.handler.chain.RowHandlerExecutionChain`.

use crate::core::WriteRowContext;

/// 对应 Java：`RowHandlerExecutionChain`.
pub struct RowHandlerExecutionChain {
    pub(crate) handler: Option<Box<dyn crate::core::WriteHandler>>,
    pub(crate) next: Option<Box<RowHandlerExecutionChain>>,
}

impl RowHandlerExecutionChain {
    /// Creates an empty chain head.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.handler.chain.RowHandlerExecutionChain。
    pub const fn new() -> Self {
        Self {
            handler: None,
            next: None,
        }
    }

    /// 对应 Java：com.alibaba.excel.write.handler.chain.RowHandlerExecutionChain。 Creates a chain whose head contains `handler`. (Java constructor)
    #[must_use]
    pub fn with_handler(handler: Box<dyn crate::core::WriteHandler>) -> Self {
        Self {
            handler: Some(handler),
            next: None,
        }
    }

    /// 对应 Java：com.alibaba.excel.write.handler.chain.RowHandlerExecutionChain。 Appends a handler. (Java `addLast`)
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

    /// 对应 Java：com.alibaba.excel.write.handler.chain.RowHandlerExecutionChain。 Runs Java `beforeRowCreate` in chain order.
    /// # Errors
    ///
    /// Propagates errors from the registered handlers (chain stops at the
    /// first failing handler).
    pub fn before_row_create(&mut self, context: &WriteRowContext) -> crate::core::Result<()> {
        if let Some(handler) = self.handler.as_mut() {
            handler.before_row_create(context)?;
        }
        if let Some(next) = self.next.as_mut() {
            next.before_row_create(context)?;
        }
        Ok(())
    }

    /// 对应 Java：com.alibaba.excel.write.handler.chain.RowHandlerExecutionChain。 Runs Java `afterRowCreate` in chain order.
    /// # Errors
    ///
    /// Propagates errors from the registered handlers (chain stops at the
    /// first failing handler).
    pub fn after_row_create(&mut self, context: &WriteRowContext) -> crate::core::Result<()> {
        if let Some(handler) = self.handler.as_mut() {
            handler.after_row_create(context)?;
        }
        if let Some(next) = self.next.as_mut() {
            next.after_row_create(context)?;
        }
        Ok(())
    }

    /// 对应 Java：com.alibaba.excel.write.handler.chain.RowHandlerExecutionChain。 Runs Java `afterRowDispose` in chain order.
    /// # Errors
    ///
    /// Propagates errors from the registered handlers (chain stops at the
    /// first failing handler).
    pub fn after_row_dispose(&mut self, context: &WriteRowContext) -> crate::core::Result<()> {
        if let Some(handler) = self.handler.as_mut() {
            handler.after_row_dispose(context)?;
        }
        if let Some(next) = self.next.as_mut() {
            next.after_row_dispose(context)?;
        }
        Ok(())
    }
}

impl Default for RowHandlerExecutionChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{WriteHandler, WriteRowContext};

    struct NoopHandler;
    impl WriteHandler for NoopHandler {}

    #[test]
    fn row_chain_default_runs_lifecycle_across_nodes() {
        let mut chain = RowHandlerExecutionChain::default();
        chain.add_last(Box::new(NoopHandler));
        chain.add_last(Box::new(NoopHandler));
        let context = WriteRowContext::new("S", 0, None, false);
        chain.before_row_create(&context).unwrap();
        chain.after_row_create(&context).unwrap();
        chain.after_row_dispose(&context).unwrap();
    }

    #[test]
    fn row_chain_with_handler_head_works() {
        let chain = RowHandlerExecutionChain::with_handler(Box::new(NoopHandler));
        let _ = chain;
    }
}
