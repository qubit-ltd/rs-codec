/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Tests for the codec-backed buffered encoder adapter.

use qubit_codec::{
    BufferedEncoder,
    Codec,
    CodecBufferedEncoder,
    CodecEncodeError,
    TranscodeStatus,
    Transcoder,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct PairByteCodec;

unsafe impl Codec<u8, u8> for PairByteCodec {
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        unsafe { core::num::NonZeroUsize::new_unchecked(2) }
    }

    unsafe fn decode_unchecked(
        &self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index < input.len());

        // SAFETY: The caller guarantees that `index` is readable.
        let value = unsafe { *input.as_ptr().add(index) };
        Ok((value, core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode_unchecked(&self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Self::EncodeError> {
        debug_assert!(index + 2 <= output.len());

        // SAFETY: The caller guarantees that two bytes are writable from `index`.
        unsafe {
            *output.as_mut_ptr().add(index) = *value;
            *output.as_mut_ptr().add(index + 1) = value.wrapping_add(1);
        }
        Ok(2)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct RejectOddCodec;

unsafe impl Codec<u8, u8> for RejectOddCodec {
    type DecodeError = core::convert::Infallible;
    type EncodeError = &'static str;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    unsafe fn decode_unchecked(
        &self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index < input.len());

        // SAFETY: The caller guarantees that `index` is readable.
        let value = unsafe { *input.as_ptr().add(index) };
        Ok((value, core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode_unchecked(&self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Self::EncodeError> {
        if !value.is_multiple_of(2) {
            return Err("odd value");
        }
        debug_assert!(index < output.len());

        // SAFETY: The caller guarantees that `index` is writable.
        unsafe {
            *output.as_mut_ptr().add(index) = *value;
        }
        Ok(1)
    }
}

#[test]
fn test_codec_buffered_encoder_encodes_until_output_needs_more_capacity() {
    fn assert_buffered_encoder<T: BufferedEncoder<u8, u8>>() {}

    assert_buffered_encoder::<CodecBufferedEncoder<PairByteCodec>>();

    let mut encoder = CodecBufferedEncoder::new(PairByteCodec);
    let mut output = [0_u8; 4];

    let progress = encoder
        .transcode(&[3, 5, 7], 0, &mut output, 0)
        .expect("encoding should be infallible");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 4,
            additional: 2,
            available: 0,
        },
        progress.status(),
    );
    assert_eq!(2, progress.read());
    assert_eq!(4, progress.written());
    assert_eq!([3, 4, 5, 6], output);
    assert_eq!(Some(6), encoder.max_output_len(3));
    assert_eq!(Some(0), encoder.max_finish_output_len());
    assert_eq!(None, encoder.max_output_len(usize::MAX));
}

#[test]
fn test_codec_buffered_encoder_respects_absolute_indices() {
    let mut encoder = CodecBufferedEncoder::new(PairByteCodec);
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
fn test_codec_buffered_encoder_reports_partial_output_capacity() {
    let mut encoder = CodecBufferedEncoder::new(PairByteCodec);
    let mut output = [0_u8; 1];

    let progress = encoder
        .transcode(&[3], 0, &mut output, 0)
        .expect("encoding should stop before unsafe call");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 0,
            additional: 1,
            available: 1,
        },
        progress.status(),
    );
    assert_eq!(0, progress.read());
    assert_eq!(0, progress.written());
    assert_eq!([0], output);
}

#[test]
fn test_codec_buffered_encoder_exposes_wrapped_codec_accessors() {
    let mut encoder = CodecBufferedEncoder::new(PairByteCodec);
    let mut output = [0_u8; 1];

    assert_eq!(&PairByteCodec, encoder.codec());
    assert_eq!(&mut PairByteCodec, encoder.codec_mut());
    encoder.reset();
    let progress = encoder.finish(&mut output, 0).expect("finish is a no-op");
    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(0, progress.read());
    assert_eq!(0, progress.written());
    assert_eq!(PairByteCodec, encoder.into_codec());
}

#[test]
fn test_codec_buffered_encoder_finish_reports_output_index_beyond_buffer() {
    let mut encoder = CodecBufferedEncoder::new(PairByteCodec);
    let mut output = [];

    let progress = encoder
        .finish(&mut output, 1)
        .expect("out-of-range finish output index should request capacity");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 1,
            additional: 1,
            available: 0,
        },
        progress.status(),
    );
    assert_eq!(0, progress.read());
    assert_eq!(0, progress.written());
}

#[test]
fn test_codec_buffered_encoder_propagates_encode_error() {
    let mut encoder = CodecBufferedEncoder::new(RejectOddCodec);
    let mut output = [0_u8; 2];

    let error = encoder
        .transcode(&[2, 3], 0, &mut output, 0)
        .expect_err("odd value should be rejected");

    assert_eq!(
        CodecEncodeError::Encode {
            source: "odd value",
            input_index: 1,
        },
        error,
    );
    assert_eq!([2, 0], output);
}
