/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Low-level value codec trait.

use core::num::NonZeroUsize;

/// Encodes and decodes one value or codec quantum against a unit buffer.
///
/// `Codec` is the lowest-level abstraction in the codec stack. It is intended
/// for hot paths that have already validated buffer capacity and want to avoid
/// constructing subslices for every value. Higher-level transcoders and
/// convenience APIs are responsible for checked buffer management and owned
/// output allocation.
///
/// `min_units_per_value` and `max_units_per_value` describe the representation
/// width bounds for one value. The minimum is a lower-bound hint for checked
/// layers: if fewer than this many units are available, no complete value can
/// exist, so a streaming caller can request more input, report an incomplete
/// EOF tail. For decoding, this minimum is the smallest safety precondition
/// checked callers must satisfy before entering
/// [`decode_unchecked`](Self::decode_unchecked). The maximum is the conservative
/// bound callers normally use to prove that unchecked writes stay inside the
/// provided output buffer.
///
/// # Type Parameters
///
/// - `Value`: Logical value decoded from or encoded into the buffer. This may be
///   a scalar such as `u64`, a `char`, or a fixed quantum such as `[u8; 3]`.
/// - `Unit`: Buffer unit used by the encoded representation.
///
/// # Safety
///
/// Implementors must uphold the safety contract documented by
/// [`decode_unchecked`](Self::decode_unchecked) and
/// [`encode_unchecked`](Self::encode_unchecked). In particular, unchecked
/// implementations must not read or write outside the caller-provided ranges.
/// Implementations should use `debug_assert!` to state the expected buffer
/// bounds at the unchecked entry point.
///
/// Implementations must also guarantee that
/// [`min_units_per_value`](Self::min_units_per_value) is less than or equal to
/// [`max_units_per_value`](Self::max_units_per_value). Both bounds are non-zero
/// by type, and `max_units_per_value` must be a valid upper bound for one
/// complete encoded value or codec quantum.
pub unsafe trait Codec<Value, Unit: Copy> {
    /// Error reported when decoding malformed units.
    type DecodeError;

    /// Error reported when encoding an unsupported value.
    type EncodeError;

    /// Returns the minimum possible unit count for one encoded value.
    ///
    /// This is a lower bound used by checked callers for planning and fast
    /// impossibility checks. If a streaming decoder has fewer than this many
    /// readable units, no complete value can be present at the current position.
    /// If the stream has reached EOF, such a tail is necessarily incomplete;
    /// otherwise the caller should read more input. Similarly, an encoder or
    /// transcoder can avoid calling into the codec when the remaining output
    /// capacity is smaller than this lower bound.
    ///
    /// This value does not prove that encoding will fit. For variable-width
    /// representations, a value may require more units, up to
    /// [`max_units_per_value`](Self::max_units_per_value). For decoding, this is
    /// the minimum safety precondition required by
    /// [`decode_unchecked`](Self::decode_unchecked); if fewer units are
    /// available, a checked caller must request more input or report a closed
    /// incomplete tail without calling into the unchecked method.
    ///
    /// # Returns
    ///
    /// Returns a non-zero lower bound for one complete value. Variable-width
    /// codecs such as LEB128 should return the shortest valid representation
    /// length. For example, a UTF-16 byte codec can return `2`, while its
    /// maximum is `4` because a surrogate pair needs four bytes.
    #[must_use]
    fn min_units_per_value(&self) -> NonZeroUsize;

    /// Returns the maximum non-zero unit count needed to encode or decode one value.
    ///
    /// # Returns
    ///
    /// Returns an upper bound for one complete value or codec quantum.
    #[must_use]
    fn max_units_per_value(&self) -> NonZeroUsize;

    /// Decodes one value from `input` starting at `index`.
    ///
    /// # Parameters
    ///
    /// - `input`: Source unit buffer.
    /// - `index`: Start index in `input`.
    ///
    /// # Returns
    ///
    /// Returns the decoded value and the non-zero number of consumed units.
    ///
    /// # Errors
    ///
    /// Returns `Self::DecodeError` when the units are malformed, non-canonical,
    /// incomplete, or otherwise invalid for this codec. The concrete error type
    /// carries the codec-specific reason and context.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `index` is a valid boundary in `input`
    /// and that at least [`min_units_per_value`](Self::min_units_per_value)
    /// units are readable from `index`. Implementations must not read beyond the
    /// currently available units under that precondition. They may return
    /// `Self::DecodeError` when those units are a valid but incomplete prefix.
    ///
    /// On success, implementations must return a consumed unit count no larger
    /// than the available input. The return type guarantees that successful
    /// decoding always consumes at least one unit. Implementations should use
    /// `debug_assert!` to state these unchecked entry-point assumptions.
    unsafe fn decode_unchecked(&self, input: &[Unit], index: usize)
    -> Result<(Value, NonZeroUsize), Self::DecodeError>;

    /// Encodes one borrowed value into `output` starting at `index`.
    ///
    /// # Parameters
    ///
    /// - `value`: Value to encode.
    /// - `output`: Destination unit buffer.
    /// - `index`: Start index in `output`.
    ///
    /// # Returns
    ///
    /// Returns the number of written units. Implementations may return `0` to
    /// represent a value that intentionally emits no encoded units.
    ///
    /// # Errors
    ///
    /// Returns `Self::EncodeError` when `value` cannot be represented by this
    /// codec.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that the implementation can write up to
    /// [`max_units_per_value`](Self::max_units_per_value) units starting at
    /// `index`.
    unsafe fn encode_unchecked(
        &self,
        value: &Value,
        output: &mut [Unit],
        index: usize,
    ) -> Result<usize, Self::EncodeError>;
}

/// Checks the public unit-bound invariant required by [`Codec`].
#[inline(always)]
pub(crate) fn debug_assert_unit_bounds<C, Value, Unit>(codec: &C)
where
    C: Codec<Value, Unit>,
    Unit: Copy,
{
    #[cfg(debug_assertions)]
    {
        debug_assert!(
            codec.min_units_per_value() <= codec.max_units_per_value(),
            "Codec::min_units_per_value() must not exceed Codec::max_units_per_value()",
        );
    }
    #[cfg(not(debug_assertions))]
    {
        let _ = codec;
    }
}
