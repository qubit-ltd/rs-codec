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

That output observation applies to the production decode-engine and adapter
window paths. The older `CopyTranscoder` `output_window_32` and
`output_window_1024` paths observe only final read and written counts; they do
not observe each window before reuse.

## Variable-width encode and convert engines

Run the synthetic variable-width encoder and converter workloads with:

```bash
cargo bench --locked --bench transcode -- engine_encode_convert
```

The `engine_encode_convert` group registers `encode_full`, `convert_full`,
`encode_window_2`, `encode_window_32`, `encode_window_1024`,
`convert_window_2`, `convert_window_32`, and `convert_window_1024`. Its 64 KiB
input cycles through all byte values; values below 128 encode as one byte and
the rest as two bytes (`0xff` followed by the value), producing 98,304 output
bytes. This synthetic fixture exercises production encode and convert engines
with variable output widths and backpressure; its timings are not UTF or real
charset performance claims.

Criterion throughput is reported in input bytes. Codec and hook construction,
oracle generation, output allocation, reset, correctness checks, and the
pending-value witness are outside the timed iterations. Each timed output
window is passed to `black_box` before reuse. The converter precheck also
confirms that an input value consumed under one-byte output pressure is later
drained exactly once from pending state.

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
captured under `/tmp/rs-codec-rc009.IBnT95` during this inspection. That
historical inspection marked its temporary evidence directory for cleanup
after its validation, integration, and branch synchronization; this is
separate from the current audit evidence retained below. This result is
limited to the recorded toolchain and host and makes no cross-platform claim.

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

### RC-009 follow-up: default-profile attribution

A further default-profile inspection on 2026-10-30 used the implementation
worktree at base commit `808b1c5f27decc27083c7fa0dd95e67042441e2b`, rustc
1.94.0 (`x86_64-unknown-linux-gnu`, LLVM 21.1.8), and an Intel Core i5-9600K
host. The bench profile was `opt-level=3`, `lto=true`, and
`codegen-units=1`. The exact build command was
`cargo bench --locked --bench transcode --no-run --message-format=json`; Cargo
reported the default-profile executable
`target/release/deps/transcode-518bb86cbd866b72` with SHA-256
`2056f892a1a9d7f95cfb4f63a012ab4f1de0066d929e2cd12c480e62f5d31677`.

`nm -C` exposed `transcode::bench_transcoder` and
`transcode::copy_safe_exact`, but no separate symbols for the timed closure
bodies. `objdump -Cd` showed the benchmark closures inlined into
`transcode::bench_transcoder`; the default executable has no source line table
to map the inlined regions back to each timed workload. This does not reliably
establish whether `direct_transcode`, `complete_lifecycle`, `decode_engine`,
`codec_adapter`, `safe_copy_loop`, or the old `output_window` workloads retain
all output writes. RC-009 therefore remains unresolved for this recorded
toolchain and host. No benchmark code was changed, no timing was run in this
inspection, and the newer output-window observations do not prove the old
workloads retained their complete writes. The artifact and inspection are
reproducible with the command above followed by `sha256sum`, `nm -C`, and
`objdump -Cd` on the reported executable; no temporary evidence path is
required.

### RC-009 follow-up: 2026-10-30 default artifact recheck

The recheck used baseline commit `c39667017b9a99160c859a53fff3741d321c31c9`
with no diff in `benches/transcode.rs`, rustc 1.94.0
(`x86_64-unknown-linux-gnu`, LLVM 21.1.8), and an Intel Core i5-9600K host
(6 CPUs). The exact command was
`CARGO_TARGET_DIR=/tmp/rs-codec-rc009-20261030/default/target cargo bench
--locked --bench transcode --no-run --message-format=json`; Cargo reported
`/tmp/rs-codec-rc009-20261030/default/target/release/deps/transcode-518bb86cbd866b72`
with SHA-256
`2056f892a1a9d7f95cfb4f63a012ab4f1de0066d929e2cd12c480e62f5d31677`.
The profile was `opt-level=3`, `lto=true`, and `codegen-units=1`.

