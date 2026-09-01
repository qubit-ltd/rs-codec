// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Progress from a scratch-backed single-value decode lifecycle.

/// Main decoded value and lifecycle output lengths written to caller storage.
///
/// # Type Parameters
///
/// - `V`: Logical value type produced by the codec.
///
/// # Examples
///
/// ```
/// use qubit_codec::DecodeLifecycleProgress;
///
/// fn inspect<V>(progress: &DecodeLifecycleProgress<V>) -> usize {
///     progress.reset_written() + progress.finish_written()
/// }
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[must_use]
pub struct DecodeLifecycleProgress<V> {
    /// Main value decoded from the supplied input.
    value: V,
    /// Number of reset values written to caller storage.
    reset_written: usize,
    /// Number of finish values written to caller storage.
    finish_written: usize,
}

impl<V> DecodeLifecycleProgress<V> {
    /// Creates progress for a completed decode lifecycle.
    ///
    /// # Parameters
    ///
    /// - `value`: Main value decoded from the supplied input.
    /// - `reset_written`: Number of reset values written to caller storage.
    /// - `finish_written`: Number of finish values written to caller storage.
    ///
    /// # Returns
    ///
    /// Returns completed lifecycle progress.
    #[inline(always)]
    pub(crate) const fn new(value: V, reset_written: usize, finish_written: usize) -> Self {
        Self {
            value,
            reset_written,
            finish_written,
        }
    }

    /// Returns the main value decoded from the supplied input.
    ///
    /// # Returns
    ///
    /// Returns a shared reference to the main decoded value.
    #[inline(always)]
    #[must_use]
    pub const fn value(&self) -> &V {
        &self.value
    }

    /// Returns the number of reset values written to caller storage.
    ///
    /// # Returns
    ///
    /// Returns the initialized reset output length.
    #[inline(always)]
    #[must_use]
    pub const fn reset_written(&self) -> usize {
        self.reset_written
    }

    /// Returns the number of finish values written to caller storage.
    ///
    /// # Returns
    ///
    /// Returns the initialized finish output length.
    #[inline(always)]
    #[must_use]
    pub const fn finish_written(&self) -> usize {
        self.finish_written
    }

    /// Consumes this progress and returns its components.
    ///
    /// # Returns
    ///
    /// Returns the main value, reset output length, and finish output length.
    #[inline(always)]
    #[must_use]
    pub fn into_parts(self) -> (V, usize, usize) {
        (self.value, self.reset_written, self.finish_written)
    }
}
