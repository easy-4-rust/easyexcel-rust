/// 转义 GFM 表格单元格，保留换行为安全的 `<br>`。
#[must_use]
pub(crate) fn escape_cell(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace("\r\n", "<br>")
        .replace(['\r', '\n'], "<br>")
}

/// 清理并转义工作表标题。
#[must_use]
pub(crate) fn escape_heading(value: &str) -> String {
    escape_cell(value).replace('#', "\\#")
}

/// 转义安全内嵌 HTML 文本。
#[must_use]
pub(crate) fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
