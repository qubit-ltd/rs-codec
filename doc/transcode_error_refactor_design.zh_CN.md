# Transcode 错误与生命周期设计

本文描述当前 `rs-codec` 的实现，而不是历史迁移方案。代码允许破坏性变更，因此
这里以最小、可验证的公共模型为准。

## 设计边界

transcode 层只负责缓冲区游标、容量、进度和流生命周期。codec 或 hook 负责格式域
语义。运行阶段的方向错误通过方向错误类型承载框架失败和领域错误；容量规划及进度
契约校验另有独立类型和入口：

1. `CapacityError`：容量规划无法表示所需输出上界时报告。
2. `TranscodeContractError`：校验 transcoder 返回的进度是否符合调用边界。
3. `TranscodeFailure`：运行阶段框架可独立判断的失败。
4. `TranscodeDomainError<E>`：带 `reset`、`main`、`finish` 阶段的 codec/hook 域错误。
5. 方向错误：解码、编码、转换分别承载各自可能出现的错误方向。

## 公共错误类型

`TranscodeFailure` 是无泛型、`#[non_exhaustive]` 的框架错误，当前包含：

- 输入/输出下标、输出范围和输出容量错误；
- 输出长度算术溢出；
- 不完整输入和尾随输入；
- 严格单值 decode 不支持 codec 生命周期输出；
- `TranscodeBeforeReset`、`FinishBeforeReset`；
- `TranscodeAfterFinish`、`FinishAfterFinish`；
- reset 或 finish 部分执行失败后的 `LifecyclePoisoned`。

`AllocationFailed` 表示实际输出存储分配失败；它不属于容量上界计算错误。
容量规划 API（如 `max_*_output_len`）返回 `CapacityError`，当前只有
`OutputLengthOverflow`。该错误转换为方向错误的 `Failure(TranscodeFailure::OutputLengthOverflow)`，
不会表示分配器实际分配失败。

`TranscodeProgress::validate` 返回独立的 `TranscodeContractError`，用于报告进度违反
调用契约：`OverRead`、`OverWritten`、`ProgressIndexOverflow`、`SatisfiedNeed` 和
`CompleteWithRemainingInput`。直接调用 `validate` 时，调用方收到
`TranscodeContractError`。完整转码的默认驱动校验不可信进度后，会把它包装为
`TranscodeFailure::InvalidProgress { source }`，再作为方向错误的 `Failure` 返回；I/O
进度校验则返回 `io::Error`，错误种类为 `InvalidData`，并保留契约错误作为 source。
EOF 后若 decoder 仍返回 `NeedInput`，I/O 层会以 `InvalidData` 拒绝该状态，但该错误
不保证以契约错误为 source。拒绝进度不会回滚 codec 状态或已经写入的输出。

三类方向错误的实际形状如下：

```rust
enum TranscodeDecodeError<E> {
    Failure(TranscodeFailure),
    Domain(TranscodeDomainError<E>),
}

enum TranscodeEncodeError<E, V> {
    Failure(TranscodeFailure),
    Unencodable { input_index: usize, value: Option<V> },
    Domain(TranscodeDomainError<E>),
}

enum TranscodeConvertError<DE, EE, V> {
    Failure(TranscodeFailure),
    DecodeDomain(TranscodeDomainError<DE>),
    EncodeDomain(TranscodeDomainError<EE>),
    Unencodable { input_index: usize, value: Option<V> },
}
```

运行阶段方向错误中的 `Failure(TranscodeFailure)` 承载框架失败；容量规划由
`CapacityError` 表达，进度校验由 `TranscodeContractError` 表达。转换关系如下：

| 来源 | 转换或包装 | 最终承载位置 |
| --- | --- | --- |
| `CapacityError::OutputLengthOverflow` | `From<CapacityError> for TranscodeFailure` 映射为 `TranscodeFailure::OutputLengthOverflow` | 解码、编码或转换错误的 `Failure` 分支 |
| 实际输出分配失败 | engine/value 边界映射为 `TranscodeFailure::AllocationFailed`；buffered I/O adapter 通过 `allocation_error` 转成 I/O 分配错误 | 对应方向错误的 `Failure` 分支，或 adapter 返回的 I/O 错误 |
| 直接调用 `TranscodeProgress::validate` | 不自动转换 | 调用方接收 `TranscodeContractError` 并自行处理 |
| 完整转码默认驱动发现无效进度 | 包装为 `TranscodeFailure::InvalidProgress { source }` | 方向错误的 `Failure` 分支 |
| I/O 进度校验发现无效进度 | 包装为 `io::Error`，种类为 `InvalidData`，契约错误作为 source | I/O adapter 返回的错误 |
| EOF 后 I/O decoder 返回 `NeedInput` | 以 `io::Error(InvalidData)` 拒绝；source 不保证是契约错误 | I/O adapter 返回的错误 |

