# 单值解码生命周期输出设计

> 状态：已实现。

## 背景

修复前，`CodecValueDecoder::decode`、`TranscodeDecodeInput::read_decoded_with`
和 `read_decoded_with_scratch` 会执行完整的
`decode_reset -> decode -> decode_finish` 生命周期，但只返回主体
`Codec::decode` 产生的一个值。reset 和 finish 阶段产生的值会写入同一块 scratch，
随后被丢弃；finish 还可能覆盖 reset 的输出。

这种行为对无状态 codec 没有影响，但对声明了
`MAX_DECODE_RESET_VALUES > 0` 或 `MAX_DECODE_FINISH_VALUES > 0` 的 codec
会造成不可观察的数据丢失。单值便利 API 也无法清楚表达“一个主体值之外还存在
流起始值和流结束值”的结果。

## 目标

- 单值 API 不再静默丢弃生命周期输出。
- reset、主体 decode、finish 三个阶段的输出边界在类型和存储上都清晰可见。
- scratch 路径不要求 `Value: Clone`，并且 reset 与 finish 不共用存储。
- 普通无状态 codec 继续拥有简单的单值调用路径。
- 有状态 codec 可以选择显式 lifecycle API，或者直接使用 streaming engine。

## 不采用的方案

### 继续复用单块 scratch，只返回写入数量

该方案无法同时保留 reset 和 finish 输出；finish 会覆盖 reset 已写入的槽位。增加
写入数量只能让覆盖行为可见，不能恢复数据。

### 将所有单值 API 改成始终返回三个 `Vec`

该方案语义完整，但会让最常见的无状态 codec 调用承担不必要的结果包装和潜在分配，
也会破坏 `ValueDecoder` 的“一个输入对应一个输出”抽象。

### 只在实际写出生命周期值后报错

此时数据已经产生并被丢弃，而且 codec 状态已经推进。错误虽然不再静默，却不能让
调用方安全重试，因此拒绝时机太晚。

## 选定方案：严格单值 API 与 lifecycle-aware API 分层

### 严格单值 API

保留以下常用入口：

```rust
CodecValueDecoder::decode(&mut self, input)
TranscodeDecodeInput::read_decoded_with(&mut self, codec, map_error)
```

它们只接受同时满足以下条件的 codec：

```rust
C::MAX_DECODE_RESET_VALUES == 0
    && C::MAX_DECODE_FINISH_VALUES == 0
```

检查必须发生在 reset、读取输入或修改 codec 状态之前。若 codec 声明了非零生命周期
输出上界，value 层返回计划新增的框架错误：

```rust
TranscodeFailure::UnsupportedDecodeLifecycleOutput {
    reset_bound: usize,
    finish_bound: usize,
}
```

I/O 层将同一条件映射为 `std::io::ErrorKind::Unsupported`。这样现有无状态调用保持
简洁，有状态调用则不会产生任何被丢弃的值。

原 `read_decoded_with_scratch` 应删除，不保留兼容别名。它的 API 形状只有一块共享
scratch，无法满足新的不覆盖契约。

### Owned lifecycle API

新增拥有全部阶段输出的结果类型：

```rust
pub struct DecodeLifecycleOutput<V> {
    reset: Vec<V>,
    value: V,
    finish: Vec<V>,
}
```

提供只读访问器和 `into_parts`，字段保持私有，以便后续调整存储实现。新增入口：

```rust
CodecValueDecoder::decode_lifecycle(&mut self, input)

TranscodeDecodeInput::read_decoded_lifecycle_with(
    &mut self,
    codec,
    map_error,
)
```

这两个便利方法分别按 reset 和 finish 的声明上界准备独立存储，成功时把实际写入
部分截断后放入 `DecodeLifecycleOutput`。主体输入仍必须恰好编码一个值；尾随输入继续
报告 `TrailingInput`。

### Scratch lifecycle API

新增无额外分配的结果类型：

```rust
pub struct DecodeLifecycleProgress<V> {
    value: V,
    reset_written: usize,
    finish_written: usize,
}
```

以及分别接收两块存储的入口：

```rust
CodecValueDecoder::decode_lifecycle_with_scratch(
    &mut self,
    input,
    reset_output,
    finish_output,
)

TranscodeDecodeInput::read_decoded_lifecycle_with_scratch(
    &mut self,
    codec,
    reset_output,
    finish_output,
    map_error,
)
```

`reset_output` 与 `finish_output` 必须分别覆盖对应声明上界。方法返回主体值以及两段
实际写入数量，调用方通过
`&reset_output[..reset_written]` 和 `&finish_output[..finish_written]`
观察生命周期输出。两块 slice 不得由库内部合并或复用。

第一阶段继续使用已初始化的 `&mut [V]`，因此 owned 便利方法仍要求
`V: Default`。改用 `MaybeUninit<V>`、追踪部分初始化和保证 drop safety 属于独立
设计，不与本次语义修复绑定。

## 执行顺序与错误语义

完整调用严格遵循：

1. 校验 unit bounds、生命周期 bounds 和 scratch 容量。
2. 执行 `decode_reset`，记录 `reset_written`。
3. 解码恰好一个主体值并拒绝尾随输入。
4. 执行 `decode_finish`，记录 `finish_written`。
5. 组装结果。

容量或声明错误必须在任何生命周期 hook 运行前返回。domain error 继续保留 reset、
main、finish 阶段信息。若 main 或 finish 失败，已写入的早期阶段值不作为成功结果
暴露；codec 的错误恢复契约与现有 streaming API 保持一致。

## 迁移方案

- reset/finish 上界均为零的调用方继续使用 `decode` 或
  `read_decoded_with`，无需改变结果处理。
- 当前依赖 `read_decoded_with_scratch` 但 codec 实际无生命周期输出的调用方改用
  `read_decoded_with`。
- 需要 reset/finish 输出的调用方改用新的 lifecycle-aware API；需要持续流语义时
  改用 `TranscodeDecodeEngine`。
- 不提供旧方法 re-export、别名或隐式兼容层。

## 测试方案

- 无状态 codec 的严格单值路径保持原行为。
- 非零 reset 或 finish 上界在运行任何 hook、读取任何输入前被严格单值路径拒绝。
- owned API 同时保留 reset、主体值和 finish 输出，且顺序正确。
- scratch API 的 reset 与 finish 输出写入不同 slice，不发生覆盖。
- reset、main、finish domain error 分别保持阶段信息。
- scratch 任一侧容量不足时不执行 codec hook。
- over-report、over-consume、不完整输入和尾随输入继续触发既有契约检查。
- I/O API 在拒绝 unsupported lifecycle codec 后不消费底层输入，可在换用正确 API 后
  重试。
