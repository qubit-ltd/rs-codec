// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use super::{
    capacity_error::CapacityError,
    transcode_error::TranscodeError,
    transcode_progress::TranscodeProgress,
    transcode_status::TranscodeStatus,
};

/// Converts one logical stream of input units into one logical stream of output
/// units.
///
/// `transcode` is the main streaming API. It transforms a provided input
/// segment and writes as much output as available buffer space allows.
///
/// A transcoder instance has a simple lifecycle:
///
/// 1. A newly created or reset instance is ready for a new logical stream.
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
/// codec-backed decoders, this means the underlying codec should be able to
/// decide each value boundary from the visible prefix plus its own state. If a
/// format needs EOF-aware maximal-munch parsing or must delay whether a prefix
/// is complete until the next chunk or EOF, implement that policy in a custom
/// `Transcoder` or a value-level facade.
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
///     TranscodeError,
///     TranscodeProgress,
///     TranscodeStatus,
///     Transcoder,
/// };
///
/// #[derive(Default)]
/// struct U16BeBytesDecoder;
///
/// impl Transcoder<u8, u16> for U16BeBytesDecoder {
///     type DomainError = core::convert::Infallible;
///     type Error = TranscodeError<Self::DomainError>;
///
///     fn map_error(&self, error: TranscodeError<Self::DomainError>) -> Self::Error {
///         error
///     }
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
///         TranscodeError::<Self::DomainError>::ensure_output_index(output.len(), output_index)?;
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
///         TranscodeError::<Self::DomainError>::ensure_transcode_indices(
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
///                     output_index: output_index + written,
///                     required: NonZeroUsize::MIN,
///                     available: 0,
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
///             let available = input.len() - (input_index + read);
///             let status = TranscodeStatus::NeedInput {
///                 input_index: input_index + read,
///                 required: qubit_io::nz!(2),
///                 available,
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
///         TranscodeError::<Self::DomainError>::ensure_output_index(output.len(), output_index)?;
///         Ok(0)
///     }
/// }
///
/// let mut transcoder = U16BeBytesDecoder;
/// let mut output = [0_u16; 1];
/// let progress = transcoder
///     .transcode(&[0x12, 0x34, 0xab, 0xcd], 0, &mut output, 0)
///     .expect("decoding cannot fail");
/// assert_eq!(TranscodeStatus::NeedOutput {
///     output_index: 1,
///     required: NonZeroUsize::MIN,
///     available: 0,
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
///     input_index: 2,
///     required: qubit_io::nz!(2),
///     available: 1,
/// }, progress.status());
/// assert_eq!(2, progress.read());
/// assert_eq!(1, progress.written());
/// assert_eq!([0x1234, 0], output);
///
/// assert!(matches!(
///     transcoder.transcode(&[0x12], 2, &mut output, 0),
///     Err(TranscodeError::InvalidInputIndex { .. }),
/// ));
/// assert!(matches!(
///     transcoder.transcode(&[0x12], 0, &mut output, 3),
///     Err(TranscodeError::InvalidOutputIndex { .. }),
/// ));
/// ```
///
/// The trait is intentionally independent from charset concepts. Implementors
/// use `input_index` and `output_index` as absolute positions in the supplied
/// slices. Returned progress counters are relative counts from those positions.
/// For raw codecs this gives a compact API; higher-level workflows can wrap
/// this trait with their own semantic policies.
///
/// # Type Parameters
///
/// - `Input`: Input unit type accepted by this transcoder.
/// - `Output`: Output unit type produced by this transcoder.
pub trait Transcoder<Input, Output> {
    /// Final error type exposed by this transcoder.
    type Error;

    /// Domain error type accepted from engine and hook internals.
    type DomainError;

    /// Maps an intermediate transcode error into the public final error.
    ///
    /// # Parameters
    ///
    /// - `error`: Intermediate transcode error produced by engine, hook, or
    ///   adapter internals.
    ///
    /// # Returns
    ///
    /// Returns the final error exposed by this transcoder.
    fn map_error(
        &self,
        error: TranscodeError<Self::DomainError>,
    ) -> Self::Error;

    /// Returns an upper bound for output units emitted when resetting stream
    /// state.
    ///
    /// Stateful encoders may need a stream-start sequence, such as a byte
    /// order mark, before the first encoded value. Callers use this bound to
    /// size the output buffer passed to [`Transcoder::reset`].
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
    /// For stateful transcoders, this bound is evaluated against the current
    /// instance state and must include any already-retained output that may be
    /// emitted before or alongside output derived from the supplied input
    /// during this phase.
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
    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError>;

