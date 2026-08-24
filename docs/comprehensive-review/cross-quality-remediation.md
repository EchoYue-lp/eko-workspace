# Cross-Layer And Quality Revalidation

> Status: implementation and final validation in progress
> Revalidation date: 2026-08-17
> Reviewed baseline: `echo-agent` `6d7d0cf`, `echo-agent-cli` `28fb4d1`,
> `echo-website` `ccc0172`
> Scope: baseline (`B-*`), cross-repository (`X-*`), quality/validation (`Q-*`),
> review-evidence consistency, and the previously omitted website

## Scope Boundary

The original review protocol explicitly scoped itself to `echo-agent` and
`echo-agent-cli`. Its `Q-WEB-01` task reviewed
`echo-agent-cli/web-frontend`; it did not review the independent
`echo-website` repository. Consequently, neither a green `Q-WEB-01` gate nor a
historical `needs_evidence` result says anything about the public website.

This revalidation excludes the completed atomic framework (`F-*`) and
application (`A-*`) layer conclusions. It does inspect framework/application
code where a `B-*`, `X-*`, or `Q-*` claim crosses those repositories. The
historical reviewer reports and validation attempts remain immutable evidence;
this document is a current-code overlay rather than a rewrite of those files.

The canonical Codex task headings contain exactly 75 in-scope atomic findings:
14 baseline (`B-*`), 37 cross-repository (`X-*`), and 24 quality (`Q-*`).
Cluster summaries below are navigation only. Final closure requires one explicit
disposition for every atomic ID; a cluster-level statement cannot close a row.
The mechanically reconciled rows live in
[cross-quality-finding-ledger.md](cross-quality-finding-ledger.md); its verifier
also locks the six fresh website IDs listed below.

## Implementation Gate

Before adding or moving behavior, the current repositories were searched by
type, field, behavior, registration, and production call path.

| Classification | Current authority |
|---|---|
| Generic mechanism | `echo-agent`: typed authentication verification, reusable dependency and submission gates, provider/cache request contracts |
| EKO product policy | `echo-agent-cli`: local shell configuration, TaskRuntime file retention/projections, GUI logging, platform and surface validation |
| Adapter boundary | Lossless ordered conversion between persisted EKO events and framework hooks; no second DAG, retry loop, or lifecycle authority |
| Website | `echo-website`: presentation, documentation projection/synchronization, route/SEO integrity, and deployment policy; no runtime dependency on either Rust repository |

The latest code already has one framework `RuntimeDagExecutor` readiness and
settlement authority. EKO's `EkoRuntimeDagController` supplies product-specific
resource semaphores, file/worktree policy, review policy, and persistence
adaptation. The latest foreground-turn service is also shared by GUI, TUI,
CLI, and channel entry points. No replacement DAG, foreground owner, store, or
validator was added during this remediation.

## Mature-Implementation References

The changes that affect event shape, task recovery, non-interactive output, or
surface ownership were treated as architecture decisions rather than local bug
fixes. The following official implementations were checked before choosing the
boundary:

