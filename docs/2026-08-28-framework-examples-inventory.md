# Framework Examples Inventory (R2)

审计与实施日期：2026-08-28。原始 inventory 基线是
`echo-agent@302453b174086c3795dc026d16eeb668ecc66bed`；本轮在该基线上执行了 R2 的
source/manifest/contract 收敛，结果提交为
`echo-agent@65f7c1e8c7c7c70df9b30756c6b15ac84376b535`。website 同步仍属于 R3。

## 结论与边界

计划中的“64 个 examples”指根 package `echo_agent` 的 64 个 Cargo example target/source
文件，不能按文件名最高编号 `demo70`、manifest 声明数或整个 workspace 的 Rust 文件数理解。

| 计数对象 | 当前事实 | 边界说明 |
|---|---:|---|
| `echo_agent/examples/*.rs` | 43 files | framework teaching/composition 与 conditional examples。 |
| `echo_agent/tests/example_contracts/*.rs` | 21 files | 已从 root examples 物理迁移的 deterministic executable contracts。 |
| 根 package `[[example]]` | 33 targets | `demo19_guard` 显式要求 `content-guard`；`demo42_browser_mcp` 显式指向 `examples/demo42_playwright_mcp.rs`。 |
| 根 package 自动发现 target | 10 targets | `demo00`, `01`, `02`, `10`, `11`, `13`, `17`, `32`, `33`, `40`；显式 target 不再重复自动发现。 |
| 根 package Cargo example targets | 43 targets | 33 显式 + 10 自动发现。 |
| `example_contracts` integration target | 21 nested tests | 单一 feature-gated harness，测试函数逐项执行原场景。 |
| `examples/support/mod.rs` | 1 module | 被 `demo00`/`demo27` 等复用的 helper，不是 Cargo target，不计入 64。 |
| `echo-agent-examples` | 1 binary, 0 examples | 只有 `src/main.rs` facade consumer probe；manifest 仅依赖 `echo_agent`，不依赖 split crate。 |
| `echo-rust-learning/examples/*.rs` | 13 files | 独立、离线、非发布的 Rust 教学 consumer crate；不属于 R2 的 64 个 framework examples，但在本文末单独核对。 |
| split crates (`echo-core`, `echo-execution`, `echo-integration`, `echo-macros`, `echo-orchestration`, `echo-state`, `echo-tools`) | 0 example targets | 当前没有额外 Cargo examples。 |

### Disposition 计数

| disposition | count | 判定 |
|---|---:|---|
| `root-composition` | 29 | 11 个 teaching、15 个 feature composition，以及 3 个仍包含 live-provider 执行的 walkthrough。 |
| `contract-test` | 21 | 已物理迁入 `tests/example_contracts/`，由 integration harness 真实执行。 |
| `conditional` | 14 | 保留 framework API 价值，但依赖凭证、网络、系统 runtime、git 或本地服务；不得混入无条件 gate。 |
| `delete` | 0 | 当前没有足够证据安全删除 public API 覆盖；`demo28` 等看似重复项仍对应公开类型/文档 owner。 |

### 依赖与风险标记

所有根 examples 都以 `echo_agent::...` 访问 facade；没有直接写
`echo_core`、`echo_tools`、`echo_state` 等 split-crate dependency。表中“内部 owner”是 facade
re-export 背后的实现 crate，不是 example 的直接 Cargo 依赖。

风险标记：

- `P`：`unwrap`/`expect`/panic 类强制提取；括号内是当前行号。
- `U`：UTF-8 不安全的字节切片，或把可能含中文/emoji 的字符串按 bytes 计数/截断。
- `J`：直接 `serde_json::Value`/数组索引；当前通常返回 `Null`，但违反结构化输入禁令，迁移时应改为 checked access。
- `S`：SQLite/`SqliteStore` 或 SQLite feature；这是 framework 的合理公共能力，不因 EKO CLI 不启用 SQLite 而删除。
- `X`：外部凭证、网络、文件系统、git、Docker、Node/Playwright、Python 等运行前置条件。
- `F`：Cargo feature/prerequisite 或文档与源码不一致。
- `I`：facade 之外的内部路径/过深路径，需在 consumer cutover 时收敛。

