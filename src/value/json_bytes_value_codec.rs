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

/// Encodes and decodes one serializable value through JSON bytes.
///
/// This type combines [`JsonBytesValueEncoder`] and [`JsonBytesValueDecoder`]
/// for registry-backed or bidirectional call sites. It holds no state between
/// calls, so it is `Copy` and `Default` without requiring those traits on `T`;
/// encoding and decoding can share one value through [`ValueEncoder`] and
/// [`ValueDecoder`].
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
#[derive(Debug)]
pub struct JsonBytesValueCodec<T> {
    /// Associates the codec with `T` without retaining a value.
    _marker: PhantomData<T>,
}

impl<T> Copy for JsonBytesValueCodec<T> {}

impl<T> Clone for JsonBytesValueCodec<T> {
    /// Returns an identical stateless codec without requiring `T: Clone`.
    ///
    /// # Returns
    ///
    /// Returns a codec associated with the same value type.
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Default for JsonBytesValueCodec<T> {
    /// Creates a codec for `T` without constructing a value of that type.
    ///
    /// # Returns
    ///
    /// Returns a stateless codec associated with `T`.
    #[inline]
    fn default() -> Self {
        Self::new()
    }
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
    ///
    /// # Parameters
    ///
    /// - `input`: Value borrowed for the duration of serialization.
    ///
    /// # Returns
    ///
    /// Returns a newly allocated vector containing the serialized JSON.
    ///
    /// # Errors
    ///
    /// Returns a JSON error if `T` rejects serialization or contains a map key
    /// that cannot be represented as a JSON object key.
    #[inline]
    fn encode(&mut self, input: &T) -> Result<Self::Output, Self::Error> {
        JsonBytesValueEncoder::<T>::new().encode(input)
    }
}

impl<T> ValueDecoder<[u8]> for JsonBytesValueCodec<T>
where
    T: DeserializeOwned,
{
    /// JSON syntax or target-value deserialization failure.
    type Error = JsonError;
    /// Owned value deserialized independently of the input byte slice.
    type Output = T;

    /// Deserializes `input` using [`serde_json::from_slice`].
    ///
    /// # Parameters
    ///
    /// - `input`: One complete JSON value, optionally surrounded by whitespace.
    ///   String-content validation follows `T`'s Serde deserialization
    ///   implementation.
    ///
    /// # Returns
    ///
    /// Returns an owned `T`; decoding may allocate according to the target
    /// type.
    ///
    /// # Errors
    ///
    /// Returns a JSON error for malformed or incomplete input, trailing
    /// non-whitespace data, or a value incompatible with `T`. Validation
    /// follows serde_json and `T`'s `Deserialize` implementation; byte-oriented
    /// targets may accept non-UTF-8 string contents.
    #[inline]
    fn decode(&mut self, input: &[u8]) -> Result<Self::Output, Self::Error> {
        JsonBytesValueDecoder::<T>::new().decode(input)
    }
}
