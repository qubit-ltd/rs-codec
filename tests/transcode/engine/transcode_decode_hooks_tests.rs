// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::{DecodeAction, DecodeContext, TranscodeDecodeHooks};
use qubit_io::nz;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct UnitCodec;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("decode failed")]
struct UnitDecodeError;

unsafe impl qubit_codec::Codec for UnitCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = UnitDecodeError;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct DefaultOnlyHooks;

impl TranscodeDecodeHooks<UnitCodec> for DefaultOnlyHooks {
    type Error = UnitDecodeError;

    fn handle_decode_error(
        &mut self,
        _codec: &mut UnitCodec,
        error: UnitDecodeError,
        _context: DecodeContext,
    ) -> Result<DecodeAction<u8>, Self::Error> {
        Err(error)
    }
}

#[test]
#[should_panic(
    expected = "TranscodeDecodeHooks::map_decode_flush_error must be implemented for fallible flush codecs"
)]
fn test_transcode_decode_hooks_default_map_decode_flush_error_panics() {
    let mut hooks = DefaultOnlyHooks;
    let mut codec = UnitCodec;

    let _ = TranscodeDecodeHooks::<UnitCodec>::map_decode_flush_error(
        &mut hooks,
        &mut codec,
        UnitDecodeError,
    );
}

#[test]
fn test_transcode_decode_hooks_default_finish_is_noop() {
    let mut hooks = DefaultOnlyHooks;
    let mut codec = UnitCodec;
    let mut output = [0_u8; 1];

    let written = TranscodeDecodeHooks::<UnitCodec>::finish(&mut hooks, &mut codec, &mut output, 0)
        .expect("default finish should be a no-op");

    assert_eq!(0, written);
}

#[test]
fn test_transcode_decode_hooks_default_reset_succeeds() {
    let mut hooks = DefaultOnlyHooks;
    let mut codec = UnitCodec;

    TranscodeDecodeHooks::<UnitCodec>::reset(&mut hooks, &mut codec)
        .expect("default reset should succeed");
}
