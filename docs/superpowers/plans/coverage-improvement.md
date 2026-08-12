# 测试覆盖率盲区闭环任务清单（COVERAGE-GAP-CLOSURE）

> 工作流：补测试盲区（Coverage Gap Closure）
> 仓库：`easyexcel-rust`
> 文档版本：v1（2026-08-10）
> 约束：本文档仅产出可执行任务清单（WBS），不包含代码改动。每项任务的验收标准均可机器校验。

---

## 0. 现状基线（事实依据）

| 维度 | 现状 | 证据（文件:行号） |
|---|---|---|
| 总测试规模 | 约 2797 个 `#[test]`（crate 内 2117 + 集成 193 + 子用例 183 + parity 240 + round_trip 17 + cross_validation 40 + web conformance 7） | 任务输入事实 #1 |
| `ExcelRows<T>` 直接单测 | **0** 个；仅 7 个框架 conformance 间接覆盖（actix/axum/hyper/poem/rocket/salvo/warp 各 1，每框架仅 CSV 上传 + XLSX 下载一个 happy path） | `crates/easyexcel-web/src/web/excel_rows.rs:20`；`tests/easyexcel-web-conformance/tests/*.rs`（7 个文件各 1 个 `#[*_test]`） |
| `ExcelWriteFillExecutor` 直接单测 | **2** 个内联 `#[test]`（基础构造 + 缺失 delegate 错误） | `crates/easyexcel/src/write/executor/excel_write_fill_executor.rs:134`、`:164`（`mod tests` 在 `:101`） |
| Web conformance 覆盖矩阵 | 每框架仅 1 个测试（CSV 上传 + XLSX 下载）；缺 XLS 上传、多 sheet、超大文件、损坏文件、取消传播 | `tests/easyexcel-web-conformance/src/lib.rs:42-60`（`upload_fixture`/`download_rows` 仅 CSV+XLSX）；7 个 `tests/*.rs` 各 1 测试 |
| parity evidence 数 | 根 catalog 6 条（facade/reader 各 3），4 子 catalog 各 3 条（analyser 3、builder 3、writer 3、converters **0**，converters 用未 materialize 的 `family_evidence`）；契约点 reader 11 字段 + writer 15 字段 = 26 | `parity/public-api-evidence.json:9`（根 evidence 6）；`parity/public-api-evidence/{analyser,builder,writer}.json` 各 3；`parity/public-api-evidence/converters.json` 仅 `family_evidence`（3 条，`java_ids` 全空）；`tests/easyexcel-test/tests/public_api_excel_reader_evidence_tests.rs:12-24`、`public_api_excel_writer_evidence_tests.rs:17-34` |
| coverage 门禁 | `scripts/coverage.sh` 用 `cargo llvm-cov --workspace --all-features`，门禁 `--fail-under-lines/regions/functions 95`；排除 `easyexcel-derive/src/lib.rs`、`locale_generated.rs`；CI 仅 release/手动触发，无产物归档 | `scripts/coverage.sh:6-27`；`.github/workflows/ci.yml:49-67`（`coverage` job，`if: github.event_name == 'release' || github.event_name == 'workflow_dispatch'`，无 `actions/upload-artifact`） |
| examples README | `examples/README.md` 仅 1 个顶层文件，10 个示例子目录（read/write/fill/axum/actix/hyper/poem/rocket/salvo/warp）均无各自 README | `examples/README.md`（25 行，框架表）；`examples/*/` 下仅 `Cargo.toml`+`src/main.rs`（read/write/fill 另有 `tests/spawn_binary.rs`） |

**覆盖率目标总览**：当前门禁 95%（容差），最终目标 = 每行可达代码均被覆盖（由 parity evidence 6 承载的权威声明）。本计划不追求字面 100%。

---

## 1. 子任务一：ExcelRows<T> 单测矩阵

### 现状证据
- 文件：`crates/easyexcel-web/src/web/excel_rows.rs`（218 行）
- 关键分支（需逐个覆盖）：
  - `spawn`（:29）：`acquire_worker_permit` 成功路径（:39）与失败路径（:41-44，`send_terminal_async`）
  - `worker` 内 `EasyExcel::read().do_read()` 成功 + 取消早退（:60-67）
  - `worker` 内解析错误：`error_is_execution_stop` 命中（:69-71，静默返回）与未命中（:72-77，`send_terminal` 转换错误）
  - `tokio::time::timeout`（:80）：正常完成（:81）、worker JoinError（:82-91，`ExcelWebError::Worker`）、超时（:92-100，`cancel` + `processing_timeout()`）
  - `next_row`（:110）：`recv().await` 返回 `Some(Ok)`、`Some(Err)`、`None`（EOF）
  - `cancel`（:115）+ `Drop`（:128-131，调用 `cancellation.cancel()`）
  - `ChannelReadListener::invoke`（:164）：已取消（:165-172）、`max_rows` 超限（:174-182，`RowLimitExceeded`）、`blocking_send` 失败（:184-187，消费者掉线 → `context.cancel()`）、正常（:183-188）
  - `has_next`（:191）：`!is_cancelled`
- 现有测试数：`crates/easyexcel-web/src/web/excel_rows.rs` 内 `#[cfg(test)]` 模块 = **0**；`crates/easyexcel-web/tests/web_contract.rs`（7 个 `#[tokio::test]`）间接覆盖部分分支（CSV 上传、upload 限制、row 限制、取消前解析、XLSX 导出、并发 permits、problem details）。

---

