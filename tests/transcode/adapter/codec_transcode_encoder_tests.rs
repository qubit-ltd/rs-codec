// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the codec-backed buffered encoder adapter.

use qubit_codec::{
    CapacityError, Codec, CodecEncodeError, CodecTranscodeEncoder, TranscodeEncoder,
    TranscodeError, TranscodeStatus, Transcoder,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct PairByteCodec;

unsafe impl Codec for PairByteCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        unsafe { core::num::NonZeroUsize::new_unchecked(2) }
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index < input.len());

        // SAFETY: The caller guarantees that `index` is readable.
        let value = unsafe { *input.as_ptr().add(index) };
        Ok((value, core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(index + 2 <= output.len());

        // SAFETY: The caller guarantees that two bytes are writable from
        // `index`.
        unsafe {
            *output.as_mut_ptr().add(index) = *value;
            *output.as_mut_ptr().add(index + 1) = value.wrapping_add(1);
        }
        Ok(qubit_codec::nz!(2))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct VariableWidthCodec;

unsafe impl Codec for VariableWidthCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        qubit_codec::nz!(3)
    }

    fn encode_len(&self, value: &u8) -> core::num::NonZeroUsize {
        match *value {
            0..=9 => qubit_codec::nz!(1),
            10..=99 => qubit_codec::nz!(2),
            _ => qubit_codec::nz!(3),
        }
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index < input.len());

        // SAFETY: The caller guarantees that `index` is readable.
        let value = unsafe { *input.as_ptr().add(index) };
        Ok((value, core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        let written = self.encode_len(value);
        debug_assert!(index + written.get() <= output.len());

        for offset in 0..written.get() {
            // SAFETY: The caller guarantees that `written` units are writable
            // from `index`.
            unsafe {
                *output.as_mut_ptr().add(index + offset) = *value;
            }
        }
        Ok(written)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct RejectOddCodec;

unsafe impl Codec for RejectOddCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = &'static str;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn can_encode_value(&self, value: &u8) -> bool {
        value.is_multiple_of(2)
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index < input.len());

        // SAFETY: The caller guarantees that `index` is readable.
        let value = unsafe { *input.as_ptr().add(index) };
        Ok((value, core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(self.can_encode_value(value));
        debug_assert!(index < output.len());

        // SAFETY: The caller guarantees that `index` is writable.
        unsafe {
            *output.as_mut_ptr().add(index) = *value;
        }
        Ok(qubit_codec::nz!(1))
    }
}

#[test]
fn test_codec_transcode_encoder_encodes_until_output_needs_more_capacity() {
    fn assert_transcode_encoder<T: TranscodeEncoder<u8, u8>>() {}

    assert_transcode_encoder::<CodecTranscodeEncoder<PairByteCodec>>();

    let mut encoder = CodecTranscodeEncoder::new(PairByteCodec);
    let mut output = [0_u8; 4];

    let progress = encoder
        .transcode(&[3, 5, 7], 0, &mut output, 0)
        .expect("encoding should be infallible");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 4,
            additional: crate::nz(2),
            available: 0,
        },
        progress.status(),
    );
    assert_eq!(2, progress.read());
    assert_eq!(4, progress.written());
    assert_eq!([3, 4, 5, 6], output);
    assert_eq!(Ok(6), encoder.max_output_len(3));
    assert_eq!(Ok(0), encoder.max_finish_output_len());
    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        encoder.max_output_len(usize::MAX),
    );
    encoder.reset(&mut [], 0).expect("reset");
}

#[test]
fn test_codec_transcode_encoder_respects_absolute_indices() {
    let mut encoder = CodecTranscodeEncoder::new(PairByteCodec);
    let mut output = [0_u8; 4];

    let progress = encoder
        .transcode(&[3, 5], 1, &mut output, 1)
        .expect("encoding should be infallible");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(1, progress.read());
    assert_eq!(2, progress.written());
    assert_eq!([0, 5, 6, 0], output);
}

#[test]
fn test_codec_transcode_encoder_reports_partial_output_capacity() {
    let mut encoder = CodecTranscodeEncoder::new(PairByteCodec);
    let mut output = [0_u8; 1];

    let progress = encoder
        .transcode(&[3], 0, &mut output, 0)
        .expect("encoding should stop before unsafe call");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 0,
            additional: crate::nz(1),
            available: 1,
        },
        progress.status(),
    );
    assert_eq!(0, progress.read());
    assert_eq!(0, progress.written());
    assert_eq!([0], output);
}

#[test]
fn test_codec_transcode_encoder_uses_encode_len_for_output_capacity() {
    let mut encoder = CodecTranscodeEncoder::new(VariableWidthCodec);
    let mut output = [0_u8; 2];

    let progress = encoder
        .transcode(&[7, 8], 0, &mut output, 0)
        .expect("short values should fit exactly");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(2, progress.read());
    assert_eq!(2, progress.written());
    assert_eq!([7, 8], output);
}

#[test]
fn test_codec_transcode_encoder_reports_output_index_beyond_buffer() {
    let mut encoder = CodecTranscodeEncoder::new(PairByteCodec);
    let mut output = [];

    let error = encoder
        .transcode(&[3], 0, &mut output, 1)
        .expect_err("out-of-range output index should fail");

    assert_eq!(
        TranscodeError::InvalidOutputIndex { index: 1, len: 0 },
        error
    );
}

#[test]
fn test_codec_transcode_encoder_finish_reports_output_index_beyond_buffer() {
    let mut encoder = CodecTranscodeEncoder::new(PairByteCodec);
    let mut output = [];

    let error = encoder
        .finish(&mut output, 1)
        .expect_err("out-of-range finish output index should be rejected");

    assert_eq!(
        TranscodeError::InvalidOutputIndex { index: 1, len: 0 },
        error
    );
}

#[test]
fn test_codec_transcode_encoder_reports_invalid_input_index() {
    let mut encoder = CodecTranscodeEncoder::new(PairByteCodec);
    let mut output = [];

    let error = encoder
        .transcode(&[3], 2, &mut output, 0)
        .expect_err("invalid input index should fail");

    assert_eq!(
        TranscodeError::InvalidInputIndex { index: 2, len: 1 },
        error
    );
}

#[test]
fn test_codec_transcode_encoder_propagates_encode_error() {
    let mut encoder = CodecTranscodeEncoder::new(RejectOddCodec);
    let mut output = [0_u8; 2];

    let error = encoder
        .transcode(&[2, 3], 0, &mut output, 0)
        .expect_err("odd value should be rejected before unsafe encode");

    assert_eq!(
        TranscodeError::Domain(CodecEncodeError::UnencodableValue { input_index: 1 }),
        error,
    );
    assert_eq!([2, 0], output);
}

#[test]
fn test_codec_transcode_encoder_reports_max_reset_output_len() {
    let encoder = CodecTranscodeEncoder::<PairByteCodec>::new(PairByteCodec);

    assert_eq!(Ok(0), Transcoder::max_reset_output_len(&encoder));
}

#[test]
fn test_codec_transcode_encoder_default_builds_encoder() {
    let mut encoder = CodecTranscodeEncoder::<PairByteCodec>::default();
    let mut output = [0_u8; 2];

    let progress = encoder
        .transcode(&[7], 0, &mut output, 0)
        .expect("default encoder should transcode one value");

    assert_eq!(1, progress.read());
    assert_eq!(2, progress.written());
    assert_eq!([7, 8], output);
}
