//! 对应 Java：`com.alibaba.excel.metadata.data.HyperlinkData`.

use crate::coordinate_data::CoordinateData;

/// Hyperlink type matching Java `HyperlinkData.HyperlinkType`.
///
/// Values mirror Apache POI `HyperlinkType` as used by EasyExcel 4.0.3.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum HyperlinkType {
    /// Not a hyperlink. (Java `NONE`)
    #[default]
    None,
    /// Link to an existing file or web page. (Java `URL`)
    Url,
    /// Link to a place in this document. (Java `DOCUMENT`)
    Document,
    /// Link to an e-mail address. (Java `EMAIL`)
    Email,
    /// Link to a file. (Java `FILE`)
    File,
}

/// Hyperlink metadata matching Java `HyperlinkData extends CoordinateData`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HyperlinkData {
    address: Option<String>,
    hyperlink_type: HyperlinkType,
    coordinates: CoordinateData,
}

impl HyperlinkData {
    /// Creates an empty hyperlink. (Java default constructor)
    #[must_use]
    pub const fn new() -> Self {
        Self {
            address: None,
            hyperlink_type: HyperlinkType::None,
            coordinates: CoordinateData::new(),
        }
    }

    /// Sets the link target. (Java `setAddress(String)`)
    #[must_use]
    pub fn address(mut self, address: impl Into<String>) -> Self {
        self.address = Some(address.into());
        self
    }

    /// Sets the hyperlink type. (Java `setHyperlinkType(HyperlinkType)`)
    #[must_use]
    pub const fn hyperlink_type(mut self, value: HyperlinkType) -> Self {
        self.hyperlink_type = value;
        self
    }

    /// Sets coordinates. (Java inherited `CoordinateData` fields)
    #[must_use]
    pub const fn coordinates(mut self, value: CoordinateData) -> Self {
        self.coordinates = value;
        self
    }

    /// Returns the address. (Java `getAddress()`)
    #[must_use]
    pub fn get_address(&self) -> Option<&str> {
        self.address.as_deref()
    }

    /// Returns the hyperlink type. (Java `getHyperlinkType()`)
    #[must_use]
    pub const fn get_hyperlink_type(&self) -> HyperlinkType {
        self.hyperlink_type
    }

    /// Returns the coordinates.
    #[must_use]
    pub const fn get_coordinates(&self) -> CoordinateData {
        self.coordinates
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
