# B-DOC-01: Historical audit and design drift index

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean

## Question

Which existing audit/plan claims still point at current code and which need
targeted revalidation?

## Scope

- `echo-agent/AUDIT_REPORT.md` (668 lines, 12 security + 9 bug findings)
- Root `docs/MASTER-PLAN.md` (5 product lines, M1-M13 milestones)
- `echo-agent-cli/docs/MASTER-PLAN.md` (~40 milestones)
- Architecture docs in both `docs/` trees

## Out Of Scope

- Re-reviewing code behind every historical claim (this is a drift INDEX)
- Full code-level revalidation of each audit finding (deferred to F-*/A-* tasks)

## Inputs

- `AGENTS.md` (product positioning, layering rules)
- B-ARCH-01 task report (facade structure)
- B-PATH-01 task report (entry points)
- Q-STA-01 task report (panic/unsafe/dead-code audit)
- F-CORE-01 task report (GLOBAL_EVENT_BUS dead)
- F-MEM-01 task report (FileStore corrupt-file handling)

## Layering Decision

This task spans both repositories and all documentation layers. The
classification uses cross-referenced evidence from completed review tasks
rather than independent code re-inspection.

## Current Path

The documentation landscape (from B-BASE-01 V01):
- Root: `docs/MASTER-PLAN.md`, `docs/PROJECT-ANALYSIS.md`
- echo-agent: `AUDIT_REPORT.md`, `docs/{en,zh,knowledge}/`
- echo-agent-cli: `docs/MASTER-PLAN.md`, 30+ design/audit docs

## Findings

### B-DOC-01-P2-01: 10 of 21 AUDIT_REPORT findings need code-level revalidation

- Priority: P2
- Confidence: high
- Layer: framework (most findings), application (some)
- Evidence: `AUDIT_REPORT.md` sections 1-2; V04 classification table
- Reachability: findings reference code paths that still exist; classification
  is based on cross-referencing with completed review tasks
- Expected invariant: audit findings should be current or explicitly marked
  as resolved in the document itself
- Observed behavior: the AUDIT_REPORT has no resolution status markers. 5
  findings confirmed fixed (by Q-STA-01), 3 are by-design for local desktop
  (per AGENTS.md), 10 remain unresolved and need revalidation
- Impact: reviewers and developers cannot tell which audit findings are still
  active without reading the code
- Root cause: audit was a point-in-time snapshot; no resolution tracking was added
- Direction: add resolution status to each finding in AUDIT_REPORT.md, or
  create a separate RESOLUTIONS.md
- Regression validation: n/a (documentation)
- Validation reports: [V04](../validations/B-DOC-01/V04-01.md)

### B-DOC-01-P2-02: AGENTS.md references echo-agent-eval which does not exist

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: AGENTS.md "三个项目的定位" table lists `echo-agent-eval` as a
  submodule of echo-agent-cli; B-BASE-01 V01 confirmed no such directory
- Reachability: AGENTS.md is the first file every reviewer/agent reads
- Expected invariant: AGENTS.md should accurately describe the project structure
- Observed behavior: `echo-agent-eval` is listed but does not exist at b3b2e81
- Impact: reviewers may waste time looking for a non-existent module
- Root cause: echo-agent-eval was removed or never created
- Direction: remove the reference from AGENTS.md or create the module
- Regression validation: n/a
- Validation reports: [V03](../validations/B-DOC-01/V03-01.md)

### B-DOC-01-P3-01: MASTER-PLAN milestones M1-M13 claims verified as current

- Priority: P3
- Confidence: high
- Layer: both
- Evidence: V02 verified one code anchor per milestone; all present
- Expected invariant: completed milestones should have their code artifacts
- Observed behavior: all sampled milestone claims (drive_chat, EventEnvelope,
  TaskRuntime, RuntimeDagExecutor, etc.) resolve to real code
- Impact: none (positive confirmation)
- Validation reports: [V02](../validations/B-DOC-01/V02-01.md)

## Validation Matrix

| ID | Claim | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Document-to-symbol link check | yes | passed | [V01-01](../validations/B-DOC-01/V01-01.md) |
| V02 | Completed-milestone code anchors | yes | passed | [V02-01](../validations/B-DOC-01/V02-01.md) |
| V03 | Obsolete path/term search | yes | passed | [V03-01](../validations/B-DOC-01/V03-01.md) |
| V04 | Unresolved historical-finding index | yes | passed | [V04-01](../validations/B-DOC-01/V04-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| AUDIT_REPORT 1.9 (unsafe env mutation) | fixed | Q-STA-01: 2 guarded unsafe sites |
| AUDIT_REPORT 2.1-2.3 (RwLock/Semaphore panics) | fixed | Q-STA-01: 0 production panic/expect |
| AUDIT_REPORT 1.5 (WebSocket no auth) | by-design | AGENTS.md: local desktop threat model |
| AUDIT_REPORT 1.1 (SQL injection) | current (low priority) | Database tool exists; local threat model |
| AUDIT_REPORT 6.2 (dead code) | current | Q-STA-01-P2-01: ~50 production allow(dead_code) |
| AGENTS.md: echo-agent-eval exists | stale | B-BASE-01: no such directory |
| MASTER-PLAN M1-M13 complete | current | V02: code anchors verified |
| MASTER-PLAN "one lifecycle, multiple triggers" | current | B-PATH-01: all paths through bootstrap |

## Coverage And Uncertainty

- 10 audit findings need code-level revalidation by downstream tasks (listed in V04)
- The CLI MASTER-PLAN's ~40 milestones were sampled, not exhaustively verified
- Architecture docs under `docs/system-deep-dive/` were not inspected

## Handoff

- **F-SEC-01** should revalidate: audit 1.2 (SSRF), 1.3 (sandbox), 2.5 (secret redaction)
- **F-EXT-02** should revalidate: audit 1.6 (symlink), 1.11 (git injection), 2.9 (process kill)
- **F-INT-01** should revalidate: audit 2.4 (MCP stdio drops)
- **F-PLG-01** should revalidate: audit 1.8 (plugin git clone)
- **F-EVO-01** should revalidate: audit 1.4 (eval runner scope)
- This report becomes stale if audit findings are resolved without updating the AUDIT_REPORT
