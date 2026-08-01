# Qubit Codec

[![Rust CI](https://github.com/qubit-ltd/rs-codec/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-codec/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-codec/coverage-badge.json)](https://qubit-ltd.github.io/rs-codec/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-codec.svg?color=blue)](https://crates.io/crates/qubit-codec)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-codec` 是 Rust codec 的领域无关基础层，清晰划分单值编码、自有值便捷 API
和调用方管理缓冲区的流式转换。它面向 codec 与 adapter 作者；具体二进制格式、字符集
和文本格式由相邻 crate 提供。

## 为什么需要这个库

一个格式 crate 往往从处理单个值的 `Codec` 开始，例如定宽标量或单个字符；随后又需要
自有输出的便捷 API、缓冲流 adapter 或畸形输入策略。若每层都重新实现游标处理与 EOF
行为，契约很容易产生分歧。

本库提供共享契约和 adapter，让格式 crate 将格式规则留在自身，同时复用经过检查的
转换机制。

## 安装

```toml
[dependencies]
qubit-codec = "0.11"
```

默认 feature 集为空。只有使用 `qubit-io` bridge 类型时才启用 `io`：

```toml
[dependencies]
qubit-codec = { version = "0.11", features = ["io"] }
```

## 快速开始

若格式的合理单位是完整输入，而非单个值，应直接实现 `ValueEncoder`。这比把没有
明确单值 quantum 的格式强行套进 `Codec` 更合适。

```rust
use qubit_codec::ValueEncoder;

struct PrefixEncoder;

impl ValueEncoder<str> for PrefixEncoder {
    type Output = String;
    type Error = core::convert::Infallible;

    fn encode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(format!("encoded:{input}"))
    }
}

let mut encoder = PrefixEncoder;
let output = encoder.encode("codec")?;
assert_eq!("encoded:codec", output);

# Ok::<(), core::convert::Infallible>(())
```

如果格式确实有局部的值边界，应实现 `Codec`，再选择现成 adapter：
`CodecValueEncoder` 或 `CodecValueDecoder` 用于自有单值输出；
`CodecTranscodeEncoder`、`CodecTranscodeDecoder` 或
`CodecTranscodeConverter` 用于严格的调用方缓冲区转换。

若 codec 会从 `decode_reset` 或 `decode_finish` 产出 value，严格单值 decode 会有意
拒绝该 codec。此时应使用 `CodecValueDecoder` 的 `decode_lifecycle` 或
`decode_lifecycle_with_scratch`；参见[生命周期感知 decode 示例](examples/decode_lifecycle.rs)。

## 核心能力

| 需求 | 公开 API |
| --- | --- |
| 在调用方缓冲区上处理一个值或 codec quantum | `Codec` 和 `DecodeFailure` |
| 自有完整值转换 | `ValueEncoder`、`ValueDecoder`、`CodecValueEncoder`、`CodecValueDecoder` |
| 严格的缓冲 encode、decode 或 unit 转换 | `TranscodeEncoder`、`TranscodeDecoder`、`TranscodeConverter` 和 `CodecTranscode*` adapter |
| 带策略的缓冲转换 | `engine::TranscodeEncodeEngine`、`engine::TranscodeDecodeEngine`、`engine::TranscodeConvertEngine` 及其 hooks |
| 调用方管理的流生命周期 | `Transcoder`、`TranscodeProgress` 和 `TranscodeStatus` |
| 共享字节序元数据 | `ByteOrder`、`ByteOrderSpec`、`BigEndian`、`LittleEndian` 和 `NativeEndian` |
| `qubit-io` 缓冲桥接 | 启用 `io` feature 后的 `TranscodeDecodeInput`、`TranscodeEncodeOutput` 和部分 I/O 的 `AsyncTranscodeDecodeInput` / `AsyncTranscodeEncodeOutput` |

流式转换的生命周期是显式的：

```text
reset(output) -> 反复 transcode(...) -> 处理 EOF 尾部 -> finish(output)
```

新建的 transcoder 处于未初始化状态，第一次成功操作必须是 `reset`；`finish` 成功后，
再次使用实例前也必须先调用 `reset`。

`Complete` 表示本次 `transcode` 从请求下标开始消费了全部可见输入。`NeedInput`
把不完整尾部保留给调用方，以便重试或显式做 EOF 决策；`NeedOutput` 表示必须扩展或
排空输出缓冲区后才能继续。

异步 bridge 直接暴露相同的进度：每次 `poll_transcode` 或 `transcode_async` 在完成
必要的 I/O 准备后至多调用一次 transcoder。返回的 `TranscodeProgress`（或
`AsyncTranscodeDecodeStep::Progress`）已经提交，不会在同一次调用中再次 await；调用方
应按返回计数推进游标，EOF 由显式的 `AsyncTranscodeDecodeStep::EndOfInput` 表示。

## 边界与保证

- 本库不实现具体二进制格式、字符集、Base64、hex、percent encoding 或
  `std::io` reader/writer extension。
- `Codec::decode` 通过 `DecodeFailure` 区分可见输入不完整与 codec-domain 非法
  输入；不完整前缀不等同于 EOF 决策。
- 对同一 value 和 codec 状态，`Codec::encode_len` 必须与成功的
  `Codec::encode` 精确一致，包括有意的零输出缓冲。
- `Transcoder` 的容量上界必须覆盖每种可达瞬态，而不是仅覆盖当前状态；`finish`
  不会收到调用方持有的不完整输入尾部。
- 自有输出 adapter 可能为返回的 `Vec` 分配内存。流式 API 使用调用方提供的缓冲区；
  `io` feature 会增加 `qubit-io` bridge 类型。

## 延伸阅读

- [用户手册](doc/user_guide.zh_CN.md)：抽象选择、生命周期规则、错误与实现清单。
- [English user guide](doc/user_guide.md)
- [API 文档](https://docs.rs/qubit-codec)
- [English README](README.md)

## 测试

```bash
# 使用默认 feature 集运行测试
cargo test

# 使用项目声明的全部 feature 运行测试
cargo test --all-features

# 运行项目 CI 检查
./ci-check.sh

# 检查代码覆盖率
./coverage.sh
```

## 许可证

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

本项目基于 Apache License 2.0 授权。完整许可证文本请参阅
[LICENSE](LICENSE)。

## 贡献

欢迎贡献。请遵循 Rust API 指南，及时更新公共 API 文档与测试，并在提交
Pull Request 前运行 `./align-ci.sh`格式化代码，运行`./ci-check.sh`对齐CI要求。

## 作者

**Haixing Hu** - *Qubit Co. Ltd.*

仓库地址：[https://github.com/qubit-ltd/rs-codec](https://github.com/qubit-ltd/rs-codec)
