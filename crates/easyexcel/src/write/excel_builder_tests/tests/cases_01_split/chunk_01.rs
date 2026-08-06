#[test]
    fn fill_config_initializes_java_defaults_and_preserves_overrides() {
        let mut defaults = FillConfig::new();
        assert_eq!(defaults.direction, None);
        assert!(!defaults.force_new_row);
        assert!(defaults.auto_style);
        assert!(!defaults.has_init());
        defaults.init();
        defaults.init();
        assert!(defaults.has_init());

        let configured = FillConfig::new()
            .direction(WriteDirection::Horizontal)
            .force_new_row(true)
            .auto_style(false);
        assert_eq!(configured.direction, Some(WriteDirection::Horizontal));
        assert!(configured.force_new_row);
        assert!(!configured.auto_style);
    }

#[test]
    fn fill_uses_explicit_excel_type_instead_of_path_extension() {
        let mut builder = ExcelBuilderImpl::from_options(
            "logical.xlsx",
            WriteOptions {
                excel_type: Some(crate::support::ExcelTypeEnum::Csv),
                template_bytes: Some(vec![1]),
                ..WriteOptions::default()
            },
        );
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        let error = builder
            .fill(&DynamicRow::default(), FillConfig::new(), &sheet)
            .expect_err("explicit CSV type must reject template fill");
        assert_eq!(
            error.to_string(),
            "unsupported operation: csv does not support filling data."
        );
    }

#[test]
    fn excel_builder_impl_delegates_add_content_and_finish() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("builder-facade.xlsx");
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        let mut builder = ExcelBuilderImpl::from_options(&path, WriteOptions::default());
        builder
            .add_content(
                [DynamicRow::new({
                    let mut cells = std::collections::BTreeMap::new();
                    cells.insert(0, crate::core::DynamicValue::String("alpha".to_owned()));
                    cells
                })],
                &sheet,
            )
            .expect("add_content should succeed");
        finish_write_context(&mut builder, false)?;
        finish_write_context(&mut builder, false)?;
        assert!(path.exists());
        Ok(())
    }

#[test]
    fn excel_builder_merge_is_applied_on_next_add_content() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("builder-merge.xlsx");
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        let mut builder = ExcelBuilderImpl::from_options(&path, WriteOptions::default());
        builder.merge(0, 0, 0, 1)?;
        builder
            .add_content(
                [DynamicRow::new({
                    let mut cells = std::collections::BTreeMap::new();
                    cells.insert(0, crate::core::DynamicValue::String("merged".to_owned()));
                    cells
                })],
                &sheet,
            )
            .expect("add_content should succeed");
        builder.finish(false)?;
        assert!(path.exists());
        Ok(())
    }

#[test]
    fn write_context_exposes_sheet_and_table_after_add_content() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("builder-context.xlsx");
        let sheet = WriteSheet::<DynamicRow>::new(" Sheet1 ");
        let table = crate::write::ExcelWriterTableBuilder::new()
            .table_no(1)
            .need_head(false)
            .build();
        let mut builder = ExcelBuilderImpl::from_options(&path, WriteOptions::default());
        builder.add_content_with_table([], &sheet, &table)?;

        let holder = builder.write_context().current_write_holder();
        assert_eq!(
            holder.sheet_context().map(WriteSheetContext::sheet_name),
            Some("Sheet1")
        );
        assert_eq!(holder.table_no(), Some(1));
        assert!(holder.workbook_context().is_some());
        Ok(())
    }

#[test]
    fn live_current_write_holder_tracks_resolved_sheet_and_table_state() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("live-holder-context.xlsx");
        let mut builder = ExcelBuilderImpl::from_options(&path, WriteOptions::default());
        let sheet = WriteSheet::<ContextRow>::from_options(WriteOptions {
            sheet_name: " Typed ".to_owned(),
            include_column_indexes: Some(vec![2, 0]),
            order_by_include_column: true,
            relative_head_row_index: 3,
            automatic_merge_head: false,
            dynamic_head: Some(vec![
                vec!["Group".to_owned(), "A*".to_owned()],
                vec!["Group".to_owned(), "B*".to_owned()],
                vec!["Group".to_owned(), "C*".to_owned()],
            ]),
            ..WriteOptions::default()
        });
        builder.add_content([], &sheet)?;

        let sheet_holder = builder.write_context().current_write_holder();
        assert_eq!(sheet_holder.holder_type(), Holder::Sheet);
        assert_eq!(
            sheet_holder
                .sheet_context()
                .map(WriteSheetContext::sheet_name),
            Some("Typed")
        );
        assert!(sheet_holder.need_head());
        assert!(!sheet_holder.automatic_merge_head());
        assert_eq!(sheet_holder.relative_head_row_index(), 3);
        assert!(sheet_holder.order_by_include_column());
        assert_eq!(
            sheet_holder.include_column_indexes(),
            Some([2, 0].as_slice())
        );
        assert_eq!(
            sheet_holder
                .excel_write_head_property()
                .head_row_height_property()
                .map(crate::metadata::RowHeightProperty::height),
            Some(28)
        );
        assert_eq!(
            sheet_holder
                .excel_write_head_property()
                .head_map()
                .values()
                .map(|head| (
                    head.column_index(),
                    head.field_name().map(str::to_owned),
                    head.head_name_list().to_vec(),
                ))
                .collect::<Vec<_>>(),
            vec![
                (
                    Some(0),
                    Some("c".to_owned()),
                    vec!["Group".to_owned(), "C*".to_owned()]
                ),
                (
                    Some(1),
                    Some("a".to_owned()),
                    vec!["Group".to_owned(), "A*".to_owned()]
                ),
            ]
        );

        let table = crate::write::ExcelWriterTableBuilder::new()
            .table_no(7)
            .need_head(false)
            .include_column_field_names(["b"])
            .build();
        builder.add_content_with_table([], &sheet, &table)?;
        let table_holder = builder.write_context().current_write_holder();
        assert_eq!(table_holder.holder_type(), Holder::Table);
        assert_eq!(table_holder.table_no(), Some(7));
        assert!(!table_holder.need_head());
        assert_eq!(
            table_holder.include_column_field_names(),
            Some(["b".to_owned()].as_slice())
        );
        assert_eq!(
            table_holder.include_column_indexes(),
            Some([2, 0].as_slice())
        );
        assert_eq!(
            table_holder
                .excel_write_head_property()
                .head_map()
                .values()
                .map(|head| head.field_name())
                .collect::<Vec<_>>(),
            vec![Some("b"), Some("c"), Some("a")]
        );
        builder.finish(false)?;
        assert!(path.exists());
        Ok(())
    }

