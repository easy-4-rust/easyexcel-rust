//! `EasyExcel` 工作簿和行流的生产级 Markdown 投影引擎。

pub mod markdown;

pub use markdown::{
    MarkdownConversionMode, MarkdownConversionReport, MarkdownExportOptions, MarkdownFormulaPolicy,
    MarkdownHeaderPolicy, MarkdownImportOptions, MarkdownMergePolicy, MarkdownProfile,
    MarkdownReadResult, MarkdownReader, MarkdownSheetSelection, MarkdownTableSelection,
    MarkdownTypeInference, MarkdownValuePolicy, MarkdownWarning, MarkdownWarningCode,
    MarkdownWorkbookWriter, MarkdownWriter, read_markdown, write_document, write_workbook,
};
