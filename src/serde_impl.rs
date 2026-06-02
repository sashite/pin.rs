//! `serde` support for [`Identifier`], compiled only with the `serde` feature.
//!
//! An identifier serializes as its canonical token string (for example `"+K^"`)
//! and deserializes by parsing that string. The implementation is `no_std`:
//! serialization borrows a stack buffer and deserialization borrows the input.

use core::fmt;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::Identifier;

impl Serialize for Identifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.encode().as_str())
    }
}

impl<'de> Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct IdentifierVisitor;

        impl Visitor<'_> for IdentifierVisitor {
            type Value = Identifier;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a PIN token string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Identifier::parse(value).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_str(IdentifierVisitor)
    }
}
