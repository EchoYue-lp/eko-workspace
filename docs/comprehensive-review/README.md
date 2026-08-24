# Echo Agent Comprehensive Review

> Status: shared review protocol and task catalog ready
> Baseline date: 2026-08-12
> Scope: `echo-agent` and `echo-agent-cli`
> Reviewers: three independent AI review tracks

This directory contains the task catalog and evidence protocol shared by three
independent AI review tracks. Each reviewer writes conclusions into its own
subdirectory so evidence, priorities, and recommendations can be compared
without accidental consensus or file collisions.

Reviewer directories:

- Codex: `codex/`
- ZCode-ds: `zcode-ds/`
- ZCode-glm: `zcode-glm/`

Authoritative cross-review application reconciliation:

- [`application-review.md`](application-review.md)
- [`application-model-comparison.md`](application-model-comparison.md) —
  per-model findings, overlap, disagreements, and final verdicts
- [`application-fix-plan.md`](application-fix-plan.md) — implementation,
  dependency, validation, and parallel-worktree plan

Framework remediation work:

- [`framework-remediation.md`](framework-remediation.md) — reconciled findings,
  ownership decisions, implemented fixes, and final validation evidence
- [`framework-finding-closure.md`](framework-finding-closure.md) — all 38
  framework reports, canonical counts, cross-model aliases, and dispositions
- [`framework-finding-ledger.md`](framework-finding-ledger.md) — mechanically
  verified accounting for 294 canonical IDs plus the 295th raw backlink

Current non-layer revalidation:

- [`cross-quality-remediation.md`](cross-quality-remediation.md) — 2026-08-16
  code-based revalidation of the baseline, cross-repository, quality, validation,
  dependency, documentation, and previously omitted website findings. This
  overlay does not reopen the completed framework (`F-*`) or application
  (`A-*`) layer reviews.

The three reviewer synthesis files remain immutable evidence inputs. When
their conclusions conflict, use the cross-review reconciliation rather than
selecting one reviewer report as canonical.

Do not write reviewer findings into a shared `reports/` directory. A later
cross-review synthesis may consume all three completed reviewer directories but
must keep backlinks to the original evidence.

## Required Reading For Every Review Task

Read only the following before opening task-specific source files:

1. Repository root `AGENTS.md` in full.
2. This file and the active reviewer's own index, for Codex `codex/README.md`.
3. `TASKS.md`, limited to the assigned task and its declared dependencies.
4. `REPORTING.md`.
5. The active reviewer's final reports of declared dependencies, if any.

Do not preload all historical design documents or all earlier review reports.
The task card identifies the minimum required material. Historical documents
are hypotheses until checked against current code.

## Review Invariants

- `echo-agent` is reviewed as an independent reusable framework. An API is not
  dead merely because `echo-agent-cli` does not call it.
- `echo-agent-cli` is EKO application policy. It does not use SQLite, and its
  local desktop threat model must not be replaced by a public multi-tenant Web
  service threat model.
- TUI, GUI, CLI, channels, and scheduled/background entry points target feature
  parity. Missing integration is a gap, not evidence that a surface does not
  need the capability.
- The project has Subagents, not Workers. Third-party fixed wire names are the
  only exception.
- Every cross-repository finding must classify the behavior as generic
  mechanism, EKO product policy, or adapter boundary.
- Definition, registration, reachability, and exercised behavior are separate
  claims and require separate evidence.
- Existing audit documents are not accepted as evidence without revalidation.
- Review tasks are read-only. Fixes belong to later implementation milestones.

## Artifact Model

For Codex, each atomic task produces one task report:

```text
codex/reports/tasks/<task-id>.md
```

Every individual validation execution produces a separate report:

```text
codex/reports/validations/<task-id>/<validation-key>--attempt-<NN>.md
```

Examples:

