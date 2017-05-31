//! A crate providing Serde deserializers for `Duration`s via the `humantime`
//! crate.
//!
//! # Examples
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
#![warn(missing_docs)]
extern crate humantime;
extern crate serde;

#[cfg(test)]
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
extern crate serde_json;

use serde::de::{Deserializer, Visitor, Error, Unexpected};
use std::fmt;
use std::time::Duration;

/// Deserializes a `Duration` via the humantime crate.
///
/// This function can be used with `serde_derive`'s `with` and
/// `deserialize_with` annotations.
pub fn deserialize<'de, D>(d: D) -> Result<Duration, D::Error>
    where D: Deserializer<'de>
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
            humantime::parse_duration(v).map_err(|_| E::invalid_value(Unexpected::Str(v), &self))
        }
    }

    d.deserialize_str(V)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn derive() {
        #[derive(Deserialize)]
        struct Foo {
            #[serde(with = "super")]
            time: Duration,
        }

        let json = r#"{"time": "15 seconds"}"#;
        let foo = serde_json::from_str::<Foo>(json).unwrap();
        assert_eq!(foo.time, Duration::from_secs(15));
    }
}
