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

- 用于完整值转换的 `Encoder`、`Decoder` 和 `Codec` trait。
- 用于调用方管理缓冲区转换的 `Coder`、`CoderProgress` 和 `CoderStatus`。
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
- **稳定进度报告**：用 `CoderProgress` 和 `CoderStatus` 明确表达调用方管理缓冲区时的转换进度。

## 特性

### 核心转换 Trait

- **`Encoder<Input>`**：把借用输入编码为自有输出。
- **`Decoder<Input>`**：把借用的编码输入解码为自有输出。
- **`Codec<EncodeInput, DecodeInput>`**：标记一个类型同时支持编码和解码。

### 缓冲区 Coder 原语

- **`Coder<Input, Output>`**：在调用方提供的缓冲区中把输入单元转换为输出单元。
- **`CoderProgress`**：报告相对读取和写入的单元数量。
- **`CoderStatus`**：区分转换完成、需要更多输入和需要更多输出空间。

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
qubit-codec = "0.1"
```

## 快速开始

```rust
use qubit_codec::{
    CoderProgress,
    CoderStatus,
    Encoder,
};

struct StringEncoder;

impl Encoder<str> for StringEncoder {
    type Output = String;
    type Error = core::convert::Infallible;

    fn encode(&self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(input.to_owned())
    }
}

let encoded = Encoder::<str>::encode(&StringEncoder, "codec")?;
assert_eq!("codec", encoded);

let progress = CoderProgress::complete(3, 4);
assert_eq!(CoderStatus::Complete, progress.status());

# Ok::<(), core::convert::Infallible>(())
```

## API 参考

### 完整值转换 Trait

| Trait | 用途 | 典型实现者 |
|-------|------|------------|
| `Encoder<Input>` | 把借用输入编码为自有输出 | 文本、二进制或 misc encoder |
| `Decoder<Input>` | 把借用输入解码为自有输出 | 文本、二进制或 misc decoder |
| `Codec<EncodeInput, DecodeInput>` | 标记一个类型同时支持编码和解码 | 双向 codec |

### `Coder` 操作

| 方法 | 描述 |
|------|------|
| `max_output_len(input_len)` | 在可确定时返回输出长度上界 |
| `reset()` | 重置转换过程保留的状态 |
| `convert(input, input_index, output, output_index)` | 把输入单元转换为输出单元 |
| `finish(output, output_index)` | 在所有输入消费完后刷新缓冲输出 |

### `CoderStatus` 取值

| 状态 | 含义 |
|------|------|
| `Complete` | 当前转换步骤已完成 |
| `NeedInput` | 需要更多输入单元 |
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

本 crate 中的核心抽象都是 trait 或标记类型。`BigEndian` 和 `LittleEndian`
是零大小类型，`ByteOrder` 是小型可复制枚举。本 crate 自身不做堆分配；
具体分配行为由下游 crate 中的具体 codec 实现决定。

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

`qubit-codec` 没有运行时依赖。

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
