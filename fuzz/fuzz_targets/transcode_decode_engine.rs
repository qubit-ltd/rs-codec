// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![no_main]

use core::num::NonZeroUsize;

use libfuzzer_sys::fuzz_target;
use qubit_codec::Codec;
use qubit_codec::DecodeFailure;
use qubit_codec::TranscodeDecodeErrorOf;
use qubit_codec::engine::DecodeContext;
use qubit_codec::engine::DecodeIncompleteAction;
use qubit_codec::engine::DecodeInvalidAction;
use qubit_codec::engine::TranscodeDecodeEngine;
use qubit_codec::engine::TranscodeDecodeHooks;

const MAX_INPUT_LEN: usize = 4 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DecodeError {
    Invalid,
    Incomplete,
}

#[derive(Clone, Copy)]
enum Policy {
    Reject,
    Skip,
    Emit,
}

impl Policy {
    fn from_byte(byte: u8) -> Self {
        match byte % 3 {
            0 => Self::Reject,
            1 => Self::Skip,
            _ => Self::Emit,
        }
    }
}

#[derive(Default)]
struct ProtocolCodec;

impl Codec for ProtocolCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = DecodeError;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
        match input[index] {
            0xff => Err(DecodeFailure::invalid(DecodeError::Invalid, NonZeroUsize::MIN)),
            0xfe if input.len() - index == 1 => Err(DecodeFailure::incomplete_with_source(
                DecodeError::Incomplete,
                NonZeroUsize::new(2).expect("two is non-zero"),
            )),
            byte => Ok((byte, NonZeroUsize::MIN)),
        }
    }

    unsafe fn decode_eof(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
        // SAFETY: decode_eof has the same input preconditions as decode.
        unsafe { self.decode(input, index) }
    }

    unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Self::EncodeError> {
        output[index] = *value;
        Ok(1)
    }
}

struct ProtocolHooks {
    invalid: Policy,
    incomplete: Policy,
}

impl TranscodeDecodeHooks<ProtocolCodec> for ProtocolHooks {
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut ProtocolCodec,
        error: &DecodeError,
        consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, TranscodeDecodeErrorOf<ProtocolCodec>> {
        assert_eq!(DecodeError::Invalid, *error);
        let consumed = consumed.expect("invalid token consumes one unit");
        Ok(match self.invalid {
            Policy::Reject => DecodeInvalidAction::Reject,
            Policy::Skip => DecodeInvalidAction::Skip { consumed },
            Policy::Emit => DecodeInvalidAction::Emit { value: b'?', consumed },
        })
    }

    fn handle_incomplete_decode(
        &mut self,
        _codec: &mut ProtocolCodec,
        source: Option<&DecodeError>,
        required_total: NonZeroUsize,
        _context: DecodeContext,
    ) -> Result<DecodeIncompleteAction<u8>, TranscodeDecodeErrorOf<ProtocolCodec>> {
        assert_eq!(Some(&DecodeError::Incomplete), source);
        assert_eq!(2, required_total.get());
        Ok(match self.incomplete {
            Policy::Reject => DecodeIncompleteAction::Reject,
            Policy::Skip => DecodeIncompleteAction::Skip,
            Policy::Emit => DecodeIncompleteAction::Emit { value: b'?' },
        })
    }
}

enum Oracle {
    Complete(Vec<u8>),
    Rejected {
        position: usize,
        prefix: Vec<u8>,
        incomplete: bool,
    },
}

