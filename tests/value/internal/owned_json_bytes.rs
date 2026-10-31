// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Owned byte-oriented Serde target for JSON string-content tests.

use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

use serde::Deserialize;
use serde::Deserializer;
use serde::de::Error;
use serde::de::Visitor;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct OwnedJsonBytes(pub(crate) Vec<u8>);

impl<'de> Deserialize<'de> for OwnedJsonBytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BytesVisitor;

        impl<'de> Visitor<'de> for BytesVisitor {
            type Value = OwnedJsonBytes;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
                formatter.write_str("a byte string")
            }

            fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(OwnedJsonBytes(value.to_vec()))
            }
        }

        deserializer.deserialize_byte_buf(BytesVisitor)
    }
}
