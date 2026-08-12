# 迁移 Gap 闭环路线图（ROADMAP-gap-closure）

> 版本：v1.0（2026-08-10）｜基线：Alibaba EasyExcel **4.0.3**
> 仓库：`easyexcel-rust` @ dev ｜ Java 源码：`/Users/wandl/workspaces/workspace-github/easyexcel`
> 范围：把 6 个迁移 gap 子任务拆成可机器校验的可执行任务清单（WBS）
> 约束：本文档只描述任务，不修改任何代码/配置/schema

---

## 0. 事实基线（已核对，修正了输入事实中的偏差）

下列数字全部来自当前仓库实际文件，**优先于任务输入中给出的"已确认事实"**——输入事实中有 4 条已过时，已在备注中标出：

| 维度 | 当前实际值 | 证据 | 输入事实对照 |
|------|-----------|------|-------------|
| parity mapping `schema_version` | **2**（已是 v2） | `parity/java-rust-public-api.json` 顶层 `schema_version: 2` | 输入称"当前是 schema v1"——**已过时** |
| status 分布 | candidate **2673** / ambiguous **479** / unmapped **84** / verified **0** | `parity/java-rust-public-api.json` `entries[]` 聚合 | 与输入一致 |
| strategy 分布 | existing **2079** / idiomatic **1073** / needs_implementation **84** | 同上 | 输入称 1657/1016/84——**已过时**（existing 已增 422，idiomatic 已增 57） |
| evidence catalog `schema_version` | **1**（未升级） | `parity/public-api-evidence.json` `schema_version:1`；`parity/public-api-evidence/converters.json` 含 `family_evidence`（3 条） | 与输入一致 |
| 4 个 POI enum | **已全部实现并已映射**（candidate/idiomatic_alternative） | `crates/easyexcel/src/enums/poi/{border_style,fill_pattern_type,horizontal_alignment,vertical_alignment}_enum.rs`；parity 中 `ContentStyle#borderBottom()` 等已 candidate | 输入称"4 个缺失"——**已过时**，gap 缩为"补 verified 证据" |
| 9 个 write/style 注解 | **9 个 parser 全部已实现**（`#[excel(...)]` 已支持） | `crates/easyexcel-derive/src/annotation/write/style/{column_width,head_row_height,content_row_height,head_style,content_style,head_font_style,content_font_style,once_absolute_merge,content_loop_merge}.rs` | 输入称"缺失 9 个注解"——**已过时**，gap 缩为"补 verified 证据 + 桥接" |
| 9 个 Java 测试类 | manifest 已 `mapped_unverified`，但其中 **7 个类无对应 Rust 测试 mod**（仅静态映射指向同名共享文件） | `docs/source-test-parity.json`；`grep -rl "mod complex_head_data_test" tests/` → NONE | 与输入"未覆盖"一致，但机制更精确 |

**结论**：真正的闭环工作量集中在 3 类——(a) evidence catalog 的 `family_evidence` → 逐 ID 物化；(b) 84 unmapped + 479 ambiguous 的逐项策略裁定与消歧；(c) 9 个测试类的真实 Rust 测试实现。POI enum 与 style 注解的"迁移"已完成，剩余仅为"补 verified 证据"。

---

## 1. 关键脚本与机制速查

| 脚本 | 作用 | 关键行为 |
|------|------|---------|
| `scripts/suggest_public_api_mapping.py` | 确定性候选器 | 输出 `target/public-api-candidates.json`，含 `candidate_rust_ids`（全集）+ `implementation_strategy` |
| `scripts/materialize_public_api_evidence.py` | 模板物化 | 把 `family_evidence` 展开为逐 Java ID 的 `evidence` 记录；任一选中 API 无 carrier 则失败 |
| `scripts/apply_public_api_evidence.py` | overlay 应用 | 读 catalog 的 `mapping_resolutions` 把 `ambiguous` 的 `rust_ids` 收窄为 resolution 子集 |
| `scripts/run_public_api_evidence.py` | 证据执行 | 跑 compile/behavior/java_golden 三类命令，写 `target/public-api-evidence-results.json` |
| `scripts/verify_public_api_parity.py` | 主验证器 | **L51**：catalog 含 `family_evidence` 直接报错；**L479-480**：`schema_version!=2` 报错；**L789**：非 verified 的每个 entry 都 `errors.append` |
| `scripts/verify-java-parity-gates.sh` | 5 道串联门禁 | gate4 重生成 mapping 并 `cmp` 检入快照；任一非 verified → 失败 |
| `scripts/generate_source_test_parity.py` | 测试静态映射 | 扫描 Rust `#[test]` 函数 doc 注释里的 `JavaClass#method` 引用；`--check` 只校验 manifest 新鲜度 |

**验证器 fail-closed 行为**（`verify_public_api_parity.py` L789）：
```python
if status != "verified":
    errors.append(f"{java_id}: status={status}")
```
即：**只要存在任意 candidate/ambiguous/unmapped，验证器返回退出码 1**。这意味着"闭环"的最终判据是 3236 项全部 `verified`，而非仅"消歧完成"。本路线图按阶段推进，每阶段定义自己的可机器校验判据。

---

## 2. 任务清单（WBS）

> 工作量单位：人时（h）。优先级 P0=阻断发布门禁、P1=显著推进 verified 数、P2=收尾完整性。

---

### 阶段 A：evidence catalog schema 与物化（子任务 1）

#### A1 升级 evidence catalog 到 schema_version=2
- **涉及文件**：
  - `parity/public-api-evidence.json`（顶层，当前 `schema_version:1`，`include` 4 个子文件）
  - `parity/public-api-evidence/converters.json`（`schema_version:1`，含 `family_evidence:3`）
  - `parity/public-api-evidence/{excel-analyser,excel-builder,excel-writer}.json`（均为 `schema_version:1`）
- **当前状态**：4 个 catalog 文件全部 `schema_version:1`；`converters.json` 含 3 条 `family_evidence`（`converter-family.compile.v1` 等，`expected_java_api_items:387`）。
- **动作步骤**：
  1. 通读 `scripts/materialize_public_api_evidence.py:178`（`materialize_family`）与 `:288`（输出 `schema_version:1`），确认物化器输出 schema 版本与 catalog 是否需要先手改。
  2. 确认 v2 catalog 新增字段要求（对照 `scripts/verify_public_api_parity.py:51` 的 `family_evidence` 拒绝逻辑与 L66/L123 的 schema 处理）。
  3. 把 5 个 catalog 文件顶层 `schema_version` 改为 `2`（仅当物化器/验证器不自行写入时）。
  4. 若 `converters.json` 仍需保留 `family_evidence` 作为源模板，则把它从 `parity/public-api-evidence/` 移到单独的 `templates/` 目录，仅作为 `--template` 输入，**不进 `include`**。