`nm -S -C` identified `transcode::bench_transcoder` at `0x979f0` with size
`0x7f24`, and `transcode::copy_safe_exact` at `0x979d0` with size `0x1f`.
`objdump -Cd` showed the benchmark registration and inlined workload setup in
`bench_transcoder`, but the default artifact has no line table and no distinct
symbols for the individual timed closures. The default and one auxiliary
`CARGO_PROFILE_BENCH_DEBUG=1` build had byte-identical disassembly for the
`bench_transcoder` and `copy_safe_exact` symbol ranges. The auxiliary artifact
(`transcode-7877a69bede2c061`, SHA-256
`c77eac7cb01ef47602499917ba01ac192ccc1a5ff387e2bad14da8a578519954`) provided
source line mappings for Criterion iteration bodies, but its separate closure
code could not be unambiguously mapped to the default artifact. A memcpy or
vector instruction elsewhere in the default binary is therefore insufficient
to classify those individual workloads.

| Timed path | Classification | Evidence |
| --- | --- | --- |
| `direct_transcode` | Unresolved | The default artifact does not isolate its timed closure from the LTO-inlined benchmark body. |
| `complete_lifecycle` | Unresolved | The default artifact does not isolate its timed closure from the LTO-inlined benchmark body. |
| `decode_engine` | Unresolved | The default artifact does not isolate its timed closure from the LTO-inlined benchmark body. |
| `codec_adapter` | Unresolved | The default artifact does not isolate its timed closure from the LTO-inlined benchmark body. |
| `safe_copy_loop` | Confirmed complete writes | `criterion::bencher::Bencher<M>::iter` at `0x7f890` (size `0x176`) calls the unique `transcode::copy_safe_exact` helper at `0x7f918`; that helper at `0x979d0` tail-calls `memcpy` with `min(input.len(), output.len())`, both 65,536 for this fixture. |
| `output_window_32` | Unresolved | The default artifact does not isolate this closure from the other streaming workloads. |
| `output_window_1024` | Unresolved | The default artifact does not isolate this closure from the other streaming workloads. |

No benchmark source change or timing run was made. The six unresolved paths
remain evidence-limited; their recorded values must not support performance
tradeoffs. The auxiliary build was used only for attribution and does not
replace the default-profile result. Both temporary build directories were
created for this inspection and are scheduled for cleanup after validation and
branch synchronization.

### RC-009 follow-up: 2026-10-30 T3 worktree artifact audit

This audit used baseline commit `51665da020bc415ba1c34bfc770f690b64cf88cb`.
There was no worktree diff in `benches/transcode.rs` or `benches/README.md`
before the audit. `rustc -Vv` reported rustc 1.94.0, host
`x86_64-unknown-linux-gnu`, LLVM 21.1.8. The host was an Intel Core i5-9600K
with 6 CPUs. The bench profile was `opt-level=3`, `lto=true`,
`codegen-units=1`.

The one default-profile build command was
`CARGO_TARGET_DIR=/tmp/rs-codec-rc009-t3-20261030/default/target cargo bench
--locked --bench transcode --no-run --message-format=json`; its full Cargo
JSON output was saved as
`/tmp/rs-codec-rc009-t3-20261030/default/cargo.json`. The JSON
`compiler-artifact` record for `target.name == "transcode"` identified
`/tmp/rs-codec-rc009-t3-20261030/default/target/release/deps/transcode-518bb86cbd866b72`
(SHA-256
`2056f892a1a9d7f95cfb4f63a012ab4f1de0066d929e2cd12c480e62f5d31677`).
`nm -S -C` and `objdump -Cd` both exited successfully; their full outputs were
saved as `default/nm.txt` and `default/objdump.txt` beside the JSON. `nm`
identified `transcode::bench_transcoder` at `0x979f0` (size `0x7f24`) and
`transcode::copy_safe_exact` at `0x979d0` (size `0x1f`), with no distinct
symbols for the six timed closures. The default executable has no debug line
or info sections. The `memcpy` references and vector instructions in the
executable could not be assigned to a particular timed closure.

