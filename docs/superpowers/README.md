# easyexcel-rust Superpowers 规格驱动开发体系

> **建立日期**：2026-08-12
> **基线**：Alibaba EasyExcel 4.0.3 + easyexcel-rust v0.1.3
> **方法论**：Superpowers SDD（Spec-Driven Development）

---

## 概述

本目录是 easyexcel-rust 项目的规格驱动开发中心。所有设计依据（specs/）和执行计划（plans/）均在此结构化管理。

### 如何使用

- **specs/** 是设计依据：RFC、审计报告、设计决策等。
- **plans/** 是执行清单：路线图、迁移计划、性能优化方案等。
- **templates/** 是文档模板（待补充）。

---

## 设计规格（specs/）

| 日期 | 文件 | 主题 | 类型 |
|---|---|---|---|
| 2026-08-12 | [project-history.md](specs/project-history.md) | 项目历史与里程碑 | 历史索引 |
| 2026-08-11 | [compliance-audit-2026-08.md](specs/compliance-audit-2026-08.md) | Rust 项目规范合规审计 | 审计报告 |
| 2026-08-11 | [deps-audit-2026-08.md](specs/deps-audit-2026-08.md) | 依赖安全审计 | 审计报告 |
| 2026-08-11 | [code-audit-2026-08.md](specs/code-audit-2026-08.md) | 代码审计（unsafe / stub / panic） | 审计报告 |
| 2026-08-11 | [fuzz-status-2026-08.md](specs/fuzz-status-2026-08.md) | Fuzz 测试状态 | 状态报告 |
| 2026-08-12 | [read-spill-decision.md](specs/read-spill-decision.md) | RFC：读链路恒定内存 spill 可行性 | RFC |
| 2026-08-12 | [formula-cache-decision.md](specs/formula-cache-decision.md) | RFC：公式引擎结果缓存（dirty-cell 增量重算） | RFC |
| 2026-08-12 | [parallel-listener-design.md](specs/parallel-listener-design.md) | RFC：并行监听器流水线设计 | RFC |
| 2026-08-12 | [xls-streaming-design.md](specs/xls-streaming-design.md) | RFC：XLS 流式读写设计 | RFC |

---

## 实施计划（plans/）

| 日期 | 文件 | 主题 | 范围 |
|---|---|---|---|
| 2026-08-11 | [roadmap-2026q3.md](plans/roadmap-2026q3.md) | 2026 Q3 推进路线图（总清单） | 全局 |
| 2026-08-10 | [migration-gap-closure.md](plans/migration-gap-closure.md) | 迁移 Gap 闭环路线图 | 迁移 |
| 2026-08-12 | [performance-optimization.md](plans/performance-optimization.md) | 事件读优化（EVENT-READ-OPTIMIZATION） | 性能 |
| 2026-08-12 | [coverage-improvement.md](plans/coverage-improvement.md) | 测试覆盖率缺口闭环 | 测试 |
| 2026-08-12 | [write-optimization.md](plans/write-optimization.md) | 写链路恒定内存优化 | 性能 |
| 2026-07-23 | [ecosystem-roadmap.md](plans/ecosystem-roadmap.md) | 生态路线图（easydoc/easyofd/easypdf） | 生态 |

---

## 与 docs/ 根目录的关系

| docs/ 根目录文件 | 定位 | 说明 |
|---|---|---|
| `ARCHITECTURE.md` | 架构说明 | 保留——用户文档，不属于计划/规格 |
| `API.md` | API 参考 | 保留——用户文档 |
| `GUIDE.md` | 使用指南 | 保留——用户文档 |
| `compatibility.md` | 兼容性矩阵 | 保留——用户文档 |
| `benchmarks.md` | 基准测试说明 | 保留——参考文档 |
| `test-parity-status.md` | 测试对等状态 | 保留——参考文档 |

---

## 目录结构

```
docs/superpowers/
├── README.md          # 本文件——约定与索引
├── plans/             # 实施计划（6 个）
│   ├── roadmap-2026q3.md
│   ├── migration-gap-closure.md
│   ├── performance-optimization.md
│   ├── coverage-improvement.md
│   ├── write-optimization.md
│   └── ecosystem-roadmap.md
├── specs/             # 设计规格（9 个）
│   ├── project-history.md
│   ├── compliance-audit-2026-08.md
│   ├── deps-audit-2026-08.md
│   ├── code-audit-2026-08.md
│   ├── fuzz-status-2026-08.md
│   ├── read-spill-decision.md
│   ├── formula-cache-decision.md
│   ├── parallel-listener-design.md
│   └── xls-streaming-design.md
└── templates/         # 文档模板（待补充）
```
