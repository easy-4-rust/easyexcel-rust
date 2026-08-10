# Java benchmark runner

This directory versions the Java side of the shared benchmark contract. It is
compiled against a clean `easy-4-java/easyexcel` v4.0.3 checkout into that
checkout's ignored `target/test-classes` directory. The Java source worktree is
therefore never modified, while release attestation still hashes the exact
runner class, Java executable, classpath, and clean Java source fingerprint.

The runner sources are benchmark infrastructure, not Rust facade compatibility
types. Their observable workload is defined only by
`../spec/benchmark-suite-v1.json`. Both runners publish the byte size of the
same canonical checksum-row payload; the comparator recomputes that value
independently before reporting physical/logical compression ratios.
