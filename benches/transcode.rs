// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Baseline benchmarks for the generic `Transcoder` lifecycle.

use std::convert::Infallible;
use std::fmt::Debug;
use std::hint::black_box;
use std::num::NonZeroUsize;
use std::time::Duration;

use criterion::BenchmarkGroup;
use criterion::Criterion;
use criterion::Throughput;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::measurement::WallTime;
use qubit_codec::CapacityError;
use qubit_codec::Codec;
use qubit_codec::CodecTranscodeDecoder;
use qubit_codec::DecodeFailure;
use qubit_codec::TranscodeDecodeError;
use qubit_codec::TranscodeEncodeErrorOf;
use qubit_codec::TranscodeProgress;
use qubit_codec::TranscodeStatus;
use qubit_codec::Transcoder;
use qubit_codec::engine::DecodeContext;
use qubit_codec::engine::DecodeInvalidAction;
use qubit_codec::engine::EncodeContext;
use qubit_codec::engine::EncodeUnencodableAction;
use qubit_codec::engine::TranscodeConvertEngine;
use qubit_codec::engine::TranscodeDecodeEngine;
use qubit_codec::engine::TranscodeDecodeHooks;
use qubit_codec::engine::TranscodeEncodeEngine;
use qubit_codec::engine::TranscodeEncodeHooks;
use qubit_utils as utils_crate;

const FIXTURE_LEN: usize = 64 * 1024;
const SAMPLE_SIZE: usize = 20;

/// Copies input units into the supplied output range.
#[derive(Default)]
struct CopyTranscoder;

#[derive(Default)]
struct CopyCodec;

struct VariableWidthCodec;

struct StrictEncodeHooks;

impl Codec for CopyCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
        Ok((input[input_index], NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        output[output_index] = *value;
        Ok(1)
    }
}

impl Codec for VariableWidthCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 2;
    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

    fn encode_len(&self, value: &u8) -> usize {
        if *value < 128 { 1 } else { 2 }
    }

    unsafe fn decode(&mut self, input: &[u8], index: usize) -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> {
        if input[index] != 0xff {
            return Ok((input[index], NonZeroUsize::MIN));
        }
        if input.len() - index < 2 {
            return Err(DecodeFailure::incomplete(utils_crate::nonzero(2)));
        }
        Ok((input[index + 1], utils_crate::nonzero(2)))
    }

    unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Infallible> {
        if *value < 128 {
            output[index] = *value;
            Ok(1)
        } else {
            output[index..index + 2].copy_from_slice(&[0xff, *value]);
            Ok(2)
        }
    }
}

impl TranscodeEncodeHooks<VariableWidthCodec> for StrictEncodeHooks {
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut VariableWidthCodec,
        _context: &EncodeContext<'_, u8>,
    ) -> Result<EncodeUnencodableAction<u8>, TranscodeEncodeErrorOf<VariableWidthCodec>> {
        Ok(EncodeUnencodableAction::Reject)
    }
}

struct CopyHooks;

impl TranscodeDecodeHooks<CopyCodec> for CopyHooks {
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut CopyCodec,
        error: &Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, TranscodeDecodeError<Infallible>> {
        match *error {}
    }
}

