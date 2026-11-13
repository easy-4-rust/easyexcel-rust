# EasyExcel-Rust 依赖安全审计报告

> 审计类型：PR #5 依赖升级后的安全审计
> 审计时间：2026-11-08 04:39:04 UTC
> 仓库：easy-4-rust/easyexcel-rust
> 审计分支：fix/compatible-security-deps-20261031
> 固定提交：dd673ea6a15909cc137d87eba26b7bedd43d323a
> Cargo.lock：该提交中的 Cargo.lock
> GitHub Actions：Security Audit Run 42447661141，Job 128629914044
> 结果：9 vulnerabilities found；11 allowed warnings found；Job failed
> PR：https://github.com/easy-4-rust/easyexcel-rust/pull/5

## 1. 执行摘要

本次审计基线不是 main 的旧依赖状态，而是 PR #5 已经包含 ZIP 9.1.3、Actix 4.14.0、Calamine 0.37.0、Tokio 1.54.1、rustls 0.23.46 等升级后的实际锁文件。

审计发现：

- 9 项 RustSec 漏洞告警，cargo audit 以失败退出。
- 11 项 unsoundness 警告：10 项来自 zstd-safe 7.2.4，1 项来自 lru 0.16.4。
- 关键依赖链涉及 Actix HTTP、Actix Web、Calamine、h2、Salvo 的 test feature、zstd/zstd-safe，以及 ssfmt。
- PR #5 当前不能合并。不得以关闭审计、添加 ignore、移除 HTTP/2/压缩功能或降低审计范围来取得绿色状态。

该报告固定的是 2026-11-08 04:39 UTC 的审计结果。RustSec 数据库会变化；每次修复后的验收必须重新运行审计，并记录新的数据库版本/提交和 Git SHA。

## 2. 证据与审计范围

### 2.1 已确认的 CI 证据

- Security Audit Run：42447661141
- Job：128629914044（cargo audit + deny）
- Job 日志报告：9 vulnerabilities found、11 allowed warnings found，退出码 1。
- 同一提交的 CI Run：42447661014（run #262）。
- 在审计记录时，CI 中 Clippy、rustfmt、Doc build、MSRV check、Bench build check 已通过；Ubuntu/macOS 测试与 coverage-pr 仍在运行。不得把尚未完成的检查记为通过。

### 2.2 锁文件中的受影响版本

| Crate | 锁定版本 | 依赖来源/父节点 |
|---|---:|---|
| actix-web | 4.14.0 | easyexcel-actix、easyexcel-demo-actix、easyexcel-web-conformance |
| actix-http | 3.19.0 | actix-web 4.14.0 |
| calamine | 0.37.0 | easyexcel、easyexcel-test |
| h2 | 0.3.27 | actix-http 3.19.0；hyper 0.14.32 |
| hyper | 0.14.32 | rocket_http 0.5.1 |
| zstd | 0.13.3 | salvo_core 0.89.3 |
| zstd-safe | 7.2.4 | zstd 0.13.3 |
| lru | 0.16.4 | ssfmt 0.1.2 |
| zip | 9.1.3 | PR #5 已迁移；不是本次审计失败项 |

Cargo.lock 还同时存在 hyper 1.11.0 / h2 0.4.20、zstd 0.14.3 / zstd-safe 8.2.0 的其他依赖路径。不能因为存在已修复的新版本，就假设旧版本依赖链已被消除。

### 2.3 代码与清单位置

以下路径是已确认的依赖声明或框架接入点。对于尚未定位到 Rust 调用点的第三方解析器，明确标记为待定位，不推测调用路径。

| 领域 | 仓库位置 | 已确认内容 |
|---|---|---|
| Actix 版本 | Cargo.toml；crates/easyexcel-actix/Cargo.toml | workspace 声明 actix-web；适配器依赖 actix-web |
| Actix 请求入口 | crates/easyexcel-actix/src/excel_request.rs | FromRequest / Payload 请求提取路径 |
| Salvo 版本与 features | Cargo.toml；crates/easyexcel-salvo/Cargo.toml | salvo 0.89.3；显式启用 http1、server、test |
| Salvo 请求入口 | crates/easyexcel-salvo/src/excel_request.rs | Salvo Request / Extractible 适配入口 |
| Rocket 版本 | Cargo.toml；crates/easyexcel-rocket/Cargo.toml | Rocket adapter 引入 rocket_http -> hyper 0.14.32 |
| Calamine 声明 | Cargo.toml；crates/easyexcel/Cargo.toml | easyexcel 直接依赖 calamine |
| Calamine 调用点 | 待定位 | 本轮锁文件已确认 easyexcel 与 easyexcel-test 依赖 calamine；具体调用文件/行号必须通过源码搜索补齐后才可修改 |
| zstd 依赖开关 | Cargo.toml 的 salvo workspace dependency | Salvo test feature 会引入 zstd 0.13 系列；需要确认其是否只用于测试工具 |
| ssfmt / lru | Cargo.toml；crates/easyexcel-format/Cargo.toml | easyexcel-format 使用 ssfmt.workspace；ssfmt 0.1.2 依赖 lru ^0.16 |

## 3. 漏洞清单（9 项）

### ACTIX-001：Actix HTTP/1 请求走私

- Advisory：RUSTSEC-2026-0446
- Crate/version：actix-http 3.19.0
- 风险：HTTP/1 chunked decoder 对最终 chunk 后的裸 LF 处理不严格；在代理连接复用、请求流水线等条件下，后续请求头可能被误读为 trailer，导致代理/WAF 与应用层请求边界不一致。
- 优先级：P0
- 源码/配置位置：Cargo.toml；crates/easyexcel-actix/Cargo.toml；crates/easyexcel-actix/src/excel_request.rs；Cargo.lock 中 actix-web 4.14.0 -> actix-http 3.19.0。
- 上游修复版本：actix-http >= 3.20.2。
- 验证：对 Actix HTTP/1 入口发送畸形 chunked trailer 原始报文；验证拒绝请求且不会将后续请求解析为同一请求的 trailer；同时做正常 chunked、keep-alive、pipelining 回归。
- 验收：锁文件不再含受影响 actix-http；回归测试证明请求边界正确；HTTP/1 仍可用。

