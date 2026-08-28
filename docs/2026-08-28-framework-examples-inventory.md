# Framework Examples Inventory (R2)

审计日期：2026-08-28。事实源是独立 integration checkout
`/Users/ls/.codex/worktrees/f24e/integration/echo-agent` 的 detached `HEAD`
`302453b174086c3795dc026d16eeb668ecc66bed`（`302453b test(tasks): cover task list cursor pagination`）。
本文是只读 inventory；本次没有重组 examples、修改 Cargo manifest、修改 website 或改变 framework
代码。

## 结论与边界

计划中的“64 个 examples”指根 package `echo_agent` 的 64 个 Cargo example target/source
文件，不能按文件名最高编号 `demo70`、manifest 声明数或整个 workspace 的 Rust 文件数理解。

| 计数对象 | 当前事实 | 边界说明 |
|---|---:|---|
| `echo_agent/examples/*.rs` | 64 files | R2 的 64 个逐文件 disposition 对象。 |
| 根 package `[[example]]` | 53 targets | 其中 `demo42_browser_mcp` 显式指向 `examples/demo42_playwright_mcp.rs`。 |
| 根 package 自动发现 target | 11 targets | `demo00`, `01`, `02`, `10`, `11`, `13`, `17`, `19`, `32`, `33`, `40`；`demo42_playwright_mcp.rs` 已被显式 alias 占用，不再生成同名自动 target。 |
| 根 package Cargo example targets | 64 targets | 53 显式 + 11 自动发现，正好覆盖 64 个 source files。 |
| `examples/support/mod.rs` | 1 module | 被 `demo00`/`demo27` 等复用的 helper，不是 Cargo target，不计入 64。 |
| `echo-agent-examples` | 1 binary, 0 examples | 只有 `src/main.rs` facade consumer probe；manifest 仅依赖 `echo_agent`，不依赖 split crate。 |
| `echo-rust-learning/examples/*.rs` | 13 files | 独立、离线、非发布的 Rust 教学 consumer crate；不属于 R2 的 64 个 framework examples，但在本文末单独核对。 |
| split crates (`echo-core`, `echo-execution`, `echo-integration`, `echo-macros`, `echo-orchestration`, `echo-state`, `echo-tools`) | 0 example targets | 当前没有额外 Cargo examples。 |

### Disposition 计数

| disposition | count | 判定 |
|---|---:|---|
| `keep-root` | 11 | 保留为 framework teaching walkthrough；可有运行时前置条件，但不是默认 acceptance gate。 |
| `move-consumer` | 15 | 场景组合或外部 facade probe；目标 owner 是 `echo-agent-examples`，保持只依赖 `echo_agent`。 |
| `move-test` | 24 | 可通过 mock/fixture 变为确定性 contract，迁到 `tests/`；若保留为强验收 example，仍需同等 fail-fast 断言。 |
| `conditional` | 14 | 保留 framework API 价值，但依赖凭证、网络、系统 runtime、git 或本地服务；不得混入无条件 gate。 |
| `delete` | 0 | 当前没有足够证据安全删除 public API 覆盖；`demo28` 等看似重复项仍对应公开类型/文档 owner。 |

### 依赖与风险标记

所有根 examples（除 `demo70` 的路径问题外）都以 `echo_agent::...` 访问 facade；没有直接写
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

当前静态扫描结果：根 64 个 source 中有 14 个文件共 52 处 `unwrap/expect`；真正的字符串
字节切片主要在 `demo08`, `demo40`, `demo61`，另有若干 `.len()` 字节计数和 JSON index。
这不是本次 source 修复的结果，而是 R2 迁移前的阻塞清单。

## 64 个根 examples

`Facade/internal` 列使用 `facade -> implementation owner`；`Docs/test owner` 给出迁移后事实源。
`T:*` 是建议的 tests owner 名称（本次不创建这些测试文件），`C:*` 指
`echo-agent-examples/src/main.rs` 与其 facade smoke/consumer gate。

