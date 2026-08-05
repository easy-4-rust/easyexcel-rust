//! BIFF8 公式 RPN（Ptg 令牌）编码器。
//!
//! 对应 Java：`org.apache.poi.hssf.record.formula.*`（FORMULA 记录 rgce 编码）。
//! 字节布局参考：`[MS-XLS] 2.5.198`（`Ptg` 系列）、Apache POI `ss/formula/ptg`、
//! xlwt `ExcelFormulaParser`（经 `Excel` / `LibreOffice` 实测验证）。
//!
//! 支持：A1 风格引用（含 `$` 绝对/相对）、区域引用、算术/比较/文本运算符、
//! 内建函数（257 个，`[MS-XLS] 2.5.198.7` 索引）、字符串/布尔/错误常量、
//! 百分比、一元正负号、空参数（tMissArg）。暂不支持：跨工作表 3D 引用
//! （返回类型化错误）。

use easyexcel_io::Error as ExcelError;

/// 内建函数表 `(索引, 名称, 最小参数, 最大参数, RVA 变体偏移)`；
/// 偏移 0=R / 0x20=V / 0x40=A。索引源自 [MS-XLS] 2.5.198.7（与 POI
/// `functionMetadata.txt`、xlwt `ExcelMagic.py` 一致，含 Excel 2007+ 的
/// CETAB 区段，如 COUNTIF=346、SUMIF=345、VLOOKUP=102）。
pub(crate) const BUILTIN_FUNCTIONS: &[(u16, &str, u8, u8, u8)] = &[
    (0x0000, "COUNT", 0, 30, 32),
    (0x0001, "IF", 2, 3, 0),
    (0x0002, "ISNA", 1, 1, 32),
    (0x0003, "ISERROR", 1, 1, 32),
    (0x0004, "SUM", 0, 30, 32),
    (0x0005, "AVERAGE", 1, 30, 32),
    (0x0006, "MIN", 1, 30, 32),
    (0x0007, "MAX", 1, 30, 32),
    (0x0008, "ROW", 0, 1, 32),
    (0x0009, "COLUMN", 0, 1, 32),
    (0x000a, "NA", 0, 0, 32),
    (0x000b, "NPV", 2, 30, 32),
    (0x000c, "STDEV", 1, 30, 32),
    (0x000d, "DOLLAR", 1, 2, 32),
    (0x000e, "FIXED", 1, 3, 32),
    (0x000f, "SIN", 1, 1, 32),
    (0x0010, "COS", 1, 1, 32),
    (0x0011, "TAN", 1, 1, 32),
    (0x0012, "ATAN", 1, 1, 32),
    (0x0013, "PI", 0, 0, 32),
    (0x0014, "SQRT", 1, 1, 32),
    (0x0015, "EXP", 1, 1, 32),
    (0x0016, "LN", 1, 1, 32),
    (0x0017, "LOG10", 1, 1, 32),
    (0x0018, "ABS", 1, 1, 32),
    (0x0019, "INT", 1, 1, 32),
    (0x001a, "SIGN", 1, 1, 32),
    (0x001b, "ROUND", 2, 2, 32),
    (0x001c, "LOOKUP", 2, 3, 32),
    (0x001d, "INDEX", 2, 4, 0),
    (0x001e, "REPT", 2, 2, 32),
    (0x001f, "MID", 3, 3, 32),
    (0x0020, "LEN", 1, 1, 32),
    (0x0021, "VALUE", 1, 1, 32),
    (0x0022, "TRUE", 0, 0, 32),
    (0x0023, "FALSE", 0, 0, 32),
    (0x0024, "AND", 1, 30, 32),
    (0x0025, "OR", 1, 30, 32),
    (0x0026, "NOT", 1, 1, 32),
    (0x0027, "MOD", 2, 2, 32),
    (0x0028, "DCOUNT", 3, 3, 32),
    (0x0029, "DSUM", 3, 3, 32),
    (0x002a, "DAVERAGE", 3, 3, 32),
    (0x002b, "DMIN", 3, 3, 32),
    (0x002c, "DMAX", 3, 3, 32),
    (0x002d, "DSTDEV", 3, 3, 32),
    (0x002e, "VAR", 1, 30, 32),
    (0x002f, "DVAR", 3, 3, 32),
    (0x0030, "TEXT", 2, 2, 32),
    (0x0031, "LINEST", 1, 4, 64),
    (0x0032, "TREND", 1, 4, 64),
    (0x0033, "LOGEST", 1, 4, 64),
    (0x0034, "GROWTH", 1, 4, 64),
    (0x0035, "GOTO", 1, 1, 0),
    (0x0037, "RETURN", 1, 1, 32),
    (0x0038, "PV", 3, 5, 32),
    (0x0039, "FV", 3, 5, 32),
    (0x003a, "NPER", 3, 5, 32),
    (0x003b, "PMT", 3, 5, 32),
    (0x003c, "RATE", 3, 6, 32),
    (0x003d, "MIRR", 3, 3, 32),
    (0x003e, "IRR", 1, 2, 32),
    (0x003f, "RAND", 0, 0, 32),
    (0x0040, "MATCH", 2, 3, 32),
    (0x0041, "DATE", 3, 3, 32),
    (0x0042, "TIME", 3, 3, 32),
    (0x0043, "DAY", 1, 1, 32),
    (0x0044, "MONTH", 1, 1, 32),
    (0x0045, "YEAR", 1, 1, 32),
    (0x0046, "WEEKDAY", 1, 2, 32),
    (0x0047, "HOUR", 1, 1, 32),
    (0x0048, "MINUTE", 1, 1, 32),
    (0x0049, "SECOND", 1, 1, 32),
    (0x004a, "NOW", 0, 0, 32),
    (0x004b, "AREAS", 1, 1, 32),
    (0x004c, "ROWS", 1, 1, 32),
    (0x004d, "COLUMNS", 1, 1, 32),
    (0x004e, "OFFSET", 3, 5, 0),
    (0x004f, "ABSREF", 2, 2, 0),
    (0x0050, "RELREF", 2, 2, 0),
    (0x0051, "ARGUMENT", 0, 3, 32),
    (0x0052, "SEARCH", 2, 3, 32),
    (0x0053, "TRANSPOSE", 1, 1, 64),
    (0x0054, "ERROR", 0, 2, 32),
    (0x0056, "TYPE", 1, 1, 32),
    (0x0061, "ATAN2", 2, 2, 32),
    (0x0062, "ASIN", 1, 1, 32),
    (0x0063, "ACOS", 1, 1, 32),
    (0x0064, "CHOOSE", 2, 30, 0),
    (0x0065, "HLOOKUP", 3, 4, 32),
    (0x0066, "VLOOKUP", 3, 4, 32),
    (0x0069, "ISREF", 1, 1, 32),
    (0x006d, "LOG", 1, 2, 32),
    (0x006e, "EXEC", 1, 4, 32),
    (0x006f, "CHAR", 1, 1, 32),
    (0x0070, "LOWER", 1, 1, 32),
    (0x0071, "UPPER", 1, 1, 32),
    (0x0072, "PROPER", 1, 1, 32),
    (0x0073, "LEFT", 1, 2, 32),
    (0x0074, "RIGHT", 1, 2, 32),
    (0x0075, "EXACT", 2, 2, 32),
    (0x0076, "TRIM", 1, 1, 32),
    (0x0077, "REPLACE", 4, 4, 32),
    (0x0078, "SUBSTITUTE", 3, 4, 32),
    (0x0079, "CODE", 1, 1, 32),
    (0x007c, "FIND", 2, 3, 32),
    (0x007d, "CELL", 1, 2, 32),
    (0x007e, "ISERR", 1, 1, 32),
    (0x007f, "ISTEXT", 1, 1, 32),
    (0x0080, "ISNUMBER", 1, 1, 32),
    (0x0081, "ISBLANK", 1, 1, 32),
    (0x0082, "T", 1, 1, 32),
    (0x0083, "N", 1, 1, 32),
    (0x008c, "DATEVALUE", 1, 1, 32),
    (0x008d, "TIMEVALUE", 1, 1, 32),
    (0x008e, "SLN", 3, 3, 32),
    (0x008f, "SYD", 4, 4, 32),
    (0x0090, "DDB", 4, 5, 32),
    (0x0094, "INDIRECT", 1, 2, 0),
    (0x0096, "CALL", 1, 3, 32),
    (0x00a2, "CLEAN", 1, 1, 32),
    (0x00a3, "MDETERM", 1, 1, 32),
    (0x00a4, "MINVERSE", 1, 1, 64),
    (0x00a5, "MMULT", 2, 2, 64),
    (0x00a7, "IPMT", 4, 6, 32),
    (0x00a8, "PPMT", 4, 6, 32),
    (0x00a9, "COUNTA", 0, 30, 32),
    (0x00b7, "PRODUCT", 0, 30, 32),
    (0x00b8, "FACT", 1, 1, 32),
    (0x00bd, "DPRODUCT", 3, 3, 32),
    (0x00be, "ISNONTEXT", 1, 1, 32),
    (0x00c1, "STDEVP", 1, 30, 32),
    (0x00c2, "VARP", 1, 30, 32),
    (0x00c3, "DSTDEVP", 3, 3, 32),
    (0x00c4, "DVARP", 3, 3, 32),
    (0x00c5, "TRUNC", 1, 2, 32),
    (0x00c6, "ISLOGICAL", 1, 1, 32),
    (0x00c7, "DCOUNTA", 3, 3, 32),
    (0x00cc, "USDOLLAR", 1, 2, 32),
    (0x00cc, "YEN", 1, 2, 32),
    (0x00cd, "FINDB", 2, 3, 32),
    (0x00ce, "SEARCHB", 2, 3, 32),
    (0x00cf, "REPLACEB", 4, 4, 32),
    (0x00d0, "LEFTB", 1, 2, 32),
    (0x00d1, "RIGHTB", 1, 2, 32),
    (0x00d2, "MIDB", 3, 3, 32),
    (0x00d3, "LENB", 1, 1, 32),
    (0x00d4, "ROUNDUP", 2, 2, 32),
    (0x00d5, "ROUNDDOWN", 2, 2, 32),
    (0x00d6, "ASC", 1, 1, 32),
    (0x00d7, "DBCS", 1, 1, 32),
    (0x00d7, "JIS", 1, 1, 32),
    (0x00d8, "RANK", 2, 3, 32),
    (0x00db, "ADDRESS", 2, 5, 32),
    (0x00dc, "DAYS360", 2, 3, 32),
    (0x00dd, "TODAY", 0, 0, 32),
    (0x00de, "VDB", 5, 7, 32),
    (0x00e3, "MEDIAN", 1, 30, 32),
    (0x00e4, "SUMPRODUCT", 1, 30, 32),
    (0x00e5, "SINH", 1, 1, 32),
    (0x00e6, "COSH", 1, 1, 32),
    (0x00e7, "TANH", 1, 1, 32),
    (0x00e8, "ASINH", 1, 1, 32),
    (0x00e9, "ACOSH", 1, 1, 32),
    (0x00ea, "ATANH", 1, 1, 32),
    (0x00eb, "DGET", 3, 3, 32),
    (0x00f4, "INFO", 1, 1, 32),
    (0x00f7, "DB", 4, 5, 32),
    (0x00fc, "FREQUENCY", 2, 2, 64),
    (0x0101, "EVALUATE", 1, 1, 32),
    (0x010d, "AVEDEV", 1, 30, 32),
    (0x010e, "BETADIST", 3, 5, 32),
    (0x010f, "GAMMALN", 1, 1, 32),
    (0x0110, "BETAINV", 3, 5, 32),
    (0x0111, "BINOMDIST", 4, 4, 32),
    (0x0112, "CHIDIST", 2, 2, 32),
    (0x0113, "CHIINV", 2, 2, 32),
    (0x0114, "COMBIN", 2, 2, 32),
    (0x0115, "CONFIDENCE", 3, 3, 32),
    (0x0116, "CRITBINOM", 3, 3, 32),
    (0x0117, "EVEN", 1, 1, 32),
    (0x0118, "EXPONDIST", 3, 3, 32),
    (0x0119, "FDIST", 3, 3, 32),
    (0x011a, "FINV", 3, 3, 32),
    (0x011b, "FISHER", 1, 1, 32),
    (0x011c, "FISHERINV", 1, 1, 32),
    (0x011d, "FLOOR", 2, 2, 32),
    (0x011e, "GAMMADIST", 4, 4, 32),
    (0x011f, "GAMMAINV", 3, 3, 32),
    (0x0120, "CEILING", 2, 2, 32),
    (0x0121, "HYPGEOMDIST", 4, 4, 32),
    (0x0122, "LOGNORMDIST", 3, 3, 32),
    (0x0123, "LOGINV", 3, 3, 32),
    (0x0124, "NEGBINOMDIST", 3, 3, 32),
    (0x0125, "NORMDIST", 4, 4, 32),
    (0x0126, "NORMSDIST", 1, 1, 32),
    (0x0127, "NORMINV", 3, 3, 32),
    (0x0128, "NORMSINV", 1, 1, 32),
    (0x0129, "STANDARDIZE", 3, 3, 32),
    (0x012a, "ODD", 1, 1, 32),
    (0x012b, "PERMUT", 2, 2, 32),
    (0x012c, "POISSON", 3, 3, 32),
    (0x012d, "TDIST", 3, 3, 32),
    (0x012e, "WEIBULL", 4, 4, 32),
    (0x012f, "SUMXMY2", 2, 2, 32),
    (0x0130, "SUMX2MY2", 2, 2, 32),
    (0x0131, "SUMX2PY2", 2, 2, 32),
    (0x0132, "CHITEST", 2, 2, 32),
    (0x0133, "CORREL", 2, 2, 32),
    (0x0134, "COVAR", 2, 2, 32),
    (0x0135, "FORECAST", 3, 3, 32),
    (0x0136, "FTEST", 2, 2, 32),
    (0x0137, "INTERCEPT", 2, 2, 32),
    (0x0138, "PEARSON", 2, 2, 32),
    (0x0139, "RSQ", 2, 2, 32),
    (0x013a, "STEYX", 2, 2, 32),
    (0x013b, "SLOPE", 2, 2, 32),
    (0x013c, "TTEST", 4, 4, 32),
    (0x013d, "PROB", 3, 4, 32),
    (0x013e, "DEVSQ", 1, 30, 32),
    (0x013f, "GEOMEAN", 1, 30, 32),
    (0x0140, "HARMEAN", 1, 30, 32),
    (0x0141, "SUMSQ", 0, 30, 32),
    (0x0142, "KURT", 1, 30, 32),
    (0x0143, "SKEW", 1, 30, 32),
    (0x0144, "ZTEST", 2, 3, 32),
    (0x0145, "LARGE", 2, 2, 32),
    (0x0146, "SMALL", 2, 2, 32),
    (0x0147, "QUARTILE", 2, 2, 32),
    (0x0148, "PERCENTILE", 2, 2, 32),
    (0x0149, "PERCENTRANK", 2, 3, 32),
    (0x014a, "MODE", 1, 30, 32),
    (0x014b, "TRIMMEAN", 2, 2, 32),
    (0x014c, "TINV", 2, 2, 32),
    (0x0150, "CONCATENATE", 0, 30, 32),
    (0x0151, "POWER", 2, 2, 32),
    (0x0156, "RADIANS", 1, 1, 32),
    (0x0157, "DEGREES", 1, 1, 32),
    (0x0158, "SUBTOTAL", 2, 30, 32),
    (0x0159, "SUMIF", 2, 3, 32),
    (0x015a, "COUNTIF", 2, 2, 32),
    (0x015b, "COUNTBLANK", 1, 1, 32),
    (0x015e, "ISPMT", 4, 4, 32),
    (0x015f, "DATEDIF", 3, 3, 32),
    (0x0160, "DATESTRING", 1, 1, 32),
    (0x0161, "NUMBERSTRING", 2, 2, 32),
    (0x0162, "ROMAN", 1, 2, 32),
    (0x0166, "GETPIVOTDATA", 2, 30, 32),
    (0x0167, "HYPERLINK", 1, 2, 32),
    (0x0168, "PHONETIC", 1, 1, 32),
    (0x0169, "AVERAGEA", 1, 30, 32),
    (0x016a, "MAXA", 1, 30, 32),
    (0x016b, "MINA", 1, 30, 32),
    (0x016c, "STDEVPA", 1, 30, 32),
    (0x016d, "VARPA", 1, 30, 32),
    (0x016e, "STDEVA", 1, 30, 32),
    (0x016f, "VARA", 1, 30, 32),
];

