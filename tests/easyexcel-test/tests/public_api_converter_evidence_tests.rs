//! Java 4.0.3 converter family 的公共 API 编译与共享行为证据。
//!
//! 本文件按具体 converter 逐类型实例化，但复用同一 trait/registry 契约，避免
//! 为 javac bridge、Class<?> 和旧三参数重载复制 46 套兼容实现。

use std::any::TypeId;
use std::io::Cursor;
use std::path::PathBuf;

use bigdecimal::BigDecimal;
use chrono::{NaiveDate, NaiveDateTime};
use easyexcel::converters::Converter;
use easyexcel::{CellDataType, ImageInputStream, JavaDate};
use num_bigint::BigInt;
use serde::Deserialize;
use url::Url;

#[derive(Debug, Deserialize)]
struct JavaConverterContract {
    authority: String,
    registered_converter_count: usize,
    converters: Vec<JavaConverterEntry>,
    auto_converter: JavaAutoConverterEntry,
}

#[derive(Debug, Deserialize)]
struct JavaConverterEntry {
    class: String,
    public_no_arg_constructor: bool,
    java_type_key: String,
    excel_type_key: String,
    declared_public_methods: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct JavaAutoConverterEntry {
    class: String,
    public_no_arg_constructor: bool,
    java_type_key: String,
    excel_type_key: String,
}

fn java_contract() -> JavaConverterContract {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden/converter_api.contract.json");
    let json = std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "Java converter contract is missing or unreadable at {}: {error}; run scripts/export-java-golden.sh",
            path.display()
        )
    });
    serde_json::from_str(&json).expect("Java converter contract must be valid JSON")
}

const JAVA_CONVERTERS: &[(&str, &str, Option<&str>)] = &[
    ("com.alibaba.excel.converters.bigdecimal.BigDecimalBooleanConverter", "java.math.BigDecimal", Some("BOOLEAN")),
    ("com.alibaba.excel.converters.bigdecimal.BigDecimalNumberConverter", "java.math.BigDecimal", Some("NUMBER")),
    ("com.alibaba.excel.converters.bigdecimal.BigDecimalStringConverter", "java.math.BigDecimal", Some("STRING")),
    ("com.alibaba.excel.converters.biginteger.BigIntegerBooleanConverter", "java.math.BigInteger", Some("BOOLEAN")),
    ("com.alibaba.excel.converters.biginteger.BigIntegerNumberConverter", "java.math.BigInteger", Some("NUMBER")),
    ("com.alibaba.excel.converters.biginteger.BigIntegerStringConverter", "java.math.BigInteger", Some("STRING")),
    ("com.alibaba.excel.converters.booleanconverter.BooleanBooleanConverter", "java.lang.Boolean", Some("BOOLEAN")),
    ("com.alibaba.excel.converters.booleanconverter.BooleanNumberConverter", "java.lang.Boolean", Some("NUMBER")),
    ("com.alibaba.excel.converters.booleanconverter.BooleanStringConverter", "java.lang.Boolean", Some("STRING")),
    ("com.alibaba.excel.converters.bytearray.ByteArrayImageConverter", "[B", None),
    ("com.alibaba.excel.converters.bytearray.BoxingByteArrayImageConverter", "[Ljava.lang.Byte;", None),
    ("com.alibaba.excel.converters.byteconverter.ByteBooleanConverter", "java.lang.Byte", Some("BOOLEAN")),
    ("com.alibaba.excel.converters.byteconverter.ByteNumberConverter", "java.lang.Byte", Some("NUMBER")),
    ("com.alibaba.excel.converters.byteconverter.ByteStringConverter", "java.lang.Byte", Some("STRING")),
    ("com.alibaba.excel.converters.date.DateDateConverter", "java.util.Date", None),
    ("com.alibaba.excel.converters.date.DateNumberConverter", "java.util.Date", Some("NUMBER")),
    ("com.alibaba.excel.converters.date.DateStringConverter", "java.util.Date", Some("STRING")),
    ("com.alibaba.excel.converters.doubleconverter.DoubleBooleanConverter", "java.lang.Double", Some("BOOLEAN")),
    ("com.alibaba.excel.converters.doubleconverter.DoubleNumberConverter", "java.lang.Double", Some("NUMBER")),
    ("com.alibaba.excel.converters.doubleconverter.DoubleStringConverter", "java.lang.Double", Some("STRING")),
    ("com.alibaba.excel.converters.file.FileImageConverter", "java.io.File", None),
    ("com.alibaba.excel.converters.floatconverter.FloatBooleanConverter", "java.lang.Float", Some("BOOLEAN")),
    ("com.alibaba.excel.converters.floatconverter.FloatNumberConverter", "java.lang.Float", Some("NUMBER")),
    ("com.alibaba.excel.converters.floatconverter.FloatStringConverter", "java.lang.Float", Some("STRING")),
    ("com.alibaba.excel.converters.inputstream.InputStreamImageConverter", "java.io.InputStream", None),
    ("com.alibaba.excel.converters.integer.IntegerBooleanConverter", "java.lang.Integer", Some("BOOLEAN")),
    ("com.alibaba.excel.converters.integer.IntegerNumberConverter", "java.lang.Integer", Some("NUMBER")),
    ("com.alibaba.excel.converters.integer.IntegerStringConverter", "java.lang.Integer", Some("STRING")),
    ("com.alibaba.excel.converters.localdate.LocalDateDateConverter", "java.time.LocalDate", None),
    ("com.alibaba.excel.converters.localdate.LocalDateNumberConverter", "java.time.LocalDate", Some("NUMBER")),
    ("com.alibaba.excel.converters.localdate.LocalDateStringConverter", "java.time.LocalDate", Some("STRING")),
    ("com.alibaba.excel.converters.localdatetime.LocalDateTimeDateConverter", "java.time.LocalDateTime", None),
    ("com.alibaba.excel.converters.localdatetime.LocalDateTimeNumberConverter", "java.time.LocalDateTime", Some("NUMBER")),
    ("com.alibaba.excel.converters.localdatetime.LocalDateTimeStringConverter", "java.time.LocalDateTime", Some("STRING")),
    ("com.alibaba.excel.converters.longconverter.LongBooleanConverter", "java.lang.Long", Some("BOOLEAN")),
    ("com.alibaba.excel.converters.longconverter.LongNumberConverter", "java.lang.Long", Some("NUMBER")),
    ("com.alibaba.excel.converters.longconverter.LongStringConverter", "java.lang.Long", Some("STRING")),
    ("com.alibaba.excel.converters.shortconverter.ShortBooleanConverter", "java.lang.Short", Some("BOOLEAN")),
    ("com.alibaba.excel.converters.shortconverter.ShortNumberConverter", "java.lang.Short", Some("NUMBER")),
    ("com.alibaba.excel.converters.shortconverter.ShortStringConverter", "java.lang.Short", Some("STRING")),
    ("com.alibaba.excel.converters.string.StringBooleanConverter", "java.lang.String", Some("BOOLEAN")),
    ("com.alibaba.excel.converters.string.StringErrorConverter", "java.lang.String", Some("ERROR")),
    ("com.alibaba.excel.converters.string.StringImageConverter", "java.lang.String", None),
    ("com.alibaba.excel.converters.string.StringNumberConverter", "java.lang.String", Some("NUMBER")),
    ("com.alibaba.excel.converters.string.StringStringConverter", "java.lang.String", Some("STRING")),
    ("com.alibaba.excel.converters.url.UrlImageConverter", "java.net.URL", None),
];

