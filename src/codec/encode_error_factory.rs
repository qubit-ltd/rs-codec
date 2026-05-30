/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Error construction contract used by buffered encoder engines.

/// Constructs adapter-level errors needed by buffered encoder engines.
///
/// [`crate::BufferedEncodeEngine`] owns generic buffered encoding control flow.
/// It needs one error that does not come from the wrapped codec: an input start
/// index outside the provided input slice. Concrete encoder error types
/// implement this trait to provide that error without making it a strategy hook.
///
/// # Type Parameters
///
/// - `C`: Codec or encoder configuration type that provides context for the
///   error.
pub trait EncodeErrorFactory<C> {
    /// Creates an input-index error.
    ///
    /// # Parameters
    ///
    /// - `codec`: Codec or encoder configuration used for error context.
    /// - `index`: Invalid input index supplied by the caller.
    /// - `input_len`: Length of the input slice.
    ///
    /// # Returns
    ///
    /// Returns an error representing `index > input_len`.
    #[must_use]
    fn invalid_input_index(codec: &C, index: usize, input_len: usize) -> Self;
}
