//! A crate providing Serde (de)serializers for `Duration` and `SystemTime`
//! via the `humantime` crate.
//!
//! # Examples
//!
//! You can use the `deserialize` and `serialize` functions with the
//! `with` or `serialize_with`/`deserialize_with` annotations:
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
//! Or use the `Serde` wrapper type:
//!
//! ```
//! extern crate serde_humantime;
//! extern crate serde;
//! #[macro_use]
//! extern crate serde_derive;
//!
//! use serde_humantime::Serde;
//! use std::time::SystemTime;
//!
//! #[derive(Deserialize)]
//! struct Foo {
//!     timeout: Vec<Serde<SystemTime>>,
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

mod wrapper;

pub use wrapper::Serde;

use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use std::time::Duration;

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
        Serde::deserialize(d).map(Serde::into_inner).map(De)
    }
}

impl<'de> Deserialize<'de> for De<Option<Duration>> {
    fn deserialize<D>(d: D) -> Result<De<Option<Duration>>, D::Error>
        where D: Deserializer<'de>
    {
        Serde::deserialize(d).map(Serde::into_inner).map(De)
    }
}

/// Deserializes a `Duration` or `SystemTime` via the humantime crate.
///
/// This function can be used with `serde_derive`'s `with` and
/// `deserialize_with` annotations.
pub fn deserialize<'a, T, D>(d: D) -> Result<T, D::Error>
    where Serde<T>: Deserialize<'a>,
          D: Deserializer<'a>,
{
    Serde::deserialize(d).map(Serde::into_inner)
}

/// Serializes a `Duration` or `SystemTime` via the humantime crate.
///
/// This function can be used with `serde_derive`'s `with` and
/// `serialize_with` annotations.
pub fn serialize<T, S>(d: &T, s: S) -> Result<S::Ok, S::Error>
    where for<'a> Serde<&'a T>: Serialize,
          S: Serializer,
{
    Serde::from(d).serialize(s)
}

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
    fn with_option() {
        #[derive(Serialize, Deserialize)]
        struct Foo {
            #[serde(with = "super", default)]
            time: Option<Duration>,
        }

        let json = r#"{"time": "15 seconds"}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time, Some(Duration::from_secs(15)));
        let reverse = serde_json::to_string(&foo).unwrap();
        assert_eq!(reverse, r#"{"time":"15s"}"#);

        let json = r#"{"time": null}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time, None);
        let reverse = serde_json::to_string(&foo).unwrap();
        assert_eq!(reverse, r#"{"time":null}"#);

        let json = r#"{}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time, None);
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
    fn time() {
        #[derive(Serialize, Deserialize)]
        struct Foo {
            #[serde(with = "super")]
            time: SystemTime,
        }

        let json = r#"{"time": "2018-05-11 18:28:30"}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time, UNIX_EPOCH + Duration::new(1526063310, 0));
        let reverse = serde_json::to_string(&foo).unwrap();
        assert_eq!(reverse, r#"{"time":"2018-05-11T18:28:30Z"}"#);
    }

    #[test]
    fn time_with_option() {
        #[derive(Serialize, Deserialize)]
        struct Foo {
            #[serde(with = "super", default)]
            time: Option<SystemTime>,
        }

        let json = r#"{"time": "2018-05-11 18:28:30"}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time, Some(UNIX_EPOCH + Duration::new(1526063310, 0)));
        let reverse = serde_json::to_string(&foo).unwrap();
        assert_eq!(reverse, r#"{"time":"2018-05-11T18:28:30Z"}"#);

        let json = r#"{"time": null}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time, None);
        let reverse = serde_json::to_string(&foo).unwrap();
        assert_eq!(reverse, r#"{"time":null}"#);

        let json = r#"{}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time, None);
    }
}
