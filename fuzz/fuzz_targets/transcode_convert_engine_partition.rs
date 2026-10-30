// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![no_main]

use core::cell::Cell;
use core::convert::Infallible;
use core::num::NonZeroUsize;
use std::rc::Rc;

use libfuzzer_sys::fuzz_target;
use qubit_codec as codec;
use qubit_codec::Codec;
use qubit_codec::TranscodeConvertError;
use qubit_codec::TranscodeStatus;
use qubit_codec::engine::DecodeContext;
use qubit_codec::engine::DecodeIncompleteAction;
use qubit_codec::engine::DecodeInvalidAction;
use qubit_codec::engine::EncodeContext;
use qubit_codec::engine::EncodeUnencodableAction;
use qubit_codec::engine::TranscodeConvertEngine;
use qubit_codec::engine::TranscodeDecodeHooks;
use qubit_codec::engine::TranscodeEncodeHooks;

const MAX_INPUT_LEN: usize = 4096;
const MAX_SCHEDULE_LEN: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct InvalidToken;

impl core::fmt::Display for InvalidToken {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("invalid source token")
    }
}

impl std::error::Error for InvalidToken {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DecodeMode {
    Reject,
    Skip,
    Emit,
}

impl DecodeMode {
    fn from_byte(byte: u8) -> Self {
        match byte % 3 {
            0 => Self::Reject,
            1 => Self::Skip,
            _ => Self::Emit,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EncodeMode {
    Reject,
    Skip,
    Replace,
}

impl EncodeMode {
    fn from_byte(byte: u8) -> Self {
        match byte % 3 {
            0 => Self::Reject,
            1 => Self::Skip,
            _ => Self::Replace,
        }
    }
}

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
        if input.len() - index < 2 {
            return Err(codec::DecodeFailure::incomplete(
                NonZeroUsize::new(2).expect("2 is nonzero"),
            ));
        }
        if input[index] == 0xff {
            return Err(codec::DecodeFailure::invalid(InvalidToken, NonZeroUsize::MIN));
        }
        Ok((
            input[index].wrapping_add(input[index + 1]),
            NonZeroUsize::new(2).expect("2 is nonzero"),
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

struct DecodePolicy {
    invalid: DecodeMode,
    incomplete: DecodeMode,
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
            DecodeMode::Reject => DecodeInvalidAction::Reject,
            DecodeMode::Skip => DecodeInvalidAction::Skip { consumed },
            DecodeMode::Emit => DecodeInvalidAction::Emit { value: 7, consumed },
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
            DecodeMode::Reject => DecodeIncompleteAction::Reject,
            DecodeMode::Skip => DecodeIncompleteAction::Skip,
            DecodeMode::Emit => DecodeIncompleteAction::Emit { value: 9 },
        })
    }
}

struct EncodePolicy {
    mode: EncodeMode,
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
            EncodeMode::Reject => EncodeUnencodableAction::Reject,
            EncodeMode::Skip => EncodeUnencodableAction::Skip,
            EncodeMode::Replace => EncodeUnencodableAction::Replace { value: 3 },
        })
    }
}

type Engine = TranscodeConvertEngine<PairSource, VariableTarget, DecodePolicy, EncodePolicy>;

#[derive(Debug, Eq, PartialEq)]
enum ModelError {
    Decode { position: usize },
    Incomplete { position: usize },
    Unencodable { position: usize, value: u8 },
}

#[derive(Debug, Eq, PartialEq)]
struct Model {
    output: Vec<u8>,
    decode_decisions: usize,
    encode_decisions: usize,
}

