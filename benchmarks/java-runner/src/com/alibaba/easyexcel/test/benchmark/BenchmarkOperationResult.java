package com.alibaba.easyexcel.test.benchmark;

/** 单次读写操作的正确性与文件结果。 */
public final class BenchmarkOperationResult {

    private final long observedRows;
    private final String checksum;
    private final long fileSizeBytes;

    public BenchmarkOperationResult(long observedRows, String checksum, long fileSizeBytes) {
        this.observedRows = observedRows;
        this.checksum = checksum;
        this.fileSizeBytes = fileSizeBytes;
    }

    public long getObservedRows() {
        return observedRows;
    }

    public String getChecksum() {
        return checksum;
    }

    public long getFileSizeBytes() {
        return fileSizeBytes;
    }
}
