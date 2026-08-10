package com.alibaba.easyexcel.test.benchmark;

import com.alibaba.fastjson2.JSONObject;

/** 共享 BenchmarkSpec 中的单个场景。 */
public final class BenchmarkScenario {

    private final String id;
    private final String format;
    private final String operation;
    private final String mode;
    private final String memory;

    private BenchmarkScenario(String id, String format, String operation, String mode, String memory) {
        this.id = id;
        this.format = format;
        this.operation = operation;
        this.mode = mode;
        this.memory = memory;
    }

    public static BenchmarkScenario fromJson(JSONObject value) {
        return new BenchmarkScenario(
            value.getString("id"),
            value.getString("format"),
            value.getString("operation"),
            value.getString("mode"),
            value.getString("memory"));
    }

    public String getId() {
        return id;
    }

    public String getFormat() {
        return format;
    }

    public String getOperation() {
        return operation;
    }

    public String getMode() {
        return mode;
    }

    public String getMemory() {
        return memory;
    }
}
