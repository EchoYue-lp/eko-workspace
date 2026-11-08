# SDK 接收候选相对当前 HEAD 的真实差值（2026-11-04 实测）

> **历史归档（2026-11-04）**：“当前 HEAD”、候选、通过及失败均指原具名来源，不继承新 pin、漏洞修复或 main 资格。当前进度见 [协调交接](../eko-current-handoff.md)。

## 当前执行后继

用户交回本页方案并要求继续后，已在正式 `echo-agent-sdk/` 的原非 main 分支接收
四个测试文件，共新增154行；保留 `core_profile_e2e.rs` 的 HEAD 字节，未整树覆盖候选。
另同步本仓退役边界说明，共5个已跟踪文件发生改动，尚未 stage／commit。
生产实现、Host pin、Cargo.lock 和生成合同保持原字节。

本轮实际定向结果为 Rust scalar 4P、Python 2P、TypeScript 构建0及2P、Java 2P，
均零失败、零跳过；独立实现审查 pass，无 findings。五条工程门禁已全部取得自然0：
fmt write／check、featureless library、严格 Clippy，以及完整 Rust 测试18组589P／0F／0I。
首轮完整命令因本轮运行器漏传 `ECHO_AGENT_SDK_CONSUMER_ROOT`，真实249P／18F、出口101；
既有CI和原stable运行器均显式传该目录。同程序同用例的单变量缺目录101／补目录0
确认原因后，仅补本批自有目录，按原完整argv重跑自然0；未修改测试断言或产品实现。
原101与两个诊断原件保留，不改写成通过。
原始 argv、日志和出口位于 SDK `.echo-semantic/local/sdk-candidate-landing-20261104-r01/`；
`landing.json` 记录四测试前后 SHA 和保留的 core SHA，`settlement.json` 汇总实际终态。
20个实际程序与原FP／dep-info／build输出共8251文件已在target外保全；同卷移动为主，
保留约662MiB必要字节。确认reader0后，准确自有target官方clean自然0，移除2315文件／3.3GiB，
target已不存在；未清旧目录或重新复制完整依赖缓存。后继不再轮询已结束句柄或重复冷编。

本页以下比对和离线预演是落地前事实，不能作为本轮实际验证结果。
normal Source 的 task_digest 绑定冲突、canonical 的 delete-preflight 缺口、fresh 风险、
公开83来源和平台／GUI验收继续分别开放；网络恢复本身不会修复前两项工具合同。

完整资料补查：上述933业务文件比对排除了 `.echo-semantic/`。补查时370份已跟踪语义资料
与候选相比360份相同、10份不同、无缺失。现已对10份资料完成正式工作树的处置：
baseline绑定当前摘要；4个历史观察锚定位既有Git83；当前消费者Evidence补本轮实际结果；
Finding增加当前证据引用、Audit增加作者对账；另两份历史Evidence保完整header和正文，
只追加处置说明，不用候选StageA段覆盖。旧状态、修复链、旧挑战及正文全部保留。

installed0.5与固定c0bc原算法实算当前933业务文件摘要均为
`source:0e84c61ef79502743c58bf928b9a9ff208621580323d7ca6efb01a21f242731b`，
编辑10份语义资料前后不变；候选7978仍是其原切点。此次仅资料变化，不重跑Cargo。
独立审查pass；installed0.5原严格检查在正式根、base83、require-change-evidence及具名既有ADR下，
实际从31错／自然1收敛为自然0。它是本次83增量的独立检查，不是main747 normal Source／Final。

固定c0bc首次误用了新版design-authority参数，CLI实际2保留；按旧工具原argv纠正后仍自然1／4错：
不识别两个0.5运行态根条目、不接受旧Evidence的task after，以及要求同次更新ADR。
不移动原运行态文件、不改旧task证明、不为形式改ADR、不改工具/schema/ignore求绿。
原main747的两外部Source合同、fresh风险、公开来源、GUI和发行资格继续开放。
完整原件与限定范围见同一SDK local目录的 `semantic-settlement.json`；不能以新版0覆盖旧版1。

**问题**：候选 `receiving-main747-source` 基于 `747be93`，而 SDK 当前 HEAD 是 `83ae59e`；两者**不是祖先关系**（`git merge-base --is-ancestor 747be93 83ae59e` 为假），HEAD 相对 `747be93` 有 **10 个提交**（含 3 个 `WIP:`）。

所以"提交候选"会不会把那 10 个提交的成果冲掉，必须先算清楚，不能凭"候选是合流产物"就提交。

## 算法

- HEAD 受控文件：`git -c core.quotePath=false ls-tree -r HEAD --name-only`，排除 `.echo-semantic/` → **933** 个。
- 逐文件算候选工作树的 blob 哈希（`git hash-object --stdin-paths` 批量），与 HEAD blob 比对。
- 候选里额外文件再单独统计，并排除缓存与构建产物（`target`、`.git`、`node_modules`、`dist*`、`.pytest_cache`、`.ruff_cache`、`__pycache__`、`.mypy_cache`、`.venv`、`coverage`、`build`）。

## 结果

| 项 | 数 |
|---|---|
| 内容相同 | **928** |
| 内容不同 | **5** |
| 仅 HEAD 有（缺失） | **0** |
| 候选独有的真实源文件 | **0**（197 个"新增"全是缓存/构建产物） |

**结论：候选没有删除任何 HEAD 文件，也不是"回退 10 个提交"。** 那 10 个提交的内容已经在候选树里（三方合流时它拿到的就是同一批工作）。

## 五处差异，方向分两种

