/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
use super::transcode_progress::TranscodeProgress;

/// Converts one logical stream of input units into one logical stream of output units.
///
/// `transcode` is the main streaming API. It transforms a provided input segment and
/// writes as much output as available buffer space allows, without automatically
/// finalizing internal pending state.
///
/// A transcoder instance has a simple lifecycle:
///
/// 1. A newly created or reset instance is ready for a new logical stream.
/// 2. Call [`Transcoder::transcode`] zero or more times while input is available.
/// 3. Call [`Transcoder::finish`] after the caller knows no more input remains.
/// 4. Continue calling [`Transcoder::finish`] while it reports
///    [`crate::TranscodeStatus::NeedOutput`].
/// 5. After [`Transcoder::finish`] reports [`crate::TranscodeStatus::Complete`],
///    call [`Transcoder::reset`] before starting another logical stream with the
///    same instance.
///
/// The method is suitable for:
/// - pull-style consumers that call conversion repeatedly as buffers arrive;
/// - bounded output sinks that need `NeedOutput` progress when capacity is hit;
/// - stateless and stateful codecs that all return progress-oriented stopping
///   reasons.
///
/// `Transcoder` is intentionally independent from any charset semantics:
///
/// - Use `Transcoder` directly for custom, policy-free unit transforms.
/// - Use `Transcoder` when you want to own malformed/unmappable decisions at the call site.
///
/// # Example: streaming byte-to-word decoder
///
/// ```rust
/// use qubit_codec::{Transcoder, TranscodeProgress, TranscodeStatus};
///
/// #[derive(Default)]
/// struct U16BeBytesDecoder;
///
/// impl Transcoder<u8, u16> for U16BeBytesDecoder {
///     type Error = core::convert::Infallible;
///
///     fn max_output_len(&self, input_len: usize) -> Option<usize> {
///         Some(input_len / 2)
///     }
///
///     fn transcode(
///         &mut self,
///         input: &[u8],
///         input_index: usize,
///         output: &mut [u16],
///         output_index: usize,
///     ) -> Result<TranscodeProgress, Self::Error> {
///         let mut read = 0;
///         let mut written = 0;
///         while input_index + read + 1 < input.len() {
///             if output_index + written == output.len() {
///                 let status = TranscodeStatus::NeedOutput {
///                     output_index: output_index + written,
///                     required: 1,
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
///             let status = TranscodeStatus::NeedInput {
///                 input_index: input_index + read,
///                 required: 2,
///                 available: input.len() - (input_index + read),
///             };
///             Ok(TranscodeProgress::new(status, read, written))
///         }
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
///     required: 1,
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
///     required: 2,
///     available: 1,
/// }, progress.status());
/// assert_eq!(2, progress.read());
/// assert_eq!(1, progress.written());
/// assert_eq!([0x1234, 0], output);
/// ```
///
/// The trait is intentionally independent from charset concepts. Implementors
/// use `input_index` and `output_index` as absolute positions in the supplied
/// slices. Returned progress counters are relative counts from those positions.
/// For raw codecs this gives a compact API; higher-level workflows can wrap this
/// trait with their own semantic policies.
///
/// # Type Parameters
///
/// - `Input`: Input unit type accepted by this transcoder.
/// - `Output`: Output unit type produced by this transcoder.
pub trait Transcoder<Input, Output> {
    /// Error reported for semantic conversion failures.
    type Error;

    /// Returns an upper bound for output units produced from `input_len` units.
    ///
    /// # Parameters
    ///
    /// - `input_len`: Number of input units the caller plans to transcode.
    ///
    /// # Returns
    ///
    /// Returns `Some(bound)` when the transcoder can provide a finite upper bound.
    /// Returns `None` when the bound is not known.
    #[must_use]
    fn max_output_len(&self, input_len: usize) -> Option<usize>;

