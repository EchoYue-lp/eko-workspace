---
title: 缓存命中优化：稳定前缀、有版本动态事实与协议后验收
artifact: design
carrier: markdown
---

# 缓存前缀稳定性设计

## 审查结论与证据

2026-11-07 独立只读实现审查结论为 **block**：3 项 Important、1 项 Minor，以及修复收益尚无实测的证据缺口。框架 HEAD `f2e44dc9be8972e286f9a84520ddb263f0012424`、应用 HEAD `5f5fd4912855cfda2326c54f294f11cb879257fa`，均须叠加实际工作区增量。没有运行 Cargo、App 或真实模型调用。

原记录为应用数据根 `echo-agent-cli/.echo-semantic/local/persistent-GUI-owner-66760c3-preparation-20261031-r01/isolated-runtime/data/runs/run_0e6c96fc-9b2c-408a-9357-462afe1ac266.jsonl`，SHA256 `877e196c2d7f18abe901276ce9227b91a4d60ef4cfdfe09b96c1be807c3c76df`。54 次请求累计输入 4,739,565，命中 1,540,352，实际为 32.49986%；其中 52 次命中固定为 29,440。对应检查点 SHA256 `e96ab169a28802560bf5ec9168f37174bea9e0779afe11fead82720abac1c309`，第 3、4 条消息为前置 Goal/recovery 动态投影。

| 编号 | 严重程度 | 根因与证据 | 处理 |
|---|---|---|---|
| CACHE-01 | Important | `phases/compact.rs:59-83` 每次 prepare 应用投影；`compact_context.rs:829,939` 混入每步变化的 tokens_used 等；`echo-state/compression/mod.rs:720-746` 删除旧投影并插到 system/history 边界。 | 动态事实不再改写已见历史前部。 |
| CACHE-02 | Important | `replace_tail_projection:791-804` 仍先删旧再追加；全部 projection 被永久 protected（`:853-856`）。hot-memory 当前是 System（`unified_memory.rs:320`），Anthropic 会将任意位置 System 合并到 wire 头（`anthropic.rs:370-387,639-650`）。 | 不能只换 tail，也不能每轮永久保护完整胶囊；需显式语义与生命周期。 |
| CACHE-03 | Important | `cache/diagnostic.rs:104-120` 未覆盖 tool description/strict、parts、tool calls、reasoning 等，适配前的粗 hash 不能证明最终 wire 稳定。 | 增加协议适配后的完整 prompt 指纹与首次差异定位。 |
| CACHE-04 | Minor | `cache/layout.rs:99-127` 按裸 `[runtime_context:` 猜尾段，实际正文有通用 projection envelope；还可能把其后的普通历史全算动态。 | 分段复用中立投影元数据，不按 Eclip marker 或文本猜测。 |

缓存低的结构性方向有原记录和源码支持，但没有修复前后相邻原始 HTTP prompt 的对照，不能量化其独占 miss 的比例，也不能承诺优化后固定达到 95%。两类粗 hash 稳定只证明其已覆盖字段稳定。

## 目标与范围

在有效前缀 generation 内，固定指令、工具及已经见过的历史保持内容和顺序；动态状态只在语义发生变化时追加新版本。旧版本可在原 compaction 安全点归并，长期运行不形成无限保护增长。以最终 provider 输入和真实 usage 分别验收结构与收益。

本设计覆盖框架现有 projection、context、cache layout/diagnostic 及三协议适配，Eclip 的 Goal/recovery、预算、记忆、进度和后台状态 producers。缓存率公式、Goal 权威、任务调度、权限和最终发布规则不借本次优化改写。不引入缓存服务器、SQLite、第二个 Context store 或另一套 plan/Goal API。

## 分层与复用

- 通用机制归框架：扩展原 `ContextProjection` 和 `ContextManager`，表达稳定替换与历史内动态版本、去重、清除、压缩保留及恢复。它不依赖 Eclip，未有第二消费者不构成拒绝理由；现有合理 replace API 保留其适用语义。
- Eclip 策略归应用：判断哪些内容是固定规则、哪些只是运行事实，生成语义版本、决定无界预算是否需告知模型。框架不识别 Eclip marker、不擅自把 System 降为 User。
- Provider adapter 归协议边界：完成 wire 转换、能力校验和缓存断点映射；通用元数据不外泄到 provider，不改 signed reasoning、native source 或显式能力错误。

