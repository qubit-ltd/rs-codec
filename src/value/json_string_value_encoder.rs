// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Serde JSON string encoder for one owned value.

use core::marker::PhantomData;

use serde::Serialize;

use super::ValueEncoder;

/// Encodes one serializable value into a UTF-8 JSON string via `serde_json`.
///
/// # Type Parameters
///
/// - `T`: Source value type serialized through Serde.
///
/// # Examples
///
/// ```rust
/// use qubit_codec::{JsonStringValueEncoder, ValueEncoder};
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Point {
///     x: i32,
///     y: i32,
/// }
///
/// let mut encoder = JsonStringValueEncoder::<Point>::new();
/// let json = encoder.encode(&Point { x: 1, y: 2 }).unwrap();
/// assert_eq!(json, r#"{"x":1,"y":2}"#);
/// ```
#[must_use]
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonStringValueEncoder<T> {
    _marker: PhantomData<T>,
}

impl<T> JsonStringValueEncoder<T> {
    /// Creates a JSON string encoder for values of type `T`.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self { _marker: PhantomData }
    }
}

impl<T> ValueEncoder<T> for JsonStringValueEncoder<T>
where
    T: Serialize,
{
    type Error = serde_json::Error;
    type Output = String;

    /// Serializes `input` into a JSON string.
    #[inline]
    fn encode(&mut self, input: &T) -> Result<Self::Output, Self::Error> {
        serde_json::to_string(input)
    }
}
