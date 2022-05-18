use crate::util::constants::DATE_FORMAT;

use chrono::{offset::TimeZone, DateTime, Utc};
use rosu_v2::model::GameMods;
use serde::{
    de::{Error, Unexpected, Visitor},
    Deserializer,
};
use std::{fmt, str::FromStr};

struct MaybeDateTimeString;

impl<'de> Visitor<'de> for MaybeDateTimeString {
    type Value = Option<DateTime<Utc>>;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a string containing a datetime")
    }

    fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
        match Utc.datetime_from_str(v, DATE_FORMAT) {
            Ok(date) => Ok(Some(date)),
            Err(_) => Ok(None),
        }
    }

    fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
        d.deserialize_str(self)
    }

    fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
        Ok(None)
    }
}

struct MaybeF32String;

impl<'de> Visitor<'de> for MaybeF32String {
    type Value = Option<f32>;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a string containing an f32")
    }

    fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
        v.parse()
            .map(Some)
            .map_err(|_| Error::invalid_value(Unexpected::Str(v), &self))
    }

    fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
        d.deserialize_str(self)
    }

    fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
        Ok(None)
    }
}

struct MaybeU32String;

impl<'de> Visitor<'de> for MaybeU32String {
    type Value = Option<u32>;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a string containing an u32")
    }

    fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
        v.parse()
            .map(Some)
            .map_err(|_| Error::invalid_value(Unexpected::Str(v), &self))
    }

    fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
        d.deserialize_str(self)
    }

    fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
        Ok(None)
    }
}

struct MaybeModsString;

impl<'de> Visitor<'de> for MaybeModsString {
    type Value = Option<GameMods>;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a string containing gamemods")
    }

    fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
        let mut mods = GameMods::NoMod;

        if v == "None" {
            return Ok(Some(mods));
        }

        for result in v.split(',').map(GameMods::from_str) {
            match result {
                Ok(m) => mods |= m,
                Err(err) => {
                    return Err(Error::custom(format_args!(r#"invalid value "{v}": {err}"#)));
                }
            }
        }

        Ok(Some(mods))
    }

    fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
        d.deserialize_str(self)
    }

    fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
        Ok(None)
    }
}