复用已有 pre-model projector、projection scope、原 checkpoint、`PreparedContextCompaction`、`CacheHints/SegmentRanges` 和 canonical JSON。不得只新增抽象却继续使用前置重写主路径，应用不得再实现一套版本/保留主循环。

## 目标行为

### 固定指令与前缀 generation

产品身份、当前生效仓库规则、固定方法、工具集合及顺序在 generation 内稳定。真正的规则更新、工具/权限/模型/协议变化允许产生新 generation 与冷 miss；新规则及时生效，不能为命中率保留过期指令。

Goal 原文/revision 及稳定延续规则与每步预算值分开。动态运行投影不能成为 Goal 或验收的第二权威。用户原文、Steer、原始输入引用及其来源事实不得为缓存目的改写。

### 动态事实追加与去重

应用明确提供 source/owner、marker、语义版本和内容。相同 workspace/conversation/run 作用域下，同版本 prepare 重入、重试 prepare 和恢复重投影不重复追加；新版本在当前安全历史之后追加，保留旧版原身份，不在历史前部删旧插新。清除是一次显式失效版本；后续不重复产生清除事件。

只在工具调用/结果已闭合的原模型安全点追加，不能插断协议配对。最新快照明确取代该来源旧版本的当前意义，但不把原用户请求降为历史背景。

预算只在有实际限制且影响剩余额度时发送有界变化；无界预算不因每次 token 增加重发完整胶囊。原始输入引用按实际来源版本变化，不因无关 journal frontier 增长重发全文。后台 cell、待确认操作和进度提醒仅在相应事实变化时更新。

hot-memory 必须由应用区分规则与记忆事实：真实规则保留 System 权限，动态事实走明确的历史内上下文；不能把任意 MEMORY.md 文本自动降权。无法可靠分类的当前规则沿固定指令 generation 更新，并接受正常冷 miss。

### 有界保留、压缩与恢复

最新必要动态版本受保护；被后继取代的版本不永久 protected。原 compaction 在其既有 prepare/commit/CAS 安全边界归并旧快照，保留最新 Goal、原文引用和必要运行状态；跨压缩边界允许前缀重建。不得每轮追加完整 recovery 并永久保护，也不得通过降低验收内容换缓存。

原 checkpoint/envelope 携带恢复所需版本与来源。cold reopen/fork/subagent 进入自己的作用域，按当前权威状态追加纠正；旧胶囊不授予最新权限或完成资格。required projector 失败继续沿现有 fail-closed 模型准入路径退出。

### 协议后的分段与观测

同一投影元数据是 append 生命周期与 cache layout 的来源。已经不可变的旧动态快照属于可复用历史；本次新加入的动态尾段单独标识。core 和 provider 不再分别按正文前缀猜测，不能用 Eclip 固定 marker 污染框架。

在 Chat、Responses、Anthropic 各自适配完成后，对实际送入模型的语义 prompt 序列做规范化诊断：角色、顺序、内容 parts/工具定义全部有效字段、调用与结果、需要回传的 reasoning 均纳入；HTTP headers、密钥与传输随机 request ID 不进入模型前缀比较。不得重排有语义的数组。记录组件摘要、首次差异路径及最长共同前缀，不能把正常追加造成的全 history hash 变化误报为缓存破坏。

敏感正文不写普通日志。生产默认只输出有界摘要、数量与路径；需要精确比较时在隔离合成/脱敏样本中比较字段，原用户请求不复制到通用日志。签名与不透明 reasoning 只参与必要的有界摘要，不能展开输出。LCP 字段/字节度量明确标注，除非使用准确 tokenizer，不把字节数叫作 token 数。

### 真实统计与收益边界

仍消费 provider 的 `effective_prompt_tokens/cached_prompt_tokens`：cache write 不算 hit，inclusive input 不重复加缓存，缺 usage 不视为零命中。UI 的累计范围、覆盖缺口与恢复由 UI 设计负责，缓存实现不通过隐藏低值制造提升。