### ACTIX-002：Actix HTTP 解压资源耗尽

- Advisory：RUSTSEC-2026-0448
- Crate/version：actix-http 3.19.0
- 风险：解压过程可能在 extractor 限制生效前消耗不受限的内存/CPU。
- 优先级：P0
- 源码/配置位置：Cargo.toml；crates/easyexcel-actix/Cargo.toml；crates/easyexcel-actix/src/excel_request.rs；Cargo.lock actix-web -> actix-http。
- 上游修复版本：actix-http >= 3.20.1；本修复组还必须满足 ACTIX-003 的更高版本门槛。
- 验证：使用小体积高压缩比请求体，验证解压前后限制、拒绝行为和资源上限；检查 extractor/body limit 不会在解压后才首次生效。
- 验收：恶意压缩请求不能造成无界分配；正常 gzip/deflate/zstd/Brotli 行为按现有功能契约保持。

### ACTIX-003：Actix Brotli 大窗口绕过解压内存限制

- Advisory：RUSTSEC-2026-0455
- Crate/version：actix-http 3.19.0
- 风险：Brotli 大窗口请求体可绕过解压内存限制，造成内存/CPU 资源耗尽。
- 优先级：P0
- 源码/配置位置：Cargo.toml；crates/easyexcel-actix/Cargo.toml；crates/easyexcel-actix/src/excel_request.rs；Cargo.lock actix-web -> actix-http。
- 上游修复版本：actix-http >= 3.20.3。
- 验证：加入大窗口 Brotli 恶意/边界样本；验证内存上限、错误响应和连接清理；正常 Brotli 流量回归。
- 验收：actix-http 至少 3.20.3；不允许通过关闭压缩 feature 规避。

### ACTIX-004：Actix Web 压缩请求体在 extractor 限制前分配

- Advisory：RUSTSEC-2026-0447
- Crate/version：actix-web 4.14.0
- 风险：小型压缩请求可能在 extractor 拒绝之前扩展为大体积解压数据，导致远程 DoS。
- 优先级：P0
- 源码/配置位置：Cargo.toml；crates/easyexcel-actix/Cargo.toml；crates/easyexcel-actix/src/excel_request.rs。
- 上游修复版本：actix-web >= 4.18.2，且 actix-http >= 3.20.1；由于 ACTIX-003，实际锁定目标至少 actix-http 3.20.3。
- MSRV 影响：actix-web 4.18.2 官方文档标注 Rust 1.89+。根 workspace 当前声明 Rust 1.88；不得无评估地整体提高 MSRV。应评估将 easyexcel-actix（以及实际依赖该版本的示例/集成包）单独声明 Rust 1.89，同时保持核心 crate 的 1.88 边界。
- 验证：压缩体超限、解压体超限、正常请求、流式读取、错误释放和资源限制集成测试；Rust 1.89 adapter 构建；Rust 1.88 核心构建。
- 验收：actix-web >= 4.18.2；Actix adapter MSRV 如实声明；核心 1.88 验证不回退。

### CALAMINE-001：恶意 CFB/OLE2 输入导致 panic / OOM

- Advisory：RUSTSEC-2026-0436
- Crate/version：calamine 0.37.0
- 风险：解析 .xls/.xlsb 及部分 VBA 项目时，畸形目录流可触发越界 panic；恶意 DIFAT 链可导致无界分配/OOM。用户上传文件属于不可信输入面。
- 优先级：P0
- 源码/配置位置：Cargo.toml；crates/easyexcel/Cargo.toml；Cargo.lock easyexcel/easyexcel-test -> calamine。具体 calamine API 调用文件尚未完成定位，必须在实施前用源码搜索确认。
- 上游修复版本：calamine >= 0.38.0。
- 验证：为短目录分块、64-byte mini-sector、循环/自引用 DIFAT 构造最小恶意 fixture；在受控子进程/资源限制下执行，避免 OOM 测试拖垮主测试进程；正常 xls/xlsb/VBA 文件兼容回归。
- 验收：锁文件不再含受影响版本；畸形输入返回受控错误而不是 panic/OOM；格式兼容测试通过。

### H2-001：HTTP/2 空 DATA frame 无界排队

- Advisory：RUSTSEC-2026-0258
- Crate/version：h2 0.3.27
- 风险：不被主动 drain 的 stream 可接收/排队无限空 DATA frame，导致内存增长或长度溢出 panic。RustSec 修复版本标记为 h2 >= 0.4.16。
- 优先级：P1（可远程触发的可用性风险；仍是审计阻断项）
- 依赖链：
  - actix-http 3.19.0 -> h2 0.3.27；
  - rocket_http 0.5.1 -> hyper 0.14.32 -> h2 0.3.27。
- 源码/配置位置：Cargo.toml；crates/easyexcel-actix/Cargo.toml；crates/easyexcel-rocket/Cargo.toml；对应适配器源码；Cargo.lock 中 actix-http、rocket_http、hyper、h2。
- 修复约束：必须同时覆盖 Actix 与 Rocket 两条依赖链；不得只升级其中一个父 crate后就宣布清零。若上游 0.3 系列没有可用修复版本，评估对 h2 0.3 的最小上游补丁/受控 fork，并提供源码审查和回归证据；不允许直接移除 HTTP/2。
- 验证：HTTP/2 stream 在不 drain 场景下接收大量空 DATA frame；测量队列/内存边界、超时与连接关闭；验证正常 HTTP/2、多路复用和 backpressure。
- 验收：锁文件不再包含受影响 h2 版本；Actix、Rocket 两条路径均验证；HTTP/2 功能保留。

### ZSTD-001：zstd 跨流复用上下文数据泄漏

