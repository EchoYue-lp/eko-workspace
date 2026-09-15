# echo-agent 全 Workspace 语义治理优化清单

> 快照：2026-09-16，`echo-agent@0e09324a`。本报告是本轮跨仓工程总结；长期行为、状态权威和 Finding 事实仍以 `echo-agent/.echo-semantic/` 为准。

## 总体结论

项目治理单位已从数千个 Rust public identity 收敛为 Capability、Behavior、Rule、状态权威、生命周期、Finding 和 Evidence。全仓共形成 95 个可追踪 Finding，其中 45 个已完成修复、验证、独立复审和远端交付，50 个仍保持 open；95 个 Finding 均映射唯一 GitHub Issue，远端状态为 45 CLOSED / 50 OPEN。SDK 独立仓库切换另由协调 Issue #122 跟踪，不混入原 95 个 Finding 统计。

SDK identity inventory 继续用于 API 漂移监控，但不再表示项目完成度。当前 9,724 个 canonical identity 的 scope 是 5,622 external contract、1,774 Host/Rust-only、790 language intrinsic、90 internal helper、1,448 deferred；后续只按可交付 capability 推进 deferred backlog。

生命周期结算 Wave 1 已通过 [PR #121](https://github.com/EchoYue-lp/echo-agent/pull/121) squash merge 为 `c5f76882`，Plugin component isolation 通过 [PR #123](https://github.com/EchoYue-lp/echo-agent/pull/123) 合入 `7e74d144`，持久化与 Workflow 权威 Wave 2 通过 [PR #124](https://github.com/EchoYue-lp/echo-agent/pull/124) 合入 `29cea08c`，Kubernetes cleanup settlement Wave 3 通过 [PR #125](https://github.com/EchoYue-lp/echo-agent/pull/125) 合入 `0e09324a`。Wave 3 的 SDK 合同、两档 workspace Clippy、all-target/all-feature 测试、no-default-features、17-feature 矩阵、semantic strict/change-evidence 与独立复审均通过；远端 Linux、Windows、SDK contract、三语言 SDK 和 dependency policy 共 10 项 CI 全绿。新增关闭 #62。

| 语义边界 | Finding | 已修复 | 待优化 |
| --- | ---: | ---: | ---: |
| Workspace 架构与公共组合 | 4 | 4 | 0 |
| Agent / Session / Turn | 2 | 0 | 2 |
| Context 与 Memory | 3 | 0 | 3 |
| Task / Subagent / Workflow | 16 | 12 | 4 |
| Observation / Persistence / Delivery | 6 | 2 | 4 |
| Tool / Permission / Sandbox | 16 | 5 | 11 |
| Extension / MCP / LSP / Plugin | 14 | 8 | 6 |
| LLM / Provider | 8 | 2 | 6 |
| Protocol / A2A / Channel / SDK | 9 | 2 | 7 |
| SDK facade parity | 5 | 5 | 0 |
| Eval / Improve / Evolution | 12 | 5 | 7 |
| **合计** | **95** | **45** | **50** |

## 已完成优化

### Workspace 架构

- [#114](https://github.com/EchoYue-lp/echo-agent/issues/114) 双语 README workspace topology 遗漏 SDK crates：已按真实 11-package Cargo DAG 统一。
- [#79](https://github.com/EchoYue-lp/echo-agent/issues/79) README feature 表列出不存在的 `tasks` feature：已改为 Task core capability，并用 Cargo metadata 合同约束。
- [#80](https://github.com/EchoYue-lp/echo-agent/issues/80) README 把 demo34 test contract 写成 Cargo example target：已路由到真实测试 target。
- [#118](https://github.com/EchoYue-lp/echo-agent/issues/118) Squash merge 后 semantic baseline 中间提交不再属于 main 祖先：baseline 已绑定 canonical main，learning/CI contract 使用真实 target SHA 和完整 Git 历史在合并前阻断 feature-only revision，并以真实 squash 反例防回归。

### Task / Subagent / Background

- [#27](https://github.com/EchoYue-lp/echo-agent/issues/27) Task relation patch 可覆盖 live claim：已把 execution snapshot 纳入 CAS，runtime mutation 返回 typed conflict。
- [#25](https://github.com/EchoYue-lp/echo-agent/issues/25) Subagent lazy factory 存在双创建窗口：已使用 revision-scoped single-flight publication。
- [#29](https://github.com/EchoYue-lp/echo-agent/issues/29) Subagent factory 取消后无法恢复 publication：已统一取消、旧代 fencing 和重试。
- [#39](https://github.com/EchoYue-lp/echo-agent/issues/39) BackgroundTask 多观察者可能永久等待：已统一共享终态、单消费者结果、取消和绝对 deadline。
- [#44](https://github.com/EchoYue-lp/echo-agent/issues/44) CommandCell cancel 可卡在 artifact finalization：finalizer 已监听 cell/owner cancellation，终态只在 typed interruption 记录后发布。
- [#45](https://github.com/EchoYue-lp/echo-agent/issues/45) CommandCell retention prune 可删除并发新 lease：候选扫描后的原子谓词复核阻止误删 live waiter。
- [#85](https://github.com/EchoYue-lp/echo-agent/issues/85) Scheduler disable/remove 与已捕获 callback 竞态：control generation fence 使旧 occurrence 失效。
- [#86](https://github.com/EchoYue-lp/echo-agent/issues/86) Scheduler Task ID 不唯一：创建与恢复路径统一执行唯一性校验。
- [#109](https://github.com/EchoYue-lp/echo-agent/issues/109) Workflow checkpoint claim 失败后不可恢复：claim 后失败统一 requeue，成功统一 ack。
- [#110](https://github.com/EchoYue-lp/echo-agent/issues/110) Workflow tag 可复活旧 claim：generation、renew、ack/requeue 共同 fence，Store 不支持 settlement 时 fail closed。
- [#113](https://github.com/EchoYue-lp/echo-agent/issues/113) Workflow sibling 失败被遮蔽：并行批次 fail-fast，成功结果按稳定注册顺序归并。
- [#112](https://github.com/EchoYue-lp/echo-agent/issues/112) Workflow 四个公开入口各自维护主循环：已统一投影同一 `Graph::execute_loop`，Token、NodeError、Completed 与 checkpoint/resume 顺序一致，drop/cancel/timeout/sibling failure 均结算 Agent producer。

### Observation / Delivery

- [#108](https://github.com/EchoYue-lp/echo-agent/issues/108) Turn terminal commit 与 projection 顺序冲突：执行结果与 delivery 结果分离，sink/projection 失败不再改写 producer terminal；ACP 只在 Completed + Delivered 时返回 EndTurn。
- [#43](https://github.com/EchoYue-lp/echo-agent/issues/43) Checkpoint 未绑定来源 Journal identity：Memory/File/Segmented Journal 统一持久化 generation identity，batch/checkpoint/retention marker 与 receipt 共同校验；完整事实可从 0 重建，已裁剪事实遇到异源 checkpoint 时 fail closed。

### Extension / MCP / LSP / Skill

- [#56](https://github.com/EchoYue-lp/echo-agent/issues/56) Extension credential Debug/redaction 不统一：共享 redaction 覆盖诊断、配置与 channel/MCP 路径。
- [#59](https://github.com/EchoYue-lp/echo-agent/issues/59) Hook source order 可绕过 deny：全部匹配 Permission action 采用全局 deny-wins，stop propagation 不能隐藏后续 deny。
- [#63](https://github.com/EchoYue-lp/echo-agent/issues/63) 派生 LSP handle 可在 manager close 后复活：manager 持有 child、generation 和 closed fence。
- [#65](https://github.com/EchoYue-lp/echo-agent/issues/65) MCP client 广告未实现 capability：广告集合只保留真实支持的 client 能力。
- [#66](https://github.com/EchoYue-lp/echo-agent/issues/66) MCP annotation 被误当权限事实：annotation 仅作提示，本地 `ToolCapabilities` 成为 permission/risk/effect 分类权威，Plan mode 在 hook 前硬阻断 mutating tool。
- [#67](https://github.com/EchoYue-lp/echo-agent/issues/67) MCP 协议版本文档漂移：协商版本、实现与中英文文档已同步。
- [#93](https://github.com/EchoYue-lp/echo-agent/issues/93) Skill activation 双状态权威：统一 activation handle/epoch/generation/single-flight，替换策略先验证再撤旧代。
- [#71](https://github.com/EchoYue-lp/echo-agent/issues/71) Plugin component isolation 与 atomic generation 冲突：无效 Skill/Hook/MCP 单组件被隔离并保留结构化诊断，健康兄弟仍在同一不可变 generation 发布；依赖闭包或完整 Plugin 准备失败继续阻断整代。

### LLM / Protocol / SDK

- [#69](https://github.com/EchoYue-lp/echo-agent/issues/69) Non-stream LLM cancellation 不对等：send、body、decode 与 projection 全链路监听统一 cancellation。
- [#95](https://github.com/EchoYue-lp/echo-agent/issues/95) SSE EOF 接受无 delimiter JSON：delimiterless EOF 现在 fail closed。
- [#88](https://github.com/EchoYue-lp/echo-agent/issues/88) 三语言 SDK gap generation 校验不对等：Host/TypeScript/Python/Java 统一校验完整 handle generation 与 sequence；后续 ACK watermark 问题独立为 #120。

### Tool / Cache / Registry

- [#26](https://github.com/EchoYue-lp/echo-agent/issues/26) Streaming Tool 跳过统一参数校验：已在 cache、permit 和 effect 前复用同一 validation kernel。
- [#30](https://github.com/EchoYue-lp/echo-agent/issues/30) Tool read cache 缺 workspace identity：cache key 已纳入 effective workspace 与 invocation lineage。
- [#28](https://github.com/EchoYue-lp/echo-agent/issues/28) In-flight Read 可在 Write 后复活陈旧 cache：已使用 write-lifetime epoch CAS 阻止旧值发布。
- [#115](https://github.com/EchoYue-lp/echo-agent/issues/115) Tool registry mutation 被跨 await guard 阻塞：执行路径改为 owned handle，保留 generation/freshness fence。
- [#62](https://github.com/EchoYue-lp/echo-agent/issues/62) K8s Sandbox Pod 清理缺少可靠 owner settlement：detached owner 统一持有 kubectl 进程组、stdin、pipe drain、绝对 deadline、caller abandonment 与 Pod 删除；删除成功要求具名 receipt 和 confirmed absence，首次 NotFound 后延迟提交的 Pod 会被重删，持续歧义转为 typed cleanup debt。

### Eval / Improve

- [#24](https://github.com/EchoYue-lp/echo-agent/issues/24) 单 case criteria 分组可 panic：已采用 train-only disposition，消除非法 clamp 区间。
- [#31](https://github.com/EchoYue-lp/echo-agent/issues/31) `EvalDrivenImprovement` 忽略 `max_iterations`：配置已无损传入唯一 ImprovementLoop，并定义零迭代行为。
- [#50](https://github.com/EchoYue-lp/echo-agent/issues/50) Eval/Improve workspace 缺 generation 隔离：每次 case 使用唯一 generation，settled 后清理，未结算时保留诊断目录。
- [#48](https://github.com/EchoYue-lp/echo-agent/issues/48) Eval timeout 后未等待 Turn settlement：取消后有界等待唯一 TurnReceipt，未结算不评分、不清理。
- [#49](https://github.com/EchoYue-lp/echo-agent/issues/49) Eval 用 product run ID 猜测 trace：已使用 invocation-scoped run/turn/execution correlation。

### SDK facade 与治理

- [#87](https://github.com/EchoYue-lp/echo-agent/issues/87) AgentComponent stream 终态与事件形状混淆：已分离 typed stream item、terminal 和 producer close。
- [#89](https://github.com/EchoYue-lp/echo-agent/issues/89) MCP 初始化后 publication 失败未关闭 transport：失败路径已结算资源。
- [#90](https://github.com/EchoYue-lp/echo-agent/issues/90) facade/bridge/improve feature 组合未闭合：feature、test target、CI 与 Linux linker 边界已补齐。
- [#91](https://github.com/EchoYue-lp/echo-agent/issues/91) Sandbox bridge 丢失取消分类：已保留 typed cancellation semantics。
- [#92](https://github.com/EchoYue-lp/echo-agent/issues/92) SkillLoadPolicy 被误归为 process-local：已接入真实 Host/consumer bridge。
- [#116](https://github.com/EchoYue-lp/echo-agent/issues/116) SDK backlog 混用 4,076 intrinsic 旧口径：已统一为 deferred identity 的 capability backlog，其他 scope 不属于语言 parity backlog；当前随 Rust facade 漂移监控为 1,448 项。

## 待优化清单

### Agent / Session / Turn

- [H][#36](https://github.com/EchoYue-lp/echo-agent/issues/36) Agent adapter close 与资源结算 owner 未闭合。
- [H][#107](https://github.com/EchoYue-lp/echo-agent/issues/107) Channel adapter 绕过统一 driven Turn authority。

### Context 与 Memory

- [M][#42](https://github.com/EchoYue-lp/echo-agent/issues/42) `AgentCheckpoint.current_plan` 是孤立恢复权威，尚未和 canonical Task artifact 对齐。
- [H][#105](https://github.com/EchoYue-lp/echo-agent/issues/105) Runtime state 与 transcript generation identity 可写出不可恢复 checkpoint。
- [H][#106](https://github.com/EchoYue-lp/echo-agent/issues/106) Transcript projection 写入失败缺少结算合同。

### Task / Subagent / Workflow

- [H][#84](https://github.com/EchoYue-lp/echo-agent/issues/84) Scheduler 缺 durable occurrence claim 与 crash replay 合同；当前 generation/control 修复不代表持久 delivery 已闭合。
- [M][#98](https://github.com/EchoYue-lp/echo-agent/issues/98) Definition-only Subagent catalog 合同冲突。
- [H][#99](https://github.com/EchoYue-lp/echo-agent/issues/99) TaskClaim 与 SubagentAttempt identity 未闭合。
- [M][#111](https://github.com/EchoYue-lp/echo-agent/issues/111) Task DAG 与 Workflow DAG 仍是平行实现候选。

### Observation / Persistence / Delivery

- [H][#46](https://github.com/EchoYue-lp/echo-agent/issues/46) Trace 与 Audit 持久化失败缺少统一可见结果。
- [M][#58](https://github.com/EchoYue-lp/echo-agent/issues/58) HookEvent catalog 与自动 producer 不一致。
- [M][#61](https://github.com/EchoYue-lp/echo-agent/issues/61) InMemoryAuditLogger 丢写仍返回成功。
- [H][#103](https://github.com/EchoYue-lp/echo-agent/issues/103) Trace 与 audit 没有统一 secret retention 合同。

### Tool / Permission / Sandbox

- [H][#37](https://github.com/EchoYue-lp/echo-agent/issues/37) PermissionService 与 Shell CommandPolicy 存在双重 approval authority。
- [H][#47](https://github.com/EchoYue-lp/echo-agent/issues/47) Artifact、Sandbox 与 Worktree cleanup owner 未闭合。
- [M][#57](https://github.com/EchoYue-lp/echo-agent/issues/57) Guard ToolInput/ToolOutput 与生产可达性错位。
- [H][#60](https://github.com/EchoYue-lp/echo-agent/issues/60) Hook Allow 可绕过 protected-path decision。
- [H][#70](https://github.com/EchoYue-lp/echo-agent/issues/70) Plan mode 未形成可靠只读 surface。
- [H][#81](https://github.com/EchoYue-lp/echo-agent/issues/81) `readonly_tools` 不约束 custom Write/Execute Tool。
- [M][#82](https://github.com/EchoYue-lp/echo-agent/issues/82) SandboxManager 建流失败丢失 typed Failed 终态。
- [H][#83](https://github.com/EchoYue-lp/echo-agent/issues/83) Sandbox minimum isolation 可被 fallback 降级。
- [M][#101](https://github.com/EchoYue-lp/echo-agent/issues/101) demo64 Tool pipeline 合同与生产顺序漂移。
- [H][#102](https://github.com/EchoYue-lp/echo-agent/issues/102) Tool caller、trace 与 audit 可记录不同终态。
- [M][#104](https://github.com/EchoYue-lp/echo-agent/issues/104) Trace Permission/File/Test 事件缺少生产点。

### Extension / MCP / LSP / Plugin

- [H][#55](https://github.com/EchoYue-lp/echo-agent/issues/55) MCP SSE 与 SDK LSP cleanup 未等待结算。
- [H][#64](https://github.com/EchoYue-lp/echo-agent/issues/64) LSP runtime status 与 restart 字段未闭合。
- [H][#72](https://github.com/EchoYue-lp/echo-agent/issues/72) Plugin wiring 缺 active generation authority。
- [H][#73](https://github.com/EchoYue-lp/echo-agent/issues/73) Plugin Registry、wiring 与 callback lifecycle 未统一编排。
- [M][#74](https://github.com/EchoYue-lp/echo-agent/issues/74) Plugin lifecycle reconcile 可形成两代资源重叠。
- [H][#75](https://github.com/EchoYue-lp/echo-agent/issues/75) Plugin MCP server 名缺少 owner 隔离。

### LLM / Provider

- [M][#68](https://github.com/EchoYue-lp/echo-agent/issues/68) 内置动态 Model facts 缺 freshness authority。
- [H][#77](https://github.com/EchoYue-lp/echo-agent/issues/77) Provider capabilities 与 ModelProfile authority 未闭合。
- [H][#78](https://github.com/EchoYue-lp/echo-agent/issues/78) Provider stream semantic terminal 不对等。
- [H][#96](https://github.com/EchoYue-lp/echo-agent/issues/96) Agent structured output 配置未进入主 ReAct request。
- [H][#97](https://github.com/EchoYue-lp/echo-agent/issues/97) Strict structured output 缺少 framework schema validation。
- [M][#100](https://github.com/EchoYue-lp/echo-agent/issues/100) Tokenizer calibration 的生产反馈未收敛到真实比例。

### Protocol / A2A / Channel / SDK

- [H][#32](https://github.com/EchoYue-lp/echo-agent/issues/32) A2A 宣告的 file/push capability 未绑定实现。
- [H][#33](https://github.com/EchoYue-lp/echo-agent/issues/33) A2A streaming cancel 与 cleanup 未闭合。
- [H][#34](https://github.com/EchoYue-lp/echo-agent/issues/34) A2A 重复 task ID 缺 admission generation authority。
- [H][#35](https://github.com/EchoYue-lp/echo-agent/issues/35) A2A server 自行拥有第二套执行终态。
- [M][#40](https://github.com/EchoYue-lp/echo-agent/issues/40) Channel attachment 在 Agent adapter 中丢失。
- [H][#41](https://github.com/EchoYue-lp/echo-agent/issues/41) Channel reset 后旧 generation 回复无法 fencing。
- [H][#120](https://github.com/EchoYue-lp/echo-agent/issues/120) SDK gap ACK 后 Host resume watermark 可回退并重发已被 snapshot 覆盖的事件。

### Eval / Evolution / Memory Improvement

- [M][#38](https://github.com/EchoYue-lp/echo-agent/issues/38) Background Review 丢 handle 后持久化无人结算。
- [H][#51](https://github.com/EchoYue-lp/echo-agent/issues/51) Evolution mutation 与 change audit 非原子。
- [H][#52](https://github.com/EchoYue-lp/echo-agent/issues/52) Evolution ChangeLog 不能作为 later rollback authority。
- [M][#53](https://github.com/EchoYue-lp/echo-agent/issues/53) Evolution 文档 namespace 与代码漂移。
- [H][#54](https://github.com/EchoYue-lp/echo-agent/issues/54) Skill Curator promotion 缺可验证批准与 audit authority。
- [H][#76](https://github.com/EchoYue-lp/echo-agent/issues/76) Pre-compaction memory 丢失混合来源 trust provenance。
- [M][#94](https://github.com/EchoYue-lp/echo-agent/issues/94) Skill candidate reinforcement 不写 audit。

## 已形成但尚未交付的候选

- [#84](https://github.com/EchoYue-lp/echo-agent/issues/84) Scheduler durable occurrence 候选 `573ee8b2`：复用 `DeliveryLedger` 实现 at-least-once claim、`OutcomeUnknown` replay、definition/control generation fencing；focused 29 项、两档 Clippy、demo70 与三轮复审通过。仍需 EKO stable data-root consumer 迁移、SDK 拆分吸收、全量门禁与集成语义归并。
- [#46](https://github.com/EchoYue-lp/echo-agent/issues/46) Trace/Audit failure visibility 候选 `080ec777`：已有有界异步 observer 与 FileAudit identity/lease/SyncData/torn-tail 修复，但当前审查结论为 BLOCK；仍缺 `Finalize` 持久化失败反例、公共 API 的 SDK inventory 分类、与 PR #124 的语义归并、完整验证证据和最终独立复审。
- [#36](https://github.com/EchoYue-lp/echo-agent/issues/36) Agent adapter close 的 ACP 子边界候选 `00b8428b`：close 前永久 fencing admission，并发取消 Run/extension、保留未结算 owner；focused 33 项、两档 Clippy 与复审通过。Finding 仍保持 open，Headless/A2A/Channel/ReactAgent Drop 要在 SDK 拆分后继续统一公共 lifecycle 合同。
- [#122](https://github.com/EchoYue-lp/echo-agent/issues/122) SDK repository extraction：独立线程正在迁移 `echo-sdk-host`/`echo-sdk-protocol`；必须从 continuity 权威分支吸收 `29cea08c` 的 Journal identity、scope 冻结与共享 catalog，并把 framework pin 推进到当前 `0e09324a`，不能从旧基线切换。#62 没有新增 SDK public identity 或 wire payload。

## 推荐推进顺序

1. 先完成 SDK repository extraction #122，再补齐 #46 的 `Finalize` 反例、SDK inventory 和语义归并；#84 先完成 EKO consumer 与 SDK 拆分依赖；#36 继续闭合 Headless/A2A/Channel/ReactAgent Drop，而不是只交付 ACP 子边界。
2. 继续闭合唯一终态和恢复权威：A2A、Channel driven Turn、Task/Subagent attempt、Scheduler durable occurrence、Transcript generation/projection。
3. 统一权限和扩展生命周期：Permission/Hook、Sandbox、Plugin/MCP/LSP generation 与 shutdown。
4. 收敛 Provider 和协议行为：stream terminal、structured output、A2A/Channel projection、SDK ACK replay watermark。
5. 最后处理 Evolution、长期记忆 provenance、动态 model facts 和中风险文档/示例漂移。

每个后续修复继续遵循一 Finding 一 Issue、一个 canonical owner、一个可回滚 repair slice，以及 repair、verification、independent rereview 三类关闭证据。

## 跨 Finding 架构与交付观察

- **Tool Surface / ToolRouter 草案仍被 review 阻塞，不得进入实现。** 下一版必须把 surface 冻结粒度从整个 Turn 改为每次 model request/iteration generation；复用现有 `ToolInvocation`、`snapshot::ToolRuntime`、`ToolExecutionContext`，明确 requested → mutable policy context → frozen admitted invocation；并把并发/副作用 traits 与动态 Permission decision、既有 ToolManager semaphore、network lifecycle 分离。该草案不计入95个Finding，也不是已接受ADR。
- **SDK 拆分必须吸收 `0e09324a`。** `echo-sdk-host`/`echo-sdk-protocol` 独立仓库线程应保留 Turn delivery、checkpoint claim settlement、Journal generation identity、gap generation、source contract 与最终五类 scope 冻结；#62 无 SDK payload，只需要 framework pin 前移。迁移完成前，新的治理切片避免无必要扩大这两个 crate。
- **CI Actions 有维护债。** PR #125 的 10 项 CI 全绿，但多个旧 Action major 仍存在 Node runtime/弃用升级提示；应以独立 Delivery MR 升级并重新验证，不混入运行时 Finding。
