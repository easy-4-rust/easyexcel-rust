//! Java 4.0.3 POI enum 三证据行为用例。
//!
//! 覆盖 4 个 POI enum 的 `values()` / `valueOf()` / `DEFAULT` / `ALL` / `java_name()` 行为。

use std::collections::BTreeMap;

use easyexcel::enums::poi::{
    BorderStyleEnum, FillPatternTypeEnum, HorizontalAlignmentEnum, VerticalAlignmentEnum,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PoiEnumsContract {
    #[allow(dead_code)]
    authority: String,
    enums: BTreeMap<String, EnumContract>,
}

#[derive(Debug, Deserialize)]
struct EnumContract {
    #[allow(dead_code)]
    java_class: String,
    all_java_names: Vec<String>,
    all_count: usize,
    default_java_name: String,
    #[allow(dead_code)]
    getter: String,
}

fn contract() -> PoiEnumsContract {
    serde_json::from_str(include_str!("golden/poi_enums_mapping.json"))
        .expect("Java POI enums mapping must be valid JSON")
}

/// BorderStyleEnum: values() ↔ ALL 遍历 + java_name() round-trip
#[test]
fn border_style_enum_values_and_java_name_round_trip() {
    let contract = contract();
    let enum_contract = &contract.enums["BorderStyleEnum"];

    assert_eq!(BorderStyleEnum::ALL.len(), enum_contract.all_count);

    for (i, variant) in BorderStyleEnum::ALL.iter().enumerate() {
        let java_name = variant.java_name();
        assert_eq!(
            java_name, enum_contract.all_java_names[i],
            "BorderStyleEnum::ALL[{i}] java_name mismatch"
        );

        // valueOf(String) round-trip
        let parsed: BorderStyleEnum = java_name
            .parse()
            .unwrap_or_else(|e: String| panic!("valueOf({java_name}) failed: {e}"));
        assert_eq!(
            parsed, *variant,
            "valueOf(java_name) round-trip failed for {java_name}"
        );
    }
}

/// FillPatternTypeEnum: values() ↔ ALL 遍历 + java_name() round-trip
#[test]
fn fill_pattern_type_enum_values_and_java_name_round_trip() {
    let contract = contract();
    let enum_contract = &contract.enums["FillPatternTypeEnum"];

    assert_eq!(FillPatternTypeEnum::ALL.len(), enum_contract.all_count);

    for (i, variant) in FillPatternTypeEnum::ALL.iter().enumerate() {
        let java_name = variant.java_name();
        assert_eq!(
            java_name, enum_contract.all_java_names[i],
            "FillPatternTypeEnum::ALL[{i}] java_name mismatch"
        );

        let parsed: FillPatternTypeEnum = java_name
            .parse()
            .unwrap_or_else(|e: String| panic!("valueOf({java_name}) failed: {e}"));
        assert_eq!(
            parsed, *variant,
            "valueOf(java_name) round-trip failed for {java_name}"
        );
    }
}

/// HorizontalAlignmentEnum: values() ↔ ALL 遍历 + java_name() round-trip
#[test]
fn horizontal_alignment_enum_values_and_java_name_round_trip() {
    let contract = contract();
    let enum_contract = &contract.enums["HorizontalAlignmentEnum"];

    assert_eq!(HorizontalAlignmentEnum::ALL.len(), enum_contract.all_count);

    for (i, variant) in HorizontalAlignmentEnum::ALL.iter().enumerate() {
        let java_name = variant.java_name();
        assert_eq!(
            java_name, enum_contract.all_java_names[i],
            "HorizontalAlignmentEnum::ALL[{i}] java_name mismatch"
        );

        let parsed: HorizontalAlignmentEnum = java_name
            .parse()
            .unwrap_or_else(|e: String| panic!("valueOf({java_name}) failed: {e}"));
        assert_eq!(
            parsed, *variant,
            "valueOf(java_name) round-trip failed for {java_name}"
        );
    }
}

/// VerticalAlignmentEnum: values() ↔ ALL 遍历 + java_name() round-trip
#[test]
fn vertical_alignment_enum_values_and_java_name_round_trip() {
    let contract = contract();
    let enum_contract = &contract.enums["VerticalAlignmentEnum"];

    assert_eq!(VerticalAlignmentEnum::ALL.len(), enum_contract.all_count);

    for (i, variant) in VerticalAlignmentEnum::ALL.iter().enumerate() {
        let java_name = variant.java_name();
        assert_eq!(
            java_name, enum_contract.all_java_names[i],
            "VerticalAlignmentEnum::ALL[{i}] java_name mismatch"
        );

        let parsed: VerticalAlignmentEnum = java_name
            .parse()
            .unwrap_or_else(|e: String| panic!("valueOf({java_name}) failed: {e}"));
        assert_eq!(
            parsed, *variant,
            "valueOf(java_name) round-trip failed for {java_name}"
        );
    }
}

/// 4 个 POI enum 的 DEFAULT const 均为第一个 variant，java_name() == "DEFAULT"
#[test]
fn poi_enum_default_const_matches_java_default_value() {
    let contract = contract();

    // BorderStyleEnum
    let bs_default = BorderStyleEnum::default();
    assert_eq!(bs_default, BorderStyleEnum::ALL[0]);
    assert_eq!(bs_default.java_name(), contract.enums["BorderStyleEnum"].default_java_name);
    assert!(bs_default.poi_border_style().is_none(), "DEFAULT maps to Java null");

    // FillPatternTypeEnum
    let fp_default = FillPatternTypeEnum::default();
    assert_eq!(fp_default, FillPatternTypeEnum::ALL[0]);
    assert_eq!(fp_default.java_name(), contract.enums["FillPatternTypeEnum"].default_java_name);
    assert!(fp_default.poi_fill_pattern_type().is_none(), "DEFAULT maps to Java null");

    // HorizontalAlignmentEnum
    let ha_default = HorizontalAlignmentEnum::default();
    assert_eq!(ha_default, HorizontalAlignmentEnum::ALL[0]);
    assert_eq!(ha_default.java_name(), contract.enums["HorizontalAlignmentEnum"].default_java_name);
    assert!(ha_default.poi_horizontal_alignment().is_none(), "DEFAULT maps to Java null");

    // VerticalAlignmentEnum
    let va_default = VerticalAlignmentEnum::default();
    assert_eq!(va_default, VerticalAlignmentEnum::ALL[0]);
    assert_eq!(va_default.java_name(), contract.enums["VerticalAlignmentEnum"].default_java_name);
    assert!(va_default.poi_vertical_alignment_enum().is_none(), "DEFAULT maps to Java null");
}
