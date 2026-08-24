# <TASK-ID>: <Title>

> Status: in_progress
> Reviewer: <model/harness or person>
> Review date: YYYY-MM-DD
> `echo-agent` commit: <hash or not-applicable>
> `echo-agent-cli` commit: <hash or not-applicable>
> Worktree state: <clean or concise dirty paths>

## Question

State the one principal question this task answers.

## Scope

Primary source paths and behaviors inspected.

## Out Of Scope

Related areas deliberately deferred to named task IDs.

## Inputs

- Required repository documents read.
- Dependency task reports read.
- Historical documents treated as hypotheses.

## Layering Decision

When relevant, classify generic mechanism, EKO product policy, and adapter
boundary. Include repository-wide duplicate-search terms and results.

## Current Path

Describe the verified call graph, data flow, identities, state owners, and
terminal/recovery points. Use source anchors.

## Findings

### <TASK-ID>-P<0-3>-01: <Finding title>

- Priority: P?
- Confidence: high | medium | low
- Layer: framework | application | adapter
- Evidence: `absolute/path:line`
- Reachability: <definition -> registration -> live caller>
- Expected invariant: <what must hold>
- Observed behavior: <what current code does>
- Impact: <concrete consequence>
- Root cause: <underlying cause>
- Direction: <fix direction and deletion target>
- Regression validation: <required test or scenario>
- Validation reports: [V01](../validations/<TASK-ID>/V01-01.md)

Write `No findings.` when appropriate. Do not omit the section.

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search | yes | pending | - |
| V02 | Registration and reachability | yes | pending | - |
| V03 | Invariant and edge cases | yes | pending | - |
| V04 | Targeted executable check | conditional | pending | - |
| V05 | Historical-document drift | conditional | pending | - |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `<document heading>` | current/fixed/stale/regressed | `path:line` or validation link |

## Coverage And Uncertainty

List code not inspected, validations not available, environmental limits, and
claims that remain uncertain.

## Handoff

- Conclusions downstream tasks may rely on.
- Reports they must read.
- Conditions that make this report stale.
- Follow-up task IDs, without implementing fixes in this review task.
