// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use super::capacity_error::CapacityError;
use super::transcode_failure::TranscodeFailure;
use super::transcode_progress::TranscodeProgress;
use super::transcode_status::TranscodeStatus;

/// Validates one-shot transcode progress and returns completed output length.
///
/// Keeping progress classification independent of the concrete transcoder
/// ensures every implementation receives identical trailing-input and
/// streaming-stop handling.
///
/// # Parameters
///
/// - `progress`: Progress returned by the streaming transcode phase.
/// - `input_len`: Total complete-input length supplied to the one-shot call.
/// - `output_index`: Absolute output index used for the transcode phase.
/// - `output_len`: Total output slice length.
///
/// # Returns
///
/// Returns the number of output units written when progress is complete.
///
/// # Errors
///
/// Returns a trailing-input, incomplete-input, or insufficient-output failure
/// when progress reports a non-complete one-shot result.
///
/// # Panics
///
/// Panics when the transcoder reports progress inconsistent with the supplied
/// input and output bounds.
fn complete_progress_written(
    progress: TranscodeProgress,
    input_len: usize,
    output_index: usize,
    output_len: usize,
) -> Result<usize, TranscodeFailure> {
    assert!(
        progress
            .validate(0, input_len, output_index, output_len.saturating_sub(output_index),)
            .is_ok(),
        "Transcoder::transcode returned invalid progress",
    );
    match progress.status() {
        TranscodeStatus::Complete => Ok(progress.written()),
        TranscodeStatus::NeedOutput { required } => Err(TranscodeFailure::insufficient_output(
            output_index + progress.written(),
            required.get(),
            output_len - output_index - progress.written(),
        )),
        TranscodeStatus::NeedInput { required } => Err(TranscodeFailure::incomplete_input(
            progress.read(),
            required.get(),
            input_len - progress.read(),
        )),
    }
}

/// Adds two independent output-capacity bounds.
///
/// # Parameters
///
/// - `first`: First output-capacity bound.
/// - `second`: Second output-capacity bound.
///
/// # Returns
///
/// Returns the sum of both bounds.
///
/// # Errors
///
/// Returns [`CapacityError::OutputLengthOverflow`] when the sum overflows.
fn add_output_bounds(first: usize, second: usize) -> Result<usize, CapacityError> {
    first.checked_add(second).ok_or(CapacityError::OutputLengthOverflow)
}

/// Adds the reset, transcode, and finish output-capacity bounds.
///
/// # Parameters
///
/// - `reset`: Reset-phase output-capacity bound.
/// - `transcode`: Streaming transcode-phase output-capacity bound.
/// - `finish`: Finish-phase output-capacity bound.
///
/// # Returns
///
/// Returns the complete lifecycle output-capacity bound.
///
/// # Errors
///
/// Returns [`CapacityError::OutputLengthOverflow`] when either addition
/// overflows.
fn sum_output_bounds(reset: usize, transcode: usize, finish: usize) -> Result<usize, CapacityError> {
    let before_finish = add_output_bounds(reset, transcode)?;
    add_output_bounds(before_finish, finish)
}

