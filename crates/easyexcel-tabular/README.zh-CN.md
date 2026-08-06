# easyexcel-tabular

[English](README.md)

安全的 HTML、JSON 表格转换与通用表格格式分派。

> 版本线：0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## 职责

- 解析、渲染静态 HTML 表格与 JSON 表格文档。
- 将 Markdown 处理委托给 `easyexcel-markdown`，不重复实现编解码器。

## 架构

```text
HTML / JSON / Markdown -> dispatcher -> TabularDocument
```

主要公共 API：`TabularFormat, TabularDocument, parse_document, render_document, parse_html, parse_json`。

## 安装与使用

```toml
[dependencies]
easyexcel-tabular = "0.1.1"
```

```rust
use easyexcel_tabular::{TabularDocument, TabularFormat, parse_document};
```

## 兼容性与边界

HTML 输入仅作为静态表格标记处理；脚本、网络加载和不受控 CSS 不在范围内。业务代码优先使用 `easyexcel::tabular`。

权威能力边界维护在[工作区兼容性矩阵](../../docs/compatibility.md)中。未支持行为必须返回明确错误或警告，禁止静默降级。

## 项目链接

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API 文档](https://docs.rs/easyexcel-tabular)
- [变更日志](../../CHANGELOG.md)
- [英文 README](README.md)
