# Performance Evaluator: easyexcel-java-rust-parity

## Objective
建立 Java/Rust 共享性能契约，完成正确性、速度、延迟、内存、CPU、并发、磁盘与稳定性对比，并通过各自历史基线回归门禁

## Evaluator Command
```sh
python3 benchmarks/scripts/compare_results.py --spec benchmarks/spec/benchmark-suite-v1.json --profile release --baseline benchmarks/baselines/release-linux-x64.json --require-baseline --output benchmarks/results/release/report.json benchmarks/results/release/raw-results.jsonl
```

## Pass/Fail Contract
release 矩阵完整；checksum 与跨读 100%；CV<=10%；Rust 自身吞吐回退<=10%、RSS<=15%；xlsx-stream-write/xlsx-event-read worker 1/2/4 Rust/Java 中位比>=1.00 且 95% CI 下界>=0.95，worker 8/16 中位比>=0.90；30 分钟 70/30 soak 通过

This evaluator must exist and produce concrete pass/fail evidence before the performance goal can be completed.
