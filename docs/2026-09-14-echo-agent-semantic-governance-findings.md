# echo-agent 全 Workspace 语义治理优化清单

> 快照：2026-09-14，`echo-agent@0878a676`。本报告是本轮跨仓工程总结；长期行为、状态权威和 Finding 事实仍以 `echo-agent/.echo-semantic/` 为准。

## 总体结论

本轮将项目治理单位从 9,684 个 Rust public identity 收敛为 Capability、Behavior、Rule、状态权威、生命周期、Finding 和 Evidence。全仓共形成 94 个可追踪 Finding，其中 23 个已完成修复、验证、独立复审和远端交付，71 个仍保持 open；94 个 Finding 均映射唯一 GitHub Issue，远端状态为 23 CLOSED / 71 OPEN。

SDK identity inventory 继续用于 API 漂移监控，但不再表示项目完成度。当前 scope 是 5,607 external contract、1,765 Host/Rust-only、781 language intrinsic、90 internal helper、1,441 deferred；后续只按可交付 capability 推进 deferred backlog。

| 语义边界 | Finding | 已修复 | 待优化 |
| --- | ---: | ---: | ---: |
| Workspace 架构与公共组合 | 4 | 4 | 0 |
| Agent / Session / Turn | 2 | 0 | 2 |
| Context 与 Memory | 3 | 0 | 3 |
| Task / Subagent / Workflow | 16 | 4 | 12 |
| Observation / Persistence / Delivery | 6 | 0 | 6 |
| Tool / Permission / Sandbox | 16 | 4 | 12 |
| Extension / MCP / LSP / Plugin | 14 | 0 | 14 |
| LLM / Provider | 8 | 0 | 8 |
| Protocol / A2A / Channel / SDK | 8 | 1 | 7 |
| SDK facade parity | 5 | 5 | 0 |
| Eval / Improve / Evolution | 12 | 5 | 7 |
| **合计** | **94** | **23** | **71** |

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

### Tool / Cache / Registry