#### 任务 1.1 — ExcelRows 正常 EOF 与背压通道语义
- **编号**：T1.1
- **优先级**：P0
- **依赖**：无
- **涉及文件**：`crates/easyexcel-web/tests/excel_rows_unit.rs`（新增）
- **当前状态**：`excel_rows.rs:110` `next_row` 无直接单测；EOF 仅在 `web_contract.rs:115` 间接断言。
- **具体动作**：
  1. 新建 `crates/easyexcel-web/tests/excel_rows_unit.rs`。
  2. 加测试函数 `fn excel_rows_normal_parse_yields_all_rows_then_eof()`：
     - 用 `ExcelImport::<WebRow>::from_bytes(b"Name,Value\na,1\nb,2\n", "csv", ...)` 构造 import（复用 `web_contract.rs:16-22` 的 `WebRow` 定义模式）。
     - `policy = ExcelWebPolicy::new(ResourceLimits::default()).with_row_channel_capacity(1)` 触发背压。
     - 调 `import.rows()` 得 `ExcelRows`，循环 `next_row().await` 收集直到 `None`。
     - 断言：收到 2 行 `[(a,1),(b,2)]`，最后 `next_row()` 返回 `None`。
- **验收标准**：
  - `cargo test -p easyexcel-web --test excel_rows_unit excel_rows_normal_parse_yields_all_rows_then_eof` 通过。
  - 新增 `#[tokio::test]` ≥ 1。
- **工作量**：0.5h

#### 任务 1.2 — Drop 取消后台解析
- **编号**：T1.2
- **优先级**：P0
- **依赖**：无
- **涉及文件**：`crates/easyexcel-web/tests/excel_rows_unit.rs`
- **当前状态**：`excel_rows.rs:128-131` `Drop::drop` 调 `cancellation.cancel()`，无直接单测证明后台 task 被取消（`web_contract.rs` 仅证明上传前取消报错）。
- **具体动作**：
  1. 加 `fn excel_rows_drop_cancels_background_task()`：
     - 构造含 ≥100 行的 CSV import，`processing_timeout` 设大（5s）。
     - `let mut rows = import.rows();` 取 1 行后立即 `drop(rows);`
     - 用 `Arc<Barrier>` + 自定义 `ReadListener` 计数（或检查 tempdir 在 drop 后被清理）断言后台 spawn_blocking 在取消信号后退出。
     - 断言：`directory_entry_count(tempdir) == 0`（参考 `web_contract.rs:78-82`、`:117`）。
- **验收标准**：
  - 测试通过；`cargo test -p easyexcel-web --test excel_rows_unit excel_rows_drop_cancels_background_task` 退出码 0。
- **工作量**：1h

#### 任务 1.3 — 主动 cancel() 终止流
- **编号**：T1.3
- **优先级**：P1
- **依赖**：无
- **涉及文件**：`crates/easyexcel-web/tests/excel_rows_unit.rs`
- **当前状态**：`excel_rows.rs:115` `ExcelRows::cancel` 公开方法无直接调用测试。
- **具体动作**：
  1. 加 `fn excel_rows_explicit_cancel_terminates_stream()`：
     - 构造 import，`rows()` 后调 `rows.cancel();`，再 `next_row().await`。
     - 断言：返回 `Some(Err(ExcelWebError::Cancelled))`（`code() == ExcelWebErrorCode::Cancelled`，对齐 `web_contract.rs:188`）。
- **验收标准**：测试通过；错误码断言成立。
- **工作量**：0.5h

#### 任务 1.4 — processing_timeout 触发
- **编号**：T1.4
- **优先级**：P0
- **依赖**：无
- **涉及文件**：`crates/easyexcel-web/tests/excel_rows_unit.rs`
- **当前状态**：`excel_rows.rs:92-100` 超时分支（`ExcelWebError::processing_timeout()`）无任何测试。
- **具体动作**：
  1. 加 `fn excel_rows_processing_timeout_emits_timeout_error()`：
     - 构造一个解析会阻塞的 import（用 gated CSV 或在 `max_rows` 很大的慢解析场景；参考 `web_contract.rs:24-47` `GatedRows` 思路，但此处需 gated **reader**——可用一个无法快速解析的大 CSV + `with_processing_timeout(Duration::from_millis(50))`）。
     - `rows()` 后 `next_row().await`。
     - 断言：收到 `Err`，`code() == ExcelWebErrorCode::ProcessingTimeout`。
  - 注意：若构造稳定超时夹具困难，标记「待确认夹具策略」并降级为用 `tokio::time::pause` 注入。
- **验收标准**：测试通过；超时分支 `excel_rows.rs:92-100` 被执行（llvm-cov 命中）。
- **工作量**：1.5h

#### 任务 1.5 — RowLimitExceeded 通过流传播
- **编号**：T1.5
- **优先级**：P1
- **依赖**：无
- **涉及文件**：`crates/easyexcel-web/tests/excel_rows_unit.rs`
- **当前状态**：`excel_rows.rs:174-182` `RowLimitExceeded` 仅在 `web_contract.rs:139-163` 间接覆盖（`row_limit_is_reported_through_bounded_row_stream`）；无 `ExcelRows` 模块归属的直接单测。
- **具体动作**：
  1. 加 `fn excel_rows_row_limit_propagates_through_stream()`：
     - `ResourceLimits::new(1024, 8, 1, 8)`（`max_rows=1`）+ 2 行 CSV。
     - `rows()` 后取 2 次 `next_row`。
     - 断言：第 1 行 Ok，第 2 行 `Err`，`code() == RowLimitExceeded`。
- **验收标准**：测试通过。
- **工作量**：0.5h

