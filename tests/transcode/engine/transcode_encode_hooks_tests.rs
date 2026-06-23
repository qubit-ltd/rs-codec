// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::{
    EncodeContext,
    EncodeOutcome,
    TranscodeEncodeHooks,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct UnitCodec;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("encode failed")]
struct UnitEncodeError;

impl qubit_codec::Codec for UnitCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = UnitEncodeError;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_ENCODE_RESET_UNITS: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::CodecDecodeFailure<Self::DecodeError>,
    > {
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

    unsafe fn encode_reset(
        &mut self,
        _output: &mut [u8],
        _index: usize,
    ) -> Result<usize, Self::EncodeError> {
        Err(UnitEncodeError)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct DefaultOnlyHooks;

impl TranscodeEncodeHooks<UnitCodec> for DefaultOnlyHooks {
    type Error = UnitEncodeError;

    fn encode_value(
        &mut self,
        _codec: &mut UnitCodec,
        context: EncodeContext<'_, u8, u8>,
    ) -> Result<EncodeOutcome, Self::Error> {
        if context.available_output() < 1 {
            return Ok(EncodeOutcome::NeedOutput {
                required: core::num::NonZeroUsize::MIN,
            });
        }
        context.output[context.output_index] = *context.input_value;
        Ok(EncodeOutcome::Consumed { written: 1 })
    }
}

#[test]
fn test_transcode_encode_hooks_default_finish_is_noop() {
    let mut hooks = DefaultOnlyHooks;
    let mut codec = UnitCodec;
    let mut output = [0_u8; 1];

    let written = TranscodeEncodeHooks::<UnitCodec>::finish(
        &mut hooks,
        &mut codec,
        &mut output,
        0,
    )
    .expect("default finish should be a no-op");

    assert_eq!(0, written);
}

#[test]
fn test_transcode_encode_hooks_default_before_reset_is_noop() {
    let mut hooks = DefaultOnlyHooks;
    let mut codec = UnitCodec;

    TranscodeEncodeHooks::<UnitCodec>::before_reset(&mut hooks, &mut codec);
}
