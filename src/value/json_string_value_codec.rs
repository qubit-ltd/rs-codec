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
/// for registry-backed or bidirectional call sites. It holds no state between
/// calls, so it is `Copy` and `Default`, and encoding and decoding can share
/// one value through [`ValueEncoder`] and [`ValueDecoder`].
///
/// # Type Parameters
///
/// - `T`: Value type serialized to a JSON string and recovered from it.
///
/// # Examples
///
/// ```
/// use qubit_codec::JsonStringValueCodec;
/// use qubit_codec::ValueDecoder;
/// use qubit_codec::ValueEncoder;
///
/// let mut codec = JsonStringValueCodec::<String>::new();
/// assert_eq!(codec.encode(&"hello".to_string()).unwrap(), r#""hello""#.to_string());
/// assert_eq!(codec.decode(r#""hello""#).unwrap(), "hello");
/// ```
#[must_use]
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonStringValueCodec<T> {
    /// Associates the codec with `T` without retaining a value.
    _marker: PhantomData<T>,
}

impl<T> JsonStringValueCodec<T> {
    /// Creates a bidirectional JSON string codec for values of type `T`.
    ///
    /// # Returns
    ///
    /// Returns a stateless codec bound to values of type `T`.
    #[inline]
    pub const fn new() -> Self {
        Self { _marker: PhantomData }
    }
}

impl<T> ValueEncoder<T> for JsonStringValueCodec<T>
where
    T: Serialize,
{
    /// Serialization failure reported by Serde JSON.
    type Error = serde_json::Error;
    /// Newly allocated UTF-8 JSON string owned by the caller.
    type Output = String;

    /// Serializes `input` into a JSON string.
    ///
    /// The codec holds no state, so this delegates to a freshly created
    /// [`JsonStringValueEncoder`] and `self` stays usable for further calls.
    ///
    /// # Errors
    ///
    /// Returns [`serde_json::Error`] when `input` cannot be serialized to
    /// JSON, including when a custom `Serialize` implementation fails or when
    /// the value contains a map key that is not a string.
    #[inline]
    fn encode(&mut self, input: &T) -> Result<Self::Output, Self::Error> {
        JsonStringValueEncoder::<T>::new().encode(input)
    }
}

impl<T> ValueDecoder<str> for JsonStringValueCodec<T>
where
    T: DeserializeOwned,
{
    /// JSON syntax or target-value deserialization failure.
    type Error = serde_json::Error;
    /// Owned value deserialized independently of the input string lifetime.
    type Output = T;

    /// Deserializes `input` from a JSON string.
    ///
    /// The codec holds no state, so this delegates to a freshly created
    /// [`JsonStringValueDecoder`] and `self` stays usable for further calls.
    ///
    /// # Errors
    ///
    /// Returns [`serde_json::Error`] when `input` is not valid JSON or when a
    /// valid JSON document does not match the shape required by `T`.
    #[inline]
    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        JsonStringValueDecoder::<T>::new().decode(input)
    }
}
