//! BIFF8 记录类型编号。
//!
//! SID 是 BIFF 二进制协议的一部分；Java EasyExcel handler 可以保留同名
//! 常量入口，但具体编号只在 XLS 引擎中定义。

/// FORMULA 记录 SID。
pub const FORMULA_SID: u16 = 0x0006;
/// EOF 记录 SID。
pub const EOF_SID: u16 = 0x000A;
/// NOTE 记录 SID。
pub const NOTE_SID: u16 = 0x001C;
/// CONTINUE 记录 SID。
pub const CONTINUE_SID: u16 = 0x003C;
/// OBJ 记录 SID。
pub const OBJ_SID: u16 = 0x005D;
/// BOUNDSHEET 记录 SID。
pub const BOUND_SHEET_SID: u16 = 0x0085;
/// MERGECELLS 记录 SID。
pub const MERGE_CELLS_SID: u16 = 0x00E5;
/// SST 记录 SID。
pub const SST_SID: u16 = 0x00FC;
/// LABELSST 记录 SID。
pub const LABEL_SST_SID: u16 = 0x00FD;
/// TxO 记录 SID。
pub const TEXT_OBJECT_SID: u16 = 0x01B6;
/// HYPERLINK 记录 SID。
pub const HYPERLINK_SID: u16 = 0x01B8;
/// BLANK 记录 SID。
pub const BLANK_SID: u16 = 0x0201;
/// NUMBER 记录 SID。
pub const NUMBER_SID: u16 = 0x0203;
/// LABEL 记录 SID。
pub const LABEL_SID: u16 = 0x0204;
/// BOOLERR 记录 SID。
pub const BOOL_ERR_SID: u16 = 0x0205;
/// STRING 记录 SID。
pub const STRING_SID: u16 = 0x0207;
/// INDEX 记录 SID。
pub const INDEX_SID: u16 = 0x020B;
/// RK 记录 SID。
pub const RK_SID: u16 = 0x027E;
/// BOF 记录 SID。
pub const BOF_SID: u16 = 0x0809;