/// Converts one logical stream of input units into one logical stream of output
/// units.
///
/// `transcode` is the main streaming API. It transforms a provided input
/// segment and writes as much output as available buffer space allows. A
/// successful `transcode` call must stop with exactly one of three meanings:
/// all visible input was consumed (`Complete`), an incomplete tail remains
/// caller-owned (`NeedInput`), or output capacity prevented further progress
/// (`NeedOutput`).
///
/// A transcoder instance has a simple lifecycle:
///
/// 1. A newly created instance is uninitialized; call [`Transcoder::reset`]
///    before the first stream. A reset instance is ready for a new logical
///    stream.
/// 2. Call [`Transcoder::transcode`] zero or more times while input is
///    available.
/// 3. Preserve any tail reported by [`crate::TranscodeStatus::NeedInput`] in
///    the caller-owned input buffer.
/// 4. Call [`Transcoder::finish`] after the caller knows no more input remains
///    and has handled any incomplete tail. Size this final output with
///    [`Transcoder::max_finish_output_len`].
/// 5. After [`Transcoder::finish`] succeeds, call [`Transcoder::reset`] with a
///    buffer sized by [`Transcoder::max_reset_output_len`] before starting
///    another logical stream with the same instance.
///
/// The method is suitable for:
/// - pull-style consumers that call conversion repeatedly as buffers arrive;
/// - bounded output sinks that use `NeedOutput` progress during `transcode`;
/// - stateless and stateful codecs that all return progress-oriented stopping
///   reasons.
///
/// `finish` finalizes retained state only; it does not receive source input and
/// does not reinterpret a tail previously reported by `NeedInput`. For
/// codec-backed decoders, streaming uses
/// [`Codec::decode`](crate::Codec::decode) and an explicit EOF call may use
/// [`Codec::decode_eof`](crate::Codec::decode_eof)
/// through [`Transcoder::transcode_eof`]. If a format needs EOF-aware
/// maximal-munch parsing or must delay whether a prefix is complete until the
/// next chunk or EOF, implement that policy in the codec or override
/// `transcode_eof`.
///
/// `Transcoder` is intentionally independent from any charset
/// semantics:
///
/// - Use `Transcoder` directly for custom, policy-free unit transforms.
/// - Use `Transcoder` when you want to own malformed/unmappable decisions at
///   the call site.
///
/// # Example: streaming byte-to-word decoder
///
/// ```rust
/// use core::num::NonZeroUsize;
/// use qubit_codec::{
///     TranscodeDecodeError,
///     TranscodeProgress,
///     TranscodeStatus,
///     Transcoder,
/// };
///
/// #[derive(Default)]
/// struct U16BeBytesDecoder;
///
/// impl Transcoder for U16BeBytesDecoder {
///     type Input = u8;
///     type Output = u16;
///     type Error = TranscodeDecodeError<core::convert::Infallible>;
///
///     fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, qubit_codec::CapacityError> {
///         Ok(input_len / 2)
///     }
///
///     fn reset(
///         &mut self,
///         output: &mut [u16],
///         output_index: usize,
///     ) -> Result<usize, Self::Error> {
///         qubit_codec::TranscodeFailure::ensure_output_index(output.len(), output_index)?;
///         Ok(0)
///     }
///
///     fn transcode(
///         &mut self,
///         input: &[u8],
///         input_index: usize,
///         output: &mut [u16],
///         output_index: usize,
///     ) -> Result<TranscodeProgress, Self::Error> {
///         qubit_codec::TranscodeFailure::ensure_transcode_indices(
///             input.len(),
///             input_index,
///             output.len(),
///             output_index,
///         )?;
///
///         let mut read = 0;
///         let mut written = 0;
///         while input_index + read + 1 < input.len() {
///             if output_index + written == output.len() {
///                 let status = TranscodeStatus::NeedOutput {
///                     required: NonZeroUsize::MIN,
///                 };
///                 return Ok(TranscodeProgress::new(status, read, written));
///             }
///             let high = input[input_index + read] as u16;
///             let low = input[input_index + read + 1] as u16;
///             output[output_index + written] = (high << 8) | low;
///             read += 2;
///             written += 1;
///         }
///         if input_index + read == input.len() {
///             Ok(TranscodeProgress::complete(read, written))
///         } else {
///             let status = TranscodeStatus::NeedInput {
///                 required: NonZeroUsize::new(2).expect("two is non-zero"),
///             };
///             Ok(TranscodeProgress::new(status, read, written))
///         }
///     }
///
///     fn finish(
///         &mut self,
///         output: &mut [u16],
///         output_index: usize,
///     ) -> Result<usize, Self::Error> {
///         qubit_codec::TranscodeFailure::ensure_output_index(output.len(), output_index)?;
///         Ok(0)
///     }
/// }
///
/// let mut transcoder = U16BeBytesDecoder;
/// let mut reset_output = [];
/// transcoder
///     .reset(&mut reset_output, 0)
///     .expect("stateless decoder reset cannot fail");
/// let mut output = [0_u16; 1];
/// let progress = transcoder
///     .transcode(&[0x12, 0x34, 0xab, 0xcd], 0, &mut output, 0)
///     .expect("decoding cannot fail");
/// assert_eq!(TranscodeStatus::NeedOutput {
///     required: NonZeroUsize::MIN,
/// }, progress.status());
/// assert_eq!(2, progress.read());
/// assert_eq!(1, progress.written());
/// assert_eq!([0x1234], output);
///
/// let mut output = [0_u16; 2];
/// let progress = transcoder
///     .transcode(&[0x12, 0x34, 0xab], 0, &mut output, 0)
///     .expect("decoding cannot fail");
/// assert_eq!(TranscodeStatus::NeedInput {
///     required: NonZeroUsize::new(2).expect("two is non-zero"),
/// }, progress.status());
/// assert_eq!(2, progress.read());
/// assert_eq!(1, progress.written());
/// assert_eq!([0x1234, 0], output);
///
/// assert!(matches!(
///     transcoder.transcode(&[0x12], 2, &mut output, 0),
///     Err(TranscodeDecodeError::Failure(
///         qubit_codec::TranscodeFailure::InvalidInputIndex { .. }
///     )),
/// ));
/// assert!(matches!(
///     transcoder.transcode(&[0x12], 0, &mut output, 3),
///     Err(TranscodeDecodeError::Failure(
///         qubit_codec::TranscodeFailure::InvalidOutputIndex { .. }
///     )),
/// ));
/// ```
///
/// The trait is intentionally independent from charset concepts. Implementors
/// use `input_index` and `output_index` as absolute positions in the supplied
/// slices. Returned progress counters are relative counts from those positions.
/// For raw codecs this gives a compact API; higher-level workflows can wrap
/// this trait with their own semantic policies.
pub trait Transcoder {
    /// Input unit type accepted by this transcoder.
    type Input;

