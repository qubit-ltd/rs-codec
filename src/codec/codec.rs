// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Low-level value codec trait.

mod codec;

pub use codec::Codec;

/// Compile-time asserts the public unit-bound invariants required by [`Codec`].
///
/// # Type Parameters
///
/// - `C`: Codec implementation to validate.
///
/// # Returns
///
/// Returns unit `()`.
///
/// # Panics
///
/// Panics at compile time when either decode width bound is zero or when
/// [`Codec::MIN_UNITS_PER_VALUE`] is greater than
/// [`Codec::MAX_DECODE_UNITS_PER_VALUE`], because these invariants must hold
/// for any well-formed [`Codec`] implementation and violating them is always a
/// bug.
#[inline]
pub(crate) fn assert_unit_bounds<C>()
where
    C: codec::Codec,
{
    const {
        assert!(
            C::MIN_UNITS_PER_VALUE > 0,
            "Codec::MIN_UNITS_PER_VALUE must be non-zero",
        );
        assert!(
            C::MAX_DECODE_UNITS_PER_VALUE > 0,
            "Codec::MAX_DECODE_UNITS_PER_VALUE must be non-zero",
        );
        assert!(
            C::MIN_UNITS_PER_VALUE <= C::MAX_DECODE_UNITS_PER_VALUE,
            "Codec::MIN_UNITS_PER_VALUE must not exceed Codec::MAX_DECODE_UNITS_PER_VALUE",
        );
    }
}
