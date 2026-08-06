/// 对应 Java：无直接对应对象；Rust 架构扩展。 Java `Holder` interface contract.
///
/// The core crate already exports [`HolderEnum`] for Java `HolderEnum`. This
/// trait mirrors Java `Holder.holderType()` without colliding with that enum
/// name.
pub trait MetadataHolder {
    /// Returns the holder scope. (Java `holderType()`)
    fn holder_type(&self) -> HolderEnum;
}

