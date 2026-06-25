// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy hooks used by buffered decoder engines.

use core::num::NonZeroUsize;

use super::super::{
    decode_context::DecodeContext,
    decode_invalid_action::DecodeInvalidAction,
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
/// decide whether invalid input should be skipped, replaced, or returned as an
/// error.
///
/// The hook receives a [`DecodeContext`] with absolute input/output cursors, so
/// errors can include useful positions without duplicating engine arithmetic.
/// Stateful hooks may also use [`finish_hooks`](Self::finish_hooks) to emit
/// final values after the caller has supplied all input and handled any
/// incomplete tail.
///
/// # Example
///
/// This hook replaces malformed units with `b'?'` and otherwise lets the engine
/// keep decoding. Incomplete input is reported by
/// [`crate::DecodeFailure`] before policy hooks are called.
///
/// ```rust
/// use core::num::NonZeroUsize;
/// use qubit_codec::{
///     TranscodeDecodeHooks,
///     Codec,
///     DecodeFailure,
///     CodecDecodeError,
///     DecodeInvalidAction,
///     DecodeContext,
/// };
///
/// #[derive(Clone, Copy)]
/// struct MyCodec;
///
/// #[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// enum MyDecodeError {
///     Malformed { consumed: NonZeroUsize },
/// }
///
/// impl Codec for MyCodec {
///     type Value = u8;
///     type Unit = u8;
///     type DecodeError = MyDecodeError;
///     type EncodeError = core::convert::Infallible;
///
///     const MIN_UNITS_PER_VALUE: NonZeroUsize = NonZeroUsize::MIN;
///     const MAX_UNITS_PER_VALUE: NonZeroUsize = NonZeroUsize::MIN;
///
///     unsafe fn decode(
///         &mut self,
///         input: &[u8],
///         index: usize,
///     ) -> Result<(u8, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
///         match input[index] {
///             0xff => Err(DecodeFailure::invalid(
///                 MyDecodeError::Malformed {
///                     consumed: NonZeroUsize::MIN,
///                 },
///                 NonZeroUsize::MIN,
///             )),
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
///     fn handle_invalid_decode(
///         &mut self,
///         _codec: &mut MyCodec,
///         error: MyDecodeError,
///         consumed: Option<NonZeroUsize>,
///         _context: DecodeContext,
///     ) -> Result<DecodeInvalidAction<u8>, Self::Error> {
///         match error {
///             MyDecodeError::Malformed { .. } => {
///                 Ok(DecodeInvalidAction::Emit {
///                     value: b'?',
///                     consumed: consumed.expect("codec reported malformed width"),
///                 })
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
    ///
    /// Engine methods wrap this type in
    /// [`crate::TranscodeDecodeEngineError::Hook`]. Codec lifecycle failures
    /// are reported separately through
    /// [`crate::TranscodeDecodeEngineError::Codec`].
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
    /// [`Codec::MIN_UNITS_PER_VALUE`].
    #[inline]
    #[must_use = "capacity planning can fail on overflow"]
    fn max_output_len(
        &self,
        _codec: &C,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(input_len / C::MIN_UNITS_PER_VALUE.get())
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
    #[inline(always)]
    #[must_use]
    fn max_finish_output_len(&self, _codec: &C) -> usize {
        0
    }

    /// Handles a codec-domain invalid decode error during `transcode`.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    /// - `error`: Invalid domain error returned by the codec.
    /// - `consumed`: Invalid input units that may be consumed by non-strict
    ///   policies.
    /// - `context`: Decode attempt context.
    ///
    /// # Returns
    ///
    /// Returns the action selected by this hook policy.
    ///
    /// Returned consuming actions must not consume more than
    /// `context.available()` input units.
    ///
    /// The engine treats violations as hook bugs and panics.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` when the policy rejects the input.
    fn handle_invalid_decode(
        &mut self,
        codec: &mut C,
        error: C::DecodeError,
        consumed: Option<NonZeroUsize>,
        context: DecodeContext,
    ) -> Result<DecodeInvalidAction<C::Value>, Self::Error>;

    /// Runs hook-owned cleanup as part of stream reset.
    ///
    /// Called after [`Codec::decode_reset`](crate::Codec::decode_reset) has
    /// written its own reset output. Stateless hooks may use the default no-op.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    #[inline(always)]
    fn reset_hooks(&mut self, _codec: &mut C) {}

    /// Finishes hook-owned state and writes any retained output.
    ///
    /// The default implementation is a no-op for stateless decode hooks.
    /// Stateful hooks may emit final values such as checksums, reset markers,
    /// or other trailer data. The caller must provide at least
    /// [`TranscodeDecodeHooks::max_finish_output_len`] writable slots from
    /// `output_index`. Implementations must not write beyond that declared
    /// final-output bound.
    ///
    /// Called after [`Codec::decode_flush`](crate::Codec::decode_flush) has
    /// written its own flush output.
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
    fn finish_hooks(
        &mut self,
        _codec: &mut C,
        _output: &mut [C::Value],
        _output_index: usize,
    ) -> Result<usize, Self::Error> {
        Ok(0)
    }
}