- **验收标准**：
  - `python3 -c "import json; [print(f, json.load(open(f))['schema_version']) for f in ['parity/public-api-evidence.json','parity/public-api-evidence/converters.json','parity/public-api-evidence/excel-analyser.json','parity/public-api-evidence/excel-builder.json','parity/public-api-evidence/excel-writer.json']]"` 全部输出 `2`。
  - `python3 scripts/materialize_public_api_evidence.py --template parity/public-api-evidence.json --java-api docs/java-public-api-v4.0.3.json --candidate-mapping target/public-api-candidates.json --repo-root . --output target/public-api-evidence-catalog.json` 退出码 0。
- **估算**：2h
- **依赖**：—
- **优先级**：P0

#### A2 物化 converters.json 的 3 个 family_evidence 模板
- **涉及文件**：
  - `parity/public-api-evidence/converters.json`（`family_evidence:3`，覆盖 `com.alibaba.excel.converters.` 前缀 387 个 Java 项）
  - 物化输出：`target/public-api-evidence-catalog.json`
  - 可能需要新增：`crates/easyexcel/src/converters/**/*.rs` 的逐文件 SHA 引用（已在 `source_globs` 声明）
- **当前状态**：`converters.json` 的 3 个 family 模板 `expected_java_api_items` 合计覆盖 converter 全家族；验证器 L51 拒绝任何含 `family_evidence` 的 catalog 进入门禁。
- **动作步骤**：
  1. 先跑 `scripts/suggest_public_api_mapping.py` 生成 `target/public-api-candidates.json`（确定性候选全集）。
  2. 跑 `materialize_public_api_evidence.py`（参数见 A1 验收命令），观察是否有 `needs_implementation` 项阻断物化（物化器 L20 `IMPLEMENTED_STRATEGIES` 不含 needs_implementation；converter 家族当前 strategy 应全为 existing/idiomatic）。
  3. 若报错"family cannot materialize while selected API has no carrier"，逐一定位 converters 中仍为 `needs_implementation` 的 Java ID 并先按阶段 B 处置。
  4. 物化成功后，把 catalog 中逐 ID 的 `evidence` 记录回填到检入 catalog（或确认 gate4 重生成 + `cmp` 流程不再需要检入 catalog 内容）。
- **验收标准**：
  - 物化输出 `target/public-api-evidence-catalog.json` 中 `family_evidence` 字段不存在或为空。
  - `python3 -c "import json;d=json.load(open('target/public-api-evidence-catalog.json'));print(len(d.get('evidence',[])), d.get('family_evidence','<absent>'))"` 输出 evidence 条数且 `family_evidence` 为 `<absent>`。
- **估算**：4h
- **依赖**：A1
- **优先级**：P0

#### A3 补齐 converter 家族的 compile_probe / behavior_test / java_golden 三类证据
- **涉及文件**：
  - `tests/easyexcel-test/tests/public_api_converter_evidence_tests.rs`（family 模板已声明，需确认存在与内容）
  - 物化 catalog 的每条 evidence 需绑定 `compile_probes`/`behavior_tests`/`java_golden`（验证器 L16 `REQUIRED_EVIDENCE`）
- **当前状态**：family 模板 `commands` 已列 `cargo test -p easyexcel-test --test public_api_converter_evidence_tests --no-run`（compile）等；待确认 behavior 与 golden 证据是否齐全。
- **动作步骤**：
  1. 打开 `tests/easyexcel-test/tests/public_api_converter_evidence_tests.rs`，核对是否覆盖 387 个 converter Java 项的可观察行为。
  2. 对照 `parity/README.md:13-18` 的 verified 四要件，逐项补 `behavior_test`（绑定 Java ID + Rust ID）与 `java_golden`（哈希 + consumer）。
  3. 跑 `scripts/run_public_api_evidence.py` 生成执行结果。
- **验收标准**：
  - `python3 scripts/run_public_api_evidence.py --catalog target/public-api-evidence-catalog.json --output target/public-api-evidence-results.json --repo-root .` 退出码 0。
  - `docs/public-api-parity-report.json` 中 converter 相关 Java ID 出现在 `verified_java_api_items` 集合（数量 > 0）。
- **估算**：12h（387 项批量，多为同类证据复用）
- **依赖**：A2
- **优先级**：P1

---

### 阶段 B：84 个 unmapped 项逐项处置（子任务 2）

> 现状：84 项 `status=unmapped, strategy=needs_implementation`（验证器 L789 直接报错）。按 Java 类分组的精确清单见下方"附录 1"。处置原则：能归入 `existing_implementation`（Rust 已有同语义实现）或 `idiomatic_alternative`（Rust trait/derive/后端中立对象承载）的优先重分类；确无载体才保留 `needs_implementation` 并新增实现。

#### B1 重分类 write/handler 家族（24 项 → CellWriteHandler/RowWriteHandler/SheetWriteHandler/WorkbookWriteHandler）
- **涉及 Java 类**：`write.handler.{CellWriteHandler(8), RowWriteHandler(6), SheetWriteHandler(4), WorkbookWriteHandler(6)}`
- **涉及文件**：engine crate 中的 handler trait（`crates/easyexcel/src/write/handler/` 或同级；待用 codegraph 定位）
- **当前状态**：unmapped 项大多是 Java 接口的**重载方法**——每个 handler 有"旧签名（多 POI 参数）"+"新签名（`*HandlerContext`）"两个 overload，Rust trait 通常只保留新签名。
- **动作步骤**：
  1. `codegraph query/callees -p <repo> -- CellWriteHandler` 定位 Rust 对应 trait。
  2. 判定：旧签名 overload → `idiomatic_alternative`（Rust 用 `*HandlerContext` 单签名替代 Java 双 overload），`semantic_notes` 写明"Rust 合并为单一 context 签名"。
  3. 在 `target/public-api-candidates.json` 重生成后，于 catalog 的 `mapping_resolutions`（或候选器 overlay）为每项写入 `rust_ids`（指向 Rust trait 方法 ID）+ `semantic_notes`。
  4. 重跑 `apply_public_api_evidence.py` 使 mapping `status` 升为 candidate。
- **验收标准**：
  - `python3 -c "import json;d=json.load(open('parity/java-rust-public-api.json'));print([e['java_id'] for e in d['entries'] if e['status']=='unmapped' and 'write.handler' in e['java_id']])"` 输出空列表。
  - 重生成后 `verify_public_api_parity.py` 对这 24 项不再报 `status=unmapped`。
- **估算**：4h
- **依赖**：A1
- **优先级**：P0

#### B2 重分类 context.WriteContext（10 项 + WriteContextImpl 1 项）
- **涉及 Java**：`context.WriteContext#currentSheet/currentTable/finish(Z)/getCurrentSheet/getOutputStream/getWorkbook/needHead/writeSheetHolder/writeTableHolder/writeWorkbookHolder`、`context.WriteContextImpl#finish(Z)`
- **涉及文件**：`crates/easyexcel/src/write/context/`（待定位 WriteContext 等价 trait/struct）
- **当前状态**：unmapped 主因是返回类型泄漏 POI（`org.apache.poi.ss.usermodel.{Sheet,Workbook}`、`java.io.OutputStream`）——Rust 不暴露 POI 类型。
- **动作步骤**：
  1. codegraph 定位 Rust `WriteContext` 等价物。
  2. POI 返回型方法 → `idiomatic_alternative`（Rust 返回引擎内部句柄或 `&mut dyn Write`），`semantic_notes` 写明"POI 类型不穿透 facade"。
  3. `finish(Z)` boolean 重载 → 若 Rust `finish()` 无 boolean 变体，记为 `idiomatic_alternative`（语义=Rust 用独立方法或 flag 表达）。
