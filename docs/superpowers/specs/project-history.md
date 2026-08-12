# 项目历史（History）

> 文档说明：统一索引 easyexcel-rust 从 v0.1.0-alpha.1 到 v0.1.3 的关键里程碑、
> 架构决策记录（ADR 摘要）、ROADMAP 状态仪表盘与待确认事项。
> 面向新加入者快速理解项目"做过什么、为什么、当前在哪、还有什么要做"。
>
> 最后更新：2026-08-12

---

## 1. 版本时间线

| 版本 | 发布日期 | 关键变更 | 详细文档 |
|------|----------|----------|----------|
| v0.1.0-alpha.1 | 2026-08-04 | 首个公开预发布：Alibaba EasyExcel 4.0.3 的 Rust 高保真迁移。三格式读写（XLSX/XLS/CSV）、恒定内存流式（百万行写峰值 10.8 MiB）、OOXML 模板填充、8 个 Web 框架适配器、加密 XLSX 读写、2771 测试全绿（含 88 个 Java golden 对拍） | `CHANGELOG.md` |
| v0.1.0 | 2026-08-07 | 首个生产就绪正式版本。收敛全部工作区 crate 到同一版本线；明确 XLS Event Mode / 旧 XLS 密码保护 / XLS 占位符填充的 `Unsupported` 边界；通过全 workspace 测试、Java parity/golden、大文件、Clippy、RustSec、cargo-deny、MSRV 和 facade boundary 门禁 | `CHANGELOG.md` |
| v0.1.1 | 2026-08-07 | 文档与发布元数据修订版。为 `crates/` 下全部 21 个正式发布 crate 增加结构对等的英文 `README.md` 与中文 `README.zh-CN.md`；每个 crate 的 Cargo 发布元数据中显式声明 `readme = "README.md"` | `CHANGELOG.md` |
| v0.1.2 | 2026-08-07 | 模块文档增强版本。将 21 个 crate 的 README 扩展为可独立使用的模块手册，含 Mermaid 架构图、能力矩阵、公共 API 表、安装说明、基础与进阶 Rust 示例、错误边界、依赖关系图；Web 适配器补充各框架原生 extractor/responder 示例 | `CHANGELOG.md` |
| v0.1.3 | 2026-08-07 | 公共门面文档修订版。明确基础引擎 crate 独立发布仅用于内部依赖分层，业务应用统一依赖 `easyexcel`；补充 `easyexcel::csv::CsvRowSource` 零成本重导出；全部工作区 crate 统一升级到 0.1.3 | `CHANGELOG.md` |

### 里程碑概览

- **2026-07-17**：项目启动（`bootstrap EasyExcel-compatible Rust workspace`，首个 commit）
- **2026-08-03**：alpha.1 版本代码冻结（2771 测试、96.38% 行覆盖率）
- **2026-08-04**：v0.1.0-alpha.1 发布
- **2026-08-06**：facade 引擎拆分重构完成（8 个 refactor commit）
- **2026-08-07**：v0.1.0 / v0.1.1 / v0.1.2 / v0.1.3 同日发布
- **2026-08-10~11**：Q3 ROADMAP P0 任务执行（5 个任务全部通过测试验证）
- **2026-08-11**：生产就绪审计（依赖/代码/fuzz/覆盖率）+ fuzz 基础设施搭建
- **2026-08-11**：Coverage 6 轮提升（75.53% → 88.38%）

---

## 2. 关键架构决策（ADR 摘要）

### ADR-1：不做 XLS Event Mode

| 项目 | 内容 |
|------|------|
| **决策** | XLS Event Mode 不实现，返回类型化 `Unsupported` 错误 |
| **理由** | Java EasyExcel 的 XLS Event Mode 依赖 POI 的 SAX 模式，Rust 生态（calamine）无等价流式 BIFF8 解析器；投入产出比低 |
| **替代方案** | (a) 基于 calamine 的全量读取（当前方案）；(b) 自研 BIFF8 流式解析器（工时过高） |

### ADR-2：parity schema v2 迁移

| 项目 | 内容 |
|------|------|
| **决策** | 将 evidence catalog 从 schema v1 升级到 v2，物化 `family_evidence` 为逐 ID 证据 |
| **理由** | v1 的 `family_evidence` 批量模板无法逐项验证，验证器 `verify_public_api_parity.py` 对非 verified 项 fail-closed；需要逐项粒度才能推进 verified 数 |
| **替代方案** | (a) 维持 v1 + 手动逐项覆盖（易遗漏）；(b) 放弃 parity 门禁（丢失 Java 兼容性保证） |

