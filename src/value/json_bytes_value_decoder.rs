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
    _marker: PhantomData<T>,
}

impl<T> JsonBytesValueDecoder<T> {
    /// Creates a JSON bytes decoder for values of type `T`.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self { _marker: PhantomData }
    }
}

impl<T> ValueDecoder<[u8]> for JsonBytesValueDecoder<T>
where
    T: DeserializeOwned,
{
    type Error = serde_json::Error;
    type Output = T;

    /// Deserializes `input` from UTF-8 JSON bytes.
    #[inline]
    fn decode(&mut self, input: &[u8]) -> Result<Self::Output, Self::Error> {
        serde_json::from_slice(input)
    }
}
