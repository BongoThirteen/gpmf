/// Entry
pub enum Entry {
    /// Simple Entry
    Simgle(KeyValue),
    /// Sequence
    Seq(Vec<KeyValue>),
}
