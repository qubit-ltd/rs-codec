/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Error construction contract used by buffered convert engines.

/// Constructs adapter-level errors needed by buffered convert engines.
///
/// [`crate::BufferedConvertEngine`] owns generic buffered conversion control
/// flow. It needs one error that does not come from a source decoder or target
/// encoder: an input start index outside the provided input slice. Concrete
/// converter error types implement this trait to provide that error without
/// making it a policy hook responsibility.
///
/// # Type Parameters
///
/// - `D`: Source decoder or codec type that provides context for the error.
pub trait ConvertErrorFactory<D> {
    /// Creates an input-index error.
    ///
    /// # Parameters
    ///
    /// - `decoder`: Source decoder or codec used for error context.
    /// - `index`: Invalid input index supplied by the caller.
    /// - `input_len`: Length of the input slice.
    ///
    /// # Returns
    ///
    /// Returns an error representing `index > input_len`.
    #[must_use]
    fn invalid_input_index(decoder: &D, index: usize, input_len: usize) -> Self;
}
