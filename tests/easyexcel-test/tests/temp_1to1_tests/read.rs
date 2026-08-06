//! 1:1 method matrix for Java `com.alibaba.easyexcel.test.temp.read.*`

use super::helpers;

/// Java `com.alibaba.easyexcel.test.temp.read.CommentTest#comment`
#[test]
fn read_comment_test_comment() {
    helpers::assert_head_read();
}

/// Java `com.alibaba.easyexcel.test.temp.read.HeadReadTest#test`
#[test]
fn read_head_read_test_test() {
    helpers::assert_head_read();
}

/// Java `com.alibaba.easyexcel.test.temp.read.HeadReadTest#testCache`
///
/// The Java file-cache case maps to Rust `ReadCacheMode::File`; three stable
/// reads on an XLS fixture (see `helpers::assert_head_read_with_disk_cache`).
#[test]
fn read_head_read_test_test_cache() {
    helpers::assert_head_read_with_disk_cache();
}
