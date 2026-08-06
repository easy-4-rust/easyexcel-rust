# easyexcel-web

[English](README.md)

EasyExcel-Rust 的框架中立 Web 导入导出运行时。

> 版本线：0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## 职责

- 提供有界上传、临时文件生命周期、类型化行流与流式下载。
- 统一资源限制、背压、取消、超时、tracing 与稳定问题详情。

## 架构

```text
HTTP body -> ExcelImport -> bounded rows -> application -> ExcelExport -> HTTP body
```

主要公共 API：`ExcelImport, ExcelRows, ExcelExport, ExcelWebPolicy, ExcelWebRuntime, ExcelProblemDetails`。

## 安装与使用

```toml
[dependencies]
easyexcel-web = "0.1.1"
```

```rust
use easyexcel_web::{ExcelExport, ExcelImport, ExcelWebPolicy, ExcelWebRuntime};
```

## 兼容性与边界

本 crate 不提供具体框架 extractor 或 responder，请选择对应 Web 框架适配器。

权威能力边界维护在[工作区兼容性矩阵](../../docs/compatibility.md)中。未支持行为必须返回明确错误或警告，禁止静默降级。

## 流式与框架契约

七个适配器统一公开 `ExcelRequest<T>` 与 `ExcelResponse<T>`，同时保留各框架原生提取和响应机制。上传元数据从 `x-excel-file-name`、`Content-Disposition` 或 `Content-Type` 解析；`x-request-id` 会传入 tracing 与稳定错误响应。

XLSX 和旧 XLS 解析器需要随机访问完整容器，因此流式上传是把请求体分块落入自动清理的临时文件，再以有界行流交付业务代码；它不代表把整个文件缓存在 `Vec<u8>` 中。下载会在响应流发送前完成生成，避免失败时输出部分有效的电子表格。

V1 已执行文件字节数和总行数限制。工作表数量与公式单元格数量仍依赖格式引擎统一计数钩子，在钩子接通前不宣称已强制执行。可运行适配器位于 `examples/{axum,actix,hyper,poem,rocket,salvo,warp}`，并共享 `tests/easyexcel-web-conformance`。

## 项目链接

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API 文档](https://docs.rs/easyexcel-web)
- [变更日志](../../CHANGELOG.md)
- [英文 README](README.md)
