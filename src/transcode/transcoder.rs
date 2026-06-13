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
///     CodecDecodeError,
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
///     type Error = CodecDecodeError<core::convert::Infallible>;
///
///     fn max_output_len(&self, input_len: usize) -> Result<usize, qubit_codec::CapacityError> {
///         Ok(input_len / 2)
///     }
///
///     fn reset(
///         &mut self,
///         output: &mut [u16],
///         output_index: usize,
///     ) -> Result<usize, TranscodeError<Self::Error>> {
///         TranscodeError::ensure_output_index(output.len(), output_index)?;
///         Ok(0)
///     }
///
///     fn transcode(
///         &mut self,
///         input: &[u8],
///         input_index: usize,
///         output: &mut [u16],
///         output_index: usize,
///     ) -> Result<TranscodeProgress, TranscodeError<Self::Error>> {
///         TranscodeError::ensure_transcode_indices(
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
///                     additional: NonZeroUsize::MIN,
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
///                 additional: NonZeroUsize::new(2 - available).expect("missing input is non-zero"),
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
///     ) -> Result<usize, TranscodeError<Self::Error>> {
///         TranscodeError::ensure_output_index(output.len(), output_index)?;
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
///     additional: NonZeroUsize::MIN,
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
///     additional: NonZeroUsize::MIN,
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
    /// Domain error reported by semantic conversion failures.
    type Error;

    /// Returns an upper bound for output units produced from `input_len` units.
    ///
    /// For stateful transcoders, this bound is evaluated against the current
    /// instance state and must include any already-retained output that may be
    /// emitted before or alongside output derived from the supplied input.
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
    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError>;

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
    ) -> Result<usize, TranscodeError<Self::Error>>;

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
    /// and why conversion stopped.
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
    ) -> Result<TranscodeProgress, TranscodeError<Self::Error>>;

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
    ///     CodecConvertError,
    ///     TranscodeError,
    ///     Transcoder,
    ///     TranscodeStatus,
    /// };
    ///
    /// #[derive(Default)]
    /// struct ByteCopy;
    ///
    /// impl Transcoder<u8, u8> for ByteCopy {
    ///     type Error =
    ///         CodecConvertError<core::convert::Infallible, core::convert::Infallible>;
    ///
    ///     fn max_output_len(&self, input_len: usize) -> Result<usize, qubit_codec::CapacityError> {
    ///         Ok(input_len)
    ///     }
    ///
    ///     fn reset(
    ///         &mut self,
    ///         output: &mut [u8],
    ///         output_index: usize,
    ///     ) -> Result<usize, TranscodeError<Self::Error>> {
    ///         TranscodeError::<Self::Error>::ensure_output_index(output.len(), output_index)?;
    ///         Ok(0)
    ///     }
    ///
    ///     fn transcode(
    ///         &mut self,
    ///         input: &[u8],
    ///         input_index: usize,
    ///         output: &mut [u8],
    ///         output_index: usize,
    ///     ) -> Result<qubit_codec::TranscodeProgress, TranscodeError<Self::Error>> {
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
    ///                 additional: NonZeroUsize::MIN,
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
    ///     ) -> Result<usize, TranscodeError<Self::Error>> {
    ///         TranscodeError::<Self::Error>::ensure_output_index(output.len(), output_index)?;
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
    ) -> Result<usize, TranscodeError<Self::Error>>;
}
