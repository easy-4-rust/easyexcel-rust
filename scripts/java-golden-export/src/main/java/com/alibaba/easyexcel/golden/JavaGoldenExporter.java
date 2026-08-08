package com.alibaba.easyexcel.golden;

import java.io.File;
import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.math.BigDecimal;
import java.math.BigInteger;
import java.nio.charset.Charset;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.text.SimpleDateFormat;
import java.time.LocalDate;
import java.time.LocalDateTime;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.Date;
import java.util.HashMap;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.concurrent.atomic.AtomicInteger;

import com.alibaba.excel.EasyExcel;
import com.alibaba.excel.EasyExcelFactory;
import com.alibaba.excel.ExcelWriter;
import com.alibaba.excel.context.AnalysisContext;
import com.alibaba.excel.event.AnalysisEventListener;
import com.alibaba.excel.enums.CellDataTypeEnum;
import com.alibaba.excel.enums.ReadDefaultReturnEnum;
import com.alibaba.excel.enums.WriteDirectionEnum;
import com.alibaba.excel.metadata.data.FormulaData;
import com.alibaba.excel.metadata.data.ReadCellData;
import com.alibaba.excel.metadata.data.WriteCellData;
import com.alibaba.excel.read.metadata.ReadSheet;
import com.alibaba.excel.read.metadata.ReadWorkbook;
import com.alibaba.excel.support.ExcelTypeEnum;
import com.alibaba.excel.util.DateUtils;
import com.alibaba.excel.write.ExcelBuilder;
import com.alibaba.excel.write.ExcelBuilderImpl;
import com.alibaba.excel.analysis.ExcelAnalyser;
import com.alibaba.excel.analysis.ExcelAnalyserImpl;
import com.alibaba.excel.analysis.csv.CsvExcelReadExecutor;
import com.alibaba.excel.analysis.v03.XlsListSheetListener;
import com.alibaba.excel.analysis.v03.XlsRecordHandler;
import com.alibaba.excel.analysis.v03.XlsSaxAnalyser;
import com.alibaba.excel.analysis.v03.IgnorableXlsRecordHandler;
import com.alibaba.excel.analysis.v03.handlers.AbstractXlsRecordHandler;
import com.alibaba.excel.analysis.v03.handlers.BoundSheetRecordHandler;
import com.alibaba.excel.analysis.v03.handlers.BlankRecordHandler;
import com.alibaba.excel.analysis.v03.handlers.BoolErrRecordHandler;
import com.alibaba.excel.analysis.v03.handlers.EofRecordHandler;
import com.alibaba.excel.analysis.v03.handlers.FormulaRecordHandler;
import com.alibaba.excel.analysis.v03.handlers.IndexRecordHandler;
import com.alibaba.excel.analysis.v03.handlers.LabelRecordHandler;
import com.alibaba.excel.analysis.v03.handlers.LabelSstRecordHandler;
import com.alibaba.excel.analysis.v03.handlers.MergeCellsRecordHandler;
import com.alibaba.excel.analysis.v03.handlers.NoteRecordHandler;
import com.alibaba.excel.analysis.v03.handlers.NumberRecordHandler;
import com.alibaba.excel.analysis.v03.handlers.ObjRecordHandler;
import com.alibaba.excel.analysis.v03.handlers.RkRecordHandler;
import com.alibaba.excel.analysis.v03.handlers.SstRecordHandler;
import com.alibaba.excel.analysis.v03.handlers.StringRecordHandler;
import com.alibaba.excel.context.csv.DefaultCsvReadContext;
import com.alibaba.excel.context.xls.DefaultXlsReadContext;
import com.alibaba.excel.context.xlsx.DefaultXlsxReadContext;
import com.alibaba.excel.enums.CellExtraTypeEnum;
import com.alibaba.excel.exception.ExcelAnalysisStopException;
import com.alibaba.excel.write.merge.LoopMergeStrategy;
import com.alibaba.excel.write.metadata.WriteSheet;
import com.alibaba.excel.write.metadata.WriteTable;
import com.alibaba.excel.write.metadata.WriteWorkbook;
import com.alibaba.excel.write.metadata.fill.FillConfig;
import com.alibaba.excel.write.metadata.style.WriteCellStyle;
import com.alibaba.excel.write.metadata.style.WriteFont;
import org.apache.poi.hssf.record.BoundSheetRecord;
import org.apache.poi.hssf.record.BlankRecord;
import org.apache.poi.hssf.record.BoolErrRecord;
import org.apache.poi.hssf.record.CommonObjectDataSubRecord;
import org.apache.poi.hssf.record.EOFRecord;
import org.apache.poi.hssf.record.FormulaRecord;
import org.apache.poi.hssf.record.IndexRecord;
import org.apache.poi.hssf.record.LabelRecord;
import org.apache.poi.hssf.record.LabelSSTRecord;
import org.apache.poi.hssf.record.MergeCellsRecord;
import org.apache.poi.hssf.record.NoteRecord;
import org.apache.poi.hssf.record.NumberRecord;
import org.apache.poi.hssf.record.ObjRecord;
import org.apache.poi.hssf.record.RecordFactory;
import org.apache.poi.hssf.record.RecordInputStream;
import org.apache.poi.hssf.record.SSTRecord;
import org.apache.poi.hssf.record.RKRecord;
import org.apache.poi.hssf.record.StringRecord;
import org.apache.poi.hssf.record.UnknownRecord;
import org.apache.poi.hssf.record.common.UnicodeString;
import org.apache.poi.ss.util.CellRangeAddress;
import org.apache.poi.poifs.filesystem.POIFSFileSystem;
import com.alibaba.excel.write.style.HorizontalCellStyleStrategy;
import com.alibaba.excel.write.style.column.SimpleColumnWidthStyleStrategy;
import com.alibaba.excel.write.style.row.SimpleRowHeightStyleStrategy;
import com.alibaba.fastjson2.JSON;
import com.alibaba.fastjson2.JSONWriter;

import org.apache.poi.ss.usermodel.FillPatternType;
import org.apache.poi.ss.usermodel.IndexedColors;

/**
 * Export Java EasyExcel read/write results as JSON golden files for easyexcel-rs.
 *
 * <p>Covers fixture reads (compatibility / BOM / demo / simple / converter / multi-sheet /
 * dataformat / template / xls) and Java-written artifacts (simple / converter / fill / style /
 * annotation / exclude-include / head / sort / encrypt / cache / celldata / charset / exception /
 * handler / large-sample / nomodel / noncamel / parameter / repetition / skip /
 * complex-head(+xls) / annotation-index(+xls) / list-head(+xls) / converter-write-xls-csv /
 * large-sample(+xls/csv) / celldata(+xls/csv full) / parameter-xls / no-head(+xls/csv) /
 * fill-horizontal(+xls) / fill-by-name).
 *
 * <p>Args:
 * <ol>
 *   <li>fixturesDir — root of fixture files (e.g. crates/easyexcel/tests/fixtures)</li>
 *   <li>outDir — golden output directory (e.g. crates/easyexcel/tests/golden)</li>
 * </ol>
 */
public final class JavaGoldenExporter {

    private JavaGoldenExporter() {
    }

    /**
     * Entry: export all registered fixtures / write scenarios to {@code outDir}.
     *
     * @param args fixturesDir outDir
     * @throws Exception on IO or EasyExcel failures
     */
    public static void main(String[] args) throws Exception {
        if (args.length < 2) {
            System.err.println("Usage: JavaGoldenExporter <fixturesDir> <outDir>");
            System.exit(2);
        }
        Path fixturesDir = Paths.get(args[0]).toAbsolutePath().normalize();
        Path outDir = Paths.get(args[1]).toAbsolutePath().normalize();
        Files.createDirectories(outDir);
        Path artifactDir = outDir.resolve("artifacts");
        Files.createDirectories(artifactDir);

        writeFacadeApiContract(outDir);

        List<ExportSpec> specs = buildSpecs(fixturesDir, artifactDir);
        writeExcelReaderContract(artifactDir.resolve("simple_data.xlsx"), outDir);
        writeExcelWriterContract(artifactDir, outDir);
        writeExcelBuilderContract(artifactDir, outDir);
        writeExcelAnalyserContract(artifactDir, outDir);
        for (ExportSpec spec : specs) {
            Map<String, Object> payload = exportOne(spec);
            Path out = outDir.resolve(spec.outName);
            String json = JSON.toJSONString(payload, JSONWriter.Feature.PrettyFormat);
            Files.write(out, (json + "\n").getBytes(StandardCharsets.UTF_8));
            System.out.println("Wrote " + out + " (row_count=" + payload.get("row_count") + ")");
        }
    }

    /**
     * Export lifecycle and fluent-return observations for Java's public {@code ExcelWriter} API.
     *
     * @param artifactDir output directory for throw-away workbook probes
     * @param outDir golden output directory
     * @throws Exception when a write/fill operation or contract write fails
     */
    private static void writeExcelWriterContract(Path artifactDir, Path outDir) throws Exception {
        Map<String, Object> payload = new LinkedHashMap<String, Object>();
        payload.put("authority", "com.alibaba:easyexcel:4.0.3");

        Path writeOutput = artifactDir.resolve("excel_writer_api.xlsx");
        WriteWorkbook directWorkbook = new WriteWorkbook();
        directWorkbook.setFile(writeOutput.toFile());
        directWorkbook.setExcelType(ExcelTypeEnum.XLSX);
        directWorkbook.setNeedHead(Boolean.FALSE);
        ExcelWriter writer = new ExcelWriter(directWorkbook);
        payload.put("direct_constructor", writer.getClass().getName());

        WriteSheet writeSheet = EasyExcel.writerSheet(Integer.valueOf(0), "Data")
            .needHead(Boolean.FALSE)
            .build();
        WriteTable writeTable = EasyExcel.writerTable(Integer.valueOf(0))
            .needHead(Boolean.FALSE)
            .build();
        List<List<String>> rows = Collections.singletonList(Collections.singletonList("row"));
        AtomicInteger writeSupplierCalls = new AtomicInteger();
        payload.put("write_collection_returns_self", writer == writer.write(rows, writeSheet));
        payload.put("write_supplier_returns_self", writer == writer.write(() -> {
            writeSupplierCalls.incrementAndGet();
            return rows;
        }, writeSheet));
        payload.put("write_table_returns_self", writer == writer.write(rows, writeSheet, writeTable));
        payload.put("write_supplier_table_returns_self", writer == writer.write(() -> {
            writeSupplierCalls.incrementAndGet();
            return rows;
        }, writeSheet, writeTable));
        payload.put("write_supplier_calls", Integer.valueOf(writeSupplierCalls.get()));
        Object firstContext = writer.writeContext();
        Object secondContext = writer.writeContext();
        payload.put("write_context_class", firstContext.getClass().getName());
        payload.put("write_context_same", Boolean.valueOf(firstContext == secondContext));
        writer.finish();
        writer.close();
        payload.put("finish_then_close", Boolean.TRUE);

        Path template = artifactDir.resolve("excel_writer_fill_template.xlsx");
        EasyExcel.write(template.toFile())
            .head(Collections.singletonList(Collections.singletonList("value")))
            .sheet("Fill")
            .doWrite(Collections.singletonList(Collections.singletonList("{name}")));
        Path fillOutput = artifactDir.resolve("excel_writer_fill_api.xlsx");
        ExcelWriter fillWriter = EasyExcel.write(fillOutput.toFile())
            .withTemplate(template.toFile())
            .build();
        WriteSheet fillSheet = EasyExcel.writerSheet("Fill").build();
        Map<String, Object> fillData = Collections.<String, Object>singletonMap("name", "filled");
        FillConfig fillConfig = FillConfig.builder()
            .direction(WriteDirectionEnum.VERTICAL)
            .forceNewRow(Boolean.FALSE)
            .autoStyle(Boolean.TRUE)
            .build();
        AtomicInteger fillSupplierCalls = new AtomicInteger();
        payload.put("fill_object_returns_self", fillWriter == fillWriter.fill(fillData, fillSheet));
        payload.put("fill_config_returns_self", fillWriter == fillWriter.fill(fillData, fillConfig, fillSheet));
        payload.put("fill_supplier_returns_self", fillWriter == fillWriter.fill(() -> {
            fillSupplierCalls.incrementAndGet();
            return fillData;
        }, fillSheet));
        payload.put("fill_supplier_config_returns_self", fillWriter == fillWriter.fill(() -> {
            fillSupplierCalls.incrementAndGet();
            return fillData;
        }, fillConfig, fillSheet));
        payload.put("fill_supplier_calls", Integer.valueOf(fillSupplierCalls.get()));
        fillWriter.finish();

        Path out = outDir.resolve("excel_writer_lifecycle.contract.json");
        String json = JSON.toJSONString(payload, JSONWriter.Feature.PrettyFormat);
        Files.write(out, (json + "\n").getBytes(StandardCharsets.UTF_8));
        System.out.println("Wrote " + out);
    }

    /**
     * Export observable semantics for every public {@code ExcelBuilder} method.
     *
     * @param artifactDir output directory for workbook probes
     * @param outDir golden output directory
     * @throws Exception when a builder operation or contract write fails
     */
    @SuppressWarnings("deprecation")
    private static void writeExcelBuilderContract(Path artifactDir, Path outDir) throws Exception {
        Map<String, Object> payload = new LinkedHashMap<String, Object>();
        payload.put("authority", "com.alibaba:easyexcel:4.0.3");

        Path writeOutput = artifactDir.resolve("excel_builder_api.xlsx");
        WriteWorkbook workbook = new WriteWorkbook();
        workbook.setFile(writeOutput.toFile());
        workbook.setExcelType(ExcelTypeEnum.XLSX);
        workbook.setNeedHead(Boolean.FALSE);
        ExcelBuilder builder = new ExcelBuilderImpl(workbook);
        payload.put("implementation_class", builder.getClass().getName());

        WriteSheet sheet = EasyExcel.writerSheet(Integer.valueOf(0), "Builder")
            .needHead(Boolean.FALSE)
            .build();
        WriteTable table = EasyExcel.writerTable(Integer.valueOf(0))
            .needHead(Boolean.FALSE)
            .build();
        builder.addContent(
            Collections.singletonList(Collections.singletonList("builder-two-arg")),
            sheet);
        builder.addContent(
            Collections.singletonList(Collections.singletonList("builder-three-arg")),
            sheet,
            table);
        payload.put("add_content_two_arg", Boolean.TRUE);
        payload.put("add_content_three_arg", Boolean.TRUE);
        Object firstContext = builder.writeContext();
        Object secondContext = builder.writeContext();
        payload.put("write_context_class", firstContext.getClass().getName());
        payload.put("write_context_same", Boolean.valueOf(firstContext == secondContext));
        builder.merge(0, 0, 0, 1);
        builder.finish(false);
        payload.put("finish_false_output_exists", Boolean.valueOf(Files.exists(writeOutput)));
        try (org.apache.poi.ss.usermodel.Workbook observed =
                 org.apache.poi.ss.usermodel.WorkbookFactory.create(Files.newInputStream(writeOutput))) {
            org.apache.poi.ss.usermodel.Sheet observedSheet = observed.getSheet("Builder");
            payload.put("merge_after_add_range",
                observedSheet.getMergedRegion(0).formatAsString());
        }

        Path template = artifactDir.resolve("excel_builder_fill_template.xlsx");
        EasyExcel.write(template.toFile())
            .head(Collections.singletonList(Collections.singletonList("value")))
            .sheet("Fill")
            .doWrite(Collections.singletonList(Collections.singletonList("{name}")));
        Path fillOutput = artifactDir.resolve("excel_builder_fill_api.xlsx");
        WriteWorkbook fillWorkbook = new WriteWorkbook();
        fillWorkbook.setFile(fillOutput.toFile());
        fillWorkbook.setExcelType(ExcelTypeEnum.XLSX);
        fillWorkbook.setTemplateFile(template.toFile());
        ExcelBuilder fillBuilder = new ExcelBuilderImpl(fillWorkbook);
        fillBuilder.fill(
            Collections.<String, Object>singletonMap("name", "builder-filled"),
            FillConfig.builder().autoStyle(Boolean.TRUE).build(),
            EasyExcel.writerSheet("Fill").build());
        fillBuilder.merge(0, 0, 0, 1);
        fillBuilder.finish(false);
        payload.put("fill_output_exists", Boolean.valueOf(Files.exists(fillOutput)));
        try (org.apache.poi.ss.usermodel.Workbook observed =
                 org.apache.poi.ss.usermodel.WorkbookFactory.create(Files.newInputStream(fillOutput))) {
            payload.put("template_merge_after_fill_range",
                observed.getSheet("Fill").getMergedRegion(0).formatAsString());
        }

        Path exceptionalOutput = artifactDir.resolve("excel_builder_exception.xlsx");
        WriteWorkbook exceptionalWorkbook = new WriteWorkbook();
        exceptionalWorkbook.setFile(exceptionalOutput.toFile());
        exceptionalWorkbook.setExcelType(ExcelTypeEnum.XLSX);
        exceptionalWorkbook.setNeedHead(Boolean.FALSE);
        ExcelBuilder exceptionalBuilder = new ExcelBuilderImpl(exceptionalWorkbook);
        exceptionalBuilder.addContent(
            Collections.singletonList(Collections.singletonList("discarded")),
            sheet);
        exceptionalBuilder.finish(true);
        payload.put("finish_true_output_exists", Boolean.valueOf(Files.exists(exceptionalOutput)));
        payload.put("finish_true_output_size", Long.valueOf(Files.size(exceptionalOutput)));

        Path missingTemplateOutput = artifactDir.resolve("excel_builder_missing_template.xlsx");
        WriteWorkbook missingTemplateWorkbook = new WriteWorkbook();
        missingTemplateWorkbook.setFile(missingTemplateOutput.toFile());
        ExcelBuilder missingTemplateBuilder = new ExcelBuilderImpl(missingTemplateWorkbook);
        try {
            missingTemplateBuilder.fill(
                Collections.<String, Object>singletonMap("name", "unused"),
                new FillConfig(),
                EasyExcel.writerSheet("Sheet1").build());
            payload.put("fill_without_template_error", "<none>");
        } catch (RuntimeException error) {
            payload.put("fill_without_template_error", error.getMessage());
        }

        Path out = outDir.resolve("excel_builder_lifecycle.contract.json");
        String json = JSON.toJSONString(payload, JSONWriter.Feature.PrettyFormat);
        Files.write(out, (json + "\n").getBytes(StandardCharsets.UTF_8));
        System.out.println("Wrote " + out);
    }