One optional attribution build used
`CARGO_TARGET_DIR=/tmp/rs-codec-rc009-t3-20261030/debug/target
CARGO_PROFILE_BENCH_DEBUG=1 cargo bench --locked --bench transcode --no-run
--message-format=json`; its full Cargo JSON was saved as
`debug/cargo.json`. It identified
`/tmp/rs-codec-rc009-t3-20261030/debug/target/release/deps/transcode-7877a69bede2c061`
(SHA-256
`1ce30e41c3d6360174fc33ea60a887e4a5d24489eb4d21f3364e7ecba55e7fd6`).
`nm -S -C` and `objdump -CdSl` succeeded; the outputs were saved as
`debug/nm.txt` and `debug/objdump.txt`. The auxiliary line table associates
closures with source lines, but enabling debug information changes the build
and does not reliably map those instructions back to the default LTO artifact.
No benchmark was timed.

The six source paths and their output observations are:

| Timed path | Input read, output range, and loop in source | Observation in timed closure | Classification and evidence limit |
| --- | --- | --- | --- |
| `direct_transcode` | Lines 129–134 pass all 65,536 input bytes to one `transcode` call with a same-size output buffer. `CopyTranscoder::transcode` (lines 101–115) copies `min(input.len(), output.len())`, which is the full 65,536-byte range for this fixture. | Only `progress.read()` and `progress.written()` are black-boxed. | **Unresolved** — default disassembly does not associate any copy/store range with this closure. |
| `complete_lifecycle` | Lines 138–143 pass the full input and same-size output to one `transcode_complete_into` call; source-level expected output is 65,536 bytes. | `written` and `output[0]` are black-boxed. | **Unresolved** — default disassembly does not associate the lifecycle copy range with this closure. |
| `decode_engine` | Lines 156–161 pass all input bytes to one engine `transcode` call with same-size output; the codec copies the full fixture in source semantics. | Only the progress read and written counts are black-boxed. | **Unresolved** — default disassembly does not establish the engine closure's complete output writes. |
| `codec_adapter` | Lines 182–187 pass all input bytes to one adapter `transcode` call with same-size output; the codec copies the full fixture in source semantics. | Progress counts and `adapter_output[0]` are black-boxed. | **Unresolved** — default disassembly does not establish the adapter closure's complete output writes. |
| `output_window_32` | Lines 201–218 repeatedly read the remaining input and write up to 32 bytes into the same 32-byte output window, advancing by `progress.read()` until all input is consumed (2,048 windows for this fixture). | Only final input and accumulated written counts are black-boxed; each window is overwritten on the next iteration. | **Unresolved** — no default instructions could be assigned to this closure to determine whether each window's full written range survives. |
| `output_window_1024` | Lines 201–218 repeatedly read the remaining input and write up to 1,024 bytes into the same 1,024-byte output window, advancing by `progress.read()` until all input is consumed (64 windows for this fixture). | Only final input and accumulated written counts are black-boxed; each window is overwritten on the next iteration. | **Unresolved** — no default instructions could be assigned to this closure to determine whether each window's full written range survives. |

The source confirms the requested input and output ranges, while the default
artifact cannot establish whether those writes are retained or eliminated for
any of the six closures. The debug auxiliary result cannot close that mapping
gap. No benchmark source was changed, no timing was run, and the historical
measurements remain unsuitable for performance tradeoffs on this evidence.

### RC-009 follow-up: 2026-10-30 implementation audit