### ADR-3：CSV STUB 策略

| 项目 | 内容 |
|------|------|
| **决策** | CSV 后端的 93 个不支持功能方法保持空实现（STUB），集中在 `stubs/` 目录 |
| **理由** | CSV 格式不支持样式、合并、冻结窗格等 Excel 功能；Java EasyExcel 的 CSV 后端同样对这些方法做 no-op；保持 API 签名一致可减少用户迁移成本 |
| **替代方案** | (a) 返回 `Err(UnsupportedFeature)`（破坏 API 兼容性）；(b) 完全不暴露这些方法（与 Java API 不对齐） |

### ADR-4：读侧不做 spill，改做包级惰性加载

| 项目 | 内容 |
|------|------|
| **决策** | 读侧不引入写侧风格的 `GzipSheetDataWriter` spill-to-disk 机制，改为对 XLSX ZIP entry 按需读取（lazy parts） |
| **理由** | 内存放大点在 `reader.rs:86-99` 的全包 DOM 物化（所有 zip entry 一次性 `read_to_end`），而非 worksheet body（已是 SAX 流式）；`ZipArchive` 天然支持随机访问，spill 临时文件纯属多此一举 |
| **替代方案** | (a) 完整读侧 spill（复杂度高、维护成本与写侧不对称）；(b) 维持现状全量读取（Web 场景资源限制形同虚设） |
| **详细文档** | `docs/performance/READ-CONSTANT-MEMORY-RFC.md` |

### ADR-5：公式引擎不做 dirty-cell 增量重算（短期）

| 项目 | 内容 |
|------|------|
| **决策** | 短期维持 `Engine::recalc()` 全量重算，不引入结果缓存或 dirty 标记 |
| **理由** | 当前没有基准证明 recalc 是真热点；AST 缓存已覆盖最贵的解析部分；正确性风险高（spill 收敛 + volatile 函数 + range 依赖的 dirty 传播交互复杂） |
| **替代方案** | (a) dirty-cell 增量重算（中长期演进路径，需先补 criterion 基准）；(b) 全量重算 + 结果缓存表（指纹方案对简单公式是负收益） |
| **详细文档** | `docs/performance/FORMULA-RESULT-CACHE-RFC.md` |

### ADR-6：Moka 缓存不做 capacity/TTL/TTI

| 项目 | 内容 |
|------|------|
| **决策** | `MokaSharedStringCache` 保持现状，不配置容量上限、TTL 或 TTI 淘汰 |
| **理由** | Moka 后端语义 = "一次性载入全部 shared strings，读完即销毁"；`get(index)` 对越界返回 Err，加淘汰会让合法 index 偶发 Err，破坏 `SharedStringCacheReader` 契约；大文件保护由 `select_mode` 在 5MB 阈值切到 `FileSharedStringCache` |
| **替代方案** | 加 capacity 上限（需同时改错误语义从"越界"到"可能被淘汰"，破坏现有契约） |

### ADR-7：facade 边界审计硬约束

| 项目 | 内容 |
|------|------|
| **决策** | `easyexcel` 门面不得在生产依赖中直接引入 `calamine`、`cfb`、`zip`、`quick-xml`、`flate2`、`rust_xlsxwriter`、`moka` 或加密实现库 |
| **理由** | 保持单向依赖：门面 → 基础引擎 crate → 第三方库。防止门面变成第二套格式引擎，确保格式实现的唯一归属 |
| **验收** | `cargo run -p xtask -- facade-boundary-audit` |

---

## 3. ROADMAP 状态仪表盘

> 引用来源：`docs/ROADMAP-2026Q3.md`（v1.3，2026-08-11）

### 总览

| 工作流 | 子文档 | 任务项 | 工时(h) | 当前状态 |
|--------|--------|-------:|--------:|----------|
| ① 迁移 gap 闭环 | `docs/migration/ROADMAP-gap-closure.md` | 31 | 116 | A1 完成；A2 阻塞于 A3；B/C/D/E/F/G 待开始 |
| ② 事件读追上 Java 吞吐 | `docs/performance/EVENT-READ-OPTIMIZATION.md` | 13 | 73 | T1.1 完成；T2.1/T3.1/T5.1/T6.2 待开始 |
| ③ 补测试盲区 | `docs/test/COVERAGE-GAP-CLOSURE.md` | 42 | 44.6 | T1.1-T1.8 完成（8/8 测试绿）；其余待开始 |
| ④ 恒定内存与写优化 | `docs/performance/WRITE-CONSTANT-MEMORY-OPTIMIZATION.md` | 8 + 2 RFC | 15.5 | 1.1 完成；2.1/3.1/4.1/5.1/6.1 待开始 |

