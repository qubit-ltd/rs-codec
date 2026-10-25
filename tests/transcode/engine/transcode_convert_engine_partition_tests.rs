// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::cell::Cell;
use core::cell::RefCell;
use core::convert::Infallible;
use core::num::NonZeroUsize;
use std::rc::Rc;

use qubit_codec as codec;
use qubit_codec::Codec;
use qubit_codec::TranscodeConvertError;
use qubit_codec::TranscodeDomainError;
use qubit_codec::TranscodeFailure;
use qubit_codec::TranscodeStatus;
use qubit_codec::engine::DecodeContext;
use qubit_codec::engine::DecodeIncompleteAction;
use qubit_codec::engine::DecodeInvalidAction;
use qubit_codec::engine::EncodeContext;
use qubit_codec::engine::EncodeUnencodableAction;
use qubit_codec::engine::TranscodeConvertEngine;
use qubit_codec::engine::TranscodeDecodeEngine;
use qubit_codec::engine::TranscodeDecodeHooks;
use qubit_codec::engine::TranscodeEncodeHooks;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("invalid source token")]
struct InvalidToken;

#[derive(Clone, Copy, Debug, Default)]
struct PairSource;

impl Codec for PairSource {
    type Value = u8;
    type Unit = u8;
    type DecodeError = InvalidToken;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 2;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, NonZeroUsize), codec::DecodeFailure<InvalidToken>> {
        if input[index] == 0xff {
            return Err(codec::DecodeFailure::invalid(InvalidToken, NonZeroUsize::MIN));
        }
        if input.len() - index < 2 {
            return Err(codec::DecodeFailure::incomplete(NonZeroUsize::new(2).unwrap()));
        }
        Ok((
            input[index].wrapping_add(input[index + 1]),
            NonZeroUsize::new(2).unwrap(),
        ))
    }

    unsafe fn encode(&mut self, _value: &u8, _output: &mut [u8], _index: usize) -> Result<usize, Infallible> {
        unreachable!()
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct VariableTarget {
    markers: bool,
}

impl Codec for VariableTarget {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 2;
    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_RESET_UNITS: usize = 1;
    const MAX_ENCODE_FINISH_UNITS: usize = 1;

    fn can_encode_value(&self, value: &u8) -> bool {
        *value != 0xfe
    }
    fn encode_len(&self, value: &u8) -> usize {
        if value & 1 == 0 { 2 } else { 1 }
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, NonZeroUsize), codec::DecodeFailure<Infallible>> {
        Ok((input[index], NonZeroUsize::MIN))
    }

    unsafe fn encode_reset(&mut self, output: &mut [u8], index: usize) -> Result<usize, Infallible> {
        if self.markers {
            output[index] = 0xa0;
            Ok(1)
        } else {
            Ok(0)
        }
    }

    unsafe fn encode_finish(&mut self, output: &mut [u8], index: usize) -> Result<usize, Infallible> {
        if self.markers {
            output[index] = 0xa1;
            Ok(1)
        } else {
            Ok(0)
        }
    }

    unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Infallible> {
        output[index] = *value;
        if self.encode_len(value) == 2 {
            output[index + 1] = !*value;
            Ok(2)
        } else {
            Ok(1)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InvalidMode {
    Reject,
    Skip,
    Emit,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IncompleteMode {
    Reject,
    Skip,
    Emit,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UnencodableMode {
    Reject,
    Skip,
    Replace,
}

#[derive(Clone, Debug)]
struct DecodePolicy {
    invalid: InvalidMode,
    incomplete: IncompleteMode,
    decisions: Rc<Cell<usize>>,
}
impl TranscodeDecodeHooks<PairSource> for DecodePolicy {
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut PairSource,
        _error: &InvalidToken,
        consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, codec::TranscodeDecodeErrorOf<PairSource>> {
        self.decisions.set(self.decisions.get() + 1);
        let consumed = consumed.unwrap_or(NonZeroUsize::MIN);
        Ok(match self.invalid {
            InvalidMode::Reject => DecodeInvalidAction::Reject,
            InvalidMode::Skip => DecodeInvalidAction::Skip { consumed },
            InvalidMode::Emit => DecodeInvalidAction::Emit { value: 7, consumed },
        })
    }
    fn handle_incomplete_decode(
        &mut self,
        _codec: &mut PairSource,
        _source: Option<&InvalidToken>,
        _required: NonZeroUsize,
        _context: DecodeContext,
    ) -> Result<DecodeIncompleteAction<u8>, codec::TranscodeDecodeErrorOf<PairSource>> {
        self.decisions.set(self.decisions.get() + 1);
        Ok(match self.incomplete {
            IncompleteMode::Reject => DecodeIncompleteAction::Reject,
            IncompleteMode::Skip => DecodeIncompleteAction::Skip,
            IncompleteMode::Emit => DecodeIncompleteAction::Emit { value: 9 },
        })
    }
}

#[derive(Clone, Debug)]
struct EncodePolicy {
    mode: UnencodableMode,
    decisions: Rc<Cell<usize>>,
}
impl TranscodeEncodeHooks<VariableTarget> for EncodePolicy {
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut VariableTarget,
        _context: &EncodeContext<'_, u8>,
    ) -> Result<EncodeUnencodableAction<u8>, codec::TranscodeEncodeErrorOf<VariableTarget>> {
        self.decisions.set(self.decisions.get() + 1);
        Ok(match self.mode {
            UnencodableMode::Reject => EncodeUnencodableAction::Reject,
            UnencodableMode::Skip => EncodeUnencodableAction::Skip,
            UnencodableMode::Replace => EncodeUnencodableAction::Replace { value: 3 },
        })
    }
}

type PartitionEngine = TranscodeConvertEngine<PairSource, VariableTarget, DecodePolicy, EncodePolicy>;
type DecisionCounter = Rc<Cell<usize>>;
type EngineWithCounters = (PartitionEngine, DecisionCounter, DecisionCounter);

/// Records decode hook contexts while applying a fixed incomplete-input policy.
struct ContextProbe {
    incomplete: IncompleteMode,
    contexts: Rc<RefCell<Vec<DecodeContext>>>,
}

impl TranscodeDecodeHooks<PairSource> for ContextProbe {
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut PairSource,
        _error: &InvalidToken,
        consumed: Option<NonZeroUsize>,
        context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, codec::TranscodeDecodeErrorOf<PairSource>> {
        self.contexts.borrow_mut().push(context);
        Ok(DecodeInvalidAction::Skip {
            consumed: consumed.unwrap_or(NonZeroUsize::MIN),
        })
    }

    fn handle_incomplete_decode(
        &mut self,
        _codec: &mut PairSource,
        _source: Option<&InvalidToken>,
        _required: NonZeroUsize,
        context: DecodeContext,
    ) -> Result<DecodeIncompleteAction<u8>, codec::TranscodeDecodeErrorOf<PairSource>> {
        self.contexts.borrow_mut().push(context);
        Ok(match self.incomplete {
            IncompleteMode::Reject => DecodeIncompleteAction::Reject,
            IncompleteMode::Skip => DecodeIncompleteAction::Skip,
            IncompleteMode::Emit => DecodeIncompleteAction::Emit { value: 9 },
        })
    }
}

fn context_probe(mode: IncompleteMode) -> (ContextProbe, Rc<RefCell<Vec<DecodeContext>>>) {
    let contexts = Rc::new(RefCell::new(Vec::new()));
    let hooks = ContextProbe {
        incomplete: mode,
        contexts: Rc::clone(&contexts),
    };
    (hooks, contexts)
}

fn engine(invalid: InvalidMode, incomplete: IncompleteMode, unencodable: UnencodableMode) -> PartitionEngine {
    engine_with_markers(invalid, incomplete, unencodable, true).0
}

fn engine_with_markers(
    invalid: InvalidMode,
    incomplete: IncompleteMode,
    unencodable: UnencodableMode,
    markers: bool,
) -> EngineWithCounters {
    let decode_decisions = Rc::new(Cell::new(0));
    let encode_decisions = Rc::new(Cell::new(0));
    (
        TranscodeConvertEngine::new(
            PairSource,
            VariableTarget { markers },
            DecodePolicy {
                invalid,
                incomplete,
                decisions: Rc::clone(&decode_decisions),
            },
            EncodePolicy {
                mode: unencodable,
                decisions: Rc::clone(&encode_decisions),
            },
        ),
        decode_decisions,
        encode_decisions,
    )
}

fn partitions(payload: &[u8]) -> Vec<(usize, Vec<usize>)> {
    if payload.is_empty() {
        return vec![(0, vec![])];
    }
    let mut all = Vec::new();
    for mask in 0..(1usize << payload.len().saturating_sub(1)) {
        let mut ends = Vec::new();
        for cut in 1..payload.len() {
            if mask & (1usize << (cut - 1)) != 0 {
                ends.push(cut);
            }
        }
        ends.push(payload.len());
        all.push((mask, ends));
    }
    all
}

#[derive(Debug, Eq, PartialEq)]
enum ModelFailure {
    Decode { source: InvalidToken, input_index: usize },
    EncodeUnencodable { input_index: usize, value: u8 },
    Incomplete { input_index: usize },
}

#[derive(Debug, Eq, PartialEq)]
struct ModelResult {
    output: Vec<u8>,
    decode_decisions: usize,
    encode_decisions: usize,
}

// This parser/model is intentionally independent of Codec and
// TranscodeConvertEngine.
fn semantic_oracle(
    payload: &[u8],
    invalid: InvalidMode,
    incomplete: IncompleteMode,
    unencodable: UnencodableMode,
    markers: bool,
) -> Result<ModelResult, ModelFailure> {
    let mut output = Vec::new();
    let mut decode_decisions = 0;
    let mut encode_decisions = 0;
    let mut index = 0;
    let mut emit_value = |value: u8, input_index: usize, output: &mut Vec<u8>| {
        let value = if value == 0xfe {
            encode_decisions += 1;
            match unencodable {
                UnencodableMode::Reject => {
                    return Err(ModelFailure::EncodeUnencodable { input_index, value });
                }
                UnencodableMode::Skip => return Ok(()),
                UnencodableMode::Replace => 3,
            }
        } else {
            value
        };
        output.push(value);
        if value & 1 == 0 {
            output.push(!value);
        }
        Ok(())
    };

    if markers {
        output.push(0xa0);
    }
    while index < payload.len() {
        if payload[index] == 0xff {
            decode_decisions += 1;
            match invalid {
                InvalidMode::Reject => {
                    return Err(ModelFailure::Decode {
                        source: InvalidToken,
                        input_index: index,
                    });
                }
                InvalidMode::Skip => index += 1,
                InvalidMode::Emit => {
                    emit_value(7, index, &mut output)?;
                    index += 1;
                }
            }
            continue;
        }
        if payload.len() - index < 2 {
            decode_decisions += 1;
            match incomplete {
                IncompleteMode::Reject => {
                    return Err(ModelFailure::Incomplete { input_index: index });
                }
                IncompleteMode::Skip => index = payload.len(),
                IncompleteMode::Emit => {
                    emit_value(9, index, &mut output)?;
                    index = payload.len();
                }
            }
            continue;
        }
        let value = payload[index].wrapping_add(payload[index + 1]);
        emit_value(value, index, &mut output)?;
        index += 2;
    }
    if markers {
        output.push(0xa1);
    }
    Ok(ModelResult {
        output,
        decode_decisions,
        encode_decisions,
    })
}

fn drive(payload: &[u8], ends: &[usize], engine: PartitionEngine) -> Result<(Vec<u8>, usize), String> {
    drive_engine(payload, ends, engine)
}

fn drive_engine(payload: &[u8], ends: &[usize], mut engine: PartitionEngine) -> Result<(Vec<u8>, usize), String> {
    let mut output = Vec::new();
    let mut pending = Vec::new();
    let mut offset = 0;
    let mut chunk_end = 0;
    let mut part = 0;
    let mut eof_done = false;
    let mut eof_requested = false;
    let mut next_window_override = None;
    let mut steps = 0;
    let bound = 64 * (payload.len() + 1) + 64;
    let mut reset = vec![0; engine.max_reset_output_len().unwrap()];
    let reset_written = engine.reset(&mut reset, 0).map_err(|e| format!("reset: {e:?}"))?;
    pending.extend_from_slice(&reset[..reset_written]);
    loop {
        steps += 1;
        assert!(
            steps <= bound,
            "bounded driver exceeded: payload={payload:?}, ends={ends:?}, offset={offset}"
        );
        if !pending.is_empty() {
            output.append(&mut pending);
        }
        if chunk_end == offset && part < ends.len() {
            chunk_end = ends[part];
            part += 1;
        }
        let visible = &payload[..chunk_end];
        let window = next_window_override.take().unwrap_or([0, 1, 2, 3, 5][(steps - 1) % 5]);
        let mut out = vec![0; window];
        if eof_done {
            break;
        }
        let calling_eof = eof_requested || (part == ends.len() && offset == payload.len());
        let result = if calling_eof {
            engine.transcode_eof(visible, offset, &mut out, 0)
        } else {
            engine.transcode(visible, offset, &mut out, 0)
        };
        match result {
            Ok(progress) => {
                offset += progress.read();
                pending.extend_from_slice(&out[..progress.written()]);
                match progress.status() {
                    TranscodeStatus::Complete => {
                        if calling_eof {
                            eof_done = true;
                        }
                    }
                    TranscodeStatus::NeedInput { .. } => {
                        if part < ends.len() {
                            chunk_end = ends[part];
                            part += 1;
                        } else {
                            eof_requested = true;
                        }
                    }
                    TranscodeStatus::NeedOutput { required, .. } => {
                        // Keep the current visible input, cursor, and EOF state;
                        // retry only after the next bounded driver iteration
                        // has committed this progress and supplied `required`.
                        next_window_override = Some(required.get());
                    }
                }
            }
            Err(error) => return Err(format!("{error:?}")),
        }
        if eof_done {
            output.append(&mut pending);
            let mut finish = vec![0; engine.max_finish_output_len().unwrap()];
            let n = engine.finish(&mut finish, 0).map_err(|e| format!("finish: {e:?}"))?;
            output.extend_from_slice(&finish[..n]);
            break;
        }
    }
    Ok((output, steps))
}

fn drive_observed(
    payload: &[u8],
    ends: &[usize],
    invalid: InvalidMode,
    incomplete: IncompleteMode,
    unencodable: UnencodableMode,
    markers: bool,
) -> Result<(Vec<u8>, usize, usize, usize), String> {
    let (engine, decode_calls, encode_calls) = engine_with_markers(invalid, incomplete, unencodable, markers);
    let result = drive_engine(payload, ends, engine)?;
    Ok((result.0, result.1, decode_calls.get(), encode_calls.get()))
}

fn drive_large_buffer(
    payload: &[u8],
    invalid: InvalidMode,
    incomplete: IncompleteMode,
    unencodable: UnencodableMode,
    markers: bool,
) -> Result<Vec<u8>, String> {
    let (mut engine, _, _) = engine_with_markers(invalid, incomplete, unencodable, markers);
    let capacity = engine
        .max_total_output_len(payload.len())
        .map_err(|e| format!("bound: {e:?}"))?;
    let mut output = vec![0; capacity];
    let written = engine
        .transcode_complete_into(payload, &mut output)
        .map_err(|e| format!("complete: {e:?}"))?;
    output.truncate(written);
    Ok(output)
}

#[test]
fn test_partition_normal_values_and_backpressure() {
    let payload = [1, 1, 3, 3, 5, 5];
    for markers in [false, true] {
        let expected = semantic_oracle(
            &payload,
            InvalidMode::Reject,
            IncompleteMode::Reject,
            UnencodableMode::Reject,
            markers,
        )
        .unwrap()
        .output;
        let large = drive_large_buffer(
            &payload,
            InvalidMode::Reject,
            IncompleteMode::Reject,
            UnencodableMode::Reject,
            markers,
        )
        .unwrap();
        assert_eq!(large, expected);
        for (mask, ends) in partitions(&payload) {
            let (configured_engine, _, _) = engine_with_markers(
                InvalidMode::Reject,
                IncompleteMode::Reject,
                UnencodableMode::Reject,
                markers,
            );
            let (actual, steps) = drive(&payload, &ends, configured_engine).unwrap_or_else(|e| {
                panic!(
                    "payload={payload:?}, markers={markers}, mask={mask}, \
                         ends={ends:?}: {e}"
                )
            });
            assert!(steps <= 64 * (payload.len() + 1) + 64);
            assert_eq!(actual, expected, "markers={markers}, mask={mask}, ends={ends:?}");
            assert_eq!(
                actual, large,
                "large-buffer comparison, markers={markers}, mask={mask}, ends={ends:?}"
            );
        }
    }
}

#[test]
fn test_partition_invalid_and_unencodable_policies() {
    let payload = [0xff, 0, 0, 0xfe, 0, 2, 2];
    for markers in [false, true] {
        for invalid in [InvalidMode::Skip, InvalidMode::Emit] {
            for unencodable in [UnencodableMode::Skip, UnencodableMode::Replace] {
                for (mask, ends) in partitions(&payload) {
                    let model =
                        semantic_oracle(&payload, invalid, IncompleteMode::Reject, unencodable, markers).unwrap();
                    let large =
                        drive_large_buffer(&payload, invalid, IncompleteMode::Reject, unencodable, markers).unwrap();
                    assert_eq!(
                        large, model.output,
                        "large-buffer oracle, markers={markers}, invalid={invalid:?}, \
                         unencodable={unencodable:?}"
                    );
                    let result = drive_observed(&payload, &ends, invalid, IncompleteMode::Reject, unencodable, markers);
                    let (actual, _, decode_calls, encode_calls) = result.unwrap_or_else(|e| {
                        panic!(
                            "markers={markers}, invalid={invalid:?}, \
                             unencodable={unencodable:?}, mask={mask}, ends={ends:?}: {e}"
                        )
                    });
                    let expected = model;
                    assert_eq!(
                        actual, expected.output,
                        "markers={markers}, invalid={invalid:?}, unencodable={unencodable:?}, \
                         mask={mask}, ends={ends:?}"
                    );
                    assert_eq!(decode_calls, expected.decode_decisions, "decode decisions, mask={mask}");
                    assert_eq!(encode_calls, expected.encode_decisions, "encode decisions, mask={mask}");
                    assert_eq!(actual, large, "large-buffer comparison, markers={markers}, mask={mask}");
                }
            }
        }
    }
    let (mut decode_engine, decode_calls, _) = engine_with_markers(
        InvalidMode::Reject,
        IncompleteMode::Reject,
        UnencodableMode::Reject,
        true,
    );
    let mut output = [0; 16];
    decode_engine.reset(&mut output, 0).unwrap();
    let error = decode_engine
        .transcode(&[1, 1, 0xff, 0], 0, &mut output, 0)
        .unwrap_err();
    assert_eq!(
        semantic_oracle(
            &[1, 1, 0xff, 0],
            InvalidMode::Reject,
            IncompleteMode::Reject,
            UnencodableMode::Reject,
            true,
        ),
        Err(ModelFailure::Decode {
            source: InvalidToken,
            input_index: 2,
        })
    );
    assert!(matches!(
        error,
        TranscodeConvertError::DecodeDomain(TranscodeDomainError::Main {
            source: InvalidToken,
            input_index: 2,
            ..
        })
    ));
    assert_eq!(decode_calls.get(), 1, "invalid decode policy decision count");

    let (mut encode_engine, _, encode_calls) = engine_with_markers(
        InvalidMode::Reject,
        IncompleteMode::Reject,
        UnencodableMode::Reject,
        true,
    );
    encode_engine.reset(&mut output, 0).unwrap();
    let error = encode_engine.transcode(&[0xfe, 0], 0, &mut output, 0).unwrap_err();
    assert_eq!(
        semantic_oracle(
            &[0xfe, 0],
            InvalidMode::Reject,
            IncompleteMode::Reject,
            UnencodableMode::Reject,
            true,
        ),
        Err(ModelFailure::EncodeUnencodable {
            input_index: 0,
            value: 254,
        })
    );
    assert!(matches!(
        error,
        TranscodeConvertError::Unencodable {
            input_index: 0,
            value: Some(254)
        }
    ));
    assert_eq!(encode_calls.get(), 1, "unencodable encode policy decision count");
}

#[test]
fn test_partition_incomplete_eof_policies() {
    let payload = [1, 2, 3];
    for markers in [false, true] {
        for mode in [IncompleteMode::Skip, IncompleteMode::Emit] {
            for (mask, ends) in partitions(&payload) {
                let expected =
                    semantic_oracle(&payload, InvalidMode::Reject, mode, UnencodableMode::Reject, markers).unwrap();
                let large =
                    drive_large_buffer(&payload, InvalidMode::Reject, mode, UnencodableMode::Reject, markers).unwrap();
                let (actual, _, decode_calls, encode_calls) = drive_observed(
                    &payload,
                    &ends,
                    InvalidMode::Reject,
                    mode,
                    UnencodableMode::Reject,
                    markers,
                )
                .unwrap_or_else(|e| {
                    panic!(
                        "markers={markers}, mode={mode:?}, mask={mask}, \
                         ends={ends:?}: {e}"
                    )
                });
                assert_eq!(
                    actual, expected.output,
                    "markers={markers}, mode={mode:?}, mask={mask}, ends={ends:?}"
                );
                assert_eq!(
                    actual, large,
                    "large-buffer comparison, markers={markers}, mode={mode:?}, mask={mask}"
                );
                assert_eq!(decode_calls, expected.decode_decisions);
                assert_eq!(encode_calls, expected.encode_decisions);
            }
        }
    }
    assert!(
        drive(
            &payload,
            &[3],
            engine(InvalidMode::Reject, IncompleteMode::Reject, UnencodableMode::Reject)
        )
        .is_err()
    );
    assert_eq!(
        semantic_oracle(
            &payload,
            InvalidMode::Reject,
            IncompleteMode::Reject,
            UnencodableMode::Reject,
            true,
        ),
        Err(ModelFailure::Incomplete { input_index: 2 })
    );
    let (mut reject_engine, reject_calls, _) = engine_with_markers(
        InvalidMode::Reject,
        IncompleteMode::Reject,
        UnencodableMode::Reject,
        true,
    );
    let mut output = [0; 16];
    reject_engine.reset(&mut output, 0).unwrap();
    assert_eq!(
        reject_engine.transcode_eof(&payload, 0, &mut output, 0).unwrap_err(),
        TranscodeConvertError::Failure(TranscodeFailure::incomplete_input(2, 2, 1))
    );
    assert_eq!(reject_calls.get(), 1, "incomplete reject decision count");
}

#[test]
fn test_partition_empty_input_and_lifecycle() {
    // Empty input still drives reset/finish and explicitly drains lifecycle output.
    let (empty, _) = drive(
        &[],
        &[],
        engine(InvalidMode::Reject, IncompleteMode::Reject, UnencodableMode::Reject),
    )
    .unwrap();
    assert_eq!(empty, [0xa0, 0xa1]);

    let (mut no_markers, _, _) = engine_with_markers(
        InvalidMode::Reject,
        IncompleteMode::Reject,
        UnencodableMode::Reject,
        false,
    );
    let mut no_marker_scratch = [0; 2];
    assert_eq!(no_markers.reset(&mut no_marker_scratch, 0).unwrap(), 0);
    let progress = no_markers.transcode_eof(&[], 0, &mut [], 0).unwrap();
    assert_eq!(progress.read(), 0);
    assert_eq!(progress.written(), 0);
    assert_eq!(no_markers.finish(&mut no_marker_scratch, 0).unwrap(), 0);

    // A decoded value retained by backpressure is drained by an empty visible-input
    // call.
    let mut draining = engine(InvalidMode::Reject, IncompleteMode::Reject, UnencodableMode::Reject);
    let mut lifecycle = [0; 2];
    assert_eq!(draining.reset(&mut lifecycle, 0).unwrap(), 1);
    let payload = [1, 1];
    let first = draining.transcode(&payload, 0, &mut [], 0).unwrap();
    assert!(matches!(
        first.status(),
        TranscodeStatus::NeedOutput { required } if required.get() == 2
    ));
    assert_eq!(first.read(), 2);
    assert_eq!(first.written(), 0);
    let mut drained = [0; 2];
    let empty_drain = draining.transcode(&payload, first.read(), &mut drained, 0).unwrap();
    assert_eq!(empty_drain.read(), 0);
    assert_eq!(empty_drain.written(), 2);
    assert_eq!(drained, [2, !2]);
    assert_eq!(
        draining
            .transcode_eof(&payload, payload.len(), &mut [], 0)
            .unwrap()
            .read(),
        0
    );
    let mut finish = [0; 1];
    assert_eq!(draining.finish(&mut finish, 0).unwrap(), 1);
    assert_eq!(finish, [0xa1]);
    let payload = [1, 1];
    for (mask, ends) in partitions(&payload) {
        let (actual, _) = drive(
            &payload,
            &ends,
            engine(InvalidMode::Reject, IncompleteMode::Reject, UnencodableMode::Reject),
        )
        .unwrap_or_else(|e| panic!("mask={mask}, ends={ends:?}: {e}"));
        assert_eq!(actual, [0xa0, 2, !2, 0xa1]);
    }
}

#[test]
fn test_decode_hook_context_counts_target_units_in_converter() {
    let (hooks, contexts) = context_probe(IncompleteMode::Skip);
    let mut engine = TranscodeConvertEngine::new(
        PairSource,
        VariableTarget { markers: false },
        hooks,
        EncodePolicy {
            mode: UnencodableMode::Skip,
            decisions: Rc::new(Cell::new(0)),
        },
    );
    engine.reset(&mut [0_u8; 2], 0).unwrap();
    let mut output = [0_u8; 8];
    let progress = engine.transcode(&[99, 0, 2, 0xff, 0], 1, &mut output, 3).unwrap();
    assert_eq!(progress.read(), 3);
    assert_eq!(progress.written(), 2);
    let recorded = contexts.borrow();
    assert_eq!(recorded.len(), 1);
    let context = recorded[0];
    assert_eq!(context.input_start(), 1);
    assert_eq!(context.input_index(), 3);
    assert_eq!(context.output_start(), 3);
    assert_eq!(context.output_index(), 5);
    assert_eq!(context.output_written(), 2);
    assert_eq!(&output[3..5], &[2, !2_u8]);
}

#[test]
fn test_converter_eof_hook_can_run_without_target_capacity() {
    for mode in [IncompleteMode::Skip, IncompleteMode::Emit] {
        let (hooks, contexts) = context_probe(mode);
        let mut engine = TranscodeConvertEngine::new(
            PairSource,
            VariableTarget { markers: false },
            hooks,
            EncodePolicy {
                mode: UnencodableMode::Skip,
                decisions: Rc::new(Cell::new(0)),
            },
        );
        engine.reset(&mut [0_u8; 2], 0).unwrap();
        let progress = engine.transcode_eof(&[5], 0, &mut [], 0).unwrap();
        assert_eq!(progress.read(), 1);
        assert_eq!(progress.written(), 0);
        assert_eq!(contexts.borrow().len(), 1);
        if mode == IncompleteMode::Skip {
            assert_eq!(progress.status(), TranscodeStatus::Complete);
        } else {
            assert!(matches!(progress.status(), TranscodeStatus::NeedOutput { .. }));
            let mut output = [0_u8; 2];
            let drained = engine.transcode_eof(&[], 0, &mut output, 0).unwrap();
            assert_eq!(drained.read(), 0);
            assert_eq!(drained.written(), 1);
            assert_eq!(drained.status(), TranscodeStatus::Complete);
            assert_eq!(output[0], 9);
            assert_eq!(contexts.borrow().len(), 1);
        }
    }
}

#[test]
fn test_decoder_defers_eof_hook_until_a_value_slot_is_available() {
    let (hooks, contexts) = context_probe(IncompleteMode::Skip);
    let mut engine = TranscodeDecodeEngine::new(PairSource, hooks);
    engine.reset(&mut [0_u8; 1], 0).unwrap();
    let progress = engine.transcode_eof(&[5], 0, &mut [], 0).unwrap();
    assert_eq!(progress.read(), 0);
    assert_eq!(progress.written(), 0);
    assert!(matches!(progress.status(), TranscodeStatus::NeedOutput { .. }));
    assert!(contexts.borrow().is_empty());
    let progress = engine.transcode_eof(&[5], 0, &mut [0_u8; 1], 0).unwrap();
    assert_eq!(progress.read(), 1);
    assert_eq!(progress.status(), TranscodeStatus::Complete);
    assert_eq!(contexts.borrow().len(), 1);
}
