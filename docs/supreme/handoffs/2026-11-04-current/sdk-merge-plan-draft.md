# SDK 两线合并方案（草稿，未执行）

> 状态：**仅分析，未 fetch / 未 push / 未 merge / 未动分支**。SDK 仓库 HEAD 仍为 `83ae59e`，工作树 0 脏路径。执行需另行授权。

## 一、事实基线

| 项 | 值 |
|---|---|
| merge-base | `0930eef` |
| 本地任务分支 `chore/Echoyue/root-workspace-convergence` | `83ae59e`（领先 main 10 个提交，含 3 个 `WIP:` 保存点） |
| `main` = `origin/main` | `747be93`（2 个提交：退役 + 校准；两者完全一致） |
| 本地改动面 | 290 文件 |
| 远端改动面 | 126 文件 |
| 双方都改 | 124 文件 |
| **文件级冲突** | **59 文件**（`git merge-tree` 只读试算；全文件级判定，未做 hunk 级合并，故为上限） |
| 仅本地有 | 166 文件（含原生消息新增的 96 个中的多数） |
| 仅远端有 | 2 文件 |

关键内容差：本地线新增的 96 个文件中 **81 个在 origin/main 不存在**，含 `echo-sdk-host/src/core_profile/native_message.rs`、`docs/adr/0043-native-message-sdk-adapter.md`、`docs/sdk/native-messages.md` 与三语言原生消息测试/夹具。反向，origin 有一个本地没有的 wire 字段 `AgentCheckpointWire.current_plan`。

## 二、冲突面按类型分布

> 口径说明：`git merge-tree`（旧式输出）把“两侧都改过”的文件一律列入 `changed in both`。因此 59 是**需要人工介入的文件数上限**；真正 hunk 级无法自动合并的会更少。

| 类别 | 数量 | 处理性质 |
|---|---|---|
| `sdks/{python,typescript,java}` 语言客户端 | 33 | 多为“远端删除 / 本地修改”，A2A-LSP 退役裁决即可批量定 |
| `contracts/sdk/**` 生成产物 | 8 | **禁止手工合并**，解决源码后按官方入口重生成 |
| `echo-sdk-protocol/src` 源码 | 3 | 语义裁决 |
| `echo-sdk-host/src` 源码 | 2 | 语义裁决 |
| 测试文件（protocol/host） | 2 | 跟随源码 |
| 构建与策略（`Cargo.toml`×2、`Cargo.lock`、`.cargo` 基线、`rust-ci.yml`） | 5 | 机械合并 + 复核 |
| 脚本与说明（`scripts/check-sdk-contracts.sh`、`AGENTS.md`、`README.md`、governance 文档） | 4 | 语义裁决 |
| `sdks/shared/**` | 2 | 由生成或退役裁决决定 |

**真正需要人判断的约 24 个**；其余属“接受删除”或“重生成”。

## 三、三种做法与取舍

| 方案 | 做法 | 优点 | 代价 |
|---|---|---|---|
| **A. 合并**（推荐） | `git merge 747be93` 到本地 main，一次解决 59 个冲突 | 保留本地 10 个提交与 WIP 痕迹；冲突只解一次；历史真实反映两条线 | 产生 merge commit；本地 main 变复杂 |
| B. 变基 | 把本地 10 个提交 rebase 到 `747be93` | 线性历史 | 每个提交都可能冲突，**要解 10 次**；`WIP:` 提交语义不清，冲突更难判 |
| C. 重建 | 以 `747be93` 为基线，把原生消息能力重新实现一遍 | 历史最干净 | 工作量最大，且等于放弃本地已验证的 81 个文件 |

推荐 **A**，理由是本地线含 3 个 WIP 保存点，变基会反复放大冲突面；而合并只需在最终状态上解一次。

## 四、建议的解决顺序