- **验收标准**：同 B1（针对 `context.WriteContext` 前缀）。
- **估算**：3h
- **依赖**：A1
- **优先级**：P0

#### B3 重分类 metadata/csv 家族（13 项：CsvCell 3 + CsvRow 2 + CsvSheet 6 + CsvWorkbook 2）
- **涉及 Java**：返回 POI `Row/Sheet/Workbook/DataFormat` 或 `commons-csv CSVFormat` 的 getter
- **涉及文件**：`crates/easyexcel-csv/`（CSV 后端 crate）
- **当前状态**：Rust CSV 后端不依赖 POI/commons-csv，无同型返回。
- **动作步骤**：
  1. 判定全部 13 项为 `idiomatic_alternative`（Rust 用 `easyexcel-csv` 内部 `CsvSheet/CsvRow` 等承载，不暴露第三方类型）。
  2. `semantic_notes` 写明"CSV 后端无 POI 依赖，类型不穿透"。
  3. catalog `mapping_resolutions` 写入对应 Rust ID。
- **验收标准**：`python3 -c "... 'metadata.csv' in e['java_id'] and e['status']=='unmapped'"` 输出空。
- **估算**：3h
- **依赖**：A1
- **优先级**：P0

#### B4 重分类 event / exception / read.builder 等小簇（合计 17 项）
- **涉及 Java**：
  - `event.{AbstractIgnoreExceptionReadListener(3), AnalysisEventListener(1), Handler(1)}` 共 5
  - `exception.ExcelGenerateException(3)` 共 3
  - `read.builder.{ExcelReaderBuilder(1), ExcelReaderSheetBuilder(2)}` 共 3
  - `read.listener.IgnoreExceptionReadListener(1)`、`read.metadata.ReadSheet(1)`
  - `metadata.property.{ColumnWidth,LoopMerge,OnceAbsoluteMerge,RowHeight}Property.build(...)` 共 5（依赖 style 注解类型）
- **涉及文件**：`crates/easyexcel/src/exception/`、`crates/easyexcel/src/read/listener/`、`crates/easyexcel/src/metadata/property/`
- **动作步骤**：
  1. `ExcelGenerateException` 3 个构造器 → Rust 已有等价 Error 类型则 `existing_implementation`，否则 `idiomatic_alternative`（Rust enum + `thiserror`）。
  2. listener 默认方法（`onException`/`extra`/`hasNext`/`invokeHead`）→ trait 默认实现 `existing_implementation`。
  3. `Handler#order()` → trait 关联方法 `existing_implementation`。
  4. `metadata.property.*Property.build(StyleAnnotation)` → Rust property builder `existing_implementation`（注解已在 derive 层解析）。
  5. `ExcelReaderBuilder.xlsxSAXParserFactoryName` → `idiomatic_alternative`（Rust 用 quick-xml，无 SAX factory 概念）。
  6. `ExcelReaderSheetBuilder.{doRead,doReadSync}` → `existing_implementation`（Rust builder 已有 `do_read_sync`）。
- **验收标准**：相关前缀 unmapped 全部清零（用与 B1 同款 python 校验）。
- **估算**：4h
- **依赖**：A1、B 阶段注解子任务（D1 后）
- **优先级**：P0

#### B5 重分类 util 静态常量与杂项（合计 14 项）
- **涉及 Java**：
  - `util.FileUtils#FIELD:EX_CACHE/POI_FILES`、`util.IntUtils#FIELD:MAX_POWER_OF_TWO`、`util.IoUtils#FIELD:EOF`、`util.StringUtils#FIELD:EMPTY/SPACE` 共 7 个静态常量
  - `util.ClassUtils$FieldCacheKey#toString()`、`read.metadata.ReadSheet#toString()`、`write.metadata.fill.FillConfig$FillConfigBuilder#toString()` 共 3 个 `toString`
  - `annotation.format.DateTimeFormat#use1904windowing()` 1
  - `cache.selector.{EternalReadCacheSelector,ReadCacheSelector,SimpleReadCacheSelector}#readCache(PackagePart)` 共 3
- **涉及文件**：`crates/easyexcel-util/`、`crates/easyexcel-cache/`、`crates/easyexcel/src/enums/`
- **动作步骤**：
  1. 静态常量 → `idiomatic_alternative`（Rust 用 `const`，不映射成 Java `static final` 字段；`semantic_notes` 写明"Rust const 已提供等价值"）。
  2. `toString()` → `idiomatic_alternative`（Rust `Display`/`Debug` impl）。
  3. `DateTimeFormat#use1904windowing()` → Rust `DateTimeFormat` 注解选项 `existing_implementation`。
  4. cache selector `readCache(PackagePart)` → `idiomatic_alternative`（Rust cache 不接 OPC PackagePart）。
- **验收标准**：`'util.' in java_id and status=='unmapped'` 与上述杂项前缀 unmapped 全为空。
- **估算**：2h
- **依赖**：A1
- **优先级**：P1

#### B6 重分类 write.metadata.holder 与 write.style 抽象类（4 项）
- **涉及 Java**：
  - `write.metadata.holder.AbstractWriteHolder#FIELD:ownSheetHandlerExecutionChain`、`ownWorkbookHandlerExecutionChain` 共 2
  - `write.style.AbstractCellStyleStrategy#{afterCellDispose,order}` 共 2
  - `write.style.column.AbstractColumnWidthStyleStrategy#afterCellDispose` 1（附录合计含此则 5，以实际 84 为准）
  - `write.style.row.AbstractRowHeightStyleStrategy#...`
- **涉及文件**：`crates/easyexcel/src/write/metadata/holder/`、`crates/easyexcel/src/write/style/`
- **动作步骤**：
  1. handler chain 字段 → `idiomatic_alternative`（Rust 用 Vec<Box<dyn Handler>> 替代 Java 字段）。
  2. abstract strategy 方法 → `existing_implementation`（Rust trait 默认/required 方法）。
- **验收标准**：`write.metadata.holder.AbstractWriteHolder` 与 `write.style.*` 前缀 unmapped 清零。
- **估算**：2h
- **依赖**：A1
- **优先级**：P1

#### B7 校验：84 unmapped 清零
- **涉及文件**：`parity/java-rust-public-api.json`
- **动作步骤**：跑 B1-B6 后重生成 mapping，跑候选器+apply，确认无 unmapped。
- **验收标准**：
  - `python3 -c "import json;d=json.load(open('parity/java-rust-public-api.json'));print(sum(1 for e in d['entries'] if e['status']=='unmapped'))"` 输出 `0`。
  - `scripts/verify_public_api_parity.py` 报告中 `unmapped` 相关 error 行为 0。