#### 任务 1.6 — 背压（消费者慢于生产者）通道不丢行
- **编号**：T1.6
- **优先级**：P1
- **依赖**：T1.1
- **涉及文件**：`crates/easyexcel-web/tests/excel_rows_unit.rs`
- **当前状态**：`excel_rows.rs:34` `mpsc::channel(capacity)` + `:184` `blocking_send` 背压路径无直接断言（生产者必须阻塞等待消费者）。
- **具体动作**：
  1. 加 `fn excel_rows_backpressure_does_not_drop_rows()`：
     - `with_row_channel_capacity(1)` + 5 行 CSV。
     - 在每次 `next_row().await` 之间 `tokio::time::sleep(10ms)` 模拟慢消费者。
     - 断言：5 行全部按序收到，无丢失/重复，EOF 正常。
- **验收标准**：测试通过；断言行数 == 输入行数。
- **工作量**：1h

#### 任务 1.7 — 解析错误（非 execution_stop）转换为 ExcelWebError
- **编号**：T1.7
- **优先级**：P1
- **依赖**：无
- **涉及文件**：`crates/easyexcel-web/tests/excel_rows_unit.rs`
- **当前状态**：`excel_rows.rs:68-77`（`error_is_execution_stop` 未命中 → `send_terminal(ExcelWebError::from(error))`）与 `:69-71`（命中 → 静默返回）两条分支无直接单测。
- **具体动作**：
  1. 加 `fn excel_rows_parse_error_surfaces_as_excel_web_error()`：
     - 构造损坏 CSV/XLSX（如 `b"Name,Value\na,notanumber\n"`，但需用能触发解析错误的格式；或直接喂损坏 xlsx 字节）。
     - `rows()` 后 `next_row`。
     - 断言：返回 `Err`，且 `code()` ∈ {`InvalidFormat`, `RowConversionFailed`}（依夹具）。
- **验收标准**：测试通过；分支 `:72-77` 被覆盖。
- **工作量**：1h

#### 任务 1.8 — ExcelRows 实现 Stream trait
- **编号**：T1.8
- **优先级**：P2
- **依赖**：无
- **涉及文件**：`crates/easyexcel-web/tests/excel_rows_unit.rs`
- **当前状态**：`excel_rows.rs:120-126` `impl Stream for ExcelRows`（`poll_next` 委托 `receiver.poll_recv`）无测试。
- **具体动作**：
  1. 加 `fn excel_rows_implements_stream_trait()`：
     - 构造 import，`rows()` 后用 `futures::StreamExt::next`（或 `try_collect`）消费。
     - 断言：与 `next_row` 循环结果一致。
- **验收标准**：测试通过；`Stream::poll_next` 被调用。
- **工作量**：0.5h

**子任务一汇总**：8 个测试，目标 `excel_rows.rs` 行覆盖 ≥ 90%（排除数学不可达分支）。

---

## 2. 子任务二：ExcelWriteFillExecutor 聚焦单测

### 现状证据
- 文件：`crates/easyexcel/src/write/executor/excel_write_fill_executor.rs`（174 行）
- 现有内联测试：2 个（`executor_delegates_fill_state_and_finish_to_real_engine` :130，`executor_without_engine_returns_visible_error` :162）
- 需补充分支：
  - `with_delegate` 构造（:41）vs `new` 构造（:28）
  - `fill` 多次调用状态累积
  - `finish(on_exception=true)` 与 `finish(on_exception=false)` 两条
  - `fill` 错误透传（delegate 返回 Err）
  - `finish` 错误透传
  - `write_context()` 访问器
  - `missing_delegate_error` 对 `finish` 的覆盖（:88-92 仅 `fill` 被 测，`finish` 的 `ok_or_else` 未测）

---

#### 任务 2.1 — fill 多次调用累积到 delegate
- **编号**：T2.1
- **优先级**：P0
- **依赖**：无
- **涉及文件**：`crates/easyexcel/src/write/executor/excel_write_fill_executor.rs`（`mod tests`）
- **当前状态**：现有测试（:130）仅调 1 次 `fill` + 1 次 `finish`；未验证多次 fill 状态。
- **具体动作**：在 `mod tests` 内加 `fn executor_multiple_fills_accumulate_in_delegate()`：
  - 复用 `ProbeFillExecutor`（:107）。
  - 连续 3 次 `executor.fill(...)`（不同 `WriteFillConfig` / `WriteFillSheet`）。
  - 断言：`probe.fills.len() == 3`，各次参数按序匹配。
- **验收标准**：测试通过。
- **工作量**：0.5h

#### 任务 2.2 — finish(false) 正常完成路径
- **编号**：T2.2
- **优先级**：P0
- **依赖**：无
- **涉及文件**：同上
- **当前状态**：现有测试（:153）仅 `finish(true)`；`finish(false)` 未覆盖。
- **具体动作**：加 `fn executor_finish_false_propagates_to_delegate()`：
  - 调 `executor.finish(false)`。
  - 断言：`probe.finished == vec![false]`。
- **验收标准**：测试通过。
- **工作量**：0.3h

#### 任务 2.3 — fill 错误透传
- **编号**：T2.3
- **优先级**：P0
- **依赖**：无
- **涉及文件**：同上
- **当前状态**：`fill`（:68-78）的 `delegate.fill(...)` 返回 Err 路径未测（现有 ProbeFillExecutor 永远 Ok）。
- **具体动作**：加一个 `FailingFillExecutor`（`fill` 返回 `ExcelError::Unsupported`），测试 `fn executor_fill_propagates_delegate_error()`：
  - 断言 `executor.fill(...)` 返回 `Err`，错误为 `Unsupported`。
- **验收标准**：测试通过。
- **工作量**：0.5h

#### 任务 2.4 — finish 错误透传
- **编号**：T2.4
- **优先级**：P0
- **依赖**：无
- **涉及文件**：同上
- **当前状态**：`finish`（:87-92）的 `delegate.finish(...)` Err 路径未测。
- **具体动作**：加 `FailingFinishExecutor`，测试 `fn executor_finish_propagates_delegate_error()`：
  - 断言 `executor.finish(false)` 返回 `Err`。
