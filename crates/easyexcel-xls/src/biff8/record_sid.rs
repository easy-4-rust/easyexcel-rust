//! BIFF8 记录类型编号。
//!
//! SID 是 BIFF 二进制协议的一部分；Java EasyExcel handler 可以保留同名
//! 常量入口，但具体编号只在 XLS 引擎中定义。

/// FORMULA 记录 SID。
pub const FORMULA_SID: u16 = 0x0006;
/// EOF 记录 SID。
pub const EOF_SID: u16 = 0x000A;
/// CALCMODE 记录 SID。
pub const CALC_MODE_SID: u16 = 0x000D;
/// NOTE 记录 SID。
pub const NOTE_SID: u16 = 0x001C;
/// DATEMODE 记录 SID。
pub const DATE_MODE_SID: u16 = 0x0022;
/// FILEPASS 记录 SID。
pub const FILE_PASS_SID: u16 = 0x002F;
/// FONT 记录 SID。
pub const FONT_SID: u16 = 0x0031;
/// CONTINUE 记录 SID。
pub const CONTINUE_SID: u16 = 0x003C;
/// PANE 记录 SID。
pub const PANE_SID: u16 = 0x0041;
/// CODEPAGE 记录 SID。
pub const CODE_PAGE_SID: u16 = 0x0042;
/// CODENAME 记录 SID。
pub const CODE_NAME_SID: u16 = 0x004A;
/// WRITEACCESS 记录 SID。
pub const WRITE_ACCESS_SID: u16 = 0x005C;
/// OBJ 记录 SID。
pub const OBJ_SID: u16 = 0x005D;
/// COLINFO 记录 SID。
pub const COLUMN_INFO_SID: u16 = 0x007D;
/// BOUNDSHEET 记录 SID。
pub const BOUND_SHEET_SID: u16 = 0x0085;
/// PALETTE 记录 SID。
pub const PALETTE_SID: u16 = 0x0092;
/// MULRK 记录 SID。
pub const MUL_RK_SID: u16 = 0x00BD;
/// MULBLANK 记录 SID。
pub const MUL_BLANK_SID: u16 = 0x00BE;
/// MMS 记录 SID。
pub const MMS_SID: u16 = 0x00C1;
/// XF 记录 SID。
pub const XF_SID: u16 = 0x00E0;
/// INTERFACEHDR 记录 SID。
pub const INTERFACE_HEADER_SID: u16 = 0x00E1;
/// INTERFACEEND 记录 SID。
pub const INTERFACE_END_SID: u16 = 0x00E2;
/// MERGECELLS 记录 SID。
pub const MERGE_CELLS_SID: u16 = 0x00E5;
/// MSODRAWING 记录 SID。
pub const MSO_DRAWING_SID: u16 = 0x00EC;
/// SST 记录 SID。
pub const SST_SID: u16 = 0x00FC;
/// LABELSST 记录 SID。
pub const LABEL_SST_SID: u16 = 0x00FD;
/// EXTSST 记录 SID。
pub const EXT_SST_SID: u16 = 0x00FF;
/// TxO 记录 SID。
pub const TEXT_OBJECT_SID: u16 = 0x01B6;
/// HYPERLINK 记录 SID。
pub const HYPERLINK_SID: u16 = 0x01B8;
/// BLANK 记录 SID。
pub const BLANK_SID: u16 = 0x0201;
/// DIMENSION 记录 SID。
pub const DIMENSION_SID: u16 = 0x0200;
/// NUMBER 记录 SID。
pub const NUMBER_SID: u16 = 0x0203;
/// LABEL 记录 SID。
pub const LABEL_SID: u16 = 0x0204;
/// BOOLERR 记录 SID。
pub const BOOL_ERR_SID: u16 = 0x0205;
/// STRING 记录 SID。
pub const STRING_SID: u16 = 0x0207;
/// ROW 记录 SID。
pub const ROW_SID: u16 = 0x0208;
/// INDEX 记录 SID。
pub const INDEX_SID: u16 = 0x020B;
/// WINDOW2 记录 SID。
pub const WINDOW2_SID: u16 = 0x023E;
/// RK 记录 SID。
pub const RK_SID: u16 = 0x027E;
/// STYLE 记录 SID。
pub const STYLE_SID: u16 = 0x0293;
/// FORMAT 记录 SID。
pub const FORMAT_SID: u16 = 0x041E;
/// BOF 记录 SID。
pub const BOF_SID: u16 = 0x0809;

/// 判断记录是否属于可在未选中工作表中直接跳过的事件记录。
///
/// 这些记录的主体只会被单元格、公式、批注、合并区域或共享字符串事件
/// 消费；当上层读取器已经判定当前工作表不需要读取时，无需再进入对应
/// handler。`BOF`、`EOF` 与 `CONTINUE` 仍必须由状态机处理，因此不在此列。
#[must_use]
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