This audit used baseline `6881abead88b21476c4d3a4fbba8cfea04e5625f`; there
was no diff in `benches/transcode.rs`. The environment was rustc 1.94.0,
LLVM 21.1.8, `x86_64-unknown-linux-gnu`, on an Intel Core i5-9600K with six
CPUs. The bench profile was `opt-level=3`, `lto=true`, and `codegen-units=1`.
The default build command was
`CARGO_TARGET_DIR=target/rc009-ec2be1ab/default/target cargo bench --locked
--bench transcode --no-run --message-format=json` (exit 0). Its Cargo JSON
identified
`target/rc009-ec2be1ab/default/target/release/deps/transcode-518bb86cbd866b72`,
SHA-256
`2056f892a1a9d7f95cfb4f63a012ab4f1de0066d929e2cd12c480e62f5d31677`.
The JSON, stderr, artifact path, hash, complete `nm -S -C`, and complete
`objdump -Cd` outputs were captured under
`target/rc009-ec2be1ab/default/` during this run. `nm` identified
`transcode::bench_transcoder` at `0x979f0` (size `0x7f24`) and
`transcode::copy_safe_exact` at `0x979d0` (size `0x1f`); no distinct symbols
identify the six timed closures. The artifact hash matches the earlier default
audit above, and it has no debug line or info sections. One additional build
would not provide new attribution evidence, so no auxiliary build was run.

| Timed path | Classification | Evidence and limitation |
| --- | --- | --- |
| `direct_transcode` | Unresolved | Source passes the full fixture; LTO output does not map the closure's complete write range. |
| `complete_lifecycle` | Unresolved | Source expects a full-fixture write; the inlined closure cannot be isolated in the default artifact. |
| `decode_engine` | Unresolved | Source passes the full fixture; no attributable default instructions establish complete output writes. |
| `codec_adapter` | Unresolved | Source passes the full fixture; no attributable default instructions establish complete output writes. |
| `output_window_32` | Unresolved | Source reuses a 32-byte window; the default artifact does not establish whether each window's writes survive. |
| `output_window_1024` | Unresolved | Source reuses a 1,024-byte window; the default artifact does not establish whether each window's writes survive. |

### RC-009 follow-up: 2026-10-30 implementation worktree recheck

This recheck used baseline `31a5dc944a3858656593f13ac4498a2e9a5df127` in
worktree `/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17`.
Before the build, `benches/transcode.rs` and this README had no worktree diff.
The environment was rustc 1.94.0 (`x86_64-unknown-linux-gnu`, LLVM 21.1.8)
on an Intel Core i5-9600K host (6 CPUs). The default bench profile was
`opt-level=3`, `lto=true`, and `codegen-units=1`; Cargo enabled the crate's
default feature set. The one default-profile build command was
`cargo bench --locked --bench transcode --no-run --message-format=json`
(exit 0). Cargo identified
`target/release/deps/transcode-518bb86cbd866b72`, SHA-256
`2056f892a1a9d7f95cfb4f63a012ab4f1de0066d929e2cd12c480e62f5d31677`.
The artifact hash matches the earlier default-profile audit above. `nm -S -C`
found `transcode::bench_transcoder` at `0x979f0` (size `0x7f24`) and
`transcode::copy_safe_exact` at `0x979d0` (size `0x1f`), but no distinct
symbols for the timed closures. `objdump -Cd` could not assign complete output
writes to individual closures, and the artifact has no debug line or info
sections. Full Cargo JSON, stderr, `nm`, `objdump`, and section output were
retained under `target/review-7a98bc17/default/` as evidence for this audit.
No auxiliary build or timing run was performed because the recheck yielded no
new attribution method.

| Timed path | Source range and observation | Classification |
| --- | --- | --- |
| `direct_transcode` | One call receives the 65,536-byte fixture and same-size output (`benches/transcode.rs:128–134`); only read/written counts are observed. | **Unresolved** — the default LTO artifact does not attribute the complete write range to this closure. |
| `complete_lifecycle` | One call receives the full fixture and same-size output (`:137–143`); only the written count and first output byte are observed. | **Unresolved** — the artifact does not attribute the lifecycle write range to this closure. |
| `decode_engine` | One call receives the full fixture and same-size output (`:155–161`); only read/written counts are observed. | **Unresolved** — the artifact does not attribute the complete write range to this closure. |
| `codec_adapter` | One call receives the full fixture and same-size adapter output (`:181–187`); only progress counts and the first output byte are observed. | **Unresolved** — the artifact does not attribute the complete write range to this closure. |
| `output_window_32` | Reuses a 32-byte output window for 2,048 iterations (`:198–217`); observes final counters, not each range before overwrite. | **Unresolved** — the artifact does not attribute each iteration's output writes to this closure. |
| `output_window_1024` | Reuses a 1,024-byte output window for 64 iterations (`:198–217`); observes final counters, not each range before overwrite. | **Unresolved** — the artifact does not attribute each iteration's output writes to this closure. |

