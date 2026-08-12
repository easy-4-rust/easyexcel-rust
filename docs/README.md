# easyexcel-rust 文档索引

> 最后更新：2026-08-12
> 结构对齐：`liteflow/docs`（用户指南 + superpowers 规格中心）

---

## 用户文档

| 文件 | 说明 |
|---|---|
| [GUIDE.md](GUIDE.md) | 使用指南（含示例） |
| [API.md](API.md) | 公共 API 参考 |
| [ARCHITECTURE.md](ARCHITECTURE.md) | 架构说明（crate 布局、数据流、依赖方向） |
| [compatibility.md](compatibility.md) | Java EasyExcel 兼容性矩阵 |
| [benchmarks.md](benchmarks.md) | 基准测试说明 |
| [LARGEREAD.md](LARGEREAD.md) | 大文件读取指南 |

## 规格与计划（superpowers/）

所有设计规格（RFC / 审计报告 / 设计决策）与实施计划（路线图 / 任务清单）集中在：

**[docs/superpowers/](superpowers/README.md)**

- 9 个实施计划（`plans/`）
- 21 个设计规格（`specs/`）
- 命名约定与 liteflow 一致：`plans/<date>-<feature>.md`、`specs/<date>-<feature>-design.md`

## 工程手册（operations/）

| 文件 | 说明 |
|---|---|
| [LINUX_RUNNER_SETUP.md](operations/LINUX_RUNNER_SETUP.md) | Linux runner 环境搭建 |
| [CROSS_RUNTIME_RUNBOOK.md](operations/CROSS_RUNTIME_RUNBOOK.md) | 跨运行时对比操作手册 |
| [CI_BENCHMARKS.md](operations/CI_BENCHMARKS.md) | CI 基准测试说明 |
| [CI_COVERAGE.md](operations/CI_COVERAGE.md) | CI 覆盖率说明 |

## 机器数据（data/）

脚本生成/消费的数据文件与迁移参考，非人工阅读文档：

| 路径 | 说明 |
|---|---|
| [data/](data/) | API 对等报告、公共 API 快照（JSON） |
| [data/migration/](data/migration/) | Java↔Rust 文件映射、对象对照表、代码树快照 |