- **估算**：1h
- **依赖**：B1-B6
- **优先级**：P0

---

### 阶段 C：479 个 ambiguous 项消歧（子任务 3）

> 现状：每个 ambiguous 项有 2-4 个确定性候选 Rust ID（`candidate_rust_ids`，均值 2.07），`rust_ids == candidate_rust_ids`（尚未收窄）。验证器要求：每项必须在 catalog 写一条 `mapping_resolutions`，含唯一 `java_id`、非空 `rust_ids`（候选子集）、非空 `semantic_notes`。按类分批消歧，工作量按"类"而非"方法"估算（同类方法消歧理由一致）。

#### C1 消歧 StyleProperty（44 项）与 FontProperty（17 项）
- **涉及 Java**：`metadata.property.StyleProperty`（44）、`metadata.property.FontProperty`（17）
- **涉及文件**：`crates/easyexcel/src/metadata/property/{style_property,font_property}.rs`（待确认）
- **动作步骤**：
  1. 对每类 property，逐方法比对 2-4 个候选 Rust ID 的 signature（用 `docs/rust-public-api.json` 的 `signature` 字段）。
  2. 选定语义最贴切的唯一 Rust ID，写入 `mapping_resolutions`，`semantic_notes` 写明选择理由（如"该 getter 对应 Rust field 访问器，排除 builder setter 候选"）。
  3. 同类方法可批量生成 resolution（理由模板化）。
- **验收标准**：
  - `python3 -c "import json;d=json.load(open('parity/java-rust-public-api.json'));print(sum(1 for e in d['entries'] if e['status']=='ambiguous' and 'StyleProperty' in e['java_id']))"` 输出 `0`，FontProperty 同理。
- **估算**：3h
- **依赖**：A1
- **优先级**：P1

#### C2 消歧 write/metadata/style 与 write/handler/context 家族
- **涉及 Java**：`write.metadata.style.WriteCellStyle`（44）、`write.handler.context.{CellWriteHandlerContext(24), RowWriteHandlerContext(12)}`、`write.metadata.holder.{AbstractWriteHolder(31), WriteWorkbookHolder(18)}`
- **涉及文件**：`crates/easyexcel/src/write/metadata/style/`、`crates/easyexcel/src/write/handler/context/`
- **动作步骤**：同 C1，按类批量消歧；context 类的候选通常区分"读 vs 写"或"holder vs value"，按 builder/setter 角色选。
- **验收标准**：上述 4 个类前缀 ambiguous 计数均为 0。
- **估算**：4h
- **依赖**：A1
- **优先级**：P1

#### C3 消歧 read/metadata 与 context 家族
- **涉及 Java**：`read.metadata.holder.ReadWorkbookHolder`（18）、`read.metadata.ReadWorkbook`（12）、`context.{AnalysisContextImpl(10), AnalysisContext(9)}`、`metadata.AbstractHolder`（9）、`metadata.GlobalConfiguration`（9）
- **涉及文件**：`crates/easyexcel/src/read/metadata/`、`crates/easyexcel/src/context/`
- **动作步骤**：同 C1；AnalysisContext 候选常区分"trait 默认 impl vs 具体结构体方法"，按 Java 接口契约选 trait 方法。
- **验收标准**：上述前缀 ambiguous 计数为 0。
- **估算**：3h
- **依赖**：A1
- **优先级**：P1

#### C4 消歧 metadata.csv 家族与 metadata 杂类
- **涉及 Java**：`metadata.csv.{CsvCell(14), CsvWorkbook(11), CsvSheet(9), CsvRow(8), CsvCellStyle(7)}`、`metadata.{CellExtra(9), CellRange(8)}`、`metadata.property.ExcelHeadProperty`（7）、`metadata.data.RichTextStringData$IntervalFont`（6）
- **涉及文件**：`crates/easyexcel-csv/`、`crates/easyexcel/src/metadata/`
- **动作步骤**：同 C1；csv 家族候选常分散在 `easyexcel` 与 `easyexcel-csv` 两个 crate，按"facade 重导出 vs 后端实现"选（默认选后端实现 ID，capability_carriers 记 facade）。
- **验收标准**：上述前缀 ambiguous 计数为 0。
- **估算**：3h
- **依赖**：A1
- **优先级**：P1

#### C5 消歧 util.FileUtils 与剩余 ambiguous（约 130 项）
- **涉及 Java**：`util.FileUtils`（10）及附录未列出的剩余类
- **涉及文件**：`crates/easyexcel-util/`、`crates/easyexcel/src/util/`
- **动作步骤**：
  1. 跑 `python3 -c "import json;from collections import Counter;d=json.load(open('parity/java-rust-public-api.json'));[print(c,n) for c,n in Counter(e['java_id'].split('#')[0].replace('com.alibaba.excel.','') for e in d['entries'] if e['status']=='ambiguous').most_common()]"` 列出全部 ambiguous 类清单。
  2. 按类批量消歧，剩余类多为 < 5 项的小簇。
- **验收标准**：`python3 -c "import json;d=json.load(open('parity/java-rust-public-api.json'));print(sum(1 for e in d['entries'] if e['status']=='ambiguous'))"` 输出 `0`。
- **估算**：6h
- **依赖**：C1-C4
- **优先级**：P1

#### C6 校验：479 ambiguous 清零
- **动作步骤**：重生成 mapping 并验证。
- **验收标准**：
  - `verify_public_api_parity.py` 中 `ambiguous` 相关 error 行为 0。
  - catalog 的 `mapping_resolutions` 数量 ≥ 479 且每条 `rust_ids` 为对应 `candidate_rust_ids` 子集。
- **估算**：1h
- **依赖**：C1-C5
- **优先级**：P0

---

### 阶段 D：9 个 write/style 注解的 verified 证据（子任务 4）

> **重大修正**：输入称"9 个注解缺失"——实际 9 个注解的 **parser 已全部实现**（`crates/easyexcel-derive/src/annotation/write/style/*.rs`），并通过 `#[excel(...)]` 暴露。parity 中这 9 个注解的所有 Java 成员已是 `candidate`（非 unmapped）。本阶段任务因此变为"为已映射注解补齐 verified 证据 + 确认 write/style 桥接 handler 存在"。

#### D1 核对 9 个注解 parser 的字段完整性与 Java 默认值对齐
- **涉及文件**：
  - `crates/easyexcel-derive/src/annotation/write/style/{column_width,head_row_height,content_row_height,head_style,content_style,head_font_style,content_font_style,once_absolute_merge,content_loop_merge}.rs`
  - Java 对照：`/Users/wandl/workspaces/workspace-github/easyexcel/easyexcel-core/src/main/java/com/alibaba/excel/annotation/write/style/*.java`（如 `HeadStyle.java` 含 22 个属性，默认值如 `dataFormat=-1`、`hidden=BooleanEnum.DEFAULT`）
