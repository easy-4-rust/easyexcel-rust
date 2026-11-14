//! Regression fixtures from calamine's RUSTSEC-2026-0436 fix:
//! Upstream fix: <https://github.com/tafia/calamine/commit/b8ec4f09cba6aea7d8359ababc240c9ed064751b>
//!
//! Both files are deliberately malformed OLE2/CFB inputs. They must only be
//! parsed by patched calamine (>= 0.38.0).

use std::io::{Cursor, Read};
#[cfg(target_os = "linux")]
use std::process::Command;

use calamine::{Reader, Xls};

#[test]
fn malformed_cfb_short_directory_chunk_does_not_panic() {
    let bytes = include_bytes!("fixtures/calamine/cfb_dir_short_chunk_panic.xls");
    let workbook = Xls::new(Cursor::new(bytes.as_slice()))
        .expect("patched CFB parser should skip a short trailing directory chunk");
    assert_eq!(workbook.sheet_names(), vec!["Sheet1"]);
}

#[test]
fn malformed_cfb_cyclic_difat_is_rejected_without_unbounded_growth() {
    let bytes = include_bytes!("fixtures/calamine/cfb_difat_oom.xls");
    assert!(
        Xls::new(Cursor::new(bytes.as_slice())).is_err(),
        "patched CFB parser must reject a cyclic DIFAT chain"
    );
}

fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn put_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn write_directory_entry(
    bytes: &mut [u8],
    name: &str,
    entry_type: u8,
    child: u32,
    start_sector: u32,
    stream_len: u64,
) {
    for (index, unit) in name.encode_utf16().enumerate() {
        put_u16(bytes, index * 2, unit);
    }
    let name_len = u16::try_from((name.encode_utf16().count() + 1) * 2)
        .expect("CFB directory name length fits the 16-bit field");
    put_u16(bytes, 64, name_len);
    bytes[66] = entry_type;
    bytes[67] = 1;
    put_u32(bytes, 68, u32::MAX);
    put_u32(bytes, 72, u32::MAX);
    put_u32(bytes, 76, child);
    put_u32(bytes, 116, start_sector);
    put_u64(bytes, 120, stream_len);
}

fn minimal_cfb_with_one_64_byte_mini_stream() -> Vec<u8> {
    const SECTOR: usize = 512;
    const END_OF_CHAIN: u32 = u32::MAX - 1;
    const FAT_SECTOR: u32 = u32::MAX - 2;

    // Header + FAT + directory + one mini-stream sector + one mini-FAT sector.
    let mut file = vec![0u8; SECTOR * 5];
    file[..8].copy_from_slice(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]);
    put_u16(&mut file, 24, 0x003E);
    put_u16(&mut file, 26, 3);
    put_u16(&mut file, 28, 0xFFFE);
    put_u16(&mut file, 30, 9);
    put_u16(&mut file, 32, 6);
    put_u32(&mut file, 40, 0);
    put_u32(&mut file, 44, 1);
    put_u32(&mut file, 48, 1);
    put_u32(&mut file, 52, 0);
    put_u32(&mut file, 56, 4096);
    put_u32(&mut file, 60, 3);
    put_u32(&mut file, 64, 1);
    put_u32(&mut file, 68, END_OF_CHAIN);
    put_u32(&mut file, 72, 0);
    file[76..SECTOR].fill(0xFF);
    put_u32(&mut file, 76, 0);

    // Sector 0: FAT; sectors 1-3 each form a one-sector chain.
    let fat = &mut file[SECTOR..SECTOR * 2];
    fat.fill(0xFF);
    put_u32(fat, 0, FAT_SECTOR);
    put_u32(fat, 4, END_OF_CHAIN);
    put_u32(fat, 8, END_OF_CHAIN);
    put_u32(fat, 12, END_OF_CHAIN);

    // Sector 1: Root Entry points to a 64-byte mini stream; Workbook is
    // stored in mini-sector 0 and deliberately contains invalid BIFF bytes.
    let directory = &mut file[SECTOR * 2..SECTOR * 3];
    write_directory_entry(&mut directory[..128], "Root Entry", 5, 1, 2, 64);
    write_directory_entry(&mut directory[128..256], "Workbook", 2, u32::MAX, 0, 64);

    // Sector 2: exactly one 64-byte mini-sector, followed by padding.
    file[SECTOR * 3..SECTOR * 3 + 64].fill(0xFF);

    // Sector 3: mini-FAT with mini-sector 0 terminating normally.
    let mini_fat = &mut file[SECTOR * 4..SECTOR * 5];
    mini_fat.fill(0xFF);
    put_u32(mini_fat, 0, END_OF_CHAIN);

    file
}

#[test]
fn test_cfb_one_64_byte_mini_sector_is_read_without_panic() {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    let bytes = minimal_cfb_with_one_64_byte_mini_stream();

    // Validate the generated container independently and prove the Workbook
    // stream is exactly one 64-byte mini-sector before exercising Calamine.
    let mut compound = cfb::CompoundFile::open(Cursor::new(bytes.as_slice()))
        .expect("generated CFB mini-sector fixture must be structurally valid");
    let mut workbook_stream = Vec::new();
    compound
        .open_stream("Workbook")
        .expect("generated CFB must contain a Workbook stream")
        .read_to_end(&mut workbook_stream)
        .expect("read the generated 64-byte Workbook mini stream");
    assert_eq!(workbook_stream.len(), 64);
    assert_eq!(workbook_stream, vec![0xFF; 64]);

    let result = catch_unwind(AssertUnwindSafe(|| Xls::new(Cursor::new(bytes))));

    assert!(
        result.is_ok(),
        "a one-mini-sector CFB workbook must not panic while reading its 64-byte mini stream"
    );
    assert!(
        result.unwrap().is_err(),
        "the deliberately invalid BIFF payload should be rejected as a workbook error"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn test_cfb_cyclic_difat_is_bounded_in_memory_limited_subprocess() {
    const CHILD: &str = "EASYEXCEL_CFB_DIFAT_LIMITED_CHILD";

    if std::env::var_os(CHILD).is_some() {
        let bytes = include_bytes!("fixtures/calamine/cfb_difat_oom.xls");
        assert!(Xls::new(Cursor::new(bytes.as_slice())).is_err());
        return;
    }

    // The vulnerable 512-byte fixture attempts a multi-gigabyte allocation.
    // util-linux prlimit applies a 512 MiB RLIMIT_AS before exec'ing the child.
    let output = Command::new("prlimit")
        .args(["--as=536870912", "--"])
        .arg(std::env::current_exe().expect("test executable path"))
        .args([
            "--exact",
            "test_cfb_cyclic_difat_is_bounded_in_memory_limited_subprocess",
            "--nocapture",
        ])
        .env(CHILD, "1")
        .output()
        .expect("spawn test child through prlimit");

    assert!(
        output.status.success(),
        "memory-limited child failed: status={:?}\nstdout={}\nstderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
