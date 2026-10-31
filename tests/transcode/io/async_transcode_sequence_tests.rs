// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::collections::VecDeque;
use std::convert::Infallible;
use std::io;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use qubit_codec::AsyncTranscodeDecodeInput;
use qubit_codec::AsyncTranscodeEncodeOutput;
use qubit_codec::CapacityError;
use qubit_codec::Codec;
use qubit_codec::CodecTranscodeDecoder;
use qubit_codec::CodecTranscodeEncoder;
use qubit_codec::DecodeFailure;
use qubit_codec::TranscodeDecodeError;
use qubit_codec::TranscodeProgress;
use qubit_codec::Transcoder;
use qubit_io::AsyncInput;
use qubit_io::AsyncOutput;

use super::internal::InputAction;
use super::internal::IoEvent;
use super::internal::OutputAction;
use super::internal::input_actions;
use super::internal::io_events;
use super::internal::next;
use super::internal::output_actions;
use super::internal::payload;

#[derive(Debug, Default)]
struct MarkerCodec {
    encode_resets: usize,
    encode_values: usize,
    encode_finishes: usize,
    decode_resets: usize,
    decode_values: usize,
    decode_finishes: usize,
}

impl Codec for MarkerCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;
    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_RESET_UNITS: usize = 1;
    const MAX_ENCODE_FINISH_UNITS: usize = 1;
    const MAX_DECODE_RESET_VALUES: usize = 1;
    const MAX_DECODE_FINISH_VALUES: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, std::num::NonZeroUsize), DecodeFailure<Infallible>> {
        self.decode_values += 1;
        Ok((input[index], std::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Infallible> {
        self.encode_values += 1;
        output[index] = *value;
        Ok(1)
    }

    unsafe fn encode_reset(&mut self, output: &mut [u8], index: usize) -> Result<usize, Infallible> {
        self.encode_resets += 1;
        output[index] = b'^';
        Ok(1)
    }

    unsafe fn encode_finish(&mut self, output: &mut [u8], index: usize) -> Result<usize, Infallible> {
        self.encode_finishes += 1;
        output[index] = b'!';
        Ok(1)
    }

    unsafe fn decode_reset(&mut self, output: &mut [u8], index: usize) -> Result<usize, Infallible> {
        self.decode_resets += 1;
        output[index] = b'^';
        Ok(1)
    }

    unsafe fn decode_finish(&mut self, output: &mut [u8], index: usize) -> Result<usize, Infallible> {
        self.decode_finishes += 1;
        output[index] = b'!';
        Ok(1)
    }
}

#[derive(Debug, Default)]
struct EofTrailerDecoder {
    emitted: bool,
}

impl Transcoder for EofTrailerDecoder {
    type Input = u8;
    type Output = u8;
    type Error = TranscodeDecodeError<Infallible>;

    fn max_transcode_output_len(&self, len: usize) -> Result<usize, CapacityError> {
        Ok(len.max(2))
    }

    fn reset(&mut self, _output: &mut [u8], _index: usize) -> Result<usize, Self::Error> {
        Ok(0)
    }

    fn transcode(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        let count = (input.len() - input_index).min(output.len() - output_index);
        output[output_index..output_index + count].copy_from_slice(&input[input_index..input_index + count]);
        Ok(TranscodeProgress::complete(count, count))
    }

    fn transcode_eof(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if self.emitted {
            return Ok(TranscodeProgress::complete(0, 0));
        }
        let available = output.len() - output_index;
        if available < 2 {
            return Ok(TranscodeProgress::need_output(
                std::num::NonZeroUsize::new(2).expect("2 is nonzero"),
                0,
                0,
            ));
        }
        output[output_index] = b'<';
        output[output_index + 1] = b'>';
        self.emitted = true;
        Ok(TranscodeProgress::complete(input.len() - input_index, 2))
    }

    fn finish(&mut self, _output: &mut [u8], _index: usize) -> Result<usize, Self::Error> {
        Ok(0)
    }
}

#[derive(Debug, Default)]
struct ScriptOutput {
    bytes: Vec<u8>,
    pending: bool,
    chunk: usize,
    fail_flush_once: bool,
    events: VecDeque<IoEvent>,
}

impl AsyncOutput for ScriptOutput {
    type Item = u8;

    unsafe fn poll_write_unchecked(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        input: &[u8],
        index: usize,
        count: usize,
    ) -> Poll<io::Result<usize>> {
        if self.events.front() == Some(&IoEvent::PendingThenWake) && !self.pending {
            self.pending = true;
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        if self.events.front() == Some(&IoEvent::PendingThenWake) {
            self.events.pop_front();
            self.pending = false;
        }
        let event = self.events.pop_front().unwrap_or(IoEvent::Ready);
        if event == IoEvent::FailOnce {
            return Poll::Ready(Err(io::Error::other("scripted write failure")));
        }
        let short_limit = match event {
            IoEvent::ReadyShort(limit) => limit,
            IoEvent::Ready | IoEvent::PendingThenWake | IoEvent::FailOnce => self.chunk.max(count),
        };
        let written = count.min(short_limit.max(1));
        self.bytes.extend_from_slice(&input[index..index + written]);
        Poll::Ready(Ok(written))
    }

    fn poll_flush(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        if self.fail_flush_once {
            self.fail_flush_once = false;
            return Poll::Ready(Err(io::Error::other("scripted flush failure")));
        }
        Poll::Ready(Ok(()))
    }
}

fn complete<F: std::future::Future>(future: F) -> F::Output {
    super::internal::complete_when_woken(future)
}

fn complete_case<F: std::future::Future>(future: F, seed: u64, action: usize) -> F::Output {
    use std::sync::Arc;
    use std::sync::atomic::Ordering;

    let wakes = Arc::new(super::internal::WakeCount::default());
    let waker = std::task::Waker::from(Arc::clone(&wakes));
    let mut context = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    for poll in 1..=128 {
        let before = wakes.0.load(Ordering::SeqCst);
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return value,
            Poll::Pending => assert!(
                wakes.0.load(Ordering::SeqCst) > before,
                "seed={seed}, action={action}, poll={poll}: Pending without notifying current waker",
            ),
        }
    }
    panic!("seed={seed}, action={action}, poll=128: future exceeded poll budget");
}

fn cancel_after_wake<F: std::future::Future>(future: F, seed: u64, action: usize) {
    use std::sync::Arc;
    use std::sync::atomic::Ordering;

    let wakes = Arc::new(super::internal::WakeCount::default());
    let waker = std::task::Waker::from(Arc::clone(&wakes));
    let mut context = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    let before = wakes.0.load(Ordering::SeqCst);
    assert!(
        matches!(future.as_mut().poll(&mut context), Poll::Pending),
        "seed={seed}, action={action}, poll=1: cancel script expected Pending"
    );
    assert!(
        wakes.0.load(Ordering::SeqCst) > before,
        "seed={seed}, action={action}, poll=1: Pending did not notify current waker before cancellation"
    );
}

fn assert_output_prefix(output: &AsyncTranscodeEncodeOutput<ScriptOutput>, expected: &[u8], seed: u64, action: usize) {
    let delivered = &output.inner().bytes;
    assert!(
        expected.starts_with(delivered),
        "seed={seed}, action={action}: sink bytes are not an oracle prefix"
    );
    assert_eq!(
        expected.len(),
        delivered.len() + output.pending_len(),
        "seed={seed}, action={action}: committed byte conservation failed"
    );
}

#[derive(Debug)]
struct ScriptInput {
    bytes: Vec<u8>,
    position: usize,
    chunk: usize,
    pending: bool,
    fail_once: bool,
    event: Option<IoEvent>,
}

impl AsyncInput for ScriptInput {
    type Item = u8;

    unsafe fn poll_read_unchecked(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        output: &mut [u8],
        index: usize,
        count: usize,
    ) -> Poll<io::Result<usize>> {
        if self.fail_once {
            self.fail_once = false;
            return Poll::Ready(Err(io::Error::other("scripted read failure")));
        }
        if self.event == Some(IoEvent::PendingThenWake) && !self.pending {
            self.pending = true;
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        self.pending = false;
        let short_limit = match self.event {
            Some(IoEvent::ReadyShort(limit)) => limit,
            None => self.chunk,
            _ => count,
        };
        let read = count.min(short_limit).min(self.bytes.len() - self.position);
        let end = self.position + read;
        output[index..index + read].copy_from_slice(&self.bytes[self.position..end]);
        self.position = end;
        Poll::Ready(Ok(read))
    }
}

fn run_output_case(seed: u64) {
    let values = payload(seed);
    let mut random = seed + 1;
    let capacity = [1, 2, 3, 8][(next(&mut random) % 4) as usize];
    let (_, write_event) = io_events(seed);
    let inject_write_error = write_event == IoEvent::FailOnce;
    let sink = ScriptOutput {
        chunk: 1 + (next(&mut random) % 4) as usize,
        events: VecDeque::from([write_event]),
        ..ScriptOutput::default()
    };
    let mut output = AsyncTranscodeEncodeOutput::with_capacity(sink, capacity);
    let mut encoder = CodecTranscodeEncoder::new(MarkerCodec::default());
    let mut mapper = |_error| io::Error::other("unexpected codec error");
    let schedule = output_actions(seed, values.len());
    assert_eq!(
        schedule,
        output_actions(seed, values.len()),
        "seed={seed}: output action generation is not reproducible"
    );
    let mut actions = VecDeque::from(schedule);
    let mut committed = Vec::new();
    let mut offset = 0;
    let mut recovered_write_error = false;
    let mut reset_count = 0;
    let mut action_index = 0;
    while let Some(action) = actions.pop_front() {
        action_index += 1;
        assert!(
            action_index <= 128,
            "seed={seed}, action={action_index}, poll=0: action budget exceeded"
        );
        match action {
            OutputAction::Reset => {
                complete_case(output.reset_async(&mut encoder, &mut mapper), seed, action_index)
                    .unwrap_or_else(|error| panic!("seed={seed}, action={action_index} reset: {error}"));
                committed.push(b'^');
                reset_count += 1;
            }
            OutputAction::Transcode(requested) => {
                let target = requested.min(values.len().saturating_sub(offset));
                let mut remaining = target;
                let mut poll = 0;
                while remaining > 0 {
                    poll += 1;
                    assert!(
                        poll <= 128,
                        "seed={seed}, action={action_index}, poll={poll}: transcode poll budget exceeded"
                    );
                    let progress = complete_case(
                        output.transcode_async(&mut encoder, &mut mapper, &values, offset, remaining),
                        seed,
                        action_index,
                    )
                    .unwrap_or_else(|error| panic!("seed={seed}, action={action_index}, poll={poll}: {error}"));
                    assert!(
                        progress.read() > 0,
                        "seed={seed}, action={action_index}, poll={poll}: no input progress"
                    );
                    committed.extend_from_slice(&values[offset..offset + progress.read()]);
                    offset += progress.read();
                    remaining -= progress.read();
                }
            }
            OutputAction::Drain | OutputAction::RecoverWrite => {
                let result = complete_case(output.drain_async(), seed, action_index);
                match (action, result) {
                    (OutputAction::Drain, Err(error)) if inject_write_error && !recovered_write_error => {
                        assert_eq!(
                            io::ErrorKind::Other,
                            error.kind(),
                            "seed={seed}, action={action_index}: FailOnce error kind"
                        );
                        recovered_write_error = true;
                    }
                    (OutputAction::RecoverWrite, Ok(())) => assert!(
                        recovered_write_error,
                        "seed={seed}, action={action_index}: recovery without FailOnce"
                    ),
                    (_, Ok(())) => {}
                    (_, Err(error)) => panic!("seed={seed}, action={action_index}: drain failed unexpectedly: {error}"),
                }
            }
            OutputAction::Flush => {
                complete_case(output.flush_async(), seed, action_index)
                    .unwrap_or_else(|error| panic!("seed={seed}, action={action_index} flush: {error}"));
            }
            OutputAction::CancelDrain => {
                cancel_after_wake(output.drain_async(), seed, action_index);
            }
            OutputAction::Finish => {
                complete_case(output.finish_async(&mut encoder, &mut mapper), seed, action_index)
                    .unwrap_or_else(|error| panic!("seed={seed}, action={action_index} finish: {error}"));
                committed.push(b'!');
            }
        }
        assert_output_prefix(&output, &committed, seed, action_index);
    }
    assert_eq!(
        values.len(),
        offset,
        "seed={seed}, action={action_index}: generated sequence left source input"
    );
    assert!(
        recovered_write_error == inject_write_error,
        "seed={seed}, action={action_index}: FailOnce recovery state mismatch"
    );
    assert!(
        committed.last() == Some(&b'!'),
        "seed={seed}, action={action_index}: generated sequence omitted finish"
    );
    assert!(
        committed.iter().filter(|&&byte| byte == b'^').count() > 0,
        "seed={seed}, action={action_index}: no reset generated"
    );
    complete_case(output.flush_async(), seed, action_index + 1)
        .unwrap_or_else(|error| panic!("seed={seed}, action={}: final flush: {error}", action_index + 1));
    let expected = committed;
    let codec = encoder.into_codec();
    assert_eq!(reset_count, codec.encode_resets, "seed={seed}: encode reset count");
    assert_eq!(values.len(), codec.encode_values, "seed={seed}: encode value count");
    assert_eq!(1, codec.encode_finishes, "seed={seed}: encode finish count");
    let (sink, pending) = output.into_parts();
    assert_eq!(expected, sink.bytes, "seed={seed}, final sink mismatch");
    assert!(pending.is_empty(), "seed={seed}, pending output after flush");
}

#[test]
fn test_async_transcode_sequence_output_matches_model() {
    for seed in 0..256 {
        run_output_case(seed);
    }
}

fn assert_input_model(
    input: &mut AsyncTranscodeDecodeInput<ScriptInput>,
    source_position: usize,
    unread: &[u8],
    expected: &[u8],
    consumed: usize,
    seed: u64,
    action: usize,
) {
    assert_eq!(
        source_position,
        input.inner_mut().position,
        "seed={seed}, action={action}: source delivery position"
    );
    assert_eq!(
        expected.get(consumed..source_position),
        Some(unread),
        "seed={seed}, action={action}: unread equals delivered unconsumed range"
    );
    let mut source_tail = unread.to_vec();
    source_tail.extend_from_slice(&expected[source_position..]);
    assert_eq!(
        expected.get(consumed..),
        Some(source_tail.as_slice()),
        "seed={seed}, action={action}: consumed + unread + undelivered suffix conservation"
    );
    assert_eq!(
        unread,
        input.unread(),
        "seed={seed}, action={action}: model unread differs from bridge"
    );
}

fn run_input_case(seed: u64) {
    let expected = payload(seed);
    let mut random = seed + 1;
    let capacity = [1, 2, 3, 8][(next(&mut random) % 4) as usize];
    let (read_event, _) = io_events(seed);
    let chunk = match read_event {
        IoEvent::ReadyShort(limit) => limit,
        _ => capacity,
    };
    let source = ScriptInput {
        bytes: expected.clone(),
        position: 0,
        chunk,
        pending: false,
        fail_once: read_event == IoEvent::FailOnce,
        event: Some(read_event),
    };
    let mut input = AsyncTranscodeDecodeInput::with_capacity(source, capacity);
    let mut decoded = [0_u8; 132];
    let output_len = decoded.len();
    let mut decoder = CodecTranscodeDecoder::new(MarkerCodec::default());
    let mut mapper = |_error| io::Error::other("decode error");
    let mut written = 0;
    let mut delivered = 0;
    let mut consumed = 0;
    let mut model_unread = Vec::new();
    let mut recovered_error = false;
    let schedule = input_actions(seed, expected.len(), input.capacity(), chunk);
    assert_eq!(
        schedule,
        input_actions(seed, expected.len(), input.capacity(), chunk),
        "seed={seed}: input action generation is not reproducible"
    );
    let mut eof_seen = false;
    for (zero_index, action) in schedule.iter().copied().enumerate() {
        let action_index = zero_index + 1;
        assert!(
            action_index <= 128,
            "seed={seed}, action={action_index}, poll=0: input action budget exceeded"
        );
        match action {
            InputAction::Reset => {
                assert_eq!(
                    0, delivered,
                    "seed={seed}, action={action_index}: reset must precede source delivery"
                );
                written = input
                    .reset(&mut decoder, &mut mapper, &mut decoded, 0, output_len)
                    .unwrap_or_else(|error| panic!("seed={seed}, action={action_index}: reset: {error}"));
            }
            InputAction::Refill => {
                let result = complete_case(input.fill_more_async(), seed, action_index);
                match result {
                    Err(error) if read_event == IoEvent::FailOnce && !recovered_error => {
                        assert_eq!(
                            io::ErrorKind::Other,
                            error.kind(),
                            "seed={seed}, action={action_index}: FailOnce read error kind"
                        );
                        recovered_error = true;
                    }
                    Ok(true) => {
                        let new_delivered = input.inner_mut().position;
                        assert!(
                            new_delivered > delivered,
                            "seed={seed}, action={action_index}: successful refill delivered no units"
                        );
                        model_unread.extend_from_slice(&expected[delivered..new_delivered]);
                        delivered = new_delivered;
                    }
                    Ok(false) if expected.is_empty() => {}
                    Ok(false) => panic!(
                        "seed={seed}, action={action_index}: EOF arrived before generated EOF action (action={action:?}, delivered={delivered}, expected={}, chunk={}, schedule={schedule:?})",
                        expected.len(),
                        chunk,
                    ),
                    Err(error) => panic!("seed={seed}, action={action_index}: unexpected refill error: {error}"),
                }
            }
            InputAction::RecoverRead => {
                assert!(
                    recovered_error,
                    "seed={seed}, action={action_index}: recovery without read error"
                );
                if complete_case(input.fill_more_async(), seed, action_index)
                    .unwrap_or_else(|error| panic!("seed={seed}, action={action_index}: read recovery: {error}"))
                {
                    let new_delivered = input.inner_mut().position;
                    model_unread.extend_from_slice(&expected[delivered..new_delivered]);
                    delivered = new_delivered;
                }
            }
            InputAction::CancelRead => {
                assert!(
                    model_unread.is_empty(),
                    "seed={seed}, action={action_index}: cancellation requires empty unread buffer"
                );
                assert!(
                    delivered < expected.len(),
                    "seed={seed}, action={action_index}: cancellation after source end"
                );
                cancel_after_wake(
                    input.transcode_async(&mut decoder, &mut mapper, &mut decoded, written, output_len - written),
                    seed,
                    action_index,
                );
            }
            InputAction::Transcode => {
                assert!(
                    !model_unread.is_empty(),
                    "seed={seed}, action={action_index}: transcode requires refilled input"
                );
                let step = complete_case(
                    input.transcode_async(&mut decoder, &mut mapper, &mut decoded, written, output_len - written),
                    seed,
                    action_index,
                )
                .unwrap_or_else(|error| panic!("seed={seed}, action={action_index}: transcode: {error}"));
                let qubit_codec::AsyncTranscodeDecodeStep::Progress(progress) = step else {
                    panic!("seed={seed}, action={action_index}: unexpected EOF before EOF action");
                };
                assert_eq!(
                    model_unread.len(),
                    progress.read(),
                    "seed={seed}, action={action_index}: decoder consumed unexpected count"
                );
                consumed += progress.read();
                model_unread.drain(..progress.read());
                written += progress.written();
            }
            InputAction::Eof => {
                assert_eq!(
                    expected.len(),
                    delivered,
                    "seed={seed}, action={action_index}: EOF before all source bytes delivered"
                );
                assert_eq!(
                    delivered, consumed,
                    "seed={seed}, action={action_index}: EOF with unread payload"
                );
                let step = complete_case(
                    input.transcode_async(&mut decoder, &mut mapper, &mut decoded, written, output_len - written),
                    seed,
                    action_index,
                )
                .unwrap_or_else(|error| panic!("seed={seed}, action={action_index}: EOF poll: {error}"));
                assert!(
                    matches!(step, qubit_codec::AsyncTranscodeDecodeStep::EndOfInput),
                    "seed={seed}, action={action_index}: expected EndOfInput"
                );
                eof_seen = true;
            }
            InputAction::EofStep => {
                assert!(
                    eof_seen,
                    "seed={seed}, action={action_index}: EOF step before EndOfInput"
                );
                let progress = input
                    .transcode_eof_step(&mut decoder, &mut mapper, &mut decoded, written, output_len - written)
                    .unwrap_or_else(|error| panic!("seed={seed}, action={action_index}: EOF step: {error}"));
                assert!(
                    progress.is_complete(),
                    "seed={seed}, action={action_index}: identity codec EOF should complete"
                );
                assert_eq!(
                    0,
                    progress.read(),
                    "seed={seed}, action={action_index}: empty EOF consumed source"
                );
                written += progress.written();
            }
            InputAction::Finish => {
                assert!(eof_seen, "seed={seed}, action={action_index}: finish before EOF");
                written += input
                    .finish(&mut decoder, &mut mapper, &mut decoded, written, output_len - written)
                    .unwrap_or_else(|error| panic!("seed={seed}, action={action_index}: finish: {error}"));
            }
        }
        assert_input_model(
            &mut input,
            delivered,
            &model_unread,
            &expected,
            consumed,
            seed,
            action_index,
        );
        let mut decoded_oracle = vec![b'^'];
        decoded_oracle.extend_from_slice(&expected[..consumed]);
        if eof_seen && matches!(action, InputAction::Finish) {
            decoded_oracle.push(b'!');
        }
        assert_eq!(
            decoded_oracle,
            decoded[..written],
            "seed={seed}, action={action_index}: decoded output oracle"
        );
    }
    assert!(
        eof_seen,
        "seed={seed}, action={}, poll=0: generated actions omitted EOF",
        schedule_len_hint(seed, expected.len(), input.capacity(), chunk)
    );
    assert_eq!(expected.len(), consumed, "seed={seed}: final decoded consumption");
    assert!(
        recovered_error == (read_event == IoEvent::FailOnce),
        "seed={seed}: read recovery state mismatch"
    );
    let codec = decoder.into_codec();
    assert_eq!(1, codec.decode_resets, "seed={seed}: decode reset count");
    assert_eq!(expected.len(), codec.decode_values, "seed={seed}: decode value count");
    assert_eq!(1, codec.decode_finishes, "seed={seed}: decode finish count");
    let (source, unread) = input.into_parts();
    assert_eq!(expected.len(), source.position, "seed={seed}: final source position");
    assert!(unread.is_empty(), "seed={seed}: final unread units");
}

fn schedule_len_hint(seed: u64, payload_len: usize, capacity: usize, chunk: usize) -> usize {
    input_actions(seed, payload_len, capacity, chunk).len()
}

#[test]
fn test_async_transcode_sequence_input_matches_model() {
    for seed in 0..256 {
        run_input_case(seed);
    }
}

#[test]
fn test_async_transcode_sequence_input_fixed_smoke() {
    for seed in [0x5eed] {
        let expected = payload(seed);
        let mut random = seed + 1;
        let capacity = [1, 2, 3, 8][(next(&mut random) % 4) as usize];
        let source = ScriptInput {
            bytes: expected.clone(),
            position: 0,
            chunk: 1 + (next(&mut random) % 4) as usize,
            pending: false,
            fail_once: seed.is_multiple_of(13),
            event: None,
        };
        let mut input = AsyncTranscodeDecodeInput::with_capacity(source, capacity);
        let mut decoded = [0_u8; 132];
        let output_len = decoded.len();
        let mut decoder = CodecTranscodeDecoder::new(MarkerCodec::default());
        let mut mapper = |_error| io::Error::other("decode error");
        let mut written = input
            .reset(&mut decoder, &mut mapper, &mut decoded, 0, output_len)
            .unwrap_or_else(|error| panic!("seed={seed}, reset: {error}"));
        let mut actions = 0;
        let mut source_done = false;
        let mut recovered_read_error = false;
        while !source_done {
            actions += 1;
            assert!(
                actions <= 128,
                "seed={seed}, action={actions}: input action budget exhausted"
            );
            let step_result = complete_case(
                input.transcode_async(&mut decoder, &mut mapper, &mut decoded, written, output_len - written),
                seed,
                actions,
            );
            let step = match step_result {
                Ok(step) => step,
                Err(error) if seed.is_multiple_of(13) && !recovered_read_error => {
                    assert_eq!(
                        io::ErrorKind::Other,
                        error.kind(),
                        "seed={seed}, action={actions}: scripted read error kind"
                    );
                    recovered_read_error = true;
                    complete_case(
                        input.transcode_async(&mut decoder, &mut mapper, &mut decoded, written, output_len - written),
                        seed,
                        actions,
                    )
                    .unwrap_or_else(|retry| panic!("seed={seed}, action={actions} retry: {retry}"))
                }
                Err(error) => panic!("seed={seed}, action={actions}: transcode: {error}"),
            };
            match step {
                qubit_codec::AsyncTranscodeDecodeStep::Progress(progress) => {
                    written += progress.written();
                }
                qubit_codec::AsyncTranscodeDecodeStep::EndOfInput => source_done = true,
            }
        }
        let eof = input
            .transcode_eof_step(&mut decoder, &mut mapper, &mut decoded, written, output_len - written)
            .unwrap_or_else(|error| panic!("seed={seed}, EOF: {error}"));
        written += eof.written();
        written += input
            .finish(&mut decoder, &mut mapper, &mut decoded, written, output_len - written)
            .unwrap_or_else(|error| panic!("seed={seed}, finish: {error}"));
        let mut expected_decoded = vec![b'^'];
        expected_decoded.extend_from_slice(&expected);
        expected_decoded.push(b'!');
        assert_eq!(
            expected_decoded,
            decoded[..written],
            "seed={seed}, decoded model mismatch"
        );
        assert!(input.unread().is_empty(), "seed={seed}, decoder left unread bytes");
        let codec = decoder.into_codec();
        assert_eq!(1, codec.decode_resets, "seed={seed}: decode reset count");
        assert_eq!(expected.len(), codec.decode_values, "seed={seed}: decode value count");
        assert_eq!(1, codec.decode_finishes, "seed={seed}: decode finish count");
        let (source, unread) = input.into_parts();
        assert_eq!(
            expected.len(),
            source.position,
            "seed={seed}, source delivery position mismatch"
        );
        assert_eq!(
            seed.is_multiple_of(13),
            recovered_read_error,
            "seed={seed}: expected read error injection was not observed"
        );
        assert!(unread.is_empty(), "seed={seed}, unread data after recovery");
    }
}

#[test]
fn test_async_transcode_sequence_fixed_pending_wake_script() {
    run_output_case(0x5eed);
    let mut output = AsyncTranscodeEncodeOutput::with_capacity(
        ScriptOutput {
            chunk: 2,
            fail_flush_once: true,
            events: VecDeque::from([IoEvent::ReadyShort(2), IoEvent::PendingThenWake]),
            ..ScriptOutput::default()
        },
        4,
    );
    let mut encoder = CodecTranscodeEncoder::new(MarkerCodec::default());
    let mut mapper = |_error| io::Error::other("unexpected encode error");
    complete(output.reset_async(&mut encoder, &mut mapper)).expect("fixed reset should succeed");
    let progress = complete(output.transcode_async(&mut encoder, &mut mapper, &[11, 12], 0, 2))
        .expect("fixed transcode should succeed");
    assert_eq!(2, progress.read());
    complete(output.finish_async(&mut encoder, &mut mapper)).expect("fixed finish should succeed");
    cancel_after_wake(output.flush_async(), 0x5eed, 1);
    assert_eq!(
        [b'^', 11],
        output.inner().bytes.as_slice(),
        "short write must commit only its prefix before Pending"
    );
    assert_eq!(
        2,
        output.pending_len(),
        "cancelled flush must retain the unwritten suffix"
    );
    let error = complete(output.flush_async()).expect_err("fixed flush error should be injected once");
    assert_eq!(io::ErrorKind::Other, error.kind());
    complete(output.flush_async()).expect("retry should deliver retained bytes");
    let (sink, pending) = output.into_parts();
    assert_eq!([b'^', 11, 12, b'!'], sink.bytes.as_slice());
    assert!(pending.is_empty());

    let mut cancel_reset = AsyncTranscodeEncodeOutput::with_capacity(
        ScriptOutput {
            chunk: 8,
            events: VecDeque::from([IoEvent::PendingThenWake]),
            ..ScriptOutput::default()
        },
        2,
    );
    let mut encoder = CodecTranscodeEncoder::new(MarkerCodec::default());
    let mut mapper = |_error| io::Error::other("unexpected encode error");
    complete(cancel_reset.reset_async(&mut encoder, &mut mapper)).expect("reset setup");
    let progress =
        complete(cancel_reset.transcode_async(&mut encoder, &mut mapper, &[31], 0, 1)).expect("transcode setup");
    assert_eq!(1, progress.read());
    cancel_after_wake(cancel_reset.reset_async(&mut encoder, &mut mapper), 0x5eed, 2);
    complete(cancel_reset.reset_async(&mut encoder, &mut mapper)).expect("cancelled reset retry");
    complete(cancel_reset.finish_async(&mut encoder, &mut mapper)).expect("finish after reset");
    complete(cancel_reset.flush_async()).expect("flush reset recovery");
    let (sink, _) = cancel_reset.into_parts();
    assert_eq!([b'^', 31, b'^', b'!'], sink.bytes.as_slice());

    let mut cancel_finish = AsyncTranscodeEncodeOutput::with_capacity(
        ScriptOutput {
            chunk: 8,
            events: VecDeque::from([IoEvent::PendingThenWake]),
            ..ScriptOutput::default()
        },
        2,
    );
    let mut encoder = CodecTranscodeEncoder::new(MarkerCodec::default());
    let mut mapper = |_error| io::Error::other("unexpected encode error");
    complete(cancel_finish.reset_async(&mut encoder, &mut mapper)).expect("finish setup reset");
    let progress = complete(cancel_finish.transcode_async(&mut encoder, &mut mapper, &[41], 0, 1))
        .expect("finish setup transcode");
    assert_eq!(1, progress.read());
    cancel_after_wake(cancel_finish.finish_async(&mut encoder, &mut mapper), 0x5eed, 3);
    complete(cancel_finish.finish_async(&mut encoder, &mut mapper)).expect("cancelled finish retry");
    complete(cancel_finish.flush_async()).expect("flush finish recovery");
    let (sink, _) = cancel_finish.into_parts();
    assert_eq!([b'^', 41, b'!'], sink.bytes.as_slice());

    let mut cancelled_input = AsyncTranscodeDecodeInput::with_capacity(
        ScriptInput {
            bytes: vec![51, 52],
            position: 0,
            chunk: 1,
            pending: false,
            fail_once: false,
            event: Some(IoEvent::PendingThenWake),
        },
        1,
    );
    let mut decoder = CodecTranscodeDecoder::new(MarkerCodec::default());
    let mut decode_mapper = |_error| io::Error::other("unexpected decode error");
    let mut decoded = [0_u8; 4];
    let mut written = cancelled_input
        .reset(&mut decoder, &mut decode_mapper, &mut decoded, 0, 4)
        .expect("fixed input reset");
    let first = complete_case(
        cancelled_input.transcode_async(&mut decoder, &mut decode_mapper, &mut decoded, written, 4 - written),
        0x5eed,
        1,
    )
    .expect("fixed first input read");
    let qubit_codec::AsyncTranscodeDecodeStep::Progress(first) = first else {
        panic!("seed=0x5eed, action=1: expected first short read");
    };
    written += first.written();
    assert_eq!(1, first.read());
    cancel_after_wake(
        cancelled_input.transcode_async(&mut decoder, &mut decode_mapper, &mut decoded, written, 4 - written),
        0x5eed,
        4,
    );
    let second = complete_case(
        cancelled_input.transcode_async(&mut decoder, &mut decode_mapper, &mut decoded, written, 4 - written),
        0x5eed,
        2,
    )
    .expect("cancelled short read should recover");
    let qubit_codec::AsyncTranscodeDecodeStep::Progress(second) = second else {
        panic!("seed=0x5eed, action=2: expected second short read");
    };
    written += second.written();
    assert_eq!(1, second.read());
    assert_eq!([b'^', 51, 52], decoded[..written]);

    let mut empty = AsyncTranscodeDecodeInput::new(ScriptInput {
        bytes: Vec::new(),
        position: 0,
        chunk: 1,
        pending: false,
        fail_once: false,
        event: None,
    });
    assert!(!complete(empty.fill_until_async(1)).expect("empty source should report EOF"));
    let mut eof_decoder = EofTrailerDecoder::default();
    let mut eof_output = [0_u8; 2];
    let mut eof_mapper = |_error| io::Error::other("unexpected EOF decoder error");
    let end = complete(empty.transcode_async(&mut eof_decoder, &mut eof_mapper, &mut eof_output, 0, 2))
        .expect("bridge should report empty-source EOF");
    assert!(matches!(end, qubit_codec::AsyncTranscodeDecodeStep::EndOfInput));
    let pending_eof = empty
        .transcode_eof_step(&mut eof_decoder, &mut eof_mapper, &mut eof_output, 0, 1)
        .expect("insufficient EOF output is progress");
    assert_eq!(
        TranscodeProgress::need_output(std::num::NonZeroUsize::new(2).expect("2 is nonzero"), 0, 0),
        pending_eof
    );
    let completed_eof = empty
        .transcode_eof_step(&mut eof_decoder, &mut eof_mapper, &mut eof_output, 0, 2)
        .expect("EOF should complete with sufficient space");
    assert_eq!(TranscodeProgress::complete(0, 2), completed_eof);
    assert_eq!([b'<', b'>'], eof_output);
    let repeated_eof = empty
        .transcode_eof_step(&mut eof_decoder, &mut eof_mapper, &mut eof_output, 0, 2)
        .expect("repeated EOF dispatch should be harmless");
    assert_eq!(TranscodeProgress::complete(0, 0), repeated_eof);

    let bytes = [1, 2, 3, 4];
    let mut input = AsyncTranscodeDecodeInput::new(ScriptInput {
        bytes: bytes.to_vec(),
        position: 0,
        chunk: 1,
        pending: false,
        fail_once: false,
        event: None,
    });
    assert!(complete(input.fill_until_async(bytes.len())).expect("fixed input script should fill"));
    assert_eq!(bytes, input.unread());
}