- **当前状态**：parser 已存在；待确认每个 parser 覆盖了 Java 注解的全部属性并匹配默认值。
- **动作步骤**：
  1. 逐个注解打开 Java 源码（如 `HeadStyle.java`）与 Rust parser 对照属性清单。
  2. 缺失属性 → 在 parser 的 `parse_field_*` 中补 meta 解析分支。
  3. 默认值不一致 → 在 `field_options.rs`/`struct_options.rs` 的 `Option<T>` 默认 None 语义上对齐 Java `-1`/`DEFAULT`。
- **验收标准**：
  - `cargo test -p easyexcel-derive` 全部通过。
  - 9 个注解各自 Java 属性数 == Rust parser 支持的 meta key 数（人工核对表见 `docs/migration/语义迁移对照表.md`，若有差异在任务里标注"待确认"）。
- **估算**：4h
- **优先级**：P1
- **依赖**：—

#### D2 确认 write/style 引擎侧桥接 handler 把注解值应用到 CellStyle
- **涉及文件**：
  - engine crate 的 write/style handler（`crates/easyexcel/src/write/style/` 下 `AbstractCellStyleStrategy` 等价物）
  - `crates/easyexcel/src/metadata/property/{StyleProperty,RowHeightProperty,ColumnWidthProperty,LoopMergeProperty,OnceAbsoluteMergeProperty}.rs`
- **当前状态**：`B6` 涉及的 `write.style.*` abstract strategy 项当前 unmapped，提示桥接 handler 可能未完整接通注解 → property → CellStyle 的链路。
- **动作步骤**：
  1. codegraph `query/callees -p <repo> -- AbstractCellStyleStrategy` 定位 Rust handler。
  2. 确认 handler 在 write 流程中读取 derive 展开后的 property 并调用 `ExcelCellStyle` setter（border/fill/font/align，对应已实现的 4 个 POI enum）。
  3. 缺失环节 → 在 engine crate 补 handler（**遵守 facade 边界：handler 走 engine，不走 facade**，见 `xtask/src/facade_boundary/audit.rs`）。
- **验收标准**：
  - `cargo test -p easyexcel-test --test core_annotation_style_handler_1to1_tests` 通过（含注解样式断言）。
  - B6 的 `write.style.*` unmapped 项归零。
- **估算**：6h
- **优先级**：P1
- **依赖**：D1、B6

#### D3 为 9 个注解的 Java 成员补 verified 证据
- **涉及 Java 成员数**：ColumnWidth 2、HeadRowHeight 2、ContentRowHeight 2、HeadStyle 22、ContentStyle 22、HeadFontStyle 10、ContentFontStyle 10、OnceAbsoluteMerge 5、ContentLoopMerge 3（合计 78）
- **涉及文件**：`parity/public-api-evidence/excel-writer.json`（追加 evidence）或新建 `parity/public-api-evidence/style-annotations.json` 并加入 `include`
- **动作步骤**：
  1. 为每个注解的 `value()` 属性方法写 compile_probe（`cargo test -p easyexcel-test --test <style_evidence_test> --no-run`）。
  2. 写 behavior_test：用 `#[excel(head_style(...), content_style(...))]` 标注一个测试 struct，写入后读取 cell 样式断言。
  3. 写 java_golden：Java 4.0.3 用 `@HeadStyle` 生成 fixture，Rust 读回比对。
- **验收标准**：
  - `docs/public-api-parity-report.json` 中 9 个注解类的 Java ID 出现在 `verified_java_api_items`。
  - `scripts/verify-java-parity-gates.sh` gate4 对这 78 项无 `status=unverified` 报错。
- **估算**：10h
- **优先级**：P2
- **依赖**：D2、A3（converter 证据模板复用）

---

### 阶段 E：4 个 POI enum 的 verified 证据（子任务 5）

> **重大修正**：输入称"4 个 POI enum 缺失"——实际 4 个 enum **已全部实现**（`crates/easyexcel/src/enums/poi/*.rs`，含 `ALL` const + `java_name()` + POI 桥接方法），并在 parity 中映射为 candidate/idiomatic_alternative。本阶段仅为"补 verified 证据"。

#### E1 核对 4 个 enum 的枚举值与 Java values() 顺序一致
- **涉及文件**：`crates/easyexcel/src/enums/poi/{border_style,fill_pattern_type,horizontal_alignment,vertical_alignment}_enum.rs`
- **当前状态**：以 `border_style_enum.rs` 为例，`ALL` 已按 Java `values()` 顺序列出 15 个值，`java_name()` 已实现。
- **动作步骤**：
  1. 对照 Java `com.alibaba.excel.enums.poi.*Enum` 的 `values()` 顺序，逐 enum 核对 `ALL` 数组与 `java_name()` 映射。
  2. 缺值/错序 → 修正（**注意：`Default` 变体对应 Java `null`，非 Java enum 常量，已正确处理**）。
- **验收标准**：
  - `cargo test -p easyexcel --lib enums::poi` 通过（若有此类单测）。
  - 4 个 enum 的 `ALL.len()` 等于 Java `values().length`。
- **估算**：2h
- **优先级**：P2
- **依赖**：—

#### E2 为 4 个 enum 补 verified 证据
- **涉及 Java 成员**：每个 enum 的 `values()`、`valueOf(String)`、`DEFAULT` 常量等（约 3-5 项/enum，合计 ~16 项）
- **涉及文件**：`parity/public-api-evidence/excel-writer.json` 或新建 enum 证据文件
- **动作步骤**：
  1. compile_probe：`cargo test -p easyexcel --lib enums::poi --no-run`。
  2. behavior_test：round-trip `java_name()` ↔ `ALL` 遍历，断言每个变体名与 Java 一致。
  3. java_golden：Java 写一个带 `BorderStyleEnum.THIN` 的 cell，Rust 读回 border 样式断言。
- **验收标准**：`BorderStyleEnum`/`FillPatternTypeEnum`/`HorizontalAlignmentEnum`/`VerticalAlignmentEnum` 的 Java ID 进 `verified_java_api_items`。
- **估算**：4h
- **优先级**：P2
- **依赖**：E1、D3（样式证据复用 golden）

---

### 阶段 F：9 个 Java 测试类的真实 Rust 实现（子任务 6）

> **澄清**：`docs/source-test-parity.json` 中这 9 个类标为 `mapped_unverified`，但 `rust_evidence` 指向的是同名共享文件（如 `sort_data_test_to_template_data_test.rs`），且 `grep -rl "mod complex_head_data_test" tests/` 返回 NONE——即**真正按 Java 方法名 1:1 实现的测试 mod 不存在**。`generate_source_test_parity.py:114` 的 `rust_evidence` 靠扫 Rust `#[test]` 函数 doc 注释里的 `JavaClass#method` 引用匹配，故 manifest 的"映射"是 doc 注释级而非行为级。本阶段为每个类新建独立测试文件并加 `/// Java: <FQCN>#<method>` 注解。