fn assert_java_type_key<T, C>()
where
    T: 'static,
    C: Converter<T> + Default,
{
    let converter = C::default();
    assert_eq!(converter.support_java_type_key(), TypeId::of::<T>());
}

fn assert_excel_type_key<T, C>(expected: CellDataType)
where
    C: Converter<T> + Default,
{
    let converter = C::default();
    assert_eq!(converter.support_excel_type_key(), expected);
}

macro_rules! assert_java_type_keys {
    ($(($target:ty, $converter:path)),+ $(,)?) => {
        $(assert_java_type_key::<$target, $converter>();)+
    };
}

macro_rules! assert_excel_type_keys {
    ($(($target:ty, $converter:path, $cell_type:expr)),+ $(,)?) => {
        $(assert_excel_type_key::<$target, $converter>($cell_type);)+
    };
}

#[test]
fn every_java_builtin_converter_has_a_default_rust_type_carrier() {
    let _: easyexcel::converters::AutoConverter = Default::default();
    assert_java_type_keys!(
        (BigDecimal, easyexcel::converters::bigdecimal::BigDecimalBooleanConverter),
        (BigDecimal, easyexcel::converters::bigdecimal::BigDecimalNumberConverter),
        (BigDecimal, easyexcel::converters::bigdecimal::BigDecimalStringConverter),
        (BigInt, easyexcel::converters::biginteger::BigIntegerBooleanConverter),
        (BigInt, easyexcel::converters::biginteger::BigIntegerNumberConverter),
        (BigInt, easyexcel::converters::biginteger::BigIntegerStringConverter),
        (bool, easyexcel::converters::booleanconverter::BooleanBooleanConverter),
        (bool, easyexcel::converters::booleanconverter::BooleanNumberConverter),
        (bool, easyexcel::converters::booleanconverter::BooleanStringConverter),
        (Vec<u8>, easyexcel::converters::bytearray::ByteArrayImageConverter),
        (Box<[u8]>, easyexcel::converters::bytearray::BoxingByteArrayImageConverter),
        (i8, easyexcel::converters::byteconverter::ByteBooleanConverter),
        (i8, easyexcel::converters::byteconverter::ByteNumberConverter),
        (i8, easyexcel::converters::byteconverter::ByteStringConverter),
        (JavaDate, easyexcel::converters::date::DateDateConverter),
        (JavaDate, easyexcel::converters::date::DateNumberConverter),
        (JavaDate, easyexcel::converters::date::DateStringConverter),
        (f64, easyexcel::converters::doubleconverter::DoubleBooleanConverter),
        (f64, easyexcel::converters::doubleconverter::DoubleNumberConverter),
        (f64, easyexcel::converters::doubleconverter::DoubleStringConverter),
        (PathBuf, easyexcel::converters::file::FileImageConverter),
        (f32, easyexcel::converters::floatconverter::FloatBooleanConverter),
        (f32, easyexcel::converters::floatconverter::FloatNumberConverter),
        (f32, easyexcel::converters::floatconverter::FloatStringConverter),
        (
            ImageInputStream<Cursor<Vec<u8>>>,
            easyexcel::converters::inputstream::InputStreamImageConverter
        ),
        (i32, easyexcel::converters::integer::IntegerBooleanConverter),
        (i32, easyexcel::converters::integer::IntegerNumberConverter),
        (i32, easyexcel::converters::integer::IntegerStringConverter),
        (NaiveDate, easyexcel::converters::localdate::LocalDateDateConverter),
        (NaiveDate, easyexcel::converters::localdate::LocalDateNumberConverter),
        (NaiveDate, easyexcel::converters::localdate::LocalDateStringConverter),
        (
            NaiveDateTime,
            easyexcel::converters::localdatetime::LocalDateTimeDateConverter
        ),
        (
            NaiveDateTime,
            easyexcel::converters::localdatetime::LocalDateTimeNumberConverter
        ),
        (
            NaiveDateTime,
            easyexcel::converters::localdatetime::LocalDateTimeStringConverter
        ),
        (i64, easyexcel::converters::longconverter::LongBooleanConverter),
        (i64, easyexcel::converters::longconverter::LongNumberConverter),
        (i64, easyexcel::converters::longconverter::LongStringConverter),
        (i16, easyexcel::converters::shortconverter::ShortBooleanConverter),
        (i16, easyexcel::converters::shortconverter::ShortNumberConverter),
        (i16, easyexcel::converters::shortconverter::ShortStringConverter),
        (String, easyexcel::converters::string::StringBooleanConverter),
        (String, easyexcel::converters::string::StringErrorConverter),
        (String, easyexcel::converters::string::StringImageConverter),
        (String, easyexcel::converters::string::StringNumberConverter),
        (String, easyexcel::converters::string::StringStringConverter),
        (Url, easyexcel::converters::url::UrlImageConverter),
    );
}