R2 基线扫描曾发现 14 个 source、52 处 `unwrap/expect`，以及 `demo08`, `demo40`,
`demo61` 的字符串字节切片、若干 byte-count 和 JSON direct index。本轮已全部改为错误传播、
checked access、`chars().take/count` 或结构化解析，并新增 executable contract 阻止这些模式回归。
`demo08`/`demo31` 的无关 SQLite gate、`demo19` 的缺失 gate、`demo43` 的 feature gate、
`demo47` 的错误 SQLite 文案和 `demo70` 的深层 facade import 也已收敛。

21 个 deterministic scenarios 已物理迁入 `tests/example_contracts/`，旧 root paths 已删除且没有
wrapper/副本；15 个 composition scenarios 经依赖审计后保留 root，因为迁入唯一依赖
`echo_agent` 的 consumer crate 会迫使复制 Tokio/Serde/tempfile 等 demo runtime dependencies，破坏
facade-only probe；`demo13`/`demo32`/`demo35` 因仍包含 live-provider 入口也保留 root。
`examples/README.md` machine-check 29/21/14 disposition 和 64 个唯一场景。

## 64 个 framework scenarios

`Facade/internal` 列使用 `facade -> implementation owner`；`Docs/test owner` 给出长期事实源。
`contract-test` 条目已迁入 `tests/example_contracts/`，其它 source 仍在 root `examples/`。

