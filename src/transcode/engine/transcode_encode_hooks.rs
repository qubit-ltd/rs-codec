// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy hooks used by buffered encoder engines.

use super::encode_unencodable_action::EncodeUnencodableAction;
use crate::{
    Codec,
    TranscodeEncodeError,
    TranscodeError,
};

/// Policy hooks for [`crate::TranscodeEncodeEngine`].
///
/// Hooks own policy state, such as replacement or ignore behavior, but not the
/// codec or engine cursor state. The engine owns the normal one-value encode
/// operation and calls hooks only when policy decisions are needed.
///
/// Implement this trait when a buffered encoder needs policy decisions around
/// individual values while reusing the common engine loop. Examples include
/// rejecting unsupported values with adapter-level context, skipping
/// unsupported values, replacing them with encodable values, resetting
/// hook-owned state in [`reset_hooks`](Self::reset_hooks), or emitting final
/// state in [`finish_hooks`](Self::finish_hooks).
///
/// The engine calls
/// [`handle_unencodable_encode`](Self::handle_unencodable_encode) only after
/// [`Codec::can_encode_value`] returns `false` for the current input
/// value. Encodable values are encoded directly by the engine with
/// [`Codec::encode_len`] and [`Codec::encode`].
///
/// # Example
///
/// This hook reports unsupported values as adapter-level errors.
///
/// ```rust
/// use core::{
///     convert::Infallible,
///     num::NonZeroUsize,
/// };
/// use qubit_codec::{
///     TranscodeEncodeHooks,
///     TranscodeEncodeError,
///     Codec,
///     DecodeFailure,
///     EncodeUnencodableAction,
/// };
///
/// #[derive(Clone, Copy)]
/// struct ByteCodec;
///
/// impl Codec for ByteCodec {
///     type Value = u8;
///     type Unit = u8;
///     type DecodeError = Infallible;
///     type EncodeError = Infallible;
///
///     const MIN_UNITS_PER_VALUE: NonZeroUsize = NonZeroUsize::MIN;
///     const MAX_UNITS_PER_VALUE: NonZeroUsize = NonZeroUsize::MIN;
///
///     unsafe fn decode(
///         &mut self,
///         input: &[u8],
///         index: usize,
///     ) -> Result<(u8, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
///         Ok((input[index], NonZeroUsize::MIN))
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
/// struct StrictHooks;
///
/// impl<C> TranscodeEncodeHooks<C> for StrictHooks
/// where
///     C: Codec,
/// {
///     fn handle_unencodable_encode(
///         &mut self,
///         _codec: &mut C,
///         _value: &C::Value,
///         input_index: usize,
///     ) -> Result<EncodeUnencodableAction<C::Value>, TranscodeEncodeError<C>> {
///         Ok(EncodeUnencodableAction::Reject)
///     }
/// }
/// ```
///
/// # Type Parameters
///
/// - `C`: Low-level codec owned by the engine.
pub trait TranscodeEncodeHooks<C>
where
    C: Codec,
{
    /// Returns the maximum output units needed for `input_len` values.
    ///
    /// This bound covers only the streaming encode phase driven by
    /// [`crate::TranscodeEncodeEngine::transcode`]. It must not include codec
    /// reset output, codec flush output, or hook finish output.
    ///
    /// The default implementation multiplies `input_len` by
    /// [`Codec::MAX_UNITS_PER_VALUE`]. This bound is valid for direct encoding,
    /// skipped unencodable values, and single-value replacements. Override it
    /// only when hook-owned pending output can be drained during `transcode`.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    /// - `input_len`: Number of input values the caller plans to encode.
    ///
    /// # Returns
    ///
    /// Returns a conservative upper bound for streaming output.
    #[inline]
    #[must_use = "capacity planning can fail on overflow"]
    fn max_transcode_output_len(
        &self,
        _codec: &C,
        input_len: usize,
    ) -> Result<usize, TranscodeEncodeError<C>> {
        input_len
            .checked_mul(C::MAX_UNITS_PER_VALUE.get())
            .ok_or_else(TranscodeError::output_length_overflow)
    }

    /// Returns an upper bound for units emitted by finishing hook-owned state.
    ///
    /// `finish` never receives more input values. Implementations must only
    /// report output derived from hook-owned state that remains after the
    /// caller has supplied all input.
    ///
    /// The default implementation returns `0`. Override it when
    /// [`finish_hooks`](Self::finish_hooks) can write trailers, checksums,
    /// delayed replacements, or any other hook-owned final output. Do not
    /// include [`Codec::MAX_ENCODE_FLUSH_UNITS`]; the engine adds the codec
    /// flush bound separately.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    ///
    /// # Returns
    ///
    /// Returns the finite final-output upper bound.
    #[inline(always)]
    #[must_use]
    fn max_finish_output_len(&self, _codec: &C) -> usize {
        0
    }

    /// Handles one unencodable input value.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    /// - `value`: Input value that [`Codec::can_encode_value`] rejected.
    /// - `input_index`: Absolute input index of `value`.
    ///
    /// # Returns
    ///
    /// Returns the action selected by this hook policy.
    ///
    /// Replacement actions must contain a value that the same codec can encode.
    /// The engine treats unencodable replacements as hook contract violations
    /// and panics.
    ///
    /// # Errors
    ///
    /// Returns [`TranscodeError`] when the policy rejects the value with a
    /// codec-domain error.
    fn handle_unencodable_encode(
        &mut self,
        codec: &mut C,
        value: &C::Value,
        input_index: usize,
    ) -> Result<EncodeUnencodableAction<C::Value>, TranscodeEncodeError<C>>;

    /// Runs hook-owned cleanup as part of stream reset.
    ///
    /// Called before [`Codec::encode_reset`](crate::Codec::encode_reset) writes
    /// its own reset output. Stateless hooks may use the default no-op.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    #[inline(always)]
    fn reset_hooks(&mut self, _codec: &mut C) {}

    /// Finishes hook-owned state and writes any retained output units.
    ///
    /// The default implementation is a no-op for stateless encode hooks.
    /// Stateful hooks may emit final units such as reset sequences, checksums,
    /// or trailers. The caller must provide at least
    /// [`TranscodeEncodeHooks::max_finish_output_len`] writable units from
    /// `output_index`. Implementations must not write beyond that declared
    /// final-output bound.
    ///
    /// Called after [`Codec::encode_flush`](crate::Codec::encode_flush) has
    /// written its own flush output.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    /// - `output`: Output unit slice visible to the hook.
    /// - `output_index`: Absolute output unit index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns the number of units written by finalization. This count must not
    /// exceed [`TranscodeEncodeHooks::max_finish_output_len`].
    ///
    /// # Errors
    ///
    /// Returns [`TranscodeError`] when hook-owned state cannot be finalized.
    #[inline(always)]
    fn finish_hooks(
        &mut self,
        _codec: &mut C,
        _output: &mut [C::Unit],
        _output_index: usize,
    ) -> Result<usize, TranscodeEncodeError<C>> {
        Ok(0)
    }
}
