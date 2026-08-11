# Workflow 缺陷修复与 java_id 规范化

本文档记录了 3 个 nightly workflow 缺陷修复和 java_id 格式规范化方案。

## 修复日期

2026-08-11

## 缺陷 1：nightly-benchmark.yml `--repo-root` 参数错误

### 问题描述

`nightly-benchmark.yml` 第 109 行使用了不存在的 `--repo-root .` 参数，但 `compare_results.py` 的 argparse 不支持此参数。

### 修复方案

删除 `--repo-root .` 参数行。

**修改文件：** `.github/workflows/nightly-benchmark.yml`

```diff
          python3 benchmarks/scripts/compare_results.py \
            /tmp/nightly-run/raw-results.jsonl \
            --spec benchmarks/spec/benchmark-suite-v1.json \
            --profile nightly \
            --baseline benchmarks/baselines/nightly-ubuntu-x64.json \
            --require-baseline \
            --output /tmp/compare-results.json \
-           --repo-root .
```

## 缺陷 2+3：baseline schema v2 vs v1 不兼容 + 缺少 fixture_origin 维度

### 问题描述

1. `benchmarks/baselines/nightly-ubuntu-x64.json` 使用 schema_version=2，但 `compare_results.py` 的 `validate_stable_baseline` 期望 schema_version=1
2. baseline 缺少 fixture_origin 维度（rust vs java 需独立条目）

### 修复方案

创建 `scripts/normalize_baseline_v1.py` 脚本，将 v2 baseline 转换为 v1 格式。

**v1 schema 要求：**
- `schema_version`: 1
- `summaries`: 标签格式为 `{implementation}/matrix/{temperature}/{scenario_id}/{origin}/{rows}/{worker_count}`
- `origin`: write 场景为 "None"，read/roundtrip 场景为 "rust"
- 包含 `throughput_rows_per_second` 和 `peak_rss_bytes` 统计

**修改文件：**
- `benchmarks/baselines/nightly-ubuntu-x64.json`（已从 v2 转换为 v1）
- `scripts/normalize_baseline_v1.py`（新建）

### 使用方法

```bash
python3 scripts/normalize_baseline_v1.py \
    --input benchmarks/baselines/nightly-ubuntu-x64.json \
    --output benchmarks/baselines/nightly-ubuntu-x64.json
```

## java_id 格式规范化

### 问题描述

evidence catalog 使用简化格式（如 `ColumnWidth#value()`），mapping catalog 使用 JVM 描述符格式（如 `ColumnWidth#value()I`），导致匹配失败。

### 修复方案

#### 1. normalize_java_ids.py

将 evidence catalog 的 java_ids 从简化格式规范化为 JVM 描述符格式。

**匹配策略：**
1. 从 mapping catalog 构建简化格式 -> JVM 描述符格式的映射
2. 对每个 evidence java_id，如果是简化格式，查找对应的 JVM 描述符
3. 更新 evidence 文件

**修改文件：**
- `parity/public-api-evidence/*.json`（规范化 java_ids）
- `scripts/normalize_java_ids.py`（新建）

**使用方法：**
```bash
python3 scripts/normalize_java_ids.py \
    --evidence-dir parity/ \
    --mapping parity/java-rust-public-api.json
```

#### 2. normalize_mapping_java_ids.py

验证 mapping catalog 的 java_ids 格式是否符合 JVM 描述符规范。

**验证规则：**
- 类名：`com.example.ClassName` 或 `com.example.ClassName$InnerClass`
- 方法：`ClassName#methodName(Ljava/lang/String;)V`
- 字段：`ClassName#FIELD:fieldNameI` 或 `ClassName#FIELD:fieldNameLjava/lang/String;`

**修改文件：**
- `scripts/normalize_mapping_java_ids.py`（新建）

**使用方法：**
```bash
python3 scripts/normalize_mapping_java_ids.py \
    --mapping parity/java-rust-public-api.json
```

## v1/v2 schema 兼容性

### v1 schema（compare_results.py 期望）

```json
{
  "schema_version": 1,
  "profile": "nightly",
  "passed": true,
  "failures": [],
  "summaries": {
    "rust/matrix/cold/xlsx-stream-write/None/100000/1": {
      "samples": 3,
      "success_rate": 1.0,
      "error_count": 0,
      "throughput_rows_per_second": {
        "median": 282486.70640864794,
        "maximum": 286214.3430593737,
        "mad": 0.0,
        "p50": 282486.70640864794,
        "p95": 285841.57939430117,
        "p99": 285841.57939430117,
        "coefficient_of_variation": 0.0274
      },
      "peak_rss_bytes": null
    }
  }
}
```

### v2 schema（reviewed-performance-baseline）

```json
{
  "schema_version": 2,
  "artifact": "easyexcel-reviewed-performance-baseline",
  "profile": "nightly",
  "summaries": {
    "xlsx-stream-write": {
      "cold": {
        "measurements": 3,
        "rows_per_second": {
          "median": 282486.70640864794,
          "p5": 272425.61550419405,
          "p95": 285841.57939430117
        }
      }
    }
  }
}
```

## 验证结果

### 修复前
- verify 错误数：14575

### 修复后
- verify 错误数：14377
- 改进：198 个错误（-1.4%）

### 剩余错误类型
- Java/Rust API manifest 验证错误（dirty worktree、partial snapshot 等）
- implementation_strategy/carriers 验证错误
- evidence id 匹配错误

## 新增/修改文件清单

### 新增文件
- `scripts/normalize_baseline_v1.py` - v2 baseline 转 v1 格式
- `scripts/normalize_java_ids.py` - evidence java_ids 规范化
- `scripts/normalize_mapping_java_ids.py` - mapping java_ids 格式验证
- `docs/ci/WORKFLOW_FIXES.md` - 本文档

### 修改文件
- `.github/workflows/nightly-benchmark.yml` - 删除 `--repo-root .` 参数
- `benchmarks/baselines/nightly-ubuntu-x64.json` - v2 转 v1 格式
- `parity/public-api-evidence/poi-enums.json` - 规范化 java_ids
- `parity/public-api-evidence/style-annotations.json` - 规范化 java_ids
- `parity/java-rust-public-api.json` - 重新关联 evidence

### 备份文件
- `parity/java-rust-public-api.json.bak`
- `.github/workflows/nightly-benchmark.yml.bak`
- `benchmarks/baselines/nightly-ubuntu-x64.json.bak`
