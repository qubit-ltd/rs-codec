# Lifecycle buffer benchmark

This Criterion target measures the preparation cost of `CodecValueDecoder`'s
owned `decode_lifecycle` buffers against caller-owned buffers reused through
`decode_lifecycle_with_scratch`. Synthetic codecs declare equal reset and
finish bounds of 0, 1, 64, or 1024 values. For each bound, reset and finish
independently emit 0, 1, or the full bound where that count is legal. The
benchmark covers both `u8`'s cheap `Default` and `ResourceValue`, whose
`Default` allocates a 64-byte `Vec`.

Codec, input, and reusable scratch are constructed before timed iterations.
Owned buffer allocation, initialization, truncation, and destruction remain
inside the owned API call. Reusable scratch stays initialized across iterations:
written slots replace and drop their previous values, while unwritten slots
remain initialized until the benchmark ends. Results and scratch are passed
through `black_box`; the difference includes these distinct drop patterns.
Untimed prechecks verify reset values, the main value, finish values, ordering,
written counts, and exact `Default` call counts (twice the bound for owned;
zero for reusable scratch). A separate group measures converter `reset` and
`finish` calls. Both phases use Criterion 0.8.2 `iter_batched_ref` with
`BatchSize::SmallInput`: setup creates the converter and output buffer, and
finish setup also performs a successful reset. The timed routine borrows that
pair, calls only the target phase, and black-boxes its written count and output
slice. Criterion's locked implementation places setup and routine-input
destruction outside the per-routine measurement interval for this batch size;
the output tuple therefore is not moved or destroyed inside the routine.
Allocations and drops performed internally by reset or finish remain timed.
Untimed phase prechecks still verify output bytes and order. Converter phase
numbers must not be read as a full transcode measurement.

## Commands

Run the correctness precheck and Criterion test mode first:

```sh
cargo bench --locked --bench lifecycle_buffers -- --test
```

Then run the same full measurement command three times on an otherwise idle
host, without changing the build or benchmark configuration:

```sh
cargo bench --locked --bench lifecycle_buffers
```

Save each complete command output separately. Criterion reports estimates and
confidence intervals; retain the raw `target/criterion` output as supporting
data. Record host contention (other CPU-heavy work, virtualization or shared
host load) for every run. Do not treat a run with material contention as a
quiet-host sample.

## Measurement record

Three serial timing runs completed on 2026-11-12 UTC. The host was
`mas-server`, Linux 6.8.0-147-generic x86_64, dual-socket Intel Xeon E5-2680 v4
(56 logical CPUs, 125 GiB RAM), Rust/Cargo 1.94.0. Each benchmark used
Criterion's default 100 samples and 95% confidence intervals. The host was
shared (five logged-in users); 1-minute load averages ranged from about 7.6 to
59.6 and 15-minute averages from about 14.9 to 48.6 during the runs. Runs 1 and
2 had substantial contention; run 3 also had load changes during the sweep.
Treat these results as cost diagnostics, not quiet-host performance evidence.

| Item | Run 1 | Run 2 | Run 3 |
| --- | --- | --- | --- |
| Approximate UTC interval | 05:23–05:40 | 05:40–05:58 | 05:58–06:16 |
| Exact command | `cargo bench --locked --bench lifecycle_buffers` | same | same |
| Exit code | 0 | 0 | 0 |
| Criterion samples / interval | 100 / 95% CI | 100 / 95% CI | 100 / 95% CI |
| Output / artifacts | stdout observed in the command session but not saved as a separate log; `/tmp/rsc-lifecycle-measurements/run1-criterion` | `/tmp/rsc-lifecycle-measurements/run2.log`; `run2-criterion` | `/tmp/rsc-lifecycle-measurements/run3.log`; `run3-criterion` |

The correctness precheck `cargo bench --locked --bench lifecycle_buffers -- --test`
passed (exit 0) for all 110 benchmark IDs. The first compile attempt
failed because the scratch benchmark closure returned references to captured
buffers; the closure now black-boxes the progress and buffers without returning
those references. A subsequent precheck passed without Rust warnings.

Each value below is `mean [95% CI]` in nanoseconds. Reduction is the scratch
mean relative to the owned mean. These are selected representative scenarios;
all declared bounds and legal output-count combinations were exercised by the
full runs. Variation between runs and outliers reflect the shared host.