/// 查找内建函数，返回 `(索引, 最小参数, 最大参数, RVA 偏移)`。
fn find_function(name: &str) -> Option<(u16, u8, u8, u8)> {
    BUILTIN_FUNCTIONS
        .iter()
        .find(|(_, n, _, _, _)| *n == name)
        .map(|(idx, _, mn, mx, delta)| (*idx, *mn, *mx, *delta))
}

fn format_error(formula: &str, detail: &str) -> ExcelError {
    ExcelError::Xls(format!("BIFF8 公式编码失败（{formula}）：{detail}"))
}

// ---------------------------------------------------------------------------
// 词法分析
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum LexTok {
    Number(f64),
    Str(String),
    Bool(bool),
    Err(u8),
    Ref {
        row: u16,
        col: u16,
        row_rel: bool,
        col_rel: bool,
    },
    Colon,
    Name(String),
    LParen,
    RParen,
    Comma,
    Percent,
    BinOp(u8),
    UnaryOp(u8),
}

/// 解析 A1 风格单元格引用（含 `$` 前缀），失败返回 None。
fn parse_reference(text: &str) -> Option<(u16, u16, bool, bool)> {
    let bytes = text.as_bytes();
    let mut i = 0usize;
    let col_abs = bytes.first() == Some(&b'$');
    if col_abs {
        i += 1;
    }
    let col_start = i;
    while i < bytes.len() && bytes[i].is_ascii_alphabetic() {
        i += 1;
    }
    if i == col_start || i - col_start > 3 {
        return None;
    }
    let mut col: u32 = 0;
    for &b in &bytes[col_start..i] {
        col = col * 26 + u32::from(b.to_ascii_uppercase() - b'A') + 1;
    }
    if col > 256 {
        return None;
    }
    // 上界 256 已保证不截断
    #[allow(clippy::cast_possible_truncation)]
    let col = (col - 1) as u16;
    let row_abs = i < bytes.len() && bytes[i] == b'$';
    if row_abs {
        i += 1;
    }
    let row_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == row_start || i != bytes.len() {
        return None;
    }
    let row: u32 = text[row_start..].parse().ok()?;
    if row == 0 || row > 65_536 {
        return None;
    }
    // row 已保证在 1..=65536，减一后必然落在 u16 范围
    #[allow(clippy::cast_possible_truncation)]
    let row = (row - 1) as u16;
    Some((row, col, !row_abs, !col_abs))
}

