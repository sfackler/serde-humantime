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
#![doc(html_root_url = "https://docs.rs/serde-humantime/0.1.2")]

extern crate humantime;
extern crate serde;

#[cfg(test)]
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
extern crate serde_json;

use serde::de::{Deserialize, Deserializer, Error, Unexpected, Visitor};
use std::fmt;
use std::time::Duration;

/// A wrapper type which implements `Deserialize` for types involving
/// `Duration`.
///
/// It can only be constructed through its `Deserialize` implementations.
#[derive(Debug)]
pub struct De<T>(T);

impl<T> De<T> {
    /// Consumes the `De`, returning the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<'de> Deserialize<'de> for De<Duration> {
    fn deserialize<D>(d: D) -> Result<De<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize(d).map(De)
    }
}

impl<'de> Deserialize<'de> for De<Option<Duration>> {
    fn deserialize<D>(d: D) -> Result<De<Option<Duration>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Option::<De<Duration>>::deserialize(d)? {
            Some(De(dur)) => Ok(De(Some(dur))),
            None => Ok(De(None)),
        }
    }
}

/// Deserializes a `Duration` via the humantime crate.
///
/// This function can be used with `serde_derive`'s `with` and
/// `deserialize_with` annotations.
pub fn deserialize<'de, D>(d: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    struct V;

    impl<'de2> Visitor<'de2> for V {
        type Value = Duration;

        fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            fmt.write_str("a duration")
        }

        fn visit_str<E>(self, v: &str) -> Result<Duration, E>
        where
            E: Error,
        {
            humantime::parse_duration(v).map_err(|_| E::invalid_value(Unexpected::Str(v), &self))
        }
    }

    d.deserialize_str(V)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn with() {
        #[derive(Deserialize)]
        struct Foo {
            #[serde(with = "super")] time: Duration,
        }

        let json = r#"{"time": "15 seconds"}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time, Duration::from_secs(15));
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
    fn de_debug_fmt() {
        #[derive(Deserialize, Debug)]
        struct Foo {
            time: De<Option<Duration>>,
        }

        let foo = Foo {
            time: De(Some(Duration::from_secs(7))),
        };

        let _ = format!("{:?}", foo);
    }
}
