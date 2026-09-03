# Skill 系统重构(2026-09-03)

status: locally merged / pending remote delivery and superproject gitlinks
owner: main branches(feature worktrees removed)

## 背景与决策

2026 年业界坐标:Claude Code 将 slash 并入 skills(用户/模型双触发)、
Codex/ChatGPT 官方支持 skills、Agent Plugins 1.0 标准(2026-08,SKILL.md
原样保留)。EKO 三层渐进机制不动,修调用模型/分发/管制三层。

已确认决策:内置 39→24(删 4 通用型 + 11 vendored Anthropic 示例);
baseline 常驻注入 4→1(仅 verification-before-completion);durable 管制
机器全砍;Agent Plugins 只留安装识别口。

## 进度

| Phase | 内容 | 状态 |
|---|---|---|
| 0 | activate_skill 未安装报错(框架) | ✅ echo-agent f34a535,全套门禁过 |
| 1 | builtin_skills_root 运行时解析($EKO_SKILLS_ROOT→resource dir→源码树)+ tauri resources | ✅ cf8390f |
| 2 | TUI /skill <name> [instructions] + GUI SkillCommand::Activate + SkillsPanel 按钮 | ✅ cf8390f |
| 3 | 目录 39→24、默认启用 8→5、baseline 4→1、workspace 路由名单 | ✅ cf8390f |
| 4 | 移除 durable 管制机器(净删 ~2200 行,ADR 0036) | ✅ echo-agent-cli 9398ae0 |
| 5 | install 识别 plugin.json 留口(仅 skills 面) | ✅ 9398ae0 |
| — | 完整门禁 + 文档同步 | ✅ clippy 双门禁/全量测试/GUI 矩阵/frontend 全绿;ADR 0036 双语、0032 部分取代标注、project-status、CHANGELOG、skill-sync 运维文档已更新 |

## Phase 4 执行要点(新窗口读这里)

- `enabled_skills.rs`:删 SkillOperationIdentity/SkillArtifactSyncDebt/
  SkillRepairTargetDebt/SkillRepairDebt 4 结构、EnabledSkillsConfig 的
  desired/settled_generation/content_identity/operation_identities/
  repair_debt 5 字段、serde u64_string 机器(L17-70)、record_operation/
  operation/set_repair_debt;保留 {category,enabled,baseline}+原子写;
  解析失败回退默认(fail-open,L478-503 改语义,
  policy_contract.rs:215 测试同步改)。
- `extension_control/skills.rs`:删 L56-810,保留 desired_skill_entries
  (L811)/reconcile_target_skills(L838)/skill_entry(L16)/简化读写。
- `service.rs`:enable/disable/install/uninstall/sync 改直写+reconcile,
  删 settle_skill_mutation_owned/refresh_with_operation CAS 部分
  (set_skill_enabled_with_operation L857、settle L902、refresh L1040、
  reconcile L1101+、install L1330、uninstall L1481、sync L1629、
  publish_curated_skill L591)。
- `types.rs`:删不再成立的 OperationConflict 与 durable receipt 字段；保留
  BeforeCommit/SettlementTask 以区分原子写前失败和 detached task 失败。
- 删测试:extension_control/tests.rs 约 16 个 settlement 测试 +
  enabled_skills 内嵌 3 个 + state/tests.rs:2541 settled/repair_debt 断言;
  补简化行为测试(坏文件回退默认、enable/disable reconcile 生效、原子写)。
- 保留:subagent_control.rs 泛化 operation_identity(非 skill)、curator
  与 skill 自动创建闭环(panels.rs)。
- 新 ADR 取代 0032(0032 文件头加 superseded 标注)。
- TS 绑定:手改 generated/ 下受影响文件(勿全量再生成,会引入缺类型
  导入的无关破坏——已验证);前端 build 验证。

## 合并注意(已执行)

- echo-agent 已先 squash 到 `main` @ `ac00815`,echo-agent-cli 后 squash 到
  `main` @ `f194c43`;website 同步到 `main` @ `35490e8`。
- Cargo.toml 已恢复相对路径,`rg "worktrees|/Users/" --glob Cargo.toml` 零命中。
- 两个 `feature/skill-refactor` worktree 与分支、CLI 临时 framework symlink 已删除；
  Cargo target 按磁盘规则清理,最终约 90 GiB 可用。

## 最终状态(2026-09-03)

- echo-agent `main` @ `ac00815`(Phase 0 + review fixes)
- echo-agent-cli `main` @ `f194c43`(Phase 1-5 + review fixes)
- echo-website `main` @ `35490e8`(绑定上述 SHA 的 docs manifest + 静态发现产物)
- 独立 reviewer 最终 `pass`,行动项 0。
- framework:fmt、双 Clippy、workspace all-target/all-feature tests、no-default 与
  12 个独立 feature check 全绿。
- CLI:fmt、双 Clippy、workspace all-feature tests(1523 app-core + 276 CLI)、
  no-default、GUI check + 207 tests、46 对/36 ADR parity 全绿；frontend
  Prettier、243 tests、production build 全绿。
- website:format/lint/shell/site/discovery/source docs check、39 tests 与 production
  build 全绿；Playwright e2e 因缺浏览器未执行页面断言,用户明确裁决静态站可豁免。
- 未执行:push；依据 AGENTS.md,子提交未推送前不提交顶层 gitlink。

## 合并前 Review 补漏

- GUI 激活改为按真实 conversation ID 获取 exact pooled Agent,TUI/GUI 不再写到
  workspace seed；缺当前会话时按钮禁用,后端空 ID 显式报错。
- Tauri host 用官方 platform resource resolver 注入 bundled root；真实 debug `.app`
  已确认 `Contents/Resources/skills` 含 24 个 `SKILL.md` 与 baseline marker。
- `EnabledSkillsConfig::load` 统一 fail-open 并归一旧 4-baseline 配置；primary 与
  pooled conversation Agent 共用同一 baseline 注入函数。
- Agent Plugins 安装复用 framework manifest/Skill validator,全量预检后包级原子
  替换、启用全部 Skill,Git source record 保存精确 `skills/<name>` subdir。
- 删除 Skill receipt 的不可达 `Committed` 状态,修正 install/uninstall/sync
  `idempotent` 语义、重复 TS union、正式双语文档、workspace 文案和 docs parity 清单。
- 删除 `webapp-testing` Skill；后续网页操作/测试直接使用 Playwright MCP,不维护
  自研 Website Skill。
