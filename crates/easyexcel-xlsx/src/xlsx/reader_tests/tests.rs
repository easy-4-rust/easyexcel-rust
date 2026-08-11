#![allow(clippy::too_many_lines)]
use super::*;
use crate::xlsx::write;
use easyexcel_io::ResourceLimits;
use easyexcel_model::styles::{CellStyle, HAlign};
use std::io::{Cursor, Write};

fn roundtrip(wb: &Workbook) -> Workbook {
    let mut buf = Vec::new();
    write(wb, Cursor::new(&mut buf)).expect("write");
    read(Cursor::new(buf)).expect("read")
}

/// 构造一个包含高压缩比条目的 ZIP 文件（模拟 ZIP bomb）。
///
/// 写入 `entry_size` 字节的全零数据到 ZIP 中。全零数据压缩比极高，
/// 可用于测试 ZIP bomb 防护是否正确拒绝。
fn make_zip_bomb(entry_size: usize) -> Vec<u8> {
    use zip::write::SimpleFileOptions;
    let mut buf = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(Cursor::new(&mut buf));
        zip.start_file("xl/workbook.xml", SimpleFileOptions::default())
            .expect("start entry");
        // 全零数据：压缩比极高（典型 > 1000:1）
        let zeros = vec![0u8; entry_size];
        zip.write_all(&zeros).expect("write entry");
        zip.finish().expect("finish zip");
    }
    buf
}

/// 构造一个包含多个小条目的 ZIP 文件（模拟累积 ZIP bomb）。
fn make_zip_bomb_multi_entry(entry_count: usize, entry_size: usize) -> Vec<u8> {
    use zip::write::SimpleFileOptions;
    let mut buf = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(Cursor::new(&mut buf));
        for i in 0..entry_count {
            let name = format!("xl/sheet{i}.xml");
            zip.start_file(name, SimpleFileOptions::default())
                .expect("start entry");
            let zeros = vec![0u8; entry_size];
            zip.write_all(&zeros).expect("write entry");
        }
        zip.finish().expect("finish zip");
    }
    buf
}

include!("tests/cases_01.rs");
