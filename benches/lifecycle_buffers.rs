// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Measures owned and reusable scratch preparation for decode lifecycles.

use std::convert::Infallible;
use std::hint::black_box;
use std::num::NonZeroUsize;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;
use qubit_codec::Codec;
use qubit_codec::CodecTranscodeConverter;
use qubit_codec::CodecValueDecoder;
use qubit_codec::DecodeFailure;

static DEFAULT_CALLS: AtomicUsize = AtomicUsize::new(0);

/// Values used by the benchmark; tags make phase ordering observable.
trait BenchValue: Clone + Default + PartialEq {
    fn tagged(tag: u8) -> Self;
    fn tag(&self) -> u8;
}

impl BenchValue for u8 {
    fn tagged(tag: u8) -> Self {
        tag
    }

    fn tag(&self) -> u8 {
        *self
    }
}

/// Simulates a value whose default construction allocates owned resources.
#[derive(Clone, Debug, PartialEq)]
struct ResourceValue {
    tag: u8,
    resource: Vec<u8>,
}

impl Default for ResourceValue {
    fn default() -> Self {
        Self {
            tag: 0,
            resource: vec![0; 64],
        }
    }
}

impl BenchValue for ResourceValue {
    fn tagged(tag: u8) -> Self {
        Self {
            tag,
            resource: vec![tag; 64],
        }
    }

    fn tag(&self) -> u8 {
        self.tag
    }
}

/// A counted default used only by the untimed call-count probe.
#[derive(Clone, Debug, PartialEq)]
struct CountedValue(u8);

impl Default for CountedValue {
    fn default() -> Self {
        DEFAULT_CALLS.fetch_add(1, Ordering::Relaxed);
        Self(0)
    }
}

impl BenchValue for CountedValue {
    fn tagged(tag: u8) -> Self {
        Self(tag)
    }

    fn tag(&self) -> u8 {
        self.0
    }
}

/// Emits a configurable number of reset and finish values around one byte.
struct LifecycleCodec<V, const BOUND: usize> {
    reset_count: usize,
    finish_count: usize,
    marker: std::marker::PhantomData<V>,
}

impl<V, const BOUND: usize> LifecycleCodec<V, BOUND> {
    fn new(reset_count: usize, finish_count: usize) -> Self {
        Self {
            reset_count,
            finish_count,
            marker: std::marker::PhantomData,
        }
    }
}

impl<V: BenchValue, const BOUND: usize> Codec for LifecycleCodec<V, BOUND> {
    type Value = V;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_RESET_VALUES: usize = BOUND;
    const MAX_DECODE_FINISH_VALUES: usize = BOUND;
    const MAX_ENCODE_RESET_UNITS: usize = BOUND;
    const MAX_ENCODE_FINISH_UNITS: usize = BOUND;

    unsafe fn decode(&mut self, input: &[u8], index: usize) -> Result<(V, NonZeroUsize), DecodeFailure<Infallible>> {
        Ok((V::tagged(input[index]), NonZeroUsize::MIN))
    }

    unsafe fn encode(&mut self, value: &V, output: &mut [u8], index: usize) -> Result<usize, Infallible> {
        output[index] = value.tag();
        Ok(1)
    }

    unsafe fn decode_reset(&mut self, output: &mut [V], index: usize) -> Result<usize, Infallible> {
        for offset in 0..self.reset_count {
            output[index + offset] = V::tagged(0x10 + offset as u8);
        }
        Ok(self.reset_count)
    }

    unsafe fn decode_finish(&mut self, output: &mut [V], index: usize) -> Result<usize, Infallible> {
        for offset in 0..self.finish_count {
            output[index + offset] = V::tagged(0x80 + offset as u8);
        }
        Ok(self.finish_count)
    }

    unsafe fn encode_reset(&mut self, output: &mut [u8], index: usize) -> Result<usize, Infallible> {
        for offset in 0..self.reset_count {
            output[index + offset] = 0x20 + offset as u8;
        }
        Ok(self.reset_count)
    }

    unsafe fn encode_finish(&mut self, output: &mut [u8], index: usize) -> Result<usize, Infallible> {
        for offset in 0..self.finish_count {
            output[index + offset] = 0x90 + offset as u8;
        }
        Ok(self.finish_count)
    }
}