- Advisory：RUSTSEC-2026-0391
- Crate/version：zstd 0.13.3
- 风险：复用 context 时，已放弃 stream 的数据可能泄漏到后续 stream。
- 优先级：P1
- 依赖链：salvo_core 0.89.3 -> zstd 0.13.3 -> zstd-safe 7.2.4。
- 源码/配置位置：Cargo.toml 的 salvo dependency features（显式 test）；crates/easyexcel-salvo/Cargo.toml；Cargo.lock salvo_core。
- 上游修复版本：zstd >= 0.13.4 或 >= 0.14.3。
- 验证：创建未完成 stream 后复用 context，确认 session reset 和输出隔离；正常压缩/解压与并发回归。
- 验收：无受影响 zstd 版本；Salvo test utilities 和压缩能力不被静默移除。

### ZSTD-002：zstd-safe Seekable 解压长度越界

- Advisory：RUSTSEC-2026-0398
- Crate/version：zstd-safe 7.2.4
- 风险：越界 offset 可使 Vec 长度被设置到 capacity 之外，造成内存暴露/破坏。
- 优先级：P0
- 依赖链：salvo_core -> zstd 0.13.3 -> zstd-safe 7.2.4。
- 源码/配置位置：Cargo.toml Salvo test feature；Cargo.lock zstd/zstd-safe。
- 上游修复版本：zstd-safe >= 8.2.0，或 7.x 修复线 >= 7.4.0。
- 验证：seekable decompression 使用末尾、越界、u64::MAX 附近 offset；检查返回错误/边界截断，不得暴露未初始化内存或挂起。
- 验收：无受影响 zstd-safe 版本；若保持 zstd 0.13，优先验证 7.4.0 与 zstd 0.13.4 的兼容性。

### ZSTD-003：zstd-safe 流式解压失败时暴露未初始化内存

- Advisory：RUSTSEC-2026-0403
- Crate/version：zstd-safe 7.2.4
- 风险：损坏的 legacy frame 可使流式解压错误路径错误提交输出 buffer position，Vec 长度覆盖未初始化内存。
- 优先级：P0
- 依赖链与配置位置：同 ZSTD-002。
- 上游修复版本：zstd-safe >= 7.4.0 或 >= 8.2.0。
- 验证：5/6-byte corrupt legacy frame；覆盖 stream Decoder write/flush/drop；检查失败路径不会提交未写入字节。
- 验收：无受影响 zstd-safe 版本；恶意 legacy frame 回归测试通过。

## 4. Unsoundness 警告（11 项）

这些项目虽然在 CI 输出中归类为 allowed warnings，但仍属于安全审计结果；不得以“不是 CVE”或“workspace 禁止 unsafe”作为忽略理由。依赖 crate 自身的 unsafe 实现仍可能影响调用者。

### ZSTD-SAFE-001：zstd-safe 7.2.4 的 10 项 unsoundness

共同依赖链：salvo_core 0.89.3 -> zstd 0.13.3 -> zstd-safe 7.2.4。

共同仓库位置：Cargo.toml 的 salvo test feature；crates/easyexcel-salvo/Cargo.toml；Cargo.lock 的 zstd/zstd-safe 项。具体调用是否触及每个 affected function 尚未证明；安全策略要求先移除受影响版本，而不是依赖“当前似乎没调用”。

| Advisory | 受影响函数/问题 | 风险 | 上游修复版本 |
|---|---|---|---|
| RUSTSEC-2026-0395 | CCtx::set_parameter / CParameter::StableInBuffer 生命周期 | use-after-free / memory exposure | >= 7.4.0 或 >= 8.2.0 |
| RUSTSEC-2026-0396 | Seekable 读取 NULL/越界 seek table | 越界读取/内存安全 | >= 7.4.0 或 >= 8.2.0 |
| RUSTSEC-2026-0397 | OutBuffer::around_pos 暴露未初始化内存 | 未初始化内存暴露 | >= 7.4.0 或 >= 8.2.0 |
| RUSTSEC-2026-0399 | CCtx::init_using_cdict / DCtx::init_using_ddict | use-after-free | >= 7.4.0 或 >= 8.2.0 |
| RUSTSEC-2026-0400 | DCtx 一次性解压与未完成 stream 混用 | use-after-free / memory exposure | >= 7.4.0 或 >= 8.2.0 |
| RUSTSEC-2026-0401 | train_from_buffer 样本长度溢出 | 越界读取 | >= 7.4.0 或 >= 8.2.0 |
| RUSTSEC-2026-0402 | CCtx::try_clone 多线程 context | heap buffer overflow | >= 7.4.0 或 >= 8.2.0 |
| RUSTSEC-2026-0404 | DCtx::try_clone | double free | >= 7.4.0 或 >= 8.2.0 |
| RUSTSEC-2026-0405 | InBuffer::pos 越界 | 越界读取 | >= 7.4.0 或 >= 8.2.0 |
| RUSTSEC-2026-0406 | unfinished frame 时修改多线程 context | use-after-free | >= 7.4.0 或 >= 8.2.0 |

验证方法：除升级锁定版本外，针对仓库可达的 zstd API 补充损坏帧、stream/context 复用和 seekable 边界回归；如仓库没有直接使用某 affected API，记录调用可达性分析，但不得保留已知受影响版本来替代修复。

### CACHE-001：lru 0.16.4 panic-safety unsoundness