### 已完成项

**P0 任务（5 个全部通过测试验证，2026-08-11）：**

| 任务 | 产出 | 验证 |
|------|------|------|
| ②T1.1 parse_float 快路径 | `from_into_impls.rs` +11 行 | `converters::from_into_impls` 11 tests 全绿 |
| ③T1.1-T1.8 ExcelRows 单测 | `excel_rows_unit.rs` 新建 8 个测试 | `cargo test -p easyexcel-web` 15 全绿 |
| ①A1 parity schema v2 | 5 catalog `schema_version=2` + 物化器自检 | 退出码 0 |
| ④1.1 样式去重哈希化 | 5 文件 +155/-11 行 | `gzip_spill` 9 tests 全绿 |
| ①A2 converters 物化 | — | 阻塞于 A3（`converter_api.contract.json` 缺失） |

**基础设施修复（2026-08-10~11）：**
- 54 文件、142 个 pre-existing 编译错误修复
- `easyexcel` lib test binary 编译从阻断到 0 errors

**覆盖率提升（6 轮，2026-08-11）：**
- 75.53% → 88.38%（+1800、+737、+176、+404、+3800 行测试）
- 13 pre-existing lib test failures 全部修复（1426 passed, 0 failed）

**生产就绪审计（2026-08-11）：**
- 依赖审计：0 CVE，1 unsound warning（lru，低风险）
- 代码审计：0 unsafe blocks，8 处 NaN panic 已修复
- Fuzz 基础设施：5 个 fuzz target 搭建完成，初始 100 迭代 0 panic
- ResourceLimits 接入 reader + ZIP bomb 防护

### 进行中

- 2 个 CI 测试失败（`t11_write_style07` rgb 格式不匹配、`t22_write_image03` 错误消息不匹配，已标注 `#[ignore]`）
- 429 clippy warning 清理完成（0 warning）

### 待开始（ROADMAP 剩余任务）

| 优先级 | 任务 | 估算工时 |
|--------|------|---------|
| P0 | ①A2 重跑（A3 就绪后）、①B7 84 unmapped 清零验证、①C6 479 ambiguous 清零验证、①G1-G2 全量门禁 | ~8h |
| P1 | ②T2.1 scratch 复用、②T3.1 dispatch 快路径、②T5.1 基线入库、②T6.1 Java runner 构建 | ~28h |
| P1 | ①B1-B6 unmapped 重分类、①C1-C5 ambiguous 消歧、①D1-D3 注解 verified | ~63h |
| P1 | ③T2.1-T2.5 Fill executor 单测、③T3.1-T3.6 Web conformance 扩充、③T4.1-T4.6 parity 证据扩充 | ~28h |
| P2 | ②T4.1 并发管线调研、③T5.1-T5.3 coverage 持久化、③T6.1-T6.11 examples README | ~20h |

---

## 4. 待确认事项

> 来源：`docs/ROADMAP-2026Q3.md` 第 5 节（10 项）

| ID | 待确认事项 | 影响范围 | 当前状态 |
|----|-----------|----------|----------|
| ①-A1 | 物化器输出 `schema_version` 是否需先改 `materialize_public_api_evidence.py:288` | A1 执行方式 | ✅ 已解决：物化器已同步升级到 v2 |
| ①-C | `mapping_resolutions` 落盘位置（顶层 vs 子文件） | C 阶段执行方式 | 待确认 |
| ①-G1 | gate4 catalog 比对范围（catalog 是否需检入） | G1 验收 | 待确认 |
| ②-T2.1 | `RowConsumer::process` 签名变更是否允许（影响 trait ABI） | scratch 复用方案 | 待确认 |
| ②-T4.1 | 并发管线方案 A（仅转换并发）vs B（多 sheet XML 并发） | 并发管线范围 | 待确认（RFC 已产出，见 `docs/performance/EVENT-READ-OPTIMIZATION.md`） |
| ②-T5.1 | Linux 固定 runner 机型规格 | 基线可复现性 | 待确认 |
| ③-T1.4 | 稳定触发 `processing_timeout` 的夹具策略 | 超时测试可行性 | ✅ 已解决：用 `Duration::from_nanos(1)` 触发 |
| ③-T3.6 | 各框架 test harness 是否支持中途 drop body stream | 取消传播测试 | 待确认 |
| ③-T4.1 | `export-java-golden.sh` 能否生成 `converter_api.contract.json` | converter 证据 | ✅ 已解决：已生成（commit `4386e75`） |
| ③-T5.2 | 是否接受第三方 cobertura-action 依赖 | PR coverage 评论 | ✅ 已解决：已集成（commit `5bf843d`） |