| # | source / Cargo target | disposition | feature / prerequisite | Facade -> internal owner | 风险 | Docs / test owner |
|---:|---|---|---|---|---|---|
| 00 | `demo00_quickstart.rs` | `move-consumer` | 无 feature；`ECHO_AGENT_PROVIDER/BASE_URL/API_PROTOCOL/MODEL/API_KEY`，真实 LLM | `agent!`, `tool` -> `echo_core` + `echo_macros` | `X` API key/network | `C:quickstart`; `D:getting-started`; `T:facade_smoke` |
| 01 | `demo01_tools.rs` | `keep-root` | 无 feature；provider config/API key，真实 LLM | `prelude`, `#[tool]` -> `echo_core` + `echo_macros` | `X` LLM；教学代码无确定性输出断言 | `D:02-tools`; `T:tool_call_contract` |
| 02 | `demo02_tasks.rs` | `keep-root` | 无 feature；`qwen3-max` provider/API key，真实 task tools | task tools/ReAct -> `echo_orchestration` + `echo_core` | `X` LLM；仅检查非空结果 | `D:09-tasks`; `T:task_tool_contract` |
| 03 | `demo03_approval.rs` | `conditional` | `human-loop`；LLM；自动 `HumanLoopManager` approval/input | human loop/permission -> `echo_orchestration` | `X` LLM + HITL；事件依赖 | `D:05-human-loop`; `T:approval_contract` |
| 04 | `demo04_subagent.rs` | `move-test` | `testing`；`MockLlmClient` + local human loop，无外部服务 | subagent -> `echo_orchestration` + `echo_core` | 无 `P/U/S`；可确定性化 | `D:06-subagent`; `T:subagent_contract` |
| 05 | `demo05_compressor.rs` | `keep-root` | `human-loop`；真实 LLM，审批工具自动响应 | compression -> `echo_state`; HITL -> `echo_orchestration` | `X` LLM/HITL | `D:04-compression`, `05-human-loop`; `T:compression_walkthrough` |
| 06 | `demo06_mcp.rs` | `conditional` | `mcp`；Node.js 18+/`npx @modelcontextprotocol/server-filesystem`；LLM key | MCP -> `echo_integration` | `X` npx/process/network/API key | `D:08-mcp`; `T:mcp_process_contract` |
| 07 | `demo07_skills.rs` | `conditional` | `files`；`/tmp` read/write；Part 3 需要 LLM | skills/files -> `echo_tools` + `echo_execution` | `X` filesystem + LLM | `D:07-skills`; `T:skill_contract` |
| 08 | `demo08_external_skills.rs` | `conditional` | manifest 要求 `sqlite`，实际主路径是 file skills/scripts；LLM 与 `/tmp` fixtures | external skills -> `echo_tools` + `echo_execution` | `U` slice@466；`J` JSON indexes@301-380；`F` sqlite requirement appears unused；`X` scripts/files/LLM | `D:07-skills`; `T:external_skill_contract` |
| 09 | `demo09_file_shell.rs` | `conditional` | `files,shell`；`/tmp` workspace、shell commands、LLM | files/shell -> `echo_tools` + `echo_execution` | `X` filesystem/shell/LLM | `D:21-common-tools`, `41-shell-text-tools`; `T:file_shell_contract` |
| 10 | `demo10_streaming.rs` | `keep-root` | 无 feature；真实 LLM streaming/provider config | streaming -> `echo_core` + root agent | `X` LLM/network | `D:10-streaming`; `T:stream_event_contract` |
| 11 | `demo11_callbacks.rs` | `keep-root` | 无 feature；真实 LLM，callback stream | callbacks -> `echo_core` | `X` LLM/network | `D:23-hooks`; `T:callback_contract` |
| 12 | `demo12_resilience.rs` | `move-test` | `testing`；`MockLlmClient`/flaky tool，离线 | retry/agent -> `echo_core` | 无 `P/U/S`；适合 deterministic test | `D:30-react-safety`; `T:resilience_contract` |
| 13 | `demo13_tool_execution.rs` | `move-test` | manifest 无 feature；当前 `ReactAgent` 真实执行需 provider/API key，含 timeout/sleep；测试应注入 mock | tool execution -> `echo_core` | 当前无 `P/U/S`；`X` live LLM；需拆出 mock acceptance | `D:02-tools`; `T:tool_execution_contract` |
| 15 | `demo15_structured_output.rs` | `keep-root` | 无 feature；真实 LLM，JSON-mode/output schema provider | structured output -> root agent + `echo_core` | `J` value index@83；`X` LLM/JSON mode | `D:11-structured-output`; `T:structured_output_contract` |
| 17 | `demo17_chat.rs` | `keep-root` | 无 feature；真实 LLM，多轮 chat/stream | chat -> root agent + `echo_core` | `X` LLM/network | `D:13-chat`; `T:chat_contract` |
| 18 | `demo18_semantic_memory.rs` | `conditional` | 无 feature；embedding endpoint + LLM；`InMemoryStore`，非 SQLite | memory/embedding -> `echo_state` + `echo_core` | `X` embedding/LLM；无 `S` | `D:14-semantic-search`; `T:semantic_memory_contract` |
| 19 | `demo19_guard.rs` | `keep-root` | 源码以 `cfg(feature="content-guard")` 为主路径，但 manifest 未声明 `required-features`；真实 LLM | guard/audit -> `echo_core` | `F` Cargo metadata 漏掉 `content-guard`；`X` LLM | `D:18-guard-system`; `T:guard_contract` |
| 20 | `demo20_audit.rs` | `keep-root` | `human-loop`；自动审批 provider + 真实 LLM | audit/permission -> `echo_core` + `echo_orchestration` | `X` LLM/HITL；无 `P/U/S` | `D:05-human-loop`; `T:audit_contract` |
| 23 | `demo23_a2a.rs` | `move-consumer` | `a2a`；构造 local server/client JSON-RPC；无外部服务但协议边界公开 | A2A -> root `a2a` + `echo_integration` | `P` unwrap@39,114；`X` URL/server semantics | `C:a2a_probe`; `D:README`; `T:a2a_contract` |
| 24 | `demo24_topology.rs` | `move-test` | `topology`；本地 tracker/callback，无 LLM call | topology -> root + `echo_core` | `P` unwrap@61 (`to_json`) | `D:README`; `T:topology_contract` |
| 25 | `demo25_macros.rs` | `move-consumer` | 无 feature；宏 facade compile + 可选真实 LLM/API key | proc macros -> `echo_macros` + `echo_core` | `P` unwrap@123；`U` bytes@75；`X` API key | `C:macro_probe`; `T:macro_facade` |
| 26 | `demo26_provider_factory.rs` | `move-consumer` | 无 feature；只构造 provider/model client，使用 placeholder endpoint/key，不发请求 | LLM config/factory -> `echo_core` | `X` endpoint/key are placeholders；无 `P/U/S` | `C:provider_factory_probe`; `D:38-factory-modes`; `T:provider_config_contract` |
| 27 | `demo27_sqlite_memory.rs` | `conditional` | `sqlite`；bundled SQLite/FTS5；可选 embedding + LLM；临时 DB/WAL/SHM | memory -> `echo_state`; SQLite facade -> `echo_state` | `S`; `P` unwrap@138,263,267；`J` JSON@224,263,267,339,455；`X` embedding/LLM/files | `D:41-persistence-concepts`, `14-semantic-search`; `T:sqlite_store_contract` |
| 28 | `demo28_workflow.rs` | `keep-root` | 无 feature；Sequential/Concurrent/DAG 三种公开 workflow 都运行真实 LLM | workflow -> `echo_orchestration` | `X` 多个 LLM；API 是公开且仍有 docs owner，非 delete | `D:17-graph-workflow`; `T:workflow_legacy_shapes` |
| 29 | `demo29_sandbox.rs` | `conditional` | 无 feature metadata；Local 必需，Docker/K8s/`curl`/`python3` 能力按环境可选 | sandbox -> `echo_execution` | `X` Docker/OS/process/network；不可把 Docker skip 当全绿 | `D:30-react-safety`; `T:sandbox_contract` |
| 30 | `demo30_mcp_server.rs` | `move-test` | `mcp`；in-process JSON-RPC server，协议流程可离线验证 | MCP server -> `echo_integration` | `J` response indexes@131,156,165-166,181-230,263-323；`unwrap_or` 本身不 panic | `D:08-mcp`; `T:mcp_json_rpc_contract` |
| 31 | `demo31_memory_tools.rs` | `move-test` | manifest 要求 `sqlite`，但源码只用 `InMemoryStore`，不构造 `SqliteStore`，不执行 LLM | memory tools -> `echo_state` + root agent | `F` 多余 sqlite required-feature；无 `P/U/S`；应改为 no-feature deterministic test | `D:14-semantic-search`; `T:memory_tools_contract` |
| 32 | `demo32_token_budget.rs` | `move-test` | 无 feature；前半可 mock，Agent 执行当前需 LLM/provider | token budget/tool result -> `echo_core` | `P` unwrap@71；`U` byte count@138；`X` live LLM；应拆 mock acceptance | `D:02-tools`; `T:token_budget_contract` |
| 33 | `demo33_retry_policy.rs` | `conditional` | 无 feature；local policy 部分可离线，LLM/MCP retry 部分需服务 | retry -> `echo_core` | `P` unwrap@151,173；`X` LLM/MCP/network | `D:30-react-safety`; `T:retry_policy_contract` |
| 34 | `demo34_workflow_stream.rs` | `move-test` | 无 feature；function-node stream 全部本地 | workflow stream -> `echo_orchestration` | `U` word byte count@87；其余无 `P/S` | `D:17-graph-workflow`; `T:workflow_stream_contract` |
| 35 | `demo35_dynamic_tools.rs` | `move-test` | 无 feature；API edge cases 离线，Part 3 当前真实 LLM；应拆出前两部分 contract | dynamic tools -> root agent + `echo_core` | `I` uses deep `agent::react::ReactAgent`; `X` Part 3 LLM；断言可迁移 | `D:02-tools`; `T:dynamic_tool_contract` |
| 36 | `demo36_multimodal.rs` | `conditional` | 无 feature metadata；YAML model config、vision-capable provider、remote image URLs/API key | multimodal LLM -> root agent + `echo_core` | `X` provider/remote images/model capability | `D:11-structured-output`; `T:multimodal_contract` |
| 37 | `demo37_declarative_workflow.rs` | `move-test` | 无 feature；YAML/JSON parse/build/invalid-input assertions 离线 | workflow loader -> `echo_orchestration` | 无 `P/U/S`；强 deterministic acceptance | `D:17-graph-workflow`; `T:declarative_workflow_contract` |
| 38 | `demo38_im_channels.rs` | `move-consumer` | `channels`；Feishu/QQ credentials、网络长轮询、LLM client | channels -> `echo_integration` | `X` credentials/network/long poll；consumer composition boundary | `C:channel_probe`; `D:15-im-channels`; `T:channel_composition_contract` |
| 39 | `demo39_workflow.rs` | `move-test` | `testing`；MockAgent/function graph，离线 | Graph/SharedState -> `echo_orchestration` | `P` unwrap@102,105,109,180,181,248,252,256,332,336,340,344,387,394,429,431,432,444,448；`U` byte counts@283,291；应全部 checked access | `D:17-graph-workflow`; `T:graph_state_contract` |
| 40 | `demo40_snapshot.rs` | `keep-root` | 无 feature；真实 LLM，多轮快照/rollback | snapshot -> root agent + `echo_core` | `U` byte slices@45,60,79；`X` LLM | `D:41-persistence-concepts`; `T:snapshot_contract` |
| 41 | `demo41_web_tools.rs` | `conditional` | `web`；DuckDuckGo/Brave/Tavily/network keys；web fetch | web tools -> `echo_tools` | `U` bytes count@213；`X` network/API keys；provider can be unavailable | `D:20-web-tools`; `T:web_tool_contract` |
| 42 | `demo42_playwright_mcp.rs` (target `demo42_browser_mcp`) | `conditional` | `mcp`；`mcp.json` + Node/Playwright MCP + browser + LLM config | MCP -> `echo_integration` | `X` browser/npx/config/network；UTF-8 previews use `chars().take` safely | `D:08-mcp`; `T:playwright_mcp_contract` |
| 43 | `demo43_data_tools.rs` | `move-test` | manifest 仅 `testing`，但实际工具覆盖 Excel/Word/Text/CSV/Parquet，目标应是 `testing,data,media`；MockLlm + generated fixtures | data tools -> `echo_tools` | `F` current target can compile while runtime tool registry lacks required `data/media`; `P` unwrap@243,249；`P` expect@341；`X` filesystem/format fixtures | `D:42-database-tools`, `43-data-output-format`; `T:data_tools_contract` |
| 44 | `demo44_code_laboratory.rs` | `move-consumer` | `content-guard`；Python 3 + local sandbox + LLM；代码执行场景 | guard -> `echo_core`; sandbox/tools -> `echo_execution` + `echo_tools` | `X` python/process/LLM；scenario should be consumer-owned | `C:code_lab`; `D:18-guard-system`; `T:code_lab_contract` |
| 45 | `demo45_customer_service.rs` | `move-consumer` | `sqlite,human-loop,content-guard`；SQLite DB + LLM + approval | memory -> `echo_state`; HITL -> `echo_orchestration`; guard -> `echo_core` | `S`; `J` `mem.value["content"]`@438；`X` LLM/HITL/files | `C:customer_service`; `D:05-human-loop`, `41-persistence-concepts`; `T:consumer_scenario_smoke` |
| 46 | `demo46_data_analyst.rs` | `move-consumer` | `sqlite`；SQLite + embedding + LLM + generated data files | memory -> `echo_state`; workflow -> `echo_orchestration` | `S`; `P` expect@513；`J` value indexes@197,483,489；`X` embedding/LLM/files | `C:data_analyst`; `D:14-semantic-search`; `T:consumer_scenario_smoke` |
| 47 | `demo47_enterprise.rs` | `move-consumer` | `testing` declared；source requires `examples/demo_skills` fixture and live LLM in several parts；README text mentions SQLite but no `SqliteStore` call | skills/tools/workflow/topology -> `echo_tools` + `echo_orchestration` + root | `F` missing fixture/SQLite prose drift；`U` code byte count@96；`X` fixture/LLM | `C:enterprise_scenario`; `D:07-skills`, `17-graph-workflow`; `T:consumer_scenario_smoke` |
| 48 | `demo48_personal_assistant.rs` | `move-consumer` | `sqlite,subagent`；SQLite + subagent + LLM；multimodal model optional | memory -> `echo_state`; subagent -> `echo_orchestration` | `S`; `J` value indexes@151,152,164；`X` LLM/subagent/model | `C:personal_assistant`; `D:06-subagent`, `41-persistence-concepts`; `T:consumer_scenario_smoke` |
| 49 | `demo49_research_agent.rs` | `move-consumer` | `sqlite,web,files`；SQLite + web + filesystem + LLM + embedding | memory -> `echo_state`; web/files -> `echo_tools`; workflow -> `echo_orchestration` | `S`; `P` expect@497；`J` value indexes@178-199；`X` network/files/LLM | `C:research_agent`; `D:20-web-tools`, `22-research-tools`; `T:consumer_scenario_smoke` |
| 50 | `demo50_eval.rs` | `move-test` | `eval`；local trace/eval/replay/HTML generation，no LLM required | eval/trace -> root package | `X` writes report files and ignores `std::fs::write` errors@459-463；tighten to checked tempdir | `D:24-eval-system`; `T:eval_contract` |
| 51 | `demo51_self_improvement.rs` | `move-test` | `eval,improve`；local synthetic runs/curator/trajectory | eval/improve/evolution -> root package | 无 `P/U/S`；deterministic assertions already substantial | `D:25-self-improvement`; `T:self_improvement_contract` |
| 53 | `demo53_adaptive_compression.rs` | `move-test` | 无 feature；local heuristic compression；L4 only optional `.with_llm()` | compression -> `echo_state` | `P` unwrap@400,450；`P` expect@401；replace with checked assertions/errors | `D:04-compression`; `T:adaptive_compression_contract` |
| 54 | `demo54_headless.rs` | `move-test` | 无 feature；main is local config/result formatting; live LLM only documented optional path | headless -> root package | `P` unwrap@133；`J` `parsed[...]`@134-136；replace with checked JSON access | `D:33-headless-mode`; `T:headless_contract` |
| 55 | `demo55_lsp_tools.rs` | `move-test` | `lsp`；YAML/config/tool construction local；actual language-server binaries optional and not started | LSP -> `echo_integration`; tools -> `echo_tools` | `P` unwrap@81,91,112,147,213,218,244；optional binaries `X` only for future live path | `D:31-lsp-integration`; `T:lsp_config_contract` |
| 56 | `demo56_plugin_system.rs` | `move-consumer` | 无 feature；tempdir plugin manifest/skill/hook lifecycle，无 LLM | plugin/skills -> root package + `echo_tools` | 无 `P/U/S`；适合作 facade consumer lifecycle probe | `C:plugin_probe`; `D:32-plugin-system`; `T:plugin_lifecycle_contract` |
| 57 | `demo57_data_pipeline.rs` | `move-test` | `testing`；MockAgent + config/state contract，离线，无文件写入 | pipeline -> `echo_orchestration` | 无 `P/U/S`；可作为 deterministic pipeline contract | `D:35-pipelines`; `T:data_pipeline_contract` |
| 58 | `demo58_git_worktree.rs` | `conditional` | `git`；当前 checkout 必须是 git repo；创建/删除 worktree 和 branch | git isolation -> `echo_tools` | `X` mutates repo/worktree and invokes git; never default CI without isolated temp repo | `D:34-git-isolation`; `T:git_worktree_contract` |
| 59 | `demo59_code_search.rs` | `conditional` | `files`；读取 repository，优先 `rg`，缺失时内置 fallback | code search -> `echo_tools` | `X` filesystem/`rg`/repository shape；结果依赖 checkout | `D:37-code-search`; `T:code_search_contract` |
| 60 | `demo60_data_quality.rs` | `move-test` | `data,statistics`；生成 `/tmp/echo_demo60_data.csv` 后运行工具；应改 tempdir 避免固定路径 | data/statistics -> `echo_tools` | `X` fixed `/tmp` path and cleanup; no `P/U/S`; fixture-based test owner | `D:36-data-quality-statistics`; `T:data_quality_contract` |
| 61 | `demo61_agent_factory.rs` | `move-consumer` | `testing`；factory + MockLlm local，第一段只是 config print | factory -> root package + `echo_core` | `U` byte slice@46；consumer cutover must use `chars().take` | `C:factory_probe`; `D:38-factory-modes`; `T:agent_factory_contract` |
| 62 | `demo62_prompt_templates.rs` | `move-test` | 无 feature；template substitution/default/conditional/thread-safe local | prompt templates -> `echo_core` | 无 `P/U/S`；25 assertions，直接 test owner | `D:40-context-system`; `T:prompt_template_contract` |
| 64 | `demo64_tool_pipeline.rs` | `move-test` | 无 feature；intervention/callback/config pipeline local；Agent execution path should use mock | tool pipeline -> `echo_core` | 无 `P/U/S`；16 assertions，保留为 contract | `D:02-tools`, `23-hooks`; `T:tool_pipeline_contract` |
| 65 | `demo65_context_assembler.rs` | `move-test` | 无 feature；local message assembly/budget | context -> root package + `echo_core` | `U` token estimate uses `text.len()/4`@261；改为 Unicode-safe count/tokenizer | `D:40-context-system`; `T:context_assembler_contract` |
| 66 | `demo66_context_selector.rs` | `move-test` | 无 feature；synthetic paths/symbols, no actual file reads; deterministic scoring | context selector -> root package | `P` `Path::to_str().unwrap()`@277,279,280,298,300,301；use checked display/fallback | `D:40-context-system`; `T:context_selector_contract` |
| 67 | `demo67_progress.rs` | `move-test` | 无 feature；local watch/bus/timer + typed extension | tasks/progress -> `echo_orchestration` | 无 `P/U/S`；timing should use bounded deterministic test clock where possible | `D:09-tasks`; `T:progress_contract` |
| 68 | `demo68_human_gate.rs` | `move-consumer` | `subagent,human-loop`；local `HumanLoopManager` selection/approval + timeout | HITL -> `echo_orchestration` | 无 `P/U/S`；public consumer selection probe | `C:human_selection_probe`; `D:05-human-loop`; `T:human_selection_contract` |
| 70 | `demo70_scheduler.rs` | `move-consumer` | 无 feature；temp file-backed cron store, local `SchedulerRunner` | scheduler -> `echo_orchestration` | `I` imports `echo_agent::workspace::orchestration::scheduler` instead of stable facade path; `X` temp file | `C:scheduler_probe`; `D:29-long-running-tasks`; `T:scheduler_contract` |

