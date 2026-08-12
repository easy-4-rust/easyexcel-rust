# Rust 501–800 行文件内聚性复核

状态：已复核（2026-08-06）  
适用快照：`dev`，基线提交 `30ac6719736414f53d09771b69dd6dce9196ac7b` 加本轮未提交迁移整改。

## 判定规则

- 超过 800 行的手写 Rust 文件一律阻塞迁移完成，必须拆分。
- 501–800 行不是自动失败；只有文件仍保持单一对象、单一算法族或单一测试夹具上下文时才允许保留。
- 已混合对象、格式后端或互不相关用例的文件必须拆分，不能仅凭本记录豁免。
- 本轮已经把 `ExcelWriter` 巨型实现、公式函数族、格式读写族和测试场景拆到同名子目录；所有手写 `.rs` 均不超过 800 行。
- 复核命令必须同时执行外部迁移审计和 AST 级生产项审计，避免文件中间的 `#[cfg(test)]` 隐藏后续生产代码。

## 当前复核结果

| 复核根目录 | 501–800 行文件数 | 最大行数 | 内聚性结论 | 决策 |
|---|---:|---:|---|---|
| `crates/easyexcel/src` | 22 | 793 | 单个 Java 风格门面对象及其 `impl`、同一转换族，或已经按写入行为拆出的实现片段；公开多对象已拆为同名子文件 | 保留，逐文件受 800 行硬门槛约束 |
| `crates/easyexcel-formula/src` | 27 | 797 | 同一公式解析阶段或同一 Excel 函数家族；注册、求值与同族边界处理需要共享局部 helper | 保留函数族内聚边界 |
| `crates/easyexcel-xlsx/src` | 7 | 757 | OOXML reader、writer、style、template package 各自独立；没有把不同格式后端重新合并 | 保留格式内聚边界 |
| `crates/easyexcel-xls/src` | 3 | 713 | BIFF8 reader、writer、style、PTG/record 分属独立文件；均为同一二进制格式算法族 | 保留格式内聚边界 |
| `crates/easyexcel-model/src` | 1 | 536 | Workbook/Cell 行为与日期模型分别内聚，且不含 ZIP、XML 或格式后端实现 | 保留模型内聚边界 |
| `crates/easyexcel-format/src` | 1 | 571 | Excel 数值格式算法及精度/舍入规则共享同一解析上下文 | 保留算法内聚边界 |
| `crates/easyexcel-io/src` | 0 | 0 | gzip 记录对象已进一步按对象拆分，不再有超过 500 行的文件 | 无需豁免 |
| `tests/easyexcel-test/tests` | 16 | 761 | 单一 Java 对照场景或共享 fixture/helper；生产逻辑不在测试文件中 | 保留用例上下文，继续按场景拆分新增用例 |
| `xtask/src` | 1 | 659 | 门面边界审计的同一规则集合；规则读取和诊断必须共享扫描上下文 | 保留审计用例内聚边界 |

## 本轮拆分证据

- `crates/easyexcel/src/write/excel_writer_core.rs` 只保留重导出和按职责 `include!`，实现已拆为 schema/head、XLS、XLSX row、template、handler lifecycle 等行为文件。
- 原来单文件暴露多个公共对象的 38 个门面文件已经拆为同名子目录；第二轮 AST 审计补出的 7 个文件也已拆分。
- 大型测试文件按测试函数边界拆成 `*_split/chunk_*.rs`，共享 fixture 留在父模块，避免复制准备逻辑。
- 生产代码中的 wildcard import/re-export 已改为具名导入；测试片段通过 `#[cfg(test)]` 明确限定作用域。

## 复核命令

外部迁移审计允许以下已经记录的根目录作为 `--reviewed-large-file` 参数：

```text
crates/easyexcel/src
crates/easyexcel-formula/src
crates/easyexcel-xlsx/src
crates/easyexcel-xls/src
crates/easyexcel-model/src
crates/easyexcel-format/src
crates/easyexcel-io/src
tests/easyexcel-test/tests
xtask/src
```

该许可只适用于本记录中的当前数量和最大行数。任一根目录的数量或最大行数发生变化时，必须重新生成清单并复核；不得把目录级参数当作永久豁免。