- **验收标准**：测试通过。
- **工作量**：0.5h

#### 任务 2.5 — finish 缺 delegate 报错
- **编号**：T2.5
- **优先级**：P1
- **依赖**：无
- **涉及文件**：同上
- **当前状态**：`finish`（:88-92）的 `ok_or_else(Self::missing_delegate_error)` 未测（仅 `fill` 的缺失 delegate 被测，:162）。
- **具体动作**：加 `fn executor_finish_without_engine_returns_visible_error()`：
  - `ExcelWriteFillExecutor::new(&context)`（无 delegate）后调 `finish(false)`。
  - 断言：`Err`，错误信息含 "not wired"（对齐 :172 现有断言风格）。
- **验收标准**：测试通过。
- **工作量**：0.3h

#### 任务 2.6 — write_context() 访问器独立断言
- **编号**：T2.6
- **优先级**：P2
- **依赖**：无
- **涉及文件**：同上
- **当前状态**：`write_context()`（:54）在 :136 间接断言 path，无独立测试覆盖 `with_delegate` 路径的 context 共享。
- **具体动作**：加 `fn executor_write_context_accessor_returns_same_reference()`：
  - 构造 context，executor，断言 `executor.write_context()` 返回的 `current_write_holder().path()` 与构造时一致；多次调用返回一致。
- **验收标准**：测试通过。
- **工作量**：0.3h

#### 任务 2.7 — FillConfig 各字段组合
- **编号**：T2.7
- **优先级**：P1
- **依赖**：T2.1
- **涉及文件**：同上
- **当前状态**：现有测试仅覆盖 `force_new_row=true, direction=Horizontal, auto_style=false` 一组（:142-146）；`WriteFillConfig` 其他组合（Vertical/默认 `force_new_row=false`/`auto_style=true`）未测。
- **具体动作**：加 `fn executor_fill_config_variants_pass_through()`：
  - 用 `WriteFillConfig::default()`（全默认）+ 一组 `{force_new_row:false, direction:Some(Vertical), auto_style:true}`。
  - 断言 ProbeFillExecutor 收到的 config 字段逐项匹配。
- **验收标准**：测试通过。
- **工作量**：0.5h

#### 任务 2.8 — WriteFillSheet 默认与具名
- **编号**：T2.8
- **优先级**：P2
- **依赖**：T2.1
- **涉及文件**：同上
- **当前状态**：现有测试仅 `sheet_name:"Data", sheet_index:Some(2)`（:148-151）；`WriteFillSheet::default()`（:169 用过但未断言）未验证透传。
- **具体动作**：加 `fn executor_fill_sheet_default_and_named_pass_through()`：
  - 分别用 `WriteFillSheet::default()` 与具名 sheet 调 `fill`，断言 ProbeFillExecutor 收到的 sheet 字段。
- **验收标准**：测试通过。
- **工作量**：0.3h

**子任务二汇总**：8 个测试，目标 `excel_write_fill_executor.rs` 行覆盖 ≥ 90%。注：BIFF8 模板 session、混合 fill+write 生命周期、supplier 模式等更复杂场景已在集成测试 `crates/easyexcel/tests/core_fill_1to1_tests.rs`（21）、`temp_fill_contract_tests.rs`（8）、`template/tests_cases/` 覆盖，本子任务聚焦 executor 单元粒度。

---

## 3. 子任务三：Web conformance 扩格式与错误路径

### 现状证据
- 文件：`tests/easyexcel-web-conformance/src/lib.rs`（94 行）+ `tests/{actix,axum,hyper,poem,rocket,salvo,warp}.rs` 各 1 测试。
- 现有夹具：`upload_fixture()`（:43，仅 CSV）、`download_rows()`（:49）、`verify_upload`（:71，断言行相等）、`verify_download`（:85，断言 200 + OOXML Content-Type + `PK` 魔数）。
- 缺口：XLS 上传、XLSX 多 sheet 上传、超大文件（触发临时文件）、损坏文件错误响应、取消传播。

---

#### 任务 3.1 — 共享夹具扩充（XLS / 多 sheet / 超大 / 损坏）
- **编号**：T3.1
- **优先级**：P0
- **依赖**：无
- **涉及文件**：`tests/easyexcel-web-conformance/src/lib.rs`
- **当前状态**：`upload_fixture()` 仅返回固定 CSV（:43-45）。
- **具体动作**：在 `lib.rs` 增加：
  - `pub fn xls_upload_fixture() -> Bytes`：返回内嵌 XLS 字节（`include_bytes!("fixtures/conformance.xls")`，需新增夹具文件，来源参考 `tests/easyexcel-test/tests/fixtures/xls/`）。
  - `pub fn xlsx_multisheet_fixture() -> Bytes`：返回 2-sheet XLSX。
  - `pub fn oversized_fixture(policy: &ExcelWebPolicy) -> Bytes`：构造超过 `max_file_bytes` 的字节。
  - `pub fn corrupted_xlsx_fixture() -> Bytes`：返回 `b"PK\x03\x04CORRUPT"` 这类带魔数但内容损坏的字节。
  - `pub async fn verify_upload_xls(...)`、`pub fn verify_error_response(snapshot: &ResponseSnapshot, expected_code: &str)` 等辅助。
- **验收标准**：`cargo test -p easyexcel-web-conformance --no-run` 编译通过；新夹具函数可被各框架测试引用。
- **工作量**：1.5h

