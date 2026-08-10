package com.alibaba.easyexcel.test.benchmark;

import java.io.File;
import java.lang.management.GarbageCollectorMXBean;
import java.lang.management.ManagementFactory;
import java.lang.management.MemoryPoolMXBean;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;

import com.alibaba.excel.EasyExcel;
import com.alibaba.excel.ExcelWriter;
import com.alibaba.excel.context.AnalysisContext;
import com.alibaba.excel.read.listener.ReadListener;
import com.alibaba.excel.write.metadata.WriteSheet;
import com.alibaba.fastjson2.JSONObject;
import org.apache.poi.xssf.usermodel.XSSFWorkbook;
import org.springframework.util.StringUtils;

/** 按共享 BenchmarkSpec 执行单阶段 Java 性能场景。 */
public final class EasyExcelBenchmarkRunner {

    private static final long XLS_DATA_ROWS_PER_SHEET = 65_535L;

    private EasyExcelBenchmarkRunner() {
    }

    public static void main(String[] args) throws Exception {
        Map<String, String> arguments = parseArguments(args);
        BenchmarkSpec spec = BenchmarkSpec.read(Paths.get(required(arguments, "--spec")));
        spec.validate();
        BenchmarkScenario scenario = spec.scenario(required(arguments, "--scenario"));
        long rows = Long.parseLong(required(arguments, "--rows"));
        int workers = Integer.parseInt(arguments.getOrDefault("--workers", "1"));
        String temperature = arguments.getOrDefault("--temperature", "cold");
        int warmups = Integer.parseInt(arguments.getOrDefault("--warmups", "0"));
        validateTemperature(temperature, warmups);
        Path input = arguments.containsKey("--input") ? Paths.get(arguments.get("--input")) : null;
        Path output = arguments.containsKey("--output") ? Paths.get(arguments.get("--output")) : null;
        validatePaths(scenario, input, output);

        for (int warmup = 0; warmup < warmups; warmup++) {
            Path warmupOutput = Objects.isNull(output) ? null : warmupPath(output, warmup);
            execute(
                scenario,
                input,
                Objects.isNull(warmupOutput) ? output : warmupOutput,
                rows,
                spec.getBatchSize());
            if (Objects.nonNull(warmupOutput)) {
                Files.deleteIfExists(warmupOutput);
            }
        }

        // Warm-up objects are reclaimed before the measured operation. Rust drops
        // the corresponding temporary values when execute() returns; an explicit
        // Java collection keeps both implementations at the same post-warm-up
        // lifecycle boundary. This collection is outside all reported operation
        // latency and GC counters.
        if (warmups > 0) {
            System.gc();
            System.runFinalization();
        }

        resetHeapPeaks();
        long cpuBefore = processCpuTime();
        long gcCountBefore = gcCount();
        long gcTimeBefore = gcTimeMillis();
        long started = System.nanoTime();
        BenchmarkOperationResult operation = execute(scenario, input, output, rows, spec.getBatchSize());
        long wallTime = System.nanoTime() - started;
        long cpuTime = Math.max(0L, processCpuTime() - cpuBefore);
        String expected = BenchmarkChecksum.expected(rows);
        boolean success = operation.getObservedRows() == rows && Objects.equals(operation.getChecksum(), expected);

        JSONObject result = resultJson(
            scenario, temperature, rows, workers, wallTime, cpuTime,
            Math.max(0L, gcCount() - gcCountBefore),
            Math.max(0L, gcTimeMillis() - gcTimeBefore),
            operation, spec.getSha256(), success);
        System.out.println(result.toJSONString());
        if (!success) {
            throw new IllegalStateException("benchmark correctness check failed");
        }
    }

    private static BenchmarkOperationResult execute(
        BenchmarkScenario scenario,
        Path input,
        Path output,
        long rows,
        int batchSize) throws Exception {
        switch (scenario.getOperation()) {
            case "read":
                return read(scenario, Objects.requireNonNull(input));
            case "write":
                return write(scenario, Objects.requireNonNull(output), rows, batchSize);
            case "roundtrip":
                return roundtrip(scenario, Objects.requireNonNull(input), Objects.requireNonNull(output));
            default:
                throw new IllegalArgumentException("unsupported operation " + scenario.getOperation());
        }
    }

    private static BenchmarkOperationResult roundtrip(
        BenchmarkScenario scenario,
        Path input,
        Path output) throws Exception {
        if (!"xlsx".equals(scenario.getFormat()) || !"workbook".equals(scenario.getMode())) {
            throw new IllegalArgumentException("v1 roundtrip requires XLSX Workbook Mode");
        }
        if (Objects.nonNull(output.getParent())) {
            Files.createDirectories(output.getParent());
        }
        try (java.io.InputStream stream = Files.newInputStream(input);
             XSSFWorkbook workbook = new XSSFWorkbook(stream);
             java.io.OutputStream target = Files.newOutputStream(output)) {
            workbook.getProperties().getCoreProperties().setTitle("easyexcel-benchmark-roundtrip");
            workbook.write(target);
        }
        try (java.io.InputStream stream = Files.newInputStream(output);
             XSSFWorkbook workbook = new XSSFWorkbook(stream)) {
            String title = workbook.getProperties().getCoreProperties().getTitle();
            if (!Objects.equals(title, "easyexcel-benchmark-roundtrip")) {
                throw new IllegalStateException("roundtrip metadata marker was not preserved");
            }
        }
        return read(scenario, output);
    }

