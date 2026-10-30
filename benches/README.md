# Transcode benchmarks

Run these commands from the `rs-codec` repository root. Compare the public
codec adapter with a direct safe copy loop using:

```bash
cargo bench --bench transcode -- codec_adapter
cargo bench --bench transcode -- safe_copy_loop
```

This measures the complete `CodecTranscodeDecoder` path, including progress
bookkeeping and lifecycle state, against an equivalent safe copy loop. The
buffers and reset setup are allocated outside the timed iterations.

Run the safe/unchecked indexing A/B benchmark with:

```bash
cargo bench --bench transcode -- safe_vs_unchecked
```

The benchmark compares the same copy loop implemented with safe indexing and
explicit `get_unchecked` access. It is a code-generation probe, not
a correctness test; the safe version is the default choice unless assembly
shows a measurable regression in a real hot path.

Emit optimized assembly for inspection with:

```bash
cargo rustc --release --bench transcode -- --emit=asm
```

Inspect the generated `target/release/deps/transcode-*.s` files and compare the
`copy_safe` and `copy_unchecked` symbols. On toolchains with `cargo-asm`, the
equivalent focused commands are:

```bash
cargo asm --bench transcode copy_safe
cargo asm --bench transcode copy_unchecked
```

## Streaming output windows

Run the 32-byte and 1024-byte output window groups with:

```bash
cargo bench --locked --bench transcode -- output_window
```

The `transcoder_streaming` group keeps the `output_window_32` and
`output_window_1024` `CopyTranscoder` baselines. It also measures the
production `TranscodeDecodeEngine` paths as `decode_engine_output_window_32`
and `decode_engine_output_window_1024`, and the public
`CodecTranscodeDecoder` paths as `codec_adapter_output_window_32` and
`codec_adapter_output_window_1024`. Each group processes the same 64 KiB
fixture. Output buffers and decoder reset are prepared outside the timed
iterations; each written window is observed before the buffer is reused.
Correctness and progress-state checks run once outside timing and verify that
the full output matches the input, with `NeedOutput` between windows and
`Complete` at the end.

### RC-009 follow-up inspection

A follow-up inspection on 2026-10-30 used the unmodified baseline
`2cad737661dc7d3d888372d38da77608e33d777c` with rustc 1.94.0, LLVM 21.1.8,
and an Intel Core i5-9600K. The bench profile remained
`opt-level=3`, `lto=true`, and `codegen-units=1`. The exact default-profile
command was `cargo bench --locked --bench transcode --no-run
--message-format=json`; Cargo's JSON artifact identified
`target/release/deps/transcode-518bb86cbd866b72` (SHA-256
`0732fc56759a44440e9abf92dc60f7f54a1e4b0a2087fcf42496f424969acb97`).
`objdump -Cd` and `nm -C` confirmed that LTO inlined the benchmark closures
into `transcode::bench_transcoder`, without separate closure symbols.

To locate the timed closures, an auxiliary build used the same bench profile
with `CARGO_PROFILE_BENCH_DEBUG=1` and a separate `CARGO_TARGET_DIR`; its
executable was `transcode-7877a69bede2c061` (SHA-256
`77f37d2367b2d3450f192b4f59a2f3b904ef8ca0fa41755100f2c21414ec4479`). Its
line-table disassembly associated the direct, engine, and old window benchmark
entrances with their Criterion loops, but did not provide enough attribution
to establish the complete output writes in the default-profile executable.
The auxiliary executable alone cannot establish default-profile behavior.
RC-009 therefore remains unresolved: no benchmark code was changed, and
neither elimination nor retention of every old path's full output writes is
claimed. The default and auxiliary artifacts and their disassemblies were
captured under `/tmp/rs-codec-rc009.IBnT95` during this inspection. This
temporary evidence directory is scheduled for cleanup after validation,
integration, and branch synchronization complete. This result is limited to
the recorded toolchain and host and makes no cross-platform claim.

On 2026-10-30, the RC-009 pre-change inspection used HEAD
`4310cde13f9dbf62728d979f2e0983aa8dac154c`, rustc 1.94.0
(`x86_64-unknown-linux-gnu`, LLVM 21.1.8), an Intel Core i5-9600K host, and
the bench profile (`opt-level=3`, `lto=true`, `codegen-units=1`). The Cargo JSON
artifact record, executable path, and `objdump -Cd` output were captured under
`target/review-20261030/` for this inspection. The temporary evidence was
removed after review; the later `align-ci.sh` run had already cleaned the
generated `target/release` directory, so the executable itself was not
retained. LTO left the benchmark closures without distinct
symbols, and the executable has no line-table debug information; the observed
`memcpy` and vector instructions could not be attributed reliably to the
`direct_transcode`, `decode_engine`, or `output_window` timed closures. RC-009
is therefore unresolved for this toolchain: the old observation code was left
unchanged, and these results do not establish whether its writes were
eliminated. The new output-window measurements are a baseline for these
production paths, not evidence of a performance improvement.

The 2026-10-30 run on that host reported these Criterion time intervals:

| Benchmark | Time |
| --- | ---: |
| `output_window_32` | 16.434–22.441 µs |
| `output_window_1024` | 4.7510–7.5959 µs |
| `decode_engine_output_window_32` | 32.452–44.766 µs |
| `decode_engine_output_window_1024` | 15.173–20.706 µs |
| `codec_adapter_output_window_32` | 21.806–30.635 µs |
| `codec_adapter_output_window_1024` | 11.446–17.900 µs |

These are observed confidence intervals from one run, not cross-platform
thresholds. A separate checkout was compiling another Criterion benchmark on
the shared host during this run, so host contention may have affected the
intervals. The window correctness checks completed before timing each group.