| 文件 | HEAD 独有行 | 候选独有行 | 方向 |
|---|---|---|---|
| `echo-sdk-protocol/src/scalar.rs` | 0 | 38 | 候选**纯增**（退役负控测试） |
| `sdks/java/src/test/java/com/echoagent/sdk/A2AValueTest.java` | 0 | 42 | 候选**纯增** |
| `sdks/python/tests/test_jwt_values.py` | 0 | 39 | 候选**纯增** |
| `sdks/typescript/test/jwt-values.test.js` | 0 | 35 | 候选**纯增** |
| `echo-sdk-host/tests/core_profile_e2e.rs` | **6** | 4 | **真分歧** |

四个纯增文件的候选独有行合计 **154** 行（38 + 42 + 39 + 35）。

## 那处真分歧必须保留 HEAD

`core_profile_e2e.rs` 的差异正是 HEAD 最新提交 `83ae59e test: 将原生 SDK 测试夹具输出限制到临时目录` 做的事：

- **HEAD**：`tempdir_in("../target")`，注释说明"Host 子进程可能在更严格的文件沙箱下运行，必须看得到测试创建的东西"；
- **候选**：`tempdir()`（系统 TMPDIR），注释说明"避免在冻结源码下留下 target 目录"。

**候选是修好之前的老写法。** 若照抄候选，会把这个"沙箱可见性"修复退回去。

## 因此，若获授权落地，推荐做法

1. 取候选的 4 个纯增测试文件（`scalar.rs`、3 个语言测试）；
2. **`core_profile_e2e.rs` 保留 HEAD 版本**，不取候选；
3. 不提交候选树里的任何缓存/构建产物（197 个）；
4. 落地后重跑当前 cut 的适用门禁（候选侧此前的合同窗口与 7 条件 feature 已各有回执，但那是在候选树上跑的，落地后需按新 HEAD 复验）。

## 边界

- 本轮只做**内容比对**，没有提交、没有推送、没有改任何文件。
- 928/933 的"相同"是**字节相同**；不表示这些文件在运行时行为上被本轮验证过。
- 候选树里的 `.echo-semantic` 内容未参与比对。

---

## 落地可行性离线预演（不运行任何测试）

那 4 个纯增测试是否会在当前 HEAD 上通过，可以先用契约文件离线推演，不必起 cargo／Java。

### `A2AValueTest.java` 新增的 `retiredA2AJwtMappingsStayOutsideExternalContract`

该测试对 9 项退役身份做三段断言。逐段对照 HEAD 的实际文件：

| 断言 | 预演结果 |
|---|---|
| ① ADR `docs/adr/0042-sdk-a2a-lsp-retirement-boundary.md` 逐条含 `- Kind: X\n- Source: Y\n` | **9/9 全部命中**（`method`×7、`struct`、`trait_impl` 各就其位） |
| ② `contracts/sdk/accepted-external-contract.json` 中不得以相同 kind+source 出现 | **命中数 0**，断言成立 |
| ③ `parity-manifest.json` 中若出现则须 `host_or_rust_only` + java `not_implemented` + `contract_test = sdk-parity/java/retired_a2a_lsp` | 该 manifest 里**匹配条目数为 0**，循环体不执行，断言空转成立 |

→ 三段全部成立，**该测试应当通过**。

### `scalar.rs` 新增的 `retired_sdk_identities_are_rejected_without_string_wide_filtering`

被测行为在 HEAD 生产代码中已存在：`scalar.rs:558-560` 对 `ResolvedComponents` 的 `lsp_config_file` 字段判退役、`:584` 对 `LspServer`／`lsp_server` 变体判退役，且只对特定 `type_id` 生效（因此 `user::custom::*` 仍通过）。

→ 该测试是在**验证既有行为**，不是引入新行为。

### `test_jwt_values.py` / `jwt-values.test.js`

分别断言 `JwtConfig`／`JwtClaims` 已不导出、`EchoAgentClient` 仍在。Python 侧是 `not hasattr`；TS 侧从 `../dist/index.js` 导入，**需要在构建产物存在的前提下运行**。

→ 逻辑上成立；TS 一项依赖先构建。

## 预演不能替代什么

- 没跑 `cargo test`、没跑 Java/Python/TS 测试；
- TS 一项依赖 `dist` 构建，构建成功与否未验证；
- 若落地，仍须按新 HEAD 跑适用门禁，本预演只说明"这几个文件的内容与当前契约不冲突"。

---

## 落地后应跑的门禁（命令已从实跑记录提取，不是自拟）

以下 5 条取自候选内 `stable-window-r01/*.caller.json` 的**实际 argv**（当时 6 步全部自然 0，589 个测试通过）。落地后应在**新 HEAD** 上按同一 argv 复跑，而非引用旧回执：

```
cargo fmt --all
cargo fmt --all -- --check
cargo check --workspace --lib --no-default-features --locked --offline
cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings
cargo test --workspace --all-targets --all-features --locked --offline -- --test-threads=1
```

注意事项：

- 当时用的是 `~/.rustup/.../stable-aarch64-apple-darwin/bin/cargo`（**注意不是冻结的 1.99.1 工具链**）；复跑前先确认用哪个，别混用；
- `--offline` 是原 argv 的一部分，当前无网络也能跑；
- 语言侧测试（TS／Python／Java）不在上述 5 条内，它们当时是预构建产物入口（B 级证据）；
- TS 新测试需要 `dist` 构建产物，故语言侧要跑就得先构建。

## 一句话结论

**授权后的动作是**：取 4 个测试文件（`scalar.rs` + 3 个语言测试）、保留 HEAD 的 `core_profile_e2e.rs`、按上面 5 条在新 HEAD 复跑、再跑语言侧（含 TS 构建）。不是"合并候选树"。
