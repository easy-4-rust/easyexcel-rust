# Java/Rust 百万行 XLSX 对照（2026-08-08）

本次使用共享 `benchmark-suite-v1.json` 的 4 列、1,000,000 行模型，单 worker、
冷启动、JDK 17/G1/`-Xms512m -Xmx4g`、Rust 1.97.1，在同一台 macOS arm64
机器上测量。每项只有一个样本，因此这是功能与数量级证据，不是 release
稳定性结论；完整 release 门禁仍要求冷热态、7 次测量及并发矩阵。

| 场景 | 实现 | 输入来源 | 操作耗时 | 行/秒 | 峰值 RSS | 临时磁盘峰值 |
|---|---|---|---:|---:|---:|---:|
| XLSX 流式写 | Rust | 生成 | 9.492 s | 105,346 | 9.95 MiB | 777 B |
| XLSX 流式写 | Java | 生成 | 5.006 s | 199,743 | 567.80 MiB | 214.58 MiB |
| XLSX Event 读 | Rust | Rust XLSX | 7.239 s | 138,139 | 10.23 MiB | 777 B |
| XLSX Event 读 | Rust | Java XLSX | 7.812 s | 128,001 | 10.28 MiB | 777 B |
| XLSX Event 读 | Java | Rust XLSX | 2.918 s | 342,707 | 439.56 MiB | 777 B |
| XLSX Event 读 | Java | Java XLSX | 3.259 s | 306,836 | 439.83 MiB | 777 B |

两端读取两份产物时都观察到 1,000,000 行，规范化语义校验和均为
`df7966ddec70e23c9df5f8890d6c512c6ea1883d30f5283ac8d09d483f876c95`。
Rust 文件为 24,918,575 字节，Java 文件为 25,322,264 字节；字节哈希不同是
允许的，交叉读取后的行模型与校验和一致才是兼容性门槛。

单样本中 Java 写入吞吐约为 Rust 的 1.90 倍；Rust 写入峰值 RSS 约为 Java 的
1/57，且没有 Java SXSSF 约 214.58 MiB 的临时磁盘开销。原始 JSON 和两份 XLSX
产物位于本目录；它们不替代完整 release 多样本报告。
