# X-INV-01: Repository invariant audit

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean (echo-agent-cli git
> metadata holds one stale prunable worktree — V06 observation)

## Question

Do both repositories obey Subagent-only terminology, CLI no-SQLite, no
parallel task CRUD, panic safety, UTF-8 safety, and relative path rules?

**Answer: five of six invariants hold. The UTF-8 safety invariant fails on
two live framework paths (deterministic panics, both reproduced with exit
101): `parse_pdf_date` in the pdf tool (`echo-tools/src/pdf.rs:225-227`)
and `extract_number_near_key` in the eval runner (`src/eval/runner.rs:728`)
— findings P1-01/P1-02 below. Terminology, no-SQLite, no-parallel-CRUD,
panic safety (macro level), and relative-path invariants hold with zero
violations; the only residue is stale doc-comment wording and stale
documentation already archived by prior tasks.**

## Scope

- **V01** Subagent-only terminology: full-tree `worker`/`Worker` grep over
  every tracked file of both repositories (Rust, TS/TSX, JSON, TOML, docs,
  frontend), each hit classified.
- **V02** CLI no-SQLite: CLI manifests, `Cargo.lock`, resolved dependency
  tree (`cargo tree --locked -e normal,build,dev`), source grep; framework
  `sqlite` feature excluded as a legitimate framework option (F-MEM-02).
- **V03** No parallel task CRUD: `todo_write`, `plan_create`, `plan_patch`,
  `plan_execute`, `TodoWriteTool`, `Plan*Tool`, `search_todo`, `find_todo`,
  global todo stores across both repositories.
- **V04** Panic safety: complete classification of `unwrap()`/`expect()`/
  `panic!`/`unreachable!` in both repos (test vs production vs doc),
  sampled direct indexing on live paths (security.rs, data.rs,
  data_quality.rs, compression, CLI CLI/TUI/Tauri commands).
- **V05** UTF-8 safety: all range-slice patterns (`s[..n]`, `s[n..]`,
  `s[..=n]`, `&s[..n]`) in production regions of both repos, with two
  standalone reproductions of the live violations.
- **V06** Relative paths: all Cargo manifest `path` fields, `worktrees|/Users/`
  residue in tracked files, both `Cargo.lock`, frontend package.json, git
  worktree list, .gitmodules.

## Out Of Scope

- Full unguarded-inventory / overflow / unsafe / dead-code matrix — Q-STA-01
  (pending in this track); the same commit anchor applies.
- Framework `sqlite` feature correctness — F-MEM-02.
- Task-model authority and adapter conformance — F-TSK-01/02, A-TSK-01..06,
  X-TSK-01.
- The `codex/` and `zcode-glm/` reviewer tracks (independence rule).

## Inputs

- Root `AGENTS.md` in full (terminology, no-SQLite, no-parallel-CRUD,
  panic/UTF-8 rules, worktree relative-path rule, layering gates).
- `docs/comprehensive-review/README.md`, `REPORTING.md`, `TASKS.md`
  (X-INV-01 card), `zcode-ds/README.md`, both report templates.
- Dependency reports read: `B-BASE-01` (build topology, manifest/lock
  inventory, zero worktrees/`/Users/` in manifests, CLI no-sqlite
  confirmation), `A-TSK-02` (forbidden-CRUD search evidence, P3-03),
  `F-TSK-01` (duplicate model search), `F-SEC-01` (V04 panic scan),
  `A-PROJ-01` (P3-01/P3-02), `A-CFG-01` (P3-01), `A-BOOT-01` (P3-06),
  `F-EXT-02` (P1-01), `F-EXT-03` (P1-03). Q-STA-01 has no report yet in
  this track (Phase Q pending), so V04 references F-SEC-01 V04-01 as the
  closest prior scan instead.