| # | source / Cargo target | disposition | feature / prerequisite | Facade -> internal owner | 风险 | Docs / test owner |
|---:|---|---|---|---|---|---|
| 00 | `demo00_quickstart.rs` | `root-composition` | 无 feature；`ECHO_AGENT_PROVIDER/BASE_URL/API_PROTOCOL/MODEL/API_KEY`，真实 LLM | `agent!`, `tool` -> `echo_core` + `echo_macros` | `X` API key/network | `C:quickstart`; `D:getting-started`; `T:facade_smoke` |
| 01 | `demo01_tools.rs` | `root-composition` | 无 feature；provider config/API key，真实 LLM | `prelude`, `#[tool]` -> `echo_core` + `echo_macros` | `X` LLM；教学代码无确定性输出断言 | `D:02-tools`; `T:tool_call_contract` |
| 02 | `demo02_tasks.rs` | `root-composition` | 无 feature；`qwen3-max` provider/API key，真实 task tools | task tools/ReAct -> `echo_orchestration` + `echo_core` | `X` LLM；仅检查非空结果 | `D:09-tasks`; `T:task_tool_contract` |
| 03 | `demo03_approval.rs` | `conditional` | `human-loop`；LLM；自动 `HumanLoopManager` approval/input | human loop/permission -> `echo_orchestration` | `X` LLM + HITL；事件依赖 | `D:05-human-loop`; `T:approval_contract` |
| 04 | `tests/example_contracts/demo04_subagent.rs` | `contract-test` | `testing`；`MockLlmClient` + local human loop，无外部服务 | subagent -> `echo_orchestration` + `echo_core` | 无 `P/U/S`；可确定性化 | `D:06-subagent`; `T:subagent_contract` |
| 05 | `demo05_compressor.rs` | `root-composition` | `human-loop`；真实 LLM，审批工具自动响应 | compression -> `echo_state`; HITL -> `echo_orchestration` | `X` LLM/HITL | `D:04-compression`, `05-human-loop`; `T:compression_walkthrough` |
| 06 | `demo06_mcp.rs` | `conditional` | `mcp`；Node.js 18+/`npx @modelcontextprotocol/server-filesystem`；LLM key | MCP -> `echo_integration` | `X` npx/process/network/API key | `D:08-mcp`; `T:mcp_process_contract` |
| 07 | `demo07_skills.rs` | `conditional` | `files`；`/tmp` read/write；Part 3 需要 LLM | skills/files -> `echo_tools` + `echo_execution` | `X` filesystem + LLM | `D:07-skills`; `T:skill_contract` |
| 08 | `demo08_external_skills.rs` | `conditional` | file skills/scripts；LLM 与 `/tmp` fixtures；无关 `sqlite` gate 已删除 | external skills -> `echo_tools` + `echo_execution` | checked JSON + delimiter parsing + Unicode char count；`X` scripts/files/LLM | `D:07-skills`; `T:external_skill_contract` |
| 09 | `demo09_file_shell.rs` | `conditional` | `files,shell`；`/tmp` workspace、shell commands、LLM | files/shell -> `echo_tools` + `echo_execution` | `X` filesystem/shell/LLM | `D:21-common-tools`, `41-shell-text-tools`; `T:file_shell_contract` |
| 10 | `demo10_streaming.rs` | `root-composition` | 无 feature；真实 LLM streaming/provider config | streaming -> `echo_core` + root agent | `X` LLM/network | `D:10-streaming`; `T:stream_event_contract` |
| 11 | `demo11_callbacks.rs` | `root-composition` | 无 feature；真实 LLM，callback stream | callbacks -> `echo_core` | `X` LLM/network | `D:23-hooks`; `T:callback_contract` |
| 12 | `tests/example_contracts/demo12_resilience.rs` | `contract-test` | `testing`；`MockLlmClient`/flaky tool，离线 | retry/agent -> `echo_core` | 无 `P/U/S`；适合 deterministic test | `D:30-react-safety`; `T:resilience_contract` |
| 13 | `demo13_tool_execution.rs` | `root-composition` | manifest 无 feature；当前 `ReactAgent` 真实执行需 provider/API key，含 timeout/sleep；测试应注入 mock | tool execution -> `echo_core` | 当前无 `P/U/S`；`X` live LLM；需拆出 mock acceptance | `D:02-tools`; `T:tool_execution_contract` |
| 15 | `demo15_structured_output.rs` | `root-composition` | 无 feature；真实 LLM，JSON-mode/output schema provider | structured output -> root agent + `echo_core` | checked typed fields；`X` LLM/JSON mode | `D:11-structured-output`; `T:structured_output_contract` |
| 17 | `demo17_chat.rs` | `root-composition` | 无 feature；真实 LLM，多轮 chat/stream | chat -> root agent + `echo_core` | `X` LLM/network | `D:13-chat`; `T:chat_contract` |
| 18 | `demo18_semantic_memory.rs` | `conditional` | 无 feature；embedding endpoint + LLM；`InMemoryStore`，非 SQLite | memory/embedding -> `echo_state` + `echo_core` | `X` embedding/LLM；无 `S` | `D:14-semantic-search`; `T:semantic_memory_contract` |
| 19 | `demo19_guard.rs` | `root-composition` | manifest 显式 `content-guard`；真实 LLM | guard/audit -> `echo_core` | feature gate 已对齐；`X` LLM | `D:18-guard-system`; `T:guard_contract` |
| 20 | `demo20_audit.rs` | `root-composition` | `human-loop`；自动审批 provider + 真实 LLM | audit/permission -> `echo_core` + `echo_orchestration` | `X` LLM/HITL；无 `P/U/S` | `D:05-human-loop`; `T:audit_contract` |
| 23 | `demo23_a2a.rs` | `root-composition` | `a2a`；构造 local server/client JSON-RPC；无外部服务但协议边界公开 | A2A -> root `a2a` + `echo_integration` | serialization errors propagated；`X` URL/server semantics | `C:a2a_probe`; `D:README`; `T:a2a_contract` |
| 24 | `tests/example_contracts/demo24_topology.rs` | `contract-test` | `topology`；本地 tracker/callback，无 LLM call | topology -> root + `echo_core` | JSON export error propagated | `D:README`; `T:topology_contract` |
| 25 | `demo25_macros.rs` | `root-composition` | 无 feature；宏 facade compile + 可选真实 LLM/API key | proc macros -> `echo_macros` + `echo_core` | serialization propagated + Unicode guard count；`X` API key | `C:macro_probe`; `T:macro_facade` |
| 26 | `demo26_provider_factory.rs` | `root-composition` | 无 feature；只构造 provider/model client，使用 placeholder endpoint/key，不发请求 | LLM config/factory -> `echo_core` | `X` endpoint/key are placeholders；无 `P/U/S` | `C:provider_factory_probe`; `D:38-factory-modes`; `T:provider_config_contract` |
| 27 | `demo27_sqlite_memory.rs` | `conditional` | `sqlite`；bundled SQLite/FTS5；可选 embedding + LLM；临时 DB/WAL/SHM | memory -> `echo_state`; SQLite facade -> `echo_state` | `S`; Option/JSON checked access；`X` embedding/LLM/files | `D:41-persistence-concepts`, `14-semantic-search`; `T:sqlite_store_contract` |
| 28 | `demo28_workflow.rs` | `root-composition` | 无 feature；Sequential/Concurrent/DAG 三种公开 workflow 都运行真实 LLM | workflow -> `echo_orchestration` | `X` 多个 LLM；API 是公开且仍有 docs owner，非 delete | `D:17-graph-workflow`; `T:workflow_legacy_shapes` |
| 29 | `demo29_sandbox.rs` | `conditional` | 无 feature metadata；Local 必需，Docker/K8s/`curl`/`python3` 能力按环境可选 | sandbox -> `echo_execution` | `X` Docker/OS/process/network；不可把 Docker skip 当全绿 | `D:30-react-safety`; `T:sandbox_contract` |
| 30 | `tests/example_contracts/demo30_mcp_server.rs` | `contract-test` | `mcp`；in-process JSON-RPC server，协议流程可离线验证 | MCP server -> `echo_integration` | JSON Pointer typed helpers，响应结构 fail-closed | `D:08-mcp`; `T:mcp_json_rpc_contract` |
| 31 | `tests/example_contracts/demo31_memory_tools.rs` | `contract-test` | no-feature；只用 `InMemoryStore`，不构造 SQLite，不执行 LLM | memory tools -> `echo_state` + root agent | 无关 sqlite required-feature 已删除 | `D:14-semantic-search`; `T:memory_tools_contract` |
| 32 | `demo32_token_budget.rs` | `root-composition` | 无 feature；前半可 mock，Agent 执行当前需 LLM/provider | token budget/tool result -> `echo_core` | build errors propagated；saturating arithmetic + char count；`X` live LLM | `D:02-tools`; `T:token_budget_contract` |
| 33 | `demo33_retry_policy.rs` | `conditional` | 无 feature；local policy 部分可离线，LLM/MCP retry 部分需服务 | retry -> `echo_core` | expected success/failure 显式验收；`X` LLM/MCP/network | `D:30-react-safety`; `T:retry_policy_contract` |
| 34 | `tests/example_contracts/demo34_workflow_stream.rs` | `contract-test` | 无 feature；function-node stream 全部本地 | workflow stream -> `echo_orchestration` | Unicode word length | `D:17-graph-workflow`; `T:workflow_stream_contract` |
| 35 | `demo35_dynamic_tools.rs` | `root-composition` | 无 feature；API edge cases 离线，Part 3 当前真实 LLM；应拆出前两部分 contract | dynamic tools -> root agent + `echo_core` | `I` uses deep `agent::react::ReactAgent`; `X` Part 3 LLM；断言可迁移 | `D:02-tools`; `T:dynamic_tool_contract` |
| 36 | `demo36_multimodal.rs` | `conditional` | 无 feature metadata；YAML model config、vision-capable provider、remote image URLs/API key | multimodal LLM -> root agent + `echo_core` | `X` provider/remote images/model capability | `D:11-structured-output`; `T:multimodal_contract` |
| 37 | `tests/example_contracts/demo37_declarative_workflow.rs` | `contract-test` | 无 feature；YAML/JSON parse/build/invalid-input assertions 离线 | workflow loader -> `echo_orchestration` | 无 `P/U/S`；强 deterministic acceptance | `D:17-graph-workflow`; `T:declarative_workflow_contract` |
| 38 | `demo38_im_channels.rs` | `root-composition` | `channels`；Feishu/QQ credentials、网络长轮询、LLM client | channels -> `echo_integration` | `X` credentials/network/long poll；consumer composition boundary | `C:channel_probe`; `D:15-im-channels`; `T:channel_composition_contract` |
| 39 | `tests/example_contracts/demo39_workflow.rs` | `contract-test` | `testing`；MockAgent/function graph，离线 | Graph/SharedState -> `echo_orchestration` | typed required-state helper + Unicode counts | `D:17-graph-workflow`; `T:graph_state_contract` |
| 40 | `demo40_snapshot.rs` | `root-composition` | 无 feature；真实 LLM，多轮快照/rollback | snapshot -> root agent + `echo_core` | snapshot IDs use `chars().take(8)`；`X` LLM | `D:41-persistence-concepts`; `T:snapshot_contract` |
| 41 | `demo41_web_tools.rs` | `conditional` | `web`；DuckDuckGo/Brave/Tavily/network keys；web fetch | web tools -> `echo_tools` | Unicode output count；`X` network/API keys | `D:20-web-tools`; `T:web_tool_contract` |
| 42 | `demo42_playwright_mcp.rs` (target `demo42_browser_mcp`) | `conditional` | `mcp`；`mcp.json` + Node/Playwright MCP + browser + LLM config | MCP -> `echo_integration` | `X` browser/npx/config/network；UTF-8 previews use `chars().take` safely | `D:08-mcp`; `T:playwright_mcp_contract` |
| 43 | `tests/example_contracts/demo43_data_tools.rs` | `contract-test` | harness `testing,data,media`；MockLlm + TempDir fixtures | data tools -> `echo_tools` | feature gate 对齐；serialization/base64 errors propagated | `D:42-database-tools`, `43-data-output-format`; `T:data_tools_contract` |
| 44 | `demo44_code_laboratory.rs` | `root-composition` | `content-guard`；Python 3 + local sandbox + LLM；代码执行场景 | guard -> `echo_core`; sandbox/tools -> `echo_execution` + `echo_tools` | `X` python/process/LLM；scenario should be consumer-owned | `C:code_lab`; `D:18-guard-system`; `T:code_lab_contract` |
| 45 | `demo45_customer_service.rs` | `root-composition` | `sqlite,human-loop,content-guard`；SQLite DB + LLM + approval | memory -> `echo_state`; HITL -> `echo_orchestration`; guard -> `echo_core` | `S`; checked memory content；`X` LLM/HITL/files | `C:customer_service`; `D:05-human-loop`, `41-persistence-concepts`; `T:consumer_scenario_smoke` |
| 46 | `demo46_data_analyst.rs` | `root-composition` | `sqlite`；SQLite + embedding + LLM + generated data files | memory -> `echo_state`; workflow -> `echo_orchestration` | `S`; checked JSON + schema serialization；`X` embedding/LLM/files | `C:data_analyst`; `D:14-semantic-search`; `T:consumer_scenario_smoke` |
| 47 | `demo47_enterprise.rs` | `root-composition` | `testing`；`examples/demo_skills` fixture + live LLM | skills/tools/workflow/topology -> `echo_tools` + `echo_orchestration` + root | fixture fail-fast；Unicode code count；错误 SQLite 文案已删除；`X` fixture/LLM | `C:enterprise_scenario`; `D:07-skills`, `17-graph-workflow`; `T:consumer_scenario_smoke` |
| 48 | `demo48_personal_assistant.rs` | `root-composition` | `sqlite,subagent`；SQLite + subagent + LLM；multimodal model optional | memory -> `echo_state`; subagent -> `echo_orchestration` | `S`; checked profile/conversation fields；`X` LLM/subagent/model | `C:personal_assistant`; `D:06-subagent`, `41-persistence-concepts`; `T:consumer_scenario_smoke` |
| 49 | `demo49_research_agent.rs` | `root-composition` | `sqlite,web,files`；SQLite + web + filesystem + LLM + embedding | memory -> `echo_state`; web/files -> `echo_tools`; workflow -> `echo_orchestration` | `S`; checked JSON + schema serialization；`X` network/files/LLM | `C:research_agent`; `D:20-web-tools`, `22-research-tools`; `T:consumer_scenario_smoke` |
| 50 | `tests/example_contracts/demo50_eval.rs` | `contract-test` | `eval`；local trace/eval/replay/HTML generation，no LLM required | eval/trace -> root package | checked TempDir report persistence | `D:24-eval-system`; `T:eval_contract` |
| 51 | `tests/example_contracts/demo51_self_improvement.rs` | `contract-test` | `eval,improve`；local synthetic runs/curator/trajectory | eval/improve/evolution -> root package | TempDir curator/trajectory + deterministic assertions | `D:25-self-improvement`; `T:self_improvement_contract` |
| 53 | `tests/example_contracts/demo53_adaptive_compression.rs` | `contract-test` | 无 feature；local heuristic compression；L4 only optional `.with_llm()` | compression -> `echo_state` | compression errors propagated；saturating arithmetic + char count | `D:04-compression`; `T:adaptive_compression_contract` |
| 54 | `tests/example_contracts/demo54_headless.rs` | `contract-test` | 无 feature；main is local config/result formatting; live LLM only documented optional path | headless -> root package | JSON parse propagated + checked fields | `D:33-headless-mode`; `T:headless_contract` |
| 55 | `tests/example_contracts/demo55_lsp_tools.rs` | `contract-test` | `lsp`；YAML/config/tool construction local；actual language-server binaries optional and not started | LSP -> `echo_integration`; tools -> `echo_tools` | YAML/schema/Option errors propagated；optional binaries `X` only for future live path | `D:31-lsp-integration`; `T:lsp_config_contract` |
| 56 | `demo56_plugin_system.rs` | `root-composition` | 无 feature；tempdir plugin manifest/skill/hook lifecycle，无 LLM | plugin/skills -> root package + `echo_tools` | 无 `P/U/S`；适合作 facade consumer lifecycle probe | `C:plugin_probe`; `D:32-plugin-system`; `T:plugin_lifecycle_contract` |
| 57 | `tests/example_contracts/demo57_data_pipeline.rs` | `contract-test` | `testing`；MockAgent + config/state contract，离线，无文件写入 | pipeline -> `echo_orchestration` | 无 `P/U/S`；可作为 deterministic pipeline contract | `D:35-pipelines`; `T:data_pipeline_contract` |
| 58 | `demo58_git_worktree.rs` | `conditional` | `git`；当前 checkout 必须是 git repo；创建/删除 worktree 和 branch | git isolation -> `echo_tools` | `X` mutates repo/worktree and invokes git; never default CI without isolated temp repo | `D:34-git-isolation`; `T:git_worktree_contract` |
| 59 | `demo59_code_search.rs` | `conditional` | `files`；读取 repository，优先 `rg`，缺失时内置 fallback | code search -> `echo_tools` | `X` filesystem/`rg`/repository shape；结果依赖 checkout | `D:37-code-search`; `T:code_search_contract` |
| 60 | `tests/example_contracts/demo60_data_quality.rs` | `contract-test` | `data,statistics`；生成 `/tmp/echo_demo60_data.csv` 后运行工具；应改 tempdir 避免固定路径 | data/statistics -> `echo_tools` | `X` fixed `/tmp` path and cleanup; no `P/U/S`; fixture-based test owner | `D:36-data-quality-statistics`; `T:data_quality_contract` |
| 61 | `demo61_agent_factory.rs` | `root-composition` | `testing`；factory + MockLlm local，第一段只是 config print | factory -> root package + `echo_core` | prompt preview uses `chars().take(60)` | `C:factory_probe`; `D:38-factory-modes`; `T:agent_factory_contract` |
| 62 | `tests/example_contracts/demo62_prompt_templates.rs` | `contract-test` | 无 feature；template substitution/default/conditional/thread-safe local | prompt templates -> `echo_core` | 无 `P/U/S`；25 assertions，直接 test owner | `D:40-context-system`; `T:prompt_template_contract` |
| 64 | `tests/example_contracts/demo64_tool_pipeline.rs` | `contract-test` | 无 feature；MockLlm + intervention/callback/config pipeline | tool pipeline -> `echo_core` | exact add result and callback start/end assertions | `D:02-tools`, `23-hooks`; `T:tool_pipeline_contract` |
| 65 | `tests/example_contracts/demo65_context_assembler.rs` | `contract-test` | 无 feature；local message assembly/budget | context -> root package + `echo_core` | token estimate uses Unicode char count | `D:40-context-system`; `T:context_assembler_contract` |
| 66 | `tests/example_contracts/demo66_context_selector.rs` | `contract-test` | 无 feature；synthetic paths/symbols, no actual file reads; deterministic scoring | context selector -> root package | path classification uses `to_string_lossy` helper | `D:40-context-system`; `T:context_selector_contract` |
| 67 | `tests/example_contracts/demo67_progress.rs` | `contract-test` | 无 feature；local watch/bus/timer + typed extension | tasks/progress -> `echo_orchestration` | 无 `P/U/S`；timing should use bounded deterministic test clock where possible | `D:09-tasks`; `T:progress_contract` |
| 68 | `demo68_human_gate.rs` | `root-composition` | `subagent,human-loop`；local `HumanLoopManager` selection/approval + timeout | HITL -> `echo_orchestration` | 无 `P/U/S`；public consumer selection probe | `C:human_selection_probe`; `D:05-human-loop`; `T:human_selection_contract` |
| 70 | `demo70_scheduler.rs` | `root-composition` | 无 feature；temp file-backed cron store, local `SchedulerRunner` | scheduler -> `echo_orchestration` | stable `echo_agent::scheduler` facade；`X` temp file | `C:scheduler_probe`; `D:29-long-running-tasks`; `T:scheduler_contract` |

