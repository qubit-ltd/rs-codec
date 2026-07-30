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
    TranscodeProgress,
    Transcoder,
};

const FIXTURE_LEN: usize = 64 * 1024;
const SAMPLE_SIZE: usize = 20;

/// Copies input units into the supplied output range.
#[derive(Default)]
struct CopyTranscoder;

impl Transcoder for CopyTranscoder {
    type Input = u8;
    type Output = u8;
    type Error = Infallible;

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
                output_index + count,
                qubit_codec::nz(1),
                output.len() - count,
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

/// Registers generic transcoder baseline measurements.
fn bench_transcoder(criterion: &mut Criterion) {
    let input: Vec<u8> = (0..FIXTURE_LEN).map(|index| index as u8).collect();
    let mut complete = criterion.benchmark_group("transcoder_complete");
    complete.throughput(Throughput::Bytes(input.len() as u64));
    complete.sample_size(SAMPLE_SIZE);
    complete.warm_up_time(Duration::from_secs(2));
    complete.measurement_time(Duration::from_secs(5));
    bench_complete_paths(&mut complete, &input);
    complete.finish();

    let mut streaming = criterion.benchmark_group("transcoder_streaming");
    streaming.throughput(Throughput::Bytes(input.len() as u64));
    streaming.sample_size(SAMPLE_SIZE);
    streaming.warm_up_time(Duration::from_secs(2));
    streaming.measurement_time(Duration::from_secs(5));
    bench_streaming_windows(&mut streaming, &input);
    streaming.finish();
}

criterion_group!(benches, bench_transcoder);
criterion_main!(benches);
