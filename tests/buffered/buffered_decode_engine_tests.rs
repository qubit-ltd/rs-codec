/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Tests for the reusable buffered decoder engine.

use core::num::NonZeroUsize;

use qubit_codec::{
    BufferedDecodeEngine,
    BufferedDecodeHooks,
    Codec,
    DecodeAction,
    DecodeContext,
    TranscodeStatus,
};

fn non_zero_consumed(consumed: usize) -> NonZeroUsize {
    NonZeroUsize::new(consumed).expect("decode policy must consume at least one source unit")
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct PrefixCodec;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PrefixDecodeError {
    Incomplete { required: usize, available: usize },
    Invalid { consumed: usize },
}

unsafe impl Codec for PrefixCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = PrefixDecodeError;
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
        let first = unsafe { *input.as_ptr().add(index) };
        match first {
            0xfe if input.len() - index < 2 => Err(PrefixDecodeError::Incomplete {
                required: 2,
                available: input.len() - index,
            }),
            0xfe => {
                // SAFETY: The branch above ensures the second byte is readable.
                let value = unsafe { *input.as_ptr().add(index + 1) };
                Ok((value, unsafe { core::num::NonZeroUsize::new_unchecked(2) }))
            }
            0xff => Err(PrefixDecodeError::Invalid { consumed: 1 }),
            value => Ok((value, core::num::NonZeroUsize::MIN)),
        }
    }

    unsafe fn encode_unchecked(&self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Self::EncodeError> {
        debug_assert!(index < output.len());

        // SAFETY: The caller guarantees that `index` is writable.
        unsafe {
            *output.as_mut_ptr().add(index) = *value;
        }
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EngineError {
    InvalidInputIndex { index: usize, input_len: usize },
}

impl EngineError {
    fn invalid_input_index(index: usize, input_len: usize) -> Self {
        Self::InvalidInputIndex { index, input_len }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ReplacingHooks;

impl BufferedDecodeHooks<PrefixCodec> for ReplacingHooks {
    type Error = EngineError;

    fn handle_decode_error(
        &mut self,
        _codec: &PrefixCodec,
        error: PrefixDecodeError,
        _context: DecodeContext,
    ) -> Result<DecodeAction<u8>, Self::Error> {
        match error {
            PrefixDecodeError::Incomplete { required, .. } => Ok(DecodeAction::NeedInput {
                required_total: required,
            }),
            PrefixDecodeError::Invalid { consumed } => Ok(DecodeAction::Emit {
                value: 99,
                consumed: non_zero_consumed(consumed),
            }),
        }
    }

    fn invalid_input_index(&mut self, _codec: &PrefixCodec, index: usize, input_len: usize) -> Self::Error {
        EngineError::invalid_input_index(index, input_len)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct SkippingHooks;

impl BufferedDecodeHooks<PrefixCodec> for SkippingHooks {
    type Error = EngineError;

    fn handle_decode_error(
        &mut self,
        _codec: &PrefixCodec,
        error: PrefixDecodeError,
        _context: DecodeContext,
    ) -> Result<DecodeAction<u8>, Self::Error> {
        match error {
            PrefixDecodeError::Incomplete { required, .. } => Ok(DecodeAction::NeedInput {
                required_total: required,
            }),
            PrefixDecodeError::Invalid { consumed } => Ok(DecodeAction::Skip {
                consumed: non_zero_consumed(consumed),
            }),
        }
    }

    fn invalid_input_index(&mut self, _codec: &PrefixCodec, index: usize, input_len: usize) -> Self::Error {
        EngineError::invalid_input_index(index, input_len)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FinishHooks {
    pending_suffix: bool,
}

impl Default for FinishHooks {
    fn default() -> Self {
        Self { pending_suffix: true }
    }
}

impl BufferedDecodeHooks<PrefixCodec> for FinishHooks {
    type Error = EngineError;

    fn handle_decode_error(
        &mut self,
        _codec: &PrefixCodec,
        error: PrefixDecodeError,
        _context: DecodeContext,
    ) -> Result<DecodeAction<u8>, Self::Error> {
        match error {
            PrefixDecodeError::Incomplete { required, .. } => Ok(DecodeAction::NeedInput {
                required_total: required,
            }),
            PrefixDecodeError::Invalid { consumed } => Ok(DecodeAction::Skip {
                consumed: non_zero_consumed(consumed),
            }),
        }
    }

    fn invalid_input_index(&mut self, _codec: &PrefixCodec, index: usize, input_len: usize) -> Self::Error {
        EngineError::invalid_input_index(index, input_len)
    }

    fn max_finish_output_len(&self, _codec: &PrefixCodec) -> usize {
        usize::from(self.pending_suffix)
    }

    fn finish(
        &mut self,
        _codec: &PrefixCodec,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<qubit_codec::TranscodeProgress, Self::Error> {
        if !self.pending_suffix {
            return Ok(qubit_codec::TranscodeProgress::complete(0, 0));
        }

        let available = output.len().saturating_sub(output_index);
        if available == 0 {
            let status = TranscodeStatus::NeedOutput {
                output_index,
                additional: super::nz(1),
                available,
            };
            return Ok(qubit_codec::TranscodeProgress::new(status, 0, 0));
        }

        output[output_index] = 0xee;
        self.pending_suffix = false;
        Ok(qubit_codec::TranscodeProgress::complete(0, 1))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct MinTwoCodec;

unsafe impl Codec for MinTwoCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = PrefixDecodeError;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::new(2).expect("literal is non-zero")
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        unsafe { core::num::NonZeroUsize::new_unchecked(2) }
    }

    unsafe fn decode_unchecked(
        &self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index + 1 < input.len());

        Ok((input[index].wrapping_add(input[index + 1]), unsafe {
            core::num::NonZeroUsize::new_unchecked(2)
        }))
    }

    unsafe fn encode_unchecked(&self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Self::EncodeError> {
        debug_assert!(index < output.len());

        output[index] = *value;
        Ok(1)
    }
}

impl BufferedDecodeHooks<MinTwoCodec> for ReplacingHooks {
    type Error = EngineError;

    fn handle_decode_error(
        &mut self,
        _codec: &MinTwoCodec,
        error: PrefixDecodeError,
        _context: DecodeContext,
    ) -> Result<DecodeAction<u8>, Self::Error> {
        match error {
            PrefixDecodeError::Incomplete { required, .. } => Ok(DecodeAction::NeedInput {
                required_total: required,
            }),
            PrefixDecodeError::Invalid { consumed } => Ok(DecodeAction::Emit {
                value: 99,
                consumed: non_zero_consumed(consumed),
            }),
        }
    }

    fn invalid_input_index(&mut self, _codec: &MinTwoCodec, index: usize, input_len: usize) -> Self::Error {
        EngineError::invalid_input_index(index, input_len)
    }
}

#[test]
fn test_buffered_decode_engine_reports_finish_bounds() {
    let mut decoder = BufferedDecodeEngine::<_, _>::new(PrefixCodec, ReplacingHooks);
    let mut output = [0_u8; 1];

    assert_eq!(Ok(3), decoder.max_output_len(3));
    assert_eq!(0, decoder.max_finish_output_len());

    decoder.reset();
    let progress = decoder
        .finish(&mut output, 0)
        .expect("generic decoder finish is a no-op");
    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(0, progress.read());
    assert_eq!(0, progress.written());
}

#[test]
fn test_buffered_decode_engine_delegates_finish_to_hooks() {
    let mut decoder = BufferedDecodeEngine::<_, _>::new(PrefixCodec, FinishHooks::default());
    let mut output = [0_u8; 1];

    assert_eq!(1, decoder.max_finish_output_len());

    let progress = decoder
        .finish(&mut [], 0)
        .expect("hook should request output for pending finish output");
    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 0,
            additional: super::nz(1),
            available: 0,
        },
        progress.status(),
    );
    assert_eq!(1, decoder.max_finish_output_len());

    let progress = decoder.finish(&mut output, 0).expect("hook should write final output");
    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(0, progress.read());
    assert_eq!(1, progress.written());
    assert_eq!([0xee], output);
    assert_eq!(0, decoder.max_finish_output_len());
}

#[test]
fn test_buffered_decode_engine_finish_reports_output_index_beyond_buffer() {
    let mut decoder = BufferedDecodeEngine::<_, _>::new(PrefixCodec, FinishHooks::default());
    let mut output = [];

    let progress = decoder
        .finish(&mut output, 1)
        .expect("out-of-range finish output index should request capacity");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 1,
            additional: super::nz(1),
            available: 0,
        },
        progress.status(),
    );
    assert_eq!(0, progress.read());
    assert_eq!(0, progress.written());
}

#[test]
fn test_buffered_decode_engine_default_finish_reports_output_index_beyond_buffer() {
    let mut decoder = BufferedDecodeEngine::<_, _>::new(PrefixCodec, ReplacingHooks);
    let mut output = [];

    let progress = decoder
        .finish(&mut output, 1)
        .expect("default finish should report out-of-range output index");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 1,
            additional: super::nz(1),
            available: 0,
        },
        progress.status(),
    );
}

#[test]
fn test_buffered_decode_hooks_default_finish_reports_output_index_beyond_buffer() {
    let mut hooks = ReplacingHooks;
    let mut output = [];

    let progress = BufferedDecodeHooks::<PrefixCodec>::finish(&mut hooks, &PrefixCodec, &mut output, 1)
        .expect("default hook finish should report out-of-range output index");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 1,
            additional: super::nz(1),
            available: 0,
        },
        progress.status(),
    );
}

#[test]
fn test_buffered_decode_engine_leaves_incomplete_input_to_caller() {
    let mut decoder = BufferedDecodeEngine::new(PrefixCodec, ReplacingHooks);
    let mut output = [0_u8; 1];

    let progress = decoder
        .transcode(&[0xfe], 0, &mut output, 0)
        .expect("incomplete prefix should be reported");

    assert_eq!(
        TranscodeStatus::NeedInput {
            input_index: 0,
            additional: super::nz(1),
            available: 1,
        },
        progress.status(),
    );
    assert_eq!(0, progress.read());
    assert_eq!(0, progress.written());

    let progress = decoder
        .transcode(&[0xfe, 7], 0, &mut output, 0)
        .expect("caller-refilled prefix should decode");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(2, progress.read());
    assert_eq!(1, progress.written());
    assert_eq!([7], output);
}

#[test]
fn test_buffered_decode_engine_reports_short_minimum_input_without_consuming_tail() {
    let mut decoder = BufferedDecodeEngine::new(MinTwoCodec, ReplacingHooks);
    let mut output = [0_u8; 1];

    let progress = decoder
        .transcode(&[7], 0, &mut output, 0)
        .expect("short input should request another unit");

    assert_eq!(
        TranscodeStatus::NeedInput {
            input_index: 0,
            additional: super::nz(1),
            available: 1,
        },
        progress.status(),
    );
    assert_eq!(0, progress.read());
    assert_eq!(0, progress.written());
}

#[test]
fn test_buffered_decode_engine_reports_incomplete_input_before_missing_output() {
    let mut decoder = BufferedDecodeEngine::new(MinTwoCodec, ReplacingHooks);
    let mut output = [];

    let progress = decoder
        .transcode(&[7], 0, &mut output, 0)
        .expect("short input should request another unit before output capacity");

    assert_eq!(
        TranscodeStatus::NeedInput {
            input_index: 0,
            additional: super::nz(1),
            available: 1,
        },
        progress.status(),
    );
    assert_eq!(0, progress.read());
    assert_eq!(0, progress.written());
}

#[test]
fn test_buffered_decode_engine_allows_policy_emit_for_invalid_input() {
    let mut decoder = BufferedDecodeEngine::new(PrefixCodec, ReplacingHooks);
    let mut output = [0_u8; 2];

    let progress = decoder
        .transcode(&[0xff, 1], 0, &mut output, 0)
        .expect("invalid input should be replaced");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(2, progress.read());
    assert_eq!(2, progress.written());
    assert_eq!([99, 1], output);
}

#[test]
fn test_buffered_decode_engine_reports_need_output_before_policy_emit() {
    let mut decoder = BufferedDecodeEngine::new(PrefixCodec, ReplacingHooks);
    let mut output = [];

    let progress = decoder
        .transcode(&[0xff], 0, &mut output, 0)
        .expect("replacement policy should stop before writing without output");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 0,
            additional: super::nz(1),
            available: 0,
        },
        progress.status(),
    );
    assert_eq!(0, progress.read());
    assert_eq!(0, progress.written());
}

