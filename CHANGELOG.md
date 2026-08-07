# Changelog

本文件记录 easyexcel-rust 各版本变更。格式参照 [Keep a Changelog](https://keepachangelog.com/)。

## [0.1.2] - 2026-08-07

模块文档增强版本。

### 文档

- 将 `crates/` 下全部 21 个正式 crate 的英文与中文 README 扩展为可独立使用的
  模块手册，不再停留在简短定位说明。
- 每个模块增加 Mermaid 架构图、能力矩阵、公共 API 表、安装说明、基础与进阶
  Rust 示例、错误边界、依赖关系图和可导航链接。
- Web 适配器分别补充各框架原生 extractor/responder、上传背压、流式下载、
  runtime 接线、响应头和稳定错误映射示例。
- 保留并扩展 derive 的 Java 注解语义映射，以及 `easyexcel-web` 的真实流式语义。

### 兼容性

- 本版本不修改公开 Rust API 或文件格式行为。
- 全部工作区 crate 与内部路径依赖统一升级到 `0.1.2`。

## [0.1.1] - 2026-08-07

文档与发布元数据修订版。

### 文档

- 为 `crates/` 下全部 21 个正式发布 crate 增加结构对等的英文 `README.md`
  与中文 `README.zh-CN.md`。
- 每份 README 明确模块定位、职责边界、数据流、主要公共 API、安装方式、
  MSRV 和已知能力边界。
- 在每个 crate 的 Cargo 发布元数据中显式声明 `readme = "README.md"`，确保
  crates.io 包页面展示对应模块文档；中文 README 同步包含在发布包中。

### 兼容性

- 本版本不修改公开 Rust API 或文件格式行为。
- 全部工作区 crate 与内部路径依赖统一升级到 `0.1.1`。

## [0.1.0] - 2026-08-07

首个生产就绪正式版本。该版本把 Java EasyExcel 风格门面与可复用的 XLS、
XLSX、CSV、公式、Markdown、缓存和 Web 流式引擎收敛为同一版本线。

### 正式版能力

- Rust 用户统一通过 `easyexcel` 门面及 `easyexcel::{model, io, csv, xls,
  xlsx, formula, markdown, tabular}` 模块使用能力。
- 支持 XLSX、XLS、CSV 的读取与写入，XLSX/CSV 事件流读取，以及带明确损失
  报告的 XLS/XLSX/CSV 与 Markdown 双向转换。
- 提供 Java EasyExcel 风格 builder、listener、converter、handler、annotation
  derive、模板填充和加密 XLSX 读写。
- 提供框架中立 `easyexcel-web` 内核，以及 Axum、Actix Web、Hyper、Poem、
  Rocket、Salvo、Warp 七个适配器和共享 conformance suite。
- 在发布候选提交上通过全 workspace 测试、Java parity/golden、大文件、文档、
  Clippy、RustSec、cargo-deny、MSRV 和 facade boundary 门禁。

### 明确边界

- XLS Event Mode、旧 XLS 密码保护与 XLS 占位符填充尚不支持，并返回类型化
  `Unsupported` 错误。
- 公式引擎不承诺覆盖 Excel 的全部函数；Cube/Web/RTD 等外部数据函数和少量
  复杂工程、金融函数保持明确的未支持状态。
- XLSX round-trip 会尽可能保留未知 OOXML 部件，但不承诺宏、图表等所有高级
  对象的无损编辑；具体能力以 `docs/compatibility.md` 为准。

## [0.1.0-alpha.1] - 2026-08-03

首个公开预发布版本：Alibaba EasyExcel 4.0.3 的 Rust 高保真迁移。

### 核心特性

- **完整语义对齐**：Java easyexcel-core 323 类 100% 文件级映射；14 个注解
  字段级覆盖；facade 方法一一对应（`EasyExcel::write` / `read` / `fill`）
- **三格式读写**：XLSX（OOXML 流式 SAX 解析 + `rust_xlsxwriter` 写入）、
  XLS（BIFF8）、CSV（多 charset）
- **恒定内存流式**：百万行写峰值 RSS 10.8 MiB（Java stream 476 MiB 的
  1/44），写 4.0s / 读 2.5s（Apple M4 Pro，rustc 1.97.1 release）
- **OOXML 模板填充**：`{placeholder}` 占位符展开、列表/集合/标量填充、
  重复区域游标对齐 Java `FillConfig`
- **8 个 Web 框架适配器**：axum / actix-web / rocket / warp / salvo /
  poem / tide / hyper，统一 `excel_download_response` / `read_upload_*` API
- **加密读写**：ECMA-376 Agile 密码保护 XLSX（crates.io 官方 office-crypto）

### 质量保证（对照 docs/compatibility.md 1.0 门禁 7 条证据）

- **2771 测试全绿**（含 88 个 Java golden 语义对拍测试）
- **CI 全绿**：fmt / clippy `-D warnings` / cargo audit（0 vulnerabilities）/
  cargo doc / MSRV job（rustc 1.94）/ coverage（96.38% lines，可达代码 100%，
  残差 195 行经 8 个 agent 逐行验证数学不可达）
- **LibreOffice 实证**：7/7 fixture 无修复警告打开（证据 4 PASS）
- **Java 同机对拍**：写比 Java stream 快约 6%、读持平、内存低 44×

### 已知限制

- `WriteHolderScope` / handler 回调链的 Java 语义完整对齐带来写路径开销，
  相对功能未对齐的早期基线（2.93s）仍慢约 1s；剩余差距在 zip deflate 与
  rust_xlsxwriter 内部序列化层
- 加密 XLSX 的 LibreOffice headless 打开需交互输入密码，由 round-trip
  测试覆盖而非 open-check 脚本
- 仅 XLSX/XLS/CSV；不支持 .xlsm（宏）与 .xlsb（二进制 XLSX）

### 安全

- 0 vulnerabilities（`cargo audit`）；`quick-xml 0.38.4` 高危传递依赖
  经 vendor `office-crypto` + `[patch.crates-io]` 升级到 0.41 修复（2026-08-05
  回退官方 crates.io 版本，0.38.4 两条 high 转已记录豁免，`cargo audit` 仍 exit 0）
