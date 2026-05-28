/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Buffered encoder adapter backed by a low-level codec.

use super::{
    BufferedEncoder,
    TranscodeProgress,
    TranscodeStatus,
    Transcoder,
};
use crate::Codec;

/// Encodes values into caller-provided output units by using a [`Codec`].
///
/// `CodecBufferedEncoder` is the default bridge from the low-level unchecked
/// [`Codec`] contract to the buffered [`Transcoder`] and [`BufferedEncoder`]
/// contracts. It encodes complete values only; when the remaining output
/// capacity is smaller than `codec.max_units_per_value()`, it stops before
/// consuming the next input value and reports [`TranscodeStatus::NeedOutput`].
///
/// # Type Parameters
///
/// - `C`: Low-level codec used to encode values.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct CodecBufferedEncoder<C> {
    /// Low-level codec used for one-value encoding.
    codec: C,
}

impl<C> CodecBufferedEncoder<C> {
    /// Creates a buffered encoder backed by `codec`.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec used to encode values.
    ///
    /// # Returns
    ///
    /// Returns a buffered encoder adapter for the supplied codec.
    #[must_use]
    #[inline]
    pub const fn new(codec: C) -> Self {
        Self { codec }
    }

    /// Returns the wrapped codec.
    ///
    /// # Returns
    ///
    /// Returns a shared reference to the wrapped low-level codec.
    #[must_use]
    #[inline]
    pub const fn codec(&self) -> &C {
        &self.codec
    }

    /// Returns a mutable reference to the wrapped codec.
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to the wrapped low-level codec.
    #[must_use]
    #[inline]
    pub fn codec_mut(&mut self) -> &mut C {
        &mut self.codec
    }

    /// Consumes the adapter and returns the wrapped codec.
    ///
    /// # Returns
    ///
    /// Returns the codec supplied at construction time.
    #[must_use]
    #[inline]
    pub fn into_codec(self) -> C {
        self.codec
    }
}

impl<C, Value, Unit> Transcoder<Value, Unit> for CodecBufferedEncoder<C>
where
    C: Codec<Value, Unit>,
    Unit: Copy,
{
    type Error = C::EncodeError;

    /// Returns the maximum number of output units needed for `input_len` values.
    fn max_output_len(&self, input_len: usize) -> Option<usize> {
        input_len.checked_mul(self.codec.max_units_per_value())
    }

    /// Encodes values into the supplied output buffer.
    fn transcode(
        &mut self,
        input: &[Value],
        input_index: usize,
        output: &mut [Unit],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        let max_units = self.codec.max_units_per_value();
        let required_units = max_units.max(1);
        let mut read = 0;
        let mut written = 0;

        while input_index + read < input.len() {
            let output_position = output_index.saturating_add(written);
            let available = output.len().saturating_sub(output_position);
            if available < max_units {
                let status = TranscodeStatus::NeedOutput {
                    output_index: output_position,
                    required: required_units.saturating_sub(available),
                    available,
                };
                return Ok(TranscodeProgress::new(status, read, written));
            }

            // SAFETY: The remaining output capacity is at least the codec's
            // declared maximum width for one encoded value.
            let produced = unsafe {
                self.codec
                    .encode_unchecked(&input[input_index + read], output, output_position)
            }?;
            read += 1;
            written += produced;
        }

        Ok(TranscodeProgress::complete(read, written))
    }
}

impl<C, Value, Unit> BufferedEncoder<Value, Unit> for CodecBufferedEncoder<C>
where
    C: Codec<Value, Unit>,
    Unit: Copy,
{
}