// Deliberately models the protocol directly; it does not call a Codec or engine
// parser.
fn semantic_oracle(
    payload: &[u8],
    invalid: DecodeMode,
    incomplete: DecodeMode,
    unencodable: EncodeMode,
    markers: bool,
) -> Result<Model, ModelError> {
    let mut model = Model {
        output: Vec::new(),
        decode_decisions: 0,
        encode_decisions: 0,
    };
    if markers {
        model.output.push(0xa0);
    }
    let mut index = 0;
    while index < payload.len() {
        let value = if payload.len() - index < 2 {
            model.decode_decisions += 1;
            match incomplete {
                DecodeMode::Reject => return Err(ModelError::Incomplete { position: index }),
                DecodeMode::Skip => {
                    index = payload.len();
                    continue;
                }
                DecodeMode::Emit => 9,
            }
        } else if payload[index] == 0xff {
            model.decode_decisions += 1;
            match invalid {
                DecodeMode::Reject => return Err(ModelError::Decode { position: index }),
                DecodeMode::Skip => {
                    index += 1;
                    continue;
                }
                DecodeMode::Emit => 7,
            }
        } else {
            payload[index].wrapping_add(payload[index + 1])
        };
        if value == 0xfe {
            model.encode_decisions += 1;
            match unencodable {
                EncodeMode::Reject => return Err(ModelError::Unencodable { position: index, value }),
                EncodeMode::Skip => {}
                EncodeMode::Replace => append_encoded(&mut model.output, 3),
            }
        } else {
            append_encoded(&mut model.output, value);
        }
        index += if payload.len() - index < 2 {
            1
        } else if payload[index] == 0xff {
            1
        } else {
            2
        };
    }
    if markers {
        model.output.push(0xa1);
    }
    Ok(model)
}

fn append_encoded(output: &mut Vec<u8>, value: u8) {
    output.push(value);
    if value & 1 == 0 {
        output.push(!value);
    }
}

fn make_engine(
    invalid: DecodeMode,
    incomplete: DecodeMode,
    unencodable: EncodeMode,
    markers: bool,
) -> (Engine, Rc<Cell<usize>>, Rc<Cell<usize>>) {
    let decode_count = Rc::new(Cell::new(0));
    let encode_count = Rc::new(Cell::new(0));
    let engine = TranscodeConvertEngine::new(
        PairSource,
        VariableTarget { markers },
        DecodePolicy {
            invalid,
            incomplete,
            decisions: decode_count.clone(),
        },
        EncodePolicy {
            mode: unencodable,
            decisions: encode_count.clone(),
        },
    );
    (engine, decode_count, encode_count)
}

#[derive(Debug, Eq, PartialEq)]
enum NormalError {
    Decode(usize),
    Incomplete(usize),
    Unencodable(usize, Option<u8>),
    Other,
}

fn normalize_error(error: &TranscodeConvertError<InvalidToken, Infallible, u8>) -> NormalError {
    match error {
        TranscodeConvertError::DecodeDomain(domain) => match domain.input_index() {
            Some(position) if domain.source() == &InvalidToken => NormalError::Decode(position),
            _ => NormalError::Other,
        },
        TranscodeConvertError::Failure(codec::TranscodeFailure::IncompleteInput { input_index, .. }) => {
            NormalError::Incomplete(*input_index)
        }
        TranscodeConvertError::Unencodable { input_index, value } => NormalError::Unencodable(*input_index, *value),
        _ => NormalError::Other,
    }
}

fn model_error(error: &ModelError) -> NormalError {
    match *error {
        ModelError::Decode { position } => NormalError::Decode(position),
        ModelError::Incomplete { position } => NormalError::Incomplete(position),
        ModelError::Unencodable { position, value } => NormalError::Unencodable(position, Some(value)),
    }
}