#[test]
fn every_java_read_converter_exposes_its_declared_excel_cell_key() {
    assert_excel_type_keys!(
        (BigDecimal, easyexcel::converters::bigdecimal::BigDecimalBooleanConverter, CellDataType::Boolean),
        (BigDecimal, easyexcel::converters::bigdecimal::BigDecimalNumberConverter, CellDataType::Number),
        (BigDecimal, easyexcel::converters::bigdecimal::BigDecimalStringConverter, CellDataType::String),
        (BigInt, easyexcel::converters::biginteger::BigIntegerBooleanConverter, CellDataType::Boolean),
        (BigInt, easyexcel::converters::biginteger::BigIntegerNumberConverter, CellDataType::Number),
        (BigInt, easyexcel::converters::biginteger::BigIntegerStringConverter, CellDataType::String),
        (bool, easyexcel::converters::booleanconverter::BooleanBooleanConverter, CellDataType::Boolean),
        (bool, easyexcel::converters::booleanconverter::BooleanNumberConverter, CellDataType::Number),
        (bool, easyexcel::converters::booleanconverter::BooleanStringConverter, CellDataType::String),
        (i8, easyexcel::converters::byteconverter::ByteBooleanConverter, CellDataType::Boolean),
        (i8, easyexcel::converters::byteconverter::ByteNumberConverter, CellDataType::Number),
        (i8, easyexcel::converters::byteconverter::ByteStringConverter, CellDataType::String),
        (JavaDate, easyexcel::converters::date::DateNumberConverter, CellDataType::Number),
        (JavaDate, easyexcel::converters::date::DateStringConverter, CellDataType::String),
        (f64, easyexcel::converters::doubleconverter::DoubleBooleanConverter, CellDataType::Boolean),
        (f64, easyexcel::converters::doubleconverter::DoubleNumberConverter, CellDataType::Number),
        (f64, easyexcel::converters::doubleconverter::DoubleStringConverter, CellDataType::String),
        (f32, easyexcel::converters::floatconverter::FloatBooleanConverter, CellDataType::Boolean),
        (f32, easyexcel::converters::floatconverter::FloatNumberConverter, CellDataType::Number),
        (f32, easyexcel::converters::floatconverter::FloatStringConverter, CellDataType::String),
        (i32, easyexcel::converters::integer::IntegerBooleanConverter, CellDataType::Boolean),
        (i32, easyexcel::converters::integer::IntegerNumberConverter, CellDataType::Number),
        (i32, easyexcel::converters::integer::IntegerStringConverter, CellDataType::String),
        (NaiveDate, easyexcel::converters::localdate::LocalDateNumberConverter, CellDataType::Number),
        (NaiveDate, easyexcel::converters::localdate::LocalDateStringConverter, CellDataType::String),
        (NaiveDateTime, easyexcel::converters::localdatetime::LocalDateTimeNumberConverter, CellDataType::Number),
        (NaiveDateTime, easyexcel::converters::localdatetime::LocalDateTimeStringConverter, CellDataType::String),
        (i64, easyexcel::converters::longconverter::LongBooleanConverter, CellDataType::Boolean),
        (i64, easyexcel::converters::longconverter::LongNumberConverter, CellDataType::Number),
        (i64, easyexcel::converters::longconverter::LongStringConverter, CellDataType::String),
        (i16, easyexcel::converters::shortconverter::ShortBooleanConverter, CellDataType::Boolean),
        (i16, easyexcel::converters::shortconverter::ShortNumberConverter, CellDataType::Number),
        (i16, easyexcel::converters::shortconverter::ShortStringConverter, CellDataType::String),
        (String, easyexcel::converters::string::StringBooleanConverter, CellDataType::Boolean),
        (String, easyexcel::converters::string::StringErrorConverter, CellDataType::Error),
        (String, easyexcel::converters::string::StringNumberConverter, CellDataType::Number),
        (String, easyexcel::converters::string::StringStringConverter, CellDataType::String),
    );
}

