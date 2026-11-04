# SDK 接收候选相对当前 HEAD 的真实差值（2026-11-04 实测）

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
