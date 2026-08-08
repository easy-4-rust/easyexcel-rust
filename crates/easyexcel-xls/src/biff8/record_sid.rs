//! BIFF8 记录类型编号。
//!
//! SID 是 BIFF 二进制协议的一部分；Java `EasyExcel` handler 可以保留同名
//! 常量入口，但具体编号只在 XLS 引擎中定义。

/// FORMULA 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const FORMULA_SID: u16 = 0x0006;
/// EOF 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const EOF_SID: u16 = 0x000A;
/// CALCMODE 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const CALC_MODE_SID: u16 = 0x000D;
/// EXTERNSHEET 记录 SID。
/// 对应 Java：POI `ExternSheetRecord`。
pub const EXTERNAL_SHEET_SID: u16 = 0x0017;
/// LBL/NAME 记录 SID。
pub const NAME_SID: u16 = 0x0018;
/// NOTE 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const NOTE_SID: u16 = 0x001C;
/// DATEMODE 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const DATE_MODE_SID: u16 = 0x0022;
/// FILEPASS 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const FILE_PASS_SID: u16 = 0x002F;
/// FONT 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const FONT_SID: u16 = 0x0031;
/// CONTINUE 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const CONTINUE_SID: u16 = 0x003C;
/// PANE 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const PANE_SID: u16 = 0x0041;
/// CODEPAGE 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const CODE_PAGE_SID: u16 = 0x0042;
/// CODENAME 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const CODE_NAME_SID: u16 = 0x004A;
/// WRITEACCESS 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const WRITE_ACCESS_SID: u16 = 0x005C;
/// OBJ 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const OBJ_SID: u16 = 0x005D;
/// COLINFO 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const COLUMN_INFO_SID: u16 = 0x007D;
/// BOUNDSHEET 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const BOUND_SHEET_SID: u16 = 0x0085;
/// PALETTE 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const PALETTE_SID: u16 = 0x0092;
/// MULRK 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const MUL_RK_SID: u16 = 0x00BD;
/// MULBLANK 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const MUL_BLANK_SID: u16 = 0x00BE;
/// MMS 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const MMS_SID: u16 = 0x00C1;
/// XF 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const XF_SID: u16 = 0x00E0;
/// INTERFACEHDR 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const INTERFACE_HEADER_SID: u16 = 0x00E1;
/// INTERFACEEND 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const INTERFACE_END_SID: u16 = 0x00E2;
/// MERGECELLS 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const MERGE_CELLS_SID: u16 = 0x00E5;
/// MSODRAWINGGROUP 记录 SID。
/// 对应 Java：POI `DrawingGroupRecord`。
pub const MSO_DRAWING_GROUP_SID: u16 = 0x00EB;
/// MSODRAWING 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const MSO_DRAWING_SID: u16 = 0x00EC;
/// SST 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const SST_SID: u16 = 0x00FC;
/// LABELSST 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const LABEL_SST_SID: u16 = 0x00FD;
/// EXTSST 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const EXT_SST_SID: u16 = 0x00FF;
/// `TxO` 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const TEXT_OBJECT_SID: u16 = 0x01B6;
/// 旧式条件格式头记录 SID。
pub const CONDITIONAL_FORMATTING_HEADER_SID: u16 = 0x01B0;
/// 旧式条件格式规则记录 SID。
pub const CONDITIONAL_FORMATTING_RULE_SID: u16 = 0x01B1;
/// 工作表数据校验公共记录 SID。
pub const DATA_VALIDATION_HEADER_SID: u16 = 0x01B2;
/// SUPBOOK 记录 SID。
/// 对应 Java：POI `SupBookRecord`。
pub const SUP_BOOK_SID: u16 = 0x01AE;
/// HYPERLINK 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const HYPERLINK_SID: u16 = 0x01B8;
/// 单个数据校验规则记录 SID。
pub const DATA_VALIDATION_SID: u16 = 0x01BE;
/// BLANK 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const BLANK_SID: u16 = 0x0201;
/// DIMENSION 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const DIMENSION_SID: u16 = 0x0200;
/// NUMBER 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const NUMBER_SID: u16 = 0x0203;
/// LABEL 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const LABEL_SID: u16 = 0x0204;
/// BOOLERR 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const BOOL_ERR_SID: u16 = 0x0205;
/// STRING 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const STRING_SID: u16 = 0x0207;
/// ROW 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const ROW_SID: u16 = 0x0208;
/// INDEX 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const INDEX_SID: u16 = 0x020B;
/// WINDOW2 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const WINDOW2_SID: u16 = 0x023E;
/// RK 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const RK_SID: u16 = 0x027E;
/// STYLE 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const STYLE_SID: u16 = 0x0293;
/// Chart AI（数据引用公式）记录 SID。
/// 对应 Java：POI `LinkedDataRecord`。
pub const CHART_AI_SID: u16 = 0x1051;
/// FORMAT 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const FORMAT_SID: u16 = 0x041E;
/// BOF 记录 SID。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const BOF_SID: u16 = 0x0809;

/// 判断记录是否属于可在未选中工作表中直接跳过的事件记录。
///
/// 这些记录的主体只会被单元格、公式、批注、合并区域或共享字符串事件
/// 消费；当上层读取器已经判定当前工作表不需要读取时，无需再进入对应
/// handler。`BOF`、`EOF` 与 `CONTINUE` 仍必须由状态机处理，因此不在此列。
#[must_use]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const fn is_skippable_event_record(record_sid: u16) -> bool {
    matches!(
        record_sid,
        BLANK_SID
            | BOOL_ERR_SID
            | BOUND_SHEET_SID
            | FORMULA_SID
            | HYPERLINK_SID
            | INDEX_SID
            | LABEL_SID
            | LABEL_SST_SID
            | MERGE_CELLS_SID
            | NOTE_SID
            | NUMBER_SID
            | OBJ_SID
            | RK_SID
            | SST_SID
            | STRING_SID
            | TEXT_OBJECT_SID
    )
}
