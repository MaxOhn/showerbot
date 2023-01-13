use std::fmt;

use serde::{
    de::{Error, Visitor},
    Deserialize, Deserializer,
};
use time::{OffsetDateTime, PrimitiveDateTime};

pub(super) mod adjust_acc {
    use super::*;

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<f32, D::Error> {
        Ok(<f32 as Deserialize>::deserialize(d)? * 100.0)
    }
}

pub(super) mod datetime {
    use time::UtcOffset;

    use crate::util::datetime::{DATETIME_FORMAT, OFFSET_FORMAT};

    use super::*;

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<OffsetDateTime, D::Error> {
        d.deserialize_str(DateTimeVisitor)
    }

    pub(super) struct DateTimeVisitor;

    impl<'de> Visitor<'de> for DateTimeVisitor {
        type Value = OffsetDateTime;

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("a datetime string")
        }

        #[inline]
        fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
            if v.len() < 19 {
                return Err(Error::custom(format!(
                    "string too short for a datetime: `{v}`"
                )));
            }

            let (prefix, suffix) = v.split_at(19);

            let primitive =
                PrimitiveDateTime::parse(prefix, DATETIME_FORMAT).map_err(Error::custom)?;

            let offset = if suffix == "Z" {
                UtcOffset::UTC
            } else {
                UtcOffset::parse(suffix, OFFSET_FORMAT).map_err(Error::custom)?
            };

            Ok(primitive.assume_offset(offset))
        }
    }
}
