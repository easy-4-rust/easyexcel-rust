# 安全审计档案

本目录保存可复核的安全审计基线、依赖链、风险判断、修复决策和验收证据。

| 文档 | 基线 | 状态 |
|---|---|---|
| [2026-11-08-dependency-security-audit.md](2026-11-08-dependency-security-audit.md) | PR #5 原始审计基线 dd673ea6a15909cc137d87eba26b7bedd43d323a | 历史基线；PR #5 于 2026-11-14 合并 main，审核证据与 OpenSpec 已归档至此 |
| [2026-11-15-postmerge-zip-upgrade.md](2026-11-15-postmerge-zip-upgrade.md) | main 3ad73e9aa0eb899f85474c9026a4c5f43e8f467e | ZIP 9.5.1 + strict Security/Release gates；PR #9 CI 与安全审计已通过，待最终评审/合并 |

## 规则

1. 每份审计必须固定 Git SHA、Cargo.lock、审计工具/数据库版本和 CI Run。
2. 每个 advisory 必须记录 crate/version、完整依赖链、仓库源码入口或明确标注尚未定位、风险、修复版本、回归验证和验收证据。
3. cargo audit / cargo deny 的策略不得通过 ignore、allow、feature 删除或缩小审计范围来规避。
4. 修复后更新原审计文档的状态和证据，不覆盖历史基线。
5. 本目录的审计报告是 OpenSpec 任务的事实来源；OpenSpec 任务不得自行改写风险结论。