fn run_partition(payload: &[u8], schedule: &[u8], engine: &mut Engine) -> Result<(Vec<u8>, usize, usize), NormalError> {
    let mut reset = vec![0; engine.max_reset_output_len().expect("reset bound")];
    let reset_written = engine.reset(&mut reset, 0).map_err(|e| normalize_error(&e))?;
    let mut output = reset[..reset_written].to_vec();
    let mut cursor = 0usize;
    let mut visible = 0usize;
    let mut schedule_index = 0usize;
    let mut eof = false;
    let mut eof_complete = false;
    let mut required_window = None;
    let mut finish_calls = 0usize;
    let mut steps = 0usize;
    let limit = payload
        .len()
        .checked_add(1)
        .and_then(|n| n.checked_mul(64))
        .and_then(|n| n.checked_add(64))
        .expect("bounded input step limit");
    while !eof_complete {
        steps = steps.checked_add(1).expect("step counter overflow");
        assert!(steps <= limit, "partition driver exceeded checked step limit");
        let control = schedule
            .get(schedule_index % schedule.len().max(1))
            .copied()
            .unwrap_or(0);
        let window = required_window.take().unwrap_or_else(|| usize::from(control % 6));
        let mut buffer = vec![0; window];
        let progress = if eof {
            engine.transcode_eof(&payload[..visible], cursor, &mut buffer, 0)
        } else {
            engine.transcode(&payload[..visible], cursor, &mut buffer, 0)
        };
        match progress {
            Err(error) => return Err(normalize_error(&error)),
            Ok(progress) => {
                progress
                    .validate(cursor, visible - cursor, 0, window)
                    .expect("engine progress validates");
                cursor = cursor.checked_add(progress.read()).expect("cursor overflow");
                output.extend_from_slice(&buffer[..progress.written()]);
                match progress.status() {
                    TranscodeStatus::NeedOutput { required, .. } => required_window = Some(required.get()),
                    TranscodeStatus::NeedInput { .. } => {
                        if visible < payload.len() {
                            visible = (visible + usize::from(control % 16) + 1).min(payload.len());
                        } else {
                            assert!(!eof, "EOF cannot request additional source input");
                            eof = true;
                        }
                    }
                    TranscodeStatus::Complete => {
                        if eof {
                            eof_complete = true;
                        } else if visible < payload.len() {
                            visible = (visible + usize::from(control % 16) + 1).min(payload.len());
                        } else {
                            eof = true;
                        }
                    }
                }
            }
        }
        schedule_index = schedule_index.checked_add(1).expect("schedule counter overflow");
    }
    assert!(eof);
    let mut finish = vec![0; engine.max_finish_output_len().expect("finish bound")];
    let written = engine.finish(&mut finish, 0).map_err(|e| normalize_error(&e))?;
    finish_calls += 1;
    assert_eq!(finish_calls, 1);
    output.extend_from_slice(&finish[..written]);
    Ok((output, steps, finish_calls))
}

fn run_complete(
    payload: &[u8],
    invalid: DecodeMode,
    incomplete: DecodeMode,
    unencodable: EncodeMode,
    markers: bool,
) -> Result<(Vec<u8>, usize, usize), NormalError> {
    let (mut engine, decode_count, encode_count) = make_engine(invalid, incomplete, unencodable, markers);
    let size = engine
        .max_total_output_len(payload.len())
        .expect("complete output bound");
    let mut output = vec![0; size];
    let written = engine
        .transcode_complete_into(payload, &mut output)
        .map_err(|e| normalize_error(&e))?;
    output.truncate(written);
    Ok((output, decode_count.get(), encode_count.get()))
}

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_INPUT_LEN)];
    let config = [
        data.first().copied().unwrap_or(0),
        data.get(1).copied().unwrap_or(0),
        data.get(2).copied().unwrap_or(0),
        data.get(3).copied().unwrap_or(0),
    ];
    let schedule_end = data.len().min(4 + MAX_SCHEDULE_LEN);
    let schedule = &data[4.min(data.len())..schedule_end];
    let payload = &data[schedule_end..];
    let invalid = DecodeMode::from_byte(config[0]);
    let incomplete = DecodeMode::from_byte(config[1]);
    let unencodable = EncodeMode::from_byte(config[2]);
    let markers = config[3] & 1 != 0;
    let expected = semantic_oracle(payload, invalid, incomplete, unencodable, markers);
    let (mut engine, decode_count, encode_count) = make_engine(invalid, incomplete, unencodable, markers);
    let partition = run_partition(payload, schedule, &mut engine);
    let complete = run_complete(payload, invalid, incomplete, unencodable, markers);
    match expected {
        Ok(model) => {
            let (partition_output, _steps, finish_calls) = partition.expect("partition driver should complete");
            let (complete_output, complete_decodes, complete_encodes) =
                complete.expect("complete lifecycle should succeed");
            assert_eq!(model.output, partition_output);
            assert_eq!(model.output, complete_output);
            assert_eq!(model.decode_decisions, decode_count.get());
            assert_eq!(model.encode_decisions, encode_count.get());
            assert_eq!(model.decode_decisions, complete_decodes);
            assert_eq!(model.encode_decisions, complete_encodes);
            assert_eq!(finish_calls, 1);
        }
        Err(model) => {
            let expected_error = model_error(&model);
            assert_eq!(Err(expected_error), partition);
            assert_eq!(Err(model_error(&model)), complete);
        }
    }
});