    /**
     * Export direct {@code ExcelAnalyserImpl(ReadWorkbook)} lifecycle semantics.
     *
     * @param artifactDir output directory for the input workbook
     * @param outDir golden output directory
     * @throws Exception when construction, analysis, or contract writing fails
     */
    private static void writeExcelAnalyserContract(Path artifactDir, Path outDir) throws Exception {
        Path input = artifactDir.resolve("excel_analyser_api.xlsx");
        EasyExcel.write(input.toFile())
            .head(Collections.singletonList(Collections.singletonList("value")))
            .sheet("Analyser")
            .doWrite(Collections.singletonList(Collections.singletonList("analysed")));

        ReadWorkbook workbook = new ReadWorkbook();
        workbook.setFile(input.toFile());
        workbook.setExcelType(ExcelTypeEnum.XLSX);
        ExcelAnalyser analyser = new ExcelAnalyserImpl(workbook);
        Map<String, Object> payload = new LinkedHashMap<String, Object>();
        payload.put("authority", "com.alibaba:easyexcel:4.0.3");
        payload.put("implementation_class", analyser.getClass().getName());
        payload.put("executor_class", analyser.excelExecutor().getClass().getName());
        payload.put("executor_sheet_count", Integer.valueOf(analyser.excelExecutor().sheetList().size()));
        Object firstContext = analyser.analysisContext();
        Object secondContext = analyser.analysisContext();
        payload.put("analysis_context_class", firstContext.getClass().getName());
        payload.put("analysis_context_same", Boolean.valueOf(firstContext == secondContext));
        analyser.analysis(null, Boolean.TRUE);
        payload.put("analysis_all_succeeded", Boolean.TRUE);
        analyser.finish();
        analyser.finish();
        payload.put("finish_twice_succeeded", Boolean.TRUE);

        Path csvInput = artifactDir.resolve("excel_read_executor_api.csv");
        Files.write(csvInput, "value\ncsv-row\n".getBytes(StandardCharsets.UTF_8));
        ReadWorkbook csvWorkbook = new ReadWorkbook();
        csvWorkbook.setFile(csvInput.toFile());
        csvWorkbook.setExcelType(ExcelTypeEnum.CSV);
        csvWorkbook.setHeadRowNumber(Integer.valueOf(1));
        DefaultCsvReadContext csvContext =
            new DefaultCsvReadContext(csvWorkbook, ExcelTypeEnum.CSV);
        csvContext.csvReadWorkbookHolder().setReadAll(Boolean.TRUE);
        CsvExcelReadExecutor csvExecutor = new CsvExcelReadExecutor(csvContext);
        payload.put("csv_executor_class", csvExecutor.getClass().getName());
        payload.put("csv_executor_sheet_count", Integer.valueOf(csvExecutor.sheetList().size()));
        payload.put("csv_executor_sheet_no", csvExecutor.sheetList().get(0).getSheetNo());
        payload.put("csv_executor_sheet_name", csvExecutor.sheetList().get(0).getSheetName());
        csvExecutor.execute();
        payload.put("csv_executor_execute_succeeded", Boolean.TRUE);
        payload.put("csv_context_sheet_no", csvContext.csvReadSheetHolder().getSheetNo());
        payload.put("csv_context_parser_initialized",
            Boolean.valueOf(csvContext.csvReadWorkbookHolder().getCsvParser() != null));
        csvContext.csvReadWorkbookHolder().getCsvParser().close();

        ReadWorkbook listenerRecordWorkbook = new ReadWorkbook();
        listenerRecordWorkbook.setExcelType(ExcelTypeEnum.XLS);
        listenerRecordWorkbook.setAutoTrim(Boolean.TRUE);
        DefaultXlsReadContext listenerRecordContext =
            new DefaultXlsReadContext(listenerRecordWorkbook, ExcelTypeEnum.XLS);
        XlsListSheetListener recordListener = new XlsListSheetListener(listenerRecordContext);
        payload.put("xls_list_listener_class", recordListener.getClass().getName());
        payload.put("xls_list_listener_need_read_sheet",
            listenerRecordContext.xlsReadWorkbookHolder().getNeedReadSheet());
        int beforeKnownRecord = listenerRecordContext.xlsReadWorkbookHolder()
            .getBoundSheetRecordList().size();
        recordListener.processRecord(new BoundSheetRecord("Manual"));
        int afterKnownRecord = listenerRecordContext.xlsReadWorkbookHolder()
            .getBoundSheetRecordList().size();
        recordListener.processRecord(new NumberRecord());
        payload.put("xls_list_listener_known_record_delta",
            Integer.valueOf(afterKnownRecord - beforeKnownRecord));
        payload.put("xls_list_listener_unknown_record_ignored",
            Boolean.valueOf(listenerRecordContext.xlsReadWorkbookHolder()
                .getBoundSheetRecordList().size() == afterKnownRecord));

        XlsRecordHandler recordHandler = new BoundSheetRecordHandler();
        BoundSheetRecord handlerRecord = new BoundSheetRecord("HandlerSheet");
        payload.put("xls_record_handler_interface", Boolean.valueOf(XlsRecordHandler.class.isInterface()));
        payload.put("xls_record_handler_support",
            Boolean.valueOf(recordHandler.support(listenerRecordContext, handlerRecord)));
        payload.put("abstract_xls_record_handler_is_abstract",
            Boolean.valueOf(java.lang.reflect.Modifier.isAbstract(AbstractXlsRecordHandler.class.getModifiers())));
        payload.put("abstract_xls_record_handler_support_default",
            Boolean.valueOf(new BoundSheetRecordHandler().support(listenerRecordContext, handlerRecord)));
        int beforeHandlerRecord = listenerRecordContext.xlsReadWorkbookHolder()
            .getBoundSheetRecordList().size();
        recordHandler.processRecord(listenerRecordContext, handlerRecord);
        payload.put("xls_record_handler_process_delta",
            Integer.valueOf(listenerRecordContext.xlsReadWorkbookHolder()
                .getBoundSheetRecordList().size() - beforeHandlerRecord));
        payload.put("ignorable_xls_record_handler_marker",
            Boolean.valueOf(IgnorableXlsRecordHandler.class.isAssignableFrom(recordHandler.getClass())));

        listenerRecordContext.currentSheet(new ReadSheet(Integer.valueOf(0), "HandlerSheet"));
        BlankRecord blankRecord = new BlankRecord();
        blankRecord.setRow(2);
        blankRecord.setColumn((short)3);
        new BlankRecordHandler().processRecord(listenerRecordContext, blankRecord);
        payload.put("blank_record_handler_cell_present",
            Boolean.valueOf(listenerRecordContext.xlsReadSheetHolder().getCellMap().containsKey(Integer.valueOf(3))));

        BoolErrRecord boolRecord = new BoolErrRecord();
        boolRecord.setRow(4);
        boolRecord.setColumn((short)5);
        boolRecord.setValue(true);
        new BoolErrRecordHandler().processRecord(listenerRecordContext, boolRecord);
        payload.put("bool_err_record_handler_cell_present",
            Boolean.valueOf(listenerRecordContext.xlsReadSheetHolder().getCellMap().containsKey(Integer.valueOf(5))));

        IndexRecord indexRecord = new IndexRecord();
        indexRecord.setLastRowAdd1(77);
        new IndexRecordHandler().processRecord(listenerRecordContext, indexRecord);
        payload.put("index_record_handler_total_rows",
            listenerRecordContext.readSheetHolder().getApproximateTotalRowNumber());

        listenerRecordContext.readWorkbookHolder().getExtraReadSet().add(CellExtraTypeEnum.MERGE);
        MergeCellsRecordHandler mergeHandler = new MergeCellsRecordHandler();
        MergeCellsRecord mergeRecord = new MergeCellsRecord(
            new CellRangeAddress[] {new CellRangeAddress(0, 1, 0, 1)}, 0, 1);
        payload.put("merge_cells_record_handler_support",
            Boolean.valueOf(mergeHandler.support(listenerRecordContext, mergeRecord)));
        mergeHandler.processRecord(listenerRecordContext, mergeRecord);
        payload.put("merge_cells_record_handler_first_row",
            listenerRecordContext.xlsReadSheetHolder().getCellExtra().getFirstRowIndex());
        payload.put("merge_cells_record_handler_last_row",
            listenerRecordContext.xlsReadSheetHolder().getCellExtra().getLastRowIndex());
        payload.put("merge_cells_record_handler_first_column",
            listenerRecordContext.xlsReadSheetHolder().getCellExtra().getFirstColumnIndex());
        payload.put("merge_cells_record_handler_last_column",
            listenerRecordContext.xlsReadSheetHolder().getCellExtra().getLastColumnIndex());

        listenerRecordContext.readWorkbookHolder().getExtraReadSet().add(CellExtraTypeEnum.COMMENT);
        listenerRecordContext.xlsReadSheetHolder().getObjectCacheMap().put(Integer.valueOf(9), "note-text");
        NoteRecordHandler noteHandler = new NoteRecordHandler();
        NoteRecord noteRecord = new NoteRecord();
        noteRecord.setRow(3);
        noteRecord.setColumn(4);
        noteRecord.setShapeId(9);
        payload.put("note_record_handler_support",
            Boolean.valueOf(noteHandler.support(listenerRecordContext, noteRecord)));
        noteHandler.processRecord(listenerRecordContext, noteRecord);
        payload.put("note_record_handler_text",
            listenerRecordContext.xlsReadSheetHolder().getCellExtra().getText());
        payload.put("note_record_handler_row",
            listenerRecordContext.xlsReadSheetHolder().getCellExtra().getFirstRowIndex());
        payload.put("note_record_handler_column",
            listenerRecordContext.xlsReadSheetHolder().getCellExtra().getFirstColumnIndex());

        byte[] labelText = "label-text".getBytes(StandardCharsets.ISO_8859_1);
        byte[] labelBytes = new byte[4 + 9 + labelText.length];
        labelBytes[0] = 0x04;
        labelBytes[1] = 0x02;
        labelBytes[2] = (byte)(9 + labelText.length);
        labelBytes[4] = 5;
        labelBytes[6] = 6;
        labelBytes[10] = (byte)labelText.length;
        labelBytes[12] = 0;
        System.arraycopy(labelText, 0, labelBytes, 13, labelText.length);
        RecordInputStream labelInput = new RecordInputStream(new ByteArrayInputStream(labelBytes));
        labelInput.nextRecord();
        LabelRecord labelRecord = (LabelRecord)RecordFactory.createSingleRecord(labelInput);
        new LabelRecordHandler().processRecord(listenerRecordContext, labelRecord);
        ReadCellData<?> labelCell =
            (ReadCellData<?>)listenerRecordContext.xlsReadSheetHolder().getCellMap().get(Integer.valueOf(6));
        payload.put("label_record_handler_text", labelCell.getStringValue());
        payload.put("label_record_handler_row", labelCell.getRowIndex());
        payload.put("label_record_handler_column", labelCell.getColumnIndex());
        payload.put("label_record_handler_row_type",
            listenerRecordContext.xlsReadSheetHolder().getTempRowType().name());

        SSTRecord sstRecord = new SSTRecord();
        int sharedIndex = sstRecord.addString(new UnicodeString("  shared  "));
        new SstRecordHandler().processRecord(listenerRecordContext, sstRecord);
        payload.put("sst_record_handler_cache_present",
            Boolean.valueOf(listenerRecordContext.readWorkbookHolder().getReadCache() != null));
        payload.put("sst_record_handler_text",
            listenerRecordContext.readWorkbookHolder().getReadCache().get(sharedIndex));

        LabelSSTRecord labelSstRecord = new LabelSSTRecord();
        labelSstRecord.setRow(7);
        labelSstRecord.setColumn((short)8);
        labelSstRecord.setXFIndex((short)0);
        labelSstRecord.setSSTIndex(sharedIndex);
        new LabelSstRecordHandler().processRecord(listenerRecordContext, labelSstRecord);
        ReadCellData<?> labelSstCell =
            (ReadCellData<?>)listenerRecordContext.xlsReadSheetHolder().getCellMap().get(Integer.valueOf(8));
        payload.put("label_sst_record_handler_text", labelSstCell.getStringValue());
        payload.put("label_sst_record_handler_row", labelSstCell.getRowIndex());
        payload.put("label_sst_record_handler_column", labelSstCell.getColumnIndex());
        payload.put("label_sst_record_handler_row_type",
            listenerRecordContext.xlsReadSheetHolder().getTempRowType().name());

        listenerRecordContext.readWorkbookHolder().setReadCache(null);
        LabelSSTRecord missingSstRecord = new LabelSSTRecord();
        missingSstRecord.setRow(9);
        missingSstRecord.setColumn((short)10);
        missingSstRecord.setXFIndex((short)0);
        missingSstRecord.setSSTIndex(99);
        new LabelSstRecordHandler().processRecord(listenerRecordContext, missingSstRecord);
        ReadCellData<?> missingSstCell =
            (ReadCellData<?>)listenerRecordContext.xlsReadSheetHolder().getCellMap().get(Integer.valueOf(10));
        payload.put("label_sst_record_handler_missing_cache_empty",
            Boolean.valueOf(missingSstCell.getType() == CellDataTypeEnum.EMPTY));

        FormulaRecord stringFormulaRecord = new FormulaRecord();
        stringFormulaRecord.setRow(11);
        stringFormulaRecord.setColumn((short)12);
        stringFormulaRecord.setXFIndex((short)0);
        stringFormulaRecord.setCachedResultTypeString();
        new FormulaRecordHandler().processRecord(listenerRecordContext, stringFormulaRecord);
        ReadCellData<?> pendingFormulaCell =
            (ReadCellData<?>)listenerRecordContext.xlsReadSheetHolder().getTempCellData();
        payload.put("formula_record_handler_string_pending",
            Boolean.valueOf(pendingFormulaCell != null));
        payload.put("formula_record_handler_row", pendingFormulaCell.getRowIndex());
        payload.put("formula_record_handler_column", pendingFormulaCell.getColumnIndex());
        payload.put("formula_record_handler_type", pendingFormulaCell.getType().name());

        StringRecord formulaStringRecord = new StringRecord();
        formulaStringRecord.setString("  formula-text  ");
        new StringRecordHandler().processRecord(listenerRecordContext, formulaStringRecord);
        ReadCellData<?> completedFormulaCell =
            (ReadCellData<?>)listenerRecordContext.xlsReadSheetHolder().getCellMap().get(Integer.valueOf(12));
        payload.put("string_record_handler_text", completedFormulaCell.getStringValue());
        payload.put("string_record_handler_formula_completed",
            Boolean.valueOf(listenerRecordContext.xlsReadSheetHolder().getTempCellData() == null));
        payload.put("string_record_handler_cell_present",
            Boolean.valueOf(completedFormulaCell != null));

        int beforeOrphanString = listenerRecordContext.xlsReadSheetHolder().getCellMap().size();
        StringRecord orphanStringRecord = new StringRecord();
        orphanStringRecord.setString("orphan");
        new StringRecordHandler().processRecord(listenerRecordContext, orphanStringRecord);
        payload.put("string_record_handler_without_formula_ignored",
            Boolean.valueOf(listenerRecordContext.xlsReadSheetHolder().getCellMap().size() == beforeOrphanString));

        byte[] rkBytes = new byte[] {
            0x7E, 0x02, 10, 0,
            13, 0, 14, 0, 0, 0,
            (byte)0xAA, 0, 0, 0
        };
        RecordInputStream rkInput = new RecordInputStream(new ByteArrayInputStream(rkBytes));
        rkInput.nextRecord();
        RKRecord rkRecord = (RKRecord)RecordFactory.createSingleRecord(rkInput);
        new RkRecordHandler().processRecord(listenerRecordContext, rkRecord);
        ReadCellData<?> rkCell =
            (ReadCellData<?>)listenerRecordContext.xlsReadSheetHolder().getCellMap().get(Integer.valueOf(14));
        payload.put("rk_record_handler_cell_present", Boolean.valueOf(rkCell != null));
        payload.put("rk_record_handler_type", rkCell.getType().name());
        payload.put("rk_record_handler_row", rkCell.getRowIndex());
        payload.put("rk_record_handler_column", rkCell.getColumnIndex());

        CommonObjectDataSubRecord commentObject = new CommonObjectDataSubRecord();
        commentObject.setObjectType(CommonObjectDataSubRecord.OBJECT_TYPE_COMMENT);
        commentObject.setObjectId(15);
        ObjRecord objRecord = new ObjRecord();
        objRecord.addSubRecord(commentObject);
        new ObjRecordHandler().processRecord(listenerRecordContext, objRecord);
        payload.put("obj_record_handler_comment_object_id",
            listenerRecordContext.xlsReadSheetHolder().getTempObjectIndex());

        CommonObjectDataSubRecord rectangleObject = new CommonObjectDataSubRecord();
        rectangleObject.setObjectType(CommonObjectDataSubRecord.OBJECT_TYPE_RECTANGLE);
        rectangleObject.setObjectId(99);
        ObjRecord nonCommentObjRecord = new ObjRecord();
        nonCommentObjRecord.addSubRecord(rectangleObject);
        new ObjRecordHandler().processRecord(listenerRecordContext, nonCommentObjRecord);
        payload.put("obj_record_handler_non_comment_ignored",
            Boolean.valueOf(Integer.valueOf(15).equals(
                listenerRecordContext.xlsReadSheetHolder().getTempObjectIndex())));

        new EofRecordHandler().processRecord(listenerRecordContext, EOFRecord.instance);
        payload.put("eof_record_handler_sheet_ended",
            listenerRecordContext.readSheetHolder().getEnded());

        Path xlsInput = artifactDir.resolve("excel_analyser_api.xls");
        EasyExcel.write(xlsInput.toFile())
            .excelType(ExcelTypeEnum.XLS)
            .head(Collections.singletonList(Collections.singletonList("value")))
            .sheet("AnalyserXls")
            .doWrite(Collections.singletonList(Collections.singletonList("analysed")));
        ReadWorkbook xlsWorkbook = new ReadWorkbook();
        xlsWorkbook.setFile(xlsInput.toFile());
        xlsWorkbook.setExcelType(ExcelTypeEnum.XLS);
        DefaultXlsReadContext xlsContext =
            new DefaultXlsReadContext(xlsWorkbook, ExcelTypeEnum.XLS);
        XlsListSheetListener xlsListSheetListener = new XlsListSheetListener(xlsContext);
        try (POIFSFileSystem fileSystem = new POIFSFileSystem(xlsInput.toFile())) {
            xlsContext.xlsReadWorkbookHolder().setPoifsFileSystem(fileSystem);
            boolean stopped = false;
            try {
                xlsListSheetListener.execute();
            } catch (ExcelAnalysisStopException expected) {
                stopped = true;
            }
            payload.put("xls_list_listener_execute_stopped", Boolean.valueOf(stopped));
        }
        payload.put("xls_list_listener_execute_sheet_count",
            Integer.valueOf(xlsContext.xlsReadWorkbookHolder().getActualSheetDataList().size()));

        ReadWorkbook saxWorkbook = new ReadWorkbook();
        saxWorkbook.setFile(xlsInput.toFile());
        saxWorkbook.setExcelType(ExcelTypeEnum.XLS);
        DefaultXlsReadContext saxContext =
            new DefaultXlsReadContext(saxWorkbook, ExcelTypeEnum.XLS);
        saxContext.xlsReadWorkbookHolder().setReadAll(Boolean.TRUE);
        try (POIFSFileSystem fileSystem = new POIFSFileSystem(xlsInput.toFile())) {
            saxContext.xlsReadWorkbookHolder().setPoifsFileSystem(fileSystem);
            XlsSaxAnalyser saxAnalyser = new XlsSaxAnalyser(saxContext);
            payload.put("xls_sax_analyser_class", saxAnalyser.getClass().getName());
            payload.put("xls_sax_analyser_sheet_count", Integer.valueOf(saxAnalyser.sheetList().size()));
            saxAnalyser.execute();
            payload.put("xls_sax_analyser_execute_succeeded", Boolean.TRUE);
            NumberRecord numberRecord = new NumberRecord();
            numberRecord.setRow(6);
            numberRecord.setColumn((short)7);
            numberRecord.setXFIndex((short)0);
            numberRecord.setValue(12.5D);
            new NumberRecordHandler().processRecord(saxContext, numberRecord);
            payload.put("number_record_handler_cell_present",
                Boolean.valueOf(saxContext.xlsReadSheetHolder().getCellMap().containsKey(Integer.valueOf(7))));
            int saxBeforeKnownRecord = saxContext.xlsReadWorkbookHolder().getBoundSheetRecordList().size();
            saxAnalyser.processRecord(new BoundSheetRecord("ManualSax"));
            int saxAfterKnownRecord = saxContext.xlsReadWorkbookHolder().getBoundSheetRecordList().size();
            saxAnalyser.processRecord(new UnknownRecord(0x0FFF, new byte[0]));
            payload.put("xls_sax_analyser_known_record_delta",
                Integer.valueOf(saxAfterKnownRecord - saxBeforeKnownRecord));
            payload.put("xls_sax_analyser_unknown_record_ignored",
                Boolean.valueOf(saxContext.xlsReadWorkbookHolder()
                    .getBoundSheetRecordList().size() == saxAfterKnownRecord));
        }
        payload.put("xls_context_workbook_type",
            xlsContext.xlsReadWorkbookHolder().getExcelType().name());
        payload.put("xls_context_sheet_before_null",
            Boolean.valueOf(xlsContext.xlsReadSheetHolder() == null));
        xlsContext.currentSheet(new ReadSheet(Integer.valueOf(0), "XlsContext"));
        payload.put("xls_context_sheet_no", xlsContext.xlsReadSheetHolder().getSheetNo());

        ReadWorkbook xlsxWorkbook = new ReadWorkbook();
        xlsxWorkbook.setExcelType(ExcelTypeEnum.XLSX);
        DefaultXlsxReadContext xlsxContext =
            new DefaultXlsxReadContext(xlsxWorkbook, ExcelTypeEnum.XLSX);
        payload.put("xlsx_context_workbook_type",
            xlsxContext.xlsxReadWorkbookHolder().getExcelType().name());
        payload.put("xlsx_context_sheet_before_null",
            Boolean.valueOf(xlsxContext.xlsxReadSheetHolder() == null));
        xlsxContext.xlsxReadWorkbookHolder()
            .setPackageRelationshipCollectionMap(new HashMap<Integer, org.apache.poi.openxml4j.opc.PackageRelationshipCollection>());
        xlsxContext.currentSheet(new ReadSheet(Integer.valueOf(0), "XlsxContext"));
        payload.put("xlsx_context_sheet_no", xlsxContext.xlsxReadSheetHolder().getSheetNo());

        Path out = outDir.resolve("excel_analyser_lifecycle.contract.json");
        String json = JSON.toJSONString(payload, JSONWriter.Feature.PrettyFormat);
        Files.write(out, (json + "\n").getBytes(StandardCharsets.UTF_8));
        System.out.println("Wrote " + out);
    }