- Historical documents treated as hypotheses; no conclusion reused without
  revalidation against current code.

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism | UTF-8-safe truncation and panic-free input handling are framework-wide contract rules (AGENTS.md); both violations sit in framework code (`echo-tools` pdf tool, `src/eval` runner). |
| EKO product policy | CLI no-SQLite is EKO's local-assistant storage decision (file-backed stores); framework keeps its `sqlite` feature as an independent option. |
| Adapter boundary | Relative path dependencies (`../echo-agent` etc.) are the build-time adapter boundary — confirmed relative and machine-independent (V06). |
| Duplicate search | Terms searched: `worker`, `Worker`, `workers`, `sqlite`, `rusqlite`, `libsqlite3-sys`, `todo_write`, `TodoWrite`, `plan_create`, `plan_patch`, `plan_execute`, `search_todo`, `find_todo`, `TodoStore`, `worktrees`, `/Users/`; plus range-slice and index patterns; across both repos, all file types, excluding target/worktrees/git/node_modules. |
| Migration deletion | No deletion targets from this audit; doc-fix items reference archived findings (A-TSK-02-P3-03, A-BOOT-01-P3-06, A-CFG-01-P3-01, A-PROJ-01-P3-02). |

## Current Path

Verified per-invariant current state (each with its own validation report):

1. **Terminology**: zero tracked `worker`/`Worker` occurrences in either
   repository. The only matches are the `NetworkError` identifier substring
   (`worke` inside `Network`+`Error`, 31 hits across echo-core error.rs,
   providers, channels, mock_llm), one untracked git-ignored IntelliJ shelf
   file (`.idea/shelf/...shelved.patch`), and the distinct git-worktree
   concept. AGENTS.md's "随手清理" rule has no remaining target.
2. **CLI no-SQLite**: zero sqlite crates in manifests, lockfile, and the
   resolved tree; 8 source hits are comments, of which 5 affirm no-SQLite
   and 2 (`infra.rs:125` "sqlite-backed", `workspace/mod.rs:15` "后台任务
   SQLite DB") are stale comments already archived (A-BOOT-01-P3-06,
   A-CFG-01-P3-01, A-PROJ-01-P3-02). EKO uses
   `FileRuntimeStateStore`/file-backed stores exclusively.
3. **No parallel CRUD**: one `task_create`/`task_update`/`task_list`
   family (framework) + EKO `task_execute`; zero forbidden definitions.
   Only hit for `todo_write` is a negative test guard
   (`builder.rs:1181`); `plan_patch` hits are the sanctioned
   `to_task_plan_patch` conversion helper; stale `demo22_plan_execute`
   doc references archived as A-TSK-02-P3-03.
4. **Panic safety (macro level)**: zero production `panic!`/
   `unreachable!`/`unwrap`/`expect` in the CLI workspace; echo-agent's 82
   production-region `unwrap`/`expect` are doc comments or the test-only
   `react/tests.rs`; all 75 `panic!` + 1 `unreachable!` verified inside
   test fns. Sampled production direct indexing is guarded everywhere;
   archived production index panic F-EXT-03-P1-03 (IQR) still present.
5. **UTF-8 safety**: CLI fully clean. Framework has two live byte-slice
   panics (below). Archived F-EXT-02-P1-01 (edit.rs) and A-PROJ-01-P3-01
   (gitignore) still present. Correct `char_indices`/`floor_char_boundary`
   patterns dominate elsewhere (levels.rs:34, classifier.rs:436,
   auto_memory.rs:121, image_fetch.rs:293).
6. **Relative paths**: all 16 manifest `path` fields relative; zero
   tracked `worktrees|/Users/` residue in manifests/lockfiles/frontend;
   tracked hits are git-ignored local config, test fixtures, or the
   legitimate git-worktree feature. One stale prunable git worktree in
   CLI metadata (observation).

## Findings

### X-INV-01-P1-01: `parse_pdf_date` byte-slices a lossy-decoded String — panic on any non-ASCII PDF date field

