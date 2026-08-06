/// 对应 Java：无直接对应对象；Rust 架构扩展。 Type-tagged carrier for `Converter<T>`.
///
/// Mirrors `TypedConverter` from the Java side. The marker phantom
/// parameter is the Rust equivalent of `Converter<T>.supportJavaTypeKey()`.
pub(crate) struct TypedConverter<T, C> {
    pub(crate) converter: C,
    pub(crate) write_target_type: Option<CellDataType>,
    pub(crate) accepts_null: bool,
    pub(crate) marker: std::marker::PhantomData<fn() -> T>,
}

impl<T, C> ErasedConverter for TypedConverter<T, C>
where
    T: 'static,
    C: Converter<T> + Send + Sync,
{
    fn target_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn target_type_name(&self) -> &'static str {
        type_name::<T>()
    }

    fn support_excel_type(&self) -> CellDataType {
        self.converter.support_excel_type()
    }

    fn write_target_type(&self) -> Option<CellDataType> {
        self.write_target_type
    }

    fn accepts_null(&self) -> bool {
        self.accepts_null
    }

    fn convert_to_rust_data(
        &self,
        context: &ReadConverterContext<'_>,
    ) -> Result<Box<dyn Any>, ExcelError> {
        self.converter
            .convert_to_rust_data(context)
            .map(|value| Box::new(value) as Box<dyn Any>)
    }

    fn convert_to_excel_data(
        &self,
        value: &dyn Any,
        column: &ExcelColumn,
        context: &ConvertContext,
    ) -> Result<WriteCellData, ExcelError> {
        let value = value.downcast_ref::<T>().ok_or_else(|| {
            ExcelError::Format(format!(
                "registered converter expected Rust type {}",
                type_name::<T>()
            ))
        })?;
        self.converter
            .convert_to_excel_data(&WriteConverterContext::new(value, column, context))
    }
}

