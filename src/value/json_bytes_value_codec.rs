// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Bidirectional Serde JSON bytes codec for one owned value.

use core::marker::PhantomData;

use serde::Serialize;
use serde::de::DeserializeOwned;

use super::JsonBytesValueDecoder;
use super::JsonBytesValueEncoder;
use super::ValueDecoder;
use super::ValueEncoder;

/// Encodes and decodes one serializable value through UTF-8 JSON bytes.
///
/// This type combines [`JsonBytesValueEncoder`] and [`JsonBytesValueDecoder`]
/// for registry-backed or bidirectional call sites.
#[must_use]
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonBytesValueCodec<T> {
    _marker: PhantomData<T>,
}

impl<T> JsonBytesValueCodec<T> {
    /// Creates a bidirectional JSON bytes codec for values of type `T`.
    #[inline]
    pub const fn new() -> Self {
        Self { _marker: PhantomData }
    }
}

impl<T> ValueEncoder<T> for JsonBytesValueCodec<T>
where
    T: Serialize,
{
    type Error = serde_json::Error;
    type Output = Vec<u8>;

    /// Serializes `input` into UTF-8 JSON bytes.
    #[inline]
    fn encode(&mut self, input: &T) -> Result<Self::Output, Self::Error> {
        JsonBytesValueEncoder::<T>::new().encode(input)
    }
}

impl<T> ValueDecoder<[u8]> for JsonBytesValueCodec<T>
where
    T: DeserializeOwned,
{
    type Error = serde_json::Error;
    type Output = T;

    /// Deserializes `input` from UTF-8 JSON bytes.
    #[inline]
    fn decode(&mut self, input: &[u8]) -> Result<Self::Output, Self::Error> {
        JsonBytesValueDecoder::<T>::new().decode(input)
    }
}