/// BIFF8 错误常量 → 错误码（[MS-XLS] 2.5.24）。
fn error_constant_code(text: &str) -> Option<u8> {
    Some(match text {
        "#NULL!" => 0x00,
        "#DIV/0!" => 0x07,
        "#VALUE!" => 0x0f,
        "#REF!" => 0x17,
        "#NAME?" => 0x1d,
        "#NUM!" => 0x24,
        "#N/A" => 0x2a,
        "#GETTING_DATA" => 0x2b,
        _ => return None,
    })
}

/// 词法分析：公式字符串 → 令牌流（`=` 前缀已剥离）。
/// 扫描字符串字面量（`""` 转义），`i` 停在结束引号之后。
fn scan_string_literal(chars: &[char], i: &mut usize, expr: &str) -> Result<String, ExcelError> {
    let n = chars.len();
    let mut s = String::new();
    *i += 1;
    let mut closed = false;
    while *i < n {
        if chars[*i] == '"' {
            if *i + 1 < n && chars[*i + 1] == '"' {
                s.push('"');
                *i += 2;
            } else {
                *i += 1;
                closed = true;
                break;
            }
        } else {
            s.push(chars[*i]);
            *i += 1;
        }
    }
    if !closed {
        return Err(format_error(expr, "字符串缺少结束引号"));
    }
    Ok(s)
}

