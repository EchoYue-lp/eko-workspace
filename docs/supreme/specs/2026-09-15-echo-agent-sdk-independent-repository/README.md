# SDK独立仓库迁移的历史来源

本目录保留SDK `MIGRATION-SOURCE.md` 所述superproject来源定位。这里记录2026-09至2026-10迁移时的原始资料；当前SDK产品事实由独立SDK仓的正式文档、合同与ADR维护。旧计划原始状态不代表当前执行入口。

原primary中的8份文件按原字节保存为历史档案，下面可直接读取；原SHA、mode、mtimeNs、已交付后继和恢复方法见[资料对账](../../plans/2026-10-24-workspace-maintenance/reconciliation.md)。

| 原目录内路径 | 历史原始文件 |
| --- | --- |
| `design.md` | [原始字节](../../plans/2026-10-24-workspace-maintenance/archive/originals/docs/supreme/specs/2026-09-15-echo-agent-sdk-independent-repository/design.md.original) |
| `plans/delivery-map.md` | [原始字节](../../plans/2026-10-24-workspace-maintenance/archive/originals/docs/supreme/specs/2026-09-15-echo-agent-sdk-independent-repository/plans/delivery-map.md.original) |
| `plans/plan_01_独立交付完整SDK产品.md` | [原始字节](../../plans/2026-10-24-workspace-maintenance/archive/originals/docs/supreme/specs/2026-09-15-echo-agent-sdk-independent-repository/plans/plan_01_独立交付完整SDK产品.md.original) |
| `plans/plan_02_建立Framework迁移基线.md` | [原始字节](../../plans/2026-10-24-workspace-maintenance/archive/originals/docs/supreme/specs/2026-09-15-echo-agent-sdk-independent-repository/plans/plan_02_建立Framework迁移基线.md.original) |
| `plans/plan_03_先迁移SDK源码.md` | [原始字节](../../plans/2026-10-24-workspace-maintenance/archive/originals/docs/supreme/specs/2026-09-15-echo-agent-sdk-independent-repository/plans/plan_03_先迁移SDK源码.md.original) |
| `plans/plan_04_Framework移除SDK资产.md` | [原始字节](../../plans/2026-10-24-workspace-maintenance/archive/originals/docs/supreme/specs/2026-09-15-echo-agent-sdk-independent-repository/plans/plan_04_Framework移除SDK资产.md.original) |
| `plans/plan_05_SDK源码连续性.md` | [原始字节](../../plans/2026-10-24-workspace-maintenance/archive/originals/docs/supreme/specs/2026-09-15-echo-agent-sdk-independent-repository/plans/plan_05_SDK源码连续性.md.original) |
| `plans/plan_06_SDK框架Pin与合同收敛.md` | [原始字节](../../plans/2026-10-24-workspace-maintenance/archive/originals/docs/supreme/specs/2026-09-15-echo-agent-sdk-independent-repository/plans/plan_06_SDK框架Pin与合同收敛.md.original) |

旧Plan01/02的abandoned、Plan03/04/05的completed及Plan06的ready原样保存；本索引不改其lifecycle。Plan06旧175487目标和后继27c7701e/f7bac62e的关系按维护对账解释，不将历史pin冒充当前Host依赖。