- Advisory：RUSTSEC-2026-0253
- Crate/version：lru 0.16.4
- 依赖链：ssfmt 0.1.2 -> lru 0.16.4；easyexcel-format 依赖 ssfmt。
- 仓库位置：Cargo.toml 的 ssfmt workspace dependency；crates/easyexcel-format/Cargo.toml 的 ssfmt.workspace；Cargo.lock 的 ssfmt/lru。
- 风险：LruCache::pop 在淘汰元素析构触发 panic 的特定场景可能破坏内部状态并导致 use-after-free。
- 上游修复版本：lru 0.18.2 已修复该 panic-safety 问题；更高版本如 0.18.5/0.19.0 需检查 MSRV 与 API。
- 约束：ssfmt 0.1.2 声明 lru ^0.16，不能通过 cargo update 将 lru 0.16 强行升级到 0.18。需要确认是否有兼容的 ssfmt 上游发布；若没有，评估维护最小 ssfmt fork/patch，或升级/替换格式化依赖。不得仅修改 Cargo.lock 伪造 semver 兼容。
- 验证：格式化结果 Java golden tests；LRU 正常淘汰/缓存行为；panic-in-drop 的 catch_unwind 回归（若 API 允许）；MSRV 和许可证检查。
- 验收：锁文件不含受影响 lru 版本；格式化结果与现有 golden fixture 一致。

## 5. 版本约束与架构边界

1. 根 workspace 当前 rust-version 为 1.88。Actix Web 4.18.2 官方文档要求 Rust 1.89+；Salvo adapter 当前也单独声明 1.89。不得为了升级 Actix 把整个核心库 MSRV 无条件提升到 1.89。需要按 crate 边界更新 adapter/example 的 MSRV，并证明核心包仍支持 1.88。
2. 不得移除 HTTP/2、Brotli、gzip/deflate、zstd 或 Salvo/Rocket/Actix 集成来减少依赖数量。
3. h2 0.3.27 通过 Actix HTTP 与 Rocket/Hyper 两条路径出现；必须分别解决并证明没有遗留。
4. Salvo 0.89.3 的 test feature 是 zstd 0.13.3 依赖来源之一。先确认其 feature 作用域和测试用法，再选择依赖更新/特性隔离/上游补丁。不能直接删掉测试 feature 导致测试覆盖缺失。
5. ssfmt 0.1.2 对 lru 的 ^0.16 约束使其不能直接升级 lru 0.18；必须通过上游升级或受控 fork 解决。
6. 现有 ZIP 9.1.3 兼容修改必须保留；不得回退 ZIP 9 或其 API 适配。

## 6. 修复顺序与 Agent 写入边界

所有任务由一个集成负责人维护 Cargo.toml、Cargo.lock、deny.toml 和最终依赖图。其他 Agent 可以并行做只读上游调查、源码定位和测试设计，但不得并发修改 Cargo.lock 或根 Cargo.toml。

建议修复顺序：

1. SEC-ACTIX：actix-web / actix-http 漏洞组，包含 adapter MSRV。
2. SEC-CALAMINE：Calamine CFB/OLE2 parser。
3. SEC-H2：Actix 与 Rocket 两条 h2 0.3 路径。
4. SEC-ZSTD：zstd 0.13.3、zstd-safe 7.2.4 的漏洞和全部 unsoundness。
5. SEC-LRU：ssfmt -> lru 0.16.4。
6. SEC-VERIFY：全 workspace 审计、许可证/来源、MSRV、测试和证据归档。

每组完成后必须单独提交，运行该组测试与 cargo tree -i；再进入下一组。任何版本解析失败、API 不兼容、MSRV 变化或功能边界变化，先更新 OpenSpec design 并说明决策，不允许 Agent 自行猜测。

## 7. 全局验收门槛

- CI Security Audit workflow 的实际命令为 cargo audit；本地可额外执行 cargo audit --no-fetch --deny warnings。两者最终均须退出码 0；0 vulnerabilities；0 unsound warnings；0 yanked crate。
- cargo deny check advisories bans licenses sources：退出码 0；无新增许可证/来源豁免。
- cargo tree -i 对本报告所有 advisory crate 检查，不存在受影响版本。
- cargo tree -d 审查新增重复版本；不得为了“减少重复”牺牲 MSRV/功能。
- cargo fmt --all -- --check、cargo clippy --workspace --all-targets --all-features -- -D warnings。
- workspace 测试及 adapter 专项安全回归通过；不得只用 cargo check 替代安全行为测试。
- 保留 ZIP 9.1.3 的 round-trip、XLSX 读写、模板与 Java golden tests。
- 记录每次验证的命令、退出码、Rust toolchain、Git SHA、Cargo.lock SHA-256、RustSec advisory-db commit、CI Run/Job 链接和测试数量。
- PR #5 在全部门槛通过前保持 Draft，不合并。

## 8. 参考来源

- RustSec RUSTSEC-2026-0446：https://rustsec.org/advisories/RUSTSEC-2026-0446.html
- RustSec RUSTSEC-2026-0447：https://rustsec.org/advisories/RUSTSEC-2026-0447.html
- RustSec RUSTSEC-2026-0448：https://rustsec.org/advisories/RUSTSEC-2026-0448.html
- RustSec RUSTSEC-2026-0455：https://rustsec.org/advisories/RUSTSEC-2026-0455.html
- RustSec RUSTSEC-2026-0436：https://rustsec.org/advisories/RUSTSEC-2026-0436.html
- RustSec RUSTSEC-2026-0258：https://rustsec.org/advisories/RUSTSEC-2026-0258.html
- RustSec zstd-safe advisories：https://rustsec.org/packages/zstd-safe.html
- Actix Web 4.18.2：https://docs.rs/crate/actix-web/4.18.2
- zstd 0.14.3 / 0.13.4 release matrix：https://docs.rs/crate/zstd/latest
- lru 0.18.5 changelog：https://docs.rs/crate/lru/latest/source/CHANGELOG.md
- ssfmt 0.1.2 dependency metadata：https://docs.rs/crate/ssfmt/0.1.2

- Actix HTTP security fix commit：https://github.com/actix/actix-web/commit/61c6a15c1c0e505a5cd6842610a5f1faea432a83

## 9. Remediation progress — SEC-ACTIX

### 9.1 Change record