    /**
     * Export lifecycle observations for Java's public {@code ExcelReader} methods.
     *
     * @param workbook Java-generated workbook used by every reader instance
     * @param outDir golden output directory
     * @throws Exception when a reader operation or contract write fails
     */
    @SuppressWarnings("deprecation")
    private static void writeExcelReaderContract(Path workbook, Path outDir) throws Exception {
        AnalysisEventListener<Object> listener = new AnalysisEventListener<Object>() {
            @Override
            public void invoke(Object data, AnalysisContext context) {
                // The contract observes lifecycle and return identity, not row values.
            }

            @Override
            public void doAfterAllAnalysed(AnalysisContext context) {
                // The contract observes lifecycle and return identity, not row values.
            }
        };
        ReadSheet sheet = EasyExcel.readSheet(Integer.valueOf(0)).build();
        Map<String, Object> payload = new LinkedHashMap<String, Object>();
        payload.put("authority", "com.alibaba:easyexcel:4.0.3");

        ReadWorkbook directWorkbook = new ReadWorkbook();
        directWorkbook.setFile(workbook.toFile());
        directWorkbook.setCustomReadListenerList(Arrays.asList(listener));
        com.alibaba.excel.ExcelReader directReader = new com.alibaba.excel.ExcelReader(directWorkbook);
        payload.put("direct_constructor", directReader.getClass().getName());
        directReader.finish();

        com.alibaba.excel.ExcelReader contextReader = EasyExcel.read(workbook.toFile(), listener).build();
        payload.put("reader_class", contextReader.getClass().getName());
        payload.put("analysis_context_class", contextReader.analysisContext().getClass().getName());
        payload.put("get_analysis_context_same", contextReader.analysisContext() == contextReader.getAnalysisContext());
        payload.put("excel_executor_class", contextReader.excelExecutor().getClass().getName());
        contextReader.finish();
        contextReader.close();
        payload.put("finish_then_close", Boolean.TRUE);

        com.alibaba.excel.ExcelReader deprecatedReader = EasyExcel.read(workbook.toFile(), listener).build();
        deprecatedReader.read();
        deprecatedReader.finish();
        payload.put("deprecated_read", Boolean.TRUE);

        com.alibaba.excel.ExcelReader allReader = EasyExcel.read(workbook.toFile(), listener).build();
        allReader.readAll();
        allReader.finish();
        payload.put("read_all", Boolean.TRUE);

        com.alibaba.excel.ExcelReader listReader = EasyExcel.read(workbook.toFile(), listener).build();
        payload.put("read_list_returns_self", listReader == listReader.read(Arrays.asList(sheet)));
        listReader.finish();

        com.alibaba.excel.ExcelReader varargsReader = EasyExcel.read(workbook.toFile(), listener).build();
        payload.put("read_varargs_returns_self", varargsReader == varargsReader.read(sheet));
        varargsReader.finish();

        Path out = outDir.resolve("excel_reader_lifecycle.contract.json");
        String json = JSON.toJSONString(payload, JSONWriter.Feature.PrettyFormat);
        Files.write(out, (json + "\n").getBytes(StandardCharsets.UTF_8));
        System.out.println("Wrote " + out);
    }

    /**
     * Export observable return types for Java's static facade overloads.
     *
     * @param outDir golden output directory
     * @throws Exception when the contract cannot be written
     */
    private static void writeFacadeApiContract(Path outDir) throws Exception {
        Map<String, Object> payload = new LinkedHashMap<String, Object>();
        payload.put("authority", "com.alibaba:easyexcel:4.0.3");
        payload.put("easy_excel_class", new EasyExcel().getClass().getName());
        payload.put("easy_excel_factory_class", new EasyExcelFactory().getClass().getName());
        payload.put("easy_excel_superclass", EasyExcel.class.getSuperclass().getName());

        Map<String, String> methods = new LinkedHashMap<String, String>();
        File file = new File("facade-contract.xlsx");
        ByteArrayInputStream input = new ByteArrayInputStream(new byte[0]);
        ByteArrayOutputStream output = new ByteArrayOutputStream();
        AnalysisEventListener<Object> listener = new AnalysisEventListener<Object>() {
            @Override
            public void invoke(Object data, AnalysisContext context) {
                // Contract probe only; no read is executed.
            }

            @Override
            public void doAfterAllAnalysed(AnalysisContext context) {
                // Contract probe only; no read is executed.
            }
        };
        methods.put("read()", EasyExcel.read().getClass().getName());
        methods.put("read(File)", EasyExcel.read(file).getClass().getName());
        methods.put("read(File,ReadListener)", EasyExcel.read(file, listener).getClass().getName());
        methods.put("read(File,Class,ReadListener)", EasyExcel.read(file, Object.class, listener).getClass().getName());
        methods.put("read(InputStream)", EasyExcel.read(input).getClass().getName());
        methods.put("read(InputStream,ReadListener)", EasyExcel.read(new ByteArrayInputStream(new byte[0]), listener).getClass().getName());
        methods.put("read(InputStream,Class,ReadListener)", EasyExcel.read(new ByteArrayInputStream(new byte[0]), Object.class, listener).getClass().getName());
        methods.put("read(String)", EasyExcel.read(file.getPath()).getClass().getName());
        methods.put("read(String,ReadListener)", EasyExcel.read(file.getPath(), listener).getClass().getName());
        methods.put("read(String,Class,ReadListener)", EasyExcel.read(file.getPath(), Object.class, listener).getClass().getName());
        methods.put("readSheet()", EasyExcel.readSheet().getClass().getName());
        methods.put("readSheet(Integer)", EasyExcel.readSheet(Integer.valueOf(1)).getClass().getName());
        methods.put("readSheet(Integer,String)", EasyExcel.readSheet(Integer.valueOf(1), "Data").getClass().getName());
        methods.put("readSheet(String)", EasyExcel.readSheet("Data").getClass().getName());
        methods.put("write()", EasyExcel.write().getClass().getName());
        methods.put("write(File)", EasyExcel.write(file).getClass().getName());
        methods.put("write(File,Class)", EasyExcel.write(file, Object.class).getClass().getName());
        methods.put("write(OutputStream)", EasyExcel.write(output).getClass().getName());
        methods.put("write(OutputStream,Class)", EasyExcel.write(output, Object.class).getClass().getName());
        methods.put("write(String)", EasyExcel.write(file.getPath()).getClass().getName());
        methods.put("write(String,Class)", EasyExcel.write(file.getPath(), Object.class).getClass().getName());
        methods.put("writerSheet()", EasyExcel.writerSheet().getClass().getName());
        methods.put("writerSheet(Integer)", EasyExcel.writerSheet(Integer.valueOf(1)).getClass().getName());
        methods.put("writerSheet(Integer,String)", EasyExcel.writerSheet(Integer.valueOf(1), "Data").getClass().getName());
        methods.put("writerSheet(String)", EasyExcel.writerSheet("Data").getClass().getName());
        methods.put("writerTable()", EasyExcel.writerTable().getClass().getName());
        methods.put("writerTable(Integer)", EasyExcel.writerTable(Integer.valueOf(1)).getClass().getName());
        payload.put("methods", methods);

        Path out = outDir.resolve("facade_api.contract.json");
        String json = JSON.toJSONString(payload, JSONWriter.Feature.PrettyFormat);
        Files.write(out, (json + "\n").getBytes(StandardCharsets.UTF_8));
        System.out.println("Wrote " + out);
    }

