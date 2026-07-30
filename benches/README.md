# Generic transcode benchmarks

`transcode.rs` records baseline throughput for the generic `Transcoder`
contract without selecting a concrete format codec. It compares a direct
in-memory call, the complete lifecycle helper, and streaming calls constrained
by two output-window sizes. Input construction and output allocation occur
outside the timed loop.

Run the benchmark with:

```bash
cargo bench --bench transcode
```

The reported throughput is input bytes per second. Do not compare absolute
numbers across machines; use the same hardware and Rust toolchain when judging
a change to transcode contracts or lifecycle helpers.
