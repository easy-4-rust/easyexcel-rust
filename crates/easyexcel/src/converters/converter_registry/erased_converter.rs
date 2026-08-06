/// 对应 Java：无直接对应对象；Rust 架构扩展。 Trait-object erase of `Converter<T>` keyed by `TypeId`.
///
/// Mirrors the role of `ConverterKeyBuild.ConverterKey` plus the dispatch
/// through `ConverterKeyBuild.buildKey(Class, CellDataTypeEnum)`. Rust uses
/// `TypeId` because `TypeId` is the type-safe `Class` equivalent.
pub(crate) trait ErasedConverter: Send + Sync {
    fn target_type_id(&self) -> TypeId;
    fn target_type_name(&self) -> &'static str;
    fn support_excel_type(&self) -> CellDataType;
    fn write_target_type(&self) -> Option<CellDataType>;
    fn accepts_null(&self) -> bool;
    fn convert_to_rust_data(
        &self,
        context: &ReadConverterContext<'_>,
    ) -> Result<Box<dyn Any>, ExcelError>;
    fn convert_to_excel_data(
        &self,
        value: &dyn Any,
        column: &ExcelColumn,
        context: &ConvertContext,
    ) -> Result<WriteCellData, ExcelError>;
}

