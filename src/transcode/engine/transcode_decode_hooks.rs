// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy hooks used by buffered decoder engines.

use super::super::{
    decode_action::DecodeAction,
    decode_context::DecodeContext,
};
use crate::{
    CapacityError,
    Codec,
};

/// Policy hooks for [`crate::TranscodeDecodeEngine`].
///
/// Hooks own policy state, such as malformed-input replacement behavior. The
/// engine passes the codec into hook methods when policy code needs codec
/// metadata.
///
/// Implement this trait when a buffered decoder needs policy decisions after
/// the low-level codec reports an error. The engine handles input/output cursor
/// bookkeeping, output-capacity checks, and successful one-value decodes; hooks
/// decide whether a decode error means "need more input", "skip these units",
/// "emit a replacement value", or "return an error".
///
/// The hook receives a [`DecodeContext`] with absolute input/output cursors, so
/// errors can include useful positions without duplicating engine arithmetic.
/// Stateful hooks may also use [`finish`](Self::finish) to emit final values
/// after the caller has supplied all input and handled any incomplete tail.
///
/// # Example
///
/// This hook maps incomplete codec errors to `NeedInput`, replaces malformed
/// units with `b'?'`, and otherwise lets the engine keep decoding.
///
/// ```rust
/// use core::num::NonZeroUsize;
/// use qubit_codec::{
///     TranscodeDecodeHooks,
///     Codec,
///     CodecDecodeError,
///     DecodeAction,
///     DecodeContext,
/// };
///
/// #[derive(Clone, Copy)]
/// struct MyCodec;
///
/// #[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// enum MyDecodeError {
///     Incomplete { required_total: usize },
///     Malformed { consumed: NonZeroUsize },
/// }
///
/// unsafe impl Codec for MyCodec {
///     type Value = u8;
///     type Unit = u8;
///     type DecodeError = MyDecodeError;
///     type EncodeError = core::convert::Infallible;
///
///     fn min_units_per_value(&self) -> NonZeroUsize {
///         NonZeroUsize::MIN
///     }
///
///     fn max_units_per_value(&self) -> NonZeroUsize {
///         NonZeroUsize::MIN
///     }
///
///     unsafe fn decode(
///         &mut self,
///         input: &[u8],
///         index: usize,
///     ) -> Result<(u8, NonZeroUsize), Self::DecodeError> {
///         match input[index] {
///             0xff => Err(MyDecodeError::Malformed {
///                 consumed: NonZeroUsize::MIN,
///             }),
///             value => Ok((value, NonZeroUsize::MIN)),
///         }
///     }
///
///     unsafe fn encode(
///         &mut self,
///         value: &u8,
///         output: &mut [u8],
///         index: usize,
///     ) -> Result<NonZeroUsize, Self::EncodeError> {
///         output[index] = *value;
///         Ok(NonZeroUsize::MIN)
///     }
/// }
///
/// struct ReplacementHooks;
///
/// impl TranscodeDecodeHooks<MyCodec> for ReplacementHooks {
///     type Error = CodecDecodeError<MyDecodeError>;
///
///     fn handle_decode_error(
///         &mut self,
///         _codec: &mut MyCodec,
///         error: MyDecodeError,
///         _context: DecodeContext,
///     ) -> Result<DecodeAction<u8>, Self::Error> {
///         match error {
///             MyDecodeError::Incomplete { required_total } => {
///                 Ok(DecodeAction::NeedInput { required_total })
///             }
///             MyDecodeError::Malformed { consumed } => {
///                 Ok(DecodeAction::Emit { value: b'?', consumed })
///             }
///         }
///     }
/// }
/// ```
///
/// # Type Parameters
///
/// - `C`: Low-level codec owned by the engine.
pub trait TranscodeDecodeHooks<C>
where
    C: Codec,
{
    /// Domain error type returned by the buffered decoder policy.
    type Error;

    /// Returns an upper bound for decoded values produced from `input_len`
    /// units.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    /// - `input_len`: Number of source units the caller plans to decode.
    ///
    /// # Returns
    ///
    /// Returns a conservative upper bound derived from
    /// [`Codec::min_units_per_value`].
    #[must_use = "capacity planning can fail on overflow"]
    #[inline]
    fn max_output_len(
        &self,
        codec: &C,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(input_len / codec.min_units_per_value().get())
    }

    /// Returns an upper bound for values emitted by finishing hook-owned state.
    ///
    /// `finish` never receives more input. Implementations must only report
    /// output derived from hook-owned state that remains after the caller has
    /// handled any incomplete input tail.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    ///
    /// # Returns
    ///
    /// Returns the finite final-output upper bound.
    #[must_use]
    #[inline(always)]
    fn max_finish_output_len(&self, _codec: &C) -> usize {
        0
    }

    /// Handles a codec decode error during `transcode`.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    /// - `error`: Error returned by the codec.
    /// - `context`: Decode attempt context.
    ///
    /// # Returns
    ///
    /// Returns the action selected by this hook policy.
    ///
    /// Returned actions must be consistent with `context.available()`:
    /// - `NeedInput.required_total` must be greater than `context.available()`;
    /// - `Skip.consumed` and `Emit.consumed` must not exceed
    ///   `context.available()`.
    ///
    /// The engine treats violations as hook bugs and panics.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` when the policy rejects the input.
    fn handle_decode_error(
        &mut self,
        codec: &mut C,
        error: C::DecodeError,
        context: DecodeContext,
    ) -> Result<DecodeAction<C::Value>, Self::Error>;

    /// Maps a codec-level flush error into this hook's public error type.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    /// - `error`: Error returned by [`Codec::decode_flush`].
    ///
    /// # Returns
    ///
    /// Returns the hook-specific error.
    #[inline]
    fn map_decode_flush_error(
        &mut self,
        _codec: &mut C,
        _error: C::DecodeError,
    ) -> Self::Error {
        panic!(
            "TranscodeDecodeHooks::map_decode_flush_error must be implemented for fallible flush codecs"
        )
    }

    /// Finishes hook-owned state and writes any retained output.
    ///
    /// The default implementation is a no-op for stateless decode hooks.
    /// Stateful hooks may emit final values such as checksums, reset markers,
    /// or other trailer data. The caller must provide at least
    /// [`TranscodeDecodeHooks::max_finish_output_len`] writable slots from
    /// `output_index`. Implementations must not write beyond that declared
    /// final-output bound.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    /// - `output`: Output value slice visible to the hook.
    /// - `output_index`: Absolute output value index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns the number of values written by finalization. This count must
    /// not exceed [`TranscodeDecodeHooks::max_finish_output_len`].
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` when hook-owned state cannot be finalized.
    #[inline]
    fn finish(
        &mut self,
        _codec: &mut C,
        _output: &mut [C::Value],
        _output_index: usize,
    ) -> Result<usize, Self::Error> {
        Ok(0)
    }

    /// Resets hook-owned policy state.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` when hook-owned state cannot be reset.
    #[inline]
    fn reset(&mut self, _codec: &mut C) -> Result<(), Self::Error> {
        Ok(())
    }
}