    private static BenchmarkOperationResult write(
        BenchmarkScenario scenario,
        Path path,
        long rows,
        int batchSize) throws Exception {
        if (Objects.nonNull(path.getParent())) {
            Files.createDirectories(path.getParent());
        }
        File file = path.toFile();
        if ("full".equals(scenario.getMemory())) {
            List<BenchmarkRow> all = new ArrayList<>(Math.toIntExact(rows));
            for (long id = 0; id < rows; id++) {
                all.add(BenchmarkRow.fromId(id));
            }
            EasyExcel.write(file, BenchmarkRow.class).sheet("Data").doWrite(all);
        } else {
            try (ExcelWriter writer = EasyExcel.write(file, BenchmarkRow.class).build()) {
                long sheetCapacity = "xls".equals(scenario.getFormat()) ? XLS_DATA_ROWS_PER_SHEET : Math.max(1L, rows);
                int sheetIndex = 0;
                for (long sheetStart = 0; sheetStart < rows; sheetStart += sheetCapacity) {
                    long sheetEnd = Math.min(rows, sheetStart + sheetCapacity);
                    String sheetName = "xls".equals(scenario.getFormat()) ? "Data-" + (++sheetIndex) : "Data";
                    WriteSheet sheet = EasyExcel.writerSheet(sheetName).build();
                    for (long start = sheetStart; start < sheetEnd; start += batchSize) {
                        int count = Math.toIntExact(Math.min(batchSize, sheetEnd - start));
                        List<BenchmarkRow> batch = new ArrayList<>(count);
                        for (long id = start; id < start + count; id++) {
                            batch.add(BenchmarkRow.fromId(id));
                        }
                        writer.write(batch, sheet);
                    }
                }
            }
        }
        return new BenchmarkOperationResult(rows, BenchmarkChecksum.expected(rows), Files.size(path));
    }

    private static BenchmarkOperationResult read(BenchmarkScenario scenario, Path path) throws Exception {
        if ("workbook".equals(scenario.getMode())) {
            List<BenchmarkRow> rows = !"xlsx".equals(scenario.getFormat())
                ? EasyExcel.read(path.toFile(), BenchmarkRow.class, null).doReadAllSync()
                : EasyExcel.read(path.toFile(), BenchmarkRow.class, null).sheet("Data").doReadSync();
            BenchmarkChecksum checksum = new BenchmarkChecksum();
            for (BenchmarkRow row : rows) {
                checksum.update(row);
            }
            return new BenchmarkOperationResult(rows.size(), checksum.finish(), Files.size(path));
        }
        EventListener listener = new EventListener();
        if (!"xlsx".equals(scenario.getFormat())) {
            EasyExcel.read(path.toFile(), BenchmarkRow.class, listener).doReadAll();
        } else {
            EasyExcel.read(path.toFile(), BenchmarkRow.class, listener).sheet("Data").doRead();
        }
        return new BenchmarkOperationResult(listener.getRows(), listener.finishChecksum(), Files.size(path));
    }

    private static JSONObject resultJson(
        BenchmarkScenario scenario,
        String temperature,
        long rows,
        int workers,
        long wallTime,
        long cpuTime,
        long gcCount,
        long gcTimeMillis,
        BenchmarkOperationResult operation,
        String specSha256,
        boolean success) {
        double seconds = wallTime / 1_000_000_000d;
        JSONObject correctness = new JSONObject();
        correctness.put("observed_rows", operation.getObservedRows());
        correctness.put("checksum", operation.getChecksum());
        correctness.put(
            "rereadable", List.of("read", "roundtrip").contains(scenario.getOperation()));
        JSONObject environment = new JSONObject();
        environment.put("git_sha", System.getProperty("easyexcel.git.sha", "unknown"));
        environment.put("runtime", System.getProperty("java.runtime.version"));
        environment.put("os", System.getProperty("os.name"));
        environment.put("arch", System.getProperty("os.arch"));
        environment.put("spec_sha256", specSha256);
        JSONObject result = new JSONObject();
        result.put("schema_version", 1);
        result.put("implementation", "java");
        result.put("phase", "single");
        result.put("temperature", temperature);
        result.put("scenario_id", scenario.getId());
        result.put("fixture_origin", null);
        result.put("input_sha256", null);
        result.put("operation", scenario.getOperation());
        result.put("rows", rows);
        result.put("cells", rows * 4L);
        result.put("wall_time_ns", wallTime);
        result.put("process_wall_time_ns", null);
        result.put("cpu_user_time_ns", cpuTime);
        result.put("cpu_system_time_ns", null);
        result.put("rows_per_second", rows / seconds);
        result.put("cells_per_second", rows * 4d / seconds);
        result.put("mib_per_second", operation.getFileSizeBytes() / 1048576d / seconds);
        result.put("peak_rss_bytes", null);
        result.put("java_heap_peak_bytes", heapPeakBytes());
        result.put("gc_count", gcCount);
        result.put("gc_time_ns", gcTimeMillis * 1_000_000L);
        result.put("gc_max_pause_ns", null);
        result.put("allocator_allocations", null);
        result.put("allocator_peak_bytes", null);
        result.put("temporary_disk_peak_bytes", null);
        result.put("file_size_bytes", operation.getFileSizeBytes());
        result.put("logical_payload_bytes", BenchmarkChecksum.logicalPayloadBytes(rows));
        result.put(
            "total_written_bytes",
            List.of("write", "roundtrip").contains(scenario.getOperation())
                ? operation.getFileSizeBytes() : null);
        result.put("worker_count", workers);
        result.put("trial", null);
        result.put("worker_id", null);
        result.put("success", success);
        result.put("errors", success ? 0 : 1);
        result.put("correctness", correctness);
        result.put("environment", environment);
        return result;
    }

