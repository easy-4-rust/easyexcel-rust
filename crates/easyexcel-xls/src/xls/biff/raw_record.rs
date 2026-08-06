/// 对应 Java：无直接对应对象；Rust 架构扩展。 A single parsed BIFF record (CONTINUE records already merged in by
/// [`Records`] for records that carry overflow data).
pub struct RawRecord {
    pub typ: u16,
    pub data: Vec<u8>,
    /// For SST specifically we need to know where each CONTINUE boundary fell
    /// in the merged byte stream, because the grbit byte restarts there. This
    /// holds the byte offset (within `data`) at which each continuation block
    /// began. Empty for records with no continuation.
    pub continue_breaks: Vec<usize>,
}