#[test]
    fn template_fill_updates_the_same_live_current_holder() -> Result<()> {
        let mut builder = ExcelBuilderImpl::from_options(
            "fill-context.xlsx",
            WriteOptions {
                template_bytes: Some(vec![1]),
                ..WriteOptions::default()
            },
        );
        builder.set_fill_executor(Box::new(ContextFillExecutor));
        let sheet = WriteSheet::<DynamicRow>::from_options(WriteOptions {
            sheet_name: " Fill ".to_owned(),
            need_head: false,
            relative_head_row_index: 4,
            automatic_merge_head: false,
            dynamic_head: Some(vec![vec!["填充列".to_owned()]]),
            ..WriteOptions::default()
        });
        builder.fill(&DynamicRow::default(), FillConfig::new(), &sheet)?;

        let holder = builder.write_context().current_write_holder();
        assert_eq!(holder.holder_type(), Holder::Sheet);
        assert_eq!(
            holder.sheet_context().map(WriteSheetContext::sheet_name),
            Some("Fill")
        );
        assert!(!holder.need_head());
        assert!(!holder.automatic_merge_head());
        assert_eq!(holder.relative_head_row_index(), 4);
        assert_eq!(
            holder
                .excel_write_head_property()
                .head_map()
                .values()
                .flat_map(crate::metadata::Head::head_name_list)
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec!["填充列"]
        );
        Ok(())
    }

/// `FillConfig::default()` must equal the Java-compatible `new()`.
    #[test]
    fn fill_config_default_matches_new() {
        assert_eq!(FillConfig::default(), FillConfig::new());
    }

/// A builder that does not override `fill` must hit the trait's default
    /// rejection path when no template is configured.
    #[test]
    fn default_trait_fill_requires_template() {
        struct StubBuilder {
            context: WriteContextImpl,
        }

        impl ExcelBuilder for StubBuilder {
            fn add_content<T, I>(&mut self, _data: I, _write_sheet: &WriteSheet<T>) -> Result<()>
            where
                T: ExcelRow,
                I: IntoIterator<Item = T>,
            {
                Ok(())
            }

            fn add_content_with_table<T, I>(
                &mut self,
                _data: I,
                _write_sheet: &WriteSheet<T>,
                _write_table: &WriteTable,
            ) -> Result<()>
            where
                T: ExcelRow,
                I: IntoIterator<Item = T>,
            {
                Ok(())
            }

            fn merge(
                &mut self,
                _first_row: u32,
                _last_row: u32,
                _first_col: u16,
                _last_col: u16,
            ) -> Result<()> {
                Ok(())
            }

            fn write_context(&self) -> &dyn WriteContext {
                &self.context
            }

            fn finish(&mut self, _on_exception: bool) -> Result<()> {
                Ok(())
            }
        }

        let mut builder = StubBuilder {
            context: WriteContextImpl::new("stub.xlsx"),
        };
        let sheet = WriteSheet::<DynamicRow>::new("Sheet1");
        let error = builder
            .fill(&DynamicRow::default(), FillConfig::new(), &sheet)
            .expect_err("default fill must be rejected without a template");
        assert_eq!(
            error.to_string(),
            "unsupported operation: Calling the 'fill' method must use a template."
        );

        // The remaining trait surface must be callable on the stub.
        let table = WriteTable::default();
        builder
            .add_content([DynamicRow::default()], &sheet)
            .expect("stub add_content");
        builder
            .add_content_with_table([], &sheet, &table)
            .expect("stub add_content_with_table");
        builder.merge(0, 0, 0, 1).expect("stub merge");
        assert_eq!(
            builder.write_context().current_write_holder().path(),
            std::path::Path::new("stub.xlsx")
        );
        builder.finish(false).expect("stub finish");
    }

/// `into_writer`, `writer_mut`, and `logical_path` surface accessors.
    #[test]
    fn builder_surface_accessors_and_into_writer() -> Result<()> {
        let directory = tempdir()?;
        let path = directory.path().join("writer-ops.xlsx");
        let mut builder = ExcelBuilderImpl::from_options(&path, WriteOptions::default());
        assert_eq!(builder.logical_path(), path.as_path());
        assert!(!builder.writer_mut().has_template_configured());
        let writer = builder.into_writer();
        assert!(!writer.has_template_configured());
        Ok(())
    }

