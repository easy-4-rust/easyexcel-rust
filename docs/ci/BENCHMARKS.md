# 性能基准工作流

本文档说明 CI 中 nightly 性能基准测试的执行、回归门禁以及 baseline 更新流程。

## Nightly Benchmark Job

### 触发条件

- **定时触发**：每天 UTC 02:00（北京时间 10:00）自动执行
- **手动触发**：通过 `workflow_dispatch` 在 GitHub Actions 页面手动启动

### 执行环境

- **Runner**：`ubuntu-latest`（Linux x64，与 baseline 命名 `nightly-ubuntu-x64` 一致）
- **Rust 工具链**：1.97.1（与 `spec/runtime_contract.rust_toolchain` 一致）
- **Java 版本**：17（Temurin 发行版）
- **Locale**：`en_US.UTF-8`（与 `spec/runtime_contract.locale` 一致）

### 执行流程

1. **编译 Rust benchmark runner**：`cargo build --release -p easyexcel-benchmark-runner`
2. **编译 Java benchmark runner**：从 Maven Central 拉取 easyexcel 4.0.3 JAR，编译 `benchmarks/java-runner/src/` 下的 runner 类
3. **执行 benchmark matrix**：`python3 benchmarks/scripts/run_matrix.py --profile nightly`
   - 行数：100,000
   - 温度：cold + steady
   - 预热：3 次（steady 温度）
   - 测量：7 次/温度
   - 场景：9 个（xlsx-stream-write, xlsx-full-write, xlsx-event-read, xlsx-workbook-read, xlsx-roundtrip, xls-batched-write, xls-event-read, csv-stream-write, csv-event-read）
4. **Baseline regression gate**：`python3 benchmarks/scripts/compare_results.py --baseline ... --require-baseline`

### 回归门禁阈值

| 指标 | 阈值 | 说明 |
|------|------|------|
| Median throughput regression | > 10% | 当前 median 低于 baseline median 的 90% |
| Peak RSS regression | > 15% | 当前 median RSS 高于 baseline median 的 115% |

若任一阈值被突破，`compare_results.py` 返回 exit code 1，CI job 失败。

## 解读回归报告

CI 产出两个 artifact：

### 1. `nightly-compare-results-{run_id}`

路径：`/tmp/compare-results.json`

关键字段：
- `passed`：`true` 表示全部通过，`false` 表示存在回归
- `failures`：失败原因列表（如 `"median throughput regression: rust/matrix/steady/xlsx-stream-write/..."`)
- `summaries`：每个 benchmark group 的统计摘要（median, p5, p95, stdev 等）
- `cross_runtime_ratios`：Rust/Java 吞吐比值及 bootstrap 置信区间

### 2. `nightly-raw-results-{run_id}`

路径：`/tmp/nightly-run/`

包含：
- `raw-results.jsonl`：每个 benchmark sample 的原始计时数据
- `environment-manifest.json`：运行环境快照（OS, CPU, 内存, Rust/Java 版本等）
- `fixtures/`：生成的测试 fixture 文件及 manifest

### 回归排查步骤

1. 下载 `nightly-compare-results-{run_id}` artifact
2. 查看 `failures` 列表，定位回归的 benchmark group
3. 对比 `summaries` 中当前值与 baseline 值
4. 下载 `nightly-raw-results-{run_id}`，检查 `raw-results.jsonl` 中对应 group 的各 trial 数据
5. 若回归由环境波动导致（如 CI runner 资源竞争），可手动重跑 nightly workflow

## Baseline 更新流程

### 当前 Baseline

- 文件：`benchmarks/baselines/nightly-ubuntu-x64.json`
- 来源：macOS 本地 100K rows 短测（commit b363cb9，Agent 38）
- 状态：临时 baseline，待首次 Linux nightly 成功后替换

### 更新 Baseline 的步骤

1. **确认回归是预期的**（如算法优化导致吞吐提升，或已知的内存增长）
2. **本地验证**：在与 CI 相同的环境下（ubuntu-latest, Rust 1.97.1, Java 17）运行 benchmark
3. **生成新 baseline**：
   ```bash
   python3 benchmarks/scripts/compare_results.py \
     /tmp/nightly-run/raw-results.jsonl \
     --spec benchmarks/spec/benchmark-suite-v1.json \
     --profile nightly \
     --output benchmarks/baselines/nightly-ubuntu-x64.json \
     --repo-root .
   ```
4. **提交 PR**：更新 `benchmarks/baselines/nightly-ubuntu-x64.json`，说明更新原因
5. **CI 验证**：PR 合并后，下一个 nightly 将使用新 baseline

### Baseline 文件格式

```json
{
  "schema_version": 1,
  "profile": "nightly",
  "passed": true,
  "failures": [],
  "spec_sha256": "...",
  "summaries": {
    "rust/matrix/cold/xlsx-stream-write/null/100000/1": {
      "throughput_rows_per_second": { "median": 123456.78, ... },
      "peak_rss_bytes": { "median": 12345678, ... },
      ...
    }
  }
}
```

注意：baseline 必须通过 `compare_results.py` 的 `validate_stable_baseline` 校验：
- `schema_version` 必须为 1
- `profile` 必须匹配
- `spec_sha256` 必须匹配当前 spec
- `passed` 必须为 `true`，`failures` 必须为空

## 文件说明

| 文件 | 用途 |
|------|------|
| `.github/workflows/nightly-benchmark.yml` | Nightly CI workflow 定义 |
| `benchmarks/baselines/nightly-ubuntu-x64.json` | Nightly profile 的稳定 baseline |
| `benchmarks/scripts/run_matrix.py` | Benchmark matrix 执行脚本 |
| `benchmarks/scripts/compare_results.py` | 结果聚合 + baseline 回归门禁 |
| `benchmarks/spec/benchmark-suite-v1.json` | Benchmark 场景、profile、门禁定义 |
| `benchmarks/rust-runner/` | Rust benchmark runner 源码 |
| `benchmarks/java-runner/` | Java benchmark runner 源码 |

## 相关文档

- [COVERAGE.md](COVERAGE.md) - 覆盖率工作流
- [ARCHITECTURE.md](../ARCHITECTURE.md) - 项目架构
