# RFC: 公式引擎结果缓存（dirty-cell 增量重算评估）

> 状态：**草案** | 关联：[WRITE-CONSTANT-MEMORY-OPTIMIZATION.md](WRITE-CONSTANT-MEMORY-OPTIMIZATION.md) 子任务 6.1
>
> 决策问题：`Engine::recalc()` 当前每次全量重算所有公式 cell。是否引入「dirty-cell 增量重算 + 结果缓存」，以减少重复计算与无变更场景的开销？

## 1. 背景

当前公式引擎（已确认证据）：

| 维度 | 现状 | 证据 |
|---|---|---|
| 缓存内容 | 仅缓存解析后 AST，**不缓存求值结果** | `crates/easyexcel-formula/src/formula/engine.rs:41-44` `ast_cache: HashMap<String, Rc<Expr>>` |
| AST 缓存入口 | `parse_cached()` 命中即复用 AST | `engine.rs:64-72` |
| 重算策略 | **每次 `recalc()` 全量重算所有公式 cell** | `engine.rs:89-281` |
| 依赖图 | 已拓扑排序（Kahn 算法） | `engine.rs:159-171` |
| range 解析阈值 | `ENUM_THRESHOLD=4096`（小 range 枚举，大 range 扫全集） | `engine.rs:126` |
| spill 收敛 | `MAX_PASSES=12`，每趟全量重算直到 spill 集合稳定 | `engine.rs:197-263` |
| 模型层 cached 字段 | `Cell::Formula { expr, cached }` 已有 `cached` 存储位 | `engine.rs:283-289` `write_cached` |
| 求值计数 | `report.evaluated` 每趟重置（`:201`），每趟对 topo 序全部 node 调 `ev.eval(ast)` | `engine.rs:202-219` |

关键观察：模型层的 `Cell::Formula.cached` 字段**已经存在**（用于把结果写回工作簿，供后续非公式读取），但引擎层没有「dirty 标记 / 结果缓存表」来跳过未变更的公式。每次 `recalc()` 都对全部公式 node 重新求值，哪怕输入完全没变。

## 2. 触发本 RFC 的场景

- 同一工作簿多次 `recalc()`（如交互式编辑器、Web 协同、CLI 反复求值）：第二次起所有公式都重算，即便 precedent cell 未改。
- 单点修改一个上游 cell，下游连带重算本应只触及依赖子图，当前却全图重算。
- 大公式表（数万公式 cell）的 recalc 延迟主要花在「没必要算的 cell」上。

## 3. 决策选项

### 选项 A：维持现状（全量重算，不引入 dirty/结果缓存）

- `recalc()` 每次对全部公式 node 求值。
- 简单、正确性容易保证、无状态。

### 选项 B：dirty-cell 增量重算（推荐评估方向）

- 给每个公式 cell 加 dirty 标记（或在引擎层维护 `HashSet<Coord>` dirty 集合）。
- 当某个 value cell 被改动时，标记其所有（传递）dependent 公式为 dirty。
- `recalc()` 只对 dirty 子集求值；依赖图已拓扑序，按 dirty 节点的拓扑闭包重算。
- 缓存命中（非 dirty）的 cell 复用 `Cell::Formula.cached` 里的旧值。

### 选项 C：全量重算 + 结果缓存表（不做 dirty 标记，仅记忆上次结果按输入指纹复用）

- 维护 `HashMap<Coord, (input_fingerprint, Value)>`，对每个公式记录其所有 precedent 的值指纹。
- `recalc()` 时若指纹未变则复用缓存值，否则重算并更新指纹。
- 不需要 dirty 传播，但指纹计算本身有开销（对每个公式遍历其 precedent range）。

## 4. 推荐方案：**选项 A（维持现状）为短期决策；选项 B 作为中长期演进路径，需先补基准测试再决定是否落地**

### 短期（本工作流内）：维持选项 A，不引入结果缓存

**理由：**

1. **当前没有基准证明这是真热点**。在引入任何缓存前，必须先用基准（`criterion` bench，构造 1k/10k/100k 公式的工作簿）量化 `recalc()` 的实际耗时分布。没有数据就上缓存，是典型的过早优化。

2. **正确性风险高**。公式引擎的正确性边界很微妙：
   - spill 动态数组（`engine.rs:197-263`）跨 cell 传播，依赖多趟收敛；dirty 传播必须与 spill 收敛循环正确交互，否则会出现「spill 区域变了但下游没标 dirty」的 stale 结果。
   - volatile 函数（NOW/TODAY/RAND）必须每趟重算（`engine.rs:196` `(now, today)` 每趟重取）。
   - 跨 sheet 引用、range 依赖（`RangeDep`，`:24-37`）的 dirty 传播比单 cell 引用复杂得多，range 改动要标记所有落在 range 内的公式 dirty。