    private static Map<String, String> parseArguments(String[] args) {
        if (args.length % 2 != 0) {
            throw new IllegalArgumentException("arguments must use --key value pairs");
        }
        Map<String, String> values = new HashMap<>();
        for (int index = 0; index < args.length; index += 2) {
            values.put(args[index], args[index + 1]);
        }
        return values;
    }

    private static String required(Map<String, String> values, String name) {
        String value = values.get(name);
        if (!StringUtils.hasText(value)) {
            throw new IllegalArgumentException("missing required argument " + name);
        }
        return value;
    }

    private static void validateTemperature(String temperature, int warmups) {
        if (!List.of("cold", "steady").contains(temperature)) {
            throw new IllegalArgumentException("--temperature must be cold or steady");
        }
        if ("cold".equals(temperature) && warmups != 0) {
            throw new IllegalArgumentException("cold measurements must not execute warmups");
        }
    }

    private static void validatePaths(BenchmarkScenario scenario, Path input, Path output) {
        switch (scenario.getOperation()) {
            case "read":
                Objects.requireNonNull(input, "read scenario requires --input");
                return;
            case "write":
                Objects.requireNonNull(output, "write scenario requires --output");
                return;
            case "roundtrip":
                Objects.requireNonNull(input, "roundtrip scenario requires --input");
                Objects.requireNonNull(output, "roundtrip scenario requires --output");
                return;
            default:
                throw new IllegalArgumentException("unsupported operation " + scenario.getOperation());
        }
    }

    private static Path warmupPath(Path output, int warmup) {
        String fileName = output.getFileName().toString();
        int extension = fileName.lastIndexOf('.');
        String stem = extension < 0 ? fileName : fileName.substring(0, extension);
        String suffix = extension < 0 ? "" : fileName.substring(extension);
        return output.resolveSibling(stem + ".warmup-" + warmup + suffix);
    }

    private static void resetHeapPeaks() {
        for (MemoryPoolMXBean pool : ManagementFactory.getMemoryPoolMXBeans()) {
            pool.resetPeakUsage();
        }
    }

    private static long processCpuTime() {
        java.lang.management.OperatingSystemMXBean bean = ManagementFactory.getOperatingSystemMXBean();
        if (bean instanceof com.sun.management.OperatingSystemMXBean) {
            return ((com.sun.management.OperatingSystemMXBean)bean).getProcessCpuTime();
        }
        return 0L;
    }

    private static long heapPeakBytes() {
        long peak = 0L;
        for (MemoryPoolMXBean pool : ManagementFactory.getMemoryPoolMXBeans()) {
            if (pool.getType() == java.lang.management.MemoryType.HEAP
                && Objects.nonNull(pool.getPeakUsage())) {
                peak += Math.max(0L, pool.getPeakUsage().getUsed());
            }
        }
        return peak;
    }

    private static long gcCount() {
        long count = 0L;
        for (GarbageCollectorMXBean collector : ManagementFactory.getGarbageCollectorMXBeans()) {
            count += Math.max(0L, collector.getCollectionCount());
        }
        return count;
    }

    private static long gcTimeMillis() {
        long millis = 0L;
        for (GarbageCollectorMXBean collector : ManagementFactory.getGarbageCollectorMXBeans()) {
            millis += Math.max(0L, collector.getCollectionTime());
        }
        return millis;
    }

    private static final class EventListener implements ReadListener<BenchmarkRow> {

        private long rows;
        private final BenchmarkChecksum checksum = new BenchmarkChecksum();

        @Override
        public void invoke(BenchmarkRow data, AnalysisContext context) {
            rows++;
            checksum.update(data);
        }

        @Override
        public void doAfterAllAnalysed(AnalysisContext context) {
            // 无需保留行，Event Mode 在回调返回后释放数据。
        }

        public long getRows() {
            return rows;
        }

        public String finishChecksum() {
            return checksum.finish();
        }
    }
}