#### F1 实现 ComplexHeadDataTest（7 @Test）
- **Java 源**：`.../core/head/ComplexHeadDataTest.java`（方法：`t01ReadAndWrite07`/`t02ReadAndWrite03`/`t03ReadAndWriteCsv`/`t11-t13ReadAndWriteAutomaticMergeHead*`/`t21-t23*`，具体以 Java 为准）
- **涉及文件**：新建 `tests/easyexcel-test/tests/core_simple_sort_head_1to1_tests_cases/complex_head_data_test.rs`（或并入现有 `_cases` 目录的独立文件），并在 `core_simple_sort_head_1to1_tests.rs` 中 `mod complex_head_data_test;`
- **动作步骤**：
  1. 读 Java 测试方法体，准备 Java `ComplexHeadData` model 的 Rust 等价（多级表头 struct + `#[excel_property]`）。
  2. 每个 `@Test` 实现为 `#[test] fn tXX_...`，doc 注释 `/// Java: com.alibaba.easyexcel.test.core.head.ComplexHeadDataTest#t01ReadAndWrite07`。
  3. 断言写入→读回字段一致；含 automatic merge 的方法断言合并单元格。
- **验收标准**：
  - `grep -rl "mod complex_head_data_test" tests/` 命中非空。
  - `cargo test -p easyexcel-test --test core_simple_sort_head_1to1_tests complex_head_data_test` 通过。
  - `python3 scripts/generate_source_test_parity.py --java-root <java> --rust-root . --check` 退出码 0（manifest 新鲜）。
- **估算**：5h
- **优先级**：P1
- **依赖**：—

#### F2 实现 ListHeadDataTest（4 @Test）与 NoHeadDataTest（4 @Test）
- **Java 源**：`.../core/head/ListHeadDataTest.java`、`.../core/head/NoHeadDataTest.java`
- **涉及文件**：`tests/easyexcel-test/tests/core_simple_sort_head_1to1_tests_cases/{list_head_data_test,no_head_data_test}.rs`
- **动作步骤**：同 F1；ListHead 测 `List<List<String>>` 表头模型；NoHead 测无表头读写。
- **验收标准**：同 F1（mod 名分别为 `list_head_data_test`、`no_head_data_test`）。
- **估算**：4h
- **优先级**：P1
- **依赖**：F1

#### F3 实现 MultipleSheetsDataTest（5 @Test）与 RepetitionDataTest（7 @Test）与 UnCamelDataTest（4 @Test）
- **Java 源**：`.../core/multiplesheets/MultipleSheetsDataTest.java`、`.../core/repetition/RepetitionDataTest.java`、`.../core/noncamel/UnCamelDataTest.java`
- **涉及文件**：`tests/easyexcel-test/tests/core_simple_sort_head_1to1_tests_cases/{multiple_sheets_data_test,repetition_data_test,un_camel_data_test}.rs`
- **动作步骤**：同 F1；MultipleSheets 测多 sheet 读写（fixture 复用 `tests/fixtures/xls/multiplesheets.xls`）；Repetition 测重复对象写入；UnCamel 测非驼峰字段名映射。
- **验收标准**：同 F1（3 个 mod 名对应）。
- **估算**：6h
- **优先级**：P1
- **依赖**：F1

#### F4 实现 ExcludeOrIncludeDataTest（19 @Test）
- **Java 源**：`.../core/excludeorinclude/ExcludeOrIncludeDataTest.java`（方法最多：`t01-t03ExcludeIndex*`、`t11-t13ExcludeFieldName*`、`t21-t23IncludeIndex*`、`t31-t33IncludeFieldName*`、`t41-t43IncludeFieldNameOrder*` 各覆盖 07/03/csv 三种格式）
- **涉及文件**：`tests/easyexcel-test/tests/core_annotation_style_handler_1to1_tests_cases/exclude_or_include_data_test.rs`（当前共享文件已存在同名 case，需扩为独立 mod）
- **动作步骤**：同 F1；19 个方法按 exclude/include × index/fieldName × {07,03,csv} 矩阵实现；测 `ExcelWriter::exclude_column_indexes`/`include_column_field_names` 等价 API。
- **验收标准**：`grep -rl "mod exclude_or_include_data_test" tests/` 命中含独立 mod 的文件；19 个 `#[test]` 全过。
- **估算**：8h
- **优先级**：P1
- **依赖**：F1

#### F5 实现 FillStyleDataTest（5 @Test）
- **Java 源**：`.../core/fill/style/FillStyleDataTest.java`（方法含 `t01Fill07`/`t02Fill03`/`t11FillStyleHandler07`/`t12FillStyleHandler03`/...）
- **涉及文件**：`tests/easyexcel-test/tests/core_fill_1to1_tests.rs`（已存在，需确认/补 `fill_style_data_test` mod 与 5 个方法）
- **动作步骤**：同 F1；测模板填充 + 样式 handler 回调断言。
- **验收标准**：`grep -rl "fill_style_data_test" tests/easyexcel-test/tests/core_fill_1to1_tests.rs` 命中；5 个 `#[test]` 全过。
- **估算**：4h
- **优先级**：P1
- **依赖**：F1、D2（fill 样式 handler）

#### F6 实现 AnnotationIndexAndNameDataTest（4 @Test）
- **Java 源**：`.../core/annotation/AnnotationIndexAndNameDataTest.java`（`t01ReadAndWrite07`/`t02ReadAndWrite03`/`t03ReadAndWriteCsv`/...）
- **涉及文件**：`tests/easyexcel-test/tests/core_annotation_style_handler_1to1_tests_cases/annotation_index_and_name_data_test.rs`
- **动作步骤**：同 F1；测 `@ExcelProperty(index=..., value=...)` 注解的组合行为。
- **验收标准**：`grep -rl "annotation_index_and_name_data_test" tests/` 命中独立 mod；4 个 `#[test]` 全过。
- **估算**：3h
- **优先级**：P1
- **依赖**：F1

#### F7 更新 docs/source-test-parity.json 与 Java测试对应关系.md
- **涉及文件**：`docs/source-test-parity.json`（重生成）、`docs/migration/Java测试对应关系.md`（更新 Rust 文件列指向真实独立 mod）
- **动作步骤**：
  1. 跑 `python3 scripts/generate_source_test_parity.py --java-root <java> --rust-root . --output docs/source-test-parity.json` 重生成。
  2. 核对 9 个类的 `rust_evidence` 不再指向共享 `sort_data_test_to_template_data_test.rs`，而是各自的独立 case 文件。
  3. 更新 `Java测试对应关系.md` 表格的 Rust 文件列。
- **验收标准**：
  - `python3 scripts/generate_source_test_parity.py --java-root <java> --rust-root . --check` 退出码 0。
  - `source-test-parity.json` 中 9 个类的 `partial_unverified` 数为 0（即全部 `mapped`，无 `PARITY_PARTIAL` 限制）——若仍有 partial，在任务里标注"待确认：X 方法因 Y 限制暂标 partial"。
- **估算**：2h
- **优先级**：P1
- **依赖**：F1-F6

