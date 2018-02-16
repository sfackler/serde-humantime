use serde::ser::{Serializer};
use serde::de::{Deserializer};


pub trait Sealed {
    fn encode<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer;
    fn decode<'de, D>(deserializer: D) -> Result<Self, D::Error>
        where Self: Sized,
              D: Deserializer<'de>;
}

/// A value serializeable using humantime crate
///
/// This trait is currently sealed. This might change in future.
pub trait HumanTime: Sealed {
}