- Priority: P1
- Confidence: high (byte-slice panic reproduced verbatim, exit 101)
- Layer: framework
- Evidence: `echo-agent/echo-tools/src/pdf.rs:225-227`
  (`let year = &rest[0..4]; let month = &rest[4..6]; let day = &rest[6..8];`
  after only `rest.len() >= 8`), `:219` (`fn parse_pdf_date`), caller
  `:196` (CreationDate/ModDate branch of `extract_pdf_metadata`, `:169`);
  registration `echo-agent/echo-tools/src/registry.rs:108-109,321`
  (`PdfExtractTool`, `PdfInfoTool`).
- Reachability: pdf tool `extract_metadata: true` parameter (`pdf.rs:67-68`)
  → `extract_pdf_metadata` → `parse_pdf_date(s)` where `s` is the raw
  `Object::String` bytes of a PDF's CreationDate/ModDate — attacker/third-
  party-controlled file content, feature-gated under `research` (`lopdf`).
- Expected invariant: AGENTS.md — no API panics on abnormal input; all
  string truncation uses char iteration; a byte-length guard is not a char-
  boundary guard.
- Observed behavior: `rest` is `String::from_utf8_lossy(date)` — always a
  *valid* UTF-8 string whose first 8 bytes may span multibyte chars.
  Reproduction: `date = [b'D', b':', 0xFF, 0xFE, 0xFD, 0x80, 0x81, 0x82]`
  panics "end byte index 4 is not a char boundary; it is inside '�'
  (bytes 3..6 of string)" (exit 101). Any PDF with non-ASCII (e.g. Chinese
  or invalid-UTF-8) date bytes panics deterministically; `from_utf8_lossy`
  turning each invalid byte into a 3-byte U+FFFD guarantees `len() >= 8`
  passes for as few as 6 bad bytes.
- Impact: the registered pdf tool aborts the entire agent run (no
  `catch_unwind` barrier — F-EXT-02 V02) when reading a crafted or
  non-ASCII PDF with metadata extraction; same defect class as
  F-EXT-02-P1-01 / F-EXT-03-P1-03, which are both P1.
- Root cause: byte-slicing a `&str` at fixed offsets with a byte-length
  guard instead of an ASCII-fast-path check or char-boundary-safe parse.
- Direction: in `parse_pdf_date`, first require the remaining bytes to be
  ASCII digits (e.g. iterate `rest.bytes()` and bail to `date_str.to_string()`
  on the first byte `> 0x7F`), or parse via `char_indices()`; the slice
  must only execute when bytes 0..8 are ASCII. Add a multibyte-date
  fixture. (Fix belongs to S-RDM-01; this review is read-only.)
- Regression validation: unit test feeding `parse_pdf_date(&[b'D', b':',
  0xFF, 0xFE, 0xFD, 0x80, 0x81, 0x82])` and a Chinese date — must return a
  string without panicking; existing ASCII date tests stay green.
- Validation reports: [V05](../validations/X-INV-01/V05-01.md)

### X-INV-01-P1-02: `extract_number_near_key` slices the original text at a lowercased-text byte offset — panic on multilingual LLM output

- Priority: P1
- Confidence: high (panic reproduced verbatim, exit 101)
- Layer: framework
- Evidence: `echo-agent/src/eval/runner.rs:728`
  (`let after = &text[pos..text.len().min(pos + key.len() + 50)];`),
  `:710` (`fn extract_number_near_key`), caller `:589` (expected-value
  metric check over `output`), feature: `eval`.
- Reachability: eval runner's numeric-assertion path — `output` is LLM/
  agent text (untrusted, commonly CJK in this project) and `key` is the
  metric name; live whenever an eval case asserts a numeric key against
  natural-language output.
- Expected invariant: AGENTS.md — byte slicing forbidden; truncation must
  use char iteration or `floor_char_boundary`.
