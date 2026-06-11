// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Contract errors reported by transcode engines and transcoder adapters.

/// Factory trait for caller-contract failures reported by transcode APIs.
///
/// Implementations combine these contract variants with their own semantic
/// failure variants in one public error type. Engines call the associated
/// functions when validating caller-supplied indices or one-shot finish/reset
/// output capacity.
///
/// # Type Parameters
///
/// - `Ctx`: Context needed to construct contract errors. Use `()` when the
///   error type does not need codec metadata.
pub trait TranscodeError<Ctx = ()> {
    /// Creates an invalid-input-index error.
    ///
    /// # Parameters
    ///
    /// - `context`: Context used to build the error.
    /// - `index`: Invalid input index supplied by the caller.
    /// - `len`: Length of the input slice.
    ///
    /// # Returns
    ///
    /// Returns the invalid-input-index error.
    fn invalid_input_index(context: Ctx, index: usize, len: usize) -> Self
    where
        Self: Sized;

    /// Creates an invalid-output-index error.
    ///
    /// # Parameters
    ///
    /// - `context`: Context used to build the error.
    /// - `index`: Invalid output index supplied by the caller.
    /// - `len`: Length of the output slice.
    ///
    /// # Returns
    ///
    /// Returns the invalid-output-index error.
    fn invalid_output_index(context: Ctx, index: usize, len: usize) -> Self
    where
        Self: Sized;

    /// Creates an insufficient-output error for one-shot finish or reset.
    ///
    /// # Parameters
    ///
    /// - `context`: Context used to build the error.
    /// - `output_index`: Output index supplied by the caller.
    /// - `required`: Output units required to finish in one call.
    /// - `available`: Output units available from `output_index`.
    ///
    /// # Returns
    ///
    /// Returns the insufficient-output error.
    fn insufficient_output(
        context: Ctx,
        output_index: usize,
        required: usize,
        available: usize,
    ) -> Self
    where
        Self: Sized;

    /// Validates that an output slice can hold one-shot finalization output.
    ///
    /// # Parameters
    ///
    /// - `context`: Context used to build contract errors.
    /// - `output_len`: Length of the output slice.
    /// - `output_index`: Output index supplied by the caller.
    /// - `required`: Output units required to finish in one call.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when output capacity is sufficient.
    ///
    /// # Errors
    ///
    /// Returns an invalid-output-index error when `output_index` is beyond the
    /// slice, or an insufficient-output error when fewer than `required` units
    /// are writable from `output_index`.
    #[inline]
    fn ensure_output_capacity(
        context: Ctx,
        output_len: usize,
        output_index: usize,
        required: usize,
    ) -> Result<(), Self>
    where
        Self: Sized,
    {
        if output_index > output_len {
            return Err(Self::invalid_output_index(
                context,
                output_index,
                output_len,
            ));
        }
        let available = output_len - output_index;
        if available < required {
            return Err(Self::insufficient_output(
                context,
                output_index,
                required,
                available,
            ));
        }
        Ok(())
    }
}
