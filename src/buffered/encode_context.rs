/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Encode context passed to buffered encoder policy hooks.

/// Context for one prepared encode write inside a buffered encoder engine.
///
/// The engine builds this value only after
/// [`crate::BufferedEncodeHooks::prepare_encode`] has returned an
/// [`crate::EncodePlan`] and the caller-provided output slice has enough
/// writable capacity for that plan.
///
/// # Type Parameters
///
/// - `Value`: Logical input value type.
/// - `Unit`: Encoded output unit type.
/// - `P`: Prepared plan action type returned by the encode hooks.
#[derive(Debug)]
pub struct EncodeContext<'a, Value, Unit, P> {
    /// Input value being encoded.
    pub input_value: &'a Value,

    /// Absolute input index of [`input_value`](Self::input_value).
    pub input_index: usize,

    /// Plan action returned by [`crate::BufferedEncodeHooks::prepare_encode`].
    pub plan_action: P,

    /// Complete output unit slice visible to the encoder.
    pub output: &'a mut [Unit],

    /// Start position in [`output`](Self::output) where writing begins.
    pub output_index: usize,
}
