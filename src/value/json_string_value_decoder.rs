// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Serde JSON string decoder for one owned value.

use core::marker::PhantomData;

use serde::de::DeserializeOwned;
use serde_json::Error as JsonError;
use serde_json::from_str;

use super::ValueDecoder;

/// Decodes one JSON string into an owned value via `serde_json`.
///
/// # Type Parameters
///
/// - `T`: Target value type deserialized through Serde.
///
/// # Examples
///
/// ```rust
/// use qubit_codec::JsonStringValueDecoder;
/// use qubit_codec::ValueDecoder;
/// use serde::Deserialize;
///
/// #[derive(Debug, Deserialize, PartialEq)]
/// struct Point {
///     x: i32,
///     y: i32,
/// }
///
/// let mut decoder = JsonStringValueDecoder::<Point>::new();
/// let point = decoder.decode(r#"{"x":1,"y":2}"#).unwrap();
/// assert_eq!(point, Point { x: 1, y: 2 });
/// ```
#[must_use]
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonStringValueDecoder<T> {
    /// Tracks the owned output type without storing a value between calls.
    _marker: PhantomData<T>,
}

impl<T> JsonStringValueDecoder<T> {
    /// Creates a JSON string decoder for values of type `T`.
    ///
    /// # Returns
    ///
    /// Returns a stateless decoder without allocating or constructing a `T`.
    #[inline]
    pub const fn new() -> Self {
        Self { _marker: PhantomData }
    }
}

impl<T> ValueDecoder<str> for JsonStringValueDecoder<T>
where
    T: DeserializeOwned,
{
    /// JSON syntax or target-value deserialization failure.
    type Error = JsonError;
    /// Owned value deserialized independently of the input string lifetime.
    type Output = T;

    /// Deserializes `input` from a JSON string.
    ///
    /// # Parameters
    ///
    /// - `input`: One complete JSON value, optionally surrounded by whitespace.
    ///
    /// # Returns
    ///
    /// Returns an owned `T`; decoding may allocate according to the target
    /// type.
    ///
    /// # Errors
    ///
    /// Returns a JSON error for malformed or incomplete input, trailing
    /// non-whitespace data, or a value incompatible with `T`.
    #[inline]
    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        from_str(input)
    }
}