- Observed behavior: `pos` is a byte offset from
  `lower_text.find(&lower_key)` applied to the original `text`
  (`to_lowercase()` is not byte-length-preserving for some Unicode, so the
  offset can even be wrong), and the window end `pos + key.len() + 50` is a
  raw byte offset that lands inside a multibyte char. Reproduction:
  `text = "结果是" + "好".repeat(18)`, `key = "结果"` → panic "end byte
  index 56 is not a char boundary; it is inside '好' (bytes 54..57 of
  string)" (exit 101). Any key followed by more than ~17 CJK characters
  panics deterministically.
- Impact: the eval capability aborts (run-killing panic) on multilingual
  output — the normal case for this project's users — making numeric
  metric evaluation unusable for non-ASCII text.
- Root cause: offset arithmetic on two different strings (`lower_text`
  vs `text`) plus a non-boundary-aware window end, instead of
  case-insensitive matching on the original string or a
  `floor_char_boundary`-guarded window.
- Direction: match the key case-insensitively on `text` itself (e.g.
  `text.to_lowercase()` for the search but translate the offset via
  `char_indices` — or simply search `text` with a case-insensitive find and
  clamp the window end with `text.floor_char_boundary(...)`). Add a CJK
  fixture to the eval metric tests.
- Regression validation: eval-assert fixture with a key followed by 20
  Chinese characters (and one with `İ`/`ẞ`-style uppercase) asserting no
  panic and the correct number extraction.
- Validation reports: [V05](../validations/X-INV-01/V05-01.md)

### X-INV-01-P3-01: `regression.rs` truncates `run_id` with a raw byte slice — latent, guarded only by today's UUID format

- Priority: P3
- Confidence: low
- Layer: framework
- Evidence: `echo-agent/src/eval/regression.rs:80`
  (`id: format!("regression_{}", &run.run_id[..12.min(run.run_id.len())])`);
  run_id production shape `format!("run_{}", uuid::Uuid::new_v4())`
  (`echo-agent/src/agent/react/mod.rs:1916`).
- Reachability: eval regression-case generation; `run_id` is ASCII today.
- Expected invariant: AGENTS.md — byte slicing only where logically
  guaranteed char-boundary-safe; the neighbor code in the same function
  correctly uses `chars().take()`.
- Observed behavior: a `String` byte slice without a boundary check; no
  panic is possible while `run_id` stays `run_` + UUID.
- Impact: latent — if `run_id` ever carries user text or a non-ASCII
  prefix, `regression.rs:80` panics; also an inconsistent style next to the
  file's own `chars().take()` calls.
- Root cause: shortcut truncation instead of the project's canonical
  `chars().take(12)` pattern.
- Direction: replace with `run.run_id.chars().take(12).collect::<String>()`;
  no behavior change today.
- Regression validation: existing regression tests green; optionally a
  non-ASCII run_id unit fixture asserting no panic.
