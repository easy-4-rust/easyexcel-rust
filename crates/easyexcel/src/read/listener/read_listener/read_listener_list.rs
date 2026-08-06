/// 对应 Java：无直接对应对象；Rust 架构扩展。 Ordered, dynamically sized Java-style custom listener list.
///
/// This is used by compatibility builders whose listener count is only known
/// at runtime. Rows are cloned because Rust listeners own their argument,
/// while Java listeners receive the same object reference.
pub struct ReadListenerList<T> {
    listeners: Vec<Box<dyn ReadListener<T>>>,
}

impl<T> Default for ReadListenerList<T> {
    fn default() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }
}

impl<T> ReadListenerList<T> {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Creates a list containing its first listener.
    #[must_use]
    pub fn new(listener: impl ReadListener<T> + 'static) -> Self {
        Self {
            listeners: vec![Box::new(listener)],
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Appends a listener in Java registration order.
    pub fn push(&mut self, listener: impl ReadListener<T> + 'static) {
        self.listeners.push(Box::new(listener));
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Appends an already boxed listener.
    pub fn push_boxed(&mut self, listener: Box<dyn ReadListener<T>>) {
        self.listeners.push(listener);
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Returns the registered listener count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.listeners.len()
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Returns whether no listeners are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.listeners.is_empty()
    }
}

impl<T> ReadListener<T> for ReadListenerList<T>
where
    T: Clone,
{
    fn on_exception(&mut self, error: &ExcelError, context: &AnalysisContext) -> ErrorAction {
        self.listeners
            .iter_mut()
            .map(|listener| listener.on_exception(error, context))
            .fold(ErrorAction::Continue, strongest_error_action)
    }

    fn invoke_head(
        &mut self,
        head: &HashMap<String, usize>,
        context: &AnalysisContext,
    ) -> Result<()> {
        for listener in &mut self.listeners {
            listener.invoke_head(head, context)?;
        }
        Ok(())
    }

    fn invoke(&mut self, data: T, context: &AnalysisContext) -> Result<()> {
        for listener in &mut self.listeners {
            listener.invoke(data.clone(), context)?;
        }
        Ok(())
    }

    fn extra(&mut self, extra: &CellExtra, context: &AnalysisContext) -> Result<()> {
        for listener in &mut self.listeners {
            listener.extra(extra, context)?;
        }
        Ok(())
    }

    fn do_after_all_analysed(&mut self, context: &AnalysisContext) -> Result<()> {
        for listener in &mut self.listeners {
            listener.do_after_all_analysed(context)?;
        }
        Ok(())
    }

    fn has_next(&mut self, context: &AnalysisContext) -> bool {
        let mut has_next = true;
        for listener in &mut self.listeners {
            has_next &= listener.has_next(context);
        }
        has_next
    }
}

