# easyexcel-cache

[English](README.md)

面向流式电子表格读取器的可复用共享字符串缓存。

> 版本线：0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## 职责

- 提供内存、文件落盘和 Moka 对象缓存实现。
- 通过统一策略与句柄 API 选择缓存实现。

## 架构

```text
shared strings -> cache policy -> memory / file / Moka cache -> reader
```

主要公共 API：`SharedStringCache, SharedStringCachePolicy, ReadCacheMode, create_cache`。

## 安装与使用

```toml
[dependencies]
easyexcel-cache = "0.1.1"
```

```rust
use easyexcel_cache::{ReadCacheMode, SharedStringCachePolicy, create_cache};
```

## 兼容性与边界

本 crate 缓存共享字符串，不是工作簿缓存，也不承载依赖淘汰时序的业务语义。

权威能力边界维护在[工作区兼容性矩阵](../../docs/compatibility.md)中。未支持行为必须返回明确错误或警告，禁止静默降级。

## 项目链接

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API 文档](https://docs.rs/easyexcel-cache)
- [变更日志](../../CHANGELOG.md)
- [英文 README](README.md)
