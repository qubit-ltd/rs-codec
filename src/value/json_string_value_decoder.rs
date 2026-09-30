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
/// use qubit_codec::{JsonStringValueDecoder, ValueDecoder};
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
    _marker: PhantomData<T>,
}

impl<T> JsonStringValueDecoder<T> {
    /// Creates a JSON string decoder for values of type `T`.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self { _marker: PhantomData }
    }
}

impl<T> ValueDecoder<str> for JsonStringValueDecoder<T>
where
    T: DeserializeOwned,
{
    type Error = serde_json::Error;
    type Output = T;

    /// Deserializes `input` from a JSON string.
    #[inline]
    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        serde_json::from_str(input)
    }
}
