// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Domain-neutral stream decode error signals.

use core::num::NonZeroUsize;

/// Optional stream-recovery signals exposed by codec decode errors.
///
/// Concrete codec errors remain responsible for describing their own domain
/// failure. This trait only standardizes the small amount of control-flow
/// information that streaming adapters need when deciding whether to read more
/// input or consume invalid units before reporting an error.
pub trait CodecDecodeSignal {
    /// Returns the total input units required from the current value start.
    ///
    /// # Returns
    ///
    /// Returns `Some(required)` for incomplete input prefixes, or `None` when
    /// the error does not request more input.
    #[must_use]
    #[inline(always)]
    fn required_total(&self) -> Option<usize> {
        None
    }

    /// Returns invalid input units that may be consumed to make progress.
    ///
    /// # Returns
    ///
    /// Returns `Some(consumed)` for malformed or non-canonical input that a
    /// streaming adapter may discard before surfacing the error, or `None`
    /// when no consumption hint is available.
    #[must_use]
    #[inline(always)]
    fn consumed_units(&self) -> Option<NonZeroUsize> {
        None
    }
}

impl CodecDecodeSignal for core::convert::Infallible {}
