// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Serde JSON bytes decoder for one owned value.

use core::marker::PhantomData;

use serde::de::DeserializeOwned;
use serde_json::Error as JsonError;
use serde_json::from_slice;

use super::ValueDecoder;

/// Decodes UTF-8 JSON bytes into one owned value via `serde_json`.
///
/// # Type Parameters
///
/// - `T`: Target value type deserialized through Serde.
///
/// # Examples
///
/// ```rust
/// use qubit_codec::{JsonBytesValueDecoder, ValueDecoder};
/// use serde::Deserialize;
///
/// #[derive(Debug, Deserialize, PartialEq)]
/// struct Point {
///     x: i32,
///     y: i32,
/// }
///
/// let mut decoder = JsonBytesValueDecoder::<Point>::new();
/// let point = decoder.decode(br#"{"x":1,"y":2}"#).unwrap();
/// assert_eq!(point, Point { x: 1, y: 2 });
/// ```
#[must_use]
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonBytesValueDecoder<T> {
    /// Associates the decoder with `T` without retaining a value.
    _marker: PhantomData<T>,
}

impl<T> JsonBytesValueDecoder<T> {
    /// Creates a JSON bytes decoder for values of type `T`.
    ///
    /// # Returns
    ///
    /// Returns a stateless decoder without allocating or constructing a `T`.
    #[inline]
    pub const fn new() -> Self {
        Self { _marker: PhantomData }
    }
}

impl<T> ValueDecoder<[u8]> for JsonBytesValueDecoder<T>
where
    T: DeserializeOwned,
{
    /// JSON syntax, UTF-8, or target-value deserialization failure.
    type Error = JsonError;
    /// Owned value deserialized independently of the input byte slice.
    type Output = T;

    /// Deserializes `input` from UTF-8 JSON bytes.
    ///
    /// # Parameters
    ///
    /// - `input`: One complete UTF-8 JSON value, optionally surrounded by
    ///   whitespace.
    ///
    /// # Returns
    ///
    /// Returns an owned `T`; decoding may allocate according to the target
    /// type.
    ///
    /// # Errors
    ///
    /// Returns a JSON error for invalid UTF-8, malformed or incomplete input,
    /// trailing non-whitespace data, or a value incompatible with `T`.
    #[inline]
    fn decode(&mut self, input: &[u8]) -> Result<Self::Output, Self::Error> {
        from_slice(input)
    }
}
