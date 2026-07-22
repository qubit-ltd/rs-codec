// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Compile-time `NonZeroUsize` construction helpers.

use core::num::NonZeroUsize;

/// Returns a [`NonZeroUsize`] from a known non-zero value.
///
/// This helper is const-evaluable, so a zero constant fails during const
/// evaluation. Constant non-zero inputs compile down without the
/// `unsafe { NonZeroUsize::new_unchecked(...) }` ceremony otherwise repeated
/// by concrete codecs.
///
/// # Parameters
///
/// - `value`: Non-zero item count.
///
/// # Returns
///
/// Returns a [`NonZeroUsize`] equal to `value`.
///
/// # Panics
///
/// Panics when `value` is zero.
#[must_use]
#[inline(always)]
pub const fn nz(value: usize) -> NonZeroUsize {
    match NonZeroUsize::new(value) {
        Some(value) => value,
        None => panic!("qubit_codec::nz!(): value must be non-zero"),
    }
}

/// Constructs a [`NonZeroUsize`] from a const-evaluable expression.
///
/// # Examples
///
/// ```
/// const WIDTH: core::num::NonZeroUsize = qubit_codec::nz!(4);
///
/// assert_eq!(WIDTH.get(), 4);
/// ```
///
/// # Panics
///
/// Panics when the supplied value is zero.
#[macro_export]
macro_rules! nz {
    ($value:expr) => {{ $crate::nz_const($value) }};
}

/// Const-friendly function used by [`nz!`](crate::nz).
///
/// The macro qualifies this function through `$crate`, allowing callers to
/// import only the macro.
///
/// # Parameters
///
/// - `value`: Non-zero item count.
///
/// # Returns
///
/// Returns a [`NonZeroUsize`] equal to `value`.
///
/// # Panics
///
/// Panics when `value` is zero.
#[doc(hidden)]
#[must_use]
#[inline(always)]
pub const fn nz_const(value: usize) -> NonZeroUsize {
    nz(value)
}