#### 任务 3.2 — 每框架新增 XLS 上传 conformance
- **编号**：T3.2
- **优先级**：P1
- **依赖**：T3.1
- **涉及文件**：`tests/easyexcel-web-conformance/tests/{actix,axum,hyper,poem,rocket,salvo,warp}.rs`
- **当前状态**：7 个框架文件各 1 测试（仅 CSV）。
- **具体动作**：每个框架文件加 `fn <framework>_xls_upload_conforms()`：
  - 用 `xls_upload_fixture()`，`content-type: application/vnd.ms-excel`，`x-excel-file-name: fixture.xls`。
  - 调 `verify_upload_xls`。
- **验收标准**：每框架 +1 测试（共 +7）；`cargo test -p easyexcel-web-conformance` 通过。
- **工作量**：2h（7 框架 × ~15min）

#### 任务 3.3 — 每框架新增 XLSX 多 sheet 上传 conformance
- **编号**：T3.3
- **优先级**：P2
- **依赖**：T3.1
- **涉及文件**：同 T3.2
- **具体动作**：每框架加 `fn <framework>_xlsx_multisheet_upload_conforms()`，用 `xlsx_multisheet_fixture()`，验证两个 sheet 行均被解析。
- **验收标准**：每框架 +1 测试（共 +7）；通过。
- **工作量**：2h

#### 任务 3.4 — 每框架新增超大文件错误响应 conformance
- **编号**：T3.4
- **优先级**：P1
- **依赖**：T3.1
- **涉及文件**：同 T3.2
- **当前状态**：`FileTooLarge` 仅在 `web_contract.rs:121` 测过（框架中立层），各框架 extractor 是否返回正确状态码未验证。
- **具体动作**：每框架加 `fn <framework>_oversized_upload_returns_file_too_large()`，用 `oversized_fixture`，断言响应 `status` ∈ {400,413} 且 `verify_error_response(snapshot, "FILE_TOO_LARGE")`。
- **验收标准**：每框架 +1（共 +7）；错误码断言成立。
- **工作量**：2h

#### 任务 3.5 — 每框架新增损坏文件错误响应 conformance
- **编号**：T3.5
- **优先级**：P1
- **依赖**：T3.1
- **涉及文件**：同 T3.2
- **具体动作**：每框架加 `fn <framework>_corrupted_upload_returns_invalid_format()`，用 `corrupted_xlsx_fixture()`，断言 `verify_error_response(snapshot, "INVALID_FORMAT")` 或 `"ROW_CONVERSION_FAILED"`。
- **验收标准**：每框架 +1（共 +7）。
- **工作量**：2h

#### 任务 3.6 — 每框架新增客户端断连取消传播 conformance
- **编号**：T3.6
- **优先级**：P2
- **依赖**：T3.1
- **涉及文件**：同 T3.2
- **当前状态**：取消传播仅在 `web_contract.rs:166`（取消后再 rows）测过；框架层 extractor 在接收中途断开的语义未验证。
- **具体动作**：每框架加 `fn <framework>_client_disconnect_propagates_cancellation()`：用一个能被 `CancellationToken` 控制的 gated 上传流，中途 drop request，断言后台解析被取消（tempdir 清理）。
  - 待确认：各框架的 test harness 是否支持中途 drop body stream；若不支持，标记「待确认夹具策略」。
- **验收标准**：每框架 +1（共 +7）；或对不支持的框架文档说明并跳过。
- **工作量**：3h

**子任务三汇总**：共享夹具 +1 文件改动，每框架 +5 测试（共 +35），目标 conformance 测试数 7 → 42。

---

## 4. 子任务四：parity 证据扩充

### 现状证据
- 根 catalog `parity/public-api-evidence.json`：6 条 evidence（facade compile/behavior/java-golden + reader compile/behavior/java-golden）。
- 子 catalog（`parity/public-api-evidence/`）：
  - `excel-writer.json`：3 条（compile/behavior/java-golden）
  - `excel-builder.json`：3 条
  - `excel-analyser.json`：3 条
  - `converters.json`：**0 条** evidence，仅 `family_evidence`（3 条，`java_ids` 全空）
- 契约点（已在 evidence 测试中 assert 的字段）：
  - reader：11 字段（`public_api_excel_reader_evidence_tests.rs:12-24`）
  - writer：15 字段（`public_api_excel_writer_evidence_tests.rs:17-34`）
  - 合计 26 个已 materialize 契约字段。
- 所有 evidence 的 `compile_probes`/`behavior_tests`/`java_golden` 子数组：当前根 catalog 用 `commands` 字段（非空），但子 catalog 的 per-entry 细粒度数组为空（任务输入事实 #5）。
- 对应测试文件：`public_api_{facade,excel_reader,excel_writer,excel_builder,excel_analyser,converter}_evidence_tests.rs`（6 个），测试数分别 3/2/3/4/2/3。
- golden 契约 JSON：`excel_reader_lifecycle.contract.json`（11 字段）、`excel_writer_lifecycle.contract.json`（15 字段）、`excel_builder_lifecycle.contract.json`（13 字段）、`excel_analyser_lifecycle.contract.json`（109 字段）、`facade_api.contract.json`（5 字段）。`converter_api.contract.json` **不存在**（待 `export-java-golden.sh` 生成）。

---

#### 任务 4.1 — converters 子 catalog：family_evidence → evidence materialize
- **编号**：T4.1
- **优先级**：P0
- **依赖**：无（但需先有 `converter_api.contract.json` golden，标记「待确认：golden 生成」）
- **涉及文件**：`parity/public-api-evidence/converters.json`、`tests/easyexcel-test/tests/public_api_converter_evidence_tests.rs`、`tests/easyexcel-test/tests/golden/converter_api.contract.json`（待生成）
- **当前状态**：`converters.json` 用 `family_evidence`（3 条，`java_ids` 全空），未 materialize；测试文件已引用 `converter_api.contract.json`（`public_api_converter_evidence_tests.rs:39-46`）但文件不存在。
- **具体动作**：
  1. 运行 `./scripts/export-java-golden.sh` 生成 `converter_api.contract.json`（待确认：是否已能生成）。
  2. 将 `converters.json` 的 `family_evidence` 拆解为按具体 converter 类（如 `BigDecimalConverter`、`UrlConverter`、`BooleanConverter` 等）的 `evidence` 数组，每条含非空 `java_ids` + `rust_ids` + `commands`。
  3. 在 `public_api_converter_evidence_tests.rs` 补 compile_probe（每 converter 实例化 + trait 方法签名断言）与 behavior_test（注册表 `registered_converter_count` 断言）。