#### F8 校验：gate2 与 gate3 通过
- **动作步骤**：跑 `scripts/verify-java-parity-gates.sh <java_repo>` 的 gate2（source inventory + `cargo test --no-run`）与 gate3（`java_parity_tests`/`java_full_parity_tests`/`temp_1to1_tests`/`codegraph_phaseE_metadata_1to1_tests`）。
- **验收标准**：
  - gate2 退出码 0（manifest 新鲜 + `cargo test -p easyexcel-test --no-run` 成功）。
  - gate3 中 9 个类对应测试方法全部 PASSED。
- **估算**：1h
- **优先级**：P0
- **依赖**：F7

---

### 阶段 G：全量门禁闭环

#### G1 重生成并提交 parity mapping 快照
- **涉及文件**：`parity/java-rust-public-api.json`、`target/public-api-candidates.json`（gate4 比对源）
- **动作步骤**：按 `parity/README.md:80-100` 的确定性重建命令依次跑 suggest → materialize → apply → run → verify。
- **验收标准**：
  - `cmp -s target/java-rust-public-api.json parity/java-rust-public-api.json` 退出码 0（gate4 的 staleness 检查）。
  - `docs/public-api-parity-report.json` 的 `progress.classified_java_api_items` == 3236、`coded_java_api_items` == 3152（3236 - 84 unmapped 已重分类，若 B 阶段完成则也 == 3236）。
- **估算**：2h
- **优先级**：P0
- **依赖**：B7、C6、D3、E2

#### G2 跑完整 5 道门禁
- **动作步骤**：`scripts/verify-java-parity-gates.sh /Users/wandl/workspaces/workspace-github/easyexcel`（需 Java 17 + Maven + LibreOffice）。
- **验收标准**：
  - 退出码 0。
  - `docs/public-api-parity-report.json` 的 `error_count` == 0（**注：当前验证器对任何非 verified 项报错，故真正"全绿"需 3236 项全 verified；若阶段性目标是"unmapped+ambiguous 清零"则 error 仅来自剩余 candidate 的 `status=unverified`，需在任务里标注当前阶段的可接受 error 上限**）。
- **估算**：2h
- **优先级**：P0
- **依赖**：G1、F8

#### G3 （最终）推进 verified 数到 3236
- **动作步骤**：此为长期目标，依赖 A3/D3/E2 等证据补齐任务的持续推进；每补一个家族的证据，重跑 G1。
- **验收标准**：`docs/public-api-parity-report.json` 的 `progress.verified_java_api_items` == 3236。
- **估算**：持续（每家族 4-12h，~40 家族）
- **优先级**：P2
- **依赖**：G2 + 所有证据任务

---

## 3. 任务依赖关系总览

```
A1 ── A2 ── A3 ──────────────────────────┐
 │                                        │
 ├── B1,B2,B3,B4,B5,B6 ── B7 ────────────┤
 │                                        ├── G1 ── G2 ── G3
 ├── C1,C2,C3,C4 ── C5 ── C6 ────────────┤
 │                                        │
 D1 ── D2 ── D3 ─────────────────────────┤
 E1 ── E2 ───────────────────────────────┤
 F1 ── F2,F3,F4,F5,F6 ── F7 ── F8 ───────┘
```

**关键路径**：A1 → A2 → (B/C 并行) → G1 → G2。其中 A1（catalog schema 升级）是几乎所有任务的强前置。

---

## 4. 工作量与优先级汇总

| 阶段 | 任务 | 估算(h) | 优先级 |
|------|------|--------|--------|
| A | A1-A3 | 18 | P0/P1 |
| B | B1-B7 | 19 | P0/P1 |
| C | C1-C6 | 20 | P0/P1 |
| D | D1-D3 | 20 | P1/P2 |
| E | E1-E2 | 6 | P2 |
| F | F1-F8 | 33 | P0/P1 |
| G | G1-G3 | 持续 | P0/P2 |
| **合计（不含 G3 持续）** | | **116** | |

---

## 附录 1：84 个 unmapped 项按 Java 类全量清单

> 来源：`parity/java-rust-public-api.json`（`status==unmapped`）。`#` 后为方法/字段签名简写。

| Java 类（缩写） | 数量 | 主要成员特征 |
|---|---|---|
| `write.handler.CellWriteHandler` | 8 | before/afterCellCreate + afterCellDataConverted + afterCellDispose，各有 context 与非 context 双 overload |
| `write.handler.RowWriteHandler` | 6 | before/afterRowCreate 双 overload |
| `write.handler.SheetWriteHandler` | 4 | before/afterSheetCreate 双 overload |
| `write.handler.WorkbookWriteHandler` | 6 | before/afterWorkbookCreate/dispose，含无参 `beforeWorkbookCreate()` |
| `context.WriteContext` | 10 | currentSheet/currentTable/finish(Z)/getCurrentSheet/getOutputStream/getWorkbook/needHead/writeSheetHolder/writeTableHolder/writeWorkbookHolder |
| `context.WriteContextImpl` | 1 | `finish(Z)` |
| `metadata.csv.CsvCell` | 3 | getNumberValue/getRow/getSheet（返 POI 型） |
| `metadata.csv.CsvRow` | 2 | getSheet/iterator |
| `metadata.csv.CsvSheet` | 6 | close/flushData/getCsvFormat/getWorkbook/iterator/setCsvFormat |
| `metadata.csv.CsvWorkbook` | 2 | createDataFormat/write |
| `cache.selector.{EternalReadCacheSelector,ReadCacheSelector,SimpleReadCacheSelector}` | 3 | readCache(PackagePart) |
| `event.{AbstractIgnoreExceptionReadListener,AnalysisEventListener,Handler}` | 5 | onException/extra/hasNext/invokeHead/order |
| `exception.ExcelGenerateException` | 3 | 三个构造器 |
| `read.builder.{ExcelReaderBuilder,ExcelReaderSheetBuilder}` | 3 | xlsxSAXParserFactoryName/doRead/doReadSync |
| `read.listener.IgnoreExceptionReadListener`、`read.metadata.ReadSheet` | 2 | onException/toString |
| `metadata.property.{ColumnWidth,LoopMerge,OnceAbsoluteMerge,RowHeight}Property` | 5 | `build(StyleAnnotation)` 静态工厂 |
| `util.{FileUtils,IntUtils,IoUtils,StringUtils}` | 7 | 静态常量字段 EX_CACHE/POI_FILES/MAX_POWER_OF_TWO/EOF/EMPTY/SPACE |
| `util.ClassUtils$FieldCacheKey`、`write.metadata.fill.FillConfig$FillConfigBuilder` | 2 | toString |
| `write.metadata.holder.AbstractWriteHolder` | 2 | own{Sheet,Workbook}HandlerExecutionChain 字段 |
| `write.style.{AbstractCellStyleStrategy,AbstractColumnWidthStyleStrategy,AbstractRowHeightStyleStrategy}` | ~4 | afterCellDispose/order |
| `annotation.format.DateTimeFormat` | 1 | use1904windowing |
| **合计** | **84** | |

---

