//! 对应 Java：`com.alibaba.excel.metadata.data.ClientAnchorData`.

use crate::core::anchor_type::AnchorType;
use crate::core::coordinate_data::CoordinateData;
use std::hash::{Hash, Hasher};

/// 对应 Java：com.alibaba.excel.metadata.data.ClientAnchorData。 Client-anchor margins and movement behavior.
///
/// Java `ClientAnchorData extends CoordinateData`; Rust uses composition
/// because the inner type is `Copy`/`Default` and we avoid the inheritance
/// bookkeeping penalty. The four pixel margin fields match Java exactly.
#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct ClientAnchorData {
    coordinates: CoordinateData,
    top: Option<u32>,
    right: Option<u32>,
    bottom: Option<u32>,
    left: Option<u32>,
    anchor_type: Option<AnchorType>,
}

// Java 的 Lombok `@EqualsAndHashCode` 默认 `callSuper = false`，因此继承自
// `CoordinateData` 的坐标不参与相等性和哈希；组合映射也必须保持这一点。
impl PartialEq for ClientAnchorData {
    fn eq(&self, other: &Self) -> bool {
        self.top == other.top
            && self.right == other.right
            && self.bottom == other.bottom
            && self.left == other.left
            && self.anchor_type == other.anchor_type
    }
}

impl Eq for ClientAnchorData {}

impl Hash for ClientAnchorData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.top.hash(state);
        self.right.hash(state);
        self.bottom.hash(state);
        self.left.hash(state);
        self.anchor_type.hash(state);
    }
}

impl ClientAnchorData {
    /// Creates a default anchor for the decorated cell. (Java default constructor)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.ClientAnchorData。
    pub const fn new() -> Self {
        Self {
            coordinates: CoordinateData::new(),
            top: None,
            right: None,
            bottom: None,
            left: None,
            anchor_type: None,
        }
    }

    /// Sets its absolute and relative cell coordinates.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.ClientAnchorData。
    pub const fn coordinates(mut self, value: CoordinateData) -> Self {
        self.coordinates = value;
        self
    }

    /// Sets the top margin in pixels.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.ClientAnchorData。
    pub const fn top(mut self, value: u32) -> Self {
        self.top = Some(value);
        self
    }

    /// Sets the right margin in pixels.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.ClientAnchorData。
    pub const fn right(mut self, value: u32) -> Self {
        self.right = Some(value);
        self
    }

    /// Sets the bottom margin in pixels.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.ClientAnchorData。
    pub const fn bottom(mut self, value: u32) -> Self {
        self.bottom = Some(value);
        self
    }

    /// Sets the left margin in pixels.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.ClientAnchorData。
    pub const fn left(mut self, value: u32) -> Self {
        self.left = Some(value);
        self
    }

    /// Sets the object movement and resize behavior.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.ClientAnchorData。
    pub const fn anchor_type(mut self, value: AnchorType) -> Self {
        self.anchor_type = Some(value);
        self
    }

    /// Returns the coordinates. (Java `getCoordinates()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.ClientAnchorData。
    pub const fn get_coordinates(self) -> CoordinateData {
        self.coordinates
    }

    /// Returns the top margin in pixels. (Java `getTop()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.ClientAnchorData。
    pub const fn get_top(self) -> Option<u32> {
        self.top
    }

    /// Returns the right margin in pixels. (Java `getRight()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.ClientAnchorData。
    pub const fn get_right(self) -> Option<u32> {
        self.right
    }

    /// Returns the bottom margin in pixels. (Java `getBottom()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.ClientAnchorData。
    pub const fn get_bottom(self) -> Option<u32> {
        self.bottom
    }

    /// Returns the left margin in pixels. (Java `getLeft()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.ClientAnchorData。
    pub const fn get_left(self) -> Option<u32> {
        self.left
    }

    /// Returns the movement and resize behavior. (Java `getAnchorType()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.ClientAnchorData。
    pub const fn get_anchor_type(self) -> Option<AnchorType> {
        self.anchor_type
    }

    /// Java `setTop`。
    pub const fn set_top(&mut self, value: Option<u32>) { self.top = value; }
    /// Java `setRight`。
    pub const fn set_right(&mut self, value: Option<u32>) { self.right = value; }
    /// Java `setBottom`。
    pub const fn set_bottom(&mut self, value: Option<u32>) { self.bottom = value; }
    /// Java `setLeft`。
    pub const fn set_left(&mut self, value: Option<u32>) { self.left = value; }
    /// Java `setAnchorType`。
    pub const fn set_anchor_type(&mut self, value: Option<AnchorType>) { self.anchor_type = value; }
    /// 替换继承坐标。
    pub const fn set_coordinates(&mut self, value: CoordinateData) { self.coordinates = value; }
}
