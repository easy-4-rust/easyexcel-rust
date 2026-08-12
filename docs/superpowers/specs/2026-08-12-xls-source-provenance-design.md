# xls fork 迁入来源记录

## 基线

- 来源仓库：同工作区 `easy-4-rust/xls` fork。
- 来源 commit：`4c13da74a87e8c6cc83bbb01c7419b1729684a24`。
- 来源许可证：MIT OR Apache-2.0。
- 目标仓库许可证：Apache-2.0；独立 `xls-cli` 产品声明 MIT OR Apache-2.0，并携带两份许可证和 NOTICE。

## 文件映射

| 来源 | 目标 | 主要改动 |
| --- | --- | --- |
| `xls/src/core/{addr,dates,error,model,numfmt,styles,value}.rs` | `crates/easyexcel-model/src/` | 移除公式反向依赖，数字文本转换下沉 model |
| `xls/src/core/formula/**` | `crates/easyexcel-formula/src/formula/**` | 根路径改为 `easyexcel_model`，保持公式注册/依赖图行为 |
| `xls/src/core/xls/**` | `crates/easyexcel-xls/src/xls/**` | 统一使用 `easyexcel_io::Error` 与 `easyexcel_model` |
| `xls/src/core/xlsx/**` | `crates/easyexcel-xlsx/src/xlsx/**` | 统一模型/I/O；保留 stream、crypto、opaque/table roundtrip |
| `xls/src/core/csv.rs` | `crates/easyexcel-csv/src/csv/codec.rs` | 统一模型/I/O |
| `xls/src/core/query.rs` | `xls-cli/src/application/query.rs` | 作为产品仓库内的 library-only 查询用例，不包含 stdout/终端逻辑 |

## 维护规则

1. fork 中的新能力先补特征测试，再按职责迁入目标 crate。
2. 迁入后生产修复以 `easyexcel-rust` 为事实来源；不得让基础 crate 重新依赖 fork。
3. 同步时记录来源 commit、源文件、行为差异和新增测试。
4. README/capabilities 只能声明测试支持或明确 partial/unsupported 的能力。