No benchmark source was changed: the available evidence does not confirm
eliminated writes. These results do not establish either retention or
elimination of the six paths' complete writes and do not support a performance
conclusion. The artifact and full inspection output were generated locally;
they were not uploaded as CI artifacts.

RC-009 remains open. `benches/transcode.rs` was not changed, no benchmark was
timed, and these results do not support performance tradeoffs or a claim that
the six paths retain or eliminate their complete writes. This conclusion is
limited to the recorded artifact, toolchain, and host.

### RC-002: 2026-10-31 bounded default-artifact recheck

This recheck used baseline HEAD `dcfa62e102079bd15bb8d0c1ea0f88399257ef41`
with rustc/Cargo 1.94.0 (LLVM 21.1.8), `x86_64-unknown-linux-gnu`, Linux
`7.0.0-38-generic`, Intel Core i5-9600K (6 CPUs), and the default bench
profile (`opt-level=3`, `lto=true`, `codegen-units=1`, default features).
The source/configuration fingerprints were `benches/transcode.rs`
`07bc0383c3ab3bab8bfb636ce27a50065841005a7630d9c894a14be7e93b6b28`,
`Cargo.toml` `cfeb931cded3c5a27f2000b0d5e7b5e7727afaa5a33996d52b4473edc7730de2`,
and `Cargo.lock`
`3dcf411b1d65da3461bf593f52a3e051c82889ec1b79733bb0f9d9a9eade3141`.
The single default-profile build command,
`cargo bench --locked --bench transcode --no-run --message-format=json`,
exited 0. Its Cargo JSON selected
`target/rc-002-evidence/default/target/release/deps/transcode-518bb86cbd866b72`,
SHA-256
`2056f892a1a9d7f95cfb4f63a012ab4f1de0066d929e2cd12c480e62f5d31677`.
`nm -S -C`, `objdump -Cd`, and `readelf -S` exited 0. The artifact contains
`transcode::bench_transcoder` (`0x979f0`, size `0x7f24`) and
`transcode::copy_safe_exact` (`0x979d0`, size `0x1f`), but no distinct symbols
or debug line sections to assign the LTO-inlined timed closures. The artifact
hash matches the prior default-profile audits, so no auxiliary build was
repeated; it would not add attribution evidence.

| Timed path | Source write and timed observation | Result |
| --- | --- | --- |
| `direct_transcode` | One 65,536-byte copy; only progress counts observed. | Unresolved: default instructions cannot be assigned to this closure. |
| `complete_lifecycle` | One 65,536-byte lifecycle copy; only count and first output byte observed. | Unresolved: default instructions cannot be assigned to this closure. |
| `decode_engine` | One 65,536-byte engine copy; only progress counts observed. | Unresolved: default instructions cannot be assigned to this closure. |
| `codec_adapter` | One 65,536-byte adapter copy; only progress counts and first output byte observed. | Unresolved: default instructions cannot be assigned to this closure. |
| `output_window_32` | Reuses a 32-byte window for 2,048 writes; only final totals observed. | Unresolved: per-window writes cannot be assigned to this closure. |
| `output_window_1024` | Reuses a 1,024-byte window for 64 writes; only final totals observed. | Unresolved: per-window writes cannot be assigned to this closure. |