## Consumer crates

### `echo-agent-examples`

`echo-agent-examples/Cargo.toml` 是 `publish = false` 的独立 package，唯一 dependency 是
`echo_agent = { path = "..", default-features = false }`。`src/main.rs` 构造
`FrameworkConfig`、`DataRoot`、`StandardToolPack` 和 `AgentInvocationContext`，验证 resource
guard identity；它没有 split-crate import、SQLite、外部服务、panic API 或 UTF-8 byte slice。
这是 R2 的 facade consumer gate，不是第二套 examples owner。原 15 个 `move-consumer` 候选经
复核后保留为 root composition examples：它们需要 Tokio、Serde、tempfile 或 feature-specific
demo runtime，搬入此 package 会迫使增加直接依赖并破坏 facade-only invariant。

Owner：`echo-agent-examples/src/main.rs`（consumer composition）、framework
`tests/facade_smoke.rs`（facade contract）；不要把 EKO workspace/UI/DomainProfile 概念带入。

### `echo-rust-learning`

`echo-rust-learning` 是非发布教学 crate，manifest 直接依赖 `echo_agent` 以及教学所需的
`futures/serde/serde_json/thiserror/tokio`。它不是 framework facade acceptance owner，13 个
example 应留在本 crate，文档事实源是 `echo-rust-learning/README.md` 和
`echo-rust-learning/docs/zh/`；不要迁回根 `examples/`，也不要把它计入计划的 64。

