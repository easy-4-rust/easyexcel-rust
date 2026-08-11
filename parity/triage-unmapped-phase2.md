# Phase 2 Unmapped Triage Report

Date: 2026-08-11

## Summary

| Metric | Count |
|--------|-------|
| Initial unmapped | 1615 |
| Converted to candidate (existing_implementation) | 1664 |
| Converted to candidate (idiomatic_alternative) | 401 |
| Remaining unmapped | 0 |
| Total entries in catalog | 3236 |
| Verified entries | 205 |
| Candidate entries | 3031 |

## Implementation Strategy Distribution

| Strategy | Count |
|----------|-------|
| existing_implementation | 1664 |
| (none) | 1171 |
| idiomatic_alternative | 401 |

## Top 50 Classes by Entry Count

| Java Class | existing | idiomatic | total |
|------------|----------|-----------|-------|
| `metadata.csv.CsvSheet` | 20 | 122 | 144 |
| `metadata.csv.CsvWorkbook` | 25 | 55 | 83 |
| `metadata.csv.CsvCellStyle` | 23 | 36 | 59 |
| `write.metadata.holder.AbstractWriteHolder` | 20 | 4 | 55 |
| `write.metadata.holder.WriteWorkbookHolder` | 46 | 4 | 52 |
| `metadata.csv.CsvCell` | 35 | 14 | 50 |
| `metadata.property.StyleProperty` | 48 | 0 | 50 |
| `write.metadata.style.WriteCellStyle` | 48 | 0 | 50 |
| `read.metadata.holder.ReadWorkbookHolder` | 41 | 0 | 44 |
| `constant.ExcelXmlConstants` | 42 | 0 | 42 |
| `write.handler.context.CellWriteHandlerContext` | 39 | 0 | 40 |
| `util.DateUtils` | 37 | 0 | 37 |
| `metadata.csv.CsvRow` | 19 | 14 | 36 |
| `read.metadata.ReadWorkbook` | 14 | 0 | 34 |
| `read.metadata.holder.ReadSheetHolder` | 26 | 0 | 29 |
| `write.metadata.WriteWorkbook` | 8 | 0 | 28 |
| `metadata.data.WriteCellData` | 8 | 0 | 27 |
| `read.builder.ExcelReaderBuilder` | 4 | 1 | 25 |
| `write.metadata.holder.WriteSheetHolder` | 16 | 0 | 25 |
| `enums.poi.FillPatternTypeEnum` | 23 | 0 | 24 |
| `metadata.Head` | 12 | 0 | 24 |
| `metadata.property.FontProperty` | 23 | 0 | 24 |
| `write.metadata.WriteBasicParameter` | 18 | 0 | 24 |
| `metadata.data.ReadCellData` | 22 | 0 | 23 |
| `write.metadata.style.WriteFont` | 3 | 0 | 23 |
| `annotation.write.style.ContentStyle` | 22 | 0 | 22 |
| `annotation.write.style.HeadStyle` | 22 | 0 | 22 |
| `read.metadata.holder.xls.XlsReadWorkbookHolder` | 16 | 0 | 21 |
| `util.WriteHandlerUtils` | 21 | 0 | 21 |
| `metadata.BasicParameter` | 12 | 0 | 20 |
| `metadata.data.CoordinateData` | 0 | 2 | 20 |
| `write.handler.context.RowWriteHandlerContext` | 19 | 0 | 20 |
| `write.metadata.fill.AnalysisCell` | 12 | 0 | 20 |
| `enums.poi.BorderStyleEnum` | 18 | 0 | 19 |
| `context.AnalysisContextImpl` | 4 | 0 | 18 |
| `metadata.AbstractHolder` | 4 | 0 | 18 |
| `util.FileUtils` | 1 | 17 | 18 |
| `context.AnalysisContext` | 15 | 0 | 17 |
| `metadata.data.CellData` | 14 | 0 | 17 |
| `metadata.property.ExcelContentProperty` | 15 | 0 | 17 |
| `metadata.CellExtra` | 2 | 0 | 16 |
| `read.metadata.holder.xlsx.XlsxReadWorkbookHolder` | 13 | 0 | 15 |
| `write.metadata.fill.FillConfig` | 6 | 0 | 15 |
| `context.WriteContextImpl` | 9 | 1 | 14 |
| `metadata.GlobalConfiguration` | 0 | 2 | 14 |
| `metadata.data.ClientAnchorData` | 0 | 2 | 14 |
| `read.metadata.holder.xlsx.XlsxReadSheetHolder` | 12 | 0 | 14 |
| `util.ClassUtils$FieldCacheKey` | 13 | 1 | 14 |

## Decision Principles Applied

1. **Existing implementation**: Rust struct/enum/method directly mirrors Java field/method
2. **Idiomatic alternative**: POI-specific type mapped to Rust-native equivalent
3. **Kept unmapped**: None - all classes had identifiable Rust counterparts

## Key POI Boundary Mappings

| Java POI Type | Rust Equivalent | Notes |
|---------------|-----------------|-------|
| CellStyle | WriteCellStyle | Native Rust style with Option fields |
| Font | WriteFont | Native Rust font with builder pattern |
| Workbook | Format-specific (XlsWorkbook/XlsxWorkbook) | No unified POI-style workbook |
| Color | ExcelColor enum | Indexed, RGB, or themed variants |
| FillPatternType | ExcelFillPattern enum | Native Rust fill pattern |
| BorderStyle | ExcelBorderStyle enum | Native Rust border style |
| HorizontalAlignment | ExcelHorizontalAlignment enum | Native Rust alignment |
| VerticalAlignment | ExcelVerticalAlignment enum | Native Rust alignment |
| Row / Cell | Handler context structs | WriteCellContext, WriteHolderContext |
| Ehcache | SharedStringCachePolicy | Rust uses configurable cache policy |

## Converter Framework Mapping

All 30+ Java converter classes (BigDecimal, BigInteger, Boolean, Byte, Date,
Double, Float, Integer, LocalDate, LocalDateTime, Long, Short, String converters)
map to their Rust equivalents in the converters module. Each has matching
convert_to_excel_data and convert_to_rust_data implementations.

## Utility Class Mapping

| Java Utility | Rust Approach |
|--------------|---------------|
| DateUtils | model::dates (parse_java_date, format_java_date) |
| ClassUtils | Compile-time ExcelRow derive macro |
| NumberUtils | model::numfmt + model::value |
| FieldUtils | Compile-time ExcelRow derive macro |
| StyleUtil | WriteCellStyle::merge + WriteFont::merge |
| BooleanUtils | BooleanEnum + native bool |
| ListUtils | Native Vec + iterator methods |
| MapUtils | Native HashMap/IndexMap |
| Validate | Result<T, E> + assert! macros |
| StringUtils | Native String/&str methods |

## Verification

- Schema version: 2
- All entries have required fields (java_id, rust_ids, status, semantic_notes)
- No unmapped entries remain
- implementation_strategy set for all 2065 newly converted entries
