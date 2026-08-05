//! OOXML XLSX 基础工作簿引擎门面。

pub use easyexcel_xlsx::{
    OoxmlPackage, OoxmlZipEntry, ReadWriteSeek, encrypt_package_to, looks_like_zip, read,
    read_path, read_path_with_password, read_with_password, stream, write, write_path,
};

pub use easyexcel_xlsx::xlsx::package;
