//! A point on the UTC timeline, as `java.time.Instant` was used here.
//!
//! Three spellings matter, and each is fixed:
//!
//! - On the wire (JSON) and in rendered text it is written the way `Instant.toString()` and
//!   Jackson's `InstantSerializer` wrote it, `DateTimeFormatter.ISO_INSTANT`: seconds always, the
//!   fraction in groups of three digits only as far as it is non-zero, and `Z`.
//! - In the database it is TEXT with exactly six fractional digits and `Z`, so that SQL ordering
//!   (`ORDER BY updated_at`) is ordering in time and an older row never sorts after a newer one.
//! - Its precision is the microsecond, which is what H2's `TIMESTAMP` column kept and what
//!   `Instant.now()` produced on the machines this ran on.

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, NaiveDateTime, SecondsFormat, TimeDelta, TimeZone, Timelike, Utc};
use sea_orm::DeriveValueType;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, DeriveValueType)]
#[sea_orm(value_type = "String", to_str = "Instant::to_db_string", from_str = "Instant::from_db_str")]
pub struct Instant(DateTime<Utc>);

impl Instant {
    pub const EPOCH: Instant = Instant(DateTime::<Utc>::UNIX_EPOCH);

    /// `Instant.now()`, cut to the microsecond the database keeps.
    pub fn now() -> Self {
        Self::from_datetime(Utc::now())
    }

    pub fn from_datetime(value: DateTime<Utc>) -> Self {
        let micros = value.nanosecond() / 1_000 * 1_000;
        Self(value.with_nanosecond(micros).unwrap_or(value))
    }

    pub fn from_epoch_millis(millis: i64) -> Option<Self> {
        Utc.timestamp_millis_opt(millis).single().map(Self)
    }

    pub fn from_epoch_seconds(seconds: i64) -> Option<Self> {
        Utc.timestamp_opt(seconds, 0).single().map(Self)
    }

    pub fn datetime(&self) -> DateTime<Utc> {
        self.0
    }

    pub fn epoch_millis(&self) -> i64 {
        self.0.timestamp_millis()
    }

    pub fn plus(&self, delta: TimeDelta) -> Self {
        Self::from_datetime(self.0 + delta)
    }

    pub fn minus(&self, delta: TimeDelta) -> Self {
        Self::from_datetime(self.0 - delta)
    }

    pub fn plus_seconds(&self, seconds: i64) -> Self {
        self.plus(TimeDelta::seconds(seconds))
    }

    pub fn plus_minutes(&self, minutes: i64) -> Self {
        self.plus(TimeDelta::minutes(minutes))
    }

    pub fn is_after(&self, other: &Instant) -> bool {
        self > other
    }

    pub fn is_before(&self, other: &Instant) -> bool {
        self < other
    }

    /// The time from `self` to `later`, negative when `later` is earlier.
    pub fn until(&self, later: &Instant) -> TimeDelta {
        later.0 - self.0
    }

    /// The column's spelling: always six fractional digits, so text order is time order.
    pub fn to_db_string(&self) -> String {
        self.0.format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string()
    }

    fn from_db_str(text: &str) -> Result<Self, chrono::ParseError> {
        Self::parse(text)
    }

    /// `Instant.parse` and `OffsetDateTime.parse(...).toInstant()` together: ISO 8601 with `Z`
    /// or an offset. A space instead of the `T`, and a bare local date-time read as UTC, are
    /// accepted too, which is what an H2 export and SQLite's own `CURRENT_TIMESTAMP` write.
    pub fn parse(text: &str) -> Result<Self, chrono::ParseError> {
        let text = text.trim();
        match DateTime::parse_from_rfc3339(text) {
            Ok(parsed) => Ok(Self::from_datetime(parsed.with_timezone(&Utc))),
            Err(first) => {
                let normalised = text.replacen(' ', "T", 1);
                if let Ok(parsed) = DateTime::parse_from_rfc3339(&normalised) {
                    return Ok(Self::from_datetime(parsed.with_timezone(&Utc)));
                }
                for format in ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%dT%H:%M:%S", "%Y-%m-%dT%H:%M"] {
                    if let Ok(local) = NaiveDateTime::parse_from_str(&normalised, format) {
                        return Ok(Self::from_datetime(local.and_utc()));
                    }
                }
                Err(first)
            }
        }
    }
}

impl fmt::Display for Instant {
    /// `DateTimeFormatter.ISO_INSTANT`, which is what `Instant.toString()` is.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // chrono's AutoSi is the same rule: no fraction when it is zero, else 3, 6 or 9 digits.
        f.write_str(&self.0.to_rfc3339_opts(SecondsFormat::AutoSi, true))
    }
}

impl FromStr for Instant {
    type Err = chrono::ParseError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::parse(text)
    }
}

impl Serialize for Instant {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Instant {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text).map_err(|_| {
            serde::de::Error::custom(format!(
                "Cannot deserialize value of type `java.time.Instant` from String \"{text}\""
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str) -> Instant {
        Instant::parse(text).unwrap()
    }

    #[test]
    fn prints_the_way_java_prints_an_instant() {
        assert_eq!(at("2024-05-01T10:15:30Z").to_string(), "2024-05-01T10:15:30Z");
        assert_eq!(at("2024-05-01T10:15:30.100Z").to_string(), "2024-05-01T10:15:30.100Z");
        assert_eq!(at("2024-05-01T10:15:30.123456Z").to_string(), "2024-05-01T10:15:30.123456Z");
        assert_eq!(at("2024-05-01T10:15:30.120001Z").to_string(), "2024-05-01T10:15:30.120001Z");
    }

    #[test]
    fn keeps_microseconds_and_no_more() {
        assert_eq!(at("2024-05-01T10:15:30.123456789Z").to_string(), "2024-05-01T10:15:30.123456Z");
    }

    #[test]
    fn reads_an_offset_as_the_same_instant() {
        assert_eq!(at("2024-05-01T12:15:30+02:00"), at("2024-05-01T10:15:30Z"));
    }

    #[test]
    fn the_stored_text_sorts_in_time_order() {
        let earlier = at("2024-05-01T10:15:30Z").to_db_string();
        let later = at("2024-05-01T10:15:30.000001Z").to_db_string();
        assert_eq!(earlier, "2024-05-01T10:15:30.000000Z");
        assert!(earlier < later);
    }
}
