# easyexcel-utils

[English](README.md)

可复用的 Java 兼容字符串、集合、坐标与校验算法。

> 版本线：0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## 职责

- 提供各引擎共享的小型确定性工具。
- 避免把可复用算法放入 EasyExcel 门面编排层。

## 架构

```text
engine input -> easyexcel-utils helpers -> normalized values
```

主要公共 API：`string_utils, coordinate_utils, list_utils, map_utils, validation`。

## 安装与使用

```toml
[dependencies]
easyexcel-utils = "0.1.1"
```

```rust
use easyexcel_utils::{coordinate_utils, string_utils, validation};
```

## 兼容性与边界

这是内部引擎 crate，不是 EasyExcel 门面的替代品，也不是通用工具框架。

权威能力边界维护在[工作区兼容性矩阵](../../docs/compatibility.md)中。未支持行为必须返回明确错误或警告，禁止静默降级。

## 项目链接

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API 文档](https://docs.rs/easyexcel-utils)
- [变更日志](../../CHANGELOG.md)
- [英文 README](README.md)
