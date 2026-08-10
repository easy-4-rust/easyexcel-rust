//! 对应 Java：`com.alibaba.excel.write.handler.chain.SheetHandlerExecutionChain`.

use crate::core::WriteSheetContext;

/// 对应 Java：`SheetHandlerExecutionChain`.
pub struct SheetHandlerExecutionChain {
    pub(crate) handler: Option<Box<dyn crate::core::WriteHandler>>,
    pub(crate) next: Option<Box<SheetHandlerExecutionChain>>,
}

impl SheetHandlerExecutionChain {
    /// Java `getHandler`。
    #[must_use]
    pub fn get_handler(&self) -> Option<&dyn crate::core::WriteHandler> {
        self.handler.as_deref()
    }
    /// Java `setHandler`。
    pub fn set_handler(&mut self, value: Option<Box<dyn crate::core::WriteHandler>>) {
        self.handler = value;
    }
    /// Java `getNext`。
    #[must_use]
    pub fn get_next(&self) -> Option<&SheetHandlerExecutionChain> { self.next.as_deref() }
    /// Java `setNext`。
    pub fn set_next(&mut self, value: Option<SheetHandlerExecutionChain>) {
        self.next = value.map(Box::new);
    }
    /// Creates an empty chain head.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.handler.chain.SheetHandlerExecutionChain。
    pub const fn new() -> Self {
        Self {
            handler: None,
            next: None,
        }
    }

    /// 对应 Java：com.alibaba.excel.write.handler.chain.SheetHandlerExecutionChain。 Creates a chain whose head contains `handler`. (Java constructor)
    #[must_use]
    pub fn with_handler(handler: Box<dyn crate::core::WriteHandler>) -> Self {
        Self {
            handler: Some(handler),
            next: None,
        }
    }

    /// 对应 Java：com.alibaba.excel.write.handler.chain.SheetHandlerExecutionChain。 Appends a handler. (Java `addLast`)
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

    /// 对应 Java：com.alibaba.excel.write.handler.chain.SheetHandlerExecutionChain。 Runs Java `beforeSheetCreate` in chain order.
    /// # Errors
    ///
    /// Propagates errors from the registered handlers (chain stops at the
    /// first failing handler).
    pub fn before_sheet_create(&mut self, context: &WriteSheetContext) -> crate::core::Result<()> {
        if let Some(handler) = self.handler.as_mut() {
            handler.before_sheet_create(context)?;
        }
        if let Some(next) = self.next.as_mut() {
            next.before_sheet_create(context)?;
        }
        Ok(())
    }

    /// 对应 Java：com.alibaba.excel.write.handler.chain.SheetHandlerExecutionChain。 Runs Java `afterSheetCreate` in chain order.
    /// # Errors
    ///
    /// Propagates errors from the registered handlers (chain stops at the
    /// first failing handler).
    pub fn after_sheet_create(&mut self, context: &WriteSheetContext) -> crate::core::Result<()> {
        if let Some(handler) = self.handler.as_mut() {
            handler.after_sheet_create(context)?;
        }
        if let Some(next) = self.next.as_mut() {
            next.after_sheet_create(context)?;
        }
        Ok(())
    }
}

impl Default for SheetHandlerExecutionChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{WriteHandler, WriteSheetContext};

    struct NoopHandler;
    impl WriteHandler for NoopHandler {}

    #[test]
    fn sheet_chain_default_runs_lifecycle_across_nodes() {
        let mut chain = SheetHandlerExecutionChain::default();
        chain.add_last(Box::new(NoopHandler));
        chain.add_last(Box::new(NoopHandler));
        let context = WriteSheetContext::new("S");
        chain.before_sheet_create(&context).unwrap();
        chain.after_sheet_create(&context).unwrap();
    }

    #[test]
    fn sheet_chain_with_handler_head_works() {
        let chain = SheetHandlerExecutionChain::with_handler(Box::new(NoopHandler));
        let _ = chain;
    }
}
