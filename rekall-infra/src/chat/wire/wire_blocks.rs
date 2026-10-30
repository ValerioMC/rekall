use std::fmt;

use serde::de::value::SeqAccessDeserializer;
use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

use super::WireBlock;

/// A message's content: its blocks, or none when `claude` printed it as a plain string.
#[derive(Debug)]
pub(in crate::chat) struct WireBlocks(pub(in crate::chat) Vec<WireBlock>);

impl<'de> Deserialize<'de> for WireBlocks {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(BlocksVisitor)
    }
}

struct BlocksVisitor;

impl<'de> Visitor<'de> for BlocksVisitor {
    type Value = WireBlocks;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a list of content blocks or a plain string")
    }

    fn visit_str<E>(self, _plain: &str) -> Result<WireBlocks, E> {
        Ok(WireBlocks(Vec::new()))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, blocks: A) -> Result<WireBlocks, A::Error> {
        Vec::<WireBlock>::deserialize(SeqAccessDeserializer::new(blocks)).map(WireBlocks)
    }
}
