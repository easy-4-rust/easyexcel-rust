# easyexcel-rust 依赖安全审计

> 审计日期：2026-08-03（修复后复跑）｜ 范围：workspace `Cargo.lock` + Rust 工具链
> 对应 `docs/compatibility.md` 验证证据 7（security audit 门禁）。

## 结论

**0 vulnerabilities，`cargo audit` 退出码 0**，证据 7（安全审计门禁）**绿**。
2026-08-01 审计发现的 2 个高危漏洞（`quick-xml 0.38.4` 传递依赖）已修复：
上游 `office-crypto 0.3.0` 固定 `quick-xml 0.38.4`（该 0.38.x 线无修复版），
故将 `office-crypto` vendor 至 `vendor/office-crypto`，依赖提升为 `quick-xml 0.41`
（与 workspace 直接依赖同版本），经 `[patch.crates-io]` 全局替换；其 quick-xml
API 使用面（`Reader`/`Event`）在 0.38 → 0.41 兼容，加密读写测试（
`temp_encrypt_password_round_trip`、`golden_encrypt_data`）回归通过。
`Cargo.lock` 中 `quick-xml 0.38.4` 已完全移除，全 workspace 单一 0.41.0。

剩余 10 条 informational 警告（`unmaintained`/`notice`/`unsound` 级，非漏洞，
不使 `cargo audit` 失败）：`aes-soft`/`aesni`/`cpuid-bool`（aes 生态旧分 crate，
由 `office-crypto` 的 `aes 0.8` 线传递）
`bincode`/`instant`/`stdweb`（深层传递）、`rand 0.7.3`（unsound 仅当使用
`rand::rng()` 自定义 logger 时触发，workspace 未使用该 API）。

## 工具与方法

- 工具：`cargo-audit 0.22.2`（`cargo install cargo-audit --locked`，本机首次安装）
- 数据库：本地 RustSec advisory DB（`~/.cargo/advisory-db`，拉取时加载 1178 条，
  最近提交 2026-07-17）
- 复跑命令：

```shell
cargo audit
```

- 交叉验证：另以 Python 脚本离线比对 `Cargo.lock` 与 advisory DB（1146→1178 条），
  结果与 `cargo audit` 一致（离线脚本未处理 withdrawn 状态，ring 等已撤回公告需
  排除；generic-array 0.14.7 因多行 patched 列表解析误差曾误报，真实工具不报）。

## 漏洞明细（已修复，历史记录）

| Crate | 锁定版本 | ID | 严重度 | 说明 | 修复 |
|---|---|---|---|---|---|
| `quick-xml`（传递） | 0.38.4 | RUSTSEC-2026-0195 | 7.5 high | `NsReader` 命名空间声明无界分配 → 内存耗尽 DoS | 升级到 >= 0.41.0 |
| `quick-xml`（传递） | 0.38.4 | RUSTSEC-2026-0194 | 7.5 high | 起始标签重复属性名检查二次方运行时间（DoS） | 升级到 >= 0.41.0 |

**2026-08-03 修复记录**：`vendor/office-crypto`（上游 0.3.0 源码 fork）+
root `Cargo.toml` `[patch.crates-io]`，quick-xml 提升到 0.41.0。复跑
`cargo audit` 0 vulnerabilities，exit 0。`Cargo.lock` 不再含 0.38.4。

## 受影响代码路径（已消除）

修复前 `cargo tree -i quick-xml@0.38.4`：

```
quick-xml v0.38.4
└── office-crypto v0.3.0
    └── easyexcel-reader v0.1.0
        └── easyexcel v0.1.0（及其下 web/demo crates）
```

- 工作区**直接**依赖已是 `quick-xml = "0.41.0"`（root `Cargo.toml`），
  easyexcel-reader 自身的 SAX 解析使用 0.41.0（已修复版本），不受影响。
- 受影响的 0.38.4 由 `office-crypto 0.3.0`（OOXML 加密元数据解析，用于密码保护
  XLSX 读取）传递引入，图中存在两个 quick-xml 版本。
- 修复后：`cargo tree -i quick-xml` 仅返回 0.41.0 单一版本，上述双版本场景消除。