    /// Returns an upper bound for a complete `reset -> transcode -> finish`
    /// stream.
    ///
    /// This is a convenience sum of [`Transcoder::max_reset_output_len`],
    /// [`Transcoder::max_transcode_output_len`], and
    /// [`Transcoder::max_finish_output_len`]. It is intended for callers that
    /// are about to run a full one-shot stream on an instance that is ready
    /// to start a new logical stream.
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
    fn max_total_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        let reset = self.max_reset_output_len()?;
        let transcode = self.max_transcode_output_len(input_len)?;
        let finish = self.max_finish_output_len()?;
        reset
            .checked_add(transcode)
            .and_then(|len| len.checked_add(finish))
            .ok_or(CapacityError::OutputLengthOverflow)
    }

    /// Returns an upper bound for output units produced by stream finalization.
    ///
    /// This bound is evaluated against the transcoder's current state. It does
    /// not include output that may be produced by future
    /// [`Transcoder::transcode`] calls. Use it before
    /// [`Transcoder::finish`] when the caller wants to size a final
    /// output buffer for the already supplied input.
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
    /// Stateless transcoders return `0`.
    ///
    /// # Errors
    ///
    /// Returns contract errors (`invalid_output_index`, `insufficient_output`)
    /// when capacity checks fail, or policy errors when reset itself fails.
    fn reset(
        &mut self,
        output: &mut [Output],
        output_index: usize,
    ) -> Result<usize, Self::Error>;

    /// Converts available input units into output units.
    ///
    /// This method processes an input segment without closing the logical input
    /// stream. When the current segment ends in a partial value, the transcoder
    /// reports [`crate::TranscodeStatus::NeedInput`] without consuming that
    /// tail. The caller owns input-buffer refill and EOF incomplete-tail
    /// policy.
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
    /// output ranges. The default one-shot helper checks this contract with a
    /// debug assertion; streaming I/O drivers may validate progress in
    /// release builds before advancing unsafe cursors.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` for semantic conversion failures that the
    /// transcoder's policy does not absorb, including caller-supplied
    /// `input_index` or `output_index` values outside their corresponding
    /// slices.
    fn transcode(
        &mut self,
        input: &[Input],
        input_index: usize,
        output: &mut [Output],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error>;

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
    ///     TranscodeError,
    ///     Transcoder,
    ///     TranscodeStatus,
    /// };
    ///
    /// #[derive(Default)]
    /// struct ByteCopy;
    ///
    /// impl Transcoder<u8, u8> for ByteCopy {
    ///     type DomainError = core::convert::Infallible;
    ///     type Error = TranscodeError<Self::DomainError>;
    ///
    ///     fn map_error(&self, error: TranscodeError<Self::DomainError>) -> Self::Error {
    ///         error
    ///     }
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
    ///         TranscodeError::<Self::DomainError>::ensure_output_index(output.len(), output_index)?;
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
    ///                 output_index: output_index + written,
    ///                 required: NonZeroUsize::MIN,
    ///                 available: output.len().saturating_sub(output_index + written),
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
    ///         TranscodeError::<Self::DomainError>::ensure_output_index(output.len(), output_index)?;
    ///         Ok(0)
    ///     }
    /// }
    ///
    /// let mut transcoder = ByteCopy;
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
    /// transcoders return `0`.
    ///
    /// # Errors
    ///
    /// Returns contract errors (`invalid_output_index`, `insufficient_output`)
    /// when capacity checks fail, or policy errors when finish itself
    /// fails.
    fn finish(
        &mut self,
        output: &mut [Output],
        output_index: usize,
    ) -> Result<usize, Self::Error>;

    /// Runs a complete one-shot `reset -> transcode -> finish` stream.
    ///
    /// The `input` slice is treated as complete input at EOF, and output is
    /// written from the beginning of `output`. Callers that need to operate on
    /// a range inside a larger buffer should slice the input or output
    /// before calling this method.
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
    /// an incomplete value. The method resets the stream before estimating
    /// the streaming and finish output, so stale pending output from a
    /// previous logical stream does not affect one-shot capacity checks.
    /// Returns domain errors from reset, transcode, or finish.
    fn transcode_complete_into(
        &mut self,
        input: &[Input],
        output: &mut [Output],
    ) -> Result<usize, Self::Error> {
        let mut output_cursor = self.reset(output, 0)?;
        let transcode_required =
            self.max_transcode_output_len(input.len()).map_err(|_| {
                self.map_error(TranscodeError::OutputLengthOverflow)
            })?;
        let finish_required = self.max_finish_output_len().map_err(|_| {
            self.map_error(TranscodeError::OutputLengthOverflow)
        })?;
        let remaining_required =
            transcode_required.checked_add(finish_required).ok_or_else(
                || self.map_error(TranscodeError::OutputLengthOverflow),
            )?;
        TranscodeError::ensure_output_capacity(
            output.len(),
            output_cursor,
            remaining_required,
        )
        .map_err(|error| self.map_error(error))?;

        let progress = self.transcode(input, 0, output, output_cursor)?;
        debug_assert!(
            progress
                .validate(
                    0,
                    input.len(),
                    output_cursor,
                    output.len().saturating_sub(output_cursor),
                )
                .is_ok(),
            "Transcoder::transcode returned invalid progress",
        );
        output_cursor += progress.written();
        match progress.status() {
            TranscodeStatus::Complete => {}
            TranscodeStatus::NeedOutput {
                output_index,
                required,
                available,
            } => {
                let error = TranscodeError::insufficient_output(
                    output_index,
                    required.get(),
                    available,
                );
                return Err(self.map_error(error));
            }
            TranscodeStatus::NeedInput {
                input_index,
                required,
                available,
            } => {
                let error = TranscodeError::incomplete_input(
                    input_index,
                    required.get(),
                    available,
                );
                return Err(self.map_error(error));
            }
        }
        output_cursor += self.finish(output, output_cursor)?;
        Ok(output_cursor)
    }
}
