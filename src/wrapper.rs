use std::fmt;
use std::ops::{Deref, DerefMut};
use std::time::{Duration, SystemTime};

use humantime;
use serde::{Deserialize, Deserializer, ser, de};

/// A wrapper type which implements `Serialize` and `Deserialize` for
/// types involving `SystemTime` and `Duration`.
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct Serde<T>(T);


impl<T> fmt::Debug for Serde<T>
    where T: fmt::Debug
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.0.fmt(formatter)
    }
}

impl<T> Deref for Serde<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Serde<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> Serde<T> {
    /// Consumes the `De`, returning the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> From<T> for Serde<T> {
    fn from(val: T) -> Serde<T> {
        Serde(val)
    }
}

impl<'de> Deserialize<'de> for Serde<Duration> {
    fn deserialize<D>(d: D) -> Result<Serde<Duration>, D::Error>
        where D: Deserializer<'de>
    {
        struct V;

        impl<'de2> de::Visitor<'de2> for V {
            type Value = Duration;

            fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt.write_str("a duration")
            }

            fn visit_str<E>(self, v: &str) -> Result<Duration, E>
                where E: de::Error
            {
                humantime::parse_duration(v)
                .map_err(|_| {
                    E::invalid_value(de::Unexpected::Str(v), &self)
                })

            }
        }

        d.deserialize_str(V).map(Serde)
    }
}

impl<'de> Deserialize<'de> for Serde<SystemTime> {
    fn deserialize<D>(d: D) -> Result<Serde<SystemTime>, D::Error>
        where D: Deserializer<'de>
    {
        struct V;

        impl<'de2> de::Visitor<'de2> for V {
            type Value = SystemTime;

            fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt.write_str("a timestamp")
            }

            fn visit_str<E>(self, v: &str) -> Result<SystemTime, E>
                where E: de::Error
            {
                humantime::parse_rfc3339_weak(v)
                .map_err(|_| {
                    E::invalid_value(de::Unexpected::Str(v), &self)
                })

            }
        }

        d.deserialize_str(V).map(Serde)
    }
}

impl<'de> Deserialize<'de> for Serde<Option<Duration>> {
    fn deserialize<D>(d: D) -> Result<Serde<Option<Duration>>, D::Error>
        where D: Deserializer<'de>
    {
        match Option::<Serde<Duration>>::deserialize(d)? {
            Some(Serde(dur)) => Ok(Serde(Some(dur))),
            None => Ok(Serde(None)),
        }
    }
}

impl<'de> Deserialize<'de> for Serde<Option<SystemTime>> {
    fn deserialize<D>(d: D) -> Result<Serde<Option<SystemTime>>, D::Error>
        where D: Deserializer<'de>
    {
        match Option::<Serde<SystemTime>>::deserialize(d)? {
            Some(Serde(dur)) => Ok(Serde(Some(dur))),
            None => Ok(Serde(None)),
        }
    }
}

impl<'a> ser::Serialize for Serde<&'a Duration> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: ser::Serializer
    {
        humantime::format_duration(*self.0).to_string().serialize(serializer)
    }
}

impl ser::Serialize for Serde<Duration> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: ser::Serializer
    {
        humantime::format_duration(self.0).to_string().serialize(serializer)
    }
}

impl<'a> ser::Serialize for Serde<&'a SystemTime> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: ser::Serializer
    {
        humantime::format_rfc3339(*self.0).to_string().serialize(serializer)
    }
}

impl ser::Serialize for Serde<SystemTime> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: ser::Serializer
    {
        humantime::format_rfc3339(self.0).to_string().serialize(serializer)
    }
}

impl<'a> ser::Serialize for Serde<&'a Option<Duration>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: ser::Serializer
    {
        match *self.0 {
            Some(dur) => serializer.serialize_some(&Serde(dur)),
            None => serializer.serialize_none(),
        }
    }
}

impl ser::Serialize for Serde<Option<Duration>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: ser::Serializer
    {
        Serde(&self.0).serialize(serializer)
    }
}

impl<'a> ser::Serialize for Serde<&'a Option<SystemTime>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: ser::Serializer
    {
        match *self.0 {
            Some(tm) => serializer.serialize_some(&Serde(tm)),
            None => serializer.serialize_none(),
        }
    }
}

impl ser::Serialize for Serde<Option<SystemTime>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: ser::Serializer
    {
        Serde(&self.0).serialize(serializer)
    }
}
