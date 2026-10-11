# SDK 独立仓迁移验收

本文件记录跨仓库迁移的阶段验收；SDK 长期产品事实源位于
[`echo-agent-sdk`](https://github.com/EchoYue-lp/echo-agent-sdk)，不在 superproject。
最终四仓快照由本文件所属提交的 gitlink 唯一记录。

## 交付边界

SDK 仓拥有完整 Protocol、Host、accepted contracts、TypeScript/Python/Java SDK、
SDK 文档、ADR、生成器、示例和 CI。Framework extraction
[PR126](https://github.com/EchoYue-lp/echo-agent/pull/126) 已删除可编辑 SDK 产品与 SDK CI；
框架保留通用 ACP adapter、运行时能力和指向独立 SDK 的链接。

SDK source import 与过滤历史通过双父 merge 保留，详见独立仓
[`MIGRATION-SOURCE.md`](https://github.com/EchoYue-lp/echo-agent-sdk/blob/main/MIGRATION-SOURCE.md)。
SDK Host 固定已推送的 framework extraction revision
`27c7701e1eb116db1076da7f84bb68898544a44c`；该 pin 与顶层 framework `95ec66c6` gitlink
分别维护。Protocol 的 Cargo DAG 不依赖任何 echo framework crate；SDK 可以独立检出构建。

## 兼容验收

阻断兼容面只包括明确接受的跨语言合同。`source-contract.json` 的输入只有
accepted external contract 和 accepted facade catalog；完整 Rust public inventory、
Host-only catalog、framework revision 与 Cargo.lock 不进入协商摘要。

三语言 catalog 测试直接读取 accepted artifact。Scope 数量、intrinsic membership hash
只报告遥测；分类样例不要求非 external identity 必须存在。分类、alias、schema 和生成器
一致性仍受结构校验。内存 mutation 测试证明非 external identity 的变化保持摘要不变，
accepted identity 的变化则确定性改变摘要。

Host 的结算计数器用 CAS 保留原有饱和递减与内存顺序，适配 Rust 1.99 lint；不增加第二套
Agent/Run/Task/Subagent 状态或结算权威。

## 关联项目

EKO 的 SDK 引用已经由 [CLI PR6](https://github.com/EchoYue-lp/echo-agent-cli/pull/6)
交付。官网 [PR2](https://github.com/EchoYue-lp/echo-website/pull/2) 指向独立 SDK，源清单
按已提交的 framework `95ec66c6`、CLI `f57b90a2` 校验。未提交且被忽略的框架 ADR0048
没有进入官网镜像；用户原始文件保持原状。

顶层 `.gitmodules` 新增第四项 `echo-agent-sdk`，路径为 `echo-agent-sdk`，remote 为
`git@github.com:EchoYue-lp/echo-agent-sdk.git`。README、协作规则和 ADR 同步四仓职责。
本次新增 SDK 归属说明与独立 SDK AGENTS.md 一致；没有修改 framework/CLI 的既有规则。

## 验证记录

SDK 候选 [PR4](https://github.com/EchoYue-lp/echo-agent-sdk/pull/4) 为
`eca01e11440f75fd5ebf8b8e8c6baea73ab5edd5`，独立实现复审 Critical / Important / Minor 均为 0。
本地 fmt、Clippy 双档、workspace all-target/all-feature tests、no-default build、Host E2E、
最终 78 项 inventory tests（含两项 mutation）、contract 与完整 telemetry 重生成均通过。
28 个 rustdoc profiles 的 95 个阻断制品与提交一致，telemetry drift 为 0。
依赖 audit 与 deny 策略通过，既有 multiple-version 警告配置保持原状。
Strict semantic snapshot/change evidence 校验通过；源码摘要为
`18de34be70afdcf30950277f1bc16142a058e3d4da6905fa4e1eee36f2a7663b`。

SDK 已从远端独立检出候选 `eca01e1`，旁边没有 framework 或 EKO 目录；workspace
no-default-feature build、exact framework provenance 与 strict semantic snapshot 通过。
只复用 Cargo 编译缓存，没有使用同级仓库源码。

SDK PR4 在 [8 项 CI 全绿](https://github.com/EchoYue-lp/echo-agent-sdk/actions/runs/38112834392)
后合入 main `30d7e219cc7d3aef0f75a19217c82898c363f50d`；合并 tree 与已验收候选完全
一致，初始 import `ea21dfc` 与过滤历史 tip `28d6709` 都仍是 main 祖先。
顶层候选已从远端重新检出并依次初始化四个 submodule，精确匹配四个 gitlink，
全部 clean。初始化使用 reference cache 后执行 dissociate；superproject 与四个子仓
均不保留对象 alternates，不依赖原工作区目录继续存在。SDK 608 个主要产品路径可读，
初始 import 与过滤历史祖先均可恢复。验收日志为
`/private/tmp/eko-sdk-workspace-reconstruction.log`。
精确 SDK main `30d7e21` 的 [8 项 CI](https://github.com/EchoYue-lp/echo-agent-sdk/actions/runs/38113904024)
全部通过。最终四仓 gitlink、实际检出与远端 main 一致，全部引用已经推送的提交。
独立复审与跨仓验收闭合后，顶层通过
[PR20](https://github.com/EchoYue-lp/eko-workspace/pull/20) 交付；保留项只有本文件明确
排除的发布制品和后续全能力语义治理。

官网完整 `npm run verify`、source-aware manifest 校验通过：39 unit tests、286 static
routes、15 Playwright tests，保留 1 项既有平台 skip。独立 reviewer 逐项核对
270 条 framework 与 118 条 CLI source revision/hash，全部一致。PR 两项检查与精确
website main `0dc2dc5097e787dc9ba26f7919074c82db718aaa` 的 CI 均通过。

Framework `95ec66c6` 精确提交的 7 项 CI 与 EKO `f57b90a2` 精确提交的 4 项 CI 已核对
全绿；本次未修改这两个交付快照的 runtime 源码。

SDK 三语言源码、示例与 live Host smoke 通过：TypeScript 157 tests、Python 177 tests，
Java Maven tests 和运行示例通过。Examples 进入语言 gate 和 Rust all-target tests。
本次未改变公开语言 API，无须增加示例接口；正式 SDK 文档与官网引用已经同步。

## 保留与后续

这次验收是源码与仓库迁移，不是 npm/PyPI/Maven 或二进制发布。SDK 全能力/生命周期
语义治理仍使用 open baseline，framework 治理中单列的 SDK #120 不在此自动关闭。
旧 CLI task worktree、SDK 修复前 stash、原顶层未提交/未跟踪材料均保留，不作为本次
交付输入。未对这些工作区做 reset、clean 或删除。