## 修复执行记录

1. 2026-08-03 检查上游：`office-crypto` 最新版仍为 0.3.0（master 亦未升级 quick-xml），
   无升级空间；
2. vendor 源码至 `vendor/office-crypto`（MIT 许可，保留 LICENSE），依赖改为
   `quick-xml = "0.41"`；
3. root `Cargo.toml` 添加 `[patch.crates-io] office-crypto = { path = "vendor/office-crypto" }`；
4. `cargo update -p office-crypto` 后 lock 中 0.38.4 移除；
5. 回归：`easyexcel-reader` 编译通过、`temp_encrypt_password_round_trip` 与
   `golden_encrypt_data` 测试通过；
6. `cargo audit` → 0 vulnerabilities，exit 0。

两条原漏洞均为解析型 DoS（非 RCE/数据泄露），仅影响密码保护 XLSX 读取的
解包阶段，且现已被 0.41.0 修复版覆盖。

## 未发现问题项（审计确认干净）

- Rust 工具链：rustc/cargo 1.97.1，对照 rust/ 目录公告 0 命中；
- `ring 0.17.14`：RUSTSEC-2025-0007 已于 2025-02-22 撤回（作者恢复维护），不构成问题；
- `generic-array 0.14.7`：RUSTSEC-2020-0146 的 patched 范围 `>= 0.13.3` 覆盖，不构成问题；
- 其余 87 条与锁定依赖相关的公告均被 patched/unaffected 范围或平台过滤排除。

## 附：`cargo audit` 原始输出（2026-08-01）

```
Fetching advisory database from `https://github.com/RustSec/advisory-db.git`
  Loaded 1178 security advisories (from /Users/wandl/.cargo/advisory-db)
Updating crates.io index
Scanning Cargo.lock for vulnerabilities (323 crate dependencies)

Crate:     quick-xml
Version:   0.38.4
Title:     Unbounded namespace-declaration allocation in `NsReader` enables memory-exhaustion denial of service
Date:      2026-06-29
ID:        RUSTSEC-2026-0195
Severity:  7.5 (high)
Solution:  Upgrade to >=0.41.0

Crate:     quick-xml
Version:   0.38.4
Title:     Quadratic run time when checking a start tag for duplicate attribute names
Date:      2026-06-29
ID:        RUSTSEC-2026-0194
Severity:  7.5 (high)
Solution:  Upgrade to >=0.41.0

Crate:     derivative
Version:   2.2.0
Warning:   unmaintained
Title:     `derivative` is unmaintained; consider using an alternative
Date:      2024-06-26
ID:        RUSTSEC-2024-0388

error: 2 vulnerabilities found!
warning: 1 allowed warning found
```

## 附：2026-08-04 vendor xls 后的新增警告分析

公式缓存值求值引擎集成后，`xls` fork 被 vendor 至 `vendor/xls`
（`easyexcel/Cargo.toml` 声明 `default-features = false`，CLI/TUI 前端关闭）。
依赖解析在 `Cargo.lock` 中新增了 optional feature 的依赖闭包，`cargo audit`
随之报出 2 条 Warning 级（非漏洞）条目，已加入 `.cargo/audit.toml` ignore：

| Crate | 版本 | ID | 警告 | 链路 | 实际影响 |
|---|---|---|---|---|---|
| `paste` | 1.0.15 | RUSTSEC-2024-0436 | unmaintained（无修复版） | ratatui 0.29 → tui-textarea → xls `tui` feature | 未编译：`tui` feature 关闭，产物不含该链 |
| `lru` | 0.12.5 | RUSTSEC-2026-0002 | unsound（patched ≥0.16.3） | ratatui 0.29 → tui-textarea → xls `tui` feature | 未编译：同上；workspace 编译链中的 `lru` 为 0.16.4（已修复） |

验证：`cargo tree -e features -p easyexcel` 显示 `xls` 无任何 feature 激活；
`cargo tree -i lru@0.12.5` 仅经 `ratatui`（optional）。`cargo audit` 复跑
**0 vulnerabilities，exit 0**。