- Remediation commit: 51d04e9b282a6a8b0f3ee6199179c3cdfcf10e17
- PR #5 branch: fix/compatible-security-deps-20261031
- actix-web: 4.14.0 -> 4.18.2
- actix-http: 3.19.0 -> 3.20.3, pinned to upstream commit 61c6a15c1c0e505a5cd6842610a5f1faea432a83 until crates.io publication.
- Actix adapter, Actix example and Web conformance package MSRV: 1.89.
- Core workspace MSRV remains 1.88.
- CI MSRV job now excludes Actix adapter/example from the 1.88 workspace check and validates Actix, Salvo and Web conformance packages with Rust 1.89.
- No HTTP/2 or compression feature was removed.

### 9.2 Regression tests and local verification

| Command | Result |
|---|---|
| cargo test --locked -p easyexcel-web-conformance --test actix_security --all-features | 3 passed: large-window Brotli rejection, decoded output limit, malformed chunked request smuggling |
| cargo test --locked -p easyexcel-web-conformance --test actix --all-features | 5 passed, 1 pre-existing ignored |
| cargo test --locked -p easyexcel-actix --all-features | 10 passed |
| cargo fmt --all -- --check | passed |
| cargo clippy --locked -p easyexcel-actix -p easyexcel-demo-actix -p easyexcel-web-conformance --all-targets --all-features | passed; existing unrelated unused-import warning in easyexcel remains |
| cargo +1.88 check --locked -p easyexcel -p easyexcel-web --all-features | passed |
| cargo +1.89 check --locked -p easyexcel-actix -p easyexcel-demo-actix -p easyexcel-web-conformance --all-features | passed |
| cargo +1.89 check --locked -p easyexcel-salvo --all-features | passed |
| git diff --check | passed |

The full workspace Rust 1.88 command was also attempted on Windows and stopped in examples/axum/src/main.rs because that example unconditionally imports tokio::signal::unix. This is a Windows-host limitation of the existing example; the repository CI runs this MSRV job on Ubuntu. The targeted core 1.88 checks passed.

### 9.3 Re-audit after SEC-ACTIX

Local cargo audit --json after commit 51d04e9 reported:

- 5 vulnerabilities remain: RUSTSEC-2026-0436 (calamine 0.37.0), RUSTSEC-2026-0258 (h2 0.3.27), RUSTSEC-2026-0391 (zstd 0.13.3), RUSTSEC-2026-0398 and RUSTSEC-2026-0403 (zstd-safe 7.2.4).
- 11 unsound warnings remain: 10 on zstd-safe 7.2.4 and 1 on lru 0.16.4.
- 0 yanked warnings.
- Actix advisories RUSTSEC-2026-0446, 0447, 0448 and 0455 are absent from the post-fix audit result.
- cargo audit exits 1 and cargo deny check advisories bans licenses sources exits 9 because the other remediation groups remain open. This is expected; the PR remains Draft.

The h2 0.3.27 tree still has two paths: patched actix-http 3.20.3 and rocket_http 0.5.1 -> hyper 0.14.32. This is explicitly not considered fixed by SEC-ACTIX and remains SEC-H2 work.

GitHub CI after the push:
- Security Audit Run 42450269311 (queued at record time).
- CI Run 42450269320 (queued at record time).
## 10. Remediation progress — SEC-CALAMINE

### 10.1 Source mapping and dependency decision

- Remediation commit: e4044f1505caac0413ecd4e8d4043e4d69c1f54e.
- Calamine 0.37.0 -> 0.38.0 in root Cargo.toml and Cargo.lock.
- Upstream security fix: https://github.com/tafia/calamine/commit/b8ec4f09cba6aea7d8359ababc240c9ed064751b.
- Calamine appears as a dev-dependency in crates/easyexcel/Cargo.toml and tests/easyexcel-test/Cargo.toml. In the production facade, crates/easyexcel/src/read/cell_conversion.rs:4-5 and row_processing.rs:14-15 gate their calamine imports with #[cfg(test)]. Native production XLS/XLSX readers remain easyexcel-xls / easyexcel-xlsx.
- Additional test-only callsites include crates/easyexcel/src/read/tests.rs, write/tests.rs, template/tests.rs, template/template_writer.rs and tests/easyexcel-test.

#### 10.1.1 Exhaustive Calamine and Salvo test-feature source references

The following inventory was verified with git grep over crates/ and tests/, plus a repository scan for salvo::test, TestClient, and Cargo test features.

**Calamine dependency boundary**
- Workspace version declaration: Cargo.toml:86.
- EasyExcel facade: [dev-dependencies] at crates/easyexcel/Cargo.toml:52-55, with calamine.workspace at line 54.
- Java-style test host: [dev-dependencies] at tests/easyexcel-test/Cargo.toml:11-15, with calamine.workspace at line 15. No direct calamine Rust source import was found under tests/easyexcel-test by the repository scan.
- CFB security integration test: crates/easyexcel/tests/calamine_cfb_security.rs:11-12 (the four tests use the patched parser and malicious fixtures).

**EasyExcel facade test-only callsites**
- crates/easyexcel/src/read/cell_conversion.rs:4-5,7-8,49-50,64-65: Calamine imports and conversion helpers are all cfg(test).
- crates/easyexcel/src/read/row_processing.rs:4-15,29-43: Calamine import and workbook/sheet helpers are cfg(test).
- crates/easyexcel/src/read/tests.rs:14-16: read-test oracle imports.
- crates/easyexcel/src/write/tests.rs:20: write-test oracle imports.
- crates/easyexcel/src/template/tests.rs:11,246: template-test oracle imports/type references.
- crates/easyexcel/src/template/template_writer.rs:958-961,1076-1124: cfg(test) module tests_extra uses Calamine to read back generated template output.
- crates/easyexcel/src/template/tests_cases/cases_01_split/chunk_01.rs:121: test-only Calamine error fixture.
- crates/easyexcel/src/write/builder/excel_writer_sheet_builder.rs:431-451: test reads back generated XLSX.
- crates/easyexcel/src/write/builder/excel_writer_table_builder.rs:372-388: cfg(test) module uses Calamine.
- crates/easyexcel/src/write/excel_writer_builder_tests/tests.rs:12 and tests/cases_01_split/chunk_01.rs:17,168,263,288,327 plus chunk_02.rs:81,106: writer-builder test read-back.
- crates/easyexcel/src/write/excel_writer_core_tests/tests_extra.rs:9 and tests_extra/cases_01_split/chunk_01.rs:304-313: XLS/XLSX writer-core test read-back.
- crates/easyexcel/src/write/executor/excel_write_add_executor.rs:376-383: cfg(test) module imports Calamine.
- crates/easyexcel/src/write/tests_cases/cases_07_split/chunk_02.rs:30: test-only range assertion.

