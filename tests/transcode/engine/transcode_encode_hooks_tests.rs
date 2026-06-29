// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::{
    EncodeUnencodableAction,
    TranscodeEncodeHooks,
    TranscodeError,
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
        Err(UnitEncodeError)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct DefaultOnlyHooks;

impl TranscodeEncodeHooks<UnitCodec> for DefaultOnlyHooks {
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut UnitCodec,
        _value: &u8,
        _input_index: usize,
    ) -> Result<EncodeUnencodableAction<u8>, TranscodeError<UnitEncodeError>>
    {
        Ok(EncodeUnencodableAction::Reject)
    }
}

#[test]
fn test_transcode_encode_hooks_default_finish_is_noop() {
    let mut hooks = DefaultOnlyHooks;
    let mut codec = UnitCodec;
    let mut output = [0_u8; 1];

    let written = TranscodeEncodeHooks::<UnitCodec>::finish_hooks(
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

    TranscodeEncodeHooks::<UnitCodec>::reset_hooks(&mut hooks, &mut codec);
}
