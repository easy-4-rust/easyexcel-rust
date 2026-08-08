/// 写出工作表内嵌图表的 Escher 锚点、OBJ 与 Chart 子流。
///
/// 记录骨架逐项对齐 POI 5.2.5 `HSSFChart#createBarChart`；系列、AI 区域、
/// 锚点和图表类型由当前工作簿模型动态生成。
fn write_charts(
    out: &mut Vec<u8>,
    charts: &[Biff8Chart],
    link_table: &super::ptg::Biff8LinkTable,
) {
    for chart in charts {
        write_chart_drawing(out, chart);
        write_chart_substream(out, chart, link_table);
    }
}

fn chart_drawing_group() -> Vec<u8> {
    hex_bytes(
        "0F0000F052000000000006F01800000001080000020000000200000001000000\
         010000000300000033000BF012000000BF0008000800810109000008C0014000\
         000840001EF1100000000D0000080C00000817000008F7000010",
    )
}

fn write_chart_drawing(out: &mut Vec<u8>, chart: &Biff8Chart) {
    let mut drawing = hex_bytes(
        "0F0002F0C0000000100008F00800000002000000020400000F0003F0A8000000\
         0F0004F028000000010009F01000000000000000000000000000000000000000\
         02000AF00800000000040000050000000F0004F070000000920C0AF008000000\
         02040000000A000093000BF0360000007F0004010401BF000800080081014E00\
         000883014D000008BF0110001100C0014D000008FF01080008003F0200000200\
         BF0300000800000010F01200000000000400C0020A00F4000E0066012000E900\
         000011F000000000",
    );
    let mut anchor = Vec::with_capacity(18);
    anchor.extend_from_slice(&0u16.to_le_bytes());
    anchor.extend_from_slice(&u16::from(chart.first_column).to_le_bytes());
    anchor.extend_from_slice(&0u16.to_le_bytes());
    anchor.extend_from_slice(&chart.first_row.to_le_bytes());
    anchor.extend_from_slice(&0u16.to_le_bytes());
    anchor.extend_from_slice(&u16::from(chart.last_column).to_le_bytes());
    anchor.extend_from_slice(&0u16.to_le_bytes());
    anchor.extend_from_slice(&chart.last_row.to_le_bytes());
    anchor.extend_from_slice(&0u16.to_le_bytes());
    drawing[174..192].copy_from_slice(&anchor);
    record(out, MSODRAWING, &drawing);
    record(
        out,
        OBJ,
        &hex_bytes("1500120005000200116000000000B80387030000000000000000"),
    );
}

fn write_chart_substream(
    out: &mut Vec<u8>,
    chart: &Biff8Chart,
    link_table: &super::ptg::Biff8LinkTable,
) {
    const DT_CHART: u16 = 0x0020;
    write_bof(out, DT_CHART);
    for (sid, payload) in [
        (0x0014, ""),
        (0x0015, ""),
        (0x0083, "0000"),
        (0x0084, "0000"),
        (0x00A1, "00001200010001000100040000000000000000000000E03F000000000000E03F0F00"),
        (0x1060, "A0230816C80000000500"),
        (0x1060, "A0230816C80000000600"),
        (0x0012, "0000"),
        (0x1001, "0000"),
        (0x1002, "00000000000000005866D00140662201"),
        (0x1033, ""),
        (0x00A0, "01000100"),
        (0x1064, "0000010000000100"),
        (0x1032, "00000200"),
        (0x1033, ""),
        (0x1007, "000000000000FFFF05004D00"),
        (0x100A, "FFFFFF0000000000010001004E004D00"),
        (0x1034, ""),
    ] {
        record_hex(out, sid, payload);
    }

    for (series_index, series) in chart.series.iter().enumerate() {
        write_chart_series(out, series, series_index, link_table);
    }

    write_chart_suffix(out, chart.kind, chart.title.as_deref());
    record(out, EOF, &[]);
}