| teaching example | 内容 / 前置条件 | disposition / owner | 风险结论 |
|---|---|---|---|
| `chapter_01_basics.rs` | 基础类型、控制流、UTF-8 | keep in `echo-rust-learning`; `L:docs/zh/02-language-basics.md` | UTF-8 helper 使用 `chars().take/count`，无真实 `P/U`。 |
| `chapter_03_domain_modeling.rs` | struct/enum/错误 | keep; `L:03-domain-modeling-pattern-matching.md` | `Result` 返回，无 panic API。 |
| `chapter_04_ownership.rs` | move/borrow/lifetime | keep; `L:04-ownership-borrowing-lifetimes.md` | character count 走 Unicode-safe helper。 |
| `chapter_05_collections_iterators.rs` | Vec/Map/Set/迭代器 | keep; `L:05-collections-closures-iterators.md` | 数组 literals/signatures 非直接索引风险。 |
| `chapter_06_errors.rs` | Option/Result/error chain | keep; `L:06-option-result-errors.md` | 结构化错误，无 `unwrap/expect/panic`。 |
| `chapter_07_traits_generics.rs` | trait/generic/builder | keep; `L:07-traits-generics-macros.md` | 无禁用 API。 |
| `chapter_08_box.rs` | Box/递归类型/Deref/Drop | keep; `L:08-smart-pointers-foundations.md` | 无禁用 API。 |
| `chapter_09_arc_weak.rs` | Arc/Weak/registry | keep; `L:09-shared-ownership.md` | 无禁用 API。 |
| `chapter_10_rc_refcell.rs` | Rc/RefCell | keep; `L:10-interior-mutability.md` | 错误通过 `Result`，无强制提取。 |
| `chapter_11_pin_future.rs` | Cow/Pin/Future | keep; `L:11-cow-pin-futures.md` | async main 无 panic API。 |
| `chapter_12_async_concurrency.rs` | Tokio/channel/timeout/cancel | keep; `L:12-async-concurrency-streams.md` | 无禁用 API，`run_subagents` 是教学函数名，不是产品 worker 术语。 |
| `chapter_13_serde.rs` | Serde JSON round-trip/validation | keep; `L:13-serde-and-configuration.md` | checked parse/validation，无 JSON direct index。 |
| `chapter_15_echo_agent_tool.rs` | 离线真实 `#[tool]` | keep; `L:15-reading-echo-agent.md`; `T:learning_contract` | 只依赖 facade `echo_agent::prelude::Result`，无外部 LLM/SQLite。 |