1. **先定退役裁决**：`sdks/{java,python,typescript}` 下 33 个 A2A/LSP 相关冲突，统一接受**远端删除**。本地对这些文件的修改属于退役前行为，不保留。
2. **再合构建层**：`Cargo.toml`、`echo-sdk-host/Cargo.toml`、`Cargo.lock`、`.cargo/cargo-deny-duplicate-baseline.json`。**两线 framework pin 不同**（origin `0087353f` / 分支 `3409932`），这一步实质是 pin 升级 + 依赖基线重建，必须逐项核对，不能取一侧。
3. **再解源码语义**（4 处必须显式裁决）：
   - `echo-sdk-protocol/src/inventory.rs`：本地有 `sandbox_command_projection` 覆盖语言合同分类的特例（注释称“已批准值投影”），origin 已移除。**保留还是放弃，须由了解该批准来源的人定**；它正是遥测产物差 20 字节的根因。
   - `echo-sdk-host/src/core_profile/mod.rs`：必须补回 `mod native_message;`（origin 无此行）。
   - `echo-sdk-protocol/src/managed.rs`：接受 origin 的 `current_plan: Option<String>`，但需确认它不与本地 `AgentCheckpointWire` 用法冲突。
   - `echo-sdk-protocol/src/{facade.rs,scalar.rs}`、`echo-sdk-host/src/core_profile/facade/{mod.rs,source_operations.rs}`：逐处看是否退役相关；退役相关取 origin，其余取本地。
4. **契约产物最后统一重生成**：`contracts/sdk/**` 8 个冲突文件以及 `sdks/shared/{contract-digests,facade-operation-catalog}.json` 一律不手工编辑，按官方生成入口重算，再跑完整合同门禁。
5. **文档与 CI**：`AGENTS.md`、`README.md`、`rust-ci.yml`、governance 文档取“两线并集”，退役表述取 origin、原生消息表述取本地。

## 五、合并后必须重跑的门禁

| 门禁 | 说明 |
|---|---|
| `cargo fmt --all -- --check` | 零 diff |
| 严格 clippy（含 `-D unwrap_used/expect_used/panic/unreachable`） | 全 workspace 全 targets |
| `cargo test --workspace --all-targets --all-features` | 需真实跑，不能只看编译 |
| 完整合同门禁 | 135 项阻断产物须全部 current、0 漂移；新基线含 `current_plan` 后**产物必然变化**，需重新接受 |
| 条件 feature 矩阵 | 7 个 profile，因触及 `Cargo.toml`/`#[cfg]` 而强制 |
| `scripts/check-a2a-lsp-retirement.py` | 确认退役账仍自洽 |

合同产物会因 `current_plan` 而变，这属于**被接受的合同变更**，必须留下新的接受记录，不能当作“重生成后自然一致”。

## 六、风险

- **最大风险是静默丢能力**：若按“origin 较新”直觉取一侧，81 个原生消息文件会消失，而编译与现有测试未必报错。
- **第二风险是合同产物手工合并**：8 个 JSON 手工编辑会破坏逐字节校验。
- **第三风险是锁文件取一侧**：两线依赖集合不同，直接取一侧可能引入未审依赖。

## 七、本方案不做的事

不执行合并、不建分支、不 fetch、不 push、不重写历史；不代替任何人裁决 `inventory.rs` 的投影特例是否保留；不预设哪条线是“正确基线”。

---

## 八、重要修订：先期合并已经存在（2026-13 轮补记）

本节修正前文“合并尚未开始”的措辞。既有证据树显示 **2026-11-01 已完成过一次三方合并**：

- 回执：`sdk-native-stagea837-formal83-threeway-candidate-20261101-r01/review-cut-r02/all44-static-review-cut.json`
- 状态：`ALL44_RESOLVED_SOURCE_ONLY_READY_FOR_ROOT_INCREMENTAL_REVIEW`，**44 个冲突全部解决**
- 合并树：`…/sdk-native-stagea837-formal83-threeway-candidate-20261101-r01/candidate-source`
- 该树**同时具备**：原生消息模块 `native_message.rs`、A2A/LSP 退役（A2A 客户端文件已不存在）、分支最新修复（`echo-sdk-channel-forward`）
- 该树**不具备**：origin 的 `AgentCheckpointWire.current_plan`；且仍保留 `sandbox_command_projection` 特例

### 与 origin/main（`747be93`）的残差

| 项 | 值 |
|---|---|
| 对比较对象追踪文件 | 1,229 |
| 完全相同 | **1,066** |
| 内容不同 | 147 |
| 合并树缺失 | ~~16~~ → **7**（见第十三节修正） |

差异分布：语言客户端 85、Rust 源码 22、语义资料 10、合同产物 10、构建/策略 8、文档 8、其它 4。
缺失实为 **7** 个（第 13 轮误报 16，见第十三节）：`JwtClaims/JwtConfig/jwt_values.{java,py,ts}` 与 `a2a_host_fixture.py`、`lsp_diagnostic_server.py`、`lsp_fault_server.py`。

### 因此实际剩余工作是两件事，不是重做合并

