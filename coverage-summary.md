## Generation record

- Generated locally on 2026-10-30 23:59 Asia/Shanghai (2026-10-30 15:59 UTC) from baseline commit `31a5dc944a3858656593f13ac4498a2e9a5df127`; the worktree was dirty with this task's Rustdoc and benchmark documentation changes. No production behavior changed.
- Source snapshot: SHA-256 `5092915dba43a0a91ffc1520cbebdabda2f95ffaf9297d593e31b87dad731a5b`, over 211 sorted repository-relative file paths and per-file SHA-256 values. The full manifest is `target/review-7a98bc17/coverage-source-manifest.json`; each entry hashes `src/**/*.rs`, `tests/**/*.rs`, `examples/**/*.rs`, `benches/**/*.rs`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.infra/coverage/coverage.json`, and `.infra/ci/cargo-matrix.json`. The report itself is excluded.
- Environment: `rustc 1.94.0 (4a4ef493e 2026-03-02)`, host `x86_64-unknown-linux-gnu`, LLVM 21.1.8; `cargo-llvm-cov 0.8.6`; `rs-infra-coverage@407d3f58fddbe55ad7eb62f7c1ce7acfc1c7c9b1`.
- Entry command: `./.infra/bin/coverage.sh` (exit 0). The wrapper selected package `qubit-codec` and ran `cargo llvm-cov --package qubit-codec --all-features --json --output-path target/infra/coverage/raw.json`; coverage collection also enabled `cfg(coverage)`. The report generator completed JSON, LCOV, Cobertura, text, HTML, and badge outputs successfully. Configured thresholds and exemptions were unchanged.
- Local outputs: HTML `target/llvm-cov/html/index.html`, LCOV `lcov.info`, JSON `coverage.json`, text `coverage.txt`, badge `coverage-badge.json`, and Cobertura `target/llvm-cov/cobertura.xml`. These are local files, not uploaded CI artifacts.
- Relevant paths were hit in the all-features report: async EOF dispatch at `async_transcode_decode_input.rs:654` (count 9); EOF `NeedInput` validation branch at `transcode_progress_validation.rs:77-80` (branch count 24, error construction count 5); fallible allocations at `transcode_convert_engine.rs:917` (count 7) and `:1042` (count 9). The report summarizes the source files, not standalone tests; relevant cases are present in `tests/transcode/io/async_transcode_decode_input_tests.rs`, `tests/transcode/io/transcode_decode_input_tests.rs`, and `tests/transcode/engine/transcode_convert_engine_allocation_tests.rs`.

## Code coverage

| Metric | Covered | Total | Coverage |
| --- | ---: | ---: | ---: |
| functions | 606 | 615 | 98.54% |
| lines | 3894 | 4015 | 96.99% |
| regions | 5020 | 5285 | 94.99% |

- HTML report: local path `target/llvm-cov/html/index.html` (not uploaded as a CI artifact).
- LCOV report: local path `lcov.info` (not uploaded as a CI artifact).
- Shields endpoint: local path `coverage-badge.json` (not uploaded as a CI artifact).

- Full text report: local path `coverage.txt` (not uploaded as a CI artifact).
