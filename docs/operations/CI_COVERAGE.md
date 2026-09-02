# 覆盖率工作流

本文档说明 CI 中覆盖率报告的生成、PR 评论以及本地运行方式。

## CI 覆盖率 Job

### 1. 主覆盖率 Job (`coverage`)

**触发条件**：仅在 `release`（发布）或 `workflow_dispatch`（手动触发）时执行。

**功能**：
- 运行全量 `cargo llvm-cov`（约 20 分钟）
- 生成 HTML 报告和 `lcov.info` 文件
- 门禁：行/区域/函数覆盖率不低于 95%
- 上传 artifact（HTML 30 天，lcov 90 天）

**排除项**：
- `easyexcel-derive/src/lib.rs`（derive 宏属性行）
- `easyexcel-format/src/format/locale_generated.rs`（生成代码）

### 2. PR 覆盖率 Job (`coverage-pr`)

**触发条件**：仅在 `pull_request` 时执行。

**功能**：
- 运行 `cargo llvm-cov` 生成 `lcov.info`
- 使用 `lcov_cobertura`（Python）将 lcov.info 转换为 Cobertura XML
- 使用 `5monkeys/cobertura-action` 在 PR 上发布覆盖率评论
- 上传 `lcov.info` 作为 artifact（30 天保留）

**第三方 Action 依赖**：
- `lcov_cobertura`（Python 包）：lcov → Cobertura XML 转换
- `5monkeys/cobertura-action@master`：解析 Cobertura XML 并在 PR 上发布覆盖率评论

**权限要求**：
- `pull-requests: write`（用于在 PR 上发布评论）

## 本地运行覆盖率

### 基本用法

```bash
# 运行全量覆盖率（HTML + lcov + JSON）
./scripts/coverage.sh

# 输出目录：coverage/
# - coverage/index.html  （HTML 报告）
# - coverage/lcov.info   （lcov 格式）
# - coverage/summary.json（JSON 摘要）
```

### 快照模式

将覆盖率摘要保存到指定目录，用于历史趋势分析：

```bash
# 按日期保存快照
./scripts/coverage.sh --snapshot reports/coverage-snapshots/$(date +%F)

# 输出：reports/coverage-snapshots/2026-08-11/summary.json
```

## 门禁策略

- **CI 门禁**：行/区域/函数覆盖率不低于 95%
- **残差说明**：195 行/37 文件为数学不可达代码（测试 `?` 错误边、防御分支、derive 属性行），已由 evidence 6 逐行验证
- **PR 评论**：仅展示覆盖率统计，不作为门禁（`coverage-pr` job 不设置 `fail_below_threshold`）

## 文件说明

| 文件 | 用途 |
|------|------|
| `.github/workflows/ci.yml` | CI workflow 定义 |
| `scripts/coverage.sh` | 本地覆盖率脚本 |
| `coverage/lcov.info` | lcov 格式覆盖率数据 |
| `coverage/summary.json` | JSON 格式覆盖率摘要 |

## 相关文档

- [BENCHMARKS.md](BENCHMARKS.md) - 性能基准工作流（nightly regression gate）
- [ARCHITECTURE.md](../ARCHITECTURE.md) - 项目架构
- [compatibility.md](../compatibility.md) - 兼容性验证（含 evidence 6 覆盖率验证）