1. 决定以 `candidate-source` 为基，还是以 `747be93` 为基（**谁先谁后尚未确认**：747 可能正是由该树派生，若是，则“缺失 16 个”可能是退役删除而非遗漏）；
2. 收敛 163 个文件的残差，其中合同产物 10 个必须**重生成**而非手工合并。

### 仍未解决的三点

- `candidate-source` 只做过源码级审阅，**未跑完整合同门禁、未重生成合同产物**（原回执明示 source-only）；
- `747be93` 仍保留 `a2a_host_fixture.py` 与 `lsp_*_server.py`，与“A2A/LSP 已退役”是否矛盾未判定；
- 前文第一~七节仍有效，但应读作“直接三路合并路径的冲突面（59）”，与本节“已合并树残差（163）”是两条不同路径。

---

## 九、pin 与方向问题的答案（2026-13/14 轮补记）

**谁先谁后，已有定论（依据提交正文）**：

- `6cd2e1b`（2026-11-01 22:42）正文原话：“仅保存 main0930 基础上的独立 StageA；框架精确 pin008、accepted5352/routes431，**正式 eea 的九项原生消息后继保持独立待三方合流**。”
- `747be93`（2026-11-02 00:37）正文：仅更新 Baseline 摘要与退役 Evidence 的源码定位，业务源码/依赖/生成合同/工具规则均无变化。

因此 **origin/main 有意不含原生消息**，那是被显式推迟到三方合流的后继；`candidate-source`（审阅切点 2026-11-01T15:14Z ≈ 23:14 北京，晚于 StageA）正是那次合流尝试的产物。

**并纠正前文一处错误**：第 4 节第 2 步原写“两线 framework pin 相同（3409932）”，这是错的——那次是在接收候选**工作树**里 grep 的，而工作树属于任务分支。

| | framework pin |
|---|---|
| origin/main `747be93` | `0087353fd78eb73ffaa68b255cf4ef807ca96d4a` |
| 任务分支 `83ae59e` | `340993201cc937fc8e4783b66be399ef9f14e3d5` |

**这改变了合流成本的性质**：不只是搬 163 个文件，而是 framework pin 008→340 的升级 + 已接受合同基线重建（正文提到 origin 侧 accepted5352/routes431，与本轮接收树上合同门禁记录的 5344 身份/430 路由口径是否一致尚未判定）。

**合同基线实测**（解析两侧 `accepted-external-contract.json`）：

| | entries | distinct routes |
|---|---|---|
| origin/main `747be93` | 5,352 | 433 |
| 任务分支 `83ae59e` | 5,344 | 432 |

origin 比分支多 8 个身份、1 条路由。提交正文写 `routes431`，我数到 433，差异可能来自是否计入 intrinsic 路由，口径未统一，故不据此改判；但 8 个身份差需要在合流时逐条确认是否退役造成的合法差。

---

## 十、合同基线差的根因（2026-15 轮补记）

逐条比对后，`5352` 与 `5344` 的差**不是退役造成的，而是 framework pin 008→340 的 API 漂移**：

| 项 | 数量 |
|---|---|
| 共有条目 | 5,341 |
| 仅 origin（pin008）有 | 11 |
| 仅分支（pin340）有 | 3 |
| 共有但签名不同 | 12 |

- **仅 origin 有**：`AgentCallback::on_tool_{start,end,error}`、`AgentEvent::{Token, Token::0, ThinkStart, ThinkEnd, ThinkEnd::completion_tokens, ThinkEnd::prompt_tokens}`、`AgentCheckpoint::current_plan`、`state::delete_persisted_conversation`。
- **仅分支有**：`AgentCallback::on_tool_{start,end,error}_with_id`。
- **12 处签名差**全部落在：Agent 执行 3、LLM 请求/用量 3、流输入输出 2、PermissionMode 4。

**结论**：基线不存在“选哪个”的自由度。合同必须按最终 pin 重新生成；把 008 形状的基线留在 340 上，会得到代码无法满足的合同。

**附带查证**：origin 的 `AgentCheckpointWire.current_plan` 在 `managed.rs` 里只有声明、没有从 framework 取值的映射，分支则完全没有该字段——因此合并时它是可安全追加的纯 wire 字段，代价只是同步更新分支里构造该结构体的穷举字面量（机械改动）。

---

## 十一、`current_plan` 字段的源码级结论（2026-16 轮补记）

