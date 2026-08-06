# easyexcel-io

[English](README.md)

共享的电子表格 I/O 契约、格式识别、流式行协议、资源限制与类型化错误。

> 版本线：0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## 职责

- 定义 `RowSource`/`RowSink`、流元数据和读写模式。
- 集中管理格式识别、工作表选择与资源限制契约。

## 架构

```text
bytes / paths -> easyexcel-io contracts -> format engines
```

主要公共 API：`Format, ResourceLimits, RowSource, RowSink, StreamCell, ReadMode, WriteMode`。

## 安装与使用

```toml
[dependencies]
easyexcel-io = "0.1.1"
```

```rust
use easyexcel_io::{Format, ResourceLimits, RowSink, RowSource};
```

## 兼容性与边界

具体 XLS、XLSX、CSV 编解码器位于各格式 crate；业务代码优先使用 `easyexcel::io`。

权威能力边界维护在[工作区兼容性矩阵](../../docs/compatibility.md)中。未支持行为必须返回明确错误或警告，禁止静默降级。

## 项目链接

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API 文档](https://docs.rs/easyexcel-io)
- [变更日志](../../CHANGELOG.md)
- [英文 README](README.md)