- **验收标准**：
  - `python3 scripts/run_public_api_evidence.py --catalog parity/public-api-evidence.json --output <tmp> --repo-root $(pwd)` 全部 evidence `commands` exit code 0。
  - `converters.json` 的 `evidence` 数组 ≥ 3（compile/behavior/java-golden 各 ≥1），`java_ids` 非空。
  - `cargo test -p easyexcel-test --test public_api_converter_evidence_tests` 通过。
- **工作量**：3h

#### 任务 4.2 — excel-writer 子 catalog：补 compile_probe / behavior_test 细粒度条目
- **编号**：T4.2
- **优先级**：P1
- **依赖**：无
- **涉及文件**：`parity/public-api-evidence/excel-writer.json`、`tests/easyexcel-test/tests/public_api_excel_writer_evidence_tests.rs`
- **当前状态**：现有 3 条 evidence（compile/behavior/java-golden），但 `fill(Supplier,...)`、`write(Supplier,...)` 等 4 个 Supplier 重载的 supplier 模式仅由 `fill_supplier_calls`/`write_supplier_calls`（契约 2 字段）覆盖，无独立 per-overload evidence 条目。
- **具体动作**：
  1. 在 `excel-writer.json` 新增 evidence 条目（建议 id：`excel-writer.supplier-fill.behavior.v1`、`excel-writer.supplier-write.behavior.v1`），每条引用对应 Java 方法签名（`fill(Ljava/util/function/Supplier;...)` 等）与 Rust id，`commands` 指向 `public_api_excel_writer_evidence_tests`。
  2. 在测试文件加 `fn excel_writer_supplier_fill_invokes_supplier_lazily()` 与 `fn excel_writer_supplier_write_invokes_supplier_lazily()`，断言 supplier 调用次数 == 契约值（`fill_supplier_calls`/`write_supplier_calls`）。
- **验收标准**：`excel-writer.json` evidence 数 3 → ≥5；新测试通过；`run_public_api_evidence.py` 全绿。
- **工作量**：2h

#### 任务 4.3 — excel-builder 子 catalog：补 behavior 细粒度条目
- **编号**：T4.3
- **优先级**：P1
- **依赖**：无
- **涉及文件**：`parity/public-api-evidence/excel-builder.json`、`tests/easyexcel-test/tests/public_api_excel_builder_evidence_tests.rs`
- **当前状态**：现有 3 条 evidence；builder 契约 13 字段，测试 4 个。
- **具体动作**：
  1. 在 `excel-builder.json` 新增 evidence（如 `excel-builder.chain-method.behavior.v1`），覆盖 `sheet()/head()/registerConverter()/relativeHead()/needHead()` 等链式方法。
  2. 在测试文件加 1-2 个 `#[test]` 验证链式构造返回 `Self`（参考 writer 模式）。
- **验收标准**：evidence 数 3 → ≥4；新测试通过。
- **工作量**：1.5h

#### 任务 4.4 — excel-analyser 子 catalog：补 behavior 细粒度条目
- **编号**：T4.4
- **优先级**：P2
- **依赖**：无
- **涉及文件**：`parity/public-api-evidence/excel-analyser.json`、`tests/easyexcel-test/tests/public_api_excel_analyser_evidence_tests.rs`
- **当前状态**：analyser 契约 109 字段（最大），但测试仅 2 个；evidence 3 条。
- **具体行动**：
  1. 在 `excel-analyser.json` 新增 evidence（如 `excel-analyser.xls-record-handlers.behavior.v1`），覆盖各 record handler SID（`BOF_SID`/`LABEL_SST_SID`/`FORMULA_SID` 等，已在测试 import 列表 :6-26）。
  2. 在测试文件加 `#[test]` 验证关键 SID 常量与契约中的 Java SID 一致。
- **验收标准**：evidence 数 3 → ≥4；新测试通过。
- **工作量**：2h

#### 任务 4.5 — 根 catalog：补 facade 子方法重载 evidence
- **编号**：T4.5
- **优先级**：P2
- **依赖**：无
- **涉及文件**：`parity/public-api-evidence.json`、`tests/easyexcel-test/tests/public_api_facade_evidence_tests.rs`
- **当前状态**：facade evidence 3 条，`java_ids` 已含全部 read/write/readSheet/writerSheet/writerTable 重载（:13-44），但无 per-method-group 的细粒度 evidence。
- **具体动作**：
  1. 在根 catalog 新增 evidence（如 `facade.read-overloads.behavior.v1`、`facade.write-overloads.behavior.v1`）。
  2. 在测试文件加 `#[test]` 验证 `EasyExcel::read()` 各重载（File/InputStream/String/Class/ReadListener 组合）编译并通过。
- **验收标准**：根 evidence 数 6 → ≥8；新测试通过。
- **工作量**：2h

