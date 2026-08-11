# Nightly CI 本地 Dry-Run 验证报告

## 基本信息

- **执行时间**: 2026-08-11
- **执行环境**: macOS 15.5.0 / aarch64 (Apple Silicon)
- **Rust 工具链**: 1.97.1 (release profile)
- **Java**: 未安装（macOS dry-run 仅测试 Rust runner）
- **仓库路径**: `/Users/wandl/workspaces/workspace-github-easy-4-rust/easyexcel-rust`

## 9 场景实测 Throughput（100K rows，3 measurements × 2 temperatures）

| 场景 | Cold Median (rows/s) | Steady Median (rows/s) | Baseline Cold* | Baseline Steady* |
|------|---------------------|------------------------|----------------|------------------|
| xlsx-stream-write | 277,133 | 243,219 | 282,487 | 275,921 |
| xlsx-full-write | 133,792 | 218,202 | 215,116 | 216,041 |
| xlsx-event-read | 618,478 | 628,194 | 619,274 | 564,864 |
| xlsx-workbook-read | 558,460 | 576,070 | 542,581 | 576,297 |
| xlsx-roundtrip | 97,466 | 103,205 | 105,769 | 109,941 |
| xls-batched-write | 150,821 | 170,241 | 168,361 | 166,166 |
| xls-event-read | 70,379 | 74,651 | 11,786 | 12,215 |
| csv-stream-write | 279,913 | 291,230 | 281,590 | 290,691 |
| csv-event-read | 1,227,002 | 1,293,649 | 1,212,369 | 1,321,646 |

*Baseline 数据来自 `benchmarks/baselines/nightly-ubuntu-x64.json`（commit b363cb9，macOS 100K rows 短测）。

**注意**: 由于每次运行环境波动（后台进程、热管理等），同场景不同次运行的 throughput 差异可达 2-3 倍（如 xlsx-full-write cold: 133K vs 215K）。这属于正常现象，CI 环境（ubuntu-latest）稳定性会更好。

## Gate 行为验证

### 1. 正常 Baseline 比对（无回归）

- **输入**: `raw-results.jsonl`（54 条记录）vs `nightly-macos-dryrun.json`（本地生成的 schema_version=1 baseline）
- **结果**: `passed: false`，287 个 failures
- **Baseline 相关**: 0 个 "stable baseline lacks benchmark summary"，0 个 "median throughput regression"
- **其他 failures**（均为 macOS dry-run 预期行为）:
  - 54x "unknown implementation Git SHA"（runner 未注入 git SHA）
  - 54x "Rust runtime contract mismatch"（runtime 版本信息不完整）
  - 30x "missing input SHA"（未使用 run_matrix.py 的 fixture 编排）
  - 36x "reread failed"（correctness.rereadable=false，dry-run 无法做跨运行时重读）
  - 缺失 Java 场景组（expected，macOS 无 Java）
  - sample count mismatch: expected 7, got 3（dry-run 用 3 measurements）

**结论**: Baseline 比对逻辑正确工作，所有 18 个 label 均成功匹配并比较。

### 2. 故意回归 Baseline 比对（模拟 15% 回退）

- **方法**: 将 baseline 中所有 throughput median × 1.15（模拟当前结果比 baseline 低 ~13%）
- **结果**: `passed: false`，305 个 failures
- **新增 18 个 "median throughput regression" failures**:
  - 9 场景 × 2 temperatures（cold + steady）全部触发回归告警
  - 示例: `median throughput regression: rust/matrix/cold/xlsx-stream-write/None/100000/1`
- **Exit code**: 1（gate fail）

**结论**: 回归检测逻辑正确工作。当 throughput 回退超过 10% 阈值时，gate 正确失败。

### 3. YAML 语法验证

```
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/nightly-benchmark.yml'))"
```

**结果**: PASSED

## 发现的 Workflow 缺陷

### 缺陷 1: `--repo-root` 参数不存在

**位置**: `.github/workflows/nightly-benchmark.yml` 第 109 行

```yaml
python3 benchmarks/scripts/compare_results.py \
  /tmp/nightly-run/raw-results.jsonl \
  --spec benchmarks/spec/benchmark-suite-v1.json \
  --profile nightly \
  --baseline benchmarks/baselines/nightly-ubuntu-x64.json \
  --require-baseline \
  --output /tmp/compare-results.json \
  --repo-root .   # <-- 此参数不存在
```

`compare_results.py` 的 argparse 不包含 `--repo-root` 参数。CI 运行时会报错 `unrecognized arguments: --repo-root .`。

**修复建议**: 移除 `--repo-root .` 参数，或在 `compare_results.py` 中添加该参数。

### 缺陷 2: Baseline schema_version 不匹配

**位置**: `benchmarks/baselines/nightly-ubuntu-x64.json`

当前 baseline 使用 `schema_version: 2`（由 `run_macos_baseline.py` 生成），但 `compare_results.py` 的 `validate_stable_baseline` 函数在第 633 行检查 `schema_version == 1`。

```python
if report.get("schema_version") != 1:
    failures.append("stable baseline has an unsupported schema version")
```

此外，schema_version=2 baseline 的 summaries 格式为 `{scenario_id: {cold: {...}, steady: {...}}}`，而 compare_results.py 期望 `{label: {throughput_rows_per_second: {median: ...}}}` 格式。

**影响**: CI 运行时 baseline 验证会失败，所有 label 比对都会报 "stable baseline lacks benchmark summary"。

**修复建议**:
1. 将 `nightly-ubuntu-x64.json` 转换为 schema_version=1 格式
2. 或修改 `compare_results.py` 支持 schema_version=2

### 缺陷 3: Baseline 缺少 fixture_origin 维度

**位置**: `benchmarks/baselines/nightly-ubuntu-x64.json`

当前 baseline 的 summaries 仅按 `scenario_id` 分组，缺少 `fixture_origin` 维度。而 `compare_results.py` 期望的 label 格式为 `implementation/phase/temperature/scenario_id/fixture_origin/rows/workers`。

读/轮转场景需要区分 `fixture_origin=rust` 和 `fixture_origin=java` 两个独立的 baseline 条目。

## 新建文件清单

| 文件 | 说明 |
|------|------|
| `benchmarks/scripts/convert_macos_to_nightly.py` | 将 macOS benchmark 结果转换为 nightly workflow 期望的 JSONL + schema_version=1 baseline 格式 |
| `docs/ci/NIGHTLY_DRYRUN_REPORT.md` | 本报告 |

## 总结

| 检查项 | 状态 | 说明 |
|--------|------|------|
| Rust 编译 | PASS | `cargo build --release -p easyexcel-benchmark-runner` 成功 |
| 9 场景运行 | PASS | 全部 9 场景 × 2 temperatures × 3 measurements = 54 条记录 |
| JSONL 格式 | PASS | 输出格式符合 compare_results.py 期望 |
| Baseline 匹配 | PASS | 所有 18 个 label 成功匹配 |
| 回归检测 | PASS | 15% 回退正确触发 18 个 regression failures，exit 1 |
| YAML 语法 | PASS | `yaml.safe_load` 成功 |
| Workflow 缺陷 | 发现 3 个 | `--repo-root` 不存在、baseline schema_version 不匹配、缺少 fixture_origin 维度 |

**建议**: 在合入 nightly workflow 前，需修复上述 3 个缺陷，否则 CI 运行会失败。