## Consumer crates

### `echo-agent-examples`

`echo-agent-examples/Cargo.toml` 是 `publish = false` 的独立 package，唯一 dependency 是
`echo_agent = { path = "..", default-features = false }`。`src/main.rs` 构造
`FrameworkConfig`、`DataRoot`、`StandardToolPack` 和 `AgentInvocationContext`，验证 resource
guard identity；它没有 split-crate import、SQLite、外部服务、panic API 或 UTF-8 byte slice。
这是 R2 的 facade consumer gate，不是第二套 examples owner。上表 `move-consumer` 项应在此
扩展为 feature-isolated probes，或在该 crate 下增加 examples，但每个新增 Cargo target 仍只
能依赖 `echo_agent`。

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

## Required follow-through for the next implementation plan

1. 先修 manifest/documentation drift：至少 `demo19` 的 `content-guard` target gate、`demo31`
   的多余 `sqlite` requirement、`demo43` 的 `testing`→`data,media,testing` gate、`demo47` 的
   missing `examples/demo_skills`/SQLite prose，以及 `demo70` 的深层 `workspace::orchestration`
   import。
2. 将 14 个 `P` 文件的强制提取全部换成 checked access；将 `demo08`, `demo40`, `demo61`
   的 byte slice 和 `demo32`, `demo34`, `demo39`, `demo41`, `demo47`, `demo65` 的字符串
   byte count 改为 Unicode-safe/typed tokenizer 路径；`J` 标记也应改为 `.get()`/typed parse。
