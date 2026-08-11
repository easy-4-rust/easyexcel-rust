//! BIFF8 XLS 工作簿读取、写入和格式识别。

// BIFF8 的记录长度、行列坐标、RK 数字和字符串长度由格式规范固定为
// u8/u16/u32 位域；编码前的工作簿上限与记录分帧保证这些窄化转换可表示。
// 解码侧的有符号重解释则是 RK 二进制布局本身，豁免仅限 XLS 格式 crate。
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
// 测试中的 RK/IEEE-754 样本是规范定义的精确位模式。
#![allow(clippy::float_cmp)]
// 分支和位掩码保持 BIFF 记录规范的原始结构，便于与记录字段逐项核对。
#![allow(
    clippy::manual_let_else,
    clippy::match_same_arms,
    clippy::similar_names,
    clippy::too_many_lines,
    clippy::verbose_bit_mask
)]
// 编码辅助函数使用统一的记录构建签名，轻量参数传递不改变所有权语义。
#![allow(clippy::needless_pass_by_value, clippy::trivially_copy_pass_by_ref)]

pub mod biff8;
pub mod xls;

pub use xls::{
    Biff8SstString, CFB_MAGIC, looks_like_cfb, parse_sst_rich, read,
    read_decrypted_workbook_stream, read_path, read_path_with_password, read_with_limits,
    read_with_password, read_with_password_and_limits,
    to_biff8_book, write, write_path, write_path_with_password, write_with_password,
};
