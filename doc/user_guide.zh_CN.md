# Qubit Codec 用户手册

`qubit-codec` 提供 Qubit binary、text、misc 和 I/O codec crate 共用的领域无关契约。
它有意不提供具体的线格式或字符集。本手册说明如何选择抽象，并在需要时正确实现其契约。

逐项的函数签名、trait bound 和错误变体请查阅生成的 API 文档。本手册重点解释实现者必须同时保持的跨方法关系。

## 选择最小够用的抽象

| 需求 | 使用 |
| --- | --- |
| 在调用方缓冲区上编码或解码一个值/quantum | `Codec` |
| 将整个借用值转换为自有输出 | `ValueEncoder` / `ValueDecoder` |
| 把已有 `Codec` 包装成严格的流式 bridge | `CodecTranscodeEncoder`、`CodecTranscodeDecoder` 或 `CodecTranscodeConverter` |
| 对畸形值或不可编码值应用策略 | 带 hooks 的 `TranscodeXxxEngine` |
| 定义专用的、由调用方提供缓冲区的流转换 | `Transcoder` |

不要仅因数据需要编码就实现 `Codec`。对“完整输入”才是合理单位的格式，例如格式化 hex 字符串或 C 字符串字面量，通常直接实现 `ValueEncoder` / `ValueDecoder` 更清晰。反过来，已有 `Codec` 能用现成 adapter 包装时，也不要重复实现缓冲循环。

## `Codec` 实现者指南

`Codec` 持有一个逻辑值或 quantum，并在调用方提供的 unit 缓冲区上工作。它的 `encode` 和 `decode` 入口为 `unsafe`，因为 checked adapter 会一次完成边界检查，热路径不必重复构造 slice。实现不得读写文档定义的调用方可用范围之外的内存。

### 必须保持的不变量

- `MIN_UNITS_PER_VALUE` 和 `MAX_UNITS_PER_VALUE` 都非零，且前者不大于后者。
- `decode` 只读取可见输入。可见前缀合法但不足以形成值时，返回 `DecodeFailure::Incomplete`；不要消费该前缀。
- 成功的 `decode` 返回非零消费数，且不超过当前可用输入。
- 对同一 value 和 codec 状态，`encode_len` 必须精确等于成功 `encode` 返回的数量；故意在内部缓冲时二者都可以为零。
- `encode_len` 不得超过 `MAX_UNITS_PER_VALUE`。
- 调用方会先查询 `can_encode_value(value)`，再调用 `encode_len` 和 `encode`；应在前者拒绝不支持的值，而不是把它伪装成任意 `EncodeError`。
- reset 与 finish 的上界必须覆盖每一种可达流状态，不能只覆盖当前状态。无状态生命周期阶段应返回零。
- 返回任何错误后，内部状态必须仍保持一致，并符合已文档化的 retry/reset 策略。

### 最小定宽 codec

下面模板适用于无状态、单字节值 codec。`debug_assert!` 用于陈述 unchecked 前置条件；checked adapter 会在调用这些方法前保证它们成立。

```rust
use core::{convert::Infallible, num::NonZeroUsize};
use qubit_codec::{Codec, DecodeFailure, nz};

struct ByteCodec;

impl Codec for ByteCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_UNITS_PER_VALUE: usize = 1;

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Infallible> {
        debug_assert!(output_index < output.len());
        output[output_index] = *value;
        Ok(1)
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> {
        debug_assert!(input_index < input.len());
        Ok((input[input_index], nz!(1)))
    }
}
```

### 变宽与有状态 codec

输出宽度依赖 value 时，应覆盖 `encode_len`。带缓冲的 encoder 可以针对一个 value 同时从 `encode_len` 和 `encode` 返回零，并在后续输入或 `encode_finish` 时输出保留内容；此时必须声明足以容纳保留输出的 finish 上界。

decode 侧状态不会得到 EOF 输入 slice。若输入尾部需要在 EOF 时重新解释，默认 `Codec` bridge 的边界就不合适：应把该策略放在自定义 `Transcoder` 或 value-level facade 中。`decode_finish` 可以输出保留值或验证 decode 状态，但不能重新读取此前报告为 incomplete 的尾部。

## `Transcoder` 实现者指南

`Transcoder` 将一个逻辑输入流转换为输出流。它的 index 是传入 slice 中的绝对位置；`TranscodeProgress::read()` 与 `written()` 是相对于这些位置的数量。调用方负责补充输入，并决定不完整尾部在 EOF 时代表什么。