3. **AST 缓存已经覆盖了最贵的部分**。`parse_cached()`（`:64-72`）已省掉重复解析；剩下的 `ev.eval(ast)` 对纯算术/简单引用相当快。要证明结果缓存有收益，得先证明 eval 而非 parse 是瓶颈。

4. **模型层 `cached` 字段已存值**。对「读侧只想要上次结果」的场景，读路径可以直接读 `Cell::Formula.cached`，不必每次都 `recalc()`。即——很多场景下「不调 recalc」本身就是缓存。

### 中长期演进：若基准证明 eval 是热点，走选项 B（dirty-cell 增量），不走选项 C

**为什么不选 C（指纹复用）：**
- 指纹方案要对每个公式遍历其 precedent range 算 hash，开销与直接重算小公式相当，对简单公式是负收益。
- 指纹方案不解决「单点改动触发全图重算」的核心问题，只解决「完全没改也重算」—— 而后者用「调用方不调 recalc」就能解决。

**选项 B 的落地前提（必须先满足）：**
1. 有 `criterion` 基准证明：在 10k+ 公式、单点改动的场景，全量 recalc 耗时不可接受。
2. dirty 传播的设计文档通过评审，特别是：
   - volatile 函数永远 dirty。
   - spill 锚点的 dirty 传播规则（锚点 dirty → 其 spill 区域内所有 dependent dirty）。
   - range 引用的 dirty 传播（改动 range 内任一 value cell → 该 range 的所有 dependent 公式 dirty）。
3. 增加了「dirty 传播正确性」的 property-based 测试（随机改一个 cell，对比全量重算 vs 增量重算结果一致）。

## 5. 风险（针对短期决策：维持现状）

| 风险 | 等级 | 缓解 |
|---|---|---|
| 某个未发现的大公式表场景下 recalc 延迟过高 | 中 | 先补基准（见任务 6.2 草案），用数据驱动决策 |
| 用户误以为已有结果缓存 | 低 | 在 `Engine`/`recalc` 的 doc comment 明确写「无结果缓存，每次全量重算；读侧如不需最新值可直接读 `Cell::Formula.cached`」 |

## 6. 风险（针对中长期若落地选项 B）

| 风险 | 等级 | 缓解 |
|---|---|---|
| spill 收敛与 dirty 传播交互产生 stale 结果 | 高 | 强制 property test：增量结果 == 全量结果，覆盖 spill/被 spill 覆盖的 cell |
| volatile 函数漏标 dirty | 高 | 在 Evaluator 对 volatile 函数节点设常驻 dirty 标记 |
| range 依赖的 dirty 传播漏标（改 range 边界外 cell 误标 dirty，或边界内 cell 漏标） | 高 | 复用 `RangeDep::contains`（`engine.rs:34-36`）做传播判断，并加 fuzz |
| 引入 `&mut` 借用冲突（dirty 集合 + 工作簿遍历） | 中 | 先收集 dirty 集合，再分阶段求值，避免边遍历边改 |

## 7. 回滚策略

- 短期决策（维持现状）：无可回滚，本就是不改动。
- 若后续落地选项 B：增量重算与全量重算应作为两个可切换的 `recalc` 入口（如 `recalc_full` / `recalc_incremental`），出问题时一行配置切回全量，无需 revert 代码。

## 8. 结论

- **短期**：维持选项 A（全量重算，不引入 dirty/结果缓存）。先把 `Cell::Formula.cached` 的「读侧复用」语义在文档里讲清楚，避免重复 recalc。
- **中长期**：若基准证明 eval 是热点，走选项 B（dirty-cell 增量重算），**不**走选项 C（指纹复用）。
- **落地前置条件**：必须先有 criterion 基准数据 + dirty 传播设计文档 + property test 一致性校验。三者未齐备前，不改 `recalc()`。

## 9. 待确认项

- 是否已有针对 `Engine::recalc` 的 benchmark（搜 `criterion` / `bench` in `easyexcel-formula`）—— 本 RFC 撰写时未确认，需 `find crates/easyexcel-formula -name "*.rs" | xargs grep -l "criterion\|bench"` 核实。
- `Cell::Formula.cached` 在读侧的填充时机：是仅在 `recalc` 后填充，还是模型层有独立的 lazy 求值入口 —— 影响「读侧复用」建议是否成立。
