/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) struct TypedRowConsumer<'a, T> {
    pub(crate) listener: &'a mut dyn ReadListener<T>,
}

impl<T: ExcelRow> RowConsumer for TypedRowConsumer<'_, T> {
    fn process(
        &mut self,
        sheet_no: usize,
        sheet_name: &str,
        row_index: u32,
        cells: Vec<CellValue>,
        metadata: SourceRowMetadata,
        options: &ReadOptions,
        headers: &mut Arc<HashMap<String, usize>>,
    ) -> Result<ReadFlow> {
        process_row_with_metadata::<T>(
            sheet_no,
            sheet_name,
            row_index,
            cells,
            metadata,
            options,
            headers,
            self.listener,
        )
    }

    fn extra(&mut self, extra: &CellExtra, context: &AnalysisContext) -> Result<ReadFlow> {
        let result = self.listener.extra(extra, context);
        listener_result(result, self.listener, context)
    }

    fn after(&mut self, context: &AnalysisContext) -> Result<()> {
        self.listener.do_after_all_analysed(context)
    }
}