No production XLS/XLSX parser callsite was found. The two production readers remain easyexcel-xls and easyexcel-xlsx; Calamine is retained as a dev-dependency/test oracle.

**Salvo test feature boundary**
- Workspace dependency: Cargo.toml:103 enables only http1 and server; it does not enable test.
- The only feature opt-in: tests/easyexcel-web-conformance/Cargo.toml:34 enables test on the dev-dependency.
- The only source consumer found: tests/easyexcel-web-conformance/tests/salvo.rs:11,17,65,84,107,144, where salvo::test::TestClient is imported and used for the five conformance requests.
- No other salvo::test or TestClient source consumer was found by the repository scan.

### 10.2 Reproducible malicious-input fixtures

Copied from the fixed upstream commit into crates/easyexcel/tests/fixtures/calamine/:

| Fixture | Bytes | SHA-256 | Expected outcome |
|---|---:|---|---|
| cfb_difat_oom.xls | 512 | B0FF76967DFBB4E4870C848A6365C74F361E48BFD98AFED1A5ED378B0AB2A6A8 | Reject cyclic DIFAT (typed error) |
| cfb_dir_short_chunk_panic.xls | 4153 | 82DE55B1A1C0EE69E8460E29172AF5711A109F7AD04A75F7D112E938FE3EC9C7 | Skip short trailing directory chunk; Sheet1 remains readable |

The intentionally malformed OLE2 fixtures should only be parsed using patched calamine or a resource-limited sandbox.

### 10.3 Verification

- `cargo test --locked -p easyexcel --test calamine_cfb_security`: Windows 3/3 cross-platform tests passed; Linux CI executes 4/4 including the memory-limited child-process regression.
- `cargo +1.88 check --locked -p easyexcel --all-targets --all-features`: passed; latest GitHub MSRV job passes.
- Latest Ubuntu workspace CI: EasyExcel facade library 2331/2331; `java_full_parity_tests` 95/95; `java_golden_tests` 88/88; `tests/easyexcel-test/tests/round_trip.rs` 17/17.
- Full workspace test jobs passed on Ubuntu and macOS in CI Run 43354243228; coverage-pr, rustfmt, Clippy, docs, MSRV and benchmark build all passed.
- `cargo audit --no-fetch --deny warnings`: 0 vulnerabilities, 0 unsound warnings and 0 yanked packages; `cargo deny check advisories bans licenses sources` passes. GitHub Security Audit Run 43354243171 / Job 131652325083 passes.
- T2.4/T2.5/T2.6 are now covered: the generated one-mini-sector fixture is structurally validated by `cfb::CompoundFile` and its Workbook stream is exactly 64 bytes; Linux CI runs the `prlimit --as=536870912` child-process regression; full workspace, Java parity, Java Golden and round-trip suites pass. EasyExcel explicitly does not support XLSB/XLSM, so those formats are not claimed as compatibility targets.
- No advisory ignore/allow and no framework features were removed.
- The CFB security test source is `crates/easyexcel/tests/calamine_cfb_security.rs`; upstream fixtures are `crates/easyexcel/tests/fixtures/calamine/cfb_difat_oom.xls` (512 bytes) and `cfb_dir_short_chunk_panic.xls` (4153 bytes). The generated fixture's `Workbook` mini stream is independently opened through cfb 0.14.0 and asserted to contain exactly 64 bytes before Calamine is invoked.
- Linux memory-bound test evidence: CI Run 43354243228 / Job 131652325519 logs `test_cfb_cyclic_difat_is_bounded_in_memory_limited_subprocess ... ok`; it invokes util-linux `prlimit --as=536870912` before starting the child test process.
- Current workspace format scope: CHANGELOG.md states XLSX/XLS/CSV only; XLSM and XLSB are not supported. Repository-wide search found no Xlsb code path or `.xlsb` fixture. Calamine remains a dev-dependency/test oracle rather than the production reader.


---

审计结论：PR #5 已经解决 ZIP 8.6.0 yanked 的构建阻塞方向，但当前依赖锁文件仍有明确的安全问题。必须完成上述逐项修复并重新审计后才能合并。


## 11. Current remediation verification snapshot — 2026-11-13

> This section records the PR #5 security remediation state through pushed commit 2a6c2d6159ad4a670eb9a1f38a307fbfa6f2c843 plus the local ref-swap 0.1.3 follow-up. The 2a6c2d Security Audit reported newly published RUSTSEC-2026-0528 on ref-swap 0.1.2; the local lock update removes it and strict audit/deny pass against the refreshed RustSec database. A new CI run is required after pushing the lock update.

### 11.1 Current dependency decisions

