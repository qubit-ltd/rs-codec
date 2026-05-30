/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Error construction contract used by buffered decoder engines.

/// Constructs adapter-level errors needed by buffered decoder engines.
///
/// [`crate::BufferedDecodeEngine`] owns generic buffered decoding control flow.
/// It needs one error that does not come from the wrapped codec: an input start
/// index outside the provided input slice. Concrete decoder error types
/// implement this trait to provide that error without making it a policy hook.
///
/// # Type Parameters
///
/// - `C`: Codec or decoder configuration type that provides context for the
///   error.
pub trait DecodeErrorFactory<C> {
    /// Creates an input-index error.
    ///
    /// # Parameters
    ///
    /// - `codec`: Codec or decoder configuration used for error context.
    /// - `index`: Invalid input index supplied by the caller.
    /// - `input_len`: Length of the input slice.
    ///
    /// # Returns
    ///
    /// Returns an error representing `index > input_len`.
    #[must_use]
    fn invalid_input_index(codec: &C, index: usize, input_len: usize) -> Self;
}