- [OpenAI Codex app-server protocol](https://github.com/openai/codex/blob/main/codex-rs/app-server/README.md):
  a stable thread/turn/item event model, JSONL-compatible stdio transport,
  explicit terminal events, resume/fork operations, and separately managed
  user-driven terminal processes. EKO therefore exposes one canonical
  envelope in `--jsonl` mode and keeps an interactive terminal out of Agent
  `permission_mode` policy.
- [Claude Code subagents](https://docs.anthropic.com/en/docs/claude-code/sub-agents)
  and [hooks](https://docs.anthropic.com/en/docs/claude-code/hooks): specialized
  execution remains part of the Agent lifecycle, while extension policy is
  configured by the local user. EKO consequently projects framework Subagent
  and Tool facts losslessly instead of creating a second application event or
  execution vocabulary.
- [Temporal Activity definition](https://docs.temporal.io/activity-definition):
  durable side effects must tolerate retry and partial failure. Conversation
  deletion and TaskRuntime bootstrap therefore use idempotent staged state,
  durable tombstones, and one publish point instead of relying on a long
  in-memory transaction.
- [AWS Transactional outbox pattern](https://docs.aws.amazon.com/prescriptive-guidance/latest/cloud-design-patterns/transactional-outbox.html):
  a dual write must first publish a durable intent, and an at-least-once relay
  requires stable identity and idempotent consumers. EKO applies the same
  invariant to rule promotion without importing the server-side architecture:
  a workspace-scoped file receipt coordinates replay, while the existing rule
  file and memory store remain the only content authorities.

The framework/application split follows from those references and this
repository's product boundary: typed Tool/event/file primitives and generic DAG
settlement are reusable framework mechanisms; local terminal/LSP management,
worktree policy, conversation deletion, GUI/TUI/CLI/channel adapters, and their
projections are EKO policy. Adapters may add product metadata, but may not own a
second graph loop, retry policy, event bus, artifact reader, or persistent Tool
result authority.

## Disposition Vocabulary

Every historical finding is assigned exactly one current disposition in the
final ledger:

- `fixed`: the current production path and a regression test prove the stated
  invariant;
- `stale`: the cited behavior is absent or the historical path is no longer
  reachable, without claiming that an unrelated replacement was "fixed";
- `retained`: the code is a deliberate, documented public framework option and
  cannot be deleted merely because EKO does not call it;
- `residual`: current code still violates all or part of the finding and the
  exact owner remains named;
- `evidence-only`: the product defect is closed, but a required gate, fault
  family, or surface scenario still needs current execution evidence.

A definition, registration, green submission gate, or prose matrix is never by
itself production-path evidence. Each `fixed` row must name the producer,
consumer/owner, negative or mutation case, and the command that executed it.

## Current-Code Disposition

| Historical cluster | Revalidation against the current baseline |
|---|---|
| `B-PATH` fragmented channel lifecycle and surface ownership | Stale: channel mode now starts the shared headless services, owns shutdown, and uses the same foreground-turn lifecycle service as the other surfaces. Wire/event parity tests remain authoritative; the old prose-only capability-matrix assertion was not accepted as evidence. |
| `B-ARCH` / `X-TSK` parallel DAG authorities | Partly stale, partly current. The framework owns the generic ready-frontier and EKO uses the canonical runtime adapter, but EKO bootstrap still creates the product run before the canonical plan commit and recovery can publish a partial cross-store generation. These are being fixed without adding another graph, validator, or executor. |
| `X-EVT` envelope, terminal, ordering, and replay loss | Current at baseline. Framework events are typed, but ordinary chat still loses canonical identity/order at the GUI boundary and has no durable replay authority. Remediation carries the canonical envelope through one per-conversation journal and makes every surface consume the same rich terminal facts. |
| `Q-TST` ignored truncated-stream test and zero DOM tests | The two literal historical claims are stale: the ReAct truncation regression is active and mounted jsdom tests exist. Provider-response parsing, mounted transport teardown, rich mocks, cross-platform compilation, and cache-propagation behavior remain separate current checks; none is inferred green from the stale claims. |
| `Q-FW` weaker-than-documented framework submission gate | Current at baseline; remediation aligns CI and contributor commands with the required all-target/all-feature contract and adds platform compile lanes. |
| `Q-DEP` JWT key/algorithm mismatch and absent advisory/license policy | Current at baseline; remediation couples each JWT algorithm to its matching decoding-key family, adds positive/negative HS256/RS256 tests, upgrades fixable vulnerable dependency chains, and adds explicit time-bounded policy for unavoidable indirect advisories. |
| `Q-PERF` hook backpressure, full replay, cache/log retention | Current at baseline; remediation separates post-commit hook delivery from the durable write lock, incrementally advances projections, evicts terminal caches, and bounds GUI log retention. Durable delivery and disk-run retention are accepted only if their final regression tests pass. |
| `Q-STA` UTF-8 decoders/image arithmetic | Stale: the latest implementations use character-safe decoding and checked image-size arithmetic. The remaining GUI token arithmetic and post-runtime environment mutation were current and are remediated with checked/saturating arithmetic and ephemeral config merging. |
| `Q-GUI` duplicate capability authorities | Current at baseline: two `identifier = default` capability files disagreed. The unused duplicate is removed and the retained Tauri authority is exercised by a GUI-only lane. |
| `Q-DOC` CLI commands, SQLite/storage, GUI status, and plan authority drift | Current at baseline. Remediation follows the actual argument parser and file-backed EKO storage. The root plan now isolates dated implementation history from its single current execution entry; subordinate and historical guides no longer claim competing authority or retain internal Worker terminology. |
| `S-QA` "all submission gates unexecuted" synthesis | Historical-only. Later remediation executed gates; this run records fresh commands and SHAs instead of changing immutable attempts. |
| Public website | Missing from the original catalog. Fresh review found no CI/test gate, manually copied drifting docs, broken relative Markdown navigation, stale `/echocowork` sitemap/SEO, missing manifest, eager documentation loading, and destructive non-atomic deployment. |

The revalidation also re-opened findings that appeared only in the ZCode task
reports rather than the Codex synthesis. Current code already fixes the
four-value IQR panic, UTF-8 path slicing, GUI terminal shutdown, and headless
Dreaming/session-review ownership. The remaining provider-stream fixtures,
surface management commands, bounded runtime/resource behavior, and production
fault/parity scenarios are tracked independently; a green submission gate does
not substitute for those behavioral checks.

## Website Authority

Framework documentation is projected from the bilingual source directories by
a deterministic sync script with source revision and per-file hashes. EKO
website pages are a deliberately small, code-audited product projection until
the independent application documentation review is complete. Registry checks
enforce unique slugs, existing localized files, and resolvable relative links;
the website does not publish a stale copy of the CLI README as an authority.

The website route contract is `/docs/...` for framework documentation and
`/eko/docs/...` for EKO documentation, with Chinese as the default and the
corresponding English routes under `/en/...`. Product and documentation routes
are emitted as crawlable static HTML rather than a client-only shell. Sitemap,
canonical/hreflang metadata, structured data, manifest, navigation, `robots.txt`,
and the supplemental `llms.txt`/`llms-full.txt` discovery files are generated
from the same registry. The LLM discovery files are treated as a proposal-based
aid, not an SEO/GEO ranking guarantee. Deployment builds and validates an
immutable release directory before an atomic live-link switch, preserving a
rollback target rather than deleting the live tree.

### Fresh website findings

The following findings were absent from the historical task catalog because
that catalog did not include the independent `echo-website` repository.

| ID | Current-code proof at the reviewed baseline | Remediation contract |
|---|---|---|
| `W-GATE-01-P1-01` | The repository had no workflow or test files, and `package.json` exposed only development, build, lint, and preview commands. | A clean-install website gate now owns formatting, lint, registry/discovery checks, unit tests, static generation, browser E2E, and the production build. |
| `W-DOC-01-P1-01` | `src/docs/loader.ts` declared manually copied documentation; 51 files had no source revision, hash manifest, or drift check, and the EKO copies still advertised obsolete SQLite storage. | Framework documentation is deterministically synchronized from its bilingual source with revision and per-file hashes. EKO publishes a small code-audited projection until its source documents are final. |
| `W-ROUTE-01-P1-01` | Markdown relative links were emitted unchanged even though the application router accepted only registry slugs, so links such as `01-react-agent.md` did not resolve to a public route. | One link resolver maps source-relative documents to localized registry routes; registry and full-link scans reject unknown targets. |
| `W-SEO-01-P1-01` | The sitemap published `/echocowork` while the application route was `/eko`; the document referenced a missing manifest and retained stale EchoCoWork metadata. | The route registry generates localized physical HTML, canonical/hreflang metadata, sitemap, robots, manifest, Open Graph/Twitter metadata, JSON-LD, and supplemental LLM discovery files from one source. |
| `W-SEO-01-P2-02` | After client-side `echo-agent -> EKO` navigation, title/canonical changed but the JSON-LD script still described `SoftwareSourceCode`; client-side 404 navigation likewise retained crawlable identity from the previous page. | Static generation and client navigation now call one structured-data builder. Product switching replaces the graph, while 404 removes canonical/alternates/JSON-LD and sets `noindex`; returning to a valid route reconstructs them. |
| `W-DEPLOY-01-P1-01` | `deploy.sh` hard-coded a host path, skipped `npm ci`, deleted the live directory, and copied a new build into place without a rollback point. | Deployment validates an immutable release directory, atomically changes the live symlink, and preserves the previous target for rollback. |

## Validation Contract

The final evidence for this overlay must include:

- framework formatting, both Clippy gates, all-target/all-feature tests,
  no-default library check, feature-isolation matrix, advisory and license
  policy checks;
- EKO formatting, both Clippy gates, all-feature workspace tests, app-core
  no-default check, GUI-only matrix, frontend Prettier/tests/build, no SQLite,
  no absolute worktree paths, and no internal Worker terminology;
- website clean install, formatting, lint, sync/drift checks, unit/route/link
  tests, production build, browser end-to-end checks, and desktop/mobile visual
  inspection;
- exact post-remediation repository SHAs and a clean diff check for every Git
  repository.
- ten deterministic ReAct/Tool fault families, ten Task/Subagent fault
  families, and the 23 declared multi-surface scenario pairs, using scripted
  local providers/tools/stores rather than optional external credentials.
- `scripts/verify-cross-quality-finding-ledger.sh`, which rejects a missing,
  duplicate, unsupported, or placeholder disposition across the 75 canonical
  B/X/Q rows and checks the separate six-ID website finding set.

Final commands, counts, SHAs, and any justified residual advisory exceptions
are recorded only after the corresponding command exits successfully.
