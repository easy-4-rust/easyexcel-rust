# Performance Evaluator: easyexcel-java-rust-parity

## Objective
建立 Java/Rust 共享性能契约，完成正确性、速度、延迟、内存、CPU、并发、磁盘与稳定性对比，并通过各自历史基线回归门禁

## Evaluator Command
```sh
python3 benchmarks/scripts/compare_results.py --spec benchmarks/spec/benchmark-suite-v1.json --baseline benchmarks/baselines/release-linux-x64.json --require-baseline --output benchmarks/results/release/report.json benchmarks/results/release/raw-results.jsonl
```

## Pass/Fail Contract
PASS 当 checksum 与双向重读 100% 通过、变异系数不超过 10%、各实现相对自身稳定基线 median 吞吐下降不超过 10%、RSS 增长不超过 15%，且 release 并发矩阵与 30 分钟压力测试完成

This evaluator must exist and produce concrete pass/fail evidence before the performance goal can be completed.
