// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Baseline benchmarks for the generic `Transcoder` lifecycle.

use std::{
    convert::Infallible,
    hint::black_box,
    time::Duration,
};

use criterion::{
    BenchmarkGroup,
    Criterion,
    Throughput,
    criterion_group,
    criterion_main,
    measurement::WallTime,
};
use qubit_codec::{
    CapacityError,
    Codec,
    CodecTranscodeDecoder,
    DecodeFailure,
    TranscodeDecodeError,
    TranscodeProgress,
    Transcoder,
    engine::{
        DecodeContext,
        DecodeInvalidAction,
        TranscodeDecodeEngine,
        TranscodeDecodeHooks,
    },
};

const FIXTURE_LEN: usize = 64 * 1024;
const SAMPLE_SIZE: usize = 20;

/// Copies input units into the supplied output range.
#[derive(Default)]
struct CopyTranscoder;

#[derive(Default)]
struct CopyCodec;

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
    ) -> Result<(u8, core::num::NonZeroUsize), DecodeFailure<Self::DecodeError>>
    {
        Ok((input[input_index], core::num::NonZeroUsize::MIN))
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

struct CopyHooks;

impl TranscodeDecodeHooks<CopyCodec> for CopyHooks {
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut CopyCodec,
        error: &Infallible,
        _consumed: Option<core::num::NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, TranscodeDecodeError<Infallible>> {
        match *error {}
    }
}

impl Transcoder for CopyTranscoder {
    type Input = u8;
    type Output = u8;
    type Error = TranscodeDecodeError<Infallible>;

    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn reset(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::Error> {
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
            Ok(TranscodeProgress::need_output(
                qubit_utils::nonzero(1),
                count,
                count,
            ))
        }
    }

    fn finish(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::Error> {
        Ok(0)
    }
}

/// Benchmarks direct and complete-lifecycle transcode paths.
fn bench_complete_paths(
    group: &mut BenchmarkGroup<'_, WallTime>,
    input: &[u8],
) {
    let mut transcoder = CopyTranscoder;
    let mut output = vec![0_u8; input.len()];
    group.bench_function("direct", |bencher| {
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
                .transcode_complete_into(
                    black_box(input),
                    output.as_mut_slice(),
                )
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
                .transcode(
                    black_box(input),
                    0,
                    adapter_output.as_mut_slice(),
                    0,
                )
                .expect("copy codec is infallible");
            black_box((progress.read(), progress.written(), adapter_output[0]));
        });
    });

    let mut safe_output = vec![0_u8; input.len()];
    group.bench_function("safe_copy_loop", |bencher| {
        bencher.iter(|| {
            copy_safe_exact(
                black_box(input),
                black_box(safe_output.as_mut_slice()),
            );
            black_box(safe_output[0]);
        });
    });
}

/// Benchmarks the same conversion under intentionally small output windows.
fn bench_streaming_windows(
    group: &mut BenchmarkGroup<'_, WallTime>,
    input: &[u8],
) {
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
                        .transcode(
                            black_box(input),
                            input_index,
                            output.as_mut_slice(),
                            0,
                        )
                        .expect("copy transcoder is infallible");
                    input_index += progress.read();
                    written_total += progress.written();
                }
                black_box((input_index, written_total));
            });
        });
    }
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
    debug_assert_eq!(input.len(), output.len());
    for index in 0..input.len() {
        // SAFETY: `index` is bounded by both slices' equal lengths.
        let source = unsafe { *input.get_unchecked(index) };
        // SAFETY: `index` is bounded by the equal output length.
        unsafe { *output.get_unchecked_mut(index) = source.wrapping_add(1) };
    }
}

/// Compares safe indexing with explicit unchecked indexing on the same loop.
fn bench_safe_vs_unchecked(
    group: &mut BenchmarkGroup<'_, WallTime>,
    input: &[u8],
) {
    let mut safe_output = vec![0_u8; input.len()];
    group.bench_function("safe_indexing", |bencher| {
        bencher.iter(|| {
            copy_safe(black_box(input), black_box(safe_output.as_mut_slice()));
            black_box(safe_output[0]);
        });
    });

    let mut unchecked_output = vec![0_u8; input.len()];
    group.bench_function("unchecked_indexing", |bencher| {
        bencher.iter(|| {
            // SAFETY: both output buffers are allocated to `input.len()`.
            unsafe {
                copy_unchecked(
                    black_box(input),
                    black_box(unchecked_output.as_mut_slice()),
                )
            };
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
    streaming.finish();

    let mut indexing = criterion.benchmark_group("safe_vs_unchecked");
    indexing.throughput(Throughput::Bytes(input.len() as u64));
    indexing.sample_size(SAMPLE_SIZE);
    indexing.warm_up_time(Duration::from_secs(2));
    indexing.measurement_time(Duration::from_secs(5));
    bench_safe_vs_unchecked(&mut indexing, &input);
    indexing.finish();
}

criterion_group!(benches, bench_transcoder);
criterion_main!(benches);