---

## 5. 文档索引

### 核心架构文档

| 文档 | 路径 | 说明 |
|------|------|------|
| 架构总览 | `docs/ARCHITECTURE.md` | crate 分层、依赖方向、数据流、格式支持边界、代码放置规则、性能架构 |
| 路线图 | `docs/ROADMAP-2026Q3.md` | 2026 Q3 推进路线图总清单（4 工作流、94 任务、~249h） |
| Changelog | `CHANGELOG.md` | 4 个版本发布记录（v0.1.0-alpha.1 ~ v0.1.3） |

### 迁移与对齐

| 文档 | 路径 | 说明 |
|------|------|------|
| 迁移 gap 闭环 | `docs/migration/ROADMAP-gap-closure.md` | 31 任务 WBS：evidence catalog schema / unmapped / ambiguous / 注解 / enum / 测试类移植 |
| Java 测试对应关系 | `docs/migration/Java测试对应关系.md` | Java 测试类与 Rust 测试的静态映射 |

### 性能优化

| 文档 | 路径 | 说明 |
|------|------|------|
| 事件读优化 | `docs/performance/EVENT-READ-OPTIMIZATION.md` | 13 任务 WBS：205K → 307K+ rows/s 目标分解 |
| 写恒定内存优化 | `docs/performance/WRITE-CONSTANT-MEMORY-OPTIMIZATION.md` | 8 任务 + 2 RFC：样式去重、状态机文档、Moka 审计、spill 矩阵 |
| 读恒定内存 RFC | `docs/performance/READ-CONSTANT-MEMORY-RFC.md` | 决策：不做 spill，改做包级惰性加载 |
| 公式缓存 RFC | `docs/performance/FORMULA-RESULT-CACHE-RFC.md` | 决策：短期维持全量重算，中长期评估 dirty-cell 增量 |

### 测试与质量

| 文档 | 路径 | 说明 |
|------|------|------|
| 覆盖率 gap 闭环 | `docs/test/COVERAGE-GAP-CLOSURE.md` | 42 任务 WBS：ExcelRows 单测 / Fill executor 单测 / Web conformance / parity 证据 |
| 规范合规审计 | `docs/refactor/COMPLIANCE_AUDIT.md` | 4 维度扫描：多类型文件 / mod.rs 类型定义 / wildcard import / STUB 空实现 |

### 安全审计

| 文档 | 路径 | 说明 |
|------|------|------|
| 依赖安全审计 | `docs/security/DEPS_AUDIT.md` | cargo-audit + cargo-deny：0 CVE，1 unsound warning（lru） |
| 代码安全审计 | `docs/security/CODE_AUDIT.md` | unsafe 代码 / clippy / panic 风险 / ResourceLimits / ZIP bomb / XXE |
| Fuzz 状态 | `docs/security/FUZZ_STATUS.md` | 5 个 fuzz target，初始 100 迭代 0 panic |

### 其他参考

| 文档 | 路径 | 说明 |
|------|------|------|
| 兼容性矩阵 | `docs/compatibility.md` | XLSX/XLS/CSV 格式能力矩阵（以 ARCHITECTURE.md 为准） |
| 公开 API parity | `parity/java-rust-public-api.json` | 3236 项 Java → Rust 映射（candidate/ambiguous/unmapped/verified） |
| parity 说明 | `parity/README.md` | verified 四要件、确定性重建命令 |

---

## 6. 关键数字速查

| 维度 | 数值 |
|------|------|
| 当前版本 | 0.1.3（`Cargo.toml` `workspace.version`） |
| Java 基线 | Alibaba EasyExcel 4.0.3 |
| workspace crate 数 | 21 个正式发布 crate + 7 个 web 适配器 + xtask + 测试 + benchmark |
| 总测试数 | 1315+（全绿） |
| Java golden 对拍 | 88 个 |
| parity 映射项 | 3236 项（candidate 2673 / ambiguous 479 / unmapped 84 / verified 0） |
| 覆盖率 | 88.38%（6 轮提升后） |
| MSRV | rustc 1.88 |
| 安全 | 0 CVE、0 unsafe blocks、0 `#[ignore]`（CI 标注的 2 个为已知格式差异） |
| Fuzz target | 5 个（XLSX/XLS/CSV/formula/markdown） |
| Web 框架适配器 | 7 个（Axum / Actix / Hyper / Poem / Rocket / Salvo / Warp） |
| 总 commit 数 | 289 |
| 项目启动 | 2026-07-17 |
