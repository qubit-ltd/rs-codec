// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::{EncodeContext, EncodePlan, TranscodeEncodeHooks};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct UnitCodec;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("encode failed")]
struct UnitEncodeError;

unsafe impl qubit_codec::Codec for UnitCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = UnitEncodeError;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_encode_reset_units(&self) -> usize {
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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct DefaultResetHooks;

impl TranscodeEncodeHooks<UnitCodec> for DefaultOnlyHooks {
    type Error = UnitEncodeError;
    type PlanAction = ();

    fn prepare_encode(
        &mut self,
        _codec: &mut UnitCodec,
        _input_value: &u8,
        _input_index: usize,
    ) -> Result<EncodePlan<Self::PlanAction>, Self::Error> {
        Ok(EncodePlan::new(1, ()))
    }

    unsafe fn write_encode(
        &mut self,
        _codec: &mut UnitCodec,
        context: EncodeContext<'_, u8, u8>,
        _plan: EncodePlan<Self::PlanAction>,
    ) -> Result<usize, Self::Error> {
        context.output[context.output_index] = *context.input_value;
        Ok(1)
    }
}

impl TranscodeEncodeHooks<UnitCodec> for DefaultResetHooks {
    type Error = UnitEncodeError;
    type PlanAction = ();

    fn prepare_encode(
        &mut self,
        _codec: &mut UnitCodec,
        _input_value: &u8,
        _input_index: usize,
    ) -> Result<EncodePlan<Self::PlanAction>, Self::Error> {
        Ok(EncodePlan::new(1, ()))
    }

    fn map_encode_reset_error(
        &mut self,
        _codec: &mut UnitCodec,
        error: UnitEncodeError,
    ) -> Self::Error {
        error
    }

    unsafe fn write_encode(
        &mut self,
        _codec: &mut UnitCodec,
        context: EncodeContext<'_, u8, u8>,
        _plan: EncodePlan<Self::PlanAction>,
    ) -> Result<usize, Self::Error> {
        context.output[context.output_index] = *context.input_value;
        Ok(1)
    }
}

#[test]
#[should_panic(
    expected = "TranscodeEncodeHooks::map_encode_reset_error must be implemented for fallible reset codecs"
)]
fn test_transcode_encode_hooks_default_map_encode_reset_error_panics() {
    let mut hooks = DefaultOnlyHooks;
    let mut codec = UnitCodec;

    let _ = TranscodeEncodeHooks::<UnitCodec>::map_encode_reset_error(
        &mut hooks,
        &mut codec,
        UnitEncodeError,
    );
}

#[test]
fn test_transcode_encode_hooks_default_write_encode_reset_maps_errors() {
    let mut hooks = DefaultResetHooks;
    let mut codec = UnitCodec;
    let mut output = [0_u8; 1];

    let error = unsafe {
        TranscodeEncodeHooks::<UnitCodec>::write_encode_reset(
            &mut hooks,
            &mut codec,
            &mut output,
            0,
        )
    }
    .expect_err("default reset writer should map codec reset errors");

    assert_eq!(UnitEncodeError, error);
}

#[test]
fn test_transcode_encode_hooks_default_finish_is_noop() {
    let mut hooks = DefaultOnlyHooks;
    let mut codec = UnitCodec;
    let mut output = [0_u8; 1];

    let written = TranscodeEncodeHooks::<UnitCodec>::finish(&mut hooks, &mut codec, &mut output, 0)
        .expect("default finish should be a no-op");

    assert_eq!(0, written);
}
