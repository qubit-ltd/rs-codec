// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the reusable buffered decoder engine.

use core::{
    cell::Cell,
    num::NonZeroUsize,
};

use qubit_codec::{
    Codec,
    CodecDecodeError,
    DecodeContext,
    DecodeInvalidAction,
    TranscodeDecodeEngine,
    TranscodeDecodeEngineError,
    TranscodeDecodeHooks,
    TranscodeError,
    TranscodeStatus,
    Transcoder,
};

fn non_zero_consumed(consumed: usize) -> NonZeroUsize {
    NonZeroUsize::new(consumed)
        .expect("decode policy must consume at least one source unit")
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct PrefixCodec;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PrefixDecodeError {
    Invalid { consumed: usize },
}

impl Codec for PrefixCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = PrefixDecodeError;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize = qubit_io::nz!(2);

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::DecodeFailure<Self::DecodeError>,
    > {
        debug_assert!(input_index < input.len());

        // SAFETY: The caller guarantees that `input_index` is readable.
        let first = unsafe { *input.as_ptr().add(input_index) };
        match first {
            0xfe if input.len() - input_index < 2 => {
                Err(qubit_codec::DecodeFailure::incomplete(qubit_io::nz!(2)))
            }
            0xfe => {
                // SAFETY: The branch above ensures the second byte is readable.
                let value = unsafe { *input.as_ptr().add(input_index + 1) };
                Ok((value, qubit_io::nz!(2)))
            }
            0xff => Err(qubit_codec::DecodeFailure::invalid(
                PrefixDecodeError::Invalid { consumed: 1 },
                core::num::NonZeroUsize::MIN,
            )),
            value => Ok((value, core::num::NonZeroUsize::MIN)),
        }
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(output_index < output.len());

        // SAFETY: The caller guarantees that `output_index` is writable.
        unsafe {
            *output.as_mut_ptr().add(output_index) = *value;
        }
        Ok(qubit_io::nz!(1))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct HintOnlyCodec;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HintOnlyDecodeError {
    Invalid,
}

impl Codec for HintOnlyCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = HintOnlyDecodeError;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: NonZeroUsize = NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: NonZeroUsize = qubit_io::nz!(2);

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>>
    {
        debug_assert!(input_index < input.len());

        match input[input_index] {
            0xaa => Err(qubit_codec::DecodeFailure::invalid(
                HintOnlyDecodeError::Invalid,
                qubit_io::nz!(2),
            )),
            value => Ok((value, NonZeroUsize::MIN)),
        }
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<NonZeroUsize, Self::EncodeError> {
        debug_assert!(output_index < output.len());

        output[output_index] = *value;
        Ok(NonZeroUsize::MIN)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OverconsumingCodec;

impl Codec for OverconsumingCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize = qubit_io::nz!(2);

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::DecodeFailure<Self::DecodeError>,
    > {
        debug_assert!(input_index < input.len());

        Ok((input[input_index], unsafe {
            core::num::NonZeroUsize::new_unchecked(2)
        }))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(output_index < output.len());

        output[output_index] = *value;
        Ok(qubit_io::nz!(1))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
enum EngineError {
    #[error("decode error")]
    Decode,
}

impl From<core::convert::Infallible> for EngineError {
    fn from(error: core::convert::Infallible) -> Self {
        match error {}
    }
}

impl From<PrefixDecodeError> for EngineError {
    fn from(_error: PrefixDecodeError) -> Self {
        Self::Decode
    }
}

impl From<HintOnlyDecodeError> for EngineError {
    fn from(_error: HintOnlyDecodeError) -> Self {
        Self::Decode
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ReplacingHooks;

impl TranscodeDecodeHooks<PrefixCodec> for ReplacingHooks {
    type Error = EngineError;
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut PrefixCodec,
        error: PrefixDecodeError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, Self::Error> {
        match error {
            PrefixDecodeError::Invalid { consumed } => {
                Ok(DecodeInvalidAction::Emit {
                    value: 99,
                    consumed: non_zero_consumed(consumed),
                })
            }
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct OverconsumingHooks;

impl TranscodeDecodeHooks<OverconsumingCodec> for OverconsumingHooks {
    type Error = EngineError;

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut OverconsumingCodec,
        error: core::convert::Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, Self::Error> {
        match error {}
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct SkippingHooks;

impl TranscodeDecodeHooks<PrefixCodec> for SkippingHooks {
    type Error = EngineError;

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut PrefixCodec,
        error: PrefixDecodeError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, Self::Error> {
        match error {
            PrefixDecodeError::Invalid { consumed } => {
                Ok(DecodeInvalidAction::Skip {
                    consumed: non_zero_consumed(consumed),
                })
            }
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct HintOnlySkippingHooks;

impl TranscodeDecodeHooks<HintOnlyCodec> for HintOnlySkippingHooks {
    type Error = EngineError;

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut HintOnlyCodec,
        error: HintOnlyDecodeError,
        consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, Self::Error> {
        match error {
            HintOnlyDecodeError::Invalid => Ok(DecodeInvalidAction::Skip {
                consumed: consumed.expect("codec should report invalid width"),
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

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut PrefixCodec,
        error: PrefixDecodeError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, Self::Error> {
        match error {
            PrefixDecodeError::Invalid { consumed } => {
                Ok(DecodeInvalidAction::Skip {
                    consumed: non_zero_consumed(consumed),
                })
            }
        }
    }

    fn max_finish_output_len(&self, _codec: &PrefixCodec) -> usize {
        usize::from(self.pending_suffix)
    }

    fn finish_hooks(
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
enum InvalidDecodeInvalidActionKind {
    Skip,
    Emit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct InvalidDecodeInvalidActionHooks {
    kind: InvalidDecodeInvalidActionKind,
}

impl TranscodeDecodeHooks<PrefixCodec> for InvalidDecodeInvalidActionHooks {
    type Error = EngineError;

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut PrefixCodec,
        _error: PrefixDecodeError,
        _consumed: Option<NonZeroUsize>,
        context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, Self::Error> {
        match self.kind {
            InvalidDecodeInvalidActionKind::Skip => {
                Ok(DecodeInvalidAction::Skip {
                    consumed: non_zero_consumed(context.available() + 1),
                })
            }
            InvalidDecodeInvalidActionKind::Emit => {
                Ok(DecodeInvalidAction::Emit {
                    value: 77,
                    consumed: non_zero_consumed(context.available() + 1),
                })
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OverwritingFinishHooks;

impl TranscodeDecodeHooks<PrefixCodec> for OverwritingFinishHooks {
    type Error = EngineError;

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut PrefixCodec,
        error: PrefixDecodeError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, Self::Error> {
        match error {
            PrefixDecodeError::Invalid { consumed } => {
                Ok(DecodeInvalidAction::Skip {
                    consumed: non_zero_consumed(consumed),
                })
            }
        }
    }

    fn max_finish_output_len(&self, _codec: &PrefixCodec) -> usize {
        1
    }

    fn finish_hooks(
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

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut PrefixCodec,
        error: PrefixDecodeError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, Self::Error> {
        match error {
            PrefixDecodeError::Invalid { consumed } => {
                Ok(DecodeInvalidAction::Skip {
                    consumed: non_zero_consumed(consumed),
                })
            }
        }
    }

    fn max_finish_output_len(&self, _codec: &PrefixCodec) -> usize {
        1
    }

    fn finish_hooks(
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

impl Codec for MinTwoCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = PrefixDecodeError;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize = qubit_io::nz!(2);

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize = qubit_io::nz!(2);

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::DecodeFailure<Self::DecodeError>,
    > {
        debug_assert!(input_index + 1 < input.len());

        Ok((
            input[input_index].wrapping_add(input[input_index + 1]),
            unsafe { core::num::NonZeroUsize::new_unchecked(2) },
        ))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(output_index < output.len());

        output[output_index] = *value;
        Ok(qubit_io::nz!(1))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OverflowFlushCodec;

impl Codec for OverflowFlushCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_DECODE_FLUSH_VALUES: usize = usize::MAX;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::DecodeFailure<Self::DecodeError>,
    > {
        Ok((input[input_index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        output[output_index] = *value;
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

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut OverflowFlushCodec,
        error: core::convert::Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, Self::Error> {
        match error {}
    }
}

impl TranscodeDecodeHooks<MinTwoCodec> for ReplacingHooks {
    type Error = EngineError;

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut MinTwoCodec,
        error: PrefixDecodeError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, Self::Error> {
        match error {
            PrefixDecodeError::Invalid { consumed } => {
                Ok(DecodeInvalidAction::Emit {
                    value: 99,
                    consumed: non_zero_consumed(consumed),
                })
            }
        }
    }
}

#[test]
fn test_transcode_decode_engine_reports_finish_bound_overflow() {
    let mut decoder = TranscodeDecodeEngine::<_, _>::new(
        OverflowFlushCodec,
        OverflowFinishHooks,
    );
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
    type Decoder = TranscodeDecodeEngine<PrefixCodec, ReplacingHooks>;
    type DecoderErrorType =
        TranscodeDecodeEngineError<PrefixDecodeError, EngineError>;
    type TranscodeAllIntoFn =
        fn(
            &mut Decoder,
            &[u8],
            &mut [u8],
        ) -> Result<usize, TranscodeError<DecoderErrorType>>;

    let mut decoder =
        TranscodeDecodeEngine::<_, _>::new(PrefixCodec, ReplacingHooks);
    let max_total_output_len: fn(
        &Decoder,
        usize,
    )
        -> Result<usize, qubit_codec::CapacityError> =
        Decoder::max_total_output_len;
    let transcode_all_into: TranscodeAllIntoFn = Decoder::transcode_all_into;
    let mut output = [0_u8; 1];

    assert_eq!(Ok(3), decoder.max_transcode_output_len(3));
    assert_eq!(Ok(3), max_total_output_len(&decoder, 3));
    assert_eq!(Ok(0), decoder.max_finish_output_len());

    let mut all_output = [0_u8; 3];
    let written = transcode_all_into(&mut decoder, &[1, 2, 3], &mut all_output)
        .expect("complete decode should fit the planned output");
    assert_eq!(3, written);
    assert_eq!(&[1, 2, 3], &all_output[..written]);

    decoder.reset(&mut [], 0).expect("reset");
    let written = decoder
        .finish(&mut output, 0)
        .expect("generic decoder finish is a no-op");
    assert_eq!(0, written);
}

#[test]
fn test_transcode_decode_engine_delegates_finish_to_hooks() {
    let mut decoder =
        TranscodeDecodeEngine::<_, _>::new(PrefixCodec, FinishHooks::default());
    let mut output = [0_u8; 1];

    assert_eq!(Ok(1), decoder.max_finish_output_len());

    let error = decoder.finish(&mut [], 0).expect_err(
        "finish should reject insufficient output before calling hooks",
    );
    assert_eq!(
        TranscodeError::InsufficientOutput {
            output_index: 0,
            required: 1,
            available: 0
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
    let mut decoder =
        TranscodeDecodeEngine::<_, _>::new(PrefixCodec, OverwritingFinishHooks);
    let mut output = [0_u8; 2];

    let written = decoder
        .finish(&mut output, 0)
        .expect("hook should receive the caller-provided output slice");

    assert_eq!(1, written);
    assert_eq!([0xee, 0xdd], output);
}

#[test]
#[should_panic(
    expected = "TranscodeDecodeEngine hook wrote beyond its finish bound"
)]
fn test_transcode_decode_engine_finish_panics_when_hook_overreports_bound() {
    let mut decoder = TranscodeDecodeEngine::<_, _>::new(
        PrefixCodec,
        OverreportingFinishHooks,
    );
    let mut output = [0_u8; 2];

    let _ = decoder.finish(&mut output, 0);
}

#[test]
fn test_transcode_decode_engine_finish_reports_output_index_beyond_buffer() {
    let mut decoder =
        TranscodeDecodeEngine::<_, _>::new(PrefixCodec, FinishHooks::default());
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
fn test_transcode_decode_engine_default_finish_reports_output_index_beyond_buffer()
 {
    let mut decoder =
        TranscodeDecodeEngine::<_, _>::new(PrefixCodec, ReplacingHooks);
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

    let written = TranscodeDecodeHooks::<PrefixCodec>::finish_hooks(
        &mut hooks,
        &mut PrefixCodec,
        &mut output,
        1,
    )
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
            required: crate::nz(2),
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
fn test_transcode_decode_engine_reports_short_minimum_input_without_consuming_tail()
 {
    let mut decoder = TranscodeDecodeEngine::new(MinTwoCodec, ReplacingHooks);
    let mut output = [0_u8; 1];

    let progress = decoder
        .transcode(&[7], 0, &mut output, 0)
        .expect("short input should request another unit");

    assert_eq!(
        TranscodeStatus::NeedInput {
            input_index: 0,
            required: crate::nz(2),
            available: 1,
        },
        progress.status(),
    );
    assert_eq!(0, progress.read());
    assert_eq!(0, progress.written());
}

#[test]
fn test_transcode_decode_engine_reports_incomplete_input_before_missing_output()
{
    let mut decoder = TranscodeDecodeEngine::new(MinTwoCodec, ReplacingHooks);
    let mut output = [];

    let progress = decoder.transcode(&[7], 0, &mut output, 0).expect(
        "short input should request another unit before output capacity",
    );

    assert_eq!(
        TranscodeStatus::NeedInput {
            input_index: 0,
            required: crate::nz(2),
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
            required: crate::nz(1),
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
fn test_transcode_decode_engine_passes_invalid_consumed_hint_to_hooks() {
    let mut decoder =
        TranscodeDecodeEngine::new(HintOnlyCodec, HintOnlySkippingHooks);
    let mut output = [0_u8; 1];

    let progress = decoder
        .transcode(&[0xaa, 0xbb, 1], 0, &mut output, 0)
        .expect("policy should skip invalid width from failure hint");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(3, progress.read());
    assert_eq!(1, progress.written());
    assert_eq!([1], output);
}

#[test]
#[should_panic(
    expected = "DecodeInvalidAction consumed units must not exceed available input"
)]
fn test_transcode_decode_engine_panics_on_invalid_skip_action() {
    let mut decoder = TranscodeDecodeEngine::new(
        PrefixCodec,
        InvalidDecodeInvalidActionHooks {
            kind: InvalidDecodeInvalidActionKind::Skip,
        },
    );
    let mut output = [0_u8; 1];

    let _ = decoder.transcode(&[0xff], 0, &mut output, 0);
}

#[test]
#[should_panic(
    expected = "DecodeInvalidAction consumed units must not exceed available input"
)]
fn test_transcode_decode_engine_panics_on_invalid_emit_action() {
    let mut decoder = TranscodeDecodeEngine::new(
        PrefixCodec,
        InvalidDecodeInvalidActionHooks {
            kind: InvalidDecodeInvalidActionKind::Emit,
        },
    );
    let mut output = [0_u8; 1];

    let _ = decoder.transcode(&[0xff], 0, &mut output, 0);
}

#[test]
fn test_transcode_decode_engine_reports_output_bounds_without_consuming_input()
{
    let mut decoder = TranscodeDecodeEngine::new(PrefixCodec, ReplacingHooks);
    let mut output = [];

    let progress = decoder
        .transcode(&[1], 0, &mut output, 0)
        .expect("empty output should request capacity");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 0,
            required: crate::nz(1),
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
fn test_transcode_decode_engine_panics_when_codec_consumes_beyond_available_input()
 {
    let mut decoder =
        TranscodeDecodeEngine::new(OverconsumingCodec, OverconsumingHooks);
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
    >>::max_transcode_output_len(&decoder, 1)
    .expect("max_transcode_output_len should be callable through trait");
    assert_eq!(1, available);

    let mut output = [0_u8; 1];
    let progress =
        <Decoder as Transcoder<
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

impl Codec for FlushFailCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = FlushFailError;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_DECODE_FLUSH_VALUES: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::DecodeFailure<Self::DecodeError>,
    > {
        Ok((input[input_index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        output[output_index] = *value;
        Ok(qubit_io::nz!(1))
    }

    unsafe fn decode_flush(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        Err(FlushFailError)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct FlushMappingHooks;

impl TranscodeDecodeHooks<FlushFailCodec> for FlushMappingHooks {
    type Error = qubit_codec::CodecDecodeError<FlushFailError>;

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut FlushFailCodec,
        error: FlushFailError,
        _consumed: Option<NonZeroUsize>,
        context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, Self::Error> {
        Err(qubit_codec::CodecDecodeError::decode(
            error,
            context.input_index(),
        ))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ResetObservingHooks {
    called: std::rc::Rc<Cell<bool>>,
}

impl TranscodeDecodeHooks<PrefixCodec> for ResetObservingHooks {
    type Error = PrefixDecodeError;

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut PrefixCodec,
        error: PrefixDecodeError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, Self::Error> {
        Err(error)
    }

    fn reset_hooks(&mut self, _codec: &mut PrefixCodec) {
        self.called.set(true);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
struct ResetFailCodec;

impl Codec for ResetFailCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = PrefixDecodeError;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize = qubit_io::nz!(1);

    const MAX_DECODE_RESET_VALUES: usize = 1;

    unsafe fn decode(
        &mut self,
        _input: &[u8],
        _input_index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::DecodeFailure<Self::DecodeError>,
    > {
        Ok((0u8, core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        _value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(output_index < output.len());

        // SAFETY: Caller guarantees that `output_index` is writable.
        unsafe {
            *output.get_unchecked_mut(output_index) = 0;
        }
        Ok(qubit_io::nz!(1))
    }

    unsafe fn decode_reset(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        Err(PrefixDecodeError::Invalid { consumed: 1 })
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ResetErrorMappingHooks;

impl TranscodeDecodeHooks<ResetFailCodec> for ResetErrorMappingHooks {
    type Error = EngineError;

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut ResetFailCodec,
        error: PrefixDecodeError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, Self::Error> {
        Err(error.into())
    }
}

#[test]
fn test_transcode_decode_engine_reports_max_reset_output_len() {
    let decoder = TranscodeDecodeEngine::<PrefixCodec, ReplacingHooks>::new(
        PrefixCodec,
        ReplacingHooks,
    );

    assert_eq!(Ok(0), decoder.max_reset_output_len());
    assert_eq!(Ok(0), Transcoder::max_reset_output_len(&decoder));
}

#[test]
fn test_transcode_decode_engine_reset_rejects_invalid_output_index() {
    let mut decoder = TranscodeDecodeEngine::<PrefixCodec, ReplacingHooks>::new(
        PrefixCodec,
        ReplacingHooks,
    );

    let error = decoder
        .reset(&mut [], 1)
        .expect_err("reset should reject invalid output index");

    assert_eq!(
        TranscodeError::InvalidOutputIndex { index: 1, len: 0 },
        error,
    );
}

#[test]
fn test_transcode_decode_engine_reset_calls_hook_before_reset() {
    let called = std::rc::Rc::new(Cell::new(false));
    let hooks = ResetObservingHooks {
        called: called.clone(),
    };
    let mut decoder = TranscodeDecodeEngine::<_, _>::new(PrefixCodec, hooks);

    decoder.reset(&mut [], 0).expect("reset should succeed");

    assert!(called.get(), "decode hook before_reset should be called");
}

#[test]
fn test_transcode_decode_engine_finish_converts_decode_flush_errors() {
    let mut decoder =
        TranscodeDecodeEngine::<_, _>::new(FlushFailCodec, FlushMappingHooks);
    let mut output = [0_u8; 1];

    let error = decoder.finish(&mut output, 0).expect_err(
        "flush errors should be converted through the hook error type",
    );

    assert_eq!(
        TranscodeError::Domain(TranscodeDecodeEngineError::Codec(
            CodecDecodeError::DecodeFlush {
                source: FlushFailError,
            },
        )),
        error,
    );
}

#[test]
fn test_transcode_decode_engine_reset_converts_decode_reset_errors() {
    let mut decoder = TranscodeDecodeEngine::<_, _>::new(
        ResetFailCodec,
        ResetErrorMappingHooks,
    );
    let mut output = [0_u8; 1];

    let error = decoder.reset(&mut output, 0).expect_err(
        "decode reset errors should be converted through the hook error type",
    );

    assert_eq!(
        TranscodeError::Domain(TranscodeDecodeEngineError::Codec(
            CodecDecodeError::DecodeReset {
                source: PrefixDecodeError::Invalid { consumed: 1 },
            },
        )),
        error,
    );
}

// ============================================================================
// Lifecycle guard wiring
//
// The guard is a debug-only check that the documented `reset → transcode* →
// finish` lifecycle is respected. `#[should_panic]` tests are gated behind
// `cfg(debug_assertions)` because the guard collapses to a ZST in release
// builds, so panic-shape tests would not fire there.
// ============================================================================

fn new_stateless_finish_engine()
-> TranscodeDecodeEngine<PrefixCodec, FinishHooks> {
    TranscodeDecodeEngine::<_, _>::new(
        PrefixCodec,
        FinishHooks {
            pending_suffix: false,
        },
    )
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(
    expected = "Transcoder::finish called twice without an intervening reset"
)]
fn test_transcode_decode_engine_lifecycle_rejects_double_finish() {
    let mut engine = new_stateless_finish_engine();
    let mut output = [0_u8; 0];
    engine
        .finish(&mut output, 0)
        .expect("first finish should succeed on a stateless decoder");
    let _ = engine.finish(&mut output, 0);
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(
    expected = "Transcoder::transcode called after finish without an \
                intervening reset"
)]
fn test_transcode_decode_engine_lifecycle_rejects_transcode_after_finish() {
    let mut engine = new_stateless_finish_engine();
    let mut output = [0_u8; 0];
    engine
        .finish(&mut output, 0)
        .expect("finish closes the logical stream");
    let mut grown = [0_u8; 1];
    let _ = engine.transcode(&[0x10], 0, &mut grown, 0);
}

#[test]
fn test_transcode_decode_engine_lifecycle_allows_finish_without_transcode() {
    let mut engine = new_stateless_finish_engine();
    let mut output = [0_u8; 0];
    let written = engine
        .finish(&mut output, 0)
        .expect("fresh stateless engine may finalize an empty stream");
    assert_eq!(0, written);
}

#[test]
fn test_transcode_decode_engine_lifecycle_allows_finish_retry_after_capacity_failure()
 {
    // FinishHooks::default() declares `pending_suffix = true`, which reserves
    // one output value at finish time. Passing an empty slice triggers an
    // `InsufficientOutput` failure; the guard must not mark the engine
    // closed when finish fails before doing any work.
    let mut engine =
        TranscodeDecodeEngine::<_, _>::new(PrefixCodec, FinishHooks::default());
    let mut tiny = [0_u8; 0];
    let _ = engine
        .finish(&mut tiny, 0)
        .expect_err("finish should reject insufficient output");
    let mut output = [0_u8; 1];
    let written = engine
        .finish(&mut output, 0)
        .expect("retry after capacity failure must succeed");
    assert_eq!(1, written);
}

#[test]
fn test_transcode_decode_engine_lifecycle_allows_reuse_after_reset() {
    let mut engine = new_stateless_finish_engine();
    let mut buf = [0_u8; 4];
    engine
        .finish(&mut buf, 0)
        .expect("close the first logical stream");
    engine
        .reset(&mut buf, 0)
        .expect("reset must reopen the engine for a new logical stream");
    let progress = engine
        .transcode(&[0x10, 0x20], 0, &mut buf, 0)
        .expect("transcode should resume after reset");
    assert_eq!(2, progress.read());
    engine
        .finish(&mut buf, 2)
        .expect("finish must close the second logical stream");
}

#[test]
fn test_transcode_decode_engine_lifecycle_allows_multiple_resets() {
    let mut engine = new_stateless_finish_engine();
    let mut buf = [0_u8; 0];
    for _ in 0..3 {
        engine
            .reset(&mut buf, 0)
            .expect("reset must always be legal");
    }
}