3. `move-test` 不得只是改目录：每个测试必须有 deterministic fixture、明确失败断言和
   feature-isolated Cargo target；`demo13`, `demo32`, `demo35` 需要先移除 live LLM 才能称为
   deterministic acceptance。
4. `move-consumer` 的唯一直接依赖仍是 `echo_agent`；任何 split-crate path 只能作为 framework
   内部 implementation 证据，不能出现在 consumer manifest/source。
5. `conditional` 项必须在 docs 中列出凭证/服务/运行时 prerequisite，并对缺失前置条件
   fail-fast；不能把 `Ok(())` 或“跳过外部能力”误报为成功。
6. 本文没有声称 source 已通过 R2 禁用扫描；它记录了扫描发现，真正清零要由后续实施计划和
   all-target gate 验收。SQLite framework API 仍保留，EKO CLI 的 no-SQLite 产品策略不改变此结论。

## Verification evidence

在指定 commit 的 integration checkout 执行：

```text
cargo metadata --no-deps --format-version 1
cargo check --workspace --all-targets --all-features --locked
```

两条命令均通过；`cargo check` 覆盖了 root `echo_agent` 的 64 个 example targets、
`echo-agent-examples` 和 `echo-rust-learning`。静态计数还由 `find examples -name '*.rs'`
（64 个根 demo source，另有 `support/mod.rs`）和 manifest/metadata 对照确认。源码禁用扫描
按上述规则得到 14 个 root files / 52 个 `unwrap/expect` occurrences；该扫描是发现性证据，
不是“已经清零”的通过结论。

完整的 `cargo test --workspace --all-targets --all-features --locked` aggregate gate 本次不作为
证据：R2 是只读 inventory，且按用户指令把 workspace/aggregate/scale/soak 验证延后到最终
重构完成后。consumer 与 learning 的 focused test 已通过；最终 gate 仍必须重新执行并取得完整
exit status，不能用本节的 `cargo check` 或 focused test 替代。