- [#26](https://github.com/EchoYue-lp/echo-agent/issues/26) Streaming Tool 跳过统一参数校验：已在 cache、permit 和 effect 前复用同一 validation kernel。
- [#30](https://github.com/EchoYue-lp/echo-agent/issues/30) Tool read cache 缺 workspace identity：cache key 已纳入 effective workspace 与 invocation lineage。
- [#28](https://github.com/EchoYue-lp/echo-agent/issues/28) In-flight Read 可在 Write 后复活陈旧 cache：已使用 write-lifetime epoch CAS 阻止旧值发布。
- [#115](https://github.com/EchoYue-lp/echo-agent/issues/115) Tool registry mutation 被跨 await guard 阻塞：执行路径改为 owned handle，保留 generation/freshness fence。

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
- [#116](https://github.com/EchoYue-lp/echo-agent/issues/116) SDK backlog 混用 4,076 intrinsic 旧口径：已统一为 1,441 deferred identity 的 capability backlog，其他 scope 不属于语言 parity backlog。

## 待优化清单

### Agent / Session / Turn

- [H][#36](https://github.com/EchoYue-lp/echo-agent/issues/36) Agent adapter close 与资源结算 owner 未闭合。
- [H][#107](https://github.com/EchoYue-lp/echo-agent/issues/107) Channel adapter 绕过统一 driven Turn authority。

### Context 与 Memory

- [M][#42](https://github.com/EchoYue-lp/echo-agent/issues/42) `AgentCheckpoint.current_plan` 是孤立恢复权威，尚未和 canonical Task artifact 对齐。
- [H][#105](https://github.com/EchoYue-lp/echo-agent/issues/105) Runtime state 与 transcript generation identity 可写出不可恢复 checkpoint。
- [H][#106](https://github.com/EchoYue-lp/echo-agent/issues/106) Transcript projection 写入失败缺少结算合同。

### Task / Subagent / Workflow

- [H][#44](https://github.com/EchoYue-lp/echo-agent/issues/44) CommandCell 普通 cancel 可卡在 artifact finalization。
- [H][#45](https://github.com/EchoYue-lp/echo-agent/issues/45) CommandCell retention prune 可删除并发新 lease。
- [H][#84](https://github.com/EchoYue-lp/echo-agent/issues/84) Scheduler store、cache 与 callback delivery 未闭合。
- [H][#85](https://github.com/EchoYue-lp/echo-agent/issues/85) Scheduler disable/remove 与已捕获 callback 不线性一致。
- [H][#86](https://github.com/EchoYue-lp/echo-agent/issues/86) Scheduler 不保证 CronTask ID 唯一。
- [M][#98](https://github.com/EchoYue-lp/echo-agent/issues/98) Definition-only Subagent catalog 合同冲突。
- [H][#99](https://github.com/EchoYue-lp/echo-agent/issues/99) TaskClaim 与 SubagentAttempt identity 未闭合。
- [H][#109](https://github.com/EchoYue-lp/echo-agent/issues/109) Workflow checkpoint claim 缺少失败恢复。
- [H][#110](https://github.com/EchoYue-lp/echo-agent/issues/110) Workflow tag 可复活已领取 checkpoint。
- [M][#111](https://github.com/EchoYue-lp/echo-agent/issues/111) Task DAG 与 Workflow DAG 仍是平行实现候选。
- [M][#112](https://github.com/EchoYue-lp/echo-agent/issues/112) Workflow 多入口主循环已发生事件漂移。
- [H][#113](https://github.com/EchoYue-lp/echo-agent/issues/113) Workflow 并行 sibling 可遮蔽失败并脱离继续执行。

### Observation / Persistence / Delivery

- [H][#43](https://github.com/EchoYue-lp/echo-agent/issues/43) Checkpoint 未绑定来源 Journal identity。
- [H][#46](https://github.com/EchoYue-lp/echo-agent/issues/46) Trace 与 Audit 持久化失败缺少统一可见结果。
- [M][#58](https://github.com/EchoYue-lp/echo-agent/issues/58) HookEvent catalog 与自动 producer 不一致。
- [M][#61](https://github.com/EchoYue-lp/echo-agent/issues/61) InMemoryAuditLogger 丢写仍返回成功。
- [H][#103](https://github.com/EchoYue-lp/echo-agent/issues/103) Trace 与 audit 没有统一 secret retention 合同。
- [H][#108](https://github.com/EchoYue-lp/echo-agent/issues/108) Turn terminal commit 与 projection 顺序可形成冲突终态。

### Tool / Permission / Sandbox

- [H][#37](https://github.com/EchoYue-lp/echo-agent/issues/37) PermissionService 与 Shell CommandPolicy 存在双重 approval authority。
- [H][#47](https://github.com/EchoYue-lp/echo-agent/issues/47) Artifact、Sandbox 与 Worktree cleanup owner 未闭合。
- [M][#57](https://github.com/EchoYue-lp/echo-agent/issues/57) Guard ToolInput/ToolOutput 与生产可达性错位。
- [H][#60](https://github.com/EchoYue-lp/echo-agent/issues/60) Hook Allow 可绕过 protected-path decision。
- [H][#62](https://github.com/EchoYue-lp/echo-agent/issues/62) K8s Sandbox Pod 清理没有可靠 owner settlement。
- [H][#70](https://github.com/EchoYue-lp/echo-agent/issues/70) Plan mode 未形成可靠只读 surface。
- [H][#81](https://github.com/EchoYue-lp/echo-agent/issues/81) `readonly_tools` 不约束 custom Write/Execute Tool。
- [M][#82](https://github.com/EchoYue-lp/echo-agent/issues/82) SandboxManager 建流失败丢失 typed Failed 终态。
- [H][#83](https://github.com/EchoYue-lp/echo-agent/issues/83) Sandbox minimum isolation 可被 fallback 降级。
- [M][#101](https://github.com/EchoYue-lp/echo-agent/issues/101) demo64 Tool pipeline 合同与生产顺序漂移。
- [H][#102](https://github.com/EchoYue-lp/echo-agent/issues/102) Tool caller、trace 与 audit 可记录不同终态。
- [M][#104](https://github.com/EchoYue-lp/echo-agent/issues/104) Trace Permission/File/Test 事件缺少生产点。

### Extension / MCP / LSP / Plugin

- [H][#55](https://github.com/EchoYue-lp/echo-agent/issues/55) MCP SSE 与 SDK LSP cleanup 未等待结算。
- [H][#56](https://github.com/EchoYue-lp/echo-agent/issues/56) Extension credential 配置缺少统一 Debug/redaction 合同。
- [H][#59](https://github.com/EchoYue-lp/echo-agent/issues/59) Hook source order 可绕过 deny-first 归约。
- [H][#63](https://github.com/EchoYue-lp/echo-agent/issues/63) 派生 LSP client handle 可在 manager 关闭后复活进程。
- [H][#64](https://github.com/EchoYue-lp/echo-agent/issues/64) LSP runtime status 与 restart 字段未闭合。
- [H][#65](https://github.com/EchoYue-lp/echo-agent/issues/65) MCP client 广告未实现 capability。
- [H][#66](https://github.com/EchoYue-lp/echo-agent/issues/66) MCP server annotation 被当成自动 Tool 权限与副作用事实。
- [M][#67](https://github.com/EchoYue-lp/echo-agent/issues/67) MCP 协议版本文档漂移。
- [M][#71](https://github.com/EchoYue-lp/echo-agent/issues/71) Plugin component isolation 与 atomic generation 冲突。
- [H][#72](https://github.com/EchoYue-lp/echo-agent/issues/72) Plugin wiring 缺 active generation authority。
- [H][#73](https://github.com/EchoYue-lp/echo-agent/issues/73) Plugin Registry、wiring 与 callback lifecycle 未统一编排。
- [M][#74](https://github.com/EchoYue-lp/echo-agent/issues/74) Plugin lifecycle reconcile 可形成两代资源重叠。
- [H][#75](https://github.com/EchoYue-lp/echo-agent/issues/75) Plugin MCP server 名缺少 owner 隔离。
- [H][#93](https://github.com/EchoYue-lp/echo-agent/issues/93) Skill activation 存在两个状态权威。

### LLM / Provider

- [M][#68](https://github.com/EchoYue-lp/echo-agent/issues/68) 内置动态 Model facts 缺 freshness authority。
- [H][#69](https://github.com/EchoYue-lp/echo-agent/issues/69) Non-stream LLM cancellation 与 stream 路径不对等。
- [H][#77](https://github.com/EchoYue-lp/echo-agent/issues/77) Provider capabilities 与 ModelProfile authority 未闭合。
- [H][#78](https://github.com/EchoYue-lp/echo-agent/issues/78) Provider stream semantic terminal 不对等。
- [H][#95](https://github.com/EchoYue-lp/echo-agent/issues/95) SSE EOF 接受缺事件边界的剩余 JSON。
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
- [H][#88](https://github.com/EchoYue-lp/echo-agent/issues/88) 三语言 SDK gap generation 校验不对等。

### Eval / Evolution / Memory Improvement

- [M][#38](https://github.com/EchoYue-lp/echo-agent/issues/38) Background Review 丢 handle 后持久化无人结算。
- [H][#51](https://github.com/EchoYue-lp/echo-agent/issues/51) Evolution mutation 与 change audit 非原子。
- [H][#52](https://github.com/EchoYue-lp/echo-agent/issues/52) Evolution ChangeLog 不能作为 later rollback authority。
- [M][#53](https://github.com/EchoYue-lp/echo-agent/issues/53) Evolution 文档 namespace 与代码漂移。
- [H][#54](https://github.com/EchoYue-lp/echo-agent/issues/54) Skill Curator promotion 缺可验证批准与 audit authority。
- [H][#76](https://github.com/EchoYue-lp/echo-agent/issues/76) Pre-compaction memory 丢失混合来源 trust provenance。
- [M][#94](https://github.com/EchoYue-lp/echo-agent/issues/94) Skill candidate reinforcement 不写 audit。

## 推荐推进顺序

1. 先闭合唯一终态和恢复权威：Turn/A2A、Task/Subagent/Workflow、Scheduler、CommandCell。
2. 再闭合持久事实与副作用结算：Journal/Checkpoint、Transcript、Delivery、Tool/Audit、cleanup owner。
3. 统一权限和扩展生命周期：Permission/Hook、Sandbox、Plugin/MCP/LSP generation 与 shutdown。
4. 收敛 Provider 和协议行为：取消、stream terminal、SSE framing、structured output、SDK generation parity。
5. 最后处理 Evolution、长期记忆 provenance、动态 model facts 和中风险文档/示例漂移。

每个后续修复继续遵循一 Finding 一 Issue、一个 canonical owner、一个可回滚 repair slice，以及 repair、verification、independent rereview 三类关闭证据。
