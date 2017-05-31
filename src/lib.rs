extern crate humantime;
extern crate serde;

#[cfg(test)]
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
extern crate serde_json;

use serde::de::{Deserialize, Deserializer, Visitor, Error, Unexpected};
use std::fmt;
use std::time::Duration;

pub struct De(pub Duration);

impl<'de> Deserialize<'de> for De {
    fn deserialize<D>(d: D) -> Result<De, D::Error>
        where D: Deserializer<'de>
    {
        struct V;

        impl<'de2> Visitor<'de2> for V {
            type Value = De;
            fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt.write_str("a duration")
            }

            fn visit_str<E>(self, v: &str) -> Result<De, E>
                where E: Error
            {
                humantime::parse_duration(v)
                    .map(De)
                    .map_err(|_| E::invalid_value(Unexpected::Str(v), &self))
            }
        }

        d.deserialize_str(V)
    }
}

pub fn deserialize<'de, D>(d: D) -> Result<Duration, D::Error>
    where D: Deserializer<'de>
{
    De::deserialize(d).map(|de| de.0)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn de() {
        let json = r#""15 seconds""#;
        let duration = serde_json::from_str::<De>(json).unwrap();
        assert_eq!(duration.0, Duration::from_secs(15));
    }

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
