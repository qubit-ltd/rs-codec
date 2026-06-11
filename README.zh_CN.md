# Qubit Codec

[![Rust CI](https://github.com/qubit-ltd/rs-codec/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-codec/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-codec/coverage-badge.json)](https://qubit-ltd.github.io/rs-codec/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-codec.svg?color=blue)](https://crates.io/crates/qubit-codec)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

面向 Rust 的核心 codec trait 和缓冲区转换原语。

## 概述

Qubit Codec 是 Qubit codec 系列 crate 的领域无关基础层。它只放 binary、
text、misc 和 I/O adapter crate 需要共享的小型 trait 与值类型，不引入
`std::io` stream helper，也不放具体格式实现。

本库提供：

- 用于底层单值缓冲区编码解码的 `Codec` trait。
- 基于给定 `Codec` 显式适配 value 与 buffered 转换的
  `CodecValueEncoder`、`CodecValueDecoder`、`CodecTranscodeEncoder`、
  `CodecTranscodeDecoder` 和 `CodecTranscodeConverter` adapter。
- 用于下游带策略 encoder 复用公共 buffered encode 循环的
  `TranscodeEncodeEngine`、`TranscodeEncodeHooks`、`EncodePlan` 和
  `EncodeContext`。
- 用于下游带策略 decoder 复用公共 buffered decode 循环的
  `TranscodeDecodeEngine`、`TranscodeDecodeHooks`、`DecodeAction` 和
  `DecodeContext`。
- 用于完整值便捷转换的 `ValueEncoder` 和 `ValueDecoder` trait。
- 用于调用方管理逻辑流缓冲区转换的 `Transcoder`、`TranscodeProgress`
  和 `TranscodeStatus`。
- 用于表达 transcoder 语义方向的 `TranscodeEncoder`、`TranscodeDecoder` 和
  `TranscodeConverter` marker trait。
- 供 binary 与 text codec 共享的 `ByteOrder`、`ByteOrderSpec`、
  `BigEndian` 和 `LittleEndian`。

具体 codec 位于相邻 crate，例如 `qubit-codec-binary`、
`qubit-codec-text` 和 `qubit-codec-misc`。

## 设计目标

- **分层边界清晰**：把领域无关 trait 与 binary、text、misc、stream 实现分开。
- **公开 API 小而稳定**：只暴露多个 codec crate 都需要共享的基础原语。
- **不耦合 I/O**：避免引入 `std::io` 依赖，使缓冲区级 codec 可用于非 stream 场景。
- **策略中立**：charset、畸形输入和线格式规则由领域 crate 自己定义。
- **零成本标记**：用可复制的类型和值标记表达字节序，不产生运行时分配。
- **稳定进度报告**：用 `TranscodeProgress` 和 `TranscodeStatus` 明确表达调用方管理缓冲区时的转换进度。

## 特性

### 核心转换 Trait

- **`Codec`**：在调用方管理的 unit 缓冲区中编码和解码一个值或 codec quantum。
- **`CodecEncodeError` / `CodecDecodeError` / `CodecConvertError`**：表达
  adapter 自己产生的 encode / decode / convert 错误，包括非法缓冲区下标，
  同时保留 codec-specific failure。
- **`ValueEncoder<Input>`**：把借用输入编码为自有输出。
- **`ValueDecoder<Input>`**：把借用的编码输入解码为自有输出。
- **`CodecValueEncoder<C>`**：把 `Codec` 包装为
  返回自有 `Vec<C::Unit>` 的 `ValueEncoder<C::Value>`。
- **`CodecValueDecoder<C>`**：把 `Codec` 包装为
  接收恰好一个完整编码值的 `ValueDecoder<[C::Unit]>`。

### 缓冲区转换原语

- **`Transcoder<Input, Output>`**：在调用方提供的缓冲区中把输入单元转换为输出单元，并在调用方处理完不完整输入尾部后完成内部收尾输出。
- **`TranscodeEncoder<Value, Unit>`**：表示 value-to-unit 缓冲区编码的语义化
  `Transcoder` bound。
- **`TranscodeDecoder<Unit, Value>`**：表示 unit-to-value 缓冲区解码的语义化
  `Transcoder` bound。
- **`TranscodeConverter<InputUnit, OutputUnit>`**：表示 unit-to-unit 缓冲区转换的语义化
  `Transcoder` bound。
- **`CodecTranscodeEncoder<C>`**：把 `Codec` 包装为在调用方输出缓冲区上工作的
  `TranscodeEncoder<C::Value, C::Unit>`。
- **`TranscodeEncodeEngine<C, H>`**：持有 codec 与策略 hooks，并运行公共
  buffered encode 循环的可复用 engine。
- **`TranscodeEncodeHooks<C>`**：供带策略 codec-backed encoder
  共享公共循环时实现的 transcode/finalization 策略 hook trait。
- **`EncodePlan<P>`**：单值写入计划，携带写入前必须保证的输出容量上界。
- **`EncodeContext<'a, Value, Unit>`**：engine 确认输出容量后传给
  encode hook 的输入值、输入索引、输出切片和游标上下文。
- **`CodecTranscodeDecoder<C>`**：把 `Codec` 包装为无策略的
  严格 `TranscodeDecoder<C::Unit, C::Value>`；engine 自己检测到的不完整尾部保留在调用方输入缓冲区中，
  codec 返回的 decode error 会被直接包装返回。
- **`TranscodeDecodeEngine<C, H>`**：持有 codec 与策略 hooks，并运行公共
  decode 循环的可复用 engine。
- **`TranscodeDecodeHooks<C>`**：供带策略 codec-backed decoder
  共享公共 decode 循环时实现的策略 hook trait。
- **`DecodeAction<Value>`**：decoder engine hook 在 transcode 阶段返回的策略动作。
- **`CodecTranscodeConverter<D, E>`**：组合一个解码
  codec 和一个编码 codec，形成无策略的 `TranscodeConverter`。
- **`TranscodeDecodeInput<I>`**：持有底层 unit `BufferedInput`，通过
  `decode_into` 驱动调用方传入的 `Codec`；由于 `Codec` 没有 finish 状态，
  `finish_into` 是 no-op。需要状态化 streaming decoder 时使用
  `transcode_into` / `finish_transcode_into`。
- **`TranscodeEncodeOutput<O>`**：持有底层 unit `BufferedOutput`，通过
  `encode_from` 驱动调用方传入的 `Codec`；普通 `flush` 只排空 unit
  buffer。状态化 streaming encoder 使用 `transcode_from` 和 `finish`。
- **`TranscodeProgress`**：报告相对读取和写入的单元数量。
- **`TranscodeStatus`**：区分转换完成、需要更多输入和需要更多输出空间。

### 字节序标记

- **`ByteOrder`**：公共 API 中使用的运行时字节序枚举。
- **`ByteOrderSpec`**：热路径 codec 使用的类型级字节序 trait。
- **`BigEndian` / `LittleEndian`**：零大小字节序标记类型。

### 聚焦的公开 API

- **`prelude` 模块**：导入常用核心 trait 和标记类型。
- **不包含具体格式**：binary、text 和 misc codec 发布在相邻 crate 中。

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
qubit-codec = "0.7"
```

## 快速开始

```rust
use qubit_codec::{
    TranscodeProgress,
    TranscodeStatus,
    ValueEncoder,
};

struct StringEncoder;

impl ValueEncoder<str> for StringEncoder {
    type Output = String;
    type Error = core::convert::Infallible;

    fn encode(&self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(input.to_owned())
    }
}

let encoded = ValueEncoder::<str>::encode(&StringEncoder, "codec")?;
assert_eq!("codec", encoded);

let progress = TranscodeProgress::complete(3, 4);
assert_eq!(TranscodeStatus::Complete, progress.status());

# Ok::<(), core::convert::Infallible>(())
```

## API 参考

### 核心 Codec Trait

| Trait | 用途 | 典型实现者 |
|-------|------|------------|
| `Codec` | 在调用方缓冲区中编码/解码一个值或 quantum | 二进制标量、字符集字符、转义 byte、Base64 quantum |
| `ValueEncoder<Input>` | 把借用输入编码为自有输出 | 文本、二进制或 misc 便捷 helper |
| `ValueDecoder<Input>` | 把借用输入解码为自有输出 | 文本、二进制或 misc 便捷 helper |
| `TranscodeEncoder<Value, Unit>` | 把逻辑值编码进调用方提供的 unit 缓冲区 | Charset 或 binary buffered encoder |
| `TranscodeDecoder<Unit, Value>` | 把编码 unit 解码进调用方提供的 value 缓冲区 | Charset 或 binary buffered decoder |
| `TranscodeConverter<InputUnit, OutputUnit>` | 在两种编码 unit 表示之间转换 | Charset 或 binary buffered converter |

| 类型 | 用途 |
|------|------|
| `CodecEncodeError<E>` | adapter 层 encode error，包装 codec error 或非法缓冲区下标 |
| `CodecDecodeError<E>` | adapter 层 decode error，包装 codec error、不完整输入、非法缓冲区下标或尾随输入 |
| `CodecConvertError<D, E>` | adapter 层 converter error，区分 decode 失败和完整的 encode-side `CodecEncodeError<E>` 失败 |

### Codec Adapter

| 类型 | 用途 |
|------|------|
| `CodecValueEncoder<C>` | 通过 `C: Codec` 把一个借用 `C::Value` 编码成自有 `Vec<C::Unit>`，不要求 `C::Value: Clone` |
| `CodecValueDecoder<C>` | 通过 `C: Codec` 把恰好一个借用 `[C::Unit]` slice 解码成 `C::Value` |
| `CodecTranscodeEncoder<C>` | 通过 `C: Codec` 把 `C::Value` slice 编码进调用方提供的 `C::Unit` 缓冲区 |
| `CodecTranscodeDecoder<C>` | 通过 `C: Codec` 严格地把 `C::Unit` slice 解码进调用方提供的 `C::Value` 缓冲区 |
| `CodecTranscodeConverter<D, E>` | 先解码 `D::Unit` source unit，再用满足 `E::Value = D::Value` 的 `E` 编码 `E::Unit` target unit |

### I/O Adapter

| 类型 | 用途 |
|------|------|
| `TranscodeDecodeInput<I>` | 调用时传入 caller-owned `Codec`，通过 `decode_into` 把 `qubit_io::Input` 中的 unit 解码为 value；状态化 streaming decoder 使用 `transcode_into` 和 `finish_transcode_into` |
| `TranscodeEncodeOutput<O>` | 调用时传入 caller-owned `Codec`，通过 `encode_from` 把 value 编码进 `qubit_io::Output`；状态化 streaming encoder 使用 `transcode_from` 和 `finish` |

### Encoder Hooks 和 Engine

| 类型 | 用途 |
|------|------|
| `TranscodeEncodeEngine<C, H>` | 基于低层 `Codec` 与策略 hooks 的可复用 buffered encoder engine |
| `TranscodeEncodeHooks<C>` | 准备、写入、重置并完成 encoded output 收尾的 hook 契约 |
| `EncodePlan<P>` | 已准备好的单值容量上界和实现自定义写入 action |
| `EncodeContext<'a, Value, Unit>` | 传递给 encode hook 的输入值、输入索引、输出切片和游标 |

### Decoder Hooks 和 Engine

| 类型 | 用途 |
|------|------|
| `TranscodeDecodeEngine<C, H>` | 基于低层 `Codec` 与策略 hooks 的可复用 buffered decoder engine |
| `TranscodeDecodeHooks<C>` | malformed/incomplete decode 策略 hook 契约 |
| `DecodeContext` | 传递给 decode policy hook 的上下文 |
| `DecodeAction<Value>` | transcode 阶段的策略动作：需要输入、跳过输入或输出一个值 |

### `Transcoder` 操作

| 方法 | 描述 |
|------|------|
| `max_output_len(input_len)` | 在可确定时返回输出长度上界 |
| `max_finish_output_len()` | 在可确定时返回 finish 收尾输出长度上界 |
| `reset()` | 保留配置并重置逻辑流状态 |
| `transcode(input, input_index, output, output_index)` | 把输入单元转换为输出单元 |
| `finish(output, output_index)` | 完成内部收尾输出，例如 reset bytes、digest 或 trailer |

### `TranscodeStatus` 取值

| 状态 | 含义 |
|------|------|
| `Complete` | 当前转换步骤已完成 |
| `NeedInput` | 需要更多输入单元；不完整尾部仍留在调用方输入缓冲区中 |
| `NeedOutput` | 需要更多输出空间 |

### 字节序类型

| 类型 | 使用场景 |
|------|----------|
| `ByteOrder` | 公共 API 中运行时选择字节序 |
| `ByteOrderSpec` | 类型级字节序抽象 |
| `BigEndian` | 大端类型标记 |
| `LittleEndian` | 小端类型标记 |

## 库边界

`qubit-codec` 不包含具体 binary 格式、字符集、percent/Base64/hex codec，
也不包含 `std::io` reader/writer adapter。这些能力应放在领域 crate 中，
让下游只依赖自己需要的层。

## 性能考虑

核心 trait 和 buffered adapter 不要求堆分配。`BigEndian` 和 `LittleEndian`
是零大小类型，`ByteOrder` 是小型可复制枚举。`CodecValueEncoder` 会分配自有
`Vec<Unit>` 输出，因为这是 `ValueEncoder` 契约；下游具体 codec 仍可能有自己的
分配行为。

## 测试与代码覆盖率

本项目通过 `tests/` 下的集成测试覆盖核心 trait 契约。

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行覆盖率报告
./coverage.sh

# 生成文本格式报告
./coverage.sh text

# 对齐 CI 要求
./align-ci.sh

# 运行 CI 检查（格式化、clippy、测试、覆盖率、安全审计）
RS_CI_SKIP_TOOLCHAIN_UPDATE=1 ./ci-check.sh
```

## 依赖项

运行时依赖保持很少：

- `thiserror` 提供公共错误类型实现。

## 许可证

Copyright (c) 2026. Haixing Hu.

根据 Apache 许可证 2.0 版（"许可证"）授权；
除非遵守许可证，否则您不得使用此文件。
您可以在以下位置获取许可证副本：

    http://www.apache.org/licenses/LICENSE-2.0

除非适用法律要求或书面同意，否则根据许可证分发的软件
按"原样"分发，不附带任何明示或暗示的担保或条件。
有关许可证下的特定语言管理权限和限制，请参阅许可证。

完整的许可证文本请参阅 [LICENSE](LICENSE)。

## 贡献

欢迎贡献！请随时提交 Pull Request。

### 开发指南

- 保持本 crate 不包含具体格式实现。
- 为公开 trait 和标记类型编写文档和示例。
- 保持测试全面且稳定。
- 提交 PR 前确保所有检查通过。

## 作者

**胡海星**

## 相关项目

- [qubit-codec-binary](https://github.com/qubit-ltd/rs-codec-binary)：二进制缓冲区级 codec。
- [qubit-codec-text](https://github.com/qubit-ltd/rs-codec-text)：charset 与 Unicode 缓冲区级 codec。
- [qubit-codec-misc](https://github.com/qubit-ltd/rs-codec-misc)：可复用的杂项字节与文本 codec。
- [qubit-io](https://github.com/qubit-ltd/rs-io)：通用 `std::io` helper。
- Qubit 旗下的更多 Rust 库发布在 GitHub 组织
  [qubit-ltd](https://github.com/qubit-ltd)。

---

仓库地址：[https://github.com/qubit-ltd/rs-codec](https://github.com/qubit-ltd/rs-codec)
