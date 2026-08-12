# easyexcel-rust Superpowers 规格驱动开发体系

> **建立日期**：2026-08-12
> **基线**：Alibaba EasyExcel 4.0.3 + easyexcel-rust v0.1.3
> **命名约定**：对齐 `liteflow/docs/superpowers` —— `plans/<date>-<feature>.md`、`specs/<date>-<feature>-design.md`

---

## 概述

本目录是 easyexcel-rust 项目的规格驱动开发中心。所有设计依据（specs/）和执行计划（plans/）均在此结构化管理，遵循 Superpowers SDD（Spec-Driven Development）方法论。

### 命名约定（与 liteflow 一致）

| 目录 | 格式 | 示例 |
|---|---|---|
| `plans/` | `<YYYY-MM-DD>-<feature>.md` | `2026-08-12-q3-roadmap.md` |
| `specs/` | `<YYYY-MM-DD>-<feature>-design.md` | `2026-08-12-xls-streaming-design.md` |

- 文件名**不携带项目名前缀**（项目上下文由 `docs/superpowers/` 目录本身承载）。
- `specs/` 是设计依据：RFC、审计报告、设计决策、状态报告。
- `plans/` 是执行清单：路线图、迁移计划、性能优化方案。

---

## 实施计划（plans/，9 个）

| 文件 | 主题 |
|---|---|
| [2026-08-12-q3-roadmap.md](plans/2026-08-12-q3-roadmap.md) | 2026 Q3 推进路线图（总清单） |
| [2026-08-12-migration-roadmap.md](plans/2026-08-12-migration-roadmap.md) | 全量迁移路线图 |
| [2026-08-12-migration-gap-closure.md](plans/2026-08-12-migration-gap-closure.md) | 迁移 Gap 闭环 |
| [2026-08-12-performance-optimization.md](plans/2026-08-12-performance-optimization.md) | 事件读追上 Java 吞吐（任务清单） |
| [2026-08-12-write-optimization.md](plans/2026-08-12-write-optimization.md) | 写侧恒定内存与写优化（任务清单） |
| [2026-08-12-coverage-improvement.md](plans/2026-08-12-coverage-improvement.md) | 测试覆盖率盲区闭环（任务清单） |
| [2026-08-12-xls-cli-integration.md](plans/2026-08-12-xls-cli-integration.md) | 基础能力拆分与 xls-cli 产品化 |
| [2026-08-12-hutool-poi-adoption.md](plans/2026-08-12-hutool-poi-adoption.md) | Hutool POI Excel 采纳计划 |
| [2026-08-12-ecosystem-roadmap.md](plans/2026-08-12-ecosystem-roadmap.md) | Easy document 生态路线图 |

## 设计规格（specs/，21 个）

| 文件 | 主题 | 类型 |
|---|---|---|
| [2026-08-12-project-history-design.md](specs/2026-08-12-project-history-design.md) | 项目历史与里程碑 | 历史索引 |
| [2026-08-12-compliance-audit-design.md](specs/2026-08-12-compliance-audit-design.md) | Rust 项目规范合规审计 | 审计报告 |
| [2026-08-12-dependency-audit-design.md](specs/2026-08-12-dependency-audit-design.md) | 依赖安全审计 | 审计报告 |
| [2026-08-12-code-audit-design.md](specs/2026-08-12-code-audit-design.md) | 代码审计（unsafe / stub / panic） | 审计报告 |
| [2026-08-12-fuzz-status-design.md](specs/2026-08-12-fuzz-status-design.md) | Fuzz 测试状态 | 状态报告 |
| [2026-08-12-test-audit-design.md](specs/2026-08-12-test-audit-design.md) | Java→Rust 测试对齐审计 | 审计报告 |
| [2026-08-12-test-parity-status-design.md](specs/2026-08-12-test-parity-status-design.md) | 测试对比迁移状态 | 状态报告 |
| [2026-08-12-coverage-verify-design.md](specs/2026-08-12-coverage-verify-design.md) | 测试覆盖率验证 | 审计报告 |
| [2026-08-12-nightly-dryrun-report-design.md](specs/2026-08-12-nightly-dryrun-report-design.md) | Nightly CI 本地 Dry-Run 验证 | 状态报告 |
| [2026-08-12-workflow-fixes-design.md](specs/2026-08-12-workflow-fixes-design.md) | Workflow 缺陷修复与 java_id 规范化 | 设计决策 |
| [2026-08-12-annotation-field-audit-design.md](specs/2026-08-12-annotation-field-audit-design.md) | Write/Style 注解字段对齐审计 | 审计报告 |
| [2026-08-12-large-file-cohesion-review-design.md](specs/2026-08-12-large-file-cohesion-review-design.md) | 501–800 行文件内聚性复核 | 审计报告 |
| [2026-08-12-csv-stub-strategy-design.md](specs/2026-08-12-csv-stub-strategy-design.md) | easyexcel-csv STUB 处置策略 | 设计决策 |
| [2026-08-12-read-spill-decision-design.md](specs/2026-08-12-read-spill-decision-design.md) | RFC：读链路恒定内存 spill 可行性 | RFC |
| [2026-08-12-formula-cache-decision-design.md](specs/2026-08-12-formula-cache-decision-design.md) | RFC：公式引擎结果缓存（dirty-cell 增量重算） | RFC |
| [2026-08-12-parallel-listener-design.md](specs/2026-08-12-parallel-listener-design.md) | RFC：并行监听器流水线设计 | RFC |
| [2026-08-12-xls-streaming-design.md](specs/2026-08-12-xls-streaming-design.md) | RFC：XLS 流式读写设计 | RFC |
| [2026-08-12-xls-cli-capability-matrix-design.md](specs/2026-08-12-xls-cli-capability-matrix-design.md) | xls-cli 能力矩阵 | 状态报告 |
| [2026-08-12-xls-source-provenance-design.md](specs/2026-08-12-xls-source-provenance-design.md) | xls fork 迁入来源记录 | 历史索引 |
| [2026-08-12-poi-probe-exclusions-design.md](specs/2026-08-12-poi-probe-exclusions-design.md) | POI Internal Probe 排除说明 | 设计决策 |
| [2026-08-12-security-audit-design.md](specs/2026-08-12-security-audit-design.md) | 依赖安全审计（汇总） | 审计报告 |

---

## 目录结构

```
docs/superpowers/
├── README.md          # 本文件——约定与索引
├── plans/             # 实施计划（9 个）—— <date>-<feature>.md
├── specs/             # 设计规格（21 个）—— <date>-<feature>-design.md
└── templates/         # 文档模板（待补充）
```