#### 任务 4.6 — 全部 evidence 的 commands 数组可执行性回归
- **编号**：T4.6
- **优先级**：P0
- **依赖**：T4.1、T4.2、T4.3、T4.4、T4.5
- **涉及文件**：`scripts/run_public_api_evidence.py`、所有 parity catalog
- **当前状态**：`run_public_api_evidence.py` 加载根 catalog（含 include 子 catalog），跑每个 evidence 的 `commands`，捕获 exit code 与 stdout/stderr SHA256。
- **具体动作**：
  1. 执行 `python3 scripts/run_public_api_evidence.py --catalog parity/public-api-evidence.json --output parity/public-api-evidence.report.json --repo-root $(pwd)`。
  2. 断言：所有 evidence exit code == 0，输出 report 中无失败条目。
- **验收标准**：evidence 总数 26 契约点 → ≥100（含细粒度条目）；report 全绿。
- **工作量**：1h

**子任务四汇总**：4 子 catalog 各补 compile_probe + behavior_test，evidence 总数 26 → 100+，所有 `commands` 可执行。

---

## 5. 子任务五：coverage 持久化（CI job + artifact 归档）

### 现状证据
- `scripts/coverage.sh`：`cargo llvm-cov --workspace --all-features --html --output-dir coverage` + `report --fail-under-lines/regions/functions 95`。
- `.github/workflows/ci.yml:49-67`：`coverage` job 仅 `if: release || workflow_dispatch`，**无** `actions/upload-artifact`，无历史归档。
- `coverage/` 目录被 `.gitignore`（任务输入事实 #6）。

---

#### 任务 5.1 — CI coverage job 增加 artifact 上传
- **编号**：T5.1
- **优先级**：P1
- **依赖**：无
- **涉及文件**：`.github/workflows/ci.yml`
- **当前状态**：`ci.yml:67` 仅 `run: ./scripts/coverage.sh`，无产物留存。
- **具体动作**：在 `coverage` job 末尾加 step：
  ```yaml
  - name: Upload coverage HTML report
    uses: actions/upload-artifact@v4
    with:
      name: coverage-html-${{ github.sha }}
      path: coverage/
      retention-days: 30
  - name: Upload coverage lcov
    uses: actions/upload-artifact@v4
    with:
      name: coverage-lcov-${{ github.sha }}
      path: coverage/lcov.info
      retention-days: 90
  ```
  并在 `scripts/coverage.sh` 追加 `--lcov --output-path coverage/lcov.info`（若 llvm-cov 支持）。
- **验收标准**：CI 触发后 GitHub Actions 出现 2 个 artifact；`coverage/lcov.info` 存在。
- **工作量**：1h

#### 任务 5.2 — coverage 在 PR 上以「非门禁」模式运行并评论
- **编号**：T5.2
- **优先级**：P2
- **依赖**：T5.1
- **涉及文件**：`.github/workflows/ci.yml`（新增 job `coverage-pr`）
- **当前状态**：PR 不跑 coverage（避免 20min 全量开销）。
- **具体行动**：
  1. 新增 job `coverage-pr`：`if: github.event_name == 'pull_request'`，跑 `cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info`（不门禁）。
  2. 用 `actions/download-artifact` + 第三方 action（如 `5monkeys/cobertura-action`）将 diff 覆盖率评论到 PR。
  - 待确认：是否接受额外第三方 action 依赖。
- **验收标准**：PR 触发后出现覆盖率评论；不阻塞合并。
- **工作量**：2h

#### 任务 5.3 — 本地 coverage 快照脚本
- **编号**：T5.3
- **优先级**：P2
- **依赖**：无
- **涉及文件**：`scripts/coverage.sh`（或新增 `scripts/coverage-snapshot.sh`）
- **当前状态**：`coverage.sh` 只生成不归档；无本地历史对比。
- **具体动作**：增加 `--snapshot <dir>` 选项，把 `coverage/summary.json` 拷到 `reports/coverage-snapshots/<date>/`（`reports/` 已存在），供本地回归对比。
- **验收标准**：`./scripts/coverage.sh --snapshot reports/coverage-snapshots/2026-08-10` 后目录含 summary。
- **工作量**：1h

**子任务五汇总**：CI artifact 上传 + PR 评论 + 本地快照，覆盖率产物可追溯。

---

## 6. 子任务六：examples 补 README

### 现状证据
- `examples/README.md`（25 行，仅框架表 + curl 示例）。
- 10 个示例子目录（`read/write/fill/axum/actix/hyper/poem/rocket/salvo/warp`）均无各自 `README.md`，仅 `Cargo.toml` + `src/main.rs`（read/write/fill 另有 `tests/spawn_binary.rs`）。

---

#### 任务 6.1 — read 示例 README
- **编号**：T6.1
- **优先级**：P2
- **依赖**：无
- **涉及文件**：`examples/read/README.md`（新增）
- **当前状态**：`examples/read/` 仅 `Cargo.toml` + `src/main.rs` + `tests/spawn_binary.rs`。
- **具体动作**：编写 README，内容含：用途（读取示例）、运行命令（`cargo run -p easyexcel-demo-read <path>`）、输入/输出说明、与 Java demo 对应关系、关联测试（`spawn_binary.rs`）。
- **验收标准**：文件存在，≥ 30 行，含运行命令。
- **工作量**：0.5h

#### 任务 6.2 — write 示例 README
- **编号**：T6.2
- **优先级**：P2
- **依赖**：无
- **涉及文件**：`examples/write/README.md`（新增）
- **具体动作**：同 T6.1，针对 write（输出 5 行 xlsx，参考 `spawn_binary.rs:19` 断言「已写入 5 行」）。
- **验收标准**：同上。
- **工作量**：0.5h

#### 任务 6.3 — fill 示例 README
- **编号**：T6.3
- **优先级**：P2
- **依赖**：无
- **涉及文件**：`examples/fill/README.md`（新增）
- **具体动作**：同上，针对 fill（模板填充，输出 `target/demo-fill-output.xlsx`，参考 `spawn_binary.rs:19-26`）。
- **验收标准**：同上。
- **工作量**：0.5h

