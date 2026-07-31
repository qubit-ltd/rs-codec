# Qubit Codec 用户手册

[English](user_guide.md) · [README](../README.zh_CN.md) · [API 文档](https://docs.rs/qubit-codec)

本手册适用于 `qubit-codec` 0.11 及 Rust 1.94 以上版本，面向需要把逻辑 value
转换为编码 unit，或在调用方管理的缓冲区中转换编码 unit 的 codec crate 与 adapter
作者。它不定义具体线格式或字符集。

## 手册目标与读者

当格式 crate 需要一个 codec quantum 的可复用契约、自有完整值 facade 或流式转换循环时，
使用本库。具体二进制布局、字符集和格式专属的 `std::io` 集成应位于相邻领域 crate。

贯穿场景是：一个格式 crate 先实现单个局部 value 的 `Codec`，随后希望同时提供自有
输出的便捷操作与缓冲区流式 API，又不想重复编写游标、容量和 EOF 规则。

## 概念模型

```text
逻辑 value <-> Codec <-> 编码 unit
                   |
       +-----------+------------+
       |                        |
自有单值 adapter             调用方缓冲区 transcoder
       |                        |
ValueEncoder / ValueDecoder  reset -> transcode* -> finish
```

应选择最小够用的层：

| 需求 | 推荐 API |
| --- | --- |
| 存在一个局部 value 边界 | 实现 `Codec` |
| 只有完整输入才是合理单位 | 直接实现 `ValueEncoder` / `ValueDecoder` |
| 已有 `Codec` 且无需自定义策略 | `CodecValue*` 或 `CodecTranscode*` adapter |
| 畸形或不可编码 value 需要策略 | `engine::TranscodeXxxEngine` 与 hooks |
| 需要 EOF 感知或其他专用流行为 | 实现 `Transcoder` |

格式化 hex 字符串或 C 字符串字面量常常没有合理的单值 quantum，直接用 value-level
实现更清晰；定宽数字或字符 codec 通常从 `Codec` 开始。

## 场景：以两个层次发布一个 codec

假定你的 crate 有一个局部 value codec，并有两种调用方：一种持有完整输入并需要自有
输出，另一种提供可复用缓冲区。先实现 `Codec`，再用 `CodecValueEncoder` /
`CodecValueDecoder` 服务前者，用 `CodecTranscodeEncoder` /
`CodecTranscodeDecoder` 服务严格的缓冲转换。

unit-to-unit 转换使用 `CodecTranscodeConverter` 构建严格的 decode-encode 管线；
只有 decode 或 encode 侧确实需要策略决策时，才使用
`engine::TranscodeConvertEngine`。

## 安装与最小配置

```toml
[dependencies]
qubit-codec = "0.11"
```

基础 crate 没有默认 feature。仅在使用 `TranscodeDecodeInput`、
`TranscodeEncodeOutput` 或部分 I/O 的 `AsyncTranscodeDecodeInput` 与 `AsyncTranscodeEncodeOutput`
（它们桥接 `qubit-io` buffered input/output trait）时启用 `io`：

```toml
[dependencies]
qubit-codec = { version = "0.11", features = ["io"] }
```

每次异步桥接调用在一次 transcoder 调用后即返回。返回的 progress 会在该调用再次
挂起前提交：按 `read()` 或 `written()` 推进调用方游标，再次调用以继续。decoder 的 EOF
会明确返回为 `AsyncTranscodeDecodeStep::EndOfInput`。

## 核心工作流

### 1. 实现 `Codec` 契约

`Codec` 声明 `Value`、`Unit`、错误类型以及每个 value 的最小/最大 unit 数。
`encode` 与 `decode` 是 unsafe 入口，checked adapter 会先做边界检查；实现仍必须
遵守其文档化前置条件。

| 规则 | 原因 |
| --- | --- |
| `MIN_UNITS_PER_VALUE` 与 `MAX_DECODE_UNITS_PER_VALUE` 均非零且有序 | 调用方据此安全进入 decode，并约束不完整输入。 |
| `encode_len` 是精确值且不超过 `MAX_ENCODE_UNITS_PER_VALUE` | 调用方可预留已知 value 的精确宽度或与 value 无关的编码上限；全缓冲 encoder 的该上限允许为零。 |
| `decode` 只读取可见输入 | 合法但过短的前缀返回 `DecodeFailure::Incomplete`，且不消费该尾部。 |
| 成功 `decode` 消费非零的可见数量 | 进度仍然可靠且可重试。 |
| `can_encode_value` 拒绝域外 value | checked encoder 会在 `encode_len` 与 `encode` 之前查询它。 |
| `encode_len` 与同状态下成功的 `encode` 输出精确相等 | 容量探测正确；允许刻意的零输出缓冲。 |
| 生命周期上界覆盖全部可达流状态 | 调用方可在 reset 或 finish 前安全分配。 |

无状态 codec 使用默认的零生命周期输出。有状态 encoder 如果在 EOF 写入 trailer，
必须声明 `MAX_ENCODE_FINISH_UNITS`；decoder 如果输出保留的最终 value，必须声明
`MAX_DECODE_FINISH_VALUES`。

### 2. 选择 adapter 或驱动流

自有完整值操作的 value trait 形状很小：

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
assert_eq!("encoded:codec", encoder.encode("codec")?);

# Ok::<(), core::convert::Infallible>(())
```

调用方缓冲区转换遵循显式生命周期：

```text
根据声明的上界分配输出
        |
reset(output)
        |
反复 transcode(input, input_index, output, output_index)
        |
保留/补充 NeedInput 尾部，或应用调用方自己的 EOF 策略
        |
finish(output)
```

`TranscodeProgress` 报告相对于传入下标的 `read` 与 `written` 数量；
`TranscodeStatus` 解释停止原因：

| 状态 | 调用方动作 |
| --- | --- |
| `Complete` | 从 `input_index` 起的全部可见输入均已消费。可提供下一段，或在 EOF 时 finish。 |
| `NeedInput` | 保留未消费尾部，补充输入后重试；EOF 时应用格式明确规定的尾部策略。 |
| `NeedOutput` | 排空或替换输出缓冲区，并按报告的进度继续。 |

不得在只消费前缀后返回 `Complete`。`finish` 不接收源输入，不能悄悄重新解释由
调用方持有的不完整尾部。

### 3. 安全完成并复用

调用方提供完全部输入并处理不完整尾部后，`finish` 输出内部保留的最终内容，例如
padding、checksum 或 stream trailer。成功后逻辑流关闭；在同一 transcoder 实例上开始
下一条逻辑流前，调用 `reset`。

`max_reset_output_len`、`max_transcode_output_len` 与
`max_finish_output_len` 必须对每种可达状态都保守成立。
`max_total_output_len` 将它们合成为一次完整 `reset -> transcode -> finish` 的上界；
`transcode_complete_into` 使用调用方提供的完整输出缓冲区执行该流程。

### Decode 生命周期输出

`CodecValueDecoder::decode` 是有意严格的：若 codec 从 `decode_reset` 或
`decode_finish` 产出 value，它会拒绝该 codec，因为单个返回值无法保留三个阶段的输出。
适合自有 `Vec` 输出时使用 `decode_lifecycle`；调用方持有可复用 reset 与 finish 缓冲区时
使用 `decode_lifecycle_with_scratch`。可运行的[生命周期感知 decode 示例](../examples/decode_lifecycle.rs)
展示了严格拒绝以及被保留的 reset、主值和 finish value。

## 进阶用法

严格的 `CodecTranscode*` adapter 直接暴露 codec-domain 错误。若格式需要
replacement、skip、count 或分阶段 report 等策略，应保留 engine 的公共循环，只实现
hooks：

- `engine::TranscodeEncodeEngine` 与 `TranscodeEncodeHooks` 处理不可编码 value、
  reset 与 finish 策略；
- `engine::TranscodeDecodeEngine` 与 `TranscodeDecodeHooks` 处理非法输入策略；
- `engine::TranscodeConvertEngine` 组合两侧及其 hooks。

若格式需要 EOF-aware maximal-munch 解析、延迟边界决策，或在 EOF 重新解释 pending
prefix，使用自定义 `Transcoder` 或 value-level facade。这类策略不能塞入 `finish`。

公开配置使用运行时 `ByteOrder`；希望静态选择时，使用 `ByteOrderSpec` 与
`BigEndian`、`LittleEndian` 或 `NativeEndian`。

## 错误与诊断

| 错误类型 | 含义与通常的恢复方式 |
| --- | --- |
| `DecodeFailure` | 底层可见前缀不完整或 codec-domain 输入非法。只有前者可重试。 |
| `TranscodeFailure` | framework failure：下标、容量、不完整整体输入、trailing input 或相关流条件。 |
| `CapacityError` | 写入输出前的容量规划算术失败。 |
| `TranscodeDomainError<E>` | 带 reset、main 或 finish 阶段标签的 domain/policy 错误。 |
| 方向性 transcode 错误 | 保留失败来源是 encode、decode 还是 conversion；encode/conversion 错误可保留不可编码 value。 |
| `TranscodeContractError` | 自定义 transcoder 报告了不一致进度，应视为实现缺陷。 |

除非下游调用方确实对两者采取相同恢复动作，不要把不完整前缀与畸形输入压平为同一错误。

## 排障

| 症状 | 检查方式 |
| --- | --- |
| 文件结尾出现 `NeedInput` | 保留尾部；按格式 EOF 策略处理它，之后调用 `finish`。 |
| `NeedOutput` 持续重复 | 用 `TranscodeProgress` 推进，提供相关容量，并确认自定义计数真实。 |
| checked adapter 拒绝容量 | 检查上界是否覆盖全部可达状态中的 reset、main 与 finish 输出。 |
| 单值 decode 拒绝输入 | `CodecValueDecoder` 是严格的；需要时改用 lifecycle-aware 或 streaming API。 |
| 格式需要错误替换 | 使用合适 engine 的 hooks，不要复制 buffered loop。 |

## 限制与最佳实践

- 具体格式、字符集和高层 reader/writer adapter 应留在领域 crate。
- 将 `NeedInput` 视为流式边界信号，而不是最终 EOF 错误。
- 保持 unsafe `Codec` 实现小而精确，确保长度与消费不变量严格成立。
- 自有输出 helper 会为返回值分配内存；需要控制分配时使用调用方缓冲区 API。
- 仅为实际使用的 `qubit-io` bridge 启用 `io`。

## 延伸阅读

- [README](../README.zh_CN.md)
- [English user guide](user_guide.md)
- [API 文档](https://docs.rs/qubit-codec)
