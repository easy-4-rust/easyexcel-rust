package com.alibaba.easyexcel.test.benchmark;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Objects;

import com.alibaba.fastjson2.JSON;
import com.alibaba.fastjson2.JSONArray;
import com.alibaba.fastjson2.JSONObject;
import org.springframework.util.StringUtils;

/** Java/Rust 共用性能契约的只读视图。 */
public final class BenchmarkSpec {

    private final int schemaVersion;
    private final String suiteId;
    private final int batchSize;
    private final List<BenchmarkScenario> scenarios;
    private final String sha256;

    private BenchmarkSpec(
        int schemaVersion,
        String suiteId,
        int batchSize,
        List<BenchmarkScenario> scenarios,
        String sha256) {
        this.schemaVersion = schemaVersion;
        this.suiteId = suiteId;
        this.batchSize = batchSize;
        this.scenarios = Collections.unmodifiableList(new ArrayList<>(scenarios));
        this.sha256 = sha256;
    }

    public static BenchmarkSpec read(Path path) throws IOException {
        byte[] bytes = Files.readAllBytes(path);
        JSONObject root = JSON.parseObject(new String(bytes, StandardCharsets.UTF_8));
        JSONArray values = root.getJSONArray("scenarios");
        List<BenchmarkScenario> scenarios = new ArrayList<>(values.size());
        for (int index = 0; index < values.size(); index++) {
            scenarios.add(BenchmarkScenario.fromJson(values.getJSONObject(index)));
        }
        return new BenchmarkSpec(
            root.getIntValue("schema_version"),
            root.getString("suite_id"),
            root.getIntValue("batch_size"),
            scenarios,
            sha256(bytes));
    }

    public BenchmarkScenario scenario(String id) {
        for (BenchmarkScenario scenario : scenarios) {
            if (Objects.equals(scenario.getId(), id)) {
                return scenario;
            }
        }
        throw new IllegalArgumentException("unknown benchmark scenario: " + id);
    }

    public void validate() {
        if (schemaVersion != 1 || !StringUtils.hasText(suiteId) || batchSize <= 0) {
            throw new IllegalArgumentException("unsupported or incomplete BenchmarkSpec");
        }
    }

    public int getBatchSize() {
        return batchSize;
    }

    public String getSha256() {
        return sha256;
    }

    private static String sha256(byte[] bytes) {
        try {
            byte[] digest = MessageDigest.getInstance("SHA-256").digest(bytes);
            StringBuilder result = new StringBuilder(digest.length * 2);
            for (byte value : digest) {
                result.append(String.format(java.util.Locale.ROOT, "%02x", value & 0xff));
            }
            return result.toString();
        } catch (NoSuchAlgorithmException exception) {
            throw new IllegalStateException("SHA-256 is required by the Java runtime", exception);
        }
    }
}
