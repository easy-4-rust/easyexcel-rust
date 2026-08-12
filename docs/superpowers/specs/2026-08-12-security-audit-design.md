# easyexcel-rust 依赖安全审计

> 最近静态审计：2026-08-06；
> 范围：workspace `Cargo.lock`、RustSec 公告、依赖许可证与来源。

## 当前结论

- `cargo audit --json`：退出码 0，502 个锁定依赖中没有命中的漏洞或警告。
- `.cargo/audit.toml`：`ignore = []`，当前结论不依赖漏洞豁免。
- `Cargo.lock`：只包含 crates.io registry 来源，不包含未知 registry 或 Git 来源。
- `cargo deny check advisories bans licenses sources`：由根目录 `deny.toml` 提供显式策略；
  advisories、许可证和来源为发布门禁，多版本依赖目前作为可见警告逐步收敛。

这里的“通过”只表示锁定依赖在审计时刻满足静态策略，不等同于运行时安全验证。

## 策略

1. RustSec 漏洞和 yanked crate 一律拒绝，默认不设置 ignore。
2. 若上游暂时没有修复版本而必须豁免，必须同时记录：公告 ID、完整依赖链、
   可利用输入、缓解措施、负责人、失效日期以及替换计划。
3. 依赖只能来自 crates.io；新增 registry 或 Git 来源必须经过单独评审并修改
   `deny.toml`，不能依赖本机默认配置。
4. 许可证采用显式 allowlist。新增许可证必须先核对发布和再分发义务。
5. 重复版本先作为警告，避免为消除重复而强行升级 MSRV；发布前应审查新增的
   重复主版本及其二进制体积、安全维护影响。

## 历史问题及清理

旧文档曾记录 `office-crypto 0.3.0 -> quick-xml 0.38.4` 的两项解析型 DoS
豁免，以及 `derivative`、`paste`、旧 `lru` 的预防性豁免。当前工作区没有 crate
实际使用根 `workspace.dependencies` 中的 `office-crypto`，锁文件也不包含
`office-crypto`、`quick-xml 0.38.4`、`derivative` 或 `paste`；`lru` 只有已修复的
0.16.4。因此已删除未使用依赖并清空所有 ignore，防止未来同名公告被静默放行。

当前 XLSX XML/SAX 路径使用 `quick-xml 0.41.0`；加密写入使用
`ms-offcrypto-writer`。具体功能正确性与恶意输入资源限制仍需由后续测试阶段验证。

## 复现命令

```shell
cargo +1.97.1 audit --json
cargo +1.97.1 deny check advisories bans licenses sources
```

审计报告必须同时保存命令退出码、RustSec 数据库提交、`Cargo.lock` 哈希与 Git SHA；
只粘贴工具的成功摘要不足以形成发布证据。
