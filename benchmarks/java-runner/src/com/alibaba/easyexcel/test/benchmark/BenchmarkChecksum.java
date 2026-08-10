package com.alibaba.easyexcel.test.benchmark;

import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.time.Instant;
import java.time.ZoneOffset;
import java.time.format.DateTimeFormatter;

/** 跨 Java/Rust 稳定的规范行 SHA-256。 */
public final class BenchmarkChecksum {

    private static final DateTimeFormatter DATE_FORMAT = DateTimeFormatter.ISO_LOCAL_DATE;

    private final MessageDigest digest;

    public BenchmarkChecksum() {
        try {
            digest = MessageDigest.getInstance("SHA-256");
        } catch (NoSuchAlgorithmException exception) {
            throw new IllegalStateException("SHA-256 is required by the Java runtime", exception);
        }
    }

    public void update(BenchmarkRow row) {
        String date = DATE_FORMAT.format(
            Instant.ofEpochMilli(row.getDate().getTime()).atZone(ZoneOffset.UTC).toLocalDate());
        String canonical = String.format(
            java.util.Locale.ROOT,
            "%d\t%s\t%s\t%016x%n",
            row.getId(),
            row.getName(),
            date,
            Double.doubleToLongBits(row.getScore()));
        digest.update(canonical.getBytes(StandardCharsets.UTF_8));
    }

    public String finish() {
        byte[] bytes = digest.digest();
        StringBuilder result = new StringBuilder(bytes.length * 2);
        for (byte value : bytes) {
            result.append(String.format(java.util.Locale.ROOT, "%02x", value & 0xff));
        }
        return result.toString();
    }

    public static String expected(long rows) {
        BenchmarkChecksum checksum = new BenchmarkChecksum();
        for (long id = 0; id < rows; id++) {
            checksum.update(BenchmarkRow.fromId(id));
        }
        return checksum.finish();
    }

    public static long logicalPayloadBytes(long rows) {
        long digitBytes = 0L;
        long lower = 0L;
        long upper = 10L;
        long digits = 1L;
        while (lower < rows) {
            long end = Math.min(rows, upper);
            digitBytes = Math.addExact(digitBytes, Math.multiplyExact(end - lower, digits));
            lower = end;
            if (upper > Long.MAX_VALUE / 10L) {
                digitBytes = Math.addExact(digitBytes, Math.multiplyExact(rows - lower, digits + 1L));
                break;
            }
            upper *= 10L;
            digits++;
        }
        return Math.addExact(Math.multiplyExact(rows, 34L), Math.multiplyExact(digitBytes, 2L));
    }
}