    /// Returns an upper bound for output units produced by finalization.
    ///
    /// This bound is evaluated against the transcoder's current state. It does
    /// not include output that may be produced by future [`Transcoder::transcode`]
    /// calls. Use it before [`Transcoder::finish`] when the caller wants to size
    /// a flush buffer for the already supplied input.
    ///
    /// # Returns
    ///
    /// Returns `Some(bound)` when the transcoder can provide a finite upper bound
    /// for finalization output. Returns `None` when the bound is not known.
    /// Stateless transcoders default to `Some(0)`.
    #[must_use]
    #[inline]
    fn max_finish_output_len(&self) -> Option<usize> {
        Some(0)
    }

    /// Resets state retained between conversion calls.
    ///
    /// This starts a new logical stream while keeping configuration such as
    /// byte order, charset policy, replacement values, and cryptographic keys.
    /// Pending input, pending output, and completed-stream state must be
    /// discarded by stateful implementations. Stateless transcoders may keep
    /// the default no-op implementation.
    #[inline]
    fn reset(&mut self) {}

    /// Converts available input units into output units.
    ///
    /// This method processes an input segment without closing the logical input
    /// stream. When the current segment ends in a partial value, a transcoder may
    /// either keep enough internal state to continue later or report
    /// [`crate::TranscodeStatus::NeedInput`]. Callers that have reached EOF must
    /// call [`Transcoder::finish`] so the transcoder can either flush, replace,
    /// ignore, or reject pending incomplete state according to its policy.
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
    /// Returns progress describing how many units were consumed and produced and
    /// why conversion stopped.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` for semantic conversion failures that the transcoder's
    /// policy does not absorb.
    fn transcode(
        &mut self,
        input: &[Input],
        input_index: usize,
        output: &mut [Output],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error>;

    /// Finalizes the current logical stream after all input has been supplied.
    ///
    /// `transcode` handles ordinary input consumption. `finish` is called only
    /// after the caller knows no more input remains. It is responsible for
    /// flushing buffered output, validating pending incomplete input, and
    /// emitting any stream trailer required by the concrete transcoder. If the
    /// provided output buffer is too small, `finish` returns
    /// [`crate::TranscodeStatus::NeedOutput`] and may be called again with more
    /// output capacity.
    ///
    /// After `finish` returns [`crate::TranscodeStatus::Complete`], the logical
    /// stream is closed. Portable callers should call [`Transcoder::reset`]
    /// before passing input for another logical stream to the same instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// use qubit_codec::{Transcoder, TranscodeStatus};
    ///
    /// #[derive(Default)]
    /// struct ByteCopy;
    ///
    /// impl Transcoder<u8, u8> for ByteCopy {
    ///     type Error = core::convert::Infallible;
    ///
    ///     fn max_output_len(&self, input_len: usize) -> Option<usize> {
    ///         Some(input_len)
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
    ///                 required: 1,
    ///                 available: output.len().saturating_sub(output_index + written),
    ///             };
    ///             Ok(qubit_codec::TranscodeProgress::new(
    ///                 status,
    ///                 read,
    ///                 written,
    ///             ))
    ///         }
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
    /// let finish = transcoder
    ///     .finish(&mut output, 1)
    ///     .expect("finish does not emit buffered state for no-op transcoders");
    /// assert_eq!(TranscodeStatus::Complete, finish.status());
    /// ```
    ///
    /// # Parameters
    ///
    /// - `output`: Complete output unit slice visible to the transcoder.
    /// - `output_index`: Absolute output unit index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns progress for units written during finalization. The `read` counter
    /// is normally zero because no new input is supplied to `finish`. Stateless
    /// transcoders return a completed progress value with zero counters.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` if pending state cannot be flushed according to the
    /// transcoder's policy.
    #[inline]
    fn finish(&mut self, _output: &mut [Output], _output_index: usize) -> Result<TranscodeProgress, Self::Error> {
        Ok(TranscodeProgress::complete(0, 0))
    }
}
