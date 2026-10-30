// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::convert::Infallible;
use core::num::NonZeroUsize;

use qubit_codec::Codec;
use qubit_codec::CodecTranscodeConverter;
use qubit_codec::DecodeFailure;
use qubit_codec::TranscodeConvertError;
use qubit_codec::TranscodeFailure;

struct AllocationCodec<const RESET: usize, const FINISH: usize>;

impl<const RESET: usize, const FINISH: usize> Codec for AllocationCodec<RESET, FINISH> {
    type Value = u64;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 0;
    const MAX_DECODE_RESET_VALUES: usize = RESET;
    const MAX_DECODE_FINISH_VALUES: usize = FINISH;

    fn can_encode_value(&self, _value: &u64) -> bool {
        false
    }

    unsafe fn decode(&mut self, input: &[u8], index: usize) -> Result<(u64, NonZeroUsize), DecodeFailure<Infallible>> {
        debug_assert!(index < input.len());
        Ok((u64::from(input[index]), NonZeroUsize::MIN))
    }

    unsafe fn encode(&mut self, _value: &u64, _output: &mut [u8], _index: usize) -> Result<usize, Infallible> {
        unreachable!("the encodable domain is empty")
    }
}

#[test]
fn test_convert_reset_allocation_failure_returns_error_and_poisoned_state() {
    let mut converter = CodecTranscodeConverter::new(AllocationCodec::<{ usize::MAX }, 0>, AllocationCodec::<0, 0>);
    assert_eq!(Ok(0), converter.max_reset_output_len());
    assert!(matches!(
        converter.reset(&mut [], 0),
        Err(TranscodeConvertError::Failure(TranscodeFailure::AllocationFailed)),
    ));
    assert!(matches!(
        converter.finish(&mut [], 0),
        Err(TranscodeConvertError::Failure(TranscodeFailure::LifecyclePoisoned)),
    ));
}

#[test]
fn test_convert_finish_allocation_failure_returns_error_and_reset_recovers() {
    let mut converter = CodecTranscodeConverter::new(AllocationCodec::<0, { usize::MAX }>, AllocationCodec::<0, 0>);
    converter.reset(&mut [], 0).expect("zero reset bound");
    assert_eq!(Ok(0), converter.max_finish_output_len());
    assert!(matches!(
        converter.finish(&mut [], 0),
        Err(TranscodeConvertError::Failure(TranscodeFailure::AllocationFailed)),
    ));
    assert!(matches!(
        converter.transcode(&[], 0, &mut [], 0),
        Err(TranscodeConvertError::Failure(TranscodeFailure::LifecyclePoisoned)),
    ));
    converter.reset(&mut [], 0).expect("reset recovers lifecycle");
    assert!(converter.transcode(&[], 0, &mut [], 0).unwrap().is_complete());
}
