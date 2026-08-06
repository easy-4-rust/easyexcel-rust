//! 对应 Java：`com.alibaba.excel.exception.*`.

pub mod excel_analysis_exception;
pub mod excel_analysis_stop_exception;
pub mod excel_analysis_stop_sheet_exception;
pub mod excel_common_exception;
pub mod excel_data_convert_exception;
pub mod excel_generate_exception;
pub mod excel_runtime_exception;
pub mod excel_write_data_convert_exception;

pub use excel_analysis_exception::ExcelAnalysisException;
pub use excel_analysis_stop_exception::ExcelAnalysisStopException;
pub use excel_analysis_stop_sheet_exception::ExcelAnalysisStopSheetException;
pub use excel_common_exception::ExcelCommonException;
pub use excel_data_convert_exception::ExcelDataConvertException;
pub use excel_generate_exception::ExcelGenerateException;
pub use excel_runtime_exception::ExcelRuntimeException;
pub use excel_write_data_convert_exception::ExcelWriteDataConvertException;
