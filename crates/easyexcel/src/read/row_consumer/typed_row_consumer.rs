/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) struct TypedRowConsumer<'a, T> {
    pub(crate) listener: &'a mut dyn ReadListener<T>,
}

impl<T: ExcelRow> RowConsumer for TypedRowConsumer<'_, T> {
    fn requires_present_columns(&self) -> bool {
        T::schema().is_empty()
    }

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
        DefaultAnalysisEventProcessor::dispatch_extra(self.listener, extra, context)
    }

    fn after(&mut self, context: &AnalysisContext) -> Result<()> {
        DefaultAnalysisEventProcessor::dispatch_end_sheet(self.listener, context)
    }

    fn process_fast(
        &mut self,
        sheet_no: usize,
        sheet_name: &str,
        row_index: u32,
        mut cells: Vec<CellValue>,
        options: &ReadOptions,
        headers: &mut Arc<HashMap<String, usize>>,
    ) -> Result<ReadFlow> {
        if options.auto_trim {
            trim_string_cells(&mut cells);
        }
        let context = analysis_context(sheet_name, sheet_no, row_index, options);
        if row_index < options.head_row_number {
            let current_headers = Arc::new(header_map(&cells, &options.header_aliases));
            if row_index + 1 == options.head_row_number {
                *headers = Arc::clone(&current_headers);
            }
            return DefaultAnalysisEventProcessor::dispatch_head(
                self.listener,
                &current_headers,
                &context,
            );
        }
        if options.ignore_empty_row && cells.iter().all(is_empty_read_cell) {
            return Ok(ReadFlow::Continue);
        }
        let row = RowData::from_stream_parts(
            sheet_name,
            row_index,
            cells,
            Arc::clone(headers),
            None,
            None,
            None,
            None,
            options.read_default_return,
            options.use_1904_windowing,
        );
        match T::from_row_with_converters(&row, &options.converters) {
            Ok(data) => {
                DefaultAnalysisEventProcessor::dispatch_data(self.listener, data, &context)
            }
            Err(error) => DefaultAnalysisEventProcessor::dispatch_error(self.listener, error, &context),
        }
    }
}