| Component | Current locked/source state | Security result |
|---|---|---|
| actix-web | 4.19.0 | RUSTSEC-2026-0447 absent |
| actix-http | 3.21.0, local compatibility package at vendor/actix-http | RUSTSEC-2026-0446/0448/0455 absent; HTTP/2 retained |
| h2 | 0.4.20 only | RUSTSEC-2026-0258 absent; no h2 0.3.x lock entry |
| Rocket / rocket_http | 0.5.2 | Uses Hyper 1.12.0 and h2 0.4.20 |
| calamine | 0.38.0 | RUSTSEC-2026-0436 absent |
| zstd | 0.13.4 and 0.14.3 (different parent semver lines) | RUSTSEC-2026-0391 absent |
| zstd-safe | 7.4.0 and 8.2.0 (matching parent semver lines) | RUSTSEC-2026-0398/0403 and all 10 unsound warnings absent |
| lru | 0.16.4, local path patch at vendor/lru | RUSTSEC-2026-0253 absent |
| rustix | 1.1.6 | RUSTSEC-2026-0486 absent; fixes newly issued zero-length xattr buffer unsoundness |
| ref-swap | 0.1.3 | RUSTSEC-2026-0528 absent; fixes unsafe variance and OptionRefSwap null handling |

### 11.2 Actix HTTP 3.21.0 / h2 0.4 bridge

Upstream Actix HTTP 3.21.0 still declares optional h2 0.3.27. The current implementation therefore uses a local copy of the upstream 3.21.0 package and changes its internal HTTP/2 bridge only:

- vendor/actix-http/Cargo.toml: h2 0.4.20 and an http1 alias for http 1.5.
- vendor/actix-http/src/h2/dispatcher.rs: convert incoming h2/http 1.x request parts into Actix http 0.2 Request; convert Actix response head into http 1.x before sending through h2 0.4.
- vendor/actix-http/src/h2/mod.rs: h2 0.4 handshake, connection and RecvStream integration.
- vendor/actix-http/EASYEXCEL_PATCH.md: upstream provenance, local diff scope, compatibility boundary and validation requirements.
- tests/easyexcel-web-conformance/tests/actix_h2.rs: end-to-end h2c request/response test using the workspace h2 0.4 client.
- h2 0.4.20 includes the upstream empty DATA-frame flood/backpressure fix and upstream regression tests from h2 PR #935 / commit 193833e87c639e39751f339a9c375a44c8bfab54.

Rocket was upgraded to 0.5.2. Its rocket_http 0.5.2 path uses Hyper 1.12.0 and h2 0.4.20, removing the former Hyper 0.14/h2 0.3 path.

### 11.2.1 Salvo test feature scope

The Salvo 0.89.3 TestClient is used only in tests/easyexcel-web-conformance/tests/salvo.rs. The root workspace Salvo dependency now enables only http1/server; the test feature is enabled only on the conformance crate's dev-dependency. Therefore zstd 0.13.4/zstd-safe 7.4.0 remain available to the test harness but are no longer part of the normal production dependency graph. Actix compression uses zstd 0.14.3/zstd-safe 8.2.0. Validation: cargo tree --locked -e normal -i zstd@0.13.4 has no matching production package; cargo test -p easyexcel-web-conformance --test salvo passed; cargo test -p easyexcel-salvo passed.

### 11.3 LRU panic-safety backport

vendor/lru contains upstream lru 0.16.4 with the panic-safety fix from lru-rs PR #238 / commit f9a7f00fcf2d33e00adb03758cb350aaaa52cddb. The node is detached from the intrusive list before invoking the key destructor. The crate version and public API remain 0.16.4 to satisfy ssfmt 0.1.2's lru ^0.16 constraint. Cargo.toml [patch.crates-io] directs lru to the local copy. vendor/lru/EASYEXCEL_PATCH.md documents provenance and scope.

### 11.4 Local verification evidence

| Command | Result |
|---|---|
| cargo tree --locked -i h2 | Only h2 0.4.20; Actix HTTP, Rocket HTTP, Hyper 1, Axum, Poem, Salvo and Warp paths converge on 0.4.20 |
| cargo tree --locked -i h2@0.3.27 | No matching package; exit 101 is expected for an absent package |
| cargo test --locked -p easyexcel-web-conformance --test actix_h2 --all-features | 1 passed |
| cargo test --locked -p easyexcel-web-conformance --all-features | 41 tests detected, 41 passed, 0 failed; Ubuntu/macOS CI also passes |
| cargo test --locked -p easyexcel-actix --all-features | 10 passed |
| cargo test --manifest-path vendor/lru/Cargo.toml easyexcel_panic_safety_tests::pop_panicking_key_drop_keeps_list_consistent | 1 passed (panic is caught by catch_unwind as expected) |
| cargo test -p easyexcel-format | 93 passed |
| cargo fmt --all -- --check | passed |
| cargo clippy --locked -p easyexcel-actix -p easyexcel-demo-actix -p easyexcel-web-conformance --all-targets --all-features | passed |
| cargo +1.88 check --locked -p easyexcel -p easyexcel-web --all-features | passed |
| cargo +1.89 check --locked -p easyexcel-actix -p easyexcel-demo-actix -p easyexcel-web-conformance --all-features | passed |
| cargo audit --deny warnings | exit 0; 0 vulnerabilities, 0 unsound warnings, 0 yanked packages; GitHub Security Audit Run 43354243171 / Job 131652325083 passed on 67dab800 |
| cargo deny check advisories bans licenses sources | passed after Salvo test feature scoping; no source allow or advisory ignore added |
| git diff --check | passed |

### 11.4.1 Fresh advisory delta after the initial security cleanup

GitHub Security Audit on commit 325ff561cb22c23272cbdca59c0ed7a651ceaed3 refreshed the RustSec database and found one new allowed unsoundness warning:

- Advisory: RUSTSEC-2026-0486
- Crate/version: rustix 1.1.4
- Fixed version: rustix >= 1.1.6
- Affected API: rustix::fs xattr query/read/list functions with a zero-length output buffer; size-query results could be incorrectly marked initialized, permitting safe code to set a Vec length beyond capacity.
- Dependency update: cargo update -p rustix --precise 1.1.6.
- Lock verification: rustix 1.1.4 absent; rustix 1.1.6 present.
- Fresh local verification: cargo audit --deny warnings exits 0; cargo deny check advisories bans licenses sources exits 0.
- Advisory source: https://rustsec.org/advisories/RUSTSEC-2026-0486.html
- Upstream fix reference: https://github.com/bytecodealliance/rustix/commit/8e6aaa6713d2bb64b1fdd8c472e3a32a10a76d70

