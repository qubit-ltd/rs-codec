/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Policy hooks used by buffered decoder engines.

use super::{
    decode_action::DecodeAction,
    decode_context::DecodeContext,
    transcode_progress::TranscodeProgress,
};
use crate::{
    CapacityError,
    Codec,
};

/// Policy hooks for [`crate::BufferedDecodeEngine`].
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
/// ```rust,ignore
/// use qubit_codec::{
///     BufferedDecodeHooks, CodecDecodeError, DecodeAction, DecodeContext,
/// };
///
/// struct ReplacementHooks;
///
/// impl BufferedDecodeHooks<MyCodec, u8, u8> for ReplacementHooks {
///     type Error = CodecDecodeError<MyDecodeError>;
///
///     fn handle_decode_error(
///         &mut self,
///         _codec: &MyCodec,
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
/// - `Unit`: Encoded input unit type.
/// - `Value`: Decoded output value type.
pub trait BufferedDecodeHooks<C, Unit, Value>
where
    C: Codec<Value, Unit>,
    Unit: Copy,
{
    /// Error type returned by the buffered decoder.
    type Error;

    /// Returns an upper bound for decoded values produced from `input_len` units.
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
    fn max_output_len(&self, codec: &C, input_len: usize) -> Result<usize, CapacityError> {
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
    /// # Errors
    ///
    /// Returns `Self::Error` when the policy rejects the input.
    fn handle_decode_error(
        &mut self,
        codec: &C,
        error: C::DecodeError,
        context: DecodeContext,
    ) -> Result<DecodeAction<Value>, Self::Error>;

    /// Creates an error for a caller-supplied input index outside the input slice.
    ///
    /// The generic engine detects this before invoking the codec. The hook owns
    /// the concrete decoder error type, so it also owns the adapter-level error
    /// construction.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    /// - `index`: Invalid input index supplied by the caller.
    /// - `input_len`: Length of the input slice.
    ///
    /// # Returns
    ///
    /// Returns the hook-specific error representing `index > input_len`.
    fn invalid_input_index(&mut self, codec: &C, index: usize, input_len: usize) -> Self::Error;

    /// Finishes hook-owned state and writes any retained output.
    ///
    /// The default implementation is a no-op for stateless decode hooks.
    /// Stateful hooks may emit final values such as checksums, reset markers, or
    /// other trailer data. If `output` does not provide enough capacity, return
    /// [`crate::TranscodeStatus::NeedOutput`] and keep the unwritten state for a
    /// later `finish` call.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    /// - `output`: Complete output value slice visible to the hook.
    /// - `output_index`: Absolute output value index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns progress for values written by finalization.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` when hook-owned state cannot be finalized.
    #[inline(always)]
    fn finish(
        &mut self,
        _codec: &C,
        output: &mut [Value],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if output_index > output.len() {
            return Ok(TranscodeProgress::need_output(output_index, 1, 0, 0, 0));
        }
        Ok(TranscodeProgress::complete(0, 0))
    }

    /// Resets hook-owned policy state.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    #[inline(always)]
    fn reset(&mut self, _codec: &C) {}
}