No old benchmark implementation was changed: this audit confirms neither
retention nor elimination of the six paths' complete writes. Their historical
measurements remain unsuitable for performance decisions. This classification
is limited to the recorded default artifact and environment; `black_box` in
source does not by itself prove machine-code retention. The Cargo JSON,
selected path, hash, full `nm`/`objdump` output, and section listing were
captured under `target/rc-002-evidence/default/` during this implementation;
they are local generated evidence, while the summary above is self-contained.

### RC-003: 2026-10-31 engine baseline

The eight `engine_encode_convert` workloads passed their output oracle and
pending-drain prechecks, then were timed with
`cargo bench --locked --bench transcode -- engine_encode_convert` on the same
host and default bench profile described above. Criterion used 20 samples,
2-second warmup, and 5-second measurement time per workload. The table records
the reported time and input-byte throughput intervals from this run:

| Workload | Time interval (median) | Input throughput interval (median) |
| --- | ---: | ---: |
| `encode_full` | 116.03–126.35 µs (121.94 µs) | 494.68–538.67 MiB/s (512.53 MiB/s) |
| `convert_full` | 148.60–156.31 µs (151.77 µs) | 399.85–420.60 MiB/s (411.82 MiB/s) |
| `encode_window_2` | 276.50–308.52 µs (291.84 µs) | 202.58–226.04 MiB/s (214.16 MiB/s) |
| `convert_window_2` | 492.41–532.65 µs (512.45 µs) | 117.34–126.93 MiB/s (121.96 MiB/s) |
| `encode_window_32` | 109.99–110.85 µs (110.44 µs) | 563.80–568.25 MiB/s (565.90 MiB/s) |
| `convert_window_32` | 163.79–166.25 µs (164.87 µs) | 375.94–381.59 MiB/s (379.10 MiB/s) |
| `encode_window_1024` | 103.94–108.30 µs (105.66 µs) | 577.12–601.29 MiB/s (591.51 MiB/s) |
| `convert_window_1024` | 148.51–167.07 µs (157.93 µs) | 374.08–420.84 MiB/s (395.74 MiB/s) |

Other Cargo builds were active on the shared host during this run, and Criterion
reported outliers for several cases. Treat these as a local baseline rather
than a cross-run threshold or a performance comparison. In the default LTO
artifact, `nm -S -C` found `transcode::bench_transcoder` at `0x9d0e0` (size
`0xac88`) but no separate symbols or debug line information for the eight
closures; `objdump -Cd` therefore cannot attribute each full write to a timed
closure. The source observes every written slice with `black_box` before
reusing its buffer, but this is not a machine-code retention proof. The
artifact SHA-256 is
`9f6fc8482482d6eb2f79831a92dc268edcd000d5b6857b6bbf17278cd311b49c`; source
fingerprints are `benches/transcode.rs`
`5f99479a9c8be3c1a85a0e3b75b97a7ecbe9c9e7d2c3a2f3e9376e9698e20609`,
`Cargo.toml` `cfeb931cded3c5a27f2000b0d5e7b5e7727afaa5a33996d52b4473edc7730de2`,
and `Cargo.lock`
`3dcf411b1d65da3461bf593f52a3e051c82889ec1b79733bb0f9d9a9eade3141`.
The RC-002 Cargo JSON and inspection outputs were generated under
`target/rc-002-evidence/default/`; the final RC-003 executable inspection is
under `target/rc-003-evidence/default/`, and Criterion estimates are under
`target/rc-002-evidence/default/target/criterion/engine_encode_convert/`.
These local artifacts are not needed to interpret this committed summary.

### RC-009 follow-up: 2026-10-31 implementation worktree audit

The audit baseline was HEAD `117a4350f8bc997ee8623ae014632e979205557b`;
`benches/transcode.rs` had no diff. The source/configuration hashes are recorded
in `target/review-fa071605/source-diff.sha256` and
`target/review-fa071605/source-config.sha256`. The environment was rustc and
Cargo 1.94.0, LLVM 21.1.8, `x86_64-unknown-linux-gnu`, Linux
`7.0.0-38-generic`, Intel Core i5-9600K (6 CPUs). The default bench profile is
`opt-level=3`, `lto=true`, `codegen-units=1`; the command used Cargo's default
feature set. One default-profile build ran:

