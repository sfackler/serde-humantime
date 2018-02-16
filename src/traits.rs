use std::fmt;
use serde::ser::{Serializer, Serialize};
use serde::de::{Deserializer, Error as DeError, Visitor};
use std::marker::PhantomData;


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

impl<T: HumanTime> HumanTime for Option<T> {}

impl<T: Sealed> Sealed for Option<T> {
    fn encode<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        struct Data<'a, V: 'a>(&'a V) where V: Sealed;

        impl<'a, V: Sealed + 'a> Serialize for Data<'a, V> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer
            {
                self.0.encode(serializer)
            }
        }

        match *self {
            Some(ref value) => serializer.serialize_some(&Data(value)),
            None => serializer.serialize_none(),
        }
    }
    fn decode<'de, D>(deserializer: D) -> Result<Self, D::Error>
        where Self: Sized,
              D: Deserializer<'de>,
    {
        struct OptionVisitor<T> {
            marker: PhantomData<T>,
        }

        impl<'de, T: Sealed> Visitor<'de> for OptionVisitor<T> {
            type Value = Option<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result
            {
                formatter.write_str("option")
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Option<T>, E>
                where E: DeError,
            {
                Ok(None)
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Option<T>, E>
                where E: DeError,
            {
                Ok(None)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D)
                -> Result<Option<T>, D::Error>
                where D: Deserializer<'de>,
            {
                T::decode(deserializer).map(Some)
            }
        }

        deserializer.deserialize_option(OptionVisitor { marker: PhantomData })
    }
}
