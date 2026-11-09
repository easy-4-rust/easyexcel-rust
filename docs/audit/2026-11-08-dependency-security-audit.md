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

### 10.2 Reproducible malicious-input fixtures

Copied from the fixed upstream commit into crates/easyexcel/tests/fixtures/calamine/:

| Fixture | Bytes | SHA-256 | Expected outcome |
|---|---:|---|---|
| cfb_difat_oom.xls | 512 | B0FF76967DFBB4E4870C848A6365C74F361E48BFD98AFED1A5ED378B0AB2A6A8 | Reject cyclic DIFAT (typed error) |
| cfb_dir_short_chunk_panic.xls | 4153 | 82DE55B1A1C0EE69E8460E29172AF5711A109F7AD04A75F7D112E938FE3EC9C7 | Skip short trailing directory chunk; Sheet1 remains readable |

The intentionally malformed OLE2 fixtures should only be parsed using patched calamine or a resource-limited sandbox.

### 10.3 Verification

- cargo test --locked -p easyexcel --test calamine_cfb_security -- --nocapture: 2 passed.
- cargo +1.88 check --locked -p easyexcel --all-targets --all-features: passed.
- cargo test --locked -p easyexcel --lib -- --skip choice_excel_executor_propagates_executor_creation_errors: 2330 passed; one existing Windows locale-dependent error-message assertion excluded.
- cargo test --locked -p easyexcel-test --test round_trip: 17 passed.
- Local cargo audit --json: RUSTSEC-2026-0436 removed. Remaining vulnerabilities: RUSTSEC-2026-0258 (h2), 2026-0391 (zstd), 2026-0398/0403 (zstd-safe). Unsound warnings remain 11.
- T2.4/T2.5/T2.6 remain partially open: separate 64-byte mini-sector fixture, bounded-subprocess stress test, full workspace and Java Golden still pending.
- No advisory ignore/allow and no framework features were removed.


---

审计结论：PR #5 已经解决 ZIP 8.6.0 yanked 的构建阻塞方向，但当前依赖锁文件仍有明确的安全问题。必须完成上述逐项修复并重新审计后才能合并。
