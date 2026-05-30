/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Decode actions returned by buffered decoder policy hooks.

/// Action selected after a codec decode attempt fails during `transcode`.
///
/// # Type Parameters
///
/// - `Value`: Decoded output value type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DecodeAction<Value> {
    /// More source units are needed before a value can be produced.
    NeedInput {
        /// Total units required from the current value start.
        required_total: usize,
    },

    /// Consume invalid input without producing output.
    Skip {
        /// Source units to consume.
        consumed: usize,
    },

    /// Produce one value and consume source units.
    Emit {
        /// Value to write to the output buffer.
        value: Value,
        /// Source units to consume.
        consumed: usize,
    },
}
