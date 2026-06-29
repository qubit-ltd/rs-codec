// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the reusable buffered encoder engine.

use qubit_codec::{
    CapacityError,
    Codec,
    CodecPhase,
    EncodeUnencodableAction,
    TranscodeEncodeEngine,
    TranscodeEncodeHooks,
    TranscodeError,
    TranscodeProgress,
    TranscodeStatus,
    Transcoder,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct WideCodec;

impl Codec for WideCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize = qubit_io::nz!(4);

    fn can_encode_value(&self, value: &u8) -> bool {
        *value != 0
    }

    fn encode_len(&self, _value: &u8) -> core::num::NonZeroUsize {
        qubit_io::nz!(1)
    }

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
        let value = unsafe { *input.as_ptr().add(input_index) };
        Ok((value, core::num::NonZeroUsize::MIN))
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
            *output.as_mut_ptr().add(output_index) = value.wrapping_add(10);
        }
        Ok(qubit_io::nz!(1))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
enum EngineError {
    #[error("rejected input at index {input_index}")]
    Rejected { input_index: usize },
}

impl From<core::convert::Infallible> for EngineError {
    fn from(error: core::convert::Infallible) -> Self {
        match error {}
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ExactWidthHooks;

impl<C> TranscodeEncodeHooks<C> for ExactWidthHooks
where
    C: Codec<Value = u8, Unit = u8>,
{
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut C,
        _value: &u8,
        _input_index: usize,
    ) -> Result<EncodeUnencodableAction<u8>, qubit_codec::TranscodeEncodeError<C>>
    {
        Ok(EncodeUnencodableAction::Reject)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct SkippingHooks;

impl TranscodeEncodeHooks<WideCodec> for SkippingHooks {
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut WideCodec,
        _value: &u8,
        _input_index: usize,
    ) -> Result<
        EncodeUnencodableAction<u8>,
        qubit_codec::TranscodeEncodeError<WideCodec>,
    > {
        Ok(EncodeUnencodableAction::Skip)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct RejectingHooks;

impl TranscodeEncodeHooks<WideCodec> for RejectingHooks {
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut WideCodec,
        _value: &u8,
        _input_index: usize,
    ) -> Result<
        EncodeUnencodableAction<u8>,
        qubit_codec::TranscodeEncodeError<WideCodec>,
    > {
        Ok(EncodeUnencodableAction::Reject)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ReplacingHooks {
    replacement: u8,
}

impl TranscodeEncodeHooks<WideCodec> for ReplacingHooks {
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut WideCodec,
        _value: &u8,
        _input_index: usize,
    ) -> Result<
        EncodeUnencodableAction<u8>,
        qubit_codec::TranscodeEncodeError<WideCodec>,
    > {
        Ok(EncodeUnencodableAction::replace(self.replacement))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ReplacementEncodeFailCodec;

impl Codec for ReplacementEncodeFailCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = EngineError;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    fn can_encode_value(&self, value: &u8) -> bool {
        *value != 0
    }

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
        if *value == 7 {
            return Err(EngineError::Rejected {
                input_index: output_index,
            });
        }
        output[output_index] = *value;
        Ok(qubit_io::nz!(1))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FailingReplacementHooks;

impl TranscodeEncodeHooks<ReplacementEncodeFailCodec>
    for FailingReplacementHooks
{
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut ReplacementEncodeFailCodec,
        _value: &u8,
        _input_index: usize,
    ) -> Result<
        EncodeUnencodableAction<u8>,
        qubit_codec::TranscodeEncodeError<ReplacementEncodeFailCodec>,
    > {
        Ok(EncodeUnencodableAction::replace(7))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FailingFinishHooks;

impl TranscodeEncodeHooks<ReplacementEncodeFailCodec> for FailingFinishHooks {
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut ReplacementEncodeFailCodec,
        _value: &u8,
        _input_index: usize,
    ) -> Result<
        EncodeUnencodableAction<u8>,
        qubit_codec::TranscodeEncodeError<ReplacementEncodeFailCodec>,
    > {
        Ok(EncodeUnencodableAction::Reject)
    }

    fn max_finish_output_len(
        &self,
        _codec: &ReplacementEncodeFailCodec,
    ) -> usize {
        1
    }

    fn finish_hooks(
        &mut self,
        _codec: &mut ReplacementEncodeFailCodec,
        _output: &mut [u8],
        output_index: usize,
    ) -> Result<
        usize,
        qubit_codec::TranscodeEncodeError<ReplacementEncodeFailCodec>,
    > {
        Err(TranscodeError::domain(
            EngineError::Rejected {
                input_index: output_index,
            },
            CodecPhase::Flush,
            None,
        ))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FlushFailingCodec;

impl Codec for FlushFailingCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = EngineError;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize = qubit_io::nz!(1);

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

    unsafe fn encode_flush(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        Err(EngineError::Rejected { input_index: 0 })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FlushFailingHooks;

impl TranscodeEncodeHooks<FlushFailingCodec> for FlushFailingHooks {
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut FlushFailingCodec,
        _value: &u8,
        _input_index: usize,
    ) -> Result<
        EncodeUnencodableAction<u8>,
        qubit_codec::TranscodeEncodeError<FlushFailingCodec>,
    > {
        Ok(EncodeUnencodableAction::Reject)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OverreportingEncodeCodec;

impl Codec for OverreportingEncodeCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

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
        Ok(qubit_io::nz!(2))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

impl TranscodeEncodeHooks<WideCodec> for FinishHooks {
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut WideCodec,
        _value: &u8,
        _input_index: usize,
    ) -> Result<
        EncodeUnencodableAction<u8>,
        qubit_codec::TranscodeEncodeError<WideCodec>,
    > {
        Ok(EncodeUnencodableAction::Reject)
    }

    fn max_finish_output_len(&self, _codec: &WideCodec) -> usize {
        usize::from(self.pending_suffix)
    }

    fn finish_hooks(
        &mut self,
        _codec: &mut WideCodec,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, qubit_codec::TranscodeEncodeError<WideCodec>> {
        if !self.pending_suffix {
            return Ok(0);
        }

        output[output_index] = 0xee;
        self.pending_suffix = false;
        Ok(1)
    }

    fn reset_hooks(&mut self, _codec: &mut WideCodec) {
        self.pending_suffix = false;
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OverwritingFinishHooks;

impl TranscodeEncodeHooks<WideCodec> for OverwritingFinishHooks {
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut WideCodec,
        _value: &u8,
        _input_index: usize,
    ) -> Result<
        EncodeUnencodableAction<u8>,
        qubit_codec::TranscodeEncodeError<WideCodec>,
    > {
        Ok(EncodeUnencodableAction::Reject)
    }

    fn max_finish_output_len(&self, _codec: &WideCodec) -> usize {
        1
    }

    fn finish_hooks(
        &mut self,
        _codec: &mut WideCodec,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, qubit_codec::TranscodeEncodeError<WideCodec>> {
        output[output_index] = 0xee;
        output[output_index + 1] = 0xdd;
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OverreportingFinishHooks;

impl TranscodeEncodeHooks<WideCodec> for OverreportingFinishHooks {
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut WideCodec,
        _value: &u8,
        _input_index: usize,
    ) -> Result<
        EncodeUnencodableAction<u8>,
        qubit_codec::TranscodeEncodeError<WideCodec>,
    > {
        Ok(EncodeUnencodableAction::Reject)
    }

    fn max_finish_output_len(&self, _codec: &WideCodec) -> usize {
        1
    }

    fn finish_hooks(
        &mut self,
        _codec: &mut WideCodec,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, qubit_codec::TranscodeEncodeError<WideCodec>> {
        output[output_index] = 0xee;
        Ok(2)
    }
}

#[test]
fn test_transcode_encode_engine_exposes_codec_hooks_and_parts() {
    let mut engine = TranscodeEncodeEngine::new(WideCodec, ExactWidthHooks);

    assert_eq!(&WideCodec, engine.codec());
    assert_eq!(&ExactWidthHooks, engine.hooks());
    *engine.codec_mut() = WideCodec;
    *engine.hooks_mut() = ExactWidthHooks;

    let (codec, hooks) = engine.into_parts();
    assert_eq!(WideCodec, codec);
    assert_eq!(ExactWidthHooks, hooks);
}

#[test]
fn test_buffered_encode_engine_reports_bounds_and_resets() {
    type Engine = TranscodeEncodeEngine<WideCodec, ExactWidthHooks>;
    type TranscodeCompleteIntoFn =
        fn(
            &mut Engine,
            &[u8],
            &mut [u8],
        ) -> Result<usize, TranscodeError<core::convert::Infallible>>;

    let mut encoder =
        TranscodeEncodeEngine::<_, _>::new(WideCodec, ExactWidthHooks);
    let max_total_output_len: fn(
        &Engine,
        usize,
    ) -> Result<
        usize,
        TranscodeError<core::convert::Infallible>,
    > = Engine::max_total_output_len;
    let transcode_complete_into: TranscodeCompleteIntoFn =
        Engine::transcode_complete_into;

    assert_eq!(Ok(8), encoder.max_transcode_output_len(2));
    assert_eq!(Ok(8), max_total_output_len(&encoder, 2));
    assert_eq!(Ok(0), encoder.max_finish_output_len());
    assert_eq!(Ok(0), encoder.max_reset_output_len());
    assert_eq!(
        Err(TranscodeError::OutputLengthOverflow),
        max_total_output_len(&encoder, usize::MAX),
    );
    assert_eq!(
        Err(TranscodeError::OutputLengthOverflow),
        encoder.max_transcode_output_len(usize::MAX),
    );
    let mut output = [0_u8; 8];
    let written = transcode_complete_into(&mut encoder, &[1, 2], &mut output)
        .expect("complete encode should fit the planned output");
    assert_eq!(2, written);
    assert_eq!(&[11, 12], &output[..written]);
}

#[test]
fn test_buffered_encode_engine_delegates_finish_to_hooks() {
    let mut encoder =
        TranscodeEncodeEngine::<_, _>::new(WideCodec, FinishHooks::default());
    let mut output = [0_u8; 1];

    assert_eq!(Ok(1), encoder.max_finish_output_len());

    let error = encoder.finish(&mut [], 0).expect_err(
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
    assert_eq!(Ok(1), encoder.max_finish_output_len());

    let written = encoder
        .finish(&mut output, 0)
        .expect("hook should write final output");
    assert_eq!(1, written);
    assert_eq!([0xee], output);
    assert_eq!(Ok(0), encoder.max_finish_output_len());

    let mut encoder =
        TranscodeEncodeEngine::<_, _>::new(WideCodec, FinishHooks::default());
    encoder.reset(&mut [], 0).expect("reset");
    assert_eq!(Ok(0), encoder.max_finish_output_len());
}

#[test]
fn test_buffered_encode_engine_implements_transcoder() {
    type Engine = TranscodeEncodeEngine<WideCodec, ExactWidthHooks>;
    type EngineResult<T> = Result<T, TranscodeError<core::convert::Infallible>>;
    type TranscodeFn = fn(
        &mut Engine,
        &[u8],
        usize,
        &mut [u8],
        usize,
    ) -> EngineResult<TranscodeProgress>;
    type OutputFn = fn(&mut Engine, &mut [u8], usize) -> EngineResult<usize>;

    let mut encoder =
        TranscodeEncodeEngine::<_, _>::new(WideCodec, ExactWidthHooks);
    let mut output = [0_u8; 2];
    let max_transcode_output_len: fn(
        &Engine,
        usize,
    ) -> Result<usize, CapacityError> = std::hint::black_box(
        <Engine as Transcoder<u8, u8>>::max_transcode_output_len,
    );
    let max_finish_output_len: fn(&Engine) -> Result<usize, CapacityError> =
        std::hint::black_box(
            <Engine as Transcoder<u8, u8>>::max_finish_output_len,
        );
    let max_reset_output_len: fn(&Engine) -> Result<usize, CapacityError> =
        std::hint::black_box(
            <Engine as Transcoder<u8, u8>>::max_reset_output_len,
        );
    let transcode: TranscodeFn =
        std::hint::black_box(<Engine as Transcoder<u8, u8>>::transcode);
    let reset: OutputFn =
        std::hint::black_box(<Engine as Transcoder<u8, u8>>::reset);
    let finish: OutputFn =
        std::hint::black_box(<Engine as Transcoder<u8, u8>>::finish);

    assert_eq!(Ok(8), max_transcode_output_len(&encoder, 2));
    assert_eq!(Ok(0), max_finish_output_len(&encoder));
    assert_eq!(Ok(0), max_reset_output_len(&encoder));
    let progress = transcode(&mut encoder, &[1, 2], 0, &mut output, 0)
        .expect("engine should transcode through the trait");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((2, 2), (progress.read(), progress.written()));
    assert_eq!([11, 12], output);

    let mut empty_output = [0_u8; 0];
    let reset = reset(&mut encoder, &mut empty_output, 0)
        .expect("engine should reset through the trait");
    let finished = finish(&mut encoder, &mut empty_output, 0)
        .expect("engine should finish through the trait");

    assert_eq!(0, reset);
    assert_eq!(0, finished);
}

#[test]
fn test_buffered_encode_engine_finish_maps_hook_errors() {
    let mut encoder = TranscodeEncodeEngine::<_, _>::new(
        ReplacementEncodeFailCodec,
        FailingFinishHooks,
    );
    let mut output = [0_u8; 1];

    let error = encoder
        .finish(&mut output, 0)
        .expect_err("finish hook error should be propagated");

    assert_eq!(
        TranscodeError::domain(
            EngineError::Rejected { input_index: 0 },
            CodecPhase::Flush,
            None,
        ),
        error,
    );
}

#[test]
fn test_buffered_encode_engine_finish_converts_codec_flush_errors() {
    let mut encoder = TranscodeEncodeEngine::<_, _>::new(
        FlushFailingCodec,
        FlushFailingHooks,
    );
    let mut output = [0_u8; 1];

    let error = encoder
        .finish(&mut output, 0)
        .expect_err("finish should convert codec flush errors");

    assert_eq!(
        TranscodeError::domain(
            EngineError::Rejected { input_index: 0 },
            CodecPhase::Flush,
            None,
        ),
        error,
    );
}

#[test]
fn test_buffered_encode_engine_finish_passes_full_output_to_hooks() {
    let mut encoder =
        TranscodeEncodeEngine::<_, _>::new(WideCodec, OverwritingFinishHooks);
    let mut output = [0_u8; 2];

    let written = encoder
        .finish(&mut output, 0)
        .expect("hook should receive the caller-provided output slice");

    assert_eq!(1, written);
    assert_eq!([0xee, 0xdd], output);
}

#[test]
#[should_panic(
    expected = "TranscodeEncodeEngine hook wrote beyond its finish bound"
)]
fn test_buffered_encode_engine_finish_panics_when_hook_overreports_bound() {
    let mut encoder =
        TranscodeEncodeEngine::<_, _>::new(WideCodec, OverreportingFinishHooks);
    let mut output = [0_u8; 2];

    let _ = encoder.finish(&mut output, 0);
}

#[test]
fn test_buffered_encode_engine_finish_reports_output_index_beyond_buffer() {
    let mut encoder =
        TranscodeEncodeEngine::<_, _>::new(WideCodec, FinishHooks::default());
    let mut output = [];

    let error = encoder
        .finish(&mut output, 1)
        .expect_err("out-of-range finish output index should be rejected");

    assert_eq!(
        TranscodeError::InvalidOutputIndex { index: 1, len: 0 },
        error,
    );
}

#[test]
fn test_buffered_encode_engine_default_finish_reports_output_index_beyond_buffer()
 {
    let mut encoder =
        TranscodeEncodeEngine::<_, _>::new(WideCodec, ExactWidthHooks);
    let mut output = [];

    let error = encoder
        .finish(&mut output, 1)
        .expect_err("default finish should reject out-of-range output index");

    assert_eq!(
        TranscodeError::InvalidOutputIndex { index: 1, len: 0 },
        error,
    );
}

#[test]
fn test_buffered_encode_hooks_default_finish_is_noop() {
    let mut hooks = ExactWidthHooks;
    let mut output = [];

    let written = TranscodeEncodeHooks::<WideCodec>::finish_hooks(
        &mut hooks,
        &mut WideCodec,
        &mut output,
        1,
    )
    .expect("default hook finish should be a no-op");

    assert_eq!(0, written);
}

#[test]
fn test_buffered_encode_engine_uses_exact_value_width_for_output_pressure() {
    let mut encoder = TranscodeEncodeEngine::new(WideCodec, ExactWidthHooks);
    let mut output = [0_u8; 1];

    let progress = encoder
        .transcode(&[1, 2], 0, &mut output, 0)
        .expect("engine encoding should succeed");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 1,
            required: crate::nz(1),
            available: 0,
        },
        progress.status(),
    );
    assert_eq!(1, progress.read());
    assert_eq!(1, progress.written());
    assert_eq!([11], output);
    assert_eq!(Ok(8), encoder.max_transcode_output_len(2));
}

#[test]
fn test_buffered_encode_engine_allows_zero_width_value_to_consume_input() {
    let mut encoder = TranscodeEncodeEngine::new(WideCodec, SkippingHooks);
    let mut output = [];

    let progress = encoder
        .transcode(&[0, 0, 0], 0, &mut output, 0)
        .expect("zero-width value should not need output");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(3, progress.read());
    assert_eq!(0, progress.written());
}

#[test]
fn test_buffered_encode_engine_reports_output_index_beyond_buffer() {
    let mut encoder = TranscodeEncodeEngine::new(WideCodec, ExactWidthHooks);
    let mut output = [];

    let error = encoder
        .transcode(&[1], 0, &mut output, 1)
        .expect_err("out-of-range output index should fail");

    assert_eq!(
        TranscodeError::InvalidOutputIndex { index: 1, len: 0 },
        error,
    );
}

#[test]
#[should_panic(
    expected = "Codec::encode wrote a different length than Codec::encode_len"
)]
fn test_buffered_encode_engine_panics_when_codec_reports_wrong_value_width() {
    let mut encoder =
        TranscodeEncodeEngine::new(OverreportingEncodeCodec, ExactWidthHooks);
    let mut output = [0_u8; 1];

    let _ = encoder.transcode(&[1], 0, &mut output, 0);
}

#[test]
fn test_buffered_encode_engine_encodes_replacement_for_unencodable_value() {
    let mut encoder = TranscodeEncodeEngine::new(
        WideCodec,
        ReplacingHooks { replacement: 5 },
    );
    let mut output = [0_u8; 1];

    let progress = encoder
        .transcode(&[0], 0, &mut output, 0)
        .expect("replacement should encode through the codec");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(1, progress.read());
    assert_eq!(1, progress.written());
    assert_eq!([15], output);
}

#[test]
fn test_buffered_encode_engine_replacement_waits_for_output_capacity() {
    let mut encoder = TranscodeEncodeEngine::new(
        WideCodec,
        ReplacingHooks { replacement: 5 },
    );
    let mut output = [];

    let progress = encoder
        .transcode(&[0], 0, &mut output, 0)
        .expect("replacement should report output pressure");

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
#[should_panic(
    expected = "EncodeUnencodableAction::Replace returned an unencodable replacement value"
)]
fn test_buffered_encode_engine_panics_when_replacement_is_unencodable() {
    let mut encoder = TranscodeEncodeEngine::new(
        WideCodec,
        ReplacingHooks { replacement: 0 },
    );
    let mut output = [0_u8; 1];

    let _ = encoder.transcode(&[0], 0, &mut output, 0);
}

#[test]
fn test_buffered_encode_engine_maps_replacement_encode_error() {
    let mut encoder = TranscodeEncodeEngine::new(
        ReplacementEncodeFailCodec,
        FailingReplacementHooks,
    );
    let mut output = [0_u8; 1];

    let error = encoder
        .transcode(&[0], 0, &mut output, 0)
        .expect_err("replacement encode failure should be mapped");

    assert!(matches!(
        error,
        TranscodeError::Domain {
            source: EngineError::Rejected { input_index: 0 },
            phase: CodecPhase::Main,
            input_index: Some(0),
        },
    ));
    assert_eq!([0], output);
}

#[test]
fn test_buffered_encode_engine_propagates_unencodable_hook_error_without_consuming_input()
 {
    let mut encoder = TranscodeEncodeEngine::new(WideCodec, RejectingHooks);
    let mut output = [0_u8; 4];

    let error = encoder
        .transcode(&[0], 0, &mut output, 0)
        .expect_err("encode hook error should be propagated");

    assert_eq!(TranscodeError::UnencodableValue { input_index: 0 }, error);
    assert_eq!([0, 0, 0, 0], output);
}

#[test]
fn test_buffered_encode_engine_uses_hooks_for_invalid_input_index() {
    let mut encoder = TranscodeEncodeEngine::new(WideCodec, ExactWidthHooks);
    let mut output = [];

    let error = encoder
        .transcode(&[1], 2, &mut output, 0)
        .expect_err("invalid input index should be rejected");

    assert_eq!(
        TranscodeError::InvalidInputIndex { index: 2, len: 1 },
        error,
    );
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetEmittingCodec;

impl Codec for ResetEmittingCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_ENCODE_RESET_UNITS: usize = 1;

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

    unsafe fn encode_reset(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        output[output_index] = 0xaa;
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetFailCodec;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("reset failed")]
struct ResetFailError;

impl Codec for ResetFailCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = ResetFailError;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_ENCODE_RESET_UNITS: usize = 1;

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

    unsafe fn encode_reset(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        Err(ResetFailError)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ResetErrorMappingHooks;

impl TranscodeEncodeHooks<ResetFailCodec> for ResetErrorMappingHooks {
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut ResetFailCodec,
        _value: &u8,
        _input_index: usize,
    ) -> Result<
        EncodeUnencodableAction<u8>,
        qubit_codec::TranscodeEncodeError<ResetFailCodec>,
    > {
        Err(TranscodeError::domain(
            ResetFailError,
            CodecPhase::Main,
            Some(_input_index),
        ))
    }
}

#[test]
fn test_buffered_encode_engine_default_builds_engine() {
    let mut encoder =
        TranscodeEncodeEngine::<WideCodec, ExactWidthHooks>::default();
    let mut output = [0_u8; 1];

    let progress = encoder
        .transcode(&[7], 0, &mut output, 0)
        .expect("default engine should encode one value");

    assert_eq!(1, progress.read());
    assert_eq!(1, progress.written());
    assert_eq!([17], output);
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetPassthroughHooks;

impl TranscodeEncodeHooks<ResetEmittingCodec> for ResetPassthroughHooks {
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut ResetEmittingCodec,
        _value: &u8,
        _input_index: usize,
    ) -> Result<
        EncodeUnencodableAction<u8>,
        qubit_codec::TranscodeEncodeError<ResetEmittingCodec>,
    > {
        Ok(EncodeUnencodableAction::Reject)
    }
}

#[test]
fn test_buffered_encode_engine_reset_emits_codec_reset_output() {
    let mut encoder = TranscodeEncodeEngine::<_, _>::new(
        ResetEmittingCodec,
        ResetPassthroughHooks,
    );
    let mut output = [0_u8; 1];

    let written = encoder
        .reset(&mut output, 0)
        .expect("reset should emit codec reset output");

    assert_eq!(1, written);
    assert_eq!([0xaa], output);
}

#[test]
fn test_buffered_encode_engine_reset_rejects_insufficient_output() {
    let mut encoder = TranscodeEncodeEngine::<_, _>::new(
        ResetEmittingCodec,
        ResetPassthroughHooks,
    );
    let mut output = [];

    let error = encoder
        .reset(&mut output, 0)
        .expect_err("reset should reject insufficient output capacity");

    assert_eq!(
        TranscodeError::InsufficientOutput {
            output_index: 0,
            required: 1,
            available: 0,
        },
        error,
    );
}

#[test]
fn test_buffered_encode_engine_reset_reports_output_index_beyond_buffer() {
    let mut encoder = TranscodeEncodeEngine::<_, _>::new(
        ResetEmittingCodec,
        ResetPassthroughHooks,
    );
    let mut output = [0_u8; 1];

    let error = encoder
        .reset(&mut output, 2)
        .expect_err("reset should reject an out-of-range output index");

    assert_eq!(
        TranscodeError::InvalidOutputIndex { index: 2, len: 1 },
        error,
    );
}

#[test]
fn test_buffered_encode_engine_max_total_output_len_reports_sum_overflow() {
    let encoder = TranscodeEncodeEngine::<_, _>::new(
        ResetEmittingCodec,
        ResetPassthroughHooks,
    );

    assert_eq!(
        Err(TranscodeError::OutputLengthOverflow),
        encoder.max_total_output_len(usize::MAX),
    );
}

#[test]
fn test_buffered_encode_engine_reset_converts_codec_reset_errors() {
    let mut encoder = TranscodeEncodeEngine::<_, _>::new(
        ResetFailCodec,
        ResetErrorMappingHooks,
    );
    let mut output = [0_u8; 1];

    let error = encoder
        .reset(&mut output, 0)
        .expect_err("reset should convert codec reset errors");

    assert_eq!(
        TranscodeError::domain(ResetFailError, CodecPhase::Reset, None),
        error,
    );
}

// ============================================================================
// Lifecycle guard wiring
// ============================================================================

#[cfg(debug_assertions)]
#[test]
#[should_panic(
    expected = "Transcoder::finish called twice without an intervening reset"
)]
fn test_buffered_encode_engine_lifecycle_rejects_double_finish() {
    let mut engine =
        TranscodeEncodeEngine::<_, _>::new(WideCodec, ExactWidthHooks);
    let mut output = [0_u8; 0];
    engine
        .finish(&mut output, 0)
        .expect("first finish should succeed for a stateless encoder");
    let _ = engine.finish(&mut output, 0);
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(
    expected = "Transcoder::transcode called after finish without an \
                intervening reset"
)]
fn test_buffered_encode_engine_lifecycle_rejects_transcode_after_finish() {
    let mut engine =
        TranscodeEncodeEngine::<_, _>::new(WideCodec, ExactWidthHooks);
    let mut output = [0_u8; 1];
    engine
        .finish(&mut output, 0)
        .expect("finish closes the logical stream");
    let _ = engine.transcode(&[1_u8], 0, &mut output, 0);
}

#[test]
fn test_buffered_encode_engine_lifecycle_allows_reuse_after_reset() {
    let mut engine =
        TranscodeEncodeEngine::<_, _>::new(WideCodec, ExactWidthHooks);
    let mut output = [0_u8; 2];
    engine
        .finish(&mut output, 0)
        .expect("first logical stream finalizes");
    engine
        .reset(&mut output, 0)
        .expect("reset reopens the engine");
    let progress = engine
        .transcode(&[1_u8], 0, &mut output, 0)
        .expect("transcode after reset");
    assert_eq!(1, progress.read());
    engine
        .finish(&mut output, 1)
        .expect("second logical stream finalizes");
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OverflowPlanningEncodeHooks;

impl TranscodeEncodeHooks<WideCodec> for OverflowPlanningEncodeHooks {
    fn max_transcode_output_len(
        &self,
        _codec: &WideCodec,
        _input_len: usize,
    ) -> Result<usize, qubit_codec::TranscodeEncodeError<WideCodec>> {
        Err(TranscodeError::output_length_overflow())
    }

    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut WideCodec,
        _value: &u8,
        input_index: usize,
    ) -> Result<
        EncodeUnencodableAction<u8>,
        qubit_codec::TranscodeEncodeError<WideCodec>,
    > {
        Err(TranscodeError::UnencodableValue { input_index })
    }
}

#[test]
fn test_buffered_encode_engine_forwards_map_error_and_capacity_failures() {
    type Engine = TranscodeEncodeEngine<WideCodec, ExactWidthHooks>;
    let encoder = Engine::new(WideCodec, ExactWidthHooks);
    let error =
        TranscodeError::<core::convert::Infallible>::invalid_input_index(1, 0);
    assert_eq!(error, Transcoder::map_error(&encoder, error));
    assert_eq!(Ok(8), Engine::max_total_output_len(&encoder, 2));

    let overflow_encoder = TranscodeEncodeEngine::<
        WideCodec,
        OverflowPlanningEncodeHooks,
    >::new(WideCodec, OverflowPlanningEncodeHooks);
    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        Transcoder::max_transcode_output_len(&overflow_encoder, 1),
    );
}