## R2 实施结论与 residual

已完成：

1. `Cargo.toml` feature/prerequisite drift 已修复；43 个 Cargo examples 与 21 个 nested contract
   tests 共同覆盖 64 个唯一 scenario。
2. framework scenarios 的 `.unwrap()`/`.expect()`/panic macro、unchecked range slicing、已知 JSON direct
   index 与文本 byte-count 已清零；错误改为传播或带上下文的 fail-closed 结果。
3. `examples/README.md` 逐项记录 29/21/14 disposition，`documentation_contract` 会校验 64 个
   scenario 恰好分类一次、feature gate、facade-only import、Subagent 术语和禁用模式。
4. `demo70` 只走稳定 root facade；SQLite examples 保留为 framework 能力，不影响 EKO CLI 的
   no-SQLite 产品决策。
5. root README/README.zh 与 14 份正式 framework 文档已改到 `tests/example_contracts/*`，链接合同
   不再按 filename fallback 掩盖 stale path。

21 个 `contract-test` 已完成物理归位且旧 source 删除，无 wrapper/副本。`demo13`、`demo32`、
`demo35` 经复核保留 root：其 entrypoint 仍包含 live-provider execution，不能冒充 deterministic
contract。14 个 `conditional` 外部运行时场景仍需 Final Gate 的环境化验收。

## Verification evidence

以下命令均在最终 implementation tree 上真实 exit 0：

```text
cargo metadata --no-deps --format-version 1
cargo test -p echo_agent --test documentation_contract --locked
cargo test -p echo_agent --test example_contracts --all-features --locked -- --test-threads=1
cargo check -p echo_agent --examples --all-features --locked
cargo check -p echo-agent-examples --locked
cargo clippy -p echo_agent --examples --all-features --locked -- -D warnings
cargo clippy -p echo_agent --test example_contracts --all-features --locked -- -D warnings
cargo fmt --all -- --check
```

最终计数为 documentation contracts `6/6`、executable example contracts `21/21`、root Cargo
examples `43`、contract scenarios `21`。独立 feature matrix
`sqlite/subagent/human-loop/mcp/lsp/a2a/git/database/rag/chart/web/media` 全部 exit 0；static
disposition、禁用 panic/UTF-8/JSON-index/split-crate/worker 术语扫描为零违规。

长时/规模/外部服务 examples 不在 R2 重复运行，继续归整个项目 Final Gate；所有实际执行命令和
exit status 必须在交付时如实报告。