- Validation reports: [V05](../validations/X-INV-01/V05-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Subagent-only terminology search (`worker`/`Worker`, all files, both repos, per-match classification) | yes | passed | [V01-01](../validations/X-INV-01/V01-01.md) |
| V02 | CLI no-SQLite (manifests, lockfile, `cargo tree --locked -e normal,build,dev`, source grep) | yes | passed | [V02-01](../validations/X-INV-01/V02-01.md) |
| V03 | No parallel task CRUD (`todo_write`/`plan_create`/`plan_patch`/`plan_execute` + TodoStore/Plan*Tool/search_todo) | yes | passed | [V03-01](../validations/X-INV-01/V03-01.md) |
| V04 | Panic safety (`unwrap`/`expect`/`panic!`/`unreachable!` full classification + direct-index sampling on live paths) | yes | passed | [V04-01](../validations/X-INV-01/V04-01.md) |
| V05 | UTF-8 byte-slicing audit (`s[..n]`/`s[n..]`/`s[..=n]` production regions, both repos) + 2 standalone panic reproductions | yes | failed (2 live violations → P1-01, P1-02) | [V05-01](../validations/X-INV-01/V05-01.md) |
| V06 | Relative-path invariant (manifest `path` fields, `worktrees|/Users/` tracked residue, lockfiles, worktree list, .gitmodules) | yes | passed | [V06-01](../validations/X-INV-01/V06-01.md) |

Every required validation has a report; every command has a known exit
code (0 or reproduced 101). The failed V05 does not block completion — it
is recorded as findings P1-01/P1-02.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| AGENTS.md: "只有 Subagent,没有 Worker(强制)…不得新建 worker 术语" | current | zero `worker`/`Worker` in all tracked files of both repos (V01-01) |
| AGENTS.md: "echo-agent-cli(EKO)不需要 SQLite…禁止引入" | current | zero sqlite in CLI manifests/lockfile/tree (V02-01); stale comments at infra.rs:125 / workspace/mod.rs:15 are archived doc items (A-BOOT-01-P3-06, A-CFG-01-P3-01) |
| AGENTS.md: "不得重新引入 todo_write/plan_create/plan_patch/plan_execute" | current | zero definitions; one negative test guard (builder.rs:1181); `to_task_plan_patch` is the sanctioned conversion (V03-01) |
| AGENTS.md: panic/UTF-8 hard rules | regressed | two live byte-slice panics (P1-01, P1-02, both reproduced); macros themselves are test/doc-only (V04-01, V05-01) |
| AGENTS.md: worktree merge gate — `grep "worktrees\|/Users/" */Cargo.toml` zero hits | current | all manifest paths relative; tracked hits are git-ignored/test/git-feature (V06-01) |
| A-TSK-02-P3-03: stale `demo22_plan_execute` doc references | current (unfixed) | re-confirmed at these commits (V03-01), not re-filed |
| F-EXT-02-P1-01 (edit.rs byte-slice panic), F-EXT-03-P1-03 (IQR index panic), A-PROJ-01-P3-01 (gitignore globstar) | current (unfixed) | present at reviewed commits (V04-01/V05-01), referenced not duplicated |

## Coverage And Uncertainty

- V04 direct-index audit is a sampled live-path classification, not an
  exhaustive inventory; Q-STA-01 owns the full overflow/unsafe/UTF-8
  matrix and should run at the same or newer commits.
- The two reproductions used standalone `rustc` programs mirroring the
  exact source logic; no project binary or test was executed, and no
  source was modified (read-only review).
- `src/eval` and `echo-tools` `research` feature are non-default features;
  reachability of the panics is static (tool registration in registry.rs is
  unconditional when the feature compiles; eval runner call path is
  compiled under `eval`).
- V01 terminology search is substring-based case-insensitive; a term
  spelled with unusual casing or splitting would not match (not observed in
  the codebase style).
- The stale prunable CLI worktree (V06 observation) was not removed
  (read-only review).

## Handoff

- Downstream tasks may rely on: terminology invariant clean (V01);
  no-SQLite dependency invariant clean (V02); no parallel task CRUD (V03);
  macro-level panic safety clean with the one archived index panic
  F-EXT-03-P1-03 (V04); relative-path invariant clean (V06); UTF-8
  invariant violated exactly at pdf.rs:225-227 and eval/runner.rs:728
  (V05, P1-01/P1-02).
- Reports to read: 6 validation reports above; F-EXT-02 (P1-01), F-EXT-03
  (P1-03), A-PROJ-01 (P3-01/P3-02), A-BOOT-01 (P3-06), A-TSK-02 (P3-03)
  for the archived items; Q-STA-01 (once available) for the full matrix.
- Stale conditions: this report becomes stale if either reviewed commit
  moves, or if any invariant-relevant symbol/pattern changes (new `worker`
  term, sqlite dependency added to CLI, forbidden CRUD reintroduced, or
  slice/index edits at the two violation sites).
- Follow-up task IDs: Q-STA-01 (full panic/UTF-8/overflow matrix), Q-FLT-01
  (fault fixtures incl. the two reproduced inputs), S-RDM-01 (roadmap
  items: P1-01, P1-02, P3-01, archived doc cleanups, optional CLI worktree
  prune).