const INPUT: [u8; 1] = [0x42];

fn counts(bound: usize) -> Vec<usize> {
    let mut values = vec![0];
    if bound >= 1 {
        values.push(1);
    }
    if bound > 1 {
        values.push(bound);
    }
    values
}

fn check_output<V: BenchValue, const BOUND: usize>(reset: usize, finish: usize) {
    let mut decoder = CodecValueDecoder::new(LifecycleCodec::<V, BOUND>::new(reset, finish));
    let output = decoder.decode_lifecycle(&INPUT).expect("synthetic lifecycle succeeds");
    assert_eq!(output.reset().len(), reset);
    assert_eq!(output.value().tag(), INPUT[0]);
    assert_eq!(output.finish().len(), finish);
    for (index, value) in output.reset().iter().enumerate() {
        assert_eq!(value.tag(), 0x10 + index as u8);
    }
    for (index, value) in output.finish().iter().enumerate() {
        assert_eq!(value.tag(), 0x80 + index as u8);
    }
    let _ = black_box(output);
}

fn check_scratch_and_default_counts<const BOUND: usize>(reset: usize, finish: usize) {
    let mut decoder = CodecValueDecoder::new(LifecycleCodec::<CountedValue, BOUND>::new(reset, finish));
    let mut reset_scratch = vec![CountedValue::default(); BOUND];
    let mut finish_scratch = vec![CountedValue::default(); BOUND];
    DEFAULT_CALLS.store(0, Ordering::Relaxed);
    let progress = decoder
        .decode_lifecycle_with_scratch(&INPUT, &mut reset_scratch, &mut finish_scratch)
        .expect("scratch lifecycle succeeds");
    assert_eq!(
        DEFAULT_CALLS.load(Ordering::Relaxed),
        0,
        "scratch API must not call Default"
    );
    assert_eq!(progress.value().tag(), INPUT[0]);
    assert_eq!(progress.reset_written(), reset);
    assert_eq!(progress.finish_written(), finish);
    assert_eq!(
        reset_scratch[..reset]
            .iter()
            .map(|value| value.tag())
            .collect::<Vec<_>>(),
        (0..reset).map(|i| 0x10 + i as u8).collect::<Vec<_>>()
    );
    assert_eq!(
        finish_scratch[..finish]
            .iter()
            .map(|value| value.tag())
            .collect::<Vec<_>>(),
        (0..finish).map(|i| 0x80 + i as u8).collect::<Vec<_>>()
    );

    DEFAULT_CALLS.store(0, Ordering::Relaxed);
    let output = decoder.decode_lifecycle(&INPUT).expect("owned lifecycle succeeds");
    assert_eq!(
        DEFAULT_CALLS.load(Ordering::Relaxed),
        BOUND * 2,
        "owned API defaults both declared buffers"
    );
    assert_eq!(output.reset().len(), reset);
    assert_eq!(output.finish().len(), finish);
}

fn check_converter_phases<const BOUND: usize>(count: usize) {
    let mut converter = CodecTranscodeConverter::new(
        LifecycleCodec::<u8, BOUND>::new(count, count),
        LifecycleCodec::<u8, BOUND>::new(count, count),
    );
    let mut output = vec![0_u8; BOUND * 2 + 1];
    let reset_written = converter.reset(&mut output, 0).expect("converter reset precheck");
    assert_eq!(reset_written, count * 2);
    for index in 0..count {
        assert_eq!(output[index], 0x20 + index as u8);
        assert_eq!(output[count + index], 0x10 + index as u8);
    }

    let finish_written = converter.finish(&mut output, 0).expect("converter finish precheck");
    assert_eq!(finish_written, count * 2);
    for index in 0..count {
        assert_eq!(output[index], 0x80 + index as u8);
        assert_eq!(output[count + index], 0x90 + index as u8);
    }
    black_box(output);
}

