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
use serde_json::Error as JsonError;

use super::JsonBytesValueDecoder;
use super::JsonBytesValueEncoder;
use super::ValueDecoder;
use super::ValueEncoder;

/// Encodes and decodes one serializable value through UTF-8 JSON bytes.
///
/// This type combines [`JsonBytesValueEncoder`] and [`JsonBytesValueDecoder`]
/// for registry-backed or bidirectional call sites. It holds no state between
/// calls, so it is `Copy` and `Default`, and encoding and decoding can share
/// one value through [`ValueEncoder`] and [`ValueDecoder`].
///
/// # Type Parameters
///
/// - `T`: Value type serialized to JSON bytes and recovered from them.
///
/// # Examples
///
/// ```
/// use qubit_codec::JsonBytesValueCodec;
/// use qubit_codec::ValueDecoder;
/// use qubit_codec::ValueEncoder;
///
/// let mut codec = JsonBytesValueCodec::<String>::new();
/// assert_eq!(codec.encode(&"hello".to_string()).unwrap(), br#""hello""#.to_vec());
/// assert_eq!(codec.decode(br#""hello""#).unwrap(), "hello");
/// ```
#[must_use]
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonBytesValueCodec<T> {
    /// Associates the codec with `T` without retaining a value.
    _marker: PhantomData<T>,
}

impl<T> JsonBytesValueCodec<T> {
    /// Creates a bidirectional JSON bytes codec for values of type `T`.
    ///
    /// # Returns
    ///
    /// Returns a stateless codec bound to values of type `T`.
    #[inline]
    pub const fn new() -> Self {
        Self { _marker: PhantomData }
    }
}

impl<T> ValueEncoder<T> for JsonBytesValueCodec<T>
where
    T: Serialize,
{
    /// Serialization failure reported by Serde JSON.
    type Error = JsonError;
    /// Newly allocated UTF-8 JSON bytes owned by the caller.
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
    /// JSON syntax, UTF-8, or target-value deserialization failure.
    type Error = JsonError;
    /// Owned value deserialized independently of the input byte slice.
    type Output = T;

    /// Deserializes `input` from UTF-8 JSON bytes.
    #[inline]
    fn decode(&mut self, input: &[u8]) -> Result<Self::Output, Self::Error> {
        JsonBytesValueDecoder::<T>::new().decode(input)
    }
}
