# Java 4.0.3 public API parity evidence

这组文件把 Java `javap` 条目与 Rust 公开 API 的对照拆成两层：

- `java-rust-public-api.json`：3236 个 Java 类型/成员的逐项状态与 Rust ID；schema v2 重建后还会
  逐项写入实现策略和 carrier：`implementation_carriers` 由实际 Rust public ID 的 package
  推导，`capability_carriers` 只记录格式引擎、模型或 derive 等下游协作者。实现策略只允许 `existing_implementation`、
  `idiomatic_alternative`、`needs_implementation`。
- `public-api-evidence.json`：人工审定、可递归 include 的 compile/behavior/Java-golden 可执行证据。

候选映射不能直接标记为 `verified`。每个 `verified` 条目必须同时满足：

1. Rust ID 存在于 default-features 与 all-features 的 `cargo-public-api` 快照。
2. `compile_probe` 声明并实际执行 stable/default-features 和 stable/all-features 命令。
3. `behavior_test` 绑定当前 Java ID 和全部 Rust ID，并通过可观察行为与错误分支测试。
4. `java_golden` 由 EasyExcel 4.0.3 重新生成，文件哈希和 Rust consumer 均通过。
5. 证据运行结果绑定当前 catalog SHA，命令参数、退出码及源码 SHA 全部匹配。

schema v2 对未验证项同样 fail-closed：candidate/ambiguous 的 Rust ID 也必须存在于权威的全
workspace 快照，并同时在 default/all-features 公开；每个 implementation carrier 必须是实际发布
crate，并覆盖其 Rust ID 所属 package；capability carrier 也必须属于发布 workspace，且不能与
public implementation carrier 重复。惯用替代和
真实缺口必须写明语义说明。证据目录中的每条记录都必须绑定已知 Java/Rust ID、合法 kind、非空
命令与源码哈希，并且与执行结果一一对应，不能依靠“尚未 verified”隐藏陈旧候选或孤儿证据。

确实存在多个静态候选时，不允许按名称顺序自动选一个。证据 catalog 可声明
`mapping_resolutions`，每项包含一个 `java_id` 和最终选择的非空 `rust_ids`；最终集合必须是候选器
原始 Rust ID 集合的子集，不能引入新载体。同一 Java ID 在完整 include 树中只能声明一次，未知
Java ID 直接报错。消歧后仍须 compile probe、Rust behavior 和 Java golden 三类证据全部覆盖最终
Rust ID 集合，才能进入 `verified`；未显式消歧的 `ambiguous` 会保持阻断状态。

候选器同时生成 Rust public API 的确定性补集：只登记没有被任何 Java 候选使用的 Rust ID，记录
所属 crate、kind、feature modes、完整签名和说明。它不增加 Java verified 数；某个 ID 一旦成为
Java carrier，就会自动从补集消失，避免同一 API 同时冒充 Java 映射和 Rust extension。

逐类型推进不等于逐类型照搬：已有同语义实现直接绑定原 owner；Java 运行时反射、POI 泄漏类型、
无状态工具类或语言协议由 Rust trait/module/derive/后端中立对象承载时记录为惯用替代；只有全
workspace 不存在有效 carrier 时才允许新增实现。facade 只提供统一入口、薄适配和必要重导出。

验证报告的 `progress` 同时给出三种不可混用的进度：`classified_java_api_items` 表示已经逐项写明
三类策略，`coded_java_api_items` 只统计存在 Rust public carrier 的 existing/alternative 项，
`verified_java_api_items` 只统计三重可执行证据齐全的项。前两者用于提高逐类型编码速度，发布完成率
仍只能引用最后一项。

`classified/coded` 也不是宽松的声明计数。条目必须是权威清单中的唯一 Java ID；Rust ID 必须同时
存在于当前 default/all-features 快照；`implementation_carriers` 必须与这些 Rust ID 的实际 package
集合精确相等；capability carrier 只能记录不重复的已发布下游协作者。空 ID、漂移 ID、重复映射、
批量附加 facade/engine carrier 或缺失替代语义说明的条目均不进入两个辅助分子。
报告中的 `implementation_strategy` 采用同一严格集合；原始声明数量另存为
`declared_implementation_strategy`，只用于诊断候选器输出，不能作为进度。
`verified_java_api_items` 同样只计算本次校验中分类、Rust ID、三类证据绑定、源码哈希和执行
attestation 全部无错的唯一 Java ID，而不是直接统计 JSON 里的 `status=verified` 声明。Java/Rust
manifest 哈希漂移时三个进度分子全部归零；evidence catalog 漂移或未提供执行结果时 verified 归零，
证据目录、执行结果或 Rust authoritative scope 的结构不合法时也按同样原则 fail-closed。报告通过
`manifest_structure_valid`、`evidence_structure_valid`、`classification_progress_authoritative` 和
`verified_progress_authoritative` 明确标记口径失效。

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

当前检入快照仍是禁测前的 schema v1（205 verified）；新版候选器和验证器已经要求 schema v2，
但在解除“禁止测试/门禁”前不重生成或冒充新的分类结果。

当前已验证 205/3236 项：facade 31 项、`ExcelReader` 11 项、`ExcelWriter` 13 项、`ExcelBuilder` 7 项、`ExcelBuilderImpl` 8 项、`ExcelAnalyser` 5 项、`ExcelAnalyserImpl` 6 项、`ExcelReadExecutor` 3 项、`CsvExcelReadExecutor` 4 项、`CsvReadContext` 3 项、`DefaultCsvReadContext` 4 项、`XlsReadContext` 3 项、`DefaultXlsReadContext` 4 项、`XlsxReadContext` 3 项、`DefaultXlsxReadContext` 4 项、`XlsListSheetListener` 4 项、`XlsSaxAnalyser` 5 项、`XlsxSaxAnalyser` 5 项、`XlsxTagHandler` 5 项、`AbstractXlsxTagHandler` 6 项、`AbstractCellValueTagHandler` 3 项、`XlsRecordHandler` 3 项、`IgnorableXlsRecordHandler` 1 项、`AbstractXlsRecordHandler` 3 项、`MergeCellsRecordHandler` 4 项、`NoteRecordHandler` 4 项、`HyperlinkRecordHandler` 4 项、`TextObjectRecordHandler` 4 项，以及 `BoundSheetRecordHandler`、`BofRecordHandler`、`BlankRecordHandler`、`BoolErrRecordHandler`、`NumberRecordHandler`、`IndexRecordHandler`、`EofRecordHandler`、`LabelRecordHandler`、`SstRecordHandler`、`LabelSstRecordHandler`、`FormulaRecordHandler`、`StringRecordHandler`、`RkRecordHandler`、`ObjRecordHandler`、`DummyRecordHandler` 各 3 项；其余 3031 项仍按 fail-closed 阻断发布。候选器现可解析带 supertrait 的 Rust trait 签名，并能将 Java public static field 对应到 Rust associated const，因此累计 39 项从 unmapped 前移到 candidate；它们尚未获得行为证据，不计入 verified。