/// 扫描数字字面量（整数/小数/科学计数法），`i` 停在数字之后。
fn scan_number(chars: &[char], i: &mut usize, expr: &str) -> Result<f64, ExcelError> {
    let n = chars.len();
    let start = *i;
    while *i < n && (chars[*i].is_ascii_digit() || chars[*i] == '.') {
        *i += 1;
    }
    if *i < n && (chars[*i] == 'e' || chars[*i] == 'E') {
        *i += 1;
        if *i < n && (chars[*i] == '+' || chars[*i] == '-') {
            *i += 1;
        }
        while *i < n && chars[*i].is_ascii_digit() {
            *i += 1;
        }
    }
    let text: String = chars[start..*i].iter().collect();
    text.parse()
        .map_err(|_| format_error(expr, &format!("非法数字字面量 {text}")))
}

/// 扫描错误常量（如 `#N/A`、`#DIV/0!`），`i` 停在常量之后。
fn scan_error_constant(chars: &[char], i: &mut usize, expr: &str) -> Result<u8, ExcelError> {
    let n = chars.len();
    let start = *i;
    while *i < n
        && !matches!(
            chars[*i],
            ',' | ')' | ' ' | '+' | '-' | '*' | '^' | '&' | '<' | '>' | '=' | '%' | '(' | ':'
        )
    {
        *i += 1;
    }
    let text: String = chars[start..*i].iter().collect();
    error_constant_code(&text).ok_or_else(|| format_error(expr, &format!("未知错误常量 {text}")))
}

/// 扫描标识符（引用/布尔常量/函数名），`i` 停在标识符之后。
fn scan_identifier(chars: &[char], i: &mut usize, expr: &str) -> Result<LexTok, ExcelError> {
    let n = chars.len();
    let start = *i;
    while *i < n
        && (chars[*i].is_ascii_alphanumeric()
            || chars[*i] == '_'
            || chars[*i] == '.'
            || chars[*i] == '$')
    {
        *i += 1;
    }
    let text: String = chars[start..*i].iter().collect();
    let upper = text.to_ascii_uppercase();
    if upper == "TRUE" {
        return Ok(LexTok::Bool(true));
    }
    if upper == "FALSE" {
        return Ok(LexTok::Bool(false));
    }
    if let Some((row, col, row_rel, col_rel)) = parse_reference(&text) {
        return Ok(LexTok::Ref {
            row,
            col,
            row_rel,
            col_rel,
        });
    }
    if text.contains('!') {
        return Err(format_error(
            expr,
            "暂不支持跨工作表引用（如 Sheet2!A1），请使用同表引用",
        ));
    }
    Ok(LexTok::Name(upper))
}

