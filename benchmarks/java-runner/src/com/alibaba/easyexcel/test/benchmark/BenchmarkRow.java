package com.alibaba.easyexcel.test.benchmark;

import java.util.Date;

import com.alibaba.excel.annotation.ExcelProperty;
import com.alibaba.excel.annotation.format.DateTimeFormat;

/** Java/Rust 共享性能契约中的四列表格行。 */
public class BenchmarkRow {

    @ExcelProperty(value = "ID", index = 0)
    private Long id;

    @ExcelProperty(value = "Name", index = 1)
    private String name;

    @ExcelProperty(value = "Date", index = 2)
    @DateTimeFormat("yyyy-MM-dd")
    private Date date;

    @ExcelProperty(value = "Score", index = 3)
    private Double score;

    public static BenchmarkRow fromId(long id) {
        BenchmarkRow row = new BenchmarkRow();
        row.setId(id);
        row.setName("row-" + id);
        row.setDate(new Date(1704067200000L + Math.floorMod(id, 28L) * 86_400_000L));
        row.setScore(id * 0.5d);
        return row;
    }

    public Long getId() {
        return id;
    }

    public void setId(Long id) {
        this.id = id;
    }

    public String getName() {
        return name;
    }

    public void setName(String name) {
        this.name = name;
    }

    public Date getDate() {
        return date;
    }

    public void setDate(Date date) {
        this.date = date;
    }

    public Double getScore() {
        return score;
    }

    public void setScore(Double score) {
        this.score = score;
    }
}
