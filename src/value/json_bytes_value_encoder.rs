// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Serde JSON bytes encoder for one owned value.

use core::marker::PhantomData;

use serde::Serialize;

use super::ValueEncoder;

/// Encodes one serializable value into UTF-8 JSON bytes via `serde_json`.
///
/// # Type Parameters
///
/// - `T`: Source value type serialized through Serde.
///
/// # Examples
///
/// ```rust
/// use qubit_codec::{JsonBytesValueEncoder, ValueEncoder};
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Point {
///     x: i32,
///     y: i32,
/// }
///
/// let mut encoder = JsonBytesValueEncoder::<Point>::new();
/// let json = encoder.encode(&Point { x: 1, y: 2 }).unwrap();
/// assert_eq!(json, br#"{"x":1,"y":2}"#);
/// ```
#[must_use]
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonBytesValueEncoder<T> {
    _marker: PhantomData<T>,
}

impl<T> JsonBytesValueEncoder<T> {
    /// Creates a JSON bytes encoder for values of type `T`.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self { _marker: PhantomData }
    }
}

impl<T> ValueEncoder<T> for JsonBytesValueEncoder<T>
where
    T: Serialize,
{
    type Error = serde_json::Error;
    type Output = Vec<u8>;

    /// Serializes `input` into UTF-8 JSON bytes.
    #[inline]
    fn encode(&mut self, input: &T) -> Result<Self::Output, Self::Error> {
        serde_json::to_vec(input)
    }
}