### 生命周期

```text
reset(output) -> 反复 transcode(...) -> 调用方处理 EOF 尾部 -> finish(output)
```

`reset` 在保留不可变配置的同时开始一条新的逻辑流。`finish` 成功后，为可移植性，调用方在复用实例前应先 reset。实现必须在 reset 时丢弃所有流本地的 pending state。

### 进度规则

| 结果 | 必须表达的含义 |
| --- | --- |
| `Complete` | 从 `input_index` 起的所有可见输入均已消费。 |
| `NeedInput` | 不完整尾部未被消费，重试时必须仍可用。 |
| `NeedOutput` | 输出容量阻止继续转换；报告绝对输出边界和未满足需求。 |

不得在只消费前缀后返回 `Complete`。也不要用 `finish` 偷偷重新解释不完整尾部：调用方必须在 finalization 前明确选择并应用 EOF 策略。

### 最小字节复制 transcoder

```rust
use core::{convert::Infallible, num::NonZeroUsize};
use qubit_codec::{
    CapacityError,
    TranscodeDecodeError,
    TranscodeProgress,
    TranscodeStatus,
    Transcoder,
};

struct ByteCopy;

impl Transcoder for ByteCopy {
    type Input = u8;
    type Output = u8;
    type Error = TranscodeDecodeError<Infallible>;

    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn reset(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        Self::Error::ensure_output_index(output.len(), output_index)?;
        Ok(0)
    }

    fn transcode(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        Self::Error::ensure_transcode_indices(
            input.len(), input_index, output.len(), output_index,
        )?;
        let count = (input.len() - input_index).min(output.len() - output_index);
        output[output_index..output_index + count]
            .copy_from_slice(&input[input_index..input_index + count]);
        if input_index + count == input.len() {
            Ok(TranscodeProgress::complete(count, count))
        } else {
            Ok(TranscodeProgress::new(
                TranscodeStatus::NeedOutput {
                    output_index: output_index + count,
                    required: NonZeroUsize::MIN,
                    available: 0,
                },
                count,
                count,
            ))
        }
    }

    fn finish(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        Self::Error::ensure_output_index(output.len(), output_index)?;
        Ok(0)
    }
}
```

有状态 transcoder 的 `max_reset_output_len`、`max_transcode_output_len` 和 `max_finish_output_len` 必须是对每种可达状态都保守成立的上界。`max_total_output_len` 将它们合成为完整 `reset -> transcode -> finish` 操作的上界。

## 错误与恢复

错误模型有意保留不同的恢复信息：

- `DecodeFailure` 仅用于低层 `Codec::decode` 边界，区分 incomplete prefix 与 invalid unit。
- `TranscodeFailure` 报告 framework 问题：非法 index、输出不足、溢出、不完整的整体输入和 trailing input。
- `CapacityError` 在写入输出前报告容量规划算术失败。
- `TranscodeDomainError` 为 domain error 添加 reset/main/finish 阶段上下文。
- 方向性错误保留失败来自 decode、encode 还是 converter；`TranscodeEncodeError` / `TranscodeConvertError` 还能保留不可编码 value。

除非调用方确实对所有类别采取相同恢复动作，否则不要在下游 API 中压平这些类别。尤其是 incomplete prefix 往往可重试，而畸形输入和进度契约破坏通常不可重试。

## 常见实现错误

| 错误 | 正确规则 |
| --- | --- |
| 部分读取后返回 `Complete` | 返回 `NeedInput` 或 `NeedOutput`；`Complete` 必须消费全部可见输入。 |
| 消费不完整尾部 | 保持其归调用方所有，并返回 `NeedInput` 或 `DecodeFailure::Incomplete`。 |
| 让 `encode_len` 仅是上界 | 它必须等于相同状态下实际成功 `encode` 的数量。 |
| 只按当前状态计算 reset/finish 大小 | 上界必须覆盖每一种可达状态。 |
| 用 `finish` 检查调用方输入 | `finish` 没有源输入；应在默认 bridge 外明确实现 EOF 策略。 |
| 在领域 crate 重写 engine 的游标逻辑 | 策略模型匹配时，使用严格 adapter 或带 hooks 的 `TranscodeXxxEngine`。 |

## 相关文档

- [README](../README.zh_CN.md)：crate 概述、特性清单和 API 地图。
- [English user guide](user_guide.md)：本手册英文版。
- Rust API 文档：详细的方法契约和错误变体。