This advisory was issued on 2026-11-11 and therefore was not present in the original 2026-11-08 baseline. The CI run on 325ff56 completed Security Audit successfully but reported one allowed unsound warning; commit 9bc05f2 upgrades rustix to 1.1.6. Fresh local cargo audit --deny warnings and cargo deny checks pass with no remaining advisory. GitHub Security Audit Run 43354243171 / Job 131652325083 passed on 67dab800.

### 11.4.2 Fresh advisory delta: RUSTSEC-2026-0528

GitHub Security Audit on commit 2a6c2d6159ad4a670eb9a1f38a307fbfa6f2c843 used a refreshed RustSec database and found a newly published unsoundness advisory:

- Advisory: RUSTSEC-2026-0528
- Date: 2026-11-12
- Crate/version: ref-swap 0.1.2
- Dependency path: Rocket 0.5.2 -> ref-swap 0.1.2 (also reached by easyexcel-rocket, easyexcel-demo-rocket and Web conformance dev-dependencies).
- Issue: incorrect variance of RefSwap/OptionRefSwap could permit use-after-free; OptionRefSwap::into_inner did not check for null.
- Fixed version: ref-swap 0.1.3.
- Upstream release: https://github.com/Nitrokey/ref-swap/releases/tag/v0.1.3
- Advisory: https://rustsec.org/advisories/RUSTSEC-2026-0528.html
- Lock update: cargo update -p ref-swap --precise 0.1.3.
- API compatibility: Rocket 0.5.2 and the EasyExcel Rocket adapter compile without source changes; Rocket adapter tests pass 8/8 and the Rocket example check passes.
- Fresh local security verification: update RustSec advisory-db to commit 4ef2df8e806a65e2f1ecc14f56f51249062b23d6 (1509 advisories); cargo audit --no-fetch --deny warnings passes with zero vulnerabilities, unsound warnings and yanked packages; cargo deny advisories/bans/licenses/sources passes.

The 2a6c2d Security Audit job completed successfully under the workflow's default warning policy but reported this one allowed unsound warning. The ref-swap 0.1.3 lock update is committed locally as 9f3988c2b1e76c8c04dabc69442fca9c893aae3b but is not yet pushed at this snapshot; push it and rerun CI before marking PR #5 ready.

### 11.5 Known validation blockers / not yet accepted


1. The Windows local full-workspace test command exceeded the 60-minute runner limit while executing long large-workbook tests; no test failure was reported before timeout. GitHub CI workspace tests pass on both Ubuntu and macOS. The focused Windows missing-file and CFB tests pass.
2. Full workspace Clippy on Windows cannot compile examples/axum/src/main.rs because it unconditionally imports tokio::signal::unix. GitHub Linux CI Clippy passes for the workspace.
3. cargo clippy --workspace --all-targets --all-features -- -D warnings additionally exposes pre-existing workspace warnings (unused imports, doc_markdown, assert_is_empty, etc.); no repository-wide warning cleanup was performed as part of this security patch.
4. Running vendored Actix HTTP standalone Rustls integration tests on this Windows host fails before test execution because OpenSSL development libraries are not installed. The end-to-end Actix h2c bridge test passes; Linux CI must run the upstream Actix HTTP Rustls/HTTP2 test targets.
5. CFB hostile-input acceptance is verified: Windows runs the three cross-platform tests 3/3; Ubuntu CI Run 43354243228 / Job 131652325519 runs all four tests, including the Linux-only prlimit --as=536870912 child-process regression. The generated mini-stream fixture is independently validated as a structurally valid CFB with a 64-byte Workbook stream. The final test uses no unsafe Rust and no libc dev-dependency.
6. Commit 2a6c2d6159ad4a670eb9a1f38a307fbfa6f2c843 is pushed to PR #5 and adds the Linux Actix HTTP/2 + Rustls compile guard. Its Security Audit Run 43460022248 completed but reported RUSTSEC-2026-0528 on ref-swap 0.1.2. Commit 9f3988c2b1e76c8c04dabc69442fca9c893aae3b upgrades ref-swap to 0.1.3; fresh strict audit/deny pass against RustSec DB 4ef2df8. The lock fix is committed locally but not yet pushed. Keep PR #5 Draft until the new lock commit passes Security Audit and all CI jobs.

Current worktree:
- Branch: fix/compatible-security-deps-20261031
- Latest pushed commit: 2a6c2d6159ad4a670eb9a1f38a307fbfa6f2c843
- Latest local lock-fix commit: 9f3988c2b1e76c8c04dabc69442fca9c893aae3b (not yet pushed).
- CI Run 43460022228: rustfmt, MSRV, benchmark build, Clippy and docs passed; Ubuntu/macOS tests and coverage-pr were still running at the time of this snapshot. Security Audit Run 43460022248 / Job 132004123745 completed but reported RUSTSEC-2026-0528 on ref-swap 0.1.2.
- Ubuntu test Job 131652325519 from the preceding fully green commit 67dab800: all four calamine_cfb_security tests passed, including the prlimit child process; Java parity 95/95, Java Golden 88/88 and round-trip 17/17 passed.
- Local Rocket validation after ref-swap update: Rocket adapter tests 8/8; Rocket example check passes; strict cargo audit/deny pass against RustSec DB 4ef2df8e806a65e2f1ecc14f56f51249062b23d6 (1509 advisories).
- Cargo.lock rustix: 1.1.6; ref-swap: 0.1.3. Current Cargo.lock SHA-256: 192556FB425768E93FB15020E6F8DB1FDBCF19804F7FA7A75229B7440F931822.
- rustc/cargo 1.99.0; cargo-audit 0.22.2; cargo-deny 0.20.2; RustSec advisory-db commit 4ef2df8e806a65e2f1ecc14f56f51249062b23d6.