    /**
     * Build fixture-read + write-export specs.
     *
     * @param fixturesDir fixture root
     * @param artifactDir directory for Java-written binary goldens
     * @return export specs
     * @throws Exception when write artifacts cannot be prepared
     */
    private static List<ExportSpec> buildSpecs(Path fixturesDir, Path artifactDir) throws Exception {
        List<ExportSpec> specs = new ArrayList<ExportSpec>();

        // CompatibilityTest#t02 — headRowNumber(0), include date/string key cells
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.compatibility.CompatibilityTest#t02",
            fixturesDir.resolve("compatibility/t02.xlsx"),
            "compatibility_t02.expected.json",
            0,
            0,
            keyCells("2.2")
        ));

        // CompatibilityTest#t04
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.compatibility.CompatibilityTest#t04",
            fixturesDir.resolve("compatibility/t04.xlsx"),
            "compatibility_t04.expected.json",
            0,
            1,
            keyCells("0.5")
        ));

        // CompatibilityTest#t01 — .xls read
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.compatibility.CompatibilityTest#t01",
            fixturesDir.resolve("compatibility/t01.xls"),
            "compatibility_t01_xls.expected.json",
            0,
            1,
            keyCells("0.0")
        ));

        // BomDataTest
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.bom.BomDataTest#t01ReadCsv",
            fixturesDir.resolve("bom/office_bom.csv"),
            "bom_office_bom.expected.json",
            0,
            1,
            keyCells("0.0", "0.1")
        ));
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.bom.BomDataTest#noBom",
            fixturesDir.resolve("bom/no_bom.csv"),
            "bom_no_bom.expected.json",
            0,
            1,
            keyCells("0.0", "0.1")
        ));

        // ReadTest#simpleRead — demo.xlsx; date col 1 fully compared (STRING)
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.demo.read.ReadTest#simpleRead",
            fixturesDir.resolve("demo/demo.xlsx"),
            "demo_demo_sheet0.expected.json",
            0,
            0,
            keyCells("0.0", "0.1", "1.0", "1.1", "1.2", "10.1")
        ));
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.demo.read.ReadTest#demoCsv",
            fixturesDir.resolve("demo/demo.csv"),
            "demo_demo_csv.expected.json",
            0,
            0,
            keyCells("0.0", "0.1", "1.0", "1.1", "1.2", "10.1")
        ));

        // SimpleDataTest#t21SheetNameRead07
        specs.add(ExportSpec.withSheetName(
            "com.alibaba.easyexcel.test.core.simple.SimpleDataTest#t21SheetNameRead07",
            fixturesDir.resolve("simple/simple07.xlsx"),
            "simple_simple07.expected.json",
            "simple",
            1,
            keyCells("0.0")
        ));

        // ConverterDataTest#t11ReadAllConverter07 — fixture STRING read (dates included)
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.converter.ConverterDataTest#t11ReadAllConverter07",
            fixturesDir.resolve("converter/converter07.xlsx"),
            "converter_converter07.expected.json",
            0,
            1,
            keyCells("0.0", "0.12", "0.13", "0.37")
        ));

        // ConverterDataTest#t12ReadAllConverter03 — .xls converter fixture (full STRING,
        // including builtin short dates via xls_display overlay).
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.converter.ConverterDataTest#t12ReadAllConverter03",
            fixturesDir.resolve("xls/converter03.xls"),
            "converter_converter03_xls.expected.json",
            0,
            1,
            keyCells("0.0", "0.12", "0.13", "0.33", "0.37")
        ));

        // ConverterDataTest#t13 — csv converter fixture
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.converter.ConverterDataTest#t13ReadAllConverterCsv",
            fixturesDir.resolve("converter/converterCsv.csv"),
            "converter_converter_csv.expected.json",
            0,
            1,
            keyCells("0.0", "0.12", "0.13")
        ));

        // MultipleSheetsDataTest — sheet 0 + sheet 1 (xlsx)
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.multiplesheets.MultipleSheetsDataTest#t01Read07#sheet0",
            fixturesDir.resolve("multiplesheets/multiplesheets.xlsx"),
            "multiplesheets_sheet0.expected.json",
            0,
            1,
            keyCells("0.0")
        ));
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.multiplesheets.MultipleSheetsDataTest#t01Read07#sheet1",
            fixturesDir.resolve("multiplesheets/multiplesheets.xlsx"),
            "multiplesheets_sheet1.expected.json",
            1,
            1,
            keyCells("0.0")
        ));

        // MultipleSheetsDataTest — .xls sheet0 + sheet1
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.multiplesheets.MultipleSheetsDataTest#t02Read03#sheet0",
            fixturesDir.resolve("xls/multiplesheets.xls"),
            "multiplesheets_xls_sheet0.expected.json",
            0,
            1,
            keyCells("0.0")
        ));
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.multiplesheets.MultipleSheetsDataTest#t02Read03#sheet1",
            fixturesDir.resolve("xls/multiplesheets.xls"),
            "multiplesheets_xls_sheet1.expected.json",
            1,
            1,
            keyCells("0.0")
        ));

        // CompatibilityTest#t03 — sparse leading null columns
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.compatibility.CompatibilityTest#t03",
            fixturesDir.resolve("compatibility/t03.xlsx"),
            "compatibility_t03.expected.json",
            0,
            1,
            keyCells("0.0", "0.11")
        ));

        // CompatibilityTest#t05 — date rounding STRING (full date cells)
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.compatibility.CompatibilityTest#t05",
            fixturesDir.resolve("compatibility/t05.xlsx"),
            "compatibility_t05.expected.json",
            0,
            1,
            keyCells("0.0", "1.0", "2.0", "3.0", "4.0")
        ));

        // CompatibilityTest#t06 — numeric precision STRING
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.compatibility.CompatibilityTest#t06",
            fixturesDir.resolve("compatibility/t06.xlsx"),
            "compatibility_t06.expected.json",
            0,
            0,
            keyCells("0.2")
        ));

        // CompatibilityTest#t07 — STRING "24.20" + trailing-space accounting formats
        // (`-1.07 ` via `\ `; `_` pads stripped like EasyExcel cleanFormatForNumber).
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.compatibility.CompatibilityTest#t07",
            fixturesDir.resolve("compatibility/t07.xlsx"),
            "compatibility_t07.expected.json",
            0,
            1,
            keyCells("0.11", "0.12", "0.13", "0.15")
        ));

        // CompatibilityTest#t09 — sharedStrings _x005f_ escape
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.compatibility.CompatibilityTest#t09",
            fixturesDir.resolve("compatibility/t09.xlsx"),
            "compatibility_t09.expected.json",
            0,
            0,
            keyCells("0.0")
        ));

        // DateFormatTest#t03Read — full STRING (unpadded month `2023-1-01`).
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.dataformat.DateFormatTest#t03Read",
            fixturesDir.resolve("dataformat/dataformatv2.xlsx"),
            "dataformat_v2.expected.json",
            0,
            0,
            keyCells("0.0", "1.0", "3.0", "6.0")
        ));

        // DateFormatTest fixtures — xlsx / xls full STRING
        // (CN `上午/下午` → AM/PM, mmmmm PUA wrap, BIFF Latin-1 ¥).
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.dataformat.DateFormatTest#t01Read07",
            fixturesDir.resolve("dataformat/dataformat.xlsx"),
            "dataformat_xlsx.expected.json",
            0,
            1,
            keyCells("0.0", "2.0", "3.0", "0.4", "16.0", "22.0")
        ));
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.dataformat.DateFormatTest#t02Read03",
            fixturesDir.resolve("dataformat/dataformat.xls"),
            "dataformat_xls.expected.json",
            0,
            1,
            keyCells("0.0", "0.4", "2.4", "16.0", "22.0")
        ));

        // temp/issue2443 date fixtures mirrored under dataformat/
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.temp.issue2443.Issue2443Test#date1",
            fixturesDir.resolve("dataformat/date1.xlsx"),
            "dataformat_date1.expected.json",
            0,
            0,
            keyCells("0.0", "0.1")
        ));
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.temp.issue2443.Issue2443Test#date2",
            fixturesDir.resolve("dataformat/date2.xlsx"),
            "dataformat_date2.expected.json",
            0,
            0,
            keyCells("0.0", "0.1")
        ));

        // Demo / extra / cellData / simple07 / template fixture reads
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.extra.ExtraDataTest#t01Read07#content",
            fixturesDir.resolve("demo/extra.xlsx"),
            "demo_extra_xlsx.expected.json",
            0,
            1,
            keyCells("0.0")
        ));
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.extra.ExtraDataTest#t02Read03#content",
            fixturesDir.resolve("xls/extra/extra.xls"),
            "demo_extra_xls.expected.json",
            0,
            1,
            keyCells("0.0")
        ));
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.demo.read.ReadTest#cellDataRead",
            fixturesDir.resolve("demo/cellDataDemo.xlsx"),
            "demo_cell_data.expected.json",
            0,
            1,
            keyCells("0.0", "0.1")
        ));
        specs.add(ExportSpec.withSheetName(
            "com.alibaba.easyexcel.test.demo.read.ReadTest#simple07",
            fixturesDir.resolve("demo/simple07.xlsx"),
            "demo_simple07.expected.json",
            "simple",
            1,
            keyCells("0.0")
        ));
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.template.TemplateDataTest#template07#read",
            fixturesDir.resolve("template/template07.xlsx"),
            "template_template07.expected.json",
            0,
            0,
            keyCells("0.0", "0.1")
        ));
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.template.TemplateDataTest#template03#read",
            fixturesDir.resolve("template/template03.xls"),
            "template_template03_xls.expected.json",
            0,
            0,
            keyCells("0.0", "0.1")
        ));

        // --- Java-written artifacts ---

        Path writtenXlsx = writeSimpleArtifact(artifactDir, "simple_data.xlsx", ExcelTypeEnum.XLSX);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.simple.SimpleDataTest#t01ReadAndWrite07",
            writtenXlsx,
            "simple_data.expected.json",
            0,
            1,
            keyCells("0.0", "9.0")
        ));

        Path writtenCsv = writeSimpleArtifact(artifactDir, "simple_data.csv", ExcelTypeEnum.CSV);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.simple.SimpleDataTest#t03ReadAndWriteCsv",
            writtenCsv,
            "simple_data_csv.expected.json",
            0,
            1,
            keyCells("0.0", "9.0")
        ));

        // SimpleData .xls write artifact — Rust read-only对照
        Path writtenXls = writeSimpleArtifact(artifactDir, "simple_data.xls", ExcelTypeEnum.XLS);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.simple.SimpleDataTest#t02ReadAndWrite03",
            writtenXls,
            "simple_data_xls.expected.json",
            0,
            1,
            keyCells("0.0", "9.0")
        ));

        // Converter write → STRING read (date columns included); xlsx/xls/csv
        Path converterArtifact = writeConverterArtifact(artifactDir, "converter_write.xlsx", ExcelTypeEnum.XLSX);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.converter.ConverterDataTest#t01ReadAndWrite07",
            converterArtifact,
            "converter_write.expected.json",
            0,
            1,
            keyCells("0.0", "0.1", "0.2", "0.3", "0.12", "0.13")
        ));
        Path converterWriteXls = writeConverterArtifact(artifactDir, "converter_write.xls", ExcelTypeEnum.XLS);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.converter.ConverterDataTest#t02ReadAndWrite03#write",
            converterWriteXls,
            "converter_write_xls.expected.json",
            0,
            1,
            keyCells("0.0", "0.1", "0.2", "0.3", "0.12", "0.13")
        ));
        Path converterWriteCsv = writeConverterArtifact(artifactDir, "converter_write.csv", ExcelTypeEnum.CSV);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.converter.ConverterDataTest#t03ReadAndWriteCsv#write",
            converterWriteCsv,
            "converter_write_csv.expected.json",
            0,
            1,
            keyCells("0.0", "0.1", "0.2", "0.3", "0.12", "0.13")
        ));

        // Fill — simpleFill against fill/simple.xlsx template
        Path fillArtifact = writeFillArtifact(fixturesDir, artifactDir);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.demo.fill.FillTest#simpleFill",
            fillArtifact,
            "fill_simple.expected.json",
            0,
            0,
            keyCells("0.0", "0.1", "1.0", "1.1")
        ));

        // Style — write with HorizontalCellStyleStrategy; STRING content对照
        Path styleArtifact = writeStyleArtifact(artifactDir);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.style.StyleDataTest#t01ReadAndWrite07",
            styleArtifact,
            "style_data.expected.json",
            0,
            1,
            keyCells("0.0", "0.1", "1.0", "1.1")
        ));

        // Annotation — DateTimeFormat + NumberFormat `#.##%` (java_compat_display → `9999%`).
        Path annotationArtifact = writeAnnotationArtifact(artifactDir);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.annotation.AnnotationDataTest#t01ReadAndWrite07",
            annotationArtifact,
            "annotation_data.expected.json",
            0,
            1,
            keyCells("0.0", "0.1")
        ));

        // Exclude column indexes {0,3} → column2, column3 remain
        Path excludeArtifact = writeExcludeArtifact(artifactDir);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.excludeorinclude.ExcludeOrIncludeDataTest#t01ExcludeIndex07",
            excludeArtifact,
            "exclude_index.expected.json",
            0,
            1,
            keyCells("0.0", "0.1")
        ));

        // Encrypt — Java write with password; golden carries password for Rust read对照
        Path encryptArtifact = writeEncryptArtifact(artifactDir);
        specs.add(ExportSpec.withPassword(
            "com.alibaba.easyexcel.test.core.encrypt.EncryptDataTest#t01ReadAndWrite07",
            encryptArtifact,
            "encrypt_data.expected.json",
            0,
            1,
            "123456",
            keyCells("0.0", "9.0")
        ));

        // ExcludeOrInclude — csv exclude index + field-name exclude + include index/field
        Path excludeCsv = writeExcludeArtifactTyped(artifactDir, "exclude_index.csv", ExcelTypeEnum.CSV, true);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.excludeorinclude.ExcludeOrIncludeDataTest#t03ExcludeIndexCsv",
            excludeCsv,
            "exclude_index_csv.expected.json",
            0,
            1,
            keyCells("0.0", "0.1")
        ));
        Path excludeField = writeExcludeFieldArtifact(artifactDir, "exclude_field.xlsx", ExcelTypeEnum.XLSX);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.excludeorinclude.ExcludeOrIncludeDataTest#t11ExcludeFieldName07",
            excludeField,
            "exclude_field.expected.json",
            0,
            1,
            keyCells("0.0")
        ));
        Path includeIndex = writeIncludeIndexArtifact(artifactDir, "include_index.xlsx", ExcelTypeEnum.XLSX);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.excludeorinclude.ExcludeOrIncludeDataTest#t21IncludeIndex07",
            includeIndex,
            "include_index.expected.json",
            0,
            1,
            keyCells("0.0", "0.1")
        ));
        Path includeField = writeIncludeFieldArtifact(artifactDir, "include_field.xlsx", ExcelTypeEnum.XLSX);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.excludeorinclude.ExcludeOrIncludeDataTest#t31IncludeFieldName07",
            includeField,
            "include_field.expected.json",
            0,
            1,
            keyCells("0.0", "0.1")
        ));
        Path includeOrder = writeIncludeFieldOrderArtifact(artifactDir);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.excludeorinclude.ExcludeOrIncludeDataTest#t41IncludeFieldNameOrder07",
            includeOrder,
            "include_field_order.expected.json",
            0,
            1,
            keyCells("0.0", "0.1", "0.2")
        ));

        // Fill — simple .xls template + horizontal fill
        Path fillXls = writeFillSimpleTyped(
            fixturesDir, artifactDir, "fill_simple_xls.xls", "xls/fill/simple.xls", ExcelTypeEnum.XLS);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.fill.FillDataTest#t02Fill03",
            fillXls,
            "fill_simple_xls.expected.json",
            0,
            0,
            keyCells("0.0", "0.1", "1.0", "1.1")
        ));
        Path fillHorizontal = writeFillHorizontalArtifact(
            fixturesDir, artifactDir, "fill_horizontal.xlsx", "fill/horizontal.xlsx", ExcelTypeEnum.XLSX);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.fill.FillDataTest#t05HorizontalFill07",
            fillHorizontal,
            "fill_horizontal.expected.json",
            0,
            0,
            keyCells("0.2")
        ));
        Path fillHorizontalXls = writeFillHorizontalArtifact(
            fixturesDir, artifactDir, "fill_horizontal_xls.xls", "xls/fill/horizontal.xls", ExcelTypeEnum.XLS);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.fill.FillDataTest#t06HorizontalFill03",
            fillHorizontalXls,
            "fill_horizontal_xls.expected.json",
            0,
            0,
            keyCells("0.2")
        ));
        Path fillByName = writeFillByNameArtifact(
            fixturesDir, artifactDir, "fill_by_name.xlsx", "fill/byName.xlsx", ExcelTypeEnum.XLSX);
        specs.add(ExportSpec.withSheetName(
            "com.alibaba.easyexcel.test.core.fill.FillDataTest#t07ByNameFill07",
            fillByName,
            "fill_by_name.expected.json",
            "Sheet2",
            0,
            keyCells("0.0", "0.1")
        ));
        Path fillByNameXls = writeFillByNameArtifact(
            fixturesDir, artifactDir, "fill_by_name_xls.xls", "xls/fill/byName.xls", ExcelTypeEnum.XLS);
        specs.add(ExportSpec.withSheetName(
            "com.alibaba.easyexcel.test.core.fill.FillDataTest#t08ByNameFill03",
            fillByNameXls,
            "fill_by_name_xls.expected.json",
            "Sheet2",
            0,
            keyCells("0.0", "0.1")
        ));
        // Complex fill — forceNewRow + LoopMerge；读侧 headRowNumber(3)
        Path fillComplex = writeFillComplexArtifact(
            fixturesDir, artifactDir, "fill_complex.xlsx", "fill/complex.xlsx", ExcelTypeEnum.XLSX);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.fill.FillDataTest#t03ComplexFill07",
            fillComplex,
            "fill_complex.expected.json",
            0,
            3,
            keyCells("0.0", "19.0")
        ));
        Path fillComplexXls = writeFillComplexArtifact(
            fixturesDir, artifactDir, "fill_complex_xls.xls", "xls/fill/complex.xls", ExcelTypeEnum.XLS);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.fill.FillDataTest#t04ComplexFill03",
            fillComplexXls,
            "fill_complex_xls.expected.json",
            0,
            3,
            keyCells("0.0", "19.0")
        ));

        // Style .xls write — Rust read对照
        Path styleXls = writeStyleArtifactTyped(artifactDir, "style_data.xls", ExcelTypeEnum.XLS);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.style.StyleDataTest#t02ReadAndWrite03",
            styleXls,
            "style_data_xls.expected.json",
            0,
            1,
            keyCells("0.0", "0.1", "1.0", "1.1")
        ));

        // NoHeadData — needHead(false)；xlsx/xls/csv 全表
        Path noHead = writeNoHeadArtifact(artifactDir, "no_head_data.xlsx", ExcelTypeEnum.XLSX);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.head.NoHeadDataTest#t01ReadAndWrite07",
            noHead,
            "no_head_data.expected.json",
            0,
            0,
            keyCells("0.0")
        ));
        Path noHeadXls = writeNoHeadArtifact(artifactDir, "no_head_data.xls", ExcelTypeEnum.XLS);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.head.NoHeadDataTest#t02ReadAndWrite03",
            noHeadXls,
            "no_head_data_xls.expected.json",
            0,
            0,
            keyCells("0.0")
        ));
        Path noHeadCsv = writeNoHeadArtifact(artifactDir, "no_head_data.csv", ExcelTypeEnum.CSV);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.head.NoHeadDataTest#t03ReadAndWriteCsv",
            noHeadCsv,
            "no_head_data_csv.expected.json",
            0,
            0,
            keyCells("0.0")
        ));

        // SortData — index/order columns
        Path sortArtifact = writeSortArtifact(artifactDir);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.sort.SortDataTest#t01ReadAndWrite07",
            sortArtifact,
            "sort_data.expected.json",
            0,
            1,
            keyCells("0.0", "0.1", "0.2", "0.3")
        ));

        // --- P0-3: previously missing core classes (full-table STRING对照) ---

        // CacheDataTest — 姓名/年龄 ×10（xlsx/xls/csv 全表）
        Path cacheArtifact = writeCacheArtifact(artifactDir, "cache_data.xlsx", ExcelTypeEnum.XLSX);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.cache.CacheDataTest#t01ReadAndWrite",
            cacheArtifact,
            "cache_data.expected.json",
            0,
            1,
            keyCells("0.0", "0.1", "9.0", "9.1")
        ));
        Path cacheXls = writeCacheArtifact(artifactDir, "cache_data.xls", ExcelTypeEnum.XLS);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.cache.CacheDataTest#t01ReadAndWrite#xls",
            cacheXls,
            "cache_data_xls.expected.json",
            0,
            1,
            keyCells("0.0", "0.1", "9.0", "9.1")
        ));
        Path cacheCsv = writeCacheArtifact(artifactDir, "cache_data.csv", ExcelTypeEnum.CSV);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.cache.CacheDataTest#t01ReadAndWrite#csv",
            cacheCsv,
            "cache_data_csv.expected.json",
            0,
            1,
            keyCells("0.0", "0.1", "9.0", "9.1")
        ));

        // CellDataDataTest — WriteCellData date/number/formula
        // xlsx/xls/csv 全表（CN DateTimeFormat STRING 已对齐）
        Path celldataXlsx = writeCellDataArtifact(artifactDir, "celldata_data.xlsx", ExcelTypeEnum.XLSX);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.celldata.CellDataDataTest#t01ReadAndWrite07",
            celldataXlsx,
            "celldata_data.expected.json",
            0,
            1,
            keyCells("0.0", "0.1", "0.2")
        ));
        Path celldataXls = writeCellDataArtifact(artifactDir, "celldata_data.xls", ExcelTypeEnum.XLS);
        // CN DateTimeFormat 已与 Rust STRING 对齐 → 全表对照（含公式列 0.3）
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.celldata.CellDataDataTest#t02ReadAndWrite03",
            celldataXls,
            "celldata_data_xls.expected.json",
            0,
            1,
            keyCells("0.0", "0.1", "0.2", "0.3")
        ));
        Path celldataCsv = writeCellDataArtifact(artifactDir, "celldata_data.csv", ExcelTypeEnum.CSV);
        // CSV stores DateTimeFormat text literally → full-table STRING 可对齐
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.celldata.CellDataDataTest#t03ReadAndWriteCsv",
            celldataCsv,
            "celldata_data_csv.expected.json",
            0,
            1,
            keyCells("0.0", "0.1", "0.2")
        ));

        // CharsetDataTest — GBK / UTF-8 CSV（全表对照）
        Path charsetGbk = writeCharsetArtifact(artifactDir, "charset_gbk.csv", Charset.forName("GBK"));
        specs.add(ExportSpec.withCharset(
            "com.alibaba.easyexcel.test.core.charset.CharsetDataTest#t01ReadAndWriteCsv#gbk",
            charsetGbk,
            "charset_gbk.expected.json",
            0,
            1,
            "GBK",
            keyCells("0.0", "0.1", "9.0", "9.1")
        ));
        Path charsetUtf8 = writeCharsetArtifact(artifactDir, "charset_utf8.csv", StandardCharsets.UTF_8);
        specs.add(ExportSpec.withCharset(
            "com.alibaba.easyexcel.test.core.charset.CharsetDataTest#t01ReadAndWriteCsv#utf8",
            charsetUtf8,
            "charset_utf8.expected.json",
            0,
            1,
            "UTF-8",
            keyCells("0.0", "0.1", "9.0", "9.1")
        ));

        // ExceptionDataTest — content write (xlsx/xls/csv) + multi-sheet stop fixture sheet0..4
        Path exceptionXlsx = writeExceptionArtifact(artifactDir, "exception_data.xlsx", ExcelTypeEnum.XLSX);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.exception.ExceptionDataTest#t01ReadAndWrite07",
            exceptionXlsx,
            "exception_data.expected.json",
            0,
            1,
            keyCells("0.0", "9.0")
        ));
        Path exceptionXls = writeExceptionArtifact(artifactDir, "exception_data.xls", ExcelTypeEnum.XLS);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.exception.ExceptionDataTest#t02ReadAndWrite03",
            exceptionXls,
            "exception_data_xls.expected.json",
            0,
            1,
            keyCells("0.0", "9.0")
        ));
        Path exceptionCsv = writeExceptionArtifact(artifactDir, "exception_data.csv", ExcelTypeEnum.CSV);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.exception.ExceptionDataTest#t03ReadAndWriteCsv",
            exceptionCsv,
            "exception_data_csv.expected.json",
            0,
            1,
            keyCells("0.0", "9.0")
        ));
        Path exceptionStop = writeExceptionStopSheetsArtifact(artifactDir);
        for (int sheet = 0; sheet < 5; sheet++) {
            specs.add(ExportSpec.of(
                "com.alibaba.easyexcel.test.core.exception.ExceptionDataTest#t21ReadAndWrite07#sheet"
                    + sheet,
                exceptionStop,
                "exception_stop_sheet" + sheet + ".expected.json",
                sheet,
                1,
                keyCells("0.0", "9.0")
            ));
        }

        // WriteHandlerTest — STRING content 姓名0..9（handler 副作用不进 golden）
        Path handlerXlsx = writeHandlerArtifact(artifactDir, "handler_data.xlsx", ExcelTypeEnum.XLSX);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.handler.WriteHandlerTest#t01WorkbookWrite07",
            handlerXlsx,
            "handler_data.expected.json",
            0,
            1,
            keyCells("0.0", "9.0")
        ));
        Path handlerXls = writeHandlerArtifact(artifactDir, "handler_data.xls", ExcelTypeEnum.XLS);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.handler.WriteHandlerTest#t02WorkbookWrite03",
            handlerXls,
            "handler_data_xls.expected.json",
            0,
            1,
            keyCells("0.0", "9.0")
        ));
        Path handlerCsv = writeHandlerArtifact(artifactDir, "handler_data.csv", ExcelTypeEnum.CSV);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.handler.WriteHandlerTest#t03WorkbookWriteCsv",
            handlerCsv,
            "handler_data_csv.expected.json",
            0,
            1,
            keyCells("0.0", "9.0")
        ));

        // LargeDataTest — 小样本（100 行 × 25 列），非 large07 74MB；xlsx + csv
        Path largeSample = writeLargeSampleArtifact(artifactDir, "large_sample.xlsx", ExcelTypeEnum.XLSX);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.large.LargeDataTest#t04Write#sample100",
            largeSample,
            "large_sample.expected.json",
            0,
            1,
            keyCells("0.0", "0.24", "99.0", "99.24")
        ));
        Path largeSampleXls = writeLargeSampleArtifact(artifactDir, "large_sample.xls", ExcelTypeEnum.XLS);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.large.LargeDataTest#t04Write#sample100xls",
            largeSampleXls,
            "large_sample_xls.expected.json",
            0,
            1,
            keyCells("0.0", "0.24", "99.0", "99.24")
        ));
        Path largeSampleCsv = writeLargeSampleArtifact(artifactDir, "large_sample.csv", ExcelTypeEnum.CSV);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.large.LargeDataTest#t03ReadAndWriteCsv#sample100",
            largeSampleCsv,
            "large_sample_csv.expected.json",
            0,
            1,
            keyCells("0.0", "0.24", "99.0", "99.24")
        ));

        // NoModelDataTest — List 写 / headRowNumber(0)
        Path nomodelXlsx = writeNoModelArtifact(artifactDir, "nomodel_data.xlsx", ExcelTypeEnum.XLSX);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.nomodel.NoModelDataTest#t01ReadAndWrite07",
            nomodelXlsx,
            "nomodel_data.expected.json",
            0,
            0,
            keyCells("0.0", "0.1", "0.2", "9.0", "9.1", "9.2")
        ));
        Path nomodelXls = writeNoModelArtifact(artifactDir, "nomodel_data.xls", ExcelTypeEnum.XLS);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.nomodel.NoModelDataTest#t02ReadAndWrite03",
            nomodelXls,
            "nomodel_data_xls.expected.json",
            0,
            0,
            keyCells("0.0", "0.1", "0.2", "9.0", "9.1", "9.2")
        ));
        Path nomodelCsv = writeNoModelArtifact(artifactDir, "nomodel_data.csv", ExcelTypeEnum.CSV);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.nomodel.NoModelDataTest#t03ReadAndWriteCsv",
            nomodelCsv,
            "nomodel_data_csv.expected.json",
            0,
            0,
            keyCells("0.0", "0.1", "0.2", "9.0", "9.1", "9.2")
        ));
        Path nomodelRepeat = writeNoModelRepeatArtifact(
            artifactDir, nomodelXlsx, "nomodel_repeat.xlsx", ExcelTypeEnum.XLSX);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.nomodel.NoModelDataTest#t01ReadAndWrite07#repeat",
            nomodelRepeat,
            "nomodel_repeat.expected.json",
            0,
            0,
            keyCells("0.0", "9.0", "9.1", "9.2")
        ));
        Path nomodelRepeatXls = writeNoModelRepeatArtifact(
            artifactDir, nomodelXls, "nomodel_repeat_xls.xls", ExcelTypeEnum.XLS);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.nomodel.NoModelDataTest#t02ReadAndWrite03#repeat",
            nomodelRepeatXls,
            "nomodel_repeat_xls.expected.json",
            0,
            0,
            keyCells("0.0", "9.0", "9.1", "9.2")
        ));
        Path nomodelRepeatCsv = writeNoModelRepeatArtifact(
            artifactDir, nomodelCsv, "nomodel_repeat_csv.csv", ExcelTypeEnum.CSV);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.nomodel.NoModelDataTest#t03ReadAndWriteCsv#repeat",
            nomodelRepeatCsv,
            "nomodel_repeat_csv.expected.json",
            0,
            0,
            keyCells("0.0", "9.0", "9.1", "9.2")
        ));

        // UnCamelDataTest — non-camel field names
        Path noncamelXlsx = writeNonCamelArtifact(artifactDir, "noncamel_data.xlsx", ExcelTypeEnum.XLSX);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.noncamel.UnCamelDataTest#t01ReadAndWrite07",
            noncamelXlsx,
            "noncamel_data.expected.json",
            0,
            1,
            keyCells("0.0", "0.5", "9.0", "9.5")
        ));
        Path noncamelXls = writeNonCamelArtifact(artifactDir, "noncamel_data.xls", ExcelTypeEnum.XLS);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.noncamel.UnCamelDataTest#t02ReadAndWrite03",
            noncamelXls,
            "noncamel_data_xls.expected.json",
            0,
            1,
            keyCells("0.0", "0.5", "9.0", "9.5")
        ));
        Path noncamelCsv = writeNonCamelArtifact(artifactDir, "noncamel_data.csv", ExcelTypeEnum.CSV);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.noncamel.UnCamelDataTest#t03ReadAndWriteCsv",
            noncamelCsv,
            "noncamel_data_csv.expected.json",
            0,
            1,
            keyCells("0.0", "0.5", "9.0", "9.5")
        ));

        // ParameterDataTest — 姓名0..9
        Path parameterXlsx = writeParameterArtifact(artifactDir, "parameter_data.xlsx", ExcelTypeEnum.XLSX);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.parameter.ParameterDataTest#t01ReadAndWrite",
            parameterXlsx,
            "parameter_data.expected.json",
            0,
            1,
            keyCells("0.0", "9.0")
        ));
        Path parameterCsv = writeParameterArtifact(artifactDir, "parameter_data.csv", ExcelTypeEnum.CSV);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.parameter.ParameterDataTest#t02ReadAndWrite",
            parameterCsv,
            "parameter_data_csv.expected.json",
            0,
            1,
            keyCells("0.0", "9.0")
        ));
        Path parameterXls = writeParameterArtifact(artifactDir, "parameter_data.xls", ExcelTypeEnum.XLS);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.parameter.ParameterDataTest#t01ReadAndWrite#xls",
            parameterXls,
            "parameter_data_xls.expected.json",
            0,
            1,
            keyCells("0.0", "9.0")
        ));

        // ComplexHeadDataTest — 三级表头 headRowNumber(3)
        Path complexHead = writeComplexHeadArtifact(artifactDir, "complex_head.xlsx", ExcelTypeEnum.XLSX);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.head.ComplexHeadDataTest#t01ReadAndWrite07",
            complexHead,
            "complex_head.expected.json",
            0,
            3,
            keyCells("0.0", "0.1", "0.2", "0.3", "0.4")
        ));
        Path complexHeadXls = writeComplexHeadArtifact(artifactDir, "complex_head.xls", ExcelTypeEnum.XLS);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.head.ComplexHeadDataTest#t02ReadAndWrite03",
            complexHeadXls,
            "complex_head_xls.expected.json",
            0,
            3,
            keyCells("0.0", "0.1", "0.2", "0.3", "0.4")
        ));
        Path complexHeadCsv = writeComplexHeadArtifact(artifactDir, "complex_head.csv", ExcelTypeEnum.CSV);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.head.ComplexHeadDataTest#t03ReadAndWriteCsv",
            complexHeadCsv,
            "complex_head_csv.expected.json",
            0,
            3,
            keyCells("0.0", "0.1", "0.2", "0.3", "0.4")
        ));

        // AnnotationIndexAndNameDataTest — index + name 列布局（缺 col3）
        Path annotationIndex = writeAnnotationIndexAndNameArtifact(
            artifactDir, "annotation_index_name.xlsx", ExcelTypeEnum.XLSX);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.annotation.AnnotationIndexAndNameDataTest#t01ReadAndWrite07",
            annotationIndex,
            "annotation_index_name.expected.json",
            0,
            1,
            keyCells("0.0", "0.1", "0.2", "0.4")
        ));
        Path annotationIndexXls = writeAnnotationIndexAndNameArtifact(
            artifactDir, "annotation_index_name.xls", ExcelTypeEnum.XLS);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.annotation.AnnotationIndexAndNameDataTest#t02ReadAndWrite03",
            annotationIndexXls,
            "annotation_index_name_xls.expected.json",
            0,
            1,
            keyCells("0.0", "0.1", "0.2", "0.4")
        ));
        Path annotationIndexCsv = writeAnnotationIndexAndNameArtifact(
            artifactDir, "annotation_index_name.csv", ExcelTypeEnum.CSV);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.annotation.AnnotationIndexAndNameDataTest#t03ReadAndWriteCsv",
            annotationIndexCsv,
            "annotation_index_name_csv.expected.json",
            0,
            1,
            keyCells("0.0", "0.1", "0.2", "0.4")
        ));

        // ListHeadDataTest — 动态 head；xlsx/xls/csv 日期 STRING 已对齐 → 全表
        Path listHead = writeListHeadArtifact(artifactDir, "list_head.xlsx", ExcelTypeEnum.XLSX);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.head.ListHeadDataTest#t01ReadAndWrite07",
            listHead,
            "list_head.expected.json",
            0,
            1,
            keyCells("0.0", "0.1", "0.2", "0.3")
        ));
        Path listHeadXls = writeListHeadArtifact(artifactDir, "list_head.xls", ExcelTypeEnum.XLS);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.head.ListHeadDataTest#t02ReadAndWrite03",
            listHeadXls,
            "list_head_xls.expected.json",
            0,
            1,
            keyCells("0.0", "0.1", "0.2", "0.3")
        ));
        Path listHeadCsv = writeListHeadArtifact(artifactDir, "list_head.csv", ExcelTypeEnum.CSV);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.head.ListHeadDataTest#t03ReadAndWriteCsv",
            listHeadCsv,
            "list_head_csv.expected.json",
            0,
            1,
            keyCells("0.0", "0.1", "0.2", "0.3")
        ));

        // RepetitionDataTest — double write → 2 data rows; table variant headRowNumber(2)
        Path repetitionXlsx = writeRepetitionArtifact(artifactDir, "repetition_data.xlsx", ExcelTypeEnum.XLSX);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.repetition.RepetitionDataTest#t01ReadAndWrite07",
            repetitionXlsx,
            "repetition_data.expected.json",
            0,
            1,
            keyCells("0.0", "1.0")
        ));
        Path repetitionXls = writeRepetitionArtifact(artifactDir, "repetition_data.xls", ExcelTypeEnum.XLS);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.repetition.RepetitionDataTest#t02ReadAndWrite03",
            repetitionXls,
            "repetition_data_xls.expected.json",
            0,
            1,
            keyCells("0.0", "1.0")
        ));
        Path repetitionCsv = writeRepetitionArtifact(artifactDir, "repetition_data.csv", ExcelTypeEnum.CSV);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.repetition.RepetitionDataTest#t03ReadAndWriteCsv",
            repetitionCsv,
            "repetition_data_csv.expected.json",
            0,
            1,
            keyCells("0.0", "1.0")
        ));
        Path repetitionTable = writeRepetitionTableArtifact(artifactDir);
        specs.add(ExportSpec.of(
            "com.alibaba.easyexcel.test.core.repetition.RepetitionDataTest#t11ReadAndWriteTable07",
            repetitionTable,
            "repetition_table.expected.json",
            0,
            2,
            keyCells("0.0", "1.0")
        ));

        // SkipDataTest — 4 sheets；按 sheet name 全表对照
        Path skipArtifact = writeSkipArtifact(artifactDir);
        specs.add(ExportSpec.withSheetName(
            "com.alibaba.easyexcel.test.core.skip.SkipDataTest#t01ReadAndWrite07#sheet0",
            skipArtifact,
            "skip_sheet0.expected.json",
            "第一个",
            1,
            keyCells("0.0")
        ));
        specs.add(ExportSpec.withSheetName(
            "com.alibaba.easyexcel.test.core.skip.SkipDataTest#t01ReadAndWrite07#sheet1",
            skipArtifact,
            "skip_sheet1.expected.json",
            "第二个",
            1,
            keyCells("0.0")
        ));
        specs.add(ExportSpec.withSheetName(
            "com.alibaba.easyexcel.test.core.skip.SkipDataTest#t01ReadAndWrite07#sheet2",
            skipArtifact,
            "skip_sheet2.expected.json",
            "第三个",
            1,
            keyCells("0.0")
        ));
        specs.add(ExportSpec.withSheetName(
            "com.alibaba.easyexcel.test.core.skip.SkipDataTest#t01ReadAndWrite07#sheet3",
            skipArtifact,
            "skip_sheet3.expected.json",
            "第四个",
            1,
            keyCells("0.0")
        ));

        return specs;
    }

    /**
     * CacheData write — 姓名/年龄 ×10.
     */
    private static Path writeCacheArtifact(Path artifactDir, String fileName, ExcelTypeEnum excelType)
        throws Exception {
        Path target = artifactDir.resolve(fileName);
        List<CacheData> list = new ArrayList<CacheData>();
        for (int i = 0; i < 10; i++) {
            CacheData row = new CacheData();
            row.setName("姓名" + i);
            row.setAge(Long.valueOf(i));
            list.add(row);
        }
        EasyExcel.write(target.toFile(), CacheData.class)
            .excelType(excelType)
            .sheet()
            .doWrite(list);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * CellDataWriteData — date WriteCellData + number + formula B2+C2.
     */
    private static Path writeCellDataArtifact(Path artifactDir, String fileName, ExcelTypeEnum excelType)
        throws Exception {
        Path target = artifactDir.resolve(fileName);
        CellDataWriteData cellDataData = new CellDataWriteData();
        cellDataData.setDate(new WriteCellData<Date>(DateUtils.parseDate("2020-01-01 01:01:01")));
        WriteCellData<Integer> integer1 = new WriteCellData<Integer>();
        integer1.setType(CellDataTypeEnum.NUMBER);
        integer1.setNumberValue(BigDecimal.valueOf(2L));
        cellDataData.setInteger1(integer1);
        cellDataData.setInteger2(Integer.valueOf(2));
        WriteCellData<?> formulaValue = new WriteCellData<Object>();
        FormulaData formulaData = new FormulaData();
        formulaData.setFormulaValue("B2+C2");
        formulaValue.setFormulaData(formulaData);
        cellDataData.setFormulaValue(formulaValue);
        List<CellDataWriteData> list = new ArrayList<CellDataWriteData>();
        list.add(cellDataData);
        EasyExcel.write(target.toFile(), CellDataWriteData.class)
            .excelType(excelType)
            .sheet()
            .doWrite(list);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * CharsetData CSV write with explicit charset.
     */
    private static Path writeCharsetArtifact(Path artifactDir, String fileName, Charset charset)
        throws Exception {
        Path target = artifactDir.resolve(fileName);
        List<CharsetData> list = new ArrayList<CharsetData>();
        for (int i = 0; i < 10; i++) {
            CharsetData row = new CharsetData();
            row.setName("姓名" + i);
            row.setAge(Integer.valueOf(i));
            list.add(row);
        }
        EasyExcel.write(target.toFile(), CharsetData.class)
            .charset(charset)
            .excelType(ExcelTypeEnum.CSV)
            .sheet()
            .doWrite(list);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * ExceptionData — 姓名0..9.
     */
    private static Path writeExceptionArtifact(Path artifactDir, String fileName, ExcelTypeEnum excelType)
        throws Exception {
        Path target = artifactDir.resolve(fileName);
        List<ExceptionData> list = new ArrayList<ExceptionData>();
        for (int i = 0; i < 10; i++) {
            ExceptionData row = new ExceptionData();
            row.setName("姓名" + i);
            list.add(row);
        }
        EasyExcel.write(target.toFile(), ExceptionData.class)
            .excelType(excelType)
            .sheet()
            .doWrite(list);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * ExceptionDataTest multi-sheet stop fixture — sheet0..sheet4 × 10 rows.
     */
    private static Path writeExceptionStopSheetsArtifact(Path artifactDir) throws Exception {
        Path target = artifactDir.resolve("exception_stop.xlsx");
        try (ExcelWriter excelWriter = EasyExcel.write(target.toFile(), ExceptionData.class).build()) {
            for (int i = 0; i < 5; i++) {
                String sheetName = "sheet" + i;
                WriteSheet writeSheet = EasyExcel.writerSheet(i, sheetName).build();
                List<ExceptionData> data = new ArrayList<ExceptionData>();
                for (int j = 0; j < 10; j++) {
                    ExceptionData row = new ExceptionData();
                    row.setName(sheetName + "-姓名" + j);
                    data.add(row);
                }
                excelWriter.write(data, writeSheet);
            }
        }
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * WriteHandlerData — 姓名0..9（handler 不注册，仅 STRING 内容对照）.
     */
    private static Path writeHandlerArtifact(Path artifactDir, String fileName, ExcelTypeEnum excelType)
        throws Exception {
        Path target = artifactDir.resolve(fileName);
        List<WriteHandlerData> list = new ArrayList<WriteHandlerData>();
        for (int i = 0; i < 10; i++) {
            WriteHandlerData row = new WriteHandlerData();
            row.setName("姓名" + i);
            list.add(row);
        }
        EasyExcel.write(target.toFile(), WriteHandlerData.class)
            .excelType(excelType)
            .sheet()
            .doWrite(list);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * LargeDataTest 小样本：100 行 × str1..str25（非 large07 全量）.
     */
    private static Path writeLargeSampleArtifact(Path artifactDir, String fileName, ExcelTypeEnum excelType)
        throws Exception {
        Path target = artifactDir.resolve(fileName);
        List<List<String>> head = new ArrayList<List<String>>();
        for (int c = 1; c <= 25; c++) {
            List<String> col = new ArrayList<String>();
            col.add("str" + c);
            head.add(col);
        }
        List<List<Object>> rows = new ArrayList<List<Object>>();
        for (int i = 0; i < 100; i++) {
            List<Object> row = new ArrayList<Object>();
            for (int c = 1; c <= 25; c++) {
                row.add("str" + c + "-" + i);
            }
            rows.add(row);
        }
        EasyExcel.write(target.toFile())
            .excelType(excelType)
            .head(head)
            .sheet()
            .doWrite(rows);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * NoModelDataTest — List&lt;List&lt;Object&gt;&gt; 写（string / number / date）.
     */
    private static Path writeNoModelArtifact(Path artifactDir, String fileName, ExcelTypeEnum excelType)
        throws Exception {
        Path target = artifactDir.resolve(fileName);
        EasyExcel.write(target.toFile())
            .excelType(excelType)
            .sheet()
            .doWrite(noModelRows());
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * NoModelDataTest repeat write — re-write STRING-read rows.
     */
    private static Path writeNoModelRepeatArtifact(
        Path artifactDir,
        Path source,
        String outName,
        ExcelTypeEnum excelType
    ) throws Exception {
        Path target = artifactDir.resolve(outName);
        List<Map<Integer, Object>> result = EasyExcel.read(source.toFile())
            .headRowNumber(Integer.valueOf(0))
            .readDefaultReturn(ReadDefaultReturnEnum.STRING)
            .sheet()
            .doReadSync();
        List<List<Object>> rows = new ArrayList<List<Object>>();
        for (Map<Integer, Object> map : result) {
            List<Object> row = new ArrayList<Object>();
            row.add(map.get(Integer.valueOf(0)));
            row.add(map.get(Integer.valueOf(1)));
            row.add(map.get(Integer.valueOf(2)));
            rows.add(row);
        }
        EasyExcel.write(target.toFile())
            .excelType(excelType)
            .sheet()
            .doWrite(rows);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * @return NoModelDataTest#data() equivalent rows
     */
    private static List<List<Object>> noModelRows() throws Exception {
        List<List<Object>> list = new ArrayList<List<Object>>();
        for (int i = 0; i < 10; i++) {
            List<Object> data = new ArrayList<Object>();
            data.add("string1" + i);
            data.add(Integer.valueOf(100 + i));
            data.add(DateUtils.parseDate("2020-01-01 01:01:01"));
            list.add(data);
        }
        return list;
    }

    /**
     * UnCamelData write — non-camel field naming.
     */
    private static Path writeNonCamelArtifact(Path artifactDir, String fileName, ExcelTypeEnum excelType)
        throws Exception {
        Path target = artifactDir.resolve(fileName);
        List<UnCamelData> list = new ArrayList<UnCamelData>();
        for (int i = 0; i < 10; i++) {
            UnCamelData row = new UnCamelData();
            row.setString1("string1");
            row.setString2("string2");
            row.setSTring3("string3");
            row.setSTring4("string4");
            row.setSTRING5("string5");
            row.setSTRing6("string6");
            list.add(row);
        }
        EasyExcel.write(target.toFile(), UnCamelData.class)
            .excelType(excelType)
            .sheet()
            .doWrite(list);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * ParameterData — 姓名0..9.
     */
    private static Path writeParameterArtifact(Path artifactDir, String fileName, ExcelTypeEnum excelType)
        throws Exception {
        Path target = artifactDir.resolve(fileName);
        List<ParameterData> list = new ArrayList<ParameterData>();
        for (int i = 0; i < 10; i++) {
            ParameterData row = new ParameterData();
            row.setName("姓名" + i);
            list.add(row);
        }
        EasyExcel.write(target.toFile(), ParameterData.class)
            .excelType(excelType)
            .sheet()
            .doWrite(list);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * RepetitionData — write data() twice on the same sheet.
     */
    private static Path writeRepetitionArtifact(Path artifactDir, String fileName, ExcelTypeEnum excelType)
        throws Exception {
        Path target = artifactDir.resolve(fileName);
        List<RepetitionData> once = repetitionOnce();
        try (ExcelWriter excelWriter = EasyExcel.write(target.toFile(), RepetitionData.class)
            .excelType(excelType)
            .build()) {
            WriteSheet writeSheet = EasyExcel.writerSheet(0).build();
            excelWriter.write(once, writeSheet).write(once, writeSheet);
        }
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * RepetitionData table write — relativeHeadRowIndex + headRowNumber(2) on read.
     */
    private static Path writeRepetitionTableArtifact(Path artifactDir) throws Exception {
        Path target = artifactDir.resolve("repetition_table.xlsx");
        List<RepetitionData> once = repetitionOnce();
        try (ExcelWriter excelWriter = EasyExcel.write(target.toFile(), RepetitionData.class).build()) {
            WriteSheet writeSheet = EasyExcel.writerSheet(0).build();
            WriteTable writeTable = EasyExcel.writerTable(0).relativeHeadRowIndex(0).build();
            excelWriter.write(once, writeSheet, writeTable).write(once, writeSheet, writeTable);
        }
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * @return single RepetitionData row used by RepetitionDataTest#data()
     */
    private static List<RepetitionData> repetitionOnce() {
        List<RepetitionData> list = new ArrayList<RepetitionData>();
        RepetitionData data = new RepetitionData();
        data.setString("字符串0");
        list.add(data);
        return list;
    }

    /**
     * SkipDataTest — 4 named sheets with one row each.
     */
    private static Path writeSkipArtifact(Path artifactDir) throws Exception {
        Path target = artifactDir.resolve("skip_data.xlsx");
        try (ExcelWriter excelWriter = EasyExcel.write(target.toFile(), SkipData.class).build()) {
            WriteSheet writeSheet0 = EasyExcel.writerSheet(0, "第一个").build();
            WriteSheet writeSheet1 = EasyExcel.writerSheet(1, "第二个").build();
            WriteSheet writeSheet2 = EasyExcel.writerSheet(2, "第三个").build();
            WriteSheet writeSheet3 = EasyExcel.writerSheet(3, "第四个").build();
            excelWriter.write(skipOnce("name1"), writeSheet0);
            excelWriter.write(skipOnce("name2"), writeSheet1);
            excelWriter.write(skipOnce("name3"), writeSheet2);
            excelWriter.write(skipOnce("name4"), writeSheet3);
        }
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * @param name sheet row name
     * @return single-row SkipData list
     */
    private static List<SkipData> skipOnce(String name) {
        List<SkipData> list = new ArrayList<SkipData>();
        SkipData data = new SkipData();
        data.setName(name);
        list.add(data);
        return list;
    }

    /**
     * Write 姓名0..9 via Java EasyExcel into {@code artifactDir/fileName}.
     */
    private static Path writeSimpleArtifact(Path artifactDir, String fileName, ExcelTypeEnum excelType)
        throws Exception {
        Path target = artifactDir.resolve(fileName);
        List<SimpleData> data = new ArrayList<SimpleData>();
        for (int i = 0; i < 10; i++) {
            SimpleData row = new SimpleData();
            row.setName("姓名" + i);
            data.add(row);
        }
        EasyExcel.write(target.toFile(), SimpleData.class)
            .excelType(excelType)
            .sheet("Sheet1")
            .doWrite(data);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * Write ConverterWriteData (TestUtil-equivalent 2020-01-01 01:01:01 values).
     */
    private static Path writeConverterArtifact(Path artifactDir, String fileName, ExcelTypeEnum excelType)
        throws Exception {
        Path target = artifactDir.resolve(fileName);
        Date testDate = new SimpleDateFormat("yyyy-MM-dd HH:mm:ss").parse("2020-01-01 01:01:01");
        ConverterWriteData row = new ConverterWriteData();
        row.setDate(testDate);
        row.setLocalDate(LocalDate.of(2020, 1, 1));
        row.setLocalDateTime(LocalDateTime.of(2020, 1, 1, 1, 1, 1));
        row.setBooleanData(Boolean.TRUE);
        row.setBigDecimal(BigDecimal.ONE);
        row.setBigInteger(BigInteger.ONE);
        row.setLongData(1L);
        row.setIntegerData(Integer.valueOf(1));
        row.setShortData(Short.valueOf((short)1));
        row.setByteData(Byte.valueOf((byte)1));
        row.setDoubleData(1.0);
        row.setFloatData(Float.valueOf(1.0f));
        row.setString("测试");
        row.setCellData(new WriteCellData<String>("自定义"));
        List<ConverterWriteData> list = new ArrayList<ConverterWriteData>();
        list.add(row);
        EasyExcel.write(target.toFile(), ConverterWriteData.class)
            .excelType(excelType)
            .sheet()
            .doWrite(list);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * ComplexHeadDataTest — multi-level head + one data row.
     */
    private static Path writeComplexHeadArtifact(Path artifactDir, String fileName, ExcelTypeEnum excelType)
        throws Exception {
        Path target = artifactDir.resolve(fileName);
        ComplexHeadData data = new ComplexHeadData();
        data.setString0("字符串0");
        data.setString1("字符串1");
        data.setString2("字符串2");
        data.setString3("字符串3");
        data.setString4("字符串4");
        List<ComplexHeadData> list = new ArrayList<ComplexHeadData>();
        list.add(data);
        EasyExcel.write(target.toFile(), ComplexHeadData.class)
            .excelType(excelType)
            .sheet()
            .doWrite(list);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * AnnotationIndexAndNameDataTest — index + name column layout.
     */
    private static Path writeAnnotationIndexAndNameArtifact(
        Path artifactDir, String fileName, ExcelTypeEnum excelType
    ) throws Exception {
        Path target = artifactDir.resolve(fileName);
        AnnotationIndexAndNameData data = new AnnotationIndexAndNameData();
        data.setIndex0("第0个");
        data.setIndex1("第1个");
        data.setIndex2("第2个");
        data.setIndex4("第4个");
        List<AnnotationIndexAndNameData> list = new ArrayList<AnnotationIndexAndNameData>();
        list.add(data);
        EasyExcel.write(target.toFile(), AnnotationIndexAndNameData.class)
            .excelType(excelType)
            .sheet()
            .doWrite(list);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * ListHeadDataTest — dynamic head 字符串/数字/日期 + one data row.
     */
    private static Path writeListHeadArtifact(Path artifactDir, String fileName, ExcelTypeEnum excelType)
        throws Exception {
        Path target = artifactDir.resolve(fileName);
        List<List<String>> head = new ArrayList<List<String>>();
        List<String> head0 = new ArrayList<String>();
        head0.add("字符串");
        List<String> head1 = new ArrayList<String>();
        head1.add("数字");
        List<String> head2 = new ArrayList<String>();
        head2.add("日期");
        head.add(head0);
        head.add(head1);
        head.add(head2);
        List<Object> data0 = new ArrayList<Object>();
        data0.add("字符串0");
        data0.add(Integer.valueOf(1));
        data0.add(DateUtils.parseDate("2020-01-01 01:01:01"));
        data0.add("额外数据");
        List<List<Object>> rows = new ArrayList<List<Object>>();
        rows.add(data0);
        EasyExcel.write(target.toFile())
            .excelType(excelType)
            .head(head)
            .sheet()
            .doWrite(rows);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * Fill {@code fill/simple.xlsx} template → artifact {@code fill_simple.xlsx}.
     */
    private static Path writeFillArtifact(Path fixturesDir, Path artifactDir) throws Exception {
        Path template = fixturesDir.resolve("fill/simple.xlsx");
        if (!template.toFile().isFile()) {
            template = fixturesDir.resolve("demo/fill/simple.xlsx");
        }
        Path target = artifactDir.resolve("fill_simple.xlsx");
        FillData fillData = new FillData();
        fillData.setName("张三");
        fillData.setNumber(5.2);
        EasyExcel.write(target.toFile()).withTemplate(template.toFile()).sheet().doFill(fillData);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * StyleData write with column/row/horizontal style strategies (content对照).
     */
    private static Path writeStyleArtifact(Path artifactDir) throws Exception {
        Path target = artifactDir.resolve("style_data.xlsx");
        WriteCellStyle headWriteCellStyle = new WriteCellStyle();
        headWriteCellStyle.setFillForegroundColor(IndexedColors.YELLOW.getIndex());
        WriteFont headWriteFont = new WriteFont();
        headWriteFont.setFontHeightInPoints(Short.valueOf((short)20));
        headWriteFont.setColor(IndexedColors.DARK_YELLOW.getIndex());
        headWriteCellStyle.setWriteFont(headWriteFont);
        WriteCellStyle contentWriteCellStyle = new WriteCellStyle();
        contentWriteCellStyle.setFillPatternType(FillPatternType.SOLID_FOREGROUND);
        contentWriteCellStyle.setFillForegroundColor(IndexedColors.TEAL.getIndex());
        WriteFont contentWriteFont = new WriteFont();
        contentWriteFont.setFontHeightInPoints(Short.valueOf((short)30));
        contentWriteCellStyle.setWriteFont(contentWriteFont);
        HorizontalCellStyleStrategy horizontal =
            new HorizontalCellStyleStrategy(headWriteCellStyle, contentWriteCellStyle);

        List<StyleData> data = new ArrayList<StyleData>();
        StyleData row0 = new StyleData();
        row0.setString("字符串0");
        row0.setString1("字符串01");
        StyleData row1 = new StyleData();
        row1.setString("字符串1");
        row1.setString1("字符串11");
        data.add(row0);
        data.add(row1);

        EasyExcel.write(target.toFile(), StyleData.class)
            .registerWriteHandler(new SimpleColumnWidthStyleStrategy(50))
            .registerWriteHandler(new SimpleRowHeightStyleStrategy((short)40, (short)50))
            .registerWriteHandler(horizontal)
            .sheet()
            .doWrite(data);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * AnnotationData write (formatted date/number; ignore field omitted).
     */
    private static Path writeAnnotationArtifact(Path artifactDir) throws Exception {
        Path target = artifactDir.resolve("annotation_data.xlsx");
        Date testDate = new SimpleDateFormat("yyyy-MM-dd HH:mm:ss").parse("2020-01-01 01:01:01");
        AnnotationData row = new AnnotationData();
        row.setDate(testDate);
        row.setNumber(Double.valueOf(99.99));
        row.setIgnore("忽略");
        List<AnnotationData> list = new ArrayList<AnnotationData>();
        list.add(row);
        EasyExcel.write(target.toFile(), AnnotationData.class).sheet().doWrite(list);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * ExcludeOrInclude excludeColumnIndexes({0,3}) write.
     */
    private static Path writeExcludeArtifact(Path artifactDir) throws Exception {
        Path target = artifactDir.resolve("exclude_index.xlsx");
        ExcludeOrIncludeData row = new ExcludeOrIncludeData();
        row.setColumn1("column1");
        row.setColumn2("column2");
        row.setColumn3("column3");
        row.setColumn4("column4");
        List<ExcludeOrIncludeData> list = new ArrayList<ExcludeOrIncludeData>();
        list.add(row);
        Set<Integer> exclude = new HashSet<Integer>();
        exclude.add(Integer.valueOf(0));
        exclude.add(Integer.valueOf(3));
        EasyExcel.write(target.toFile(), ExcludeOrIncludeData.class)
            .excludeColumnIndexes(exclude)
            .sheet()
            .doWrite(list);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * EncryptData write with password {@code 123456}.
     */
    private static Path writeEncryptArtifact(Path artifactDir) throws Exception {
        Path target = artifactDir.resolve("encrypt_data.xlsx");
        List<EncryptData> data = new ArrayList<EncryptData>();
        for (int i = 0; i < 10; i++) {
            EncryptData row = new EncryptData();
            row.setName("姓名" + i);
            data.add(row);
        }
        EasyExcel.write(target.toFile(), EncryptData.class)
            .password("123456")
            .sheet()
            .doWrite(data);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * ExcludeOrInclude excludeColumnIndexes({0,3}) for the given excel type.
     *
     * @param artifactDir artifact root
     * @param fileName output file name
     * @param excelType XLSX / XLS / CSV
     * @param excludeIndex whether to apply exclude indexes
     * @return written path
     */
    private static Path writeExcludeArtifactTyped(
        Path artifactDir,
        String fileName,
        ExcelTypeEnum excelType,
        boolean excludeIndex
    ) throws Exception {
        Path target = artifactDir.resolve(fileName);
        ExcludeOrIncludeData row = sampleExcludeRow();
        List<ExcludeOrIncludeData> list = new ArrayList<ExcludeOrIncludeData>();
        list.add(row);
        com.alibaba.excel.write.builder.ExcelWriterBuilder builder =
            EasyExcel.write(target.toFile(), ExcludeOrIncludeData.class).excelType(excelType);
        if (excludeIndex) {
            Set<Integer> exclude = new HashSet<Integer>();
            exclude.add(Integer.valueOf(0));
            exclude.add(Integer.valueOf(3));
            builder.excludeColumnIndexes(exclude);
        }
        builder.sheet().doWrite(list);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * ExcludeOrInclude excludeColumnFieldNames(column1/3/4).
     */
    private static Path writeExcludeFieldArtifact(Path artifactDir, String fileName, ExcelTypeEnum excelType)
        throws Exception {
        Path target = artifactDir.resolve(fileName);
        List<ExcludeOrIncludeData> list = new ArrayList<ExcludeOrIncludeData>();
        list.add(sampleExcludeRow());
        Set<String> exclude = new HashSet<String>();
        exclude.add("column1");
        exclude.add("column3");
        exclude.add("column4");
        EasyExcel.write(target.toFile(), ExcludeOrIncludeData.class)
            .excelType(excelType)
            .excludeColumnFieldNames(exclude)
            .sheet()
            .doWrite(list);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * ExcludeOrInclude includeColumnIndexes({1,2}).
     */
    private static Path writeIncludeIndexArtifact(Path artifactDir, String fileName, ExcelTypeEnum excelType)
        throws Exception {
        Path target = artifactDir.resolve(fileName);
        List<ExcludeOrIncludeData> list = new ArrayList<ExcludeOrIncludeData>();
        list.add(sampleExcludeRow());
        Set<Integer> include = new HashSet<Integer>();
        include.add(Integer.valueOf(1));
        include.add(Integer.valueOf(2));
        EasyExcel.write(target.toFile(), ExcludeOrIncludeData.class)
            .excelType(excelType)
            .includeColumnIndexes(include)
            .sheet()
            .doWrite(list);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * ExcludeOrInclude includeColumnFieldNames(column2, column3).
     */
    private static Path writeIncludeFieldArtifact(Path artifactDir, String fileName, ExcelTypeEnum excelType)
        throws Exception {
        Path target = artifactDir.resolve(fileName);
        List<ExcludeOrIncludeData> list = new ArrayList<ExcludeOrIncludeData>();
        list.add(sampleExcludeRow());
        Set<String> include = new HashSet<String>();
        include.add("column2");
        include.add("column3");
        EasyExcel.write(target.toFile(), ExcludeOrIncludeData.class)
            .excelType(excelType)
            .sheet()
            .includeColumnFieldNames(include)
            .doWrite(list);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * ExcludeOrInclude includeColumnFieldNames order (column4, column2, column3).
     */
    private static Path writeIncludeFieldOrderArtifact(Path artifactDir) throws Exception {
        Path target = artifactDir.resolve("include_field_order.xlsx");
        List<ExcludeOrIncludeData> list = new ArrayList<ExcludeOrIncludeData>();
        list.add(sampleExcludeRow());
        List<String> include = new ArrayList<String>();
        include.add("column4");
        include.add("column2");
        include.add("column3");
        EasyExcel.write(target.toFile(), ExcludeOrIncludeData.class)
            .includeColumnFieldNames(include)
            .orderByIncludeColumn(true)
            .sheet()
            .doWrite(list);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * @return sample ExcludeOrIncludeData row used by Java ExcludeOrIncludeDataTest
     */
    private static ExcludeOrIncludeData sampleExcludeRow() {
        ExcludeOrIncludeData row = new ExcludeOrIncludeData();
        row.setColumn1("column1");
        row.setColumn2("column2");
        row.setColumn3("column3");
        row.setColumn4("column4");
        return row;
    }

    /**
     * Simple fill against a typed template path under fixtures.
     */
    private static Path writeFillSimpleTyped(
        Path fixturesDir,
        Path artifactDir,
        String outName,
        String templateRel,
        ExcelTypeEnum excelType
    ) throws Exception {
        Path template = fixturesDir.resolve(templateRel);
        Path target = artifactDir.resolve(outName);
        FillData fillData = new FillData();
        fillData.setName("张三");
        fillData.setNumber(5.2);
        EasyExcel.write(target.toFile())
            .excelType(excelType)
            .withTemplate(template.toFile())
            .sheet()
            .doFill(fillData);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * Horizontal fill mirroring FillDataTest#t05 / t06.
     */
    private static Path writeFillHorizontalArtifact(
        Path fixturesDir,
        Path artifactDir,
        String outName,
        String templateRel,
        ExcelTypeEnum excelType
    ) throws Exception {
        Path template = fixturesDir.resolve(templateRel);
        Path target = artifactDir.resolve(outName);
        List<FillData> rows = sampleFillRows();
        try (ExcelWriter excelWriter = EasyExcel.write(target.toFile())
            .excelType(excelType)
            .withTemplate(template.toFile())
            .build()) {
            WriteSheet writeSheet = EasyExcel.writerSheet().build();
            FillConfig fillConfig = FillConfig.builder().direction(WriteDirectionEnum.HORIZONTAL).build();
            excelWriter.fill(rows, fillConfig, writeSheet);
            excelWriter.fill(rows, fillConfig, writeSheet);
            Map<String, Object> map = new HashMap<String, Object>();
            map.put("date", "2019年10月9日13:28:28");
            excelWriter.fill(map, writeSheet);
        }
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * By-name sheet fill mirroring FillDataTest#t07ByNameFill07 / #t08 (Sheet2).
     */
    private static Path writeFillByNameArtifact(
        Path fixturesDir,
        Path artifactDir,
        String outName,
        String templateRel,
        ExcelTypeEnum excelType
    ) throws Exception {
        Path template = fixturesDir.resolve(templateRel);
        Path target = artifactDir.resolve(outName);
        FillData fillData = new FillData();
        fillData.setName("张三");
        fillData.setNumber(5.2);
        EasyExcel.write(target.toFile(), FillData.class)
            .excelType(excelType)
            .withTemplate(template.toFile())
            .sheet("Sheet2")
            .doFill(fillData);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * Complex fill mirroring FillDataTest#t03/#t04 — forceNewRow + LoopMergeStrategy.
     */
    private static Path writeFillComplexArtifact(
        Path fixturesDir,
        Path artifactDir,
        String outName,
        String templateRel,
        ExcelTypeEnum excelType
    ) throws Exception {
        Path template = fixturesDir.resolve(templateRel);
        Path target = artifactDir.resolve(outName);
        List<FillData> rows = sampleFillRowsWithNull();
        try (ExcelWriter excelWriter = EasyExcel.write(target.toFile())
            .excelType(excelType)
            .withTemplate(template.toFile())
            .build()) {
            WriteSheet writeSheet = EasyExcel.writerSheet()
                .registerWriteHandler(new LoopMergeStrategy(2, 0))
                .build();
            FillConfig fillConfig = FillConfig.builder().forceNewRow(Boolean.TRUE).build();
            excelWriter.fill(rows, fillConfig, writeSheet);
            excelWriter.fill(rows, fillConfig, writeSheet);
            Map<String, Object> map = new HashMap<String, Object>();
            map.put("date", "2019年10月9日13:28:28");
            map.put("total", Integer.valueOf(1000));
            excelWriter.fill(map, writeSheet);
        }
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * @return FillDataTest#data() — 10 rows, index 5 name null
     */
    private static List<FillData> sampleFillRowsWithNull() {
        List<FillData> list = new ArrayList<FillData>();
        for (int i = 0; i < 10; i++) {
            FillData fillData = new FillData();
            fillData.setName("张三");
            fillData.setNumber(5.2);
            if (i == 5) {
                fillData.setName(null);
            }
            list.add(fillData);
        }
        return list;
    }

    /**
     * @return 10 FillData rows (张三 / 5.2) matching FillDataTest#data()
     */
    private static List<FillData> sampleFillRows() {
        List<FillData> list = new ArrayList<FillData>();
        for (int i = 0; i < 10; i++) {
            FillData fillData = new FillData();
            fillData.setName("张三");
            fillData.setNumber(5.2);
            list.add(fillData);
        }
        return list;
    }

    /**
     * StyleData write for a given excel type.
     */
    private static Path writeStyleArtifactTyped(Path artifactDir, String fileName, ExcelTypeEnum excelType)
        throws Exception {
        Path target = artifactDir.resolve(fileName);
        WriteCellStyle headWriteCellStyle = new WriteCellStyle();
        headWriteCellStyle.setFillForegroundColor(IndexedColors.YELLOW.getIndex());
        WriteFont headWriteFont = new WriteFont();
        headWriteFont.setFontHeightInPoints(Short.valueOf((short)20));
        headWriteFont.setColor(IndexedColors.DARK_YELLOW.getIndex());
        headWriteCellStyle.setWriteFont(headWriteFont);
        WriteCellStyle contentWriteCellStyle = new WriteCellStyle();
        contentWriteCellStyle.setFillPatternType(FillPatternType.SOLID_FOREGROUND);
        contentWriteCellStyle.setFillForegroundColor(IndexedColors.TEAL.getIndex());
        WriteFont contentWriteFont = new WriteFont();
        contentWriteFont.setFontHeightInPoints(Short.valueOf((short)30));
        contentWriteCellStyle.setWriteFont(contentWriteFont);
        HorizontalCellStyleStrategy horizontal =
            new HorizontalCellStyleStrategy(headWriteCellStyle, contentWriteCellStyle);

        List<StyleData> data = new ArrayList<StyleData>();
        StyleData row0 = new StyleData();
        row0.setString("字符串0");
        row0.setString1("字符串01");
        StyleData row1 = new StyleData();
        row1.setString("字符串1");
        row1.setString1("字符串11");
        data.add(row0);
        data.add(row1);

        EasyExcel.write(target.toFile(), StyleData.class)
            .excelType(excelType)
            .registerWriteHandler(new SimpleColumnWidthStyleStrategy(50))
            .registerWriteHandler(new SimpleRowHeightStyleStrategy((short)40, (short)50))
            .registerWriteHandler(horizontal)
            .sheet()
            .doWrite(data);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * NoHeadData write with needHead(false) for a given excel type.
     */
    private static Path writeNoHeadArtifact(Path artifactDir, String fileName, ExcelTypeEnum excelType)
        throws Exception {
        Path target = artifactDir.resolve(fileName);
        NoHeadData row = new NoHeadData();
        row.setString("字符串0");
        List<NoHeadData> list = new ArrayList<NoHeadData>();
        list.add(row);
        EasyExcel.write(target.toFile(), NoHeadData.class)
            .excelType(excelType)
            .needHead(Boolean.FALSE)
            .sheet()
            .doWrite(list);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * SortData write with index/order columns.
     */
    private static Path writeSortArtifact(Path artifactDir) throws Exception {
        Path target = artifactDir.resolve("sort_data.xlsx");
        SortData row = new SortData();
        row.setColumn1("column1");
        row.setColumn2("column2");
        row.setColumn3("column3");
        row.setColumn4("column4");
        row.setColumn5("column5");
        row.setColumn6("column6");
        List<SortData> list = new ArrayList<SortData>();
        list.add(row);
        EasyExcel.write(target.toFile(), SortData.class).sheet().doWrite(list);
        System.out.println("Wrote artifact " + target);
        return target.toAbsolutePath().normalize();
    }

    /**
     * Read one fixture with Java EasyExcel STRING mode and build JSON payload.
     *
     * @param spec export specification
     * @return JSON-serializable map
     */
    private static Map<String, Object> exportOne(ExportSpec spec) {
        File file = spec.fixture.toFile();
        if (!file.isFile()) {
            throw new IllegalStateException("Fixture missing: " + file.getAbsolutePath());
        }

        com.alibaba.excel.read.builder.ExcelReaderBuilder reader = EasyExcel.read(file)
            .readDefaultReturn(ReadDefaultReturnEnum.STRING)
            .headRowNumber(Integer.valueOf(spec.headRowNumber));
        if (spec.password != null && !spec.password.isEmpty()) {
            reader.password(spec.password);
        }
        if (spec.charsetName != null && !spec.charsetName.isEmpty()) {
            reader.charset(Charset.forName(spec.charsetName));
        }
        List<Map<Integer, Object>> list;
        if (spec.sheetName != null && !spec.sheetName.isEmpty()) {
            list = reader.sheet(spec.sheetName).doReadSync();
        } else {
            list = reader.sheet(Integer.valueOf(spec.sheetIndex)).doReadSync();
        }

        List<List<String>> rows = new ArrayList<List<String>>();
        int maxCol = 0;
        for (Map<Integer, Object> row : list) {
            if (row == null) {
                continue;
            }
            for (Integer col : row.keySet()) {
                if (col != null && col.intValue() > maxCol) {
                    maxCol = col.intValue();
                }
            }
        }
        for (Map<Integer, Object> row : list) {
            List<String> cells = new ArrayList<String>();
            for (int c = 0; c <= maxCol; c++) {
                Object v = row == null ? null : row.get(Integer.valueOf(c));
                cells.add(displayText(v));
            }
            rows.add(cells);
        }

        Map<String, String> cells = new LinkedHashMap<String, String>();
        for (String coord : spec.keyCellCoords) {
            String[] parts = coord.split("\\.");
            int r = Integer.parseInt(parts[0]);
            int c = Integer.parseInt(parts[1]);
            if (r < 0 || r >= rows.size()) {
                throw new IllegalStateException(
                    "Key cell " + coord + " out of range for " + spec.source
                        + " (row_count=" + rows.size() + ")");
            }
            List<String> row = rows.get(r);
            String text = c < row.size() ? row.get(c) : "";
            cells.put(coord, text);
        }

        Map<String, Object> payload = new LinkedHashMap<String, Object>();
        payload.put("source", spec.source);
        payload.put("fixture", relativizeFixture(spec.fixture));
        payload.put("sheet_index", Integer.valueOf(spec.sheetIndex));
        if (spec.sheetName != null && !spec.sheetName.isEmpty()) {
            payload.put("sheet_name", spec.sheetName);
        }
        payload.put("head_row_number", Integer.valueOf(spec.headRowNumber));
        if (spec.password != null && !spec.password.isEmpty()) {
            payload.put("password", spec.password);
        }
        if (spec.charsetName != null && !spec.charsetName.isEmpty()) {
            payload.put("charset", spec.charsetName);
        }
        payload.put("row_count", Integer.valueOf(rows.size()));
        payload.put("cells", cells);
        if (spec.includeRows) {
            payload.put("rows", rows);
        } else {
            payload.put("rows", new ArrayList<List<String>>());
        }
        return payload;
    }

    /**
     * Convert a Java EasyExcel cell Object to display text.
     *
     * @param value cell value (usually String in STRING mode)
     * @return display string; empty for null
     */
    private static String displayText(Object value) {
        if (value == null) {
            return "";
        }
        return Objects.toString(value, "");
    }

    /**
     * @param coords row.col keys
     * @return list of coordinate strings
     */
    private static List<String> keyCells(String... coords) {
        List<String> list = new ArrayList<String>();
        for (String c : coords) {
            list.add(c);
        }
        return list;
    }

    /**
     * Prefer a stable relative fixture path ending at {@code fixtures/...} or {@code artifacts/...}.
     *
     * @param fixture absolute fixture path
     * @return relative-looking path for JSON
     */
    private static String relativizeFixture(Path fixture) {
        String s = fixture.toString().replace('\\', '/');
        int idx = s.lastIndexOf("/fixtures/");
        if (idx >= 0) {
            return s.substring(idx + "/fixtures/".length());
        }
        int art = s.lastIndexOf("/artifacts/");
        if (art >= 0) {
            return "artifacts/" + s.substring(art + "/artifacts/".length());
        }
        return fixture.getFileName().toString();
    }

    /**
     * One fixture → one expected JSON file.
     */
    private static final class ExportSpec {
        private final String source;
        private final Path fixture;
        private final String outName;
        private final int sheetIndex;
        private final String sheetName;
        private final int headRowNumber;
        private final String password;
        private final String charsetName;
        private final boolean includeRows;
        private final List<String> keyCellCoords;

        private ExportSpec(
            String source,
            Path fixture,
            String outName,
            int sheetIndex,
            String sheetName,
            int headRowNumber,
            String password,
            String charsetName,
            boolean includeRows,
            List<String> keyCellCoords
        ) {
            this.source = source;
            this.fixture = fixture;
            this.outName = outName;
            this.sheetIndex = sheetIndex;
            this.sheetName = sheetName;
            this.headRowNumber = headRowNumber;
            this.password = password;
            this.charsetName = charsetName;
            this.includeRows = includeRows;
            this.keyCellCoords = keyCellCoords;
        }

        private static ExportSpec of(
            String source,
            Path fixture,
            String outName,
            int sheetIndex,
            int headRowNumber,
            List<String> keyCellCoords
        ) {
            return new ExportSpec(
                source, fixture, outName, sheetIndex, null, headRowNumber, null, null, true, keyCellCoords);
        }

        /** Key-cell + row_count only (omit full rows when a known STRING format gap remains). */
        private static ExportSpec ofNoRows(
            String source,
            Path fixture,
            String outName,
            int sheetIndex,
            int headRowNumber,
            List<String> keyCellCoords
        ) {
            return new ExportSpec(
                source, fixture, outName, sheetIndex, null, headRowNumber, null, null, false, keyCellCoords);
        }

        private static ExportSpec withSheetName(
            String source,
            Path fixture,
            String outName,
            String sheetName,
            int headRowNumber,
            List<String> keyCellCoords
        ) {
            return new ExportSpec(
                source, fixture, outName, 0, sheetName, headRowNumber, null, null, true, keyCellCoords);
        }

        private static ExportSpec withPassword(
            String source,
            Path fixture,
            String outName,
            int sheetIndex,
            int headRowNumber,
            String password,
            List<String> keyCellCoords
        ) {
            return new ExportSpec(
                source, fixture, outName, sheetIndex, null, headRowNumber, password, null, true, keyCellCoords);
        }

        /**
         * CSV charset-aware export (full rows).
         */
        private static ExportSpec withCharset(
            String source,
            Path fixture,
            String outName,
            int sheetIndex,
            int headRowNumber,
            String charsetName,
            List<String> keyCellCoords
        ) {
            return new ExportSpec(
                source, fixture, outName, sheetIndex, null, headRowNumber, null, charsetName, true,
                keyCellCoords);
        }
    }
}
