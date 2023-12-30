/// The Camera Model
///
/// This enum is so that the user can intepret fields that change order and or sign
#[derive(Debug, PartialEq, EnumString, EnumIter, Display)]
pub enum Model {
    /// Herp 5
    Hero5,
    //TODO add other common camera models
    /// Other model
    Other(String),
}
