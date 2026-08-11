//! 对应 Java：`com.alibaba.excel.metadata.data.HyperlinkData`.

use crate::core::coordinate_data::CoordinateData;
use std::hash::{Hash, Hasher};

include!("hyperlink_data/hyperlink_type.rs");

/// 对应 Java：com.alibaba.excel.metadata.data.HyperlinkData。 Hyperlink metadata matching Java `HyperlinkData extends CoordinateData`.
#[derive(Debug, Clone, Default)]
pub struct HyperlinkData {
    address: Option<String>,
    hyperlink_type: HyperlinkType,
    coordinates: CoordinateData,
}

// 对应 Java Lombok 默认 `callSuper = false`：坐标属于父类状态，不参与
// `HyperlinkData.equals/hashCode`，但仍完整保留给格式后端。
impl PartialEq for HyperlinkData {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address && self.hyperlink_type == other.hyperlink_type
    }
}

impl Eq for HyperlinkData {}

impl Hash for HyperlinkData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address.hash(state);
        self.hyperlink_type.hash(state);
    }
}

impl HyperlinkData {
    /// Creates an empty hyperlink. (Java default constructor)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.HyperlinkData。
    pub const fn new() -> Self {
        Self {
            address: None,
            hyperlink_type: HyperlinkType::None,
            coordinates: CoordinateData::new(),
        }
    }

    /// 对应 Java：com.alibaba.excel.metadata.data.HyperlinkData。 Sets the link target. (Java `setAddress(String)`)
    #[must_use]
    pub fn address(mut self, address: impl Into<String>) -> Self {
        self.address = Some(address.into());
        self
    }

    /// Sets the hyperlink type. (Java `setHyperlinkType(HyperlinkType)`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.HyperlinkData。
    pub const fn hyperlink_type(mut self, value: HyperlinkType) -> Self {
        self.hyperlink_type = value;
        self
    }

    /// Sets coordinates. (Java inherited `CoordinateData` fields)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.HyperlinkData。
    pub const fn coordinates(mut self, value: CoordinateData) -> Self {
        self.coordinates = value;
        self
    }

    /// 对应 Java：com.alibaba.excel.metadata.data.HyperlinkData。 Returns the address. (Java `getAddress()`)
    #[must_use]
    pub fn get_address(&self) -> Option<&str> {
        self.address.as_deref()
    }
    /// Java `setAddress` 原位 setter。
    pub fn set_address(&mut self, value: Option<String>) {
        self.address = value;
    }

    /// Returns the hyperlink type. (Java `getHyperlinkType()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.HyperlinkData。
    pub const fn get_hyperlink_type(&self) -> HyperlinkType {
        self.hyperlink_type
    }
    /// Java `setHyperlinkType` 原位 setter。
    pub const fn set_hyperlink_type(&mut self, value: HyperlinkType) {
        self.hyperlink_type = value;
    }

    /// Returns the coordinates.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.HyperlinkData。
    pub const fn get_coordinates(&self) -> CoordinateData {
        self.coordinates
    }
    /// 设置继承的坐标数据。
    pub const fn set_coordinates(&mut self, value: CoordinateData) {
        self.coordinates = value;
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn builders_and_getters_round_trip() {
        // 对应 Java：HyperlinkData 构建与 getter
        let coordinates = CoordinateData::new();
        let link = HyperlinkData::new()
            .address("https://example.com")
            .hyperlink_type(HyperlinkType::Url)
            .coordinates(coordinates);

        assert_eq!(link.get_address(), Some("https://example.com"));
        assert_eq!(link.get_hyperlink_type(), HyperlinkType::Url);
        assert_eq!(link.get_coordinates(), coordinates);
        assert_eq!(HyperlinkData::default(), HyperlinkData::new());
    }

    #[test]
    fn defaults_and_none_type() {
        // 对应 Java：默认超链接类型为 NONE
        let link = HyperlinkData::new();
        assert_eq!(link.get_address(), None);
        assert_eq!(link.get_hyperlink_type(), HyperlinkType::None);
        assert_eq!(link.hyperlink_type, HyperlinkType::None);
    }
}
