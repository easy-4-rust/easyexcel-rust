# easyexcel-rust 依赖安全审计

> 审计日期：2026-08-01 ｜ 范围：workspace `Cargo.lock`（323 个锁定依赖实例）+ Rust 工具链
> 对应 `docs/compatibility.md` 验证证据 7（security audit 门禁）。

## 结论

**2 个高危漏洞（未修复）+ 1 个 unmaintained 警告**，均在传递依赖上；workspace
直接依赖无已知漏洞。证据 7（安全审计门禁）当前**不绿**，需依赖升级后复跑确认。

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

## 漏洞明细

| Crate | 锁定版本 | ID | 严重度 | 说明 | 修复 |
|---|---|---|---|---|---|
| `quick-xml`（传递） | 0.38.4 | RUSTSEC-2026-0195 | 7.5 high | `NsReader` 命名空间声明无界分配 → 内存耗尽 DoS | 升级到 >= 0.41.0 |
| `quick-xml`（传递） | 0.38.4 | RUSTSEC-2026-0194 | 7.5 high | 起始标签重复属性名检查二次方运行时间（DoS） | 升级到 >= 0.41.0 |

警告（无漏洞、`informational`）：

| Crate | 锁定版本 | ID | 说明 |
|---|---|---|---|
| `derivative` | 2.2.0 | RUSTSEC-2024-0388 | 已停止维护（unmaintained），建议替换 |

## 受影响代码路径

`cargo tree -i quick-xml@0.38.4`：

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

## 修复建议（不改代码，仅记录）

1. 升级 `office-crypto` 到使用 `quick-xml >= 0.41.0` 的版本（若上游已发布）；
2. 或在 `Cargo.toml` 用 `[patch]`/直接依赖约束让 office-crypto 复用 0.41.x；
3. 修复后复跑 `cargo audit`，应显示 0 vulnerabilities。

两条漏洞均为解析型 DoS（非 RCE/数据泄露），且仅影响密码保护 XLSX 读取的
解包阶段；官方 1.0 发布前完成升级即可满足验证证据 7。

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
