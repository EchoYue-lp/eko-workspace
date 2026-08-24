# Q-DOC-01: Current public and operator documentation validation

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean (both repositories)
> Note: validation faces V01-V04 were executed by the reviewing session
> before a network failure; V05 (canonical aggregation) and this report were
> synthesized by the main reviewer from the four completed validation
> reports. No command results were invented.

## Question

Do README, feature/config references, examples, EKO setup docs, and
architecture claims match reviewed code and executable commands?

## Scope

- Both repositories' READMEs, docs trees, examples catalogs.
- Feature/config reference tables (`28-config-reference.md` family).
- CLI architecture docs and stale-terminology search.
- Command/example execution sampling (V02).

## Out Of Scope

- Full example compilation across feature combinations — Q-FW-02.
- Framework gate, frontend gate, GUI matrix — Q-FW-01 / Q-CLI-01 / Q-WEB-01 /
  Q-GUI-01 (all complete).

## Inputs

- Root `AGENTS.md`, shared `REPORTING.md`/`TASKS.md`, `zcode-ds/README.md`.
- Dependency reports: `B-DOC-01`, `F-API-01`, and the V05 drift findings of
  completed F-/A-/X- task reports.
- The four completed validation faces (V01-V04) and the synthesized V05.

## Layering Decision

- Documentation is the product's operator surface; drift findings are
  classified by owning layer (framework docs vs EKO docs vs shared).

## Findings

No P0/P1/P2 findings — all drift is P3 (documentation-only, no runtime
impact). Eleven new P3 findings (all high confidence, application/framework
docs):

- **Q-DOC-01-P3-01** — link inventory: broken/nonexistent doc references
  across both repos (V01).
- **Q-DOC-01-P3-02** — CLI README data-root/config paths claim `~/.echo-agent`
  while the runtime uses `~/.eko`; LICENSE references (V01/V03/V04).
- **Q-DOC-01-P3-03** — workspace tree claims and example counts in READMEs
  disagree with the actual catalog (V01).
- **Q-DOC-01-P3-04** — phantom catalog rows: `demo14`/`demo16`/`demo63` in
  `examples/README.md` acceptance list have no files (V02/V04).
- **Q-DOC-01-P3-05** — `demo37` documented command missing
  `--features workflow` (V02).
- **Q-DOC-01-P3-06** — `demo32` acceptance classification vs live-LLM
  dependency (V02).
- **Q-DOC-01-P3-07** — README models/embedding misplacement (V03).
- **Q-DOC-01-P3-08** — `28-config-reference.md` feature table partial drift
  (V03).
- **Q-DOC-01-P3-09** — `28-config-reference.md` phantom feature rows
  (`self-reflection`, `plan-execute`, `sqlite`) and missing real features
  (`a2a`/`lsp`/`telemetry`/`handoff`/`topology`/`channels`/`statistics`/
  `eval`/`improve`/`testing`/`content-guard`/`project-rules`/`rag`) (V04).
- **Q-DOC-01-P3-10** — `29-long-running-tasks.md` and
  `knowledge/agent-patterns.md` instruct nonexistent `execute_with_planning()`
  and `src/agents/plan_execute/mod.rs` (V04).
- **Q-DOC-01-P3-11** — CLI `docs/architecture.md` SQLite storage claims
  (":160 会话历史持久化（SQLite）", ":190-191 SQLite（默认）", ":222 记忆存储
  (SQLite)") contradict the AGENTS.md no-SQLite invariant and X-INV-01's
  zero-SQLite finding (V04).

Canonical aggregation (V05): previously filed doc-drift findings carried by
canonical ID — B-DOC-01, A-TSK-02-P3-03 (demo22_plan_execute), F-EVO-01-P3-03
(echo-agent-eval/CritiqueStore), A-FE-01-P3-02 (generated/*.ts prettier
drift), A-BOOT-01-P3-06/A-CFG-01-P3-01/A-PROJ-01-P3-02 (stale sqlite
comments), A-CHAT-01-P3-01 ("worker" term in PROJECT-ANALYSIS), F-MAC-01-P3-05
(demo25_macros), F-INT-01-P3-03, F-INTENT-01-P3-04, F-SUB-02-P2-03 (mailbox
doc-vs-code), A-OUT-01-P3-01 (dead-code facade doc), F-API-01-P2-01.

## Validation Matrix

| ID | Face | Required | Status | Report |
|---|---|---|---:|---|
| V01 | Link/path checks | yes | passed | [V01](../validations/Q-DOC-01/V01-01.md) |
| V02 | Command/example execution sampling | yes | passed | [V02](../validations/Q-DOC-01/V02-01.md) |
| V03 | Feature/config option matrix | yes | passed | [V03](../validations/Q-DOC-01/V03-01.md) |
| V04 | Stale terminology/architecture search | yes | passed | [V04](../validations/Q-DOC-01/V04-01.md) |
| V05 | Canonical drift aggregation | yes | passed | [V05](../validations/Q-DOC-01/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| AGENTS.md no-SQLite invariant | current (docs contradict it, code complies) | Q-DOC-01-P3-11 + X-INV-01 zero-SQLite |
| B-DOC-01 historical audit claims | current/stale per item, targeted revalidation done | V05 aggregation |
| "67 registered tools" (README.md:181,389) | not re-verified (counting requires tracing echo_tools registration) | deferred to Q-FW-02 |

## Coverage And Uncertainty

- Link *semantic* correctness only sampled on three faces (V01).
- Feature compile truth not re-verified here — Q-FW-02 owns it.
- All drift findings are P3 (documentation-only); no P0/P1/P2.

## Handoff

- Q-FW-02: verify tool counts and per-feature compile claims referenced in
  docs (P3-09's missing/phantom feature rows).
- Roadmap: documentation cleanup batch (P3-01..11) can be one atomic
  docs-commit milestone with the V05 canonical list as checklist.
- This report becomes stale when any referenced doc or its subject code
  changes.