fn write_chart_series(
    out: &mut Vec<u8>,
    series: &Biff8ChartSeries,
    series_index: usize,
    link_table: &super::ptg::Biff8LinkTable,
) {
    let categories_count = series
        .categories
        .as_ref()
        .map_or(series.values.cell_count(), Biff8ChartRange::cell_count);
    let mut series_payload = Vec::with_capacity(12);
    series_payload.extend_from_slice(&1u16.to_le_bytes());
    series_payload.extend_from_slice(&1u16.to_le_bytes());
    series_payload.extend_from_slice(&categories_count.to_le_bytes());
    series_payload.extend_from_slice(&series.values.cell_count().to_le_bytes());
    series_payload.extend_from_slice(&1u16.to_le_bytes());
    series_payload.extend_from_slice(&0u16.to_le_bytes());
    record(out, 0x1003, &series_payload);
    record(out, 0x1033, &[]);
    record(out, 0x1051, &hex_bytes("0001000000000000"));
    if let Some(name) = &series.name {
        write_series_text(out, name);
    }
    if let Some(categories) = &series.categories {
        record(out, 0x1051, &chart_ai_payload(2, categories, link_table));
    } else {
        record(out, 0x1051, &hex_bytes("0201000000000000"));
    }
    record(out, 0x1051, &chart_ai_payload(1, &series.values, link_table));
    let index = u16::try_from(series_index).unwrap_or(u16::MAX);
    let mut data_format = Vec::with_capacity(8);
    data_format.extend_from_slice(&u16::MAX.to_le_bytes());
    data_format.extend_from_slice(&index.to_le_bytes());
    data_format.extend_from_slice(&index.to_le_bytes());
    data_format.extend_from_slice(&0u16.to_le_bytes());
    record(out, 0x1006, &data_format);
    record(out, 0x1045, &0u16.to_le_bytes());
    record(out, 0x1034, &[]);
}

fn chart_ai_payload(
    link_type: u8,
    range: &Biff8ChartRange,
    link_table: &super::ptg::Biff8LinkTable,
) -> Vec<u8> {
    let ixti = link_table
        .ixti(&range.sheet_name, &range.sheet_name)
        .unwrap_or(u16::MAX);
    let mut payload = Vec::with_capacity(19);
    payload.push(link_type);
    payload.push(2);
    payload.extend_from_slice(&0u16.to_le_bytes());
    payload.extend_from_slice(&0u16.to_le_bytes());
    payload.extend_from_slice(&11u16.to_le_bytes());
    payload.push(0x3B);
    payload.extend_from_slice(&ixti.to_le_bytes());
    payload.extend_from_slice(&range.first_row.to_le_bytes());
    payload.extend_from_slice(&range.last_row.to_le_bytes());
    payload.extend_from_slice(&u16::from(range.first_column).to_le_bytes());
    payload.extend_from_slice(&u16::from(range.last_column).to_le_bytes());
    payload
}

fn write_chart_suffix(out: &mut Vec<u8>, kind: Biff8ChartKind, title: Option<&str>) {
    for (sid, payload) in [
        (0x1044, "0A000000"),
        (0x1024, "0200"),
        (0x1025, "0202010000000000DBFFFFFFC4FFFFFF0000000000000000B5004D0000000000"),
        (0x1033, ""),
        (0x1026, "0500"),
        (0x1051, "0001000000000000"),
        (0x1034, ""),
        (0x1024, "0300"),
        (0x1025, "0202010000000000DBFFFFFFC4FFFFFF0000000000000000B1004D0000000000"),
        (0x1033, ""),
        (0x1026, "0600"),
        (0x1051, "0001000000000000"),
        (0x1034, ""),
        (0x1046, "0100"),
        (0x1041, "0000DF010000DD000000B30B0000560B0000"),
        (0x1033, ""),
        (0x101D, "000000000000000000000000000000000000"),
        (0x1033, ""),
        (0x1020, "0100010001000100"),
        (0x1062, "1C90D58F020000000100000000001C90FF00"),
        (0x101E, "02000301000000000000000000000000000000000000000022004D002D00"),
        (0x1034, ""),
        (0x101D, "010000000000000000000000000000000000"),
        (0x1033, ""),
        (0x101F, "000000000000000000000000000000000000000000000000000000000000000000000000000000001F01"),
        (0x101E, "02000301000000000000000000000000000000000000000022004D000000"),
        (0x1021, "0100"),
        (0x1007, "000000000000FFFF01004D00"),
        (0x1034, ""),
        (0x1035, ""),
        (0x1032, "00000300"),
        (0x1033, ""),
        (0x1007, "808080000000000000001700"),
        (0x100A, "C0C0C000000000000100000016004F00"),
        (0x1034, ""),
        (0x1014, "0000000000000000000000000000000000000000"),
        (0x1033, ""),
    ] {
        record_hex(out, sid, payload);
    }
    match kind {
        Biff8ChartKind::Bar => record_hex(out, 0x1017, "000096000000"),
        Biff8ChartKind::Line => record_hex(out, 0x1018, "0000"),
        Biff8ChartKind::Pie => record_hex(out, 0x1019, "000000000000"),
    }
    record_hex(out, 0x1015, "D60D00001E060000B5010000D500000003011F00");
    for (sid, payload) in [
        (0x1033, ""),
        (0x1025, "0202010000000000DBFFFFFFC4FFFFFF0000000000000000B1004D0000000000"),
        (0x1033, ""),
        (0x1051, "0001000000000000"),
        (0x1034, ""),
    ] {
        record_hex(out, sid, payload);
    }
    for (sid, payload) in [(0x1034, ""), (0x1034, ""), (0x1034, "")] {
        record_hex(out, sid, payload);
    }
    if let Some(title) = title {
        write_chart_title_group(out, title);
    }
    for (sid, payload) in [
        (0x1034, ""),
        (0x0200, "000000001F000000000001000000"),
        (0x1065, "0200"),
        (0x1065, "0100"),
        (0x1065, "0300"),
    ] {
        record_hex(out, sid, payload);
    }
}