fn bench_bound<V: BenchValue, const BOUND: usize>(criterion: &mut Criterion, value_name: &str) {
    for reset_count in counts(BOUND) {
        for finish_count in counts(BOUND) {
            check_output::<V, BOUND>(reset_count, finish_count);
            check_scratch_and_default_counts::<BOUND>(reset_count, finish_count);
            let id = format!("{value_name}/reset-{reset_count}/finish-{finish_count}");
            let mut group = criterion.benchmark_group(format!("decode_lifecycle/bound-{BOUND}"));

            group.bench_function(BenchmarkId::new("owned", &id), |bencher| {
                let mut decoder = CodecValueDecoder::new(LifecycleCodec::<V, BOUND>::new(reset_count, finish_count));
                bencher.iter(|| {
                    let output = decoder.decode_lifecycle(black_box(&INPUT)).expect("decode lifecycle");
                    black_box(output)
                });
            });

            group.bench_function(BenchmarkId::new("reusable_scratch", &id), |bencher| {
                let mut decoder = CodecValueDecoder::new(LifecycleCodec::<V, BOUND>::new(reset_count, finish_count));
                let mut reset = vec![V::default(); BOUND];
                let mut finish = vec![V::default(); BOUND];
                bencher.iter(|| {
                    let progress = decoder
                        .decode_lifecycle_with_scratch(black_box(&INPUT), black_box(&mut reset), black_box(&mut finish))
                        .expect("decode lifecycle with scratch");
                    let _ = black_box(progress);
                    black_box(&reset);
                    black_box(&finish);
                });
            });
            group.finish();
        }
    }
}

fn bench_converter_phases(criterion: &mut Criterion) {
    for bound in [0, 1, 64, 1024] {
        macro_rules! run_bound {
            ($bound:expr) => {{
                for count in counts($bound) {
                    check_converter_phases::<$bound>(count);
                    let id = format!("bound-{}/count-{count}", $bound);
                    let mut group = criterion.benchmark_group("converter_lifecycle_phase");
                    group.bench_function(BenchmarkId::new("reset", &id), |bencher| {
                        bencher.iter_batched(
                            || {
                                CodecTranscodeConverter::new(
                                    LifecycleCodec::<u8, $bound>::new(count, count),
                                    LifecycleCodec::<u8, $bound>::new(count, count),
                                )
                            },
                            |mut converter| {
                                let mut output = vec![0_u8; $bound * 2 + 1];
                                black_box(converter.reset(&mut output, 0).expect("converter reset"));
                                black_box(output)
                            },
                            criterion::BatchSize::SmallInput,
                        );
                    });
                    group.bench_function(BenchmarkId::new("finish", &id), |bencher| {
                        bencher.iter_batched(
                            || {
                                let mut converter = CodecTranscodeConverter::new(
                                    LifecycleCodec::<u8, $bound>::new(count, count),
                                    LifecycleCodec::<u8, $bound>::new(count, count),
                                );
                                let mut output = vec![0_u8; $bound * 2 + 1];
                                converter.reset(&mut output, 0).expect("converter setup reset");
                                (converter, output)
                            },
                            |(mut converter, mut output)| {
                                black_box(converter.finish(&mut output, 0).expect("converter finish"));
                                black_box(output)
                            },
                            criterion::BatchSize::SmallInput,
                        );
                    });
                    group.finish();
                }
            }};
        }
        match bound {
            0 => run_bound!(0),
            1 => run_bound!(1),
            64 => run_bound!(64),
            1024 => run_bound!(1024),
            _ => unreachable!(),
        }
    }
}

fn lifecycle_buffers(criterion: &mut Criterion) {
    bench_bound::<u8, 0>(criterion, "cheap_default");
    bench_bound::<u8, 1>(criterion, "cheap_default");
    bench_bound::<u8, 64>(criterion, "cheap_default");
    bench_bound::<u8, 1024>(criterion, "cheap_default");
    bench_bound::<ResourceValue, 0>(criterion, "resource_default");
    bench_bound::<ResourceValue, 1>(criterion, "resource_default");
    bench_bound::<ResourceValue, 64>(criterion, "resource_default");
    bench_bound::<ResourceValue, 1024>(criterion, "resource_default");
    bench_converter_phases(criterion);
}

criterion_group!(benches, lifecycle_buffers);
criterion_main!(benches);
