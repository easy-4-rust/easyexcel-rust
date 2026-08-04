# Changelog

本文件记录 easyexcel-rust 各版本变更。格式参照 [Keep a Changelog](https://keepachangelog.com/)。

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