#### 任务 6.4 — axum 示例 README
- **编号**：T6.4
- **优先级**：P2
- **依赖**：无
- **涉及文件**：`examples/axum/README.md`（新增）
- **当前状态**：`examples/axum/src/main.rs`（104 行）含 download/upload 路由、`ExcelWebPolicy`、graceful shutdown。
- **具体动作**：编写 README，含：端口 8080、`GET /download` + `POST /upload`、curl 示例（CSV/XLS/XLSX）、policy 参数说明、与 conformance 套件关系。
- **验收标准**：≥ 40 行，含 curl 与端口。
- **工作量**：0.7h

#### 任务 6.5 — actix 示例 README
- **编号**：T6.5
- **优先级**：P2
- **依赖**：无
- **涉及文件**：`examples/actix/README.md`（新增）
- **具体动作**：同 T6.4，端口 8081，actix 特有 extractor 用法。
- **验收标准**：同上。
- **工作量**：0.7h

#### 任务 6.6 — hyper 示例 README
- **编号**：T6.6
- **优先级**：P2
- **依赖**：无
- **涉及文件**：`examples/hyper/README.md`（新增）
- **具体动作**：同 T6.4，端口 8082。
- **验收标准**：同上。
- **工作量**：0.7h

#### 任务 6.7 — poem 示例 README
- **编号**：T6.7
- **优先级**：P2
- **依赖**：无
- **涉及文件**：`examples/poem/README.md`（新增）
- **具体动作**：同 T6.4，端口 8083。
- **验收标准**：同上。
- **工作量**：0.7h

#### 任务 6.8 — rocket 示例 README
- **编号**：T6.8
- **优先级**：P2
- **依赖**：无
- **涉及文件**：`examples/rocket/README.md`（新增）
- **具体动作**：同 T6.4，端口 8000。
- **验收标准**：同上。
- **工作量**：0.7h

#### 任务 6.9 — salvo 示例 README
- **编号**：T6.9
- **优先级**：P2
- **依赖**：无
- **涉及文件**：`examples/salvo/README.md`（新增）
- **具体动作**：同 T6.4，端口 8084，注明 MSRV 1.89。
- **验收标准**：同上。
- **工作量**：0.7h

#### 任务 6.10 — warp 示例 README
- **编号**：T6.10
- **优先级**：P2
- **依赖**：无
- **涉及文件**：`examples/warp/README.md`（新增）
- **具体动作**：同 T6.4，端口 8085。
- **验收标准**：同上。
- **工作量**：0.7h

#### 任务 6.11 — 顶层 examples/README.md 增补索引与各子示例链接
- **编号**：T6.11
- **优先级**：P2
- **依赖**：T6.1-T6.10
- **涉及文件**：`examples/README.md`（修改）
- **当前状态**：现有 README 仅框架表 + curl，无 CLI 示例（read/write/fill）章节，无子 README 链接。
- **具体动作**：补充：(1) read/write/fill 三个 CLI 示例章节 + 运行命令；(2) 7 框架示例分别链接到各自 README；(3) conformance 套件引用。
- **验收标准**：README 含全部 10 个子示例链接；≥ 50 行。
- **工作量**：0.5h

**子任务六汇总**：10 个子 README 新增 + 1 个顶层 README 增补。

---

## 7. 全局汇总

| 子任务 | 任务数 | 新增测试数（目标） | 优先级 | 总工作量 |
|---|---:|---:|---|---:|
| 1. ExcelRows<T> 单测矩阵 | 8 | 8 | P0-P2 | 6.5h |
| 2. ExcelWriteFillExecutor 单测 | 8 | 8 | P0-P2 | 3.2h |
| 3. Web conformance 扩格式 | 6（含夹具） | +35（每框架 +5） | P0-P2 | 12.5h |
| 4. parity 证据扩充 | 6 | +6-10（evidence 条目） | P0-P2 | 11.5h |
| 5. coverage 持久化 | 3 | — | P1-P2 | 4h |
| 6. examples README | 11 | — | P2 | 6.9h |
| **合计** | **42** | **+57 测试 + evidence 26→100+** | | **44.6h** |

### 优先级排序（建议执行顺序）
1. **P0 先行**（约 18h）：T1.1、T1.2、T1.4、T1.5、T2.1-T2.5、T3.1、T4.1、T4.6
2. **P1 跟进**（约 16h）：T1.3、T1.6、T1.7、T2.7、T3.2、T3.4、T3.5、T4.2、T4.3、T5.1
3. **P2 收尾**（约 10h）：剩余

### 全局验收命令（最终回归）
```bash
# 1. 全量测试
cargo test --workspace --all-features
# 2. coverage 门禁
./scripts/coverage.sh
# 3. parity evidence 回归
python3 scripts/run_public_api_evidence.py \
  --catalog parity/public-api-evidence.json \
  --output parity/public-api-evidence.report.json \
  --repo-root $(pwd)
# 4. 文档存在性校验
test -f examples/read/README.md && \
test -f examples/axum/README.md && ... # 10 个子 README
```

### 待确认事项
1. **T1.4**：稳定触发 `processing_timeout` 的夹具策略（是否用 `tokio::time::pause` 或 gated reader）。
2. **T3.6**：各框架 test harness 是否支持中途 drop body stream（取消传播测试可行性）。
3. **T4.1**：`./scripts/export-java-golden.sh` 当前是否能生成 `converter_api.contract.json`；若需 Java 环境，CI 中是否已具备。
4. **T5.2**：是否接受第三方 cobertura-action 依赖。

---

> 本文档所有任务项均基于代码与测试现状（文件:行号）推导，未编造。涉及不确定处已标注「待确认」。