```text
codex/reports/tasks/F-RCT-02.md
codex/reports/validations/F-RCT-02/V01--attempt-01.md
codex/reports/validations/F-RCT-02/V01--attempt-02.md
codex/reports/validations/F-RCT-02/V01-02--attempt-01.md
```

`V01--attempt-02` is a retry of `V01--attempt-01`; `V01-02--attempt-01` is the
first attempt of a distinct `V01-02` subcase. New reports must declare
`Schema: validation-v2`, the exact validation key, and a positive attempt
number in their headers. Historical validation filenames remain immutable and
are not retroactively reinterpreted. Static source tracing, grep-based
reachability checks, compilation, tests, GUI smoke tests, and external
reference checks all count as validations and each get a report. Run
`scripts/verify-validation-lineage.sh` before accepting new reports.

The task report contains findings and links to validation reports. It must not
duplicate full command output or large traces. Phase synthesis reports consume
task reports, not raw conversations.

## Status Values

Tasks use exactly these states:

- `pending`: no review has started.
- `in_progress`: an assigned task is actively reviewing.
- `needs_evidence`: analysis exists but required validation is missing.
- `blocked`: an external prerequisite prevents progress.
- `complete`: task report exists and every declared validation has a report.
- `superseded`: replaced by a more precisely scoped task; replacement is linked.

Validation reports use `passed`, `failed`, `inconclusive`, or `not_run`.
`not_run` is allowed only with a concrete environmental reason and does not make
the parent task complete unless the task card explicitly marks that validation
as conditional.

## Context Budget

Each atomic task should normally stay within all of these limits:

- one principal question;
- one subsystem or one end-to-end path;
- about 3,000 to 8,000 production source lines inspected deeply;
- no more than 15 primary production files, excluding small type definitions;
- no more than 5 dependency reports loaded;
- one fresh Codex task unless debugging a strongly coupled failed validation.

If a task exceeds a limit, stop and split it in `TASKS.md`. Do not compensate by
skimming files. Large files such as the ReAct engine and EKO TaskRuntime are
reviewed by behavior slices, not as single directory-sized tasks.

## Cross-Task Workflow

1. Claim one task by setting it to `in_progress` in `TASKS.md`.
2. Record both repository commits and the task scope in the task report.
3. Perform the repository-wide duplicate/reachability search required by the
   task before proposing new abstractions or deletion.
4. Create one validation report immediately after each validation execution.
5. Write findings with source locations and validation links.
6. Mark the task complete only when the completion rule in `REPORTING.md` holds.
7. Update the task status in the reviewer index; Codex uses `codex/README.md`.
8. Start a fresh Codex task for the next weakly coupled task.

## Shared Baseline

Baseline observed on 2026-08-12:

| Repository | Branch | Baseline commit | Approximate source size |
|---|---|---|---:|
| `echo-agent` | `main` | `9b0e0fa` | 490 Rust files, about 183k production Rust lines |
| `echo-agent-cli` | `main` | `b3b2e81` | 200 Rust and 377 TS/TSX files, about 130k production lines |

The baseline commits are planning anchors, not a requirement to freeze active
development. Each report records the actual reviewed commits. If relevant code
changes after a report, the phase synthesizer marks it stale and schedules a
targeted revalidation instead of silently carrying the conclusion forward.

The authoritative task list and dependency graph are in `TASKS.md`. Per-reviewer
progress belongs in the reviewer directory and must not be inferred from another
AI's results.

## Final Deliverables

The Codex review is complete only when these synthesis documents exist:

- `codex/reports/synthesis/framework-review.md`
- `codex/reports/synthesis/application-review.md`
- `codex/reports/synthesis/cross-repository-review.md`
- `codex/reports/synthesis/quality-and-validation-review.md`
- `codex/reports/synthesis/iteration-roadmap.md`

The roadmap must order work by correctness and data integrity first, then
authority convergence and layering, surface parity, maintainability,
performance, and documentation. Every roadmap item links back to findings and
validation evidence and includes repository ownership, dependency order,
deletion target, regression tests, and acceptance criteria.
