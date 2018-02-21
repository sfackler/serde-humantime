//! A crate providing Serde deserializers for `Duration`s via the `humantime`
//! crate.
//!
//! # Examples
//!
//! You can use the `deserialize` function with the `with` or `deserialize_with`
//! annotations:
//!
//! ```
//! extern crate serde_humantime;
//! extern crate serde;
//! #[macro_use]
//! extern crate serde_derive;
//!
//! use std::time::Duration;
//!
//! #[derive(Deserialize)]
//! struct Foo {
//!     #[serde(with = "serde_humantime")]
//!     timeout: Duration,
//! }
//!
//! # fn main() {}
//! ```
//!
//! Or use the `De` wrapper type:
//!
//! ```
//! extern crate serde_humantime;
//! extern crate serde;
//! #[macro_use]
//! extern crate serde_derive;
//!
//! use serde_humantime::De;
//! use std::time::Duration;
//!
//! #[derive(Deserialize)]
//! struct Foo {
//!     timeout: De<Option<Duration>>,
//! }
//!
//! # fn main() {}
//! ```
#![warn(missing_docs)]
#![doc(html_root_url="https://docs.rs/serde-humantime/0.1.1")]

extern crate humantime;
extern crate serde;

#[cfg(test)]
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
extern crate serde_json;

use serde::de::{Deserialize, Deserializer, Visitor, Error, Unexpected};
use serde::ser::{Serializer, Serialize};
use std::fmt;
use std::time::{Duration, SystemTime};

mod traits;

pub use traits::HumanTime;

/// A wrapper type which implements `Deserialize` for types involving
/// `Duration`.
///
/// It can only be constructed through its `Deserialize` implementations.
pub struct De<T>(T);

impl<T> De<T> {
    /// Consumes the `De`, returning the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<'de> Deserialize<'de> for De<Duration> {
    fn deserialize<D>(d: D) -> Result<De<Duration>, D::Error>
        where D: Deserializer<'de>
    {
        deserialize::<Duration, _>(d).map(De)
    }
}

impl<'de> Deserialize<'de> for De<Option<Duration>> {
    fn deserialize<D>(d: D) -> Result<De<Option<Duration>>, D::Error>
        where D: Deserializer<'de>
    {
        match Option::<De<Duration>>::deserialize(d)? {
            Some(De(dur)) => Ok(De(Some(dur))),
            None => Ok(De(None)),
        }
    }
}

/// Deserializes a `Duration` or `SystemTime` via the humantime crate.
///
/// This function can be used with `serde_derive`'s `with` and
/// `deserialize_with` annotations.
pub fn deserialize<'de, T, D>(d: D) -> Result<T, D::Error>
    where D: Deserializer<'de>,
          T: HumanTime,
{
    T::decode(d)
}

/// Deserializes a `Duration` or `SystemTime` via the humantime crate.
///
/// This function can be used with `serde_derive`'s `with` and
/// `serialize_with` annotations.
pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where T: HumanTime,
          S: Serializer
{
    traits::Sealed::encode(value, serializer)
}

impl traits::Sealed for Duration {

    fn encode<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        humantime::format_duration(*self).to_string().serialize(serializer)
    }

    fn decode<'de, D>(deserializer: D) -> Result<Self, D::Error>
        where Self: Sized,
              D: Deserializer<'de>
    {
        struct V;

        impl<'de2> Visitor<'de2> for V {
            type Value = Duration;

            fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt.write_str("a duration")
            }

            fn visit_str<E>(self, v: &str) -> Result<Duration, E>
                where E: Error
            {
                humantime::parse_duration(v)
                    .map_err(|_| E::invalid_value(Unexpected::Str(v), &self))
            }
        }

        deserializer.deserialize_str(V)
    }
}

impl traits::Sealed for SystemTime {

    fn encode<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        humantime::format_rfc3339(*self).to_string().serialize(serializer)
    }

    fn decode<'de, D>(deserializer: D) -> Result<Self, D::Error>
        where Self: Sized,
              D: Deserializer<'de>
    {
        struct V;

        impl<'de2> Visitor<'de2> for V {
            type Value = SystemTime;

            fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt.write_str("a timestamp")
            }

            fn visit_str<E>(self, v: &str) -> Result<SystemTime, E>
                where E: Error
            {
                humantime::parse_rfc3339_weak(v)
                    .map_err(|_| E::invalid_value(Unexpected::Str(v), &self))
            }
        }

        deserializer.deserialize_str(V)
    }
}

impl HumanTime for Duration {}
impl HumanTime for SystemTime {}

#[cfg(test)]
mod test {
    use std::time::{SystemTime, UNIX_EPOCH};
    use super::*;

    #[test]
    fn with() {
        #[derive(Serialize, Deserialize)]
        struct Foo {
            #[serde(with = "super")]
            time: Duration,
        }

        let json = r#"{"time": "15 seconds"}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time, Duration::from_secs(15));
        let reverse = serde_json::to_string(&foo).unwrap();
        assert_eq!(reverse, r#"{"time":"15s"}"#);
    }

    #[test]
    fn timestamp() {
        #[derive(Serialize, Deserialize)]
        struct Foo {
            #[serde(with = "super")]
            time: SystemTime,
        }

        let json = r#"{"time": "2013-01-01T15:44:00Z"}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time, UNIX_EPOCH + Duration::new(1357055040, 0));
        let reverse = serde_json::to_string(&foo).unwrap();
        assert_eq!(reverse, r#"{"time":"2013-01-01T15:44:00Z"}"#);
    }

    #[test]
    fn de_option() {
        #[derive(Deserialize)]
        struct Foo {
            time: De<Option<Duration>>,
        }

        let json = r#"{"time": "15 seconds"}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time.into_inner(), Some(Duration::from_secs(15)));

        let json = r#"{"time": null}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time.into_inner(), None);

        let json = r#"{}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time.into_inner(), None);
    }

    #[test]
    fn unwrapped_option() {
        #[derive(Deserialize)]
        struct Foo {
            #[serde(with = "super", default)]
            time: Option<Duration>,
        }

        let json = r#"{"time": "15 seconds"}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time, Some(Duration::from_secs(15)));

        let json = r#"{"time": null}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time, None);

        let json = r#"{}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time, None);
    }
}
