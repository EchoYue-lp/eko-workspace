# Skill 系统重构(2026-09-03)

status: implementation-complete / pending merge
owner: feature/skill-refactor worktrees(echo-agent + echo-agent-cli)

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
- `types.rs`:删 SkillMutationError 的 OperationConflict/BeforeCommit 变体。
- 删测试:extension_control/tests.rs 约 16 个 settlement 测试 +
  enabled_skills 内嵌 3 个 + state/tests.rs:2541 settled/repair_debt 断言;
  补简化行为测试(坏文件回退默认、enable/disable reconcile 生效、原子写)。
- 保留:subagent_control.rs 泛化 operation_identity(非 skill)、curator
  与 skill 自动创建闭环(panels.rs)。
- 新 ADR 取代 0032(0032 文件头加 superseded 标注)。
- TS 绑定:手改 generated/ 下受影响文件(勿全量再生成,会引入缺类型
  导入的无关破坏——已验证);前端 build 验证。

## 合并注意

- echo-agent 先合并,echo-agent-cli 后(依赖新 activate_skill 语义)。
- CLI worktree 的 Cargo.toml path 已临时指向 echo-agent worktree 绝对
  路径,合并前改回相对路径(grep worktrees /Users 零命中)。
- 磁盘紧张:本轮已两次耗尽,echo-agent 主 target 与 worktree target 均已
  清理;注意 df 检查。

## 最终状态(2026-09-03)

- echo-agent `feature/skill-refactor` @ f34a535(Phase 0,全套门禁过)
- echo-agent-cli `feature/skill-refactor` @ cf8390f(Phase 1-3)+ 9398ae0(Phase 4+5)
- echo-website 检查:被删 skill 为 CLI 侧内置,website 无引用(见合并说明)
- 已知残留:developer.rs 的 pre_dispatch 时序测试在高负载全量运行下偶发
  5s 超时(与本次改动无关,隔离与复跑均通过)
- 合并前必做:CLI worktree Cargo.toml 的绝对 path 改回相对路径
  (grep worktrees /Users 零命中);先合 echo-agent 再合 echo-agent-cli
