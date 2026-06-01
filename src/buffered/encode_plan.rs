/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! One-value encoding plan used by buffered encoder hooks.

/// Describes how much output capacity one encoded value needs before writing.
///
/// `EncodePlan` is produced by [`crate::BufferedEncodeHooks::prepare_encode`]
/// and consumed by [`crate::BufferedEncodeHooks::write_encode`]. The capacity
/// field is a safe upper bound required by the concrete writer, not necessarily
/// the exact number of units that will be written.
///
/// # Type Parameters
///
/// - `P`: Concrete action interpreted by the encoder implementation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EncodePlan<P> {
    /// Output units that must be writable before calling `write_encode`.
    ///
    /// Default codec-backed encoders usually use
    /// [`crate::Codec::max_units_per_value`]. Domain-specific encoders may use
    /// a tighter bound, such as a charset encoder using its exact encoded
    /// length probe. A value of zero is valid for policies that consume input
    /// without producing output.
    pub max_output_units: usize,

    /// Concrete write action interpreted by the encoder implementation.
    pub action: P,
}

impl<P> EncodePlan<P> {
    /// Creates an encoding plan.
    ///
    /// # Parameters
    ///
    /// - `max_output_units`: Output capacity required before writing.
    /// - `action`: Concrete plan action for the encoder implementation.
    ///
    /// # Returns
    ///
    /// Returns an encoding plan carrying the supplied capacity bound and
    /// action.
    #[must_use]
    #[inline(always)]
    pub const fn new(max_output_units: usize, action: P) -> Self {
        Self {
            max_output_units,
            action,
        }
    }
}
