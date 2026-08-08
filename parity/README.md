# Java 4.0.3 public API parity evidence

这组文件把 Java `javap` 条目与 Rust 公开 API 的对照拆成两层：

- `java-rust-public-api.json`：3236 个 Java 类型/成员的逐项状态与 Rust ID。
- `public-api-evidence.json`：人工审定、可递归 include 的 compile/behavior/Java-golden 可执行证据。

候选映射不能直接标记为 `verified`。每个 `verified` 条目必须同时满足：

1. Rust ID 存在于 default-features 与 all-features 的 `cargo-public-api` 快照。
2. `compile_probe` 声明并实际执行 stable/default-features 和 stable/all-features 命令。
3. `behavior_test` 绑定当前 Java ID 和全部 Rust ID，并通过可观察行为与错误分支测试。
4. `java_golden` 由 EasyExcel 4.0.3 重新生成，文件哈希和 Rust consumer 均通过。
5. 证据运行结果绑定当前 catalog SHA，命令参数、退出码及源码 SHA 全部匹配。

确定性重建：

```bash
python3 scripts/suggest_public_api_mapping.py \
  --java-api docs/java-public-api-v4.0.3.json \
  --rust-api docs/rust-public-api.json \
  --output target/public-api-candidates.json
python3 scripts/apply_public_api_evidence.py \
  --mapping target/public-api-candidates.json \
  --catalog parity/public-api-evidence.json \
  --output parity/java-rust-public-api.json
python3 scripts/run_public_api_evidence.py \
  --catalog parity/public-api-evidence.json \
  --output target/public-api-evidence-results.json \
  --repo-root .
```

完整发布门禁使用 `scripts/verify-java-parity-gates.sh`。任何 candidate、ambiguous、unmapped、缺失/陈旧/失败证据都会使门禁失败。

当前已验证 169/3236 项：facade 31 项、`ExcelReader` 11 项、`ExcelWriter` 13 项、`ExcelBuilder` 7 项、`ExcelBuilderImpl` 8 项、`ExcelAnalyser` 5 项、`ExcelAnalyserImpl` 6 项、`ExcelReadExecutor` 3 项、`CsvExcelReadExecutor` 4 项、`CsvReadContext` 3 项、`DefaultCsvReadContext` 4 项、`XlsReadContext` 3 项、`DefaultXlsReadContext` 4 项、`XlsxReadContext` 3 项、`DefaultXlsxReadContext` 4 项、`XlsListSheetListener` 4 项、`XlsSaxAnalyser` 5 项、`XlsRecordHandler` 3 项、`IgnorableXlsRecordHandler` 1 项、`AbstractXlsRecordHandler` 3 项、`MergeCellsRecordHandler` 4 项、`NoteRecordHandler` 4 项，以及 `BoundSheetRecordHandler`、`BofRecordHandler`、`BlankRecordHandler`、`BoolErrRecordHandler`、`NumberRecordHandler`、`IndexRecordHandler`、`EofRecordHandler`、`LabelRecordHandler`、`SstRecordHandler`、`LabelSstRecordHandler`、`FormulaRecordHandler`、`StringRecordHandler` 各 3 项；其余 3067 项仍按 fail-closed 阻断发布。候选器现可解析带 supertrait 的 Rust trait 签名，因此 37 项从 unmapped 前移到 candidate；它们尚未获得行为证据，不计入 verified。
