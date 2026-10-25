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
