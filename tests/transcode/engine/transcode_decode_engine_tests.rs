// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the reusable buffered decoder engine.

use core::num::NonZeroUsize;

use qubit_codec::{
    Codec, DecodeAction, DecodeContext, TranscodeDecodeEngine, TranscodeDecodeHooks,
    TranscodeError, TranscodeStatus, Transcoder, nz,
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

    unsafe fn decode(
        &mut self,
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

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(index < output.len());

        // SAFETY: The caller guarantees that `index` is writable.
        unsafe {
            *output.as_mut_ptr().add(index) = *value;
        }
        Ok(qubit_io::nz!(1))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OverconsumingCodec;

unsafe impl Codec for OverconsumingCodec {
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

        Ok((input[index], unsafe {
            core::num::NonZeroUsize::new_unchecked(2)
        }))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(index < output.len());

        output[index] = *value;
        Ok(qubit_io::nz!(1))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
enum EngineError {}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ReplacingHooks;

impl TranscodeDecodeHooks<PrefixCodec> for ReplacingHooks {
    type Error = EngineError;
    fn handle_decode_error(
        &mut self,
        _codec: &mut PrefixCodec,
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
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct OverconsumingHooks;

impl TranscodeDecodeHooks<OverconsumingCodec> for OverconsumingHooks {
    type Error = EngineError;

    fn handle_decode_error(
        &mut self,
        _codec: &mut OverconsumingCodec,
        error: core::convert::Infallible,
        _context: DecodeContext,
    ) -> Result<DecodeAction<u8>, Self::Error> {
        match error {}
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct SkippingHooks;

impl TranscodeDecodeHooks<PrefixCodec> for SkippingHooks {
    type Error = EngineError;

    fn handle_decode_error(
        &mut self,
        _codec: &mut PrefixCodec,
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FinishHooks {
    pending_suffix: bool,
}

impl Default for FinishHooks {
    fn default() -> Self {
        Self {
            pending_suffix: true,
        }
    }
}

impl TranscodeDecodeHooks<PrefixCodec> for FinishHooks {
    type Error = EngineError;

    fn handle_decode_error(
        &mut self,
        _codec: &mut PrefixCodec,
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

    fn max_finish_output_len(&self, _codec: &PrefixCodec) -> usize {
        usize::from(self.pending_suffix)
    }

    fn finish(
        &mut self,
        _codec: &mut PrefixCodec,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        if !self.pending_suffix {
            return Ok(0);
        }

        output[output_index] = 0xee;
        self.pending_suffix = false;
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InvalidDecodeActionKind {
    NeedInput,
    Skip,
    Emit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct InvalidDecodeActionHooks {
    kind: InvalidDecodeActionKind,
}

impl TranscodeDecodeHooks<PrefixCodec> for InvalidDecodeActionHooks {
    type Error = EngineError;

    fn handle_decode_error(
        &mut self,
        _codec: &mut PrefixCodec,
        _error: PrefixDecodeError,
        context: DecodeContext,
    ) -> Result<DecodeAction<u8>, Self::Error> {
        match self.kind {
            InvalidDecodeActionKind::NeedInput => Ok(DecodeAction::NeedInput {
                required_total: context.available(),
            }),
            InvalidDecodeActionKind::Skip => Ok(DecodeAction::Skip {
                consumed: non_zero_consumed(context.available() + 1),
            }),
            InvalidDecodeActionKind::Emit => Ok(DecodeAction::Emit {
                value: 77,
                consumed: non_zero_consumed(context.available() + 1),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OverwritingFinishHooks;

impl TranscodeDecodeHooks<PrefixCodec> for OverwritingFinishHooks {
    type Error = EngineError;

    fn handle_decode_error(
        &mut self,
        _codec: &mut PrefixCodec,
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

    fn max_finish_output_len(&self, _codec: &PrefixCodec) -> usize {
        1
    }

    fn finish(
        &mut self,
        _codec: &mut PrefixCodec,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        output[output_index] = 0xee;
        output[output_index + 1] = 0xdd;
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OverreportingFinishHooks;

impl TranscodeDecodeHooks<PrefixCodec> for OverreportingFinishHooks {
    type Error = EngineError;

    fn handle_decode_error(
        &mut self,
        _codec: &mut PrefixCodec,
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

    fn max_finish_output_len(&self, _codec: &PrefixCodec) -> usize {
        1
    }

    fn finish(
        &mut self,
        _codec: &mut PrefixCodec,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        output[output_index] = 0xee;
        Ok(2)
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

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index + 1 < input.len());

        Ok((input[index].wrapping_add(input[index + 1]), unsafe {
            core::num::NonZeroUsize::new_unchecked(2)
        }))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(index < output.len());

        output[index] = *value;
        Ok(qubit_io::nz!(1))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OverflowFlushCodec;

unsafe impl Codec for OverflowFlushCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_decode_flush_values(&self) -> usize {
        usize::MAX
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        Ok((input[index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        output[index] = *value;
        Ok(qubit_io::nz!(1))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OverflowFinishHooks;

impl TranscodeDecodeHooks<OverflowFlushCodec> for OverflowFinishHooks {
    type Error = EngineError;

    fn max_finish_output_len(&self, _codec: &OverflowFlushCodec) -> usize {
        1
    }

    fn handle_decode_error(
        &mut self,
        _codec: &mut OverflowFlushCodec,
        error: core::convert::Infallible,
        _context: DecodeContext,
    ) -> Result<DecodeAction<u8>, Self::Error> {
        match error {}
    }
}

impl TranscodeDecodeHooks<MinTwoCodec> for ReplacingHooks {
    type Error = EngineError;

    fn handle_decode_error(
        &mut self,
        _codec: &mut MinTwoCodec,
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
}

#[test]
fn test_transcode_decode_engine_reports_finish_bound_overflow() {
    let mut decoder = TranscodeDecodeEngine::<_, _>::new(OverflowFlushCodec, OverflowFinishHooks);
    let mut output = [0_u8; 1];

    assert_eq!(
        Err(qubit_codec::CapacityError::OutputLengthOverflow),
        decoder.max_finish_output_len(),
    );

    let error = decoder
        .finish(&mut output, 0)
        .expect_err("finish should report capacity overflow before writing");
    assert_eq!(TranscodeError::OutputLengthOverflow, error);
}

#[test]
fn test_transcode_decode_engine_reports_finish_bounds() {
    let mut decoder = TranscodeDecodeEngine::<_, _>::new(PrefixCodec, ReplacingHooks);
    let mut output = [0_u8; 1];

    assert_eq!(Ok(3), decoder.max_output_len(3));
    assert_eq!(Ok(0), decoder.max_finish_output_len());

    decoder.reset(&mut [], 0).expect("reset");
    let written = decoder
        .finish(&mut output, 0)
        .expect("generic decoder finish is a no-op");
    assert_eq!(0, written);
}

#[test]
fn test_transcode_decode_engine_delegates_finish_to_hooks() {
    let mut decoder = TranscodeDecodeEngine::<_, _>::new(PrefixCodec, FinishHooks::default());
    let mut output = [0_u8; 1];

    assert_eq!(Ok(1), decoder.max_finish_output_len());

    let error = decoder
        .finish(&mut [], 0)
        .expect_err("finish should reject insufficient output before calling hooks");
    assert_eq!(
        TranscodeError::InsufficientOutput {
            output_index: 0,
            required: 1,
            available: 0,
        },
        error,
    );
    assert_eq!(Ok(1), decoder.max_finish_output_len());

    let written = decoder
        .finish(&mut output, 0)
        .expect("hook should write final output");
    assert_eq!(1, written);
    assert_eq!([0xee], output);
    assert_eq!(Ok(0), decoder.max_finish_output_len());
}

#[test]
fn test_transcode_decode_engine_finish_passes_full_output_to_hooks() {
    let mut decoder = TranscodeDecodeEngine::<_, _>::new(PrefixCodec, OverwritingFinishHooks);
    let mut output = [0_u8; 2];

    let written = decoder
        .finish(&mut output, 0)
        .expect("hook should receive the caller-provided output slice");

    assert_eq!(1, written);
    assert_eq!([0xee, 0xdd], output);
}

#[test]
#[should_panic(expected = "TranscodeDecodeEngine hook wrote beyond its finish bound")]
fn test_transcode_decode_engine_finish_panics_when_hook_overreports_bound() {
    let mut decoder = TranscodeDecodeEngine::<_, _>::new(PrefixCodec, OverreportingFinishHooks);
    let mut output = [0_u8; 2];

    let _ = decoder.finish(&mut output, 0);
}

#[test]
fn test_transcode_decode_engine_finish_reports_output_index_beyond_buffer() {
    let mut decoder = TranscodeDecodeEngine::<_, _>::new(PrefixCodec, FinishHooks::default());
    let mut output = [];

    let error = decoder
        .finish(&mut output, 1)
        .expect_err("out-of-range finish output index should be rejected");

    assert_eq!(
        TranscodeError::InvalidOutputIndex { index: 1, len: 0 },
        error,
    );
}

#[test]
fn test_transcode_decode_engine_default_finish_reports_output_index_beyond_buffer() {
    let mut decoder = TranscodeDecodeEngine::<_, _>::new(PrefixCodec, ReplacingHooks);
    let mut output = [];

    let error = decoder
        .finish(&mut output, 1)
        .expect_err("default finish should reject out-of-range output index");

    assert_eq!(
        TranscodeError::InvalidOutputIndex { index: 1, len: 0 },
        error,
    );
}

#[test]
fn test_buffered_decode_hooks_default_finish_is_noop() {
    let mut hooks = ReplacingHooks;
    let mut output = [];

    let written =
        TranscodeDecodeHooks::<PrefixCodec>::finish(&mut hooks, &mut PrefixCodec, &mut output, 1)
            .expect("default hook finish should be a no-op");

    assert_eq!(0, written);
}

#[test]
fn test_transcode_decode_engine_leaves_incomplete_input_to_caller() {
    let mut decoder = TranscodeDecodeEngine::new(PrefixCodec, ReplacingHooks);
    let mut output = [0_u8; 1];

    let progress = decoder
        .transcode(&[0xfe], 0, &mut output, 0)
        .expect("incomplete prefix should be reported");

    assert_eq!(
        TranscodeStatus::NeedInput {
            input_index: 0,
            additional: nz(1),
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
fn test_transcode_decode_engine_reports_short_minimum_input_without_consuming_tail() {
    let mut decoder = TranscodeDecodeEngine::new(MinTwoCodec, ReplacingHooks);
    let mut output = [0_u8; 1];

    let progress = decoder
        .transcode(&[7], 0, &mut output, 0)
        .expect("short input should request another unit");

    assert_eq!(
        TranscodeStatus::NeedInput {
            input_index: 0,
            additional: nz(1),
            available: 1,
        },
        progress.status(),
    );
    assert_eq!(0, progress.read());
    assert_eq!(0, progress.written());
}

#[test]
fn test_transcode_decode_engine_reports_incomplete_input_before_missing_output() {
    let mut decoder = TranscodeDecodeEngine::new(MinTwoCodec, ReplacingHooks);
    let mut output = [];

    let progress = decoder
        .transcode(&[7], 0, &mut output, 0)
        .expect("short input should request another unit before output capacity");

    assert_eq!(
        TranscodeStatus::NeedInput {
            input_index: 0,
            additional: nz(1),
            available: 1,
        },
        progress.status(),
    );
    assert_eq!(0, progress.read());
    assert_eq!(0, progress.written());
}

#[test]
fn test_transcode_decode_engine_allows_policy_emit_for_invalid_input() {
    let mut decoder = TranscodeDecodeEngine::new(PrefixCodec, ReplacingHooks);
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
fn test_transcode_decode_engine_reports_need_output_before_policy_emit() {
    let mut decoder = TranscodeDecodeEngine::new(PrefixCodec, ReplacingHooks);
    let mut output = [];

    let progress = decoder
        .transcode(&[0xff], 0, &mut output, 0)
        .expect("replacement policy should stop before writing without output");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 0,
            additional: nz(1),
            available: 0,
        },
        progress.status(),
    );
    assert_eq!(0, progress.read());
    assert_eq!(0, progress.written());
}

#[test]
fn test_transcode_decode_engine_allows_policy_skip_for_invalid_input() {
    let mut decoder = TranscodeDecodeEngine::new(PrefixCodec, SkippingHooks);
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
#[should_panic(expected = "DecodeAction::NeedInput required_total must exceed available input")]
fn test_transcode_decode_engine_panics_on_invalid_need_input_action() {
    let mut decoder = TranscodeDecodeEngine::new(
        PrefixCodec,
        InvalidDecodeActionHooks {
            kind: InvalidDecodeActionKind::NeedInput,
        },
    );
    let mut output = [0_u8; 1];

    let _ = decoder.transcode(&[0xfe], 0, &mut output, 0);
}

#[test]
#[should_panic(expected = "DecodeAction consumed units must not exceed available input")]
fn test_transcode_decode_engine_panics_on_invalid_skip_action() {
    let mut decoder = TranscodeDecodeEngine::new(
        PrefixCodec,
        InvalidDecodeActionHooks {
            kind: InvalidDecodeActionKind::Skip,
        },
    );
    let mut output = [0_u8; 1];

    let _ = decoder.transcode(&[0xff], 0, &mut output, 0);
}

#[test]
#[should_panic(expected = "DecodeAction consumed units must not exceed available input")]
fn test_transcode_decode_engine_panics_on_invalid_emit_action() {
    let mut decoder = TranscodeDecodeEngine::new(
        PrefixCodec,
        InvalidDecodeActionHooks {
            kind: InvalidDecodeActionKind::Emit,
        },
    );
    let mut output = [0_u8; 1];

    let _ = decoder.transcode(&[0xff], 0, &mut output, 0);
}

#[test]
fn test_transcode_decode_engine_reports_output_bounds_without_consuming_input() {
    let mut decoder = TranscodeDecodeEngine::new(PrefixCodec, ReplacingHooks);
    let mut output = [];

    let progress = decoder
        .transcode(&[1], 0, &mut output, 0)
        .expect("empty output should request capacity");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 0,
            additional: nz(1),
            available: 0,
        },
        progress.status(),
    );
    assert_eq!(0, progress.read());
    assert_eq!(0, progress.written());

    let error = decoder
        .transcode(&[1], 0, &mut output, 1)
        .expect_err("out-of-range output index should fail");

    assert_eq!(
        TranscodeError::InvalidOutputIndex { index: 1, len: 0 },
        error,
    );
}

#[test]
#[should_panic(expected = "Codec::decode consumed beyond available input")]
fn test_transcode_decode_engine_panics_when_codec_consumes_beyond_available_input() {
    let mut decoder = TranscodeDecodeEngine::new(OverconsumingCodec, OverconsumingHooks);
    let mut output = [0_u8; 1];

    let _ = decoder.transcode(&[1], 0, &mut output, 0);
}

#[test]
fn test_transcode_decode_engine_uses_hooks_for_invalid_input_index() {
    let mut decoder = TranscodeDecodeEngine::new(PrefixCodec, ReplacingHooks);
    let mut output = [];

    let error = decoder
        .transcode(&[1], 2, &mut output, 0)
        .expect_err("invalid input index should be rejected");

    assert_eq!(
        TranscodeError::InvalidInputIndex { index: 2, len: 1 },
        error,
    );
}

#[test]
fn test_transcode_decode_engine_implements_buffered_transcoder() {
    type Decoder = TranscodeDecodeEngine<PrefixCodec, ReplacingHooks>;
    let mut decoder = Decoder::new(PrefixCodec, ReplacingHooks);

    let available = <Decoder as Transcoder<
        <PrefixCodec as Codec>::Unit,
        <PrefixCodec as Codec>::Value,
    >>::max_output_len(&decoder, 1)
    .expect("max_output_len should be callable through trait");
    assert_eq!(1, available);

    let mut output = [0_u8; 1];
    let progress = <Decoder as Transcoder<
        <PrefixCodec as Codec>::Unit,
        <PrefixCodec as Codec>::Value,
    >>::transcode(&mut decoder, &[0xfe, 7], 0, &mut output, 0)
    .expect("trait transcode should decode a prefixed value");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(2, progress.read());
    assert_eq!(1, progress.written());

    let finish = Transcoder::finish(&mut decoder, &mut output, 0)
        .expect("trait finish should delegate to hooks");
    assert_eq!(0, finish);

    let finish_output_len = <Decoder as Transcoder<
        <PrefixCodec as Codec>::Unit,
        <PrefixCodec as Codec>::Value,
    >>::max_finish_output_len(&decoder)
    .expect("max_finish_output_len should be callable through trait");
    assert_eq!(0, finish_output_len);

    assert_eq!(7, output[0]);

    Transcoder::reset(&mut decoder, &mut output, 0).expect("reset");
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FlushFailCodec;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("flush failed")]
struct FlushFailError;

unsafe impl Codec for FlushFailCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = FlushFailError;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_decode_flush_values(&self) -> usize {
        1
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        Ok((input[index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        output[index] = *value;
        Ok(qubit_io::nz!(1))
    }

    unsafe fn decode_flush(
        &mut self,
        _output: &mut [u8],
        _index: usize,
    ) -> Result<usize, Self::DecodeError> {
        Err(FlushFailError)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct FlushMappingHooks;

impl TranscodeDecodeHooks<FlushFailCodec> for FlushMappingHooks {
    type Error = qubit_codec::CodecDecodeError<FlushFailError>;

    fn handle_decode_error(
        &mut self,
        _codec: &mut FlushFailCodec,
        error: FlushFailError,
        context: DecodeContext,
    ) -> Result<DecodeAction<u8>, Self::Error> {
        Err(qubit_codec::CodecDecodeError::decode(
            error,
            context.input_index(),
        ))
    }

    fn map_decode_flush_error(
        &mut self,
        _codec: &mut FlushFailCodec,
        error: FlushFailError,
    ) -> Self::Error {
        qubit_codec::CodecDecodeError::decode(error, 0)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetFailHooks;

impl TranscodeDecodeHooks<PrefixCodec> for ResetFailHooks {
    type Error = PrefixDecodeError;

    fn handle_decode_error(
        &mut self,
        _codec: &mut PrefixCodec,
        error: PrefixDecodeError,
        _context: DecodeContext,
    ) -> Result<DecodeAction<u8>, Self::Error> {
        Err(error)
    }

    fn reset(&mut self, _codec: &mut PrefixCodec) -> Result<(), Self::Error> {
        Err(PrefixDecodeError::Invalid { consumed: 0 })
    }
}

#[test]
fn test_transcode_decode_engine_reports_max_reset_output_len() {
    let decoder =
        TranscodeDecodeEngine::<PrefixCodec, ReplacingHooks>::new(PrefixCodec, ReplacingHooks);

    assert_eq!(0, decoder.max_reset_output_len());
    assert_eq!(Ok(0), Transcoder::max_reset_output_len(&decoder));
}

#[test]
fn test_transcode_decode_engine_reset_rejects_invalid_output_index() {
    let mut decoder =
        TranscodeDecodeEngine::<PrefixCodec, ReplacingHooks>::new(PrefixCodec, ReplacingHooks);

    let error = decoder
        .reset(&mut [], 1)
        .expect_err("reset should reject invalid output index");

    assert_eq!(
        TranscodeError::InvalidOutputIndex { index: 1, len: 0 },
        error,
    );
}

#[test]
fn test_transcode_decode_engine_reset_maps_hook_reset_errors() {
    let mut decoder = TranscodeDecodeEngine::<_, _>::new(PrefixCodec, ResetFailHooks);

    let error = decoder
        .reset(&mut [], 0)
        .expect_err("hook reset errors should be mapped through the engine");

    assert_eq!(
        TranscodeError::Domain(PrefixDecodeError::Invalid { consumed: 0 }),
        error,
    );
}

#[test]
fn test_transcode_decode_engine_finish_maps_decode_flush_errors() {
    let mut decoder = TranscodeDecodeEngine::<_, _>::new(FlushFailCodec, FlushMappingHooks);
    let mut output = [0_u8; 1];

    let error = decoder
        .finish(&mut output, 0)
        .expect_err("flush errors should be mapped through hooks");

    assert_eq!(
        TranscodeError::Domain(qubit_codec::CodecDecodeError::Decode {
            source: FlushFailError,
            input_index: 0,
        }),
        error,
    );
}