## 附录 2：479 个 ambiguous 项按 Java 类 Top 20

| Java 类（缩写） | 数量 |
|---|---|
| `metadata.property.StyleProperty` | 44 |
| `write.metadata.style.WriteCellStyle` | 44 |
| `write.metadata.holder.AbstractWriteHolder` | 31 |
| `write.handler.context.CellWriteHandlerContext` | 24 |
| `read.metadata.holder.ReadWorkbookHolder` | 18 |
| `write.metadata.holder.WriteWorkbookHolder` | 18 |
| `metadata.property.FontProperty` | 17 |
| `metadata.csv.CsvCell` | 14 |
| `read.metadata.ReadWorkbook` | 12 |
| `write.handler.context.RowWriteHandlerContext` | 12 |
| `metadata.csv.CsvWorkbook` | 11 |
| `write.metadata.WriteWorkbook` | 11 |
| `context.AnalysisContextImpl` | 10 |
| `util.FileUtils` | 10 |
| `context.AnalysisContext` | 9 |
| `metadata.AbstractHolder` | 9 |
| `metadata.CellExtra` | 9 |
| `metadata.GlobalConfiguration` | 9 |
| `metadata.csv.CsvSheet` | 9 |
| `metadata.CellRange` | 8 |

完整清单获取命令：
```bash
python3 -c "import json;from collections import Counter;d=json.load(open('parity/java-rust-public-api.json'));[print(c,n) for c,n in Counter(e['java_id'].split('#')[0].replace('com.alibaba.excel.','') for e in d['entries'] if e['status']=='ambiguous').most_common()]"
```

---

## 附录 3：9 个 Java 测试类与 @Test 方法数（权威值，源自 Java 4.0.3）

| Java 测试类 | @Test 数 | Java 路径（相对 java repo） | 当前 Rust mod 状态 |
|---|---|---|---|
| `ExcludeOrIncludeDataTest` | 19 | `easyexcel-test/src/test/java/.../core/excludeorinclude/ExcludeOrIncludeDataTest.java` | 共享 case 文件，需独立 mod |
| `ComplexHeadDataTest` | 7 | `.../core/head/ComplexHeadDataTest.java` | mod 不存在 |
| `RepetitionDataTest` | 7 | `.../core/repetition/RepetitionDataTest.java` | mod 不存在 |
| `MultipleSheetsDataTest` | 5 | `.../core/multiplesheets/MultipleSheetsDataTest.java` | mod 不存在 |
| `FillStyleDataTest` | 5 | `.../core/fill/style/FillStyleDataTest.java` | core_fill_1to1_tests.rs 有 mod，待补全 |
| `AnnotationIndexAndNameDataTest` | 4 | `.../core/annotation/AnnotationIndexAndNameDataTest.java` | mod 不存在 |
| `UnCamelDataTest` | 4 | `.../core/noncamel/UnCamelDataTest.java` | mod 不存在 |
| `ListHeadDataTest` | 4 | `.../core/head/ListHeadDataTest.java` | mod 不存在 |
| `NoHeadDataTest` | 4 | `.../core/head/NoHeadDataTest.java` | mod 不存在 |
| **合计** | **59** | | |

> 注：输入事实称"60 测试方法"，权威 `grep -c "@Test"` 合计为 59（ExcludeOrIncludeDataTest 实测 19 非 19+，差异 1 为输入取整误差）。

---

## 附录 4：关键文件路径索引（绝对路径）

- parity mapping：`/Users/wandl/workspaces/workspace-github-easy-4-rust/easyexcel-rust/parity/java-rust-public-api.json`
- evidence catalog 顶层：`/Users/wandl/workspaces/workspace-github-easy-4-rust/easyexcel-rust/parity/public-api-evidence.json`
- evidence 子目录：`/Users/wandl/workspaces/workspace-github-easy-4-rust/easyexcel-rust/parity/public-api-evidence/`（含 `converters.json`、`excel-analyser.json`、`excel-builder.json`、`excel-writer.json`）
- 主验证器：`/Users/wandl/workspaces/workspace-github-easy-4-rust/easyexcel-rust/scripts/verify_public_api_parity.py`
- 5 道门禁：`/Users/wandl/workspaces/workspace-github-easy-4-rust/easyexcel-rust/scripts/verify-java-parity-gates.sh`
- 候选器：`/Users/wandl/workspaces/workspace-github-easy-4-rust/easyexcel-rust/scripts/suggest_public_api_mapping.py`
- 物化器：`/Users/wandl/workspaces/workspace-github-easy-4-rust/easyexcel-rust/scripts/materialize_public_api_evidence.py`
- overlay 应用：`/Users/wandl/workspaces/workspace-github-easy-4-rust/easyexcel-rust/scripts/apply_public_api_evidence.py`
- 证据执行：`/Users/wandl/workspaces/workspace-github-easy-4-rust/easyexcel-rust/scripts/run_public_api_evidence.py`
- 测试静态映射：`/Users/wandl/workspaces/workspace-github-easy-4-rust/easyexcel-rust/scripts/generate_source_test_parity.py` + `docs/source-test-parity.json`
- derive 注解入口：`/Users/wandl/workspaces/workspace-github-easy-4-rust/easyexcel-rust/crates/easyexcel-derive/src/annotation/write/style/`
- POI enum：`/Users/wandl/workspaces/workspace-github-easy-4-rust/easyexcel-rust/crates/easyexcel/src/enums/poi/`
- facade 边界审计：`/Users/wandl/workspaces/workspace-github-easy-4-rust/easyexcel-rust/xtask/src/facade_boundary/audit.rs`
- Java 源根：`/Users/wandl/workspaces/workspace-github/easyexcel`
- Java 测试根：`/Users/wandl/workspaces/workspace-github/easyexcel/easyexcel-test/src/test/java`

---

## 附录 5：待确认事项

1. **物化器输出 schema_version**（A1）：`materialize_public_api_evidence.py:288` 写死 `schema_version:1`，需确认是否要先改物化器才能让 catalog 升 v2，或 catalog 的 v2 标记仅在检入文件层。
2. **mapping_resolutions 落盘位置**（C 阶段）：overlay 从 catalog 树读 resolutions，需确认是写在 `public-api-evidence.json` 顶层还是某个子文件，以及是否需要检入（gate4 会 `cmp` 整个 mapping）。
3. **gate4 的 catalog 比对范围**（G1）：gate4 仅 `cmp target/java-rust-public-api.json parity/java-rust-public-api.json`，但 catalog（`target/public-api-evidence-catalog.json`）未检入——需确认 resolutions 是否通过检入 catalog 持久化，还是每次从检入 evidence 模板重生成。
4. **core_fill_1to1_tests.rs 现有 fill_style_data_test mod 的完整度**（F5）：文件存在（24363 字节），需打开确认是否已含 5 个方法或仅部分。
5. **facade 边界对新 style handler 的约束**（D2）：`xtask/src/facade_boundary/audit.rs`（2998 行）强制 engine 依赖位置，新增 handler 前需读该文件确认允许的模块路径。
