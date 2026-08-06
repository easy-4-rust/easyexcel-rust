use std::io::Read;

use easyexcel_io::{Error, Result};
use pulldown_cmark::{Options, Parser};

use super::markdown_parser_state::MarkdownParserState;
use super::{MarkdownImportOptions, MarkdownReadResult};

/// 从 UTF-8 输入读取 GFM 表格。
pub struct MarkdownReader<R: Read> {
    reader: R,
    options: MarkdownImportOptions,
}

impl<R: Read> MarkdownReader<R> {
    /// 创建 Markdown reader。
    #[must_use]
    pub fn new(reader: R, options: MarkdownImportOptions) -> Self {
        Self { reader, options }
    }

    /// 解析输入并返回中立表格文档和转换报告。
    ///
    /// # Errors
    ///
    /// 输入超过资源限制、不是 UTF-8、没有合法 GFM 表格或解析失败时返回错误。
    pub fn read(self) -> Result<MarkdownReadResult> {
        let limit = self.options.limits().max_file_bytes();
        let mut bytes = Vec::new();
        self.reader
            .take(limit.saturating_add(1))
            .read_to_end(&mut bytes)?;
        let actual = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        if actual > limit {
            return Err(Error::ResourceLimit {
                resource: "file_bytes",
                limit,
                actual,
            });
        }
        let source = String::from_utf8(bytes).map_err(|error| Error::Markdown {
            line: None,
            message: format!("Markdown input must be UTF-8: {error}"),
        })?;
        let mut parser_options = Options::empty();
        parser_options.insert(Options::ENABLE_TABLES);
        let parser = Parser::new_ext(&source, parser_options);
        let mut state = MarkdownParserState::new(&self.options);
        for event in parser {
            state.accept(event)?;
        }
        state.finish()
    }
}

/// 解析任意 UTF-8 Markdown reader。
///
/// # Errors
///
/// 输入超过资源限制、不是 UTF-8、没有合法 GFM 表格或解析失败时返回错误。
pub fn read_markdown<R: Read>(
    reader: R,
    options: &MarkdownImportOptions,
) -> Result<MarkdownReadResult> {
    MarkdownReader::new(reader, options.clone()).read()
}