fn reference(payload: &[u8], invalid: Policy, incomplete: Policy) -> Oracle {
    let mut output = Vec::with_capacity(payload.len());
    for (position, &byte) in payload.iter().enumerate() {
        match byte {
            0xff => match invalid {
                Policy::Reject => {
                    return Oracle::Rejected {
                        position,
                        prefix: output,
                        incomplete: false,
                    };
                }
                Policy::Skip => {}
                Policy::Emit => output.push(b'?'),
            },
            0xfe if position + 1 == payload.len() => match incomplete {
                Policy::Reject => {
                    return Oracle::Rejected {
                        position,
                        prefix: output,
                        incomplete: true,
                    };
                }
                Policy::Skip => {}
                Policy::Emit => output.push(b'?'),
            },
            _ => output.push(byte),
        }
    }
    Oracle::Complete(output)
}

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_INPUT_LEN)];
    let control = &data[..data.len().min(8)];
    let raw_payload = &data[control.len()..];
    let byte = |index: usize| control.get(index).copied().unwrap_or(0);
    let invalid = Policy::from_byte(byte(0));
    let incomplete = Policy::from_byte(byte(1));
    let stride = usize::from(byte(2) % 7) + 1;
    let eof_cut = raw_payload
        .len()
        .saturating_sub(usize::from(byte(3)) % (raw_payload.len() + 1));
    let payload = &raw_payload[..eof_cut];
    let expected = reference(payload, invalid, incomplete);
    let mut decoder = TranscodeDecodeEngine::new(ProtocolCodec, ProtocolHooks { invalid, incomplete });
    let mut reset = [];
    decoder.reset(&mut reset, 0).expect("stateless reset succeeds");

    let mut cursor = 0usize;
    let mut visible = 0usize;
    let mut output = Vec::new();
    let mut output_window = usize::from(byte(4) % 2);
    let mut eof = false;
    let mut steps = 0usize;
    let step_limit = 16 * (payload.len() + 1) + 32;

    loop {
        steps += 1;
        assert!(steps <= step_limit, "decoder exceeded linear step bound");
        let previous_visible = visible;
        let previous_eof = eof;
        let previous_window = output_window;
        let end = visible;
        let window = if output_window == 0 { 0 } else { 1 };
        let mut slot = [0_u8; 31];
        let result = if eof {
            decoder.transcode_eof(&payload[..end], cursor, &mut slot[..window], 0)
        } else {
            decoder.transcode(&payload[..end], cursor, &mut slot[..window], 0)
        };

        match result {
            Ok(progress) => {
                progress
                    .validate(cursor, end - cursor, 0, window)
                    .expect("valid decoder progress");
                cursor += progress.read();
                output.extend_from_slice(&slot[..progress.written()]);
                if progress.is_need_output() {
                    if window == 0 {
                        output_window = 1;
                    } else {
                        assert!(
                            progress.read() > 0 || progress.written() > 0,
                            "NeedOutput with one slot must follow committed progress"
                        );
                    }
                } else if progress.is_need_input() {
                    assert!(!eof, "EOF decoding cannot request more input for this protocol");
                    if visible < payload.len() {
                        visible = (visible + stride).min(payload.len());
                    } else {
                        eof = true;
                    }
                } else {
                    assert!(progress.is_complete());
                    assert_eq!(cursor, visible);
                    if visible < payload.len() {
                        visible = (visible + stride).min(payload.len());
                    } else if !eof {
                        eof = true;
                    } else {
                        break;
                    }
                }
                assert!(
                    progress.read() > 0
                        || visible > previous_visible
                        || eof != previous_eof
                        || output_window > previous_window
                        || progress.is_complete()
                );
            }
            Err(error) => {
                let (position, prefix, is_incomplete) = match &expected {
                    Oracle::Rejected {
                        position,
                        prefix,
                        incomplete,
                    } => (*position, prefix, *incomplete),
                    Oracle::Complete(_) => panic!("unexpected decode rejection: {error:?}"),
                };
                assert!(error.is_domain(), "protocol rejects with domain source: {error:?}");
                let domain = error.domain_error_ref().expect("domain error");
                assert_eq!(Some(position), domain.input_index());
                assert_eq!(
                    Some(if is_incomplete {
                        &DecodeError::Incomplete
                    } else {
                        &DecodeError::Invalid
                    }),
                    error.domain_ref()
                );
                assert_eq!(prefix, &output);
                return;
            }
        }
    }

    match expected {
        Oracle::Complete(expected) => {
            assert_eq!(payload.len(), cursor);
            assert_eq!(expected, output);
            let mut finish = [0_u8; 31];
            assert_eq!(0, decoder.finish(&mut finish, 0).expect("finish succeeds"));
        }
        Oracle::Rejected { .. } => panic!("decoder accepted a rejected protocol token"),
    }
});
