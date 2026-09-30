// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Bidirectional Serde JSON string codec for one owned value.

use core::marker::PhantomData;

use serde::Serialize;
use serde::de::DeserializeOwned;

use super::JsonStringValueDecoder;
use super::JsonStringValueEncoder;
use super::ValueDecoder;
use super::ValueEncoder;

/// Encodes and decodes one serializable value through UTF-8 JSON strings.
///
/// This type combines [`JsonStringValueEncoder`] and [`JsonStringValueDecoder`]
/// for registry-backed or bidirectional call sites.
#[must_use]
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonStringValueCodec<T> {
    _marker: PhantomData<T>,
}

impl<T> JsonStringValueCodec<T> {
    /// Creates a bidirectional JSON string codec for values of type `T`.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self { _marker: PhantomData }
    }
}

impl<T> ValueEncoder<T> for JsonStringValueCodec<T>
where
    T: Serialize,
{
    type Error = serde_json::Error;
    type Output = String;

    /// Serializes `input` into a JSON string.
    #[inline]
    fn encode(&mut self, input: &T) -> Result<Self::Output, Self::Error> {
        JsonStringValueEncoder::<T>::new().encode(input)
    }
}

impl<T> ValueDecoder<str> for JsonStringValueCodec<T>
where
    T: DeserializeOwned,
{
    type Error = serde_json::Error;
    type Output = T;

    /// Deserializes `input` from a JSON string.
    #[inline]
    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        JsonStringValueDecoder::<T>::new().decode(input)
    }
}
