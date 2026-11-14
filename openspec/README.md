# OpenSpec

本仓库此前以 docs/superpowers 的规格/计划文档为主，未发现现有 openspec/changes 变更目录。本目录从 dependency-security-remediation 开始引入 OpenSpec change 结构。

## 约定

- 每项跨模块/依赖治理工作建立一个 change 目录。
- proposal.md 定义目标和边界；design.md 记录唯一共享技术决策；tasks.md 是执行清单；specs/ 下的 spec.md 定义可验收要求。
- 安全审计的详细事实、依赖链和历史证据以 docs/audit/ 下的审计报告为准。
- 每项源码修改必须引用审计 ID 与 OpenSpec Task ID。
- Cargo.toml / Cargo.lock 由单一集成负责人写入；多个 Agent 只读调查可以并行，依赖解析和锁文件更新必须串行。
- 任务完成前，必须把提交 SHA、测试命令/结果和 CI 链接回填到 tasks.md 与审计报告。