    /// Output unit type produced by this transcoder.
    type Output;

    /// Complete error type produced by this transcoder.
    type Error: From<TranscodeFailure>;

    /// Returns an upper bound for output units emitted when resetting stream
    /// state.
    ///
    /// Stateful encoders may need a stream-start sequence, such as a byte
    /// order mark, before the first encoded value. Callers use this bound to
    /// size the output buffer passed to [`Transcoder::reset`].
    ///
    /// The bound may depend on immutable transcoder configuration, but it must
    /// cover reset output from every reachable transient stream state. It must
    /// not shrink merely because the current stream has already been reset or
    /// finished.
    ///
    /// # Returns
    ///
    /// Returns `Ok(bound)` when the upper bound can be represented as `usize`.
    /// Returns [`CapacityError::OutputLengthOverflow`] when capacity arithmetic
    /// overflows. Stateless transcoders default to `Ok(0)`.
    #[must_use = "capacity planning can fail on overflow"]
    #[inline(always)]
    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        Ok(0)
    }

    /// Returns an upper bound for output units produced from `input_len` units
    /// during the streaming transcode phase.
    ///
    /// This bound excludes stream-start output emitted by
    /// [`Transcoder::reset`] and final output emitted by
    /// [`Transcoder::finish`]. Callers that need a complete one-shot stream
    /// bound should use [`Transcoder::max_total_output_len`].
    ///
    /// The bound may depend on immutable transcoder configuration and
    /// `input_len`, but it must cover the streaming output reachable from every
    /// transient stream state. In particular, it must include the maximum
    /// retained output that may be emitted before or alongside output derived
    /// from the supplied input, even when no output is currently retained.
    ///
    /// # Parameters
    ///
    /// - `input_len`: Number of input units the caller plans to transcode.
    ///
    /// # Returns
    ///
    /// Returns `Ok(bound)` when the upper bound can be represented as `usize`.
    /// Returns [`CapacityError::OutputLengthOverflow`] when capacity arithmetic
    /// overflows.
    #[must_use = "capacity planning can fail on overflow"]
    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError>;

    /// Returns an upper bound for a complete `reset -> transcode -> finish`
    /// stream.
    ///
    /// This is a convenience sum of [`Transcoder::max_reset_output_len`],
    /// [`Transcoder::max_transcode_output_len`], and
    /// [`Transcoder::max_finish_output_len`]. Each component is independent of
    /// transient stream state, so the sum is valid before a full one-shot
    /// stream regardless of how the previous logical stream ended.
    ///
    /// # Parameters
    ///
    /// - `input_len`: Number of input units in the complete stream.
    ///
    /// # Returns
    ///
    /// Returns `Ok(bound)` when the full-stream upper bound can be represented
    /// as `usize`. Returns [`CapacityError::OutputLengthOverflow`] when
    /// capacity arithmetic overflows.
    #[must_use = "capacity planning can fail on overflow"]
    #[inline]
    fn max_total_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        let reset = self.max_reset_output_len()?;
        let transcode = self.max_transcode_output_len(input_len)?;
        let finish = self.max_finish_output_len()?;
        sum_output_bounds(reset, transcode, finish)
    }

    /// Returns an upper bound for output units produced by stream finalization.
    ///
    /// The bound may depend on immutable transcoder configuration, but it must
    /// cover final output from every reachable transient stream state. For
    /// example, an encoder that can emit one checksum byte must return at least
    /// `1` before input, while accumulating the checksum, and after finishing a
    /// previous stream. It must not shrink merely because no finish output is
    /// currently pending.
    ///
    /// # Returns
    ///
    /// Returns `Ok(bound)` when the upper bound can be represented as `usize`.
    /// Returns [`CapacityError::OutputLengthOverflow`] when capacity arithmetic
    /// overflows. Stateless transcoders default to `Ok(0)`.
    #[must_use = "capacity planning can fail on overflow"]
    #[inline(always)]
    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(0)
    }

    /// Resets stream state and emits stream-start output into `output`.
    ///
    /// This starts a new logical stream while keeping configuration such as
    /// byte order, charset policy, replacement values, and cryptographic keys.
    /// Pending input, pending output, and completed-stream state must be
    /// discarded by stateful implementations. The caller must provide enough
    /// output capacity for [`Transcoder::max_reset_output_len`].
    ///
    /// # Parameters
    ///
    /// - `output`: Complete output unit slice visible to the transcoder.
    /// - `output_index`: Absolute output unit index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns the number of units written while resetting stream state.
    /// Stateless transcoders return `0`. Implementations must not return a
    /// count greater than [`Transcoder::max_reset_output_len`] or the output
    /// units available from `output_index`.
    ///
    /// # Errors
    ///
    /// Returns contract errors (`invalid_output_index`, `insufficient_output`)
    /// when capacity checks fail, or policy errors when reset itself fails.
    fn reset(&mut self, output: &mut [Self::Output], output_index: usize) -> Result<usize, Self::Error>;

    /// Converts available input units into output units.
    ///
    /// This method processes an input segment without closing the logical input
    /// stream. When the current segment ends in a partial value, the transcoder
    /// reports [`crate::TranscodeStatus::NeedInput`] without consuming that
    /// tail. The caller owns input-buffer refill and EOF incomplete-tail
    /// policy.
    ///
    /// Returning [`crate::TranscodeStatus::Complete`] means all input visible
    /// from `input_index` was consumed. Implementations must not return
    /// `Complete` after consuming only a prefix of the supplied input; they
    /// must instead continue processing, report `NeedInput` for an
    /// incomplete tail, or report `NeedOutput` if output capacity prevents
    /// progress. This invariant is checked by
    /// [`crate::TranscodeProgress::validate`].
    ///
    /// # Parameters
    ///
    /// - `input`: Complete input unit slice visible to the transcoder.
    /// - `input_index`: Absolute input unit index where conversion starts.
    /// - `output`: Complete output unit slice visible to the transcoder.
    /// - `output_index`: Absolute output unit index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns progress describing how many units were consumed and produced
    /// and why conversion stopped. Implementations must keep the returned
    /// counters and status fields consistent with the supplied input and
    /// output ranges. For `Complete`, `read` must equal `input.len() -
    /// input_index`. The default one-shot helper rejects incomplete `Complete`
    /// progress and asserts the remaining progress contract in all builds;
    /// streaming I/O drivers may convert validation failures before advancing
    /// unsafe cursors.
    ///
    /// # Errors
    ///
    /// Returns framework failures for invalid indices, insufficient output,
    /// incomplete input, and capacity overflow. Returns domain errors for
    /// semantic conversion failures that the transcoder's policy does not
    /// absorb.
    fn transcode(
        &mut self,
        input: &[Self::Input],
        input_index: usize,
        output: &mut [Self::Output],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error>;

    /// Converts source units after the caller has established end of input.
    ///
    /// The default preserves ordinary streaming behavior. Implementations that
    /// can resolve a trailing prefix using EOF-aware format rules should
    /// override it. If the default still reports
    /// [`TranscodeStatus::NeedInput`], this method returns
    /// [`TranscodeFailure::IncompleteInput`] instead. The default validates
    /// its input and output indices before delegating, then validates returned
    /// progress before calculating the incomplete-input context. A broken
    /// implementation therefore returns [`TranscodeFailure::InvalidProgress`]
    /// rather than causing arithmetic overflow or advancing invalid cursors.
    #[inline]
    fn transcode_eof(
        &mut self,
        input: &[Self::Input],
        input_index: usize,
        output: &mut [Self::Output],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        TranscodeFailure::ensure_transcode_indices(input.len(), input_index, output.len(), output_index)?;
        let available_input = input.len() - input_index;
        let available_output = output.len() - output_index;
        let progress = self.transcode(input, input_index, output, output_index)?;
        progress
            .validate(input_index, available_input, output_index, available_output)
            .map_err(TranscodeFailure::invalid_progress)?;
        match progress.status() {
            TranscodeStatus::NeedInput { required } => Err(TranscodeFailure::incomplete_input(
                input_index + progress.read(),
                required.get(),
                available_input - progress.read(),
            )
            .into()),
            _ => Ok(progress),
        }
    }

    /// Finishes internally retained output after all input has been supplied.
    ///
    /// `transcode` handles ordinary input consumption. `finish` is called once
    /// after the caller knows no more input remains and has handled any
    /// incomplete input tail reported by `transcode`. It emits final output
    /// derived from internal state, such as reset bytes, checksums, digests, or
    /// trailers. The caller must provide enough output capacity for
    /// [`Transcoder::max_finish_output_len`].
    ///
    /// After `finish` succeeds, the logical stream is closed. Portable callers
    /// should call [`Transcoder::reset`] with a buffer sized by
    /// [`Transcoder::max_reset_output_len`] before passing input for another
    /// logical stream to the same instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// use core::num::NonZeroUsize;
    /// use qubit_codec::{
    ///     TranscodeDecodeError,
    ///     Transcoder,
    ///     TranscodeStatus,
    /// };
    ///
    /// #[derive(Default)]
    /// struct ByteCopy;
    ///
    /// impl Transcoder for ByteCopy {
    ///     type Input = u8;
    ///     type Output = u8;
    ///     type Error = TranscodeDecodeError<core::convert::Infallible>;
    ///
    ///     fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, qubit_codec::CapacityError> {
    ///         Ok(input_len)
    ///     }
    ///
    ///     fn reset(
    ///         &mut self,
    ///         output: &mut [u8],
    ///         output_index: usize,
    ///     ) -> Result<usize, Self::Error> {
    ///         qubit_codec::TranscodeFailure::ensure_output_index(output.len(), output_index)?;
    ///         Ok(0)
    ///     }
    ///
    ///     fn transcode(
    ///         &mut self,
    ///         input: &[u8],
    ///         input_index: usize,
    ///         output: &mut [u8],
    ///         output_index: usize,
    ///     ) -> Result<qubit_codec::TranscodeProgress, Self::Error> {
    ///         let mut read = 0;
    ///         let mut written = 0;
    ///         while input_index + read < input.len() && output_index + written < output.len() {
    ///             output[output_index + written] = input[input_index + read];
    ///             read += 1;
    ///             written += 1;
    ///         }
    ///         if input_index + read == input.len() {
    ///             Ok(qubit_codec::TranscodeProgress::complete(read, written))
    ///         } else {
    ///             let status = qubit_codec::TranscodeStatus::NeedOutput {
    ///                 required: NonZeroUsize::MIN,
    ///             };
    ///             Ok(qubit_codec::TranscodeProgress::new(
    ///                 status,
    ///                 read,
    ///                 written,
    ///             ))
    ///         }
    ///     }
    ///
    ///     fn finish(
    ///         &mut self,
    ///         output: &mut [u8],
    ///         output_index: usize,
    ///     ) -> Result<usize, Self::Error> {
    ///         qubit_codec::TranscodeFailure::ensure_output_index(output.len(), output_index)?;
    ///         Ok(0)
    ///     }
    /// }
    ///
    /// let mut transcoder = ByteCopy;
    /// let mut reset_output = [];
    /// transcoder
    ///     .reset(&mut reset_output, 0)
    ///     .expect("stateless transcoder reset cannot fail");
    /// let mut output = [1_u8; 1];
    /// let progress = transcoder
    ///     .transcode(&[7], 0, &mut output, 0)
    ///     .expect("writer consumes one unit");
    /// assert_eq!(TranscodeStatus::Complete, progress.status());
    ///
    /// let written = transcoder
    ///     .finish(&mut output, 1)
    ///     .expect("finish does not emit final state for no-op transcoders");
    /// assert_eq!(0, written);
    /// ```
    ///
    /// # Parameters
    ///
    /// - `output`: Complete output unit slice visible to the transcoder.
    /// - `output_index`: Absolute output unit index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns the number of units written during finalization. Stateless
    /// transcoders return `0`. Implementations must not return a count greater
    /// than [`Transcoder::max_finish_output_len`] or the output units available
    /// from `output_index`.
    ///
    /// # Errors
    ///
    /// Returns contract errors (`invalid_output_index`, `insufficient_output`)
    /// when capacity checks fail, or policy errors when finish itself
    /// fails.
    fn finish(&mut self, output: &mut [Self::Output], output_index: usize) -> Result<usize, Self::Error>;

    /// Runs a complete one-shot `reset -> transcode -> finish` stream.
    ///
    /// The `input` slice is treated as complete input at EOF, and output is
    /// written from the beginning of `output`. Callers that need to operate on
    /// a range inside a larger buffer should slice the input or output
    /// before calling this method.
    ///
    /// Before invoking any lifecycle method, this method queries the
    /// state-independent reset, transcode, and finish bounds and requires
    /// `output` to provide at least the complete upper bound returned by
    /// [`Transcoder::max_total_output_len`] for `input.len()` units.
    ///
    /// # Parameters
    ///
    /// - `input`: Complete input unit slice.
    /// - `output`: Complete output unit slice where the stream starts at index
    ///   `0`.
    ///
    /// # Returns
    ///
    /// Returns the number of output units written to `output`.
    ///
    /// # Errors
    ///
    /// Returns framework errors when the output buffer is too small, when
    /// capacity arithmetic overflows, or when the complete input ends with
    /// an incomplete value. Capacity-bound overflow and insufficient
    /// complete-stream capacity are reported before `reset`, `transcode`, or
    /// `finish` is called, so those preflight failures do not write to
    /// `output` or advance transient stream state through lifecycle methods.
    /// Once reset begins, domain errors, incomplete input, runtime
    /// backpressure caused by an invalid bound implementation, or contract
    /// violations may leave partial output and advanced state; this method
    /// does not provide general transactional rollback.
    ///
    /// # Panics
    ///
    /// Panics when `reset` or `finish` reports writing beyond its declared
    /// bound or the available output, when an overridden `transcode_eof`
    /// returns invalid progress, or when a returned write count cannot be
    /// added to the output cursor. The default `transcode_eof` converts
    /// invalid `transcode` progress into [`TranscodeFailure::InvalidProgress`].
    fn transcode_complete_into(
        &mut self,
        input: &[Self::Input],
        output: &mut [Self::Output],
    ) -> Result<usize, Self::Error> {
        let reset_required = self.max_reset_output_len().map_err(TranscodeFailure::from)?;
        let transcode_required = self
            .max_transcode_output_len(input.len())
            .map_err(TranscodeFailure::from)?;
        let finish_required = self.max_finish_output_len().map_err(TranscodeFailure::from)?;
        let total_required =
            sum_output_bounds(reset_required, transcode_required, finish_required).map_err(TranscodeFailure::from)?;
        TranscodeFailure::ensure_output_capacity(output.len(), 0, total_required)?;

        let reset_written = self.reset(output, 0)?;
        assert!(
            reset_written <= reset_required,
            "Transcoder::reset wrote beyond its bound",
        );
        assert!(
            reset_written <= output.len(),
            "Transcoder::reset wrote beyond available output",
        );
        let mut output_cursor = reset_written;

        let progress = self.transcode_eof(input, 0, output, output_cursor)?;
        let transcode_written = complete_progress_written(progress, input.len(), output_cursor, output.len())?;
        output_cursor = output_cursor
            .checked_add(transcode_written)
            .expect("Transcoder::transcode write count overflowed the output cursor");

        let finish_available = output.len() - output_cursor;
        let finish_written = self.finish(output, output_cursor)?;
        assert!(
            finish_written <= finish_required,
            "Transcoder::finish wrote beyond its bound",
        );
        assert!(
            finish_written <= finish_available,
            "Transcoder::finish wrote beyond available output",
        );
        output_cursor = output_cursor
            .checked_add(finish_written)
            .expect("Transcoder::finish write count overflowed the output cursor");
        Ok(output_cursor)
    }
}