```text
CARGO_TARGET_DIR=target/review-fa071605/baseline cargo bench --locked --bench transcode --no-run --message-format=json
```

It exited 0. Cargo's `compiler-artifact` record selected
`target/review-fa071605/baseline/release/deps/transcode-518bb86cbd866b72`,
SHA-256
`2056f892a1a9d7f95cfb4f63a012ab4f1de0066d929e2cd12c480e62f5d31677`.
The complete Cargo JSON and stderr, selected executable path and hash, `nm -S
-C`, and `objdump -Cd` outputs were captured under
`target/review-fa071605/baseline/` for this audit. The committed summary below
records the relevant evidence; these generated files are temporary build
artifacts. `nm` identifies
`transcode::bench_transcoder` at `0x979f0` (size `0x7f24`) and
`transcode::copy_safe_exact` at `0x979d0` (size `0x1f`), but no separate
symbols for the six timed closures. Disassembly contains memcpy references,
but they cannot be assigned to the six individual paths. The executable has
`.text` and `.symtab` sections and no debug line or info sections. The
artifact's SHA-256 matches prior default-profile audits; their auxiliary
debug build likewise did not establish attribution to this default LTO
artifact. No auxiliary build or timing run was repeated because it would not
add attribution evidence.

| Timed path | Source input/output range and write operation | Timed observation point | Classification and evidence limit |
| --- | --- | --- | --- |
| `direct_transcode` | `benches/transcode.rs:128–134` passes the full 65,536-byte fixture to one call with a same-size output. `CopyTranscoder::transcode` slices at index zero and calls `copy_from_slice` for all 65,536 bytes (`:101–115`). | After the call, only `progress.read()` and `progress.written()` are black-boxed (`:133`). | **Unresolved** — default artifact has no attributable closure address range proving the complete output write is retained or omitted. |
| `complete_lifecycle` | `:137–144` passes the full fixture and same-size output to `transcode_complete_into`; source semantics expect 65,536 output bytes. | After the call, only `written` and `output[0]` are black-boxed (`:142`). | **Unresolved** — inlined lifecycle instructions cannot be assigned to this closure in the default artifact. |
| `decode_engine` | `:155–162` passes the full fixture and same-size output to one engine transcode call; source codec writes one byte per input byte (`CopyCodec::decode`, `:84–94`). | After the call, only the progress counts are black-boxed (`:160`). | **Unresolved** — no attributable default instruction range establishes complete engine output writes. |
| `codec_adapter` | `:181–188` passes the full fixture and same-size adapter output to one call; source codec semantics expect 65,536 decoded bytes. | After the call, progress counts and only `adapter_output[0]` are black-boxed (`:186`). | **Unresolved** — no attributable default instruction range establishes complete adapter output writes. |
| `output_window_32` | `:201–218` reuses one 32-byte output buffer. Each transcode call copies up to 32 bytes (`:101–115`); input advances by `progress.read()` until the 65,536-byte fixture is consumed (2,048 windows). | Only final input and accumulated written counts are observed (`:216`); no window's written slice is observed before the next iteration overwrites it. | **Unresolved** — default artifact does not attribute the loop's per-window stores to this closure. |
| `output_window_1024` | `:201–218` reuses one 1,024-byte output buffer; the loop consumes the 65,536-byte fixture in 64 windows. | Only final input and accumulated written counts are observed (`:216`); no window's written slice is observed before overwrite. | **Unresolved** — default artifact does not attribute the loop's per-window stores to this closure. |

For all six paths, source code specifies the intended ranges, but the default
LTO machine code cannot resolve whether each complete write survives. No
benchmark source was changed: the evidence does not confirm omitted writes.
This audit does not establish retention or elimination and cannot support
performance tradeoffs.