/// 前一个令牌是否期待操作数（用于区分一元/二元 `+` `-`）。
fn expects_operand(toks: &[LexTok]) -> bool {
    match toks.last() {
        None => true,
        Some(t) => matches!(
            t,
            LexTok::BinOp(_) | LexTok::UnaryOp(_) | LexTok::LParen | LexTok::Comma
        ),
    }
}

fn tokenize(expr: &str) -> Result<Vec<LexTok>, ExcelError> {
    let chars: Vec<char> = expr.chars().collect();
    let n = chars.len();
    let mut i = 0usize;
    let mut out: Vec<LexTok> = Vec::new();

    while i < n {
        let c = chars[i];
        match c {
            ' ' | '\t' => i += 1,
            '"' => {
                let s = scan_string_literal(&chars, &mut i, expr)?;
                out.push(LexTok::Str(s));
            }
            '0'..='9' | '.' => {
                let value = scan_number(&chars, &mut i, expr)?;
                out.push(LexTok::Number(value));
            }
            'A'..='Z' | 'a'..='z' | '_' | '$' | '\\' => {
                let tok = scan_identifier(&chars, &mut i, expr)?;
                out.push(tok);
            }
            '#' => {
                let code = scan_error_constant(&chars, &mut i, expr)?;
                out.push(LexTok::Err(code));
            }
            ':' => {
                out.push(LexTok::Colon);
                i += 1;
            }
            '+' | '-' => {
                if expects_operand(&out) {
                    out.push(LexTok::UnaryOp(if c == '-' { 0x13 } else { 0x12 }));
                } else {
                    out.push(LexTok::BinOp(if c == '+' { 0x03 } else { 0x04 }));
                }
                i += 1;
            }
            '*' => {
                out.push(LexTok::BinOp(0x05));
                i += 1;
            }
            '/' => {
                out.push(LexTok::BinOp(0x06));
                i += 1;
            }
            '^' => {
                out.push(LexTok::BinOp(0x07));
                i += 1;
            }
            '&' => {
                out.push(LexTok::BinOp(0x08));
                i += 1;
            }
            '=' => {
                out.push(LexTok::BinOp(0x0b));
                i += 1;
            }
            '<' => {
                if i + 1 < n && chars[i + 1] == '=' {
                    out.push(LexTok::BinOp(0x0a));
                    i += 2;
                } else if i + 1 < n && chars[i + 1] == '>' {
                    out.push(LexTok::BinOp(0x0e));
                    i += 2;
                } else {
                    out.push(LexTok::BinOp(0x09));
                    i += 1;
                }
            }
            '>' => {
                if i + 1 < n && chars[i + 1] == '=' {
                    out.push(LexTok::BinOp(0x0c));
                    i += 2;
                } else {
                    out.push(LexTok::BinOp(0x0d));
                    i += 1;
                }
            }
            '%' => {
                out.push(LexTok::Percent);
                i += 1;
            }
            '(' => {
                out.push(LexTok::LParen);
                i += 1;
            }
            ')' => {
                out.push(LexTok::RParen);
                i += 1;
            }
            ',' => {
                out.push(LexTok::Comma);
                i += 1;
            }
            other => {
                return Err(format_error(expr, &format!("不支持的字符 {other:?}")));
            }
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// 递归下降语法分析 → RPN 输出
// ---------------------------------------------------------------------------

/// RPN 输出令牌（后序发射，与 Ptg 一一对应）。
#[derive(Debug, Clone, PartialEq)]
enum RpnTok {
    Int(i16),
    Num(f64),
    Str(String),
    Bool(bool),
    Err(u8),
    Ref(u16, u16, bool, bool),
    Area(u16, u16, u16, u16, bool, bool, bool, bool),
    Func(u8, u16),
    FuncVar(u8, u8, u16),
    MissArg,
    BinOp(u8),
    UnaryOp(u8),
    Paren,
    Percent,
}

struct Parser<'a> {
    tokens: &'a [LexTok],
    pos: usize,
    formula: &'a str,
    out: Vec<RpnTok>,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&LexTok> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<LexTok> {
        let tok = self.tokens.get(self.pos).cloned();
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn err(&self, detail: &str) -> ExcelError {
        format_error(self.formula, detail)
    }

    fn parse(&mut self) -> Result<Vec<RpnTok>, ExcelError> {
        self.parse_expr()?;
        if let Some(tok) = self.peek() {
            return Err(self.err(&format!("表达式结尾出现多余令牌 {tok:?}")));
        }
        Ok(std::mem::take(&mut self.out))
    }

    // expr := comparison
    fn parse_expr(&mut self) -> Result<(), ExcelError> {
        self.parse_compare()
    }

    // comparison := concat (comp_op concat)*
    fn parse_compare(&mut self) -> Result<(), ExcelError> {
        self.parse_concat()?;
        loop {
            match self.peek() {
                Some(LexTok::BinOp(op)) if matches!(op, 0x09..=0x0e) => {
                    let op = *op;
                    self.next();
                    self.parse_concat()?;
                    self.out.push(RpnTok::BinOp(op));
                }
                _ => return Ok(()),
            }
        }
    }

    // concat := additive ('&' additive)*
    fn parse_concat(&mut self) -> Result<(), ExcelError> {
        self.parse_additive()?;
        while matches!(self.peek(), Some(LexTok::BinOp(0x08))) {
            self.next();
            self.parse_additive()?;
            self.out.push(RpnTok::BinOp(0x08));
        }
        Ok(())
    }

    // additive := multiplicative (('+'|'-') multiplicative)*
    fn parse_additive(&mut self) -> Result<(), ExcelError> {
        self.parse_multiplicative()?;
        loop {
            match self.peek() {
                Some(LexTok::BinOp(op)) if matches!(op, 0x03 | 0x04) => {
                    let op = *op;
                    self.next();
                    self.parse_multiplicative()?;
                    self.out.push(RpnTok::BinOp(op));
                }
                _ => return Ok(()),
            }
        }
    }

    // multiplicative := power (('*'|'/') power)*
    fn parse_multiplicative(&mut self) -> Result<(), ExcelError> {
        self.parse_power()?;
        loop {
            match self.peek() {
                Some(LexTok::BinOp(op)) if matches!(op, 0x05 | 0x06) => {
                    let op = *op;
                    self.next();
                    self.parse_power()?;
                    self.out.push(RpnTok::BinOp(op));
                }
                _ => return Ok(()),
            }
        }
    }

    // power := unary ('^' power)?   （右结合）
    fn parse_power(&mut self) -> Result<(), ExcelError> {
        self.parse_unary()?;
        if matches!(self.peek(), Some(LexTok::BinOp(0x07))) {
            self.next();
            self.parse_power()?;
            self.out.push(RpnTok::BinOp(0x07));
        }
        Ok(())
    }

    // unary := ('-'|'+') unary | postfix
    fn parse_unary(&mut self) -> Result<(), ExcelError> {
        match self.peek() {
            Some(LexTok::UnaryOp(op)) => {
                let op = *op;
                self.next();
                self.parse_unary()?;
                self.out.push(RpnTok::UnaryOp(op));
                Ok(())
            }
            _ => self.parse_postfix(),
        }
    }

    // postfix := primary ('%')*
    fn parse_postfix(&mut self) -> Result<(), ExcelError> {
        self.parse_primary()?;
        while matches!(self.peek(), Some(LexTok::Percent)) {
            self.next();
            self.out.push(RpnTok::Percent);
        }
        Ok(())
    }

    // primary := number | string | bool | error | ref[:ref] | name(args) | (expr)
    fn parse_primary(&mut self) -> Result<(), ExcelError> {
        match self.next() {
            Some(LexTok::Number(v)) => {
                if v.fract() == 0.0 && (-32_768.0..=32_767.0).contains(&v) {
                    // 范围内必然可精确转换为 i16
                    #[allow(clippy::cast_possible_truncation)]
                    self.out.push(RpnTok::Int(v as i16));
                } else {
                    self.out.push(RpnTok::Num(v));
                }
                Ok(())
            }
            Some(LexTok::Str(s)) => {
                self.out.push(RpnTok::Str(s));
                Ok(())
            }
            Some(LexTok::Bool(b)) => {
                self.out.push(RpnTok::Bool(b));
                Ok(())
            }
            Some(LexTok::Err(code)) => {
                self.out.push(RpnTok::Err(code));
                Ok(())
            }
            Some(LexTok::Ref {
                row,
                col,
                row_rel,
                col_rel,
            }) => {
                if matches!(self.peek(), Some(LexTok::Colon)) {
                    self.next();
                    match self.next() {
                        Some(LexTok::Ref {
                            row: row2,
                            col: col2,
                            row_rel: row2_rel,
                            col_rel: col2_rel,
                        }) => {
                            self.out.push(RpnTok::Area(
                                row, row2, col, col2, row_rel, row2_rel, col_rel, col2_rel,
                            ));
                            Ok(())
                        }
                        Some(other) => {
                            Err(self.err(&format!("区域引用后应为单元格引用，得到 {other:?}")))
                        }
                        None => Err(self.err("区域引用缺少结束单元格")),
                    }
                } else {
                    self.out.push(RpnTok::Ref(row, col, row_rel, col_rel));
                    Ok(())
                }
            }
            Some(LexTok::Name(name)) => {
                // 必须为函数调用
                if !matches!(self.peek(), Some(LexTok::LParen)) {
                    return Err(self.err(&format!(
                        "无法解析名称 {name}（命名区域暂不支持，请检查拼写）"
                    )));
                }
                let meta = find_function(&name).ok_or_else(|| {
                    self.err(&format!("未知函数 {name}（BIFF8 内建函数表中不存在）"))
                })?;
                self.next(); // '('
                let args = self.parse_args()?;
                if args < meta.1 || args > meta.2 {
                    return Err(self.err(&format!(
                        "函数 {name} 参数个数 {args} 超出范围 {}..={}",
                        meta.1, meta.2
                    )));
                }
                if meta.1 == meta.2 {
                    self.out.push(RpnTok::Func(0x21 + meta.3, meta.0));
                } else {
                    self.out.push(RpnTok::FuncVar(0x22 + meta.3, args, meta.0));
                }
                Ok(())
            }
            Some(LexTok::LParen) => {
                self.parse_expr()?;
                match self.next() {
                    Some(LexTok::RParen) => {
                        self.out.push(RpnTok::Paren);
                        Ok(())
                    }
                    _ => Err(self.err("括号不匹配：缺少 )")),
                }
            }
            Some(other) => Err(self.err(&format!("此处不允许出现 {other:?}"))),
            None => Err(self.err("表达式意外结束")),
        }
    }

    /// 解析函数参数列表（`(` 已消费，`)` 由调用方消费）。
    /// 空参数以 tMissArg 填充；返回参数个数。
    fn parse_args(&mut self) -> Result<u8, ExcelError> {
        let mut count = 0u8;
        loop {
            if matches!(self.peek(), Some(LexTok::RParen)) {
                self.next();
                return Ok(count);
            }
            if matches!(self.peek(), Some(LexTok::Comma)) {
                // 空参数
                self.next();
                self.out.push(RpnTok::MissArg);
                count += 1;
                continue;
            }
            self.parse_expr()?;
            count += 1;
            match self.peek() {
                Some(LexTok::Comma) => {
                    self.next();
                }
                Some(LexTok::RParen) => {
                    self.next();
                    return Ok(count);
                }
                _ => return Err(self.err("函数参数列表缺少 ) 或 ,")),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Ptg 字节编码
// ---------------------------------------------------------------------------

/// 把公式字符串编码为 BIFF8 FORMULA 记录 rgce（RPN Ptg 令牌数组）。
///
/// `formula` 可带或不带前导 `=`。
///
/// # Errors
///
/// 语法错误、未知函数或不受支持的引用风格时返回 [`ExcelError::Xls`]。
pub fn encode_formula_rpn(formula: &str) -> Result<Vec<u8>, ExcelError> {
    let expr = formula.strip_prefix('=').unwrap_or(formula);
    let tokens = tokenize(expr)?;
    let mut parser = Parser {
        tokens: &tokens,
        pos: 0,
        formula: expr,
        out: Vec::new(),
    };
    let rpn = parser.parse()?;
    let mut out = Vec::new();
    for tok in rpn {
        encode_token(&tok, &mut out)?;
    }
    Ok(out)
}

fn encode_token(tok: &RpnTok, out: &mut Vec<u8>) -> Result<(), ExcelError> {
    match tok {
        RpnTok::Int(v) => {
            // tInt (0x1E)：2 字节有符号短整型
            out.push(0x1e);
            out.extend_from_slice(&v.to_le_bytes());
        }
        RpnTok::Num(v) => {
            // tNum (0x1F)：8 字节 IEEE754
            out.push(0x1f);
            out.extend_from_slice(&v.to_le_bytes());
        }
        RpnTok::Str(s) => {
            // tStr (0x17)：cch(1) + grbit(1) + 字符数据
            let chars: Vec<char> = s.chars().collect();
            if chars.len() > 255 {
                return Err(format_error(s, "公式字符串超过 255 字符"));
            }
            out.push(0x17);
            // 已检查 <= 255，不会截断
            #[allow(clippy::cast_possible_truncation)]
            out.push(chars.len() as u8);
            let compressed: Option<Vec<u8>> = chars
                .iter()
                .map(|c| u8::try_from(u32::from(*c)).ok())
                .collect();
            if let Some(bytes) = compressed {
                out.push(0x00);
                out.extend_from_slice(&bytes);
            } else {
                out.push(0x01);
                for c in chars {
                    out.extend_from_slice(&(c as u32).to_le_bytes()[..2]);
                }
            }
        }
        RpnTok::Bool(b) => {
            out.push(0x1d);
            out.push(u8::from(*b));
        }
        RpnTok::Err(code) => {
            out.push(0x1c);
            out.push(*code);
        }
        RpnTok::MissArg => out.push(0x16),
        RpnTok::Ref(row, col, row_rel, col_rel) => {
            // tRef (0x24)：rw(2) + col(2)，相对标志在 col 字段
            out.push(0x24);
            out.extend_from_slice(&row.to_le_bytes());
            let mut col_field = *col;
            if *row_rel {
                col_field |= 0x8000;
            }
            if *col_rel {
                col_field |= 0x4000;
            }
            out.extend_from_slice(&col_field.to_le_bytes());
        }
        RpnTok::Area(
            rw_first,
            rw_last,
            col_first,
            col_last,
            rw_first_rel,
            rw_last_rel,
            col_first_rel,
            col_last_rel,
        ) => {
            // tArea (0x25)：rwFirst + rwLast + colFirst + colLast
            out.push(0x25);
            out.extend_from_slice(&rw_first.to_le_bytes());
            out.extend_from_slice(&rw_last.to_le_bytes());
            let mut cf = *col_first;
            if *rw_first_rel {
                cf |= 0x8000;
            }
            if *col_first_rel {
                cf |= 0x4000;
            }
            out.extend_from_slice(&cf.to_le_bytes());
            let mut cl = *col_last;
            if *rw_last_rel {
                cl |= 0x8000;
            }
            if *col_last_rel {
                cl |= 0x4000;
            }
            out.extend_from_slice(&cl.to_le_bytes());
        }
        RpnTok::Func(base, ifunc) => {
            // tFunc (0x21/0x41/0x61)：ptg + UShort(ifunc)，3 字节
            out.push(*base);
            out.extend_from_slice(&ifunc.to_le_bytes());
        }
        RpnTok::FuncVar(base, args, ifunc) => {
            // tFuncVar (0x22/0x42/0x62)：ptg + cparams(1) + UShort(ifunc)，4 字节
            out.push(*base);
            out.push(*args);
            out.extend_from_slice(&ifunc.to_le_bytes());
        }
        RpnTok::BinOp(op) | RpnTok::UnaryOp(op) => out.push(*op),
        RpnTok::Paren => out.push(0x15),
        RpnTok::Percent => out.push(0x14),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enc(formula: &str) -> Vec<u8> {
        encode_formula_rpn(formula).unwrap_or_else(|e| panic!("编码失败 {formula}: {e}"))
    }

    fn hex(formula: &str) -> String {
        enc(formula)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn binary_add_encodes_refs_with_relative_flags() {
        // A1+B1 → [tRef A1][tRef B1][tAdd]
        // tRef: 0x24 rw(00 00) col(00 C0: col=0 | rowRel 0x8000 | colRel 0x4000)
        assert_eq!(hex("A1+B1"), "24 00 00 00 c0 24 00 00 01 c0 03");
    }

    #[test]
    fn absolute_and_mixed_refs() {
        // $A$1 → col 字段无相对标志；A$1 → colRel(0x4000)
        assert_eq!(hex("$A$1"), "24 00 00 00 00");
        assert_eq!(hex("A$1"), "24 00 00 00 40");
        assert_eq!(hex("$A1"), "24 00 00 00 80");
    }

    #[test]
    fn integer_and_float_literals() {
        assert_eq!(hex("1+2"), "1e 01 00 1e 02 00 03");
        assert_eq!(hex("1.5"), "1f 00 00 00 00 00 00 f8 3f");
        assert_eq!(hex("32768"), "1f 00 00 00 00 00 00 e0 40");
    }

    #[test]
    fn string_and_bool_and_error() {
        assert_eq!(hex("\"Y\""), "17 01 00 59");
        assert_eq!(hex("\"中文\""), "17 02 01 2d 4e 87 65");
        assert_eq!(hex("TRUE"), "1d 01");
        assert_eq!(hex("#N/A"), "1c 2a");
        assert_eq!(hex("#DIV/0!"), "1c 07");
    }

    #[test]
    fn area_range_encodes_corners() {
        // SUM(A1:B2) → [tArea 0..1 × 0..1][tFuncVar(SUM, 1 参)]
        // tArea: 25 rwFirst(00 80: row0+rel) rwLast(01 80) colFirst(00 c0) colLast(01 c0)
        // SUM 索引 4, V 类 → base 0x42, cparams=1
        assert_eq!(hex("SUM(A1:B2)"), "25 00 00 01 00 00 c0 01 c0 42 01 04 00");
    }

    #[test]
    fn if_with_string_args_and_missing_arg() {
        // IF(A1>0,"Y","N")：A1 tRef, 0 tInt, tGT, "Y", "N", FuncVar(IF,3)
        assert_eq!(
            hex("IF(A1>0,\"Y\",\"N\")"),
            "24 00 00 00 c0 1e 00 00 0d 17 01 00 59 17 01 00 4e 22 03 01 00"
        );
        // IF(A1,,2) → 空参数 tMissArg
        assert_eq!(hex("IF(A1,,2)"), "24 00 00 00 c0 16 1e 02 00 22 03 01 00");
    }

    #[test]
    fn unary_and_percent_and_power() {
        // -2^2 = 4（Excel 语义：一元负号先于幂运算）
        assert_eq!(hex("-2^2"), "1e 02 00 13 1e 02 00 07");
        // 50% → tPercent
        assert_eq!(hex("50%"), "1e 32 00 14");
        // 2^3^2 右结合
        assert_eq!(hex("2^3^2"), "1e 02 00 1e 03 00 1e 02 00 07 07");
    }

    #[test]
    fn parenthesis_emits_tparen() {
        assert_eq!(
            hex("(A1+B1)*2"),
            "24 00 00 00 c0 24 00 00 01 c0 03 15 1e 02 00 05"
        );
    }

    #[test]
    fn concat_and_comparison() {
        assert_eq!(hex("A1&\"x\""), "24 00 00 00 c0 17 01 00 78 08");
        assert_eq!(hex("A1>=2"), "24 00 00 00 c0 1e 02 00 0c");
        assert_eq!(hex("A1<>B1"), "24 00 00 00 c0 24 00 00 01 c0 0e");
    }

    #[test]
    fn fixed_arg_function_uses_tfunc() {
        // ROUND(A1,2) 固定 2 参数 → tFunc(0x41+0x20? ROUND 是 V 类 → 0x41)
        // ROUND 索引 27
        assert_eq!(hex("ROUND(A1,2)"), "24 00 00 00 c0 1e 02 00 41 1b 00");
        // PI() 0 参数固定 → tFunc 0x41, 索引 19
        assert_eq!(hex("PI()"), "41 13 00");
    }

    #[test]
    fn variable_arg_function_uses_tfuncvar() {
        // SUM(A1,B1,2) → 3 参数 → FuncVar 0x42, cparams=3, ifunc=4
        assert_eq!(
            hex("SUM(A1,B1,2)"),
            "24 00 00 00 c0 24 00 00 01 c0 1e 02 00 42 03 04 00"
        );
    }

    #[test]
    fn leading_equals_is_stripped() {
        assert_eq!(enc("=A1+B1"), enc("A1+B1"));
    }

    #[test]
    fn nested_function_and_parens() {
        // IF(SUM(A1:B2)>10,SUM(C1:C2),0)
        let rpn = enc("IF(SUM(A1:B2)>10,SUM(C1:C2),0)");
        // 以 tFuncVar(IF) 结尾：0x42 03 01 00
        assert!(rpn.ends_with(&[0x22, 0x03, 0x01, 0x00]));
        // SUM 出现两次
        assert_eq!(
            rpn.windows(4)
                .filter(|w| *w == [0x42, 0x01, 0x04, 0x00])
                .count(),
            2
        );
    }

    #[test]
    fn errors_are_typed() {
        assert!(encode_formula_rpn("UNKNOWNFN(A1)").is_err());
        assert!(encode_formula_rpn("Sheet2!A1").is_err());
        assert!(encode_formula_rpn("A1:").is_err());
        assert!(encode_formula_rpn("(A1+B1").is_err());
        assert!(encode_formula_rpn("\"unterminated").is_err());
    }

    #[test]
    fn countif_cetab_index_is_encoded() {
        // COUNTIF=346 (0x015a)，固定 2 参数 → tFunc(0x21+0x20=0x41) + UShort
        let rpn = enc("COUNTIF(A1:A10,\">5\")");
        assert!(rpn.ends_with(&[0x41, 0x5a, 0x01]));
    }
}
