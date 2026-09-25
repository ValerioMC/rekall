//! A row id: a random UUID, stored as its lowercase hyphenated text.
//!
//! Text rather than SQLite's 16-byte blob, because the `note:` anchor is a prefix of the id as a
//! string (`LOWER(CAST(id AS String)) LIKE 'prefix%'` in the Java repository), and because an id
//! read out of the file with any SQLite client should look like the one the API hands out.

use std::fmt;
use std::str::FromStr;

use sea_orm::DeriveValueType;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, DeriveValueType)]
#[sea_orm(value_type = "String")]
pub struct Id(pub Uuid);

impl Id {
    /// `GenerationType.UUID`: a random (version 4) UUID.
    pub fn random() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.hyphenated())
    }
}

impl FromStr for Id {
    type Err = uuid::Error;

    /// `UUID.fromString`: the hyphenated form. Other spellings `uuid` accepts (braced, simple,
    /// urn) are accepted too; they name the same id.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(text).map(Self)
    }
}

impl From<Uuid> for Id {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Serialize for Id {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        text.parse().map_err(serde::de::Error::custom)
    }
}

/// Primary keys have to be buildable from a `u64` in SeaORM's trait bounds; a UUID key never is.
impl sea_orm::TryFromU64 for Id {
    fn try_from_u64(_: u64) -> Result<Self, sea_orm::DbErr> {
        Err(sea_orm::DbErr::ConvertFromU64("Id"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_its_text() {
        let id = Id::random();
        let text = id.to_string();
        assert_eq!(text.len(), 36);
        assert_eq!(text, text.to_lowercase());
        assert_eq!(text.parse::<Id>().unwrap(), id);
        assert_eq!(serde_json::to_string(&id).unwrap(), format!("\"{text}\""));
    }
}