纯合同可证明前缀未被内部重写，loopback 可证明真实 adapter/wire 行为；真实 provider 的磁盘缓存为 best effort，只有同 vendor、endpoint、protocol、model 与 workload 的前后对照才能支持收益结论。调用前固定每组请求数、重复次数、执行顺序和总调用预算，冷启动与同一运行内历史连续增长阶段分开报告；不能反复发送同一完整请求来代替动态运行 workload 的验证。不能把三类验证混称为 live 95%。

## 参考实现与候选取舍

| 参考 | 已核对做法 | 本项目取舍 |
|---|---|---|
| DeepSeek 官方 Harness 0.2.2-rc.1，`dsh-system-prompt/README.zh.md:156` | 支持 in-history 更新的路由在缓存历史后追加变化；不支持的路由按能力重建 System。 | 采用显式能力/语义分类，不承诺任意 System 尾移都有效。 |
| 同安装包 `dsh-time-context/README.md:12,129` | 有来源的追加式时间读数、默认最小刷新间隔，旧记录可被 compaction shadow。 | 动态事实按需刷新，并有压缩生命周期。 |
| ZCode `context/builder.ts:240-282`、`turn-loop.ts:208-233` | 区分 identity/stable/dynamic，最终 provider projection 后设置缓存控制。 | 在最终请求上验收，而非只看抽象消息 hash。 |
| [DeepSeek 官方缓存说明](https://api-docs.deepseek.com/guides/kv_cache) | 复用匹配的前缀，构建和保留不保证每次命中。 | 验收内部稳定性与外部实际收益分别记录。 |

不采用只缩短系统提示、只调用 replace_tail、无条件保护每步完整快照或隐藏低命中值；它们分别未解决早部重写、旧节点删除、无限增长和真实收益问题。

## 验收标准

| 场景 | 必须成立 |
|---|---|
| 同 generation 连续至少三次真实构建链请求 | 已见历史的完整规范化字段及顺序不变；仅新增合法事实或原输出，预算变化不再改写早部。 |
| 同 prepare 重入、版本更新、清除 | 同版本恰一次；新版本只追加；清除一次；不跨 workspace/conversation/run 泄漏。 |
| 动态 System 与三协议 | 固定权限不变，Anthropic 头部变化符合真实规则更新，动态事实不会被重新提升为 System。 |
| 自动/手动压缩、重开、Fork/Subagent | 最新必需状态与原文可恢复，旧版本可归并；protected 量不随已过期版本无限增长。 |
| 工具、reasoning 与发布 | 调用/结果配对、原厂 ID、签名、Guard/Stop、pending transcript 与唯一最终发布保持。 |
| 粗 hash 未变负控 | tool description/strict、parts、工具参数或 reasoning 实际变更可在 post-adapter 诊断定位。 |
| usage 边界 | Chat inclusive、Anthropic disjoint、Responses details、0/缺 usage、write-only 均准确。 |
| 同模型 workload 前后对照 | 调用前固定组大小、重复次数、顺序及预算，冷/连续增长阶段分别记录真实 hit/miss、延迟、输入增长和压缩次数；保留自然失败，不用相同完整请求的暖命中替代原问题，未完成 live 时不声称收益达标。 |

## 协作、文档与交付边界

本文件是跨仓阶段设计；通用上下文/持久化合同与 ADR 放框架，产品 producer 策略和 ADR 放应用。公共构造/API 变化盘点整个框架及合理消费者，不以 CLI 是否使用决定删除；同步 examples 编译入口。SDK pin 和已接受跨语言合同独立，不自动升级。官网只在真实收益有证据时使用量化宣传。

与 UI 计划共享 usage/计时消费者，与执行计划共享 checkpoint/压缩路径时，按文件单一写入、冻结验证；不得并发修改同一 `snapshot.rs/think.rs`。计划必须先取得失败基线再切主路径，最后验证压缩/恢复与真实收益。本轮仅产出文档，未授权实施、提交、推送或模型/App 调用；`ready` 不代表已采用。