结构错误的 `invalid_*` 与 `ensure_*` 构造和校验方法仍定义在 `TranscodeFailure` 上。
方向错误可通过 `From` 实现或 `?` 包装框架失败。`map_domain` 与 `map_value` 分别映射
方向错误各自分支中的领域错误或 value 类型，不会把错误统一转换为 `Failure`。

## 强制生命周期

所有内置 transcode engine 都强制遵循，I/O adapter 则按该顺序调用底层
`Transcoder`：

```text
reset → transcode* → finish
          ↑       ↓
       可返回 NeedInput / NeedOutput
```

新建实例处于 `Uninitialized`，只有一次成功的 `reset` 后才能调用 `transcode` 或
`finish`。`finish` 成功后实例处于 `Finished`，必须再次 `reset` 才能开始下一条流。
reset 或 finish 在 codec/hook 已可能修改状态后失败，会进入 `Poisoned`；只有成功的
后续 reset 能恢复。容量和下标预检失败不改变可重试状态。

`LifecycleGuard` 在所有构建 profile 都执行检查，避免 debug/release 行为分叉。对应
的错误是 `TranscodeBeforeReset`、`FinishBeforeReset`、`TranscodeAfterFinish`、
`FinishAfterFinish` 和 `LifecyclePoisoned`。

`Transcoder` trait 允许自定义实现直接定义三个方法，因此 trait 本身不能替实现者
拦截任意状态突变；自定义实现必须遵守同一契约，或由调用方包在能执行该检查的
engine 中。

## 编码 hook 上下文

`EncodeContext<'a, Value>` 是只读值对象，只暴露：

```rust
input_value() -> &Value
input_index() -> usize
output_index() -> usize
available_output() -> usize
```

hook 不再取得可写输出切片，也不能伪造写入计数。引擎内部的私有
`EncodeAttempt` 独占 `&mut [Unit]`，负责把 hook 返回的策略（拒绝、跳过或替换）落实
到 codec 调用和进度结算中。这样输出写入、边界检查和 `written` 计数只有一个可信来源。

## 引擎执行顺序

- `TranscodeDecodeEngine`：reset hook → `Codec::decode_reset` → 重复 decode →
  `Codec::decode_finish`/finish hook。
- `TranscodeEncodeEngine`：reset hook → `Codec::encode_reset` → 重复 encode →
  `Codec::encode_finish`/finish hook。
- `TranscodeConvertEngine`：先 reset 目标 encoder，再处理源 decoder 的 reset 值；
  streaming 阶段在内部 pending slot 中保留尚未写出的值，finish 时先排空 pending，
  再处理 decoder finish 值，最后完成 encoder finish。

错误中的输入下标坐标由具体适配器定义。converter 的普通解码值使用相对于当前输入
切片的真实来源下标；库不会自动累计跨分片的全流偏移。源 codec 的 decode reset 或
finish 生成值没有对应源 unit，编码上下文及错误使用占位下标 0；该占位值无法与源下标
0 的普通解码值区分。错误阶段描述底层 codec 操作，因此这类值即使在 converter 的
reset 或 finish 中编码，目标编码错误仍属于 main 阶段。`TranscodeProgress` 计数则是
相对本次调用的读取和写入量。容量 bound 必须覆盖 reset、transcode、finish 的所有
可达瞬态。

## EOF 语义

普通 `transcode` 只处理当前可见输入；`NeedInput` 的尾部仍由调用方持有。调用方声明
EOF 后，可使用 codec-backed decoder 的 `transcode_eof`，由 codec 的
`decode_eof` 决定是否能完成尾部。`finish` 不接收或重新解释调用方的输入尾部，只
处理引擎已经保留的状态和 codec/hook 的 finish 输出。

## 取舍与暂缓项

- 不恢复旧的 phantom `FailureValue`/统一错误枚举；方向错误直接表达 decode、encode
  和 convert 语义。
- 方向错误保留各自的错误分支；结构错误继续复用 `TranscodeFailure`，不恢复已移除的
  forwarding helper。
- `D::Value: Default` 对 converter finish 冷路径的限制暂不改变；普通 transcode 热
  路不依赖该 bound，后续可另行设计无需 Default 的 finish scratch。
- 不把 `EncodeContext` 的内部输出权限暴露给公共 hook API；若未来确需更强扩展点，
  应新增显式的 engine-owned writer，而不是恢复裸 `&mut [Unit]`。

## 验证策略

生命周期测试覆盖首次 reset 前的拒绝、reset 后 streaming、finish 后拒绝和 reset 复用。
编码上下文测试验证 hook 只能观察元数据。`benches/transcode.rs` 中的
`safe_vs_unchecked` 基准对比安全索引与 `get_unchecked`，并配合 release 汇编输出检查
优化器是否已经消除边界检查；基准结果不作为 API 正确性的依据。

上述约束与 `src/transcode`、`tests/transcode`、README 及下游 codec/text adapter 的
当前实现保持一致。
