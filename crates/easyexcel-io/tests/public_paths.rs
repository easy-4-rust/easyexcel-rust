//! `easyexcel-io` 公共路径兼容性测试。

use easyexcel_io::io::{Format as NestedFormat, ResourceLimits as NestedLimits};
use easyexcel_io::{Format, ReadMode, ResourceLimits, WriteMode};

#[test]
fn root_and_io_paths_resolve_to_the_same_types() {
    let root_format: Format = NestedFormat::Xlsx;
    assert_eq!(root_format, Format::Xlsx);

    let root_limits: ResourceLimits = NestedLimits::default();
    assert_eq!(root_limits, ResourceLimits::default());

    let _ = ReadMode::Event;
    let _ = WriteMode::Generate;
}