#[test]
fn java_runtime_registry_contract_matches_the_46_rust_type_carriers() {
    let contract = java_contract();
    assert_eq!(contract.authority, "com.alibaba:easyexcel:4.0.3");
    assert_eq!(JAVA_CONVERTERS.len(), 46);
    assert_eq!(contract.registered_converter_count, JAVA_CONVERTERS.len());
    assert_eq!(contract.converters.len(), JAVA_CONVERTERS.len());

    let mut convert_to_java_bridge_methods = 0_usize;
    for &(class, java_type_key, excel_type_key) in JAVA_CONVERTERS {
        let matching = contract
            .converters
            .iter()
            .filter(|entry| entry.class == class)
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), 1, "Java registry entry is not unique: {class}");
        let entry = matching[0];
        assert!(
            entry.public_no_arg_constructor,
            "Java converter lost its public no-arg constructor: {class}"
        );
        assert_eq!(
            entry.java_type_key.as_str(),
            java_type_key,
            "Java type key: {class}"
        );
        assert_eq!(
            entry.excel_type_key.as_str(),
            excel_type_key.unwrap_or("<unsupported>"),
            "Java Excel type key: {class}"
        );
        assert!(
            entry
                .declared_public_methods
                .iter()
                .any(|method| method.starts_with("supportJavaTypeKey(")),
            "Java converter must declare supportJavaTypeKey: {class}"
        );
        let read_methods = entry
            .declared_public_methods
            .iter()
            .filter(|method| method.starts_with("convertToJavaData("))
            .count();
        if excel_type_key.is_some() {
            // javac emits the typed method plus its erased Object-return bridge.
            assert_eq!(read_methods, 2, "Java read bridge methods: {class}");
        } else {
            assert_eq!(read_methods, 0, "write-only Java converter: {class}");
        }
        convert_to_java_bridge_methods += read_methods;
    }
    assert_eq!(convert_to_java_bridge_methods, 74);

    assert_eq!(
        contract.auto_converter.class,
        "com.alibaba.excel.converters.AutoConverter"
    );
    assert!(contract.auto_converter.public_no_arg_constructor);
    assert_eq!(
        contract.auto_converter.java_type_key,
        "java.lang.UnsupportedOperationException"
    );
    assert_eq!(
        contract.auto_converter.excel_type_key,
        "java.lang.UnsupportedOperationException"
    );
}