fn write_chart_title_group(out: &mut Vec<u8>, title: &str) {
    for (sid, payload) in [
        (0x1025, "0202010000000000000000000000000000000000000000008000080000000000"),
        (0x1033, ""),
        (0x104F, "0200020000000000000000000000000000000000"),
        (0x1026, "0600"),
        (0x1051, "0001000000000000"),
    ] {
        record_hex(out, sid, payload);
    }
    write_series_text(out, title);
    record_hex(out, 0x1027, "010000000000");
    record(out, 0x1034, &[]);
}

fn write_series_text(out: &mut Vec<u8>, text: &str) {
    let units = text.encode_utf16().collect::<Vec<_>>();
    let compressed = units.iter().all(|unit| *unit <= 0xFF);
    let mut payload = Vec::with_capacity(4 + units.len() * usize::from(!compressed) + units.len());
    payload.extend_from_slice(&0u16.to_le_bytes());
    payload.push(u8::try_from(units.len()).unwrap_or(u8::MAX));
    payload.push(u8::from(!compressed));
    if compressed {
        payload.extend(units.into_iter().map(|unit| u8::try_from(unit).unwrap_or(b'?')));
    } else {
        for unit in units {
            payload.extend_from_slice(&unit.to_le_bytes());
        }
    }
    record(out, 0x100D, &payload);
}

fn record_hex(out: &mut Vec<u8>, sid: u16, payload: &str) {
    record(out, sid, &hex_bytes(payload));
}

fn hex_bytes(value: &str) -> Vec<u8> {
    let compact = value
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect::<Vec<_>>();
    assert!(compact.len().is_multiple_of(2), "static BIFF8 hex payload");
    compact
        .chunks_exact(2)
        .map(|pair| (hex_nibble(pair[0]) << 4) | hex_nibble(pair[1]))
        .collect()
}

fn hex_nibble(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        b'A'..=b'F' => value - b'A' + 10,
        _ => panic!("invalid static BIFF8 hex payload"),
    }
}

#[cfg(test)]
mod chart_tests {
    use std::io::{Cursor, Read};

    use super::*;

