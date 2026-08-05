//! OOXML XLSX 工作簿读取、写入、流式读取和 `RoundTrip` 支持。

pub mod xlsx;

pub use xlsx::{
    OoxmlPackage, OoxmlZipEntry, ReadWriteSeek, encrypt_package_to, looks_like_zip, read,
    read_path, read_path_with_password, read_with_password, stream, write, write_path,
};