第 15 轮说该字段是“可安全追加的纯 wire 字段”，**需修正**。直接读两个 pin 的框架源码：

| pin | `src/state/mod.rs` 的 `AgentCheckpoint::current_plan` |
|---|---|
| `0087353f` | 存在（第 106 行），文档注释：「Legacy plan text retained for checkpoint decoding and Store round trips. ReactAgent neither restores this as task state nor writes new values; the revisioned Task graph is the planning authority.」 |
| `3409932` | **整个字段（含注释）已删除** |

SDK 侧在 `echo-sdk-host/src/core_profile/wire.rs` 用宏做字段映射：

```
structure!(state::AgentCheckpoint, AgentCheckpointWire,
           conversation_id, messages_json, current_plan, active_skills, …)
```

**因此在 pin 340 上，宏列表里的 `current_plan` 找不到源字段，会直接编译失败**，不是“顺手追加”的机械改动。

正确做法：从宏列表移除 `current_plan`，同时让 `AgentCheckpointWire` 的该字段带默认值（serde default / `Default`），保留跨语言合同形状。`core_profile_e2e.rs:5651` 已期望 `"current_plan": null`，与“340 下恒为 None”一致。

**性质**：framework 自己把该字段标为遗留、规划权威已迁到 revisioned Task graph。所以保留是“合同形状向后兼容”的选择，删除是“过时字段直接删”的选择——两者都能编译，属合同稳定性取舍，不是功能差异。

---

## 十二、163 个残差文件的归因（2026-17 轮补记）

按三个来源集合归因（退役 = `diff(0930eef,6cd2e1b)`；校准 = `diff(6cd2e1b,747be93)`；分支 = `diff(0930eef,83ae59e)`）：

**147 个内容不同**

| 归因 | 数量 |
|---|---|
| 仅分支/原生 | 78 |
| 退役 + 分支同时触及 | 66 |
| 仅退役 | 1 |
| 仅校准 | 1 |
| 退役 + 校准 + 分支 | 1 |

**16 个缺失**

| 归因 | 数量 | 处置 |
|---|---|---|
| 仅 origin 新增的 plan 文档 | 9 | 机械补入（`0930eef`/`83ae59e` 均无，只在上游出现） |
| 分支已删除的退役相关文件 | 7 | 保持删除 |

那 7 个是 `JwtClaims.java`、`JwtConfig.java`、`jwt_values.py`、`jwt_values.ts`、`a2a_host_fixture.py`、`lsp_diagnostic_server.py`、`lsp_fault_server.py`，删除发生在 `6cb12d3`。**已核实不是悬空删除**：origin/main 对两个夹具零引用；分支把 `test_jwt_values.py` 改写成了 `test_a2a_jwt_exports_are_retired`（断言 `not hasattr(sdk,"JwtConfig"/"JwtClaims")`）。

**结论**：144/147 属分支侧改动，说明已合并树内容主体与任务分支一致；origin 独有内容主要是 9 份文档 + 1 个纯校准文件。执行顺序可定为：以 `candidate-source` 为基 → 补 9 份文档 → 逐个复核 66 个双改文件 → 重生成合同 → 跑门禁。

---

## 十三、修正：候选树并不缺那 9 份文档（2026-24 轮）

第 12 节把“16 个缺失”拆成“9 份 plan 文档 + 7 个退役文件”，**9 份是误报**。

**原因**：`git ls-tree` 默认 `core.quotePath` 开启，把非 ASCII 路径输出成带八进制转义的引号串（`plan_01_\345\206\273...`）；脚本把该串直接当文件系统路径判断存在性，必然判为缺失。

**实测（`-c core.quotePath=false`）**：候选树对 `747be93` 追踪文件的结果是 **相同 1075 / 不同 147 / 缺失 7**，缺失的 7 个正是退役删除项，与候选自身回执的 `D=7` 吻合。

**因此**：

- 接收候选是**完整**的 merged tree（`747be93` + 原生消息 81 新增 + 147 修改 + 7 退役删除）；
- 既有“完整合同门禁自然退出 0”就是在这棵树上取得的，适用性不受影响；
- “还需补 9 份文档”这个工作项**不存在**，第 12 节的执行顺序应删去该步；
- 残差总量由 163 修正为 **154**（147 + 7）。

**教训**：与此前 `git cat-file -e` 只看 stdout 的错误同类——都是把 git 的输出格式当成文件系统事实。判定存在性必须用 returncode 或关闭路径转义。