impl Transcoder for CopyTranscoder {
    type Input = u8;
    type Output = u8;
    type Error = TranscodeDecodeError<Infallible>;

    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn reset(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::Error> {
        Ok(0)
    }

    fn transcode(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        let input = &input[input_index..];
        let output = &mut output[output_index..];
        let count = input.len().min(output.len());
        output[..count].copy_from_slice(&input[..count]);
        if count == input.len() {
            Ok(TranscodeProgress::complete(count, count))
        } else {
            Ok(TranscodeProgress::need_output(utils_crate::nonzero(1), count, count))
        }
    }

    fn finish(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::Error> {
        Ok(0)
    }
}

/// Benchmarks direct and complete-lifecycle transcode paths.
fn bench_complete_paths(group: &mut BenchmarkGroup<'_, WallTime>, input: &[u8]) {
    let mut transcoder = CopyTranscoder;
    let mut output = vec![0_u8; input.len()];
    group.bench_function("direct_transcode", |bencher| {
        bencher.iter(|| {
            let progress = transcoder
                .transcode(black_box(input), 0, output.as_mut_slice(), 0)
                .expect("copy transcoder is infallible");
            black_box((progress.read(), progress.written()));
        });
    });

    group.bench_function("complete_lifecycle", |bencher| {
        bencher.iter(|| {
            let written = transcoder
                .transcode_complete_into(black_box(input), output.as_mut_slice())
                .expect("copy transcoder is infallible");
            black_box((written, output[0]));
        });
    });
}

/// Benchmarks the production codec-backed decoder engine hot path.
fn bench_decode_engine(group: &mut BenchmarkGroup<'_, WallTime>, input: &[u8]) {
    let mut decoder = TranscodeDecodeEngine::new(CopyCodec, CopyHooks);
    let mut reset_output = [];
    decoder
        .reset(&mut reset_output, 0)
        .expect("copy codec reset is infallible");
    let mut output = vec![0_u8; input.len()];
    group.bench_function("decode_engine", |bencher| {
        bencher.iter(|| {
            let progress = decoder
                .transcode(black_box(input), 0, output.as_mut_slice(), 0)
                .expect("copy codec is infallible");
            black_box((progress.read(), progress.written()));
        });
    });
}

/// Copies the fixture with ordinary safe indexing.
#[inline(never)]
fn copy_safe_exact(input: &[u8], output: &mut [u8]) {
    for (source, destination) in input.iter().zip(output.iter_mut()) {
        *destination = *source;
    }
}

/// Benchmarks the public codec adapter against an equivalent safe copy loop.
fn bench_codec_adapter(group: &mut BenchmarkGroup<'_, WallTime>, input: &[u8]) {
    let mut decoder = CodecTranscodeDecoder::new(CopyCodec);
    let mut reset_output = [];
    decoder
        .reset(&mut reset_output, 0)
        .expect("copy codec reset is infallible");
    let mut adapter_output = vec![0_u8; input.len()];
    group.bench_function("codec_adapter", |bencher| {
        bencher.iter(|| {
            let progress = decoder
                .transcode(black_box(input), 0, adapter_output.as_mut_slice(), 0)
                .expect("copy codec is infallible");
            black_box((progress.read(), progress.written(), adapter_output[0]));
        });
    });

    let mut safe_output = vec![0_u8; input.len()];
    group.bench_function("safe_copy_loop", |bencher| {
        bencher.iter(|| {
            copy_safe_exact(black_box(input), black_box(safe_output.as_mut_slice()));
            black_box(safe_output[0]);
        });
    });
}

/// Benchmarks the same conversion under intentionally small output windows.
fn bench_streaming_windows(group: &mut BenchmarkGroup<'_, WallTime>, input: &[u8]) {
    for window in [32_usize, 1024] {
        let name = format!("output_window_{window}");
        group.bench_function(name, |bencher| {
            let mut transcoder = CopyTranscoder;
            let mut output = vec![0_u8; window];
            bencher.iter(|| {
                let mut input_index = 0;
                let mut written_total = 0;
                while input_index < input.len() {
                    let progress = transcoder
                        .transcode(black_box(input), input_index, output.as_mut_slice(), 0)
                        .expect("copy transcoder is infallible");
                    input_index += progress.read();
                    written_total += progress.written();
                }
                black_box((input_index, written_total));
            });
        });
    }
}

/// Drives a complete visible input through reusable output windows.
fn run_output_windows<T>(transcoder: &mut T, input: &[u8], output: &mut [u8]) -> usize
where
    T: Transcoder<Input = u8, Output = u8>,
    T::Error: Debug,
{
    let mut input_index = 0;
    let mut written_total = 0;
    while input_index < input.len() {
        let progress = transcoder
            .transcode(black_box(input), input_index, output, 0)
            .expect("copy conversion is infallible");
        black_box(&output[..progress.written()]);
        input_index += progress.read();
        written_total += progress.written();
    }
    black_box((input_index, written_total));
    written_total
}

/// Checks output and status transitions outside timed iterations.
fn verify_output_windows<T>(transcoder: &mut T, input: &[u8], window: usize)
where
    T: Transcoder<Input = u8, Output = u8>,
    T::Error: Debug,
{
    let mut output = vec![0_u8; window];
    let mut decoded = Vec::with_capacity(input.len());
    let mut input_index = 0;
    while input_index < input.len() {
        let progress = transcoder
            .transcode(input, input_index, &mut output, 0)
            .expect("copy conversion is infallible");
        assert!(progress.read() > 0);
        decoded.extend_from_slice(&output[..progress.written()]);
        input_index += progress.read();
        if input_index == input.len() {
            assert!(progress.is_complete());
        } else {
            assert!(progress.is_need_output());
        }
    }
    assert_eq!(input.len(), input_index);
    assert_eq!(input, decoded.as_slice());
}

/// Benchmarks codec-backed production decoders with small output windows.
fn bench_production_streaming_windows(group: &mut BenchmarkGroup<'_, WallTime>, input: &[u8]) {
    for window in [32_usize, 1024] {
        let mut decoder = TranscodeDecodeEngine::new(CopyCodec, CopyHooks);
        decoder.reset(&mut [], 0).expect("copy codec reset is infallible");
        verify_output_windows(&mut decoder, input, window);
        let mut output = vec![0_u8; window];
        group.bench_function(format!("decode_engine_output_window_{window}"), |bencher| {
            bencher.iter(|| run_output_windows(&mut decoder, input, &mut output));
        });

        let mut adapter = CodecTranscodeDecoder::new(CopyCodec);
        adapter.reset(&mut [], 0).expect("copy codec reset is infallible");
        verify_output_windows(&mut adapter, input, window);
        let mut adapter_output = vec![0_u8; window];
        group.bench_function(format!("codec_adapter_output_window_{window}"), |bencher| {
            bencher.iter(|| run_output_windows(&mut adapter, input, &mut adapter_output));
        });
    }
}

fn expected_variable_output(input: &[u8]) -> Vec<u8> {
    let mut expected = Vec::with_capacity(input.len() * 2);
    for &value in input {
        if value >= 128 {
            expected.push(0xff);
        }
        expected.push(value);
    }
    expected
}

fn run_engine_windows<T>(engine: &mut T, input: &[u8], output: &mut [u8]) -> usize
where
    T: Transcoder<Input = u8, Output = u8>,
    T::Error: Debug,
{
    let mut cursor = 0;
    let mut written = 0;
    loop {
        let progress = engine
            .transcode(black_box(input), cursor, output, 0)
            .expect("benchmark fixture conversion succeeds");
        black_box(&output[..progress.written()]);
        cursor += progress.read();
        written += progress.written();
        match progress.status() {
            TranscodeStatus::Complete => break,
            TranscodeStatus::NeedOutput { .. } => {}
            TranscodeStatus::NeedInput { .. } => panic!("complete fixture needs no input"),
        }
    }
    black_box((cursor, written));
    written
}

fn verify_variable_windows<T>(engine: &mut T, input: &[u8], expected: &[u8], window: usize)
where
    T: Transcoder<Input = u8, Output = u8>,
    T::Error: Debug,
{
    let mut output = vec![0_u8; window];
    assert_eq!(0, engine.reset(&mut output, 0).expect("reset"));
    let mut cursor = 0;
    let mut actual = Vec::with_capacity(expected.len());
    let mut saw_need_output = false;
    loop {
        let progress = engine.transcode(input, cursor, &mut output, 0).expect("main");
        progress
            .validate(cursor, input.len() - cursor, 0, output.len())
            .expect("valid main progress");
        actual.extend_from_slice(&output[..progress.written()]);
        cursor += progress.read();
        match progress.status() {
            TranscodeStatus::Complete => break,
            TranscodeStatus::NeedOutput { .. } => {
                saw_need_output = true;
                assert!(progress.read() > 0 || progress.written() > 0);
            }
            TranscodeStatus::NeedInput { .. } => panic!("complete input"),
        }
    }
    assert_eq!(input.len(), cursor);
    loop {
        let progress = engine.transcode_eof(input, cursor, &mut output, 0).expect("EOF");
        progress
            .validate(cursor, input.len() - cursor, 0, output.len())
            .expect("valid EOF progress");
        actual.extend_from_slice(&output[..progress.written()]);
        cursor += progress.read();
        match progress.status() {
            TranscodeStatus::Complete => break,
            TranscodeStatus::NeedOutput { .. } => {
                assert!(progress.read() > 0 || progress.written() > 0);
            }
            TranscodeStatus::NeedInput { .. } => panic!("EOF needs no input"),
        }
    }
    assert_eq!(0, engine.finish(&mut output, 0).expect("finish"));
    assert_eq!(expected, actual.as_slice());
    if window < expected.len() {
        assert!(saw_need_output);
    }
}

fn bench_variable_engine<F, T>(
    group: &mut BenchmarkGroup<'_, WallTime>,
    name: &str,
    input: &[u8],
    expected: &[u8],
    window: usize,
    make_engine: F,
) where
    F: Fn() -> T,
    T: Transcoder<Input = u8, Output = u8>,
    T::Error: Debug,
{
    verify_variable_windows(&mut make_engine(), input, expected, window);
    let mut engine = make_engine();
    engine.reset(&mut [], 0).expect("timed engine reset");
    let mut output = vec![0_u8; window];
    group.bench_function(name, |bencher| {
        bencher.iter(|| run_engine_windows(&mut engine, input, &mut output));
    });
}

fn bench_variable_paths(criterion: &mut Criterion, input: &[u8]) {
    let expected = expected_variable_output(input);
    assert_eq!(98_304, expected.len());
    let mut pending = TranscodeConvertEngine::new(CopyCodec, VariableWidthCodec, CopyHooks, StrictEncodeHooks);
    pending.reset(&mut [], 0).expect("pending reset");
    let progress = pending.transcode(&[128], 0, &mut [0_u8; 1], 0).expect("retain");
    assert!(progress.is_need_output());
    assert_eq!((1, 0), (progress.read(), progress.written()));
    let mut drained = [0_u8; 2];
    let progress = pending.transcode(&[128], 1, &mut drained, 0).expect("drain");
    assert!(progress.is_complete());
    assert_eq!((0, 2), (progress.read(), progress.written()));
    assert_eq!([0xff, 128], drained);

    let mut group = criterion.benchmark_group("engine_encode_convert");
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.sample_size(SAMPLE_SIZE);
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(5));
    for (suffix, window) in [
        ("full", expected.len()),
        ("window_2", 2),
        ("window_32", 32),
        ("window_1024", 1024),
    ] {
        bench_variable_engine(
            &mut group,
            &format!("encode_{suffix}"),
            input,
            &expected,
            window,
            || TranscodeEncodeEngine::new(VariableWidthCodec, StrictEncodeHooks),
        );
        bench_variable_engine(
            &mut group,
            &format!("convert_{suffix}"),
            input,
            &expected,
            window,
            || TranscodeConvertEngine::new(CopyCodec, VariableWidthCodec, CopyHooks, StrictEncodeHooks),
        );
    }
    group.finish();
}

#[inline(never)]
fn copy_safe(input: &[u8], output: &mut [u8]) {
    for index in 0..input.len() {
        output[index] = input[index].wrapping_add(1);
    }
}

/// Copies the same fixture with unchecked indexing for an A/B code-generation
/// comparison. The caller guarantees equal input and output lengths.
#[inline(never)]
unsafe fn copy_unchecked(input: &[u8], output: &mut [u8]) {
    for index in 0..input.len() {
        // SAFETY: `index` is bounded by both slices' equal lengths.
        let source = unsafe { *input.get_unchecked(index) };
        // SAFETY: `index` is bounded by the equal output length.
        unsafe { *output.get_unchecked_mut(index) = source.wrapping_add(1) };
    }
}

/// Compares safe indexing with explicit unchecked indexing on the same loop.
fn bench_safe_vs_unchecked(group: &mut BenchmarkGroup<'_, WallTime>, input: &[u8]) {
    let mut safe_output = vec![0_u8; input.len()];
    group.bench_function("safe_indexing", |bencher| {
        bencher.iter(|| {
            copy_safe(black_box(input), black_box(safe_output.as_mut_slice()));
            black_box(safe_output[0]);
        });
    });

    let mut unchecked_output = vec![0_u8; input.len()];
    assert_eq!(input.len(), unchecked_output.len());
    group.bench_function("unchecked_indexing", |bencher| {
        bencher.iter(|| {
            // SAFETY: both output buffers are allocated to `input.len()`.
            unsafe { copy_unchecked(black_box(input), black_box(unchecked_output.as_mut_slice())) };
            black_box(unchecked_output[0]);
        });
    });
}

/// Registers generic transcoder baseline measurements.
fn bench_transcoder(criterion: &mut Criterion) {
    let input: Vec<u8> = (0..FIXTURE_LEN).map(|index| index as u8).collect();
    let mut complete = criterion.benchmark_group("transcoder_complete");
    complete.throughput(Throughput::Bytes(input.len() as u64));
    complete.sample_size(SAMPLE_SIZE);
    complete.warm_up_time(Duration::from_secs(2));
    complete.measurement_time(Duration::from_secs(5));
    bench_complete_paths(&mut complete, &input);
    bench_decode_engine(&mut complete, &input);
    bench_codec_adapter(&mut complete, &input);
    complete.finish();

    let mut streaming = criterion.benchmark_group("transcoder_streaming");
    streaming.throughput(Throughput::Bytes(input.len() as u64));
    streaming.sample_size(SAMPLE_SIZE);
    streaming.warm_up_time(Duration::from_secs(2));
    streaming.measurement_time(Duration::from_secs(5));
    bench_streaming_windows(&mut streaming, &input);
    bench_production_streaming_windows(&mut streaming, &input);
    streaming.finish();

    let mut indexing = criterion.benchmark_group("safe_vs_unchecked");
    indexing.throughput(Throughput::Bytes(input.len() as u64));
    indexing.sample_size(SAMPLE_SIZE);
    indexing.warm_up_time(Duration::from_secs(2));
    indexing.measurement_time(Duration::from_secs(5));
    bench_safe_vs_unchecked(&mut indexing, &input);
    indexing.finish();

    bench_variable_paths(criterion, &input);
}

criterion_group!(benches, bench_transcoder);
criterion_main!(benches);
