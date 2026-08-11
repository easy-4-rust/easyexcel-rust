//! OOXML-preserving XLSX template filling.

mod builder_fill_executor;
mod fill_config;
mod fill_engine;
mod fill_wrapper;
mod sheet_fill_state;
mod template_data;
mod template_entry;
mod template_output;
mod template_sheet;
mod template_writer;

pub use builder_fill_executor::{BuilderFillExecutor, create_builder_fill_executor};
pub(crate) use builder_fill_executor::CompiledTemplateFillStyles;
pub use fill_config::{FillConfig, FillConfigBuilder, FillDirection};
pub use fill_wrapper::FillWrapper;
pub use template_data::{IntoTemplateValue, TemplateData};
pub(crate) use template_output::TemplateOutput;
pub use template_sheet::TemplateSheet;
pub use template_writer::{ExcelTemplateWriter, fill_xlsx_template, fill_xlsx_template_list};

#[cfg(test)]
mod tests;