#[test]
fn test_buffered_decode_engine_allows_policy_skip_for_invalid_input() {
    let mut decoder = BufferedDecodeEngine::new(PrefixCodec, SkippingHooks);
    let mut output = [0_u8; 1];

    let progress = decoder
        .transcode(&[0xff, 1], 0, &mut output, 0)
        .expect("invalid input should be skipped");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(2, progress.read());
    assert_eq!(1, progress.written());
    assert_eq!([1], output);
}

#[test]
fn test_buffered_decode_engine_reports_output_bounds_without_consuming_input() {
    let mut decoder = BufferedDecodeEngine::new(PrefixCodec, ReplacingHooks);
    let mut output = [];

    let progress = decoder
        .transcode(&[1], 0, &mut output, 0)
        .expect("empty output should request capacity");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 0,
            additional: super::nz(1),
            available: 0,
        },
        progress.status(),
    );
    assert_eq!(0, progress.read());
    assert_eq!(0, progress.written());

    let progress = decoder
        .transcode(&[1], 0, &mut output, 1)
        .expect("out-of-range output index should request capacity");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 1,
            additional: super::nz(1),
            available: 0,
        },
        progress.status(),
    );
}

#[test]
fn test_buffered_decode_engine_uses_hooks_for_invalid_input_index() {
    let mut decoder = BufferedDecodeEngine::new(PrefixCodec, ReplacingHooks);
    let mut output = [];

    let error = decoder
        .transcode(&[1], 2, &mut output, 0)
        .expect_err("invalid input index should be rejected");

    assert_eq!(EngineError::InvalidInputIndex { index: 2, input_len: 1 }, error,);
}