| Benchmark case | Run | Owned mean [95% CI] | Scratch mean [95% CI] | Scratch reduction |
| --- | ---: | ---: | ---: | ---: |
| `cheap_default`, bound 1, reset/finish 1/1 | 1 | 47.53 [46.44, 49.06] | 13.37 [13.09, 13.67] | 71.9% |
| same | 2 | 50.29 [48.37, 52.48] | 12.99 [12.80, 13.20] | 74.2% |
| same | 3 | 48.31 [47.07, 49.82] | 13.08 [12.86, 13.33] | 72.9% |
| `cheap_default`, bound 64, reset/finish 64/64 | 1 | 117.59 [117.13, 118.17] | 54.55 [54.34, 54.78] | 53.6% |
| same | 2 | 119.36 [117.98, 121.08] | 63.33 [60.87, 65.95] | 46.9% |
| same | 3 | 119.49 [117.60, 121.88] | 56.43 [54.86, 58.28] | 52.8% |
| `cheap_default`, bound 1024, reset/finish 0/0 | 1 | 175.00 [174.39, 175.75] | 11.28 [11.19, 11.37] | 93.6% |
| same | 2 | 146.44 [140.75, 152.96] | 12.68 [12.48, 12.88] | 91.3% |
| same | 3 | 175.93 [174.17, 178.30] | 11.33 [11.22, 11.45] | 93.6% |
| `cheap_default`, bound 1024, reset/finish 1024/1024 | 1 | 278.00 [276.15, 280.44] | 124.88 [119.38, 130.78] | 55.1% |
| same | 2 | 271.54 [270.45, 272.80] | 108.19 [107.27, 109.65] | 60.2% |
| same | 3 | 268.40 [267.96, 268.93] | 106.06 [105.94, 106.20] | 60.5% |
| `resource_default`, bound 64, reset/finish 64/64 | 1 | 7917.56 [7885.08, 7959.08] | 2022.87 [2019.11, 2026.88] | 74.5% |
| same | 2 | 8704.82 [8407.96, 9043.43] | 2056.84 [2043.91, 2071.60] | 76.4% |
| same | 3 | 9683.78 [9177.55, 10217.53] | 2296.23 [2212.11, 2387.64] | 76.3% |
| `resource_default`, bound 1024, reset/finish 0/0 | 1 | 81742.06 [81493.31, 82013.83] | 28.60 [28.43, 28.79] | 100.0% |
| same | 2 | 86808.57 [84982.99, 88908.37] | 27.96 [27.89, 28.04] | 100.0% |
| same | 3 | 86383.74 [84421.77, 88684.98] | 28.23 [28.13, 28.36] | 100.0% |
| `resource_default`, bound 1024, reset/finish 1024/1024 | 1 | 129333.54 [123942.03, 135363.60] | 34952.43 [34071.19, 35973.45] | 73.0% |
| same | 2 | 270463.66 [244580.68, 298454.10] | 37653.85 [35694.70, 39925.55] | 86.1% |
| same | 3 | 121231.60 [120023.10, 122634.72] | 32778.55 [32682.24, 32882.12] | 73.0% |

The synthetic comparisons show a measurable preparation cost, especially when
`Default` allocates and the declared bound is large. The run-to-run spread is
material for some cases (for example resource-heavy bound 1024 with full
output), and the suite has no identified production workload for these owned
lifecycle bounds. The 10% comparison rule alone is therefore insufficient to
justify a production change.

**Historical converter phase results (old timing boundary).** These have no
owned vs. scratch pair and predate the `iter_batched_ref` correction above.
The old reset routine created and initialized the caller output Vec inside the
timed closure, then dropped the converter there; old finish setup created the
output and reset the converter outside timing, while its routine dropped the
converter before returning the output Vec. Do not compare these values with
future measurements using the corrected boundary. Selected bound-1024 means
and 95% intervals in nanoseconds from those historical runs were:

| Phase/count | Run 1 | Run 2 | Run 3 |
| --- | ---: | ---: | ---: |
| reset / 0 | 220.56 [219.59, 221.60] | 223.24 [222.31, 224.35] | 271.41 [257.33, 286.88] |
| finish / 0 | 66.26 [64.90, 67.86] | 81.07 [80.24, 82.22] | 76.45 [71.32, 82.03] |
| reset / 1 | 429.96 [387.40, 475.14] | 229.22 [227.57, 231.14] | 334.00 [300.16, 375.43] |
| finish / 1 | 94.25 [93.20, 95.27] | 95.05 [93.87, 96.38] | 79.94 [78.56, 81.29] |
| reset / 1024 | 525.88 [521.78, 530.32] | 517.63 [513.78, 522.54] | 659.94 [628.71, 692.69] |
| finish / 1024 | 326.78 [325.80, 327.87] | 321.78 [321.10, 322.62] | 331.99 [324.20, 341.07] |

All three runs meet the mechanics of repeated measurement, but not the quiet
host condition. No production lifecycle-buffer optimization was made.

No measurements using the corrected converter phase boundary are recorded
here. The required test-mode and filtered runs verify the benchmark entry
points only; they are not new performance samples. Publish new phase values
only after three repeatable measurements with complete logs and recorded
environment and host contention.

## Downstream relevance checked

The source trees for `rs-codec-text`, `rs-codec-binary`, and `rs-codec-misc`
were searched for `MAX_DECODE_RESET_VALUES`, `MAX_DECODE_FINISH_VALUES`,
`decode_reset`, and `decode_finish`. No declarations or direct low-level hook
implementations were found in those three production `src/` trees. Their
converter/decoder wrappers call reusable engine `reset` and `finish` methods
(for example `rs-codec-text/src/convert/charset_converter.rs`), but those
calls alone do not establish nonzero lifecycle output or invoke the owned
single-value API. In the broader workspace, `rs-io-text` contains test codec
fixtures with reset/finish bounds (including 16 and 32), and the core crate
contains lifecycle examples and tests; these are synthetic/test evidence, not
proof of normal downstream workload frequency.

Thus the synthetic 64/1024 bounds and resource-heavy value are stress cases.
The current downstream source scan gives no direct evidence that these large
bounds or resource-heavy defaults represent the named production codec crates.
Before recommending a production change, identify a real caller and workload
that uses the relevant owned lifecycle API, and relate its value type and
declared bounds to these scenarios. No search result is evidence that every
external codec has zero lifecycle output.

## Decision rule

Do not recommend production optimization unless the same owned-vs-scratch
comparison shows a stable difference of at least 10% across all three
uncontended runs **and** an actual target workload demonstrates that the
measured path matters. If both conditions hold, first check whether using the
existing scratch API addresses that workload. Explain the cost paid by the
owned path when it initializes the full declared bound and then truncates,
including the drop of unused initialized values. Large synthetic bounds alone
do not establish a production bottleneck.
