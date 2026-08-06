# easyexcel-xls

[English](README.md)

BIFF8 `.xls` 工作簿读取与写入引擎。

> 版本线：0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## 职责

- 识别复合文档容器，并把 BIFF8 记录映射到共享模型。
- 通过路径和流 API 读取、写入 XLS 工作簿。

## 架构

```text
CFB / BIFF8 bytes <-> easyexcel-xls <-> Workbook
```

主要公共 API：`read, read_path, write, write_path, looks_like_cfb`。

## 安装与使用

```toml
[dependencies]
easyexcel-xls = "0.1.1"
```

```rust
use easyexcel_xls::{read_path, write_path};
```

## 兼容性与边界

不宣称支持 XLS Event Mode、旧 XLS 密码保护和占位符填充；业务代码优先使用 `easyexcel::xls`。

权威能力边界维护在[工作区兼容性矩阵](../../docs/compatibility.md)中。未支持行为必须返回明确错误或警告，禁止静默降级。

## 项目链接

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API 文档](https://docs.rs/easyexcel-xls)
- [变更日志](../../CHANGELOG.md)
- [英文 README](README.md)
