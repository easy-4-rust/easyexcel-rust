/// 对应 Java：无直接对应对象；Rust 架构扩展。 Dispatches every read callback to two listeners in registration order.
///
/// Java stores a list of custom `ReadListener`s on `ReadBasicParameter`.
/// Rust models the same ordered fan-out as a nested, statically typed listener
/// so registering another listener does not require runtime type erasure.
pub struct CompositeReadListener<T, First, Second> {
    first: First,
    second: Second,
    marker: std::marker::PhantomData<fn() -> T>,
}

impl<T, First, Second> CompositeReadListener<T, First, Second> {
    /// Creates an ordered pair where `first` is invoked before `second`.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn new(first: First, second: Second) -> Self {
        Self {
            first,
            second,
            marker: std::marker::PhantomData,
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Returns both listeners after a read completes.
    #[must_use]
    pub fn into_inner(self) -> (First, Second) {
        (self.first, self.second)
    }
}

impl<T, First, Second> ReadListener<T> for CompositeReadListener<T, First, Second>
where
    T: Clone,
    First: ReadListener<T>,
    Second: ReadListener<T>,
{
    fn on_exception(&mut self, error: &ExcelError, context: &AnalysisContext) -> ErrorAction {
        let first_action = self.first.on_exception(error, context);
        let second_action = self.second.on_exception(error, context);
        strongest_error_action(first_action, second_action)
    }

    fn invoke_head(
        &mut self,
        head: &HashMap<String, usize>,
        context: &AnalysisContext,
    ) -> Result<()> {
        self.first.invoke_head(head, context)?;
        self.second.invoke_head(head, context)
    }

    fn invoke(&mut self, data: T, context: &AnalysisContext) -> Result<()> {
        self.first.invoke(data.clone(), context)?;
        self.second.invoke(data, context)
    }

    fn extra(&mut self, extra: &CellExtra, context: &AnalysisContext) -> Result<()> {
        self.first.extra(extra, context)?;
        self.second.extra(extra, context)
    }

    fn do_after_all_analysed(&mut self, context: &AnalysisContext) -> Result<()> {
        self.first.do_after_all_analysed(context)?;
        self.second.do_after_all_analysed(context)
    }

    fn has_next(&mut self, context: &AnalysisContext) -> bool {
        let first_has_next = self.first.has_next(context);
        let second_has_next = self.second.has_next(context);
        first_has_next && second_has_next
    }
}

