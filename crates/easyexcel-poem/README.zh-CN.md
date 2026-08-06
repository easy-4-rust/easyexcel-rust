# easyexcel-poem

[English](README.md)

共享 EasyExcel Web 运行时的 Poem 原生集成。

> 版本线：0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## 职责

- 以 `ExcelRequest<T>` 与 `ExcelResponse<T>` 提供 Poem 原生extractor 与 responder。
- 把共享策略与问题详情映射到 Poem 传输原语。

## 架构

```text
Poem request -> easyexcel-poem -> easyexcel-web -> EasyExcel engines -> Poem response
```

主要公共 API：`ExcelRequest, ExcelResponse, ExcelPoemError, ExcelWebPolicy, ExcelWebRuntime`。

## 安装与使用

```toml
[dependencies]
easyexcel-poem = "0.1.1"
```

```rust
use easyexcel_poem::{ExcelRequest, ExcelResponse, ExcelWebPolicy};
```

## 兼容性与边界

业务规则、解析与资源限制位于 `easyexcel-web`；本 crate 只负责 Poem 传输适配。可运行示例位于仓库的 `examples/poem`。

权威能力边界维护在[工作区兼容性矩阵](../../docs/compatibility.md)中。未支持行为必须返回明确错误或警告，禁止静默降级。

## 项目链接

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API 文档](https://docs.rs/easyexcel-poem)
- [变更日志](../../CHANGELOG.md)
- [英文 README](README.md)