    #[test]
    fn generated_bar_line_and_pie_emit_anchor_series_and_type_records() {
        for (kind, expected_sid) in [
            (Biff8ChartKind::Bar, 0x1017),
            (Biff8ChartKind::Line, 0x1018),
            (Biff8ChartKind::Pie, 0x1019),
        ] {
            let mut book = Biff8Book::default();
            let sheet = book.create_sheet("Data").expect("sheet");
            let series = Biff8ChartSeries::new(Biff8ChartRange::new("Data", 0, 1, 2, 1))
                .with_categories(Biff8ChartRange::new("Data", 0, 0, 2, 0));
            sheet.add_chart(Biff8Chart::new(kind, 4, 3, 20, 12).with_series(series));
            let bytes = book.to_cfb_bytes().expect("chart workbook");
            let mut compound = cfb::CompoundFile::open(Cursor::new(bytes)).expect("cfb");
            let mut workbook = Vec::new();
            compound
                .open_stream("Workbook")
                .expect("Workbook stream")
                .read_to_end(&mut workbook)
                .expect("read stream");
            assert!(has_record(&workbook, expected_sid));
            assert!(has_record(&workbook, MSODRAWINGGROUP));
            assert!(has_record(&workbook, MSODRAWING));
            assert!(record_payloads(&workbook, 0x1051).iter().any(|payload| {
                *payload == hex_bytes("0102000000000B003B00000000020001000100")
            }));
            let drawing = record_payloads(&workbook, MSODRAWING)
                .into_iter()
                .next()
                .expect("drawing");
            assert_eq!(&drawing[174..192], &hex_bytes("000003000000040000000C00000014000000"));
        }
    }

    #[test]
    fn generated_chart_serializes_chart_and_series_titles() {
        let mut book = Biff8Book::default();
        let sheet = book.create_sheet("Data").expect("sheet");
        let series = Biff8ChartSeries::new(Biff8ChartRange::new("Data", 0, 1, 2, 1))
            .with_name("Amount");
        sheet.add_chart(
            Biff8Chart::new(Biff8ChartKind::Bar, 4, 3, 20, 12)
                .with_title("Sales")
                .with_series(series),
        );
        let bytes = book.to_cfb_bytes().expect("titled chart");
        let mut compound = cfb::CompoundFile::open(Cursor::new(bytes)).expect("cfb");
        let mut workbook = Vec::new();
        compound
            .open_stream("Workbook")
            .expect("Workbook stream")
            .read_to_end(&mut workbook)
            .expect("read stream");
        let titles = record_payloads(&workbook, 0x100D);
        assert!(titles.iter().any(|payload| payload.ends_with(b"Amount")));
        assert!(titles.iter().any(|payload| payload.ends_with(b"Sales")));
    }

    #[test]
    fn multiple_generated_charts_and_comments_emit_independent_drawing_groups() {
        let mut book = Biff8Book::default();
        let sheet = book.create_sheet("Data").expect("sheet");
        sheet
            .add_comment(0, 0, "note", "easyexcel")
            .expect("comment");
        for (kind, first_row) in [(Biff8ChartKind::Bar, 4), (Biff8ChartKind::Line, 24)] {
            let series = Biff8ChartSeries::new(Biff8ChartRange::new("Data", 0, 1, 2, 1));
            sheet.add_chart(
                Biff8Chart::new(kind, first_row, 3, first_row + 16, 12).with_series(series),
            );
        }
        let bytes = book.to_cfb_bytes().expect("drawings workbook");
        let mut compound = cfb::CompoundFile::open(Cursor::new(bytes)).expect("cfb");
        let mut workbook = Vec::new();
        compound
            .open_stream("Workbook")
            .expect("Workbook stream")
            .read_to_end(&mut workbook)
            .expect("read stream");
        assert_eq!(record_payloads(&workbook, MSODRAWINGGROUP).len(), 3);
        assert_eq!(record_payloads(&workbook, 0x1002).len(), 2);
        assert_eq!(record_payloads(&workbook, NOTE).len(), 1);
    }

    fn has_record(bytes: &[u8], sid: u16) -> bool {
        !record_payloads(bytes, sid).is_empty()
    }

    fn record_payloads(bytes: &[u8], wanted_sid: u16) -> Vec<Vec<u8>> {
        let mut payloads = Vec::new();
        let mut offset = 0usize;
        while offset + 4 <= bytes.len() {
            let sid = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
            let length = usize::from(u16::from_le_bytes([bytes[offset + 2], bytes[offset + 3]]));
            if offset + 4 + length > bytes.len() {
                break;
            }
            if sid == wanted_sid {
                payloads.push(bytes[offset + 4..offset + 4 + length].to_vec());
            }
            offset += 4 + length;
        }
        payloads
    }
}
