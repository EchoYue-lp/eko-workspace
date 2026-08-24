# Q-DEP-01: Dependency, supply-chain, and license health

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean (verified `git status
> --porcelain`, empty output)

## Question

Are duplicate versions, stale/unmaintained crates/packages, build scripts,
native dependencies, licenses, and advisories understood for both
repositories?

**Answer: yes, with actionable findings.** Duplicates are mostly benign
transitive splits (38 framework / 76 CLI multi-version names; the only
directly-controllable one is crossterm 0.28/0.29 — the reqwest 0.12/0.13
"split" is a mobile-target-only lockfile entry). A live advisory load sits in
the shipped EKO binary through EKO-enabled framework features: 13
vulnerabilities in the CLI lockfile, 6 distinct live crates (lopdf,
quick-xml ×4 versions, crossbeam-epoch) reachable from untrusted document
parsing (PDF/XLSX/DOCX/XML) plus 6 live informational warnings (anyhow —
direct dep, lru via ratatui, event-listener/memmap2/bincode/fxhash via
polars/scraper). Build scripts are few and safe except one transitive
polars-ops script whose documented fragility EKO re-enables. Licenses are
uniformly permissive (all workspace crates MIT; no strong copyleft
anywhere). Frontend: no dead tree, but one dead-and-advisory-carrying
production dependency (dompurify) and four dev-toolchain advisories.

## Scope

- Both `Cargo.lock` files (621 framework / 903 CLI packages), all 10
  workspace `Cargo.toml` files, feature wiring (`echo-agent/Cargo.toml:57-67`,
  `echo-agent-cli/echo-agent-app-core/Cargo.toml:10-15`), the single
  workspace `build.rs` (`echo-agent-cli/build.rs`), the
  `web-frontend/package.json` + `package-lock.json` (350 entries).
- Registry-source inspection of polars-ops, onig_sys, tree-sitter, tauri
  (native/build-script crates).
- Advisory data: rustsec/advisory-db snapshot (2026-08-12, 1216 advisories).
- Reachability traces (`cargo tree -i`) for every flagged crate.

## Out Of Scope

- License obligations review of vendored third-party text (declared license
  fields only).
- Runtime behavior of the flagged parsers beyond registration-level
  reachability (tool-level behavior is owned by A-TOOL-01 / A-INP-01).
- Windows/Linux GUI build execution (the `gui` feature is classified
  statically; `Q-GUI-01` owns gate execution).
- The other two reviewer tracks' reports (independence rule).

## Inputs

- Root `AGENTS.md` in full (panic rules, layering gates, no-SQLite, CI
  gates, feature-parity).
- `docs/comprehensive-review/README.md`, `REPORTING.md`, `TASKS.md`
  (Q-DEP-01 card only), `zcode-ds/README.md`, both templates.
- Dependency task reports read: `B-BASE-01` (build topology, both lockfiles,
  CI workflows), `Q-STA-01` (duplicate scan V07 + finding P3-03).
- No historical audit conclusion accepted without revalidation; all advisory
  data produced by this run.

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism | The duplicate/advice/license health of the framework workspace is a framework property; the vulnerable crates live in framework manifests (echo-tools media/research deps, polars via `data`/`statistics`). The framework's own default build ships none of them. |
| EKO product policy | The feature selection that makes the vulnerable parsers live is EKO's (`echo-agent-app-core/Cargo.toml:10-15` enables `media`, `research`, `data`, `statistics`, `chart`). The frontend package inventory is EKO product surface. The local-desktop threat model (AGENTS.md) governs severity: no remote attacker, user-triggered file parsing only. |
| Adapter boundary | `echo-agent-cli/build.rs` is the single build adapter (tauri gate, safe). Feature forwarding from EKO to the framework is the boundary where `data` re-enables a documented fragile build script. |
| Duplicate search | Searched: Cargo.lock name groups (both), `cargo tree -d` (both, default features), reverse traces for reqwest/crossterm/quick-xml/lopdf/crossbeam-epoch/lru/quinn-proto/rsa/ring/onig_sys/tree-sitter/openssl-sys, package-lock version groups, `npm ls --all`. |
| Migration deletion | Delete targets: `dompurify` + `@types/dompurify` from `web-frontend/package.json` if the secure-by-default react-markdown stance is kept (or upgrade to 3.4.13 and wire deliberately); duplicate `@tailwindcss/vite` declaration in devDependencies. |

## Current Path

1. **Duplicate versions**: 38 (framework) / 76 (CLI) multi-version names in
   the lockfiles; in-graph duplicates are all transitive except the real
   crossterm 0.28.1 (TUI stack: direct + ratatui + reedline) + 0.29.0
   (comfy-table → polars-core → polars) split, both linked in the shipped
   binary. reqwest 0.12.28 + 0.13.3: 0.13.3 is declared by `tauri 2.11.2`
   under a **mobile-target-only** cfg
   (`[target.'cfg(any(target_os = "android", all(target_vendor = "apple",
   not(target_os = "macos"))))'.dependencies.reqwest]`, tauri Cargo.toml:321)
   and is absent from every desktop resolution — a lockfile-universe entry,
   not a second shipped HTTP client.
2. **Advisories (CLI lockfile)**: 13 vulnerabilities — 6 live crates in the
   default binary: `lopdf` 0.34.0 (RUSTSEC-2026-0187, stack overflow on
   nested PDF, fix 0.42.0; `PdfExtractTool`/`PdfInfoTool` registered at
   `echo-tools/src/registry.rs:97,312`), `quick-xml` 0.31.0 (calamine/XLSX),
   0.36.2 (docx-rs/DOCX), 0.37.5 (echo-tools research), 0.39.4 (object_store
   → polars) all under RUSTSEC-2026-0194+0195 (quadratic dup-attr / namespace
   alloc DoS, fix 0.41.0); `crossbeam-epoch` 0.9.18 (RUSTSEC-2026-0204, fix
   0.9.20, via polars). Build-time only: quick-xml 0.38.4 (tauri-build/plist).
   Never linked: quinn-proto 0.11.14 (reqwest optional HTTP/3) and rsa 0.9.10
   (framework lockfile only, via sqlx-mysql; no fix exists). Live warnings:
   anyhow 1.0.102 (unsound, **direct CLI dep**, fix 1.0.103), lru 0.12.5
   (unsound, ratatui), event-listener 5.4.1 + memmap2 0.9.10 + bincode 2.0.1
   (polars), fxhash 0.2.1 (scraper/web).
3. **Frontend**: `npm ls` clean (exit 0, 350 entries, one benign transitive
   duplicate `@types/unist` 2+3); npmjs-registry audit = 5 advisories (0
   critical): dompurify 3.4.7 (moderate, 5 advisories — **never imported**,
   dead dependency), vite 6.4.2, postcss 8.5.8, nanoid 3.3.11, @babel/core
   7.29.0 (dev-toolchain). `npm audit` is impossible against the configured
   registry (`registry.npmmirror.com` returns 404 [NOT_IMPLEMENTED]).
4. **Build scripts / natives**: one workspace build.rs
   (`echo-agent-cli/build.rs`, tauri gate, `unwrap_or_default` — safe).
   Transitive: polars-ops 0.53.0 build.rs uses
   `version_check::Channel::read().unwrap()` (panic on rustc-probe failure;
   the framework documents the resulting failures and excludes `data` from
   defaults, `echo-agent/Cargo.toml:59-63`; EKO re-enables it). Native C
   compiled into the shipped binary: tree-sitter + 6 grammars (echo-tools
   `files`), oniguruma via onig_sys (syntect `regex-onig`), ring and
   zstd-sys (polars). GUI-only natives (wry/webkit2gtk/gtk) and openssl-sys
   are lockfile-universe, not linked in the default binary.
5. **Licenses**: all 10 workspace crates MIT; third-party graph has no
   GPL/AGPL/SSPL or non-commercial license. MPL-2.0 (cssparser, selectors,
   dtoa-short, option-ext — file-level notice only), BSL-1.0 (Boost: ryu,
   xxhash-rust, whoami, clipboard-win, error-code), CC0-1.0 (notify,
   tiny-keccak), Unlicense-OR-MIT (memchr, byteorder, globset, walkdir).
   `web-frontend/package.json` has no license field.

## Findings

### Q-DEP-01-P2-01: live RUSTSEC vulnerabilities in the shipped EKO binary via EKO-enabled framework features (media/research/data)

- Priority: P2
- Confidence: high (real audit run + per-crate reverse traces + tool
  registration evidence)
- Layer: adapter (feature selection) with framework-owned crates
- Evidence: `echo-agent-cli/echo-agent-app-core/Cargo.toml:10-15` (enables
  `media`, `research`, `data`, `statistics`, `chart`);
  `echo-agent/echo-tools/src/registry.rs:97,312` (`PdfExtractTool`/
  `PdfInfoTool`); `echo-agent/echo-tools/src/excel.rs` (calamine);
  `echo-tools/Cargo.toml:18,30` (`research` → quick-xml, `media` →
  lopdf/calamine/docx-rs); `echo-agent/Cargo.toml:83-84` (`data`/`statistics`
  → polars)
- Reachability: cargo-audit (DB snapshot 2026-08-12, 1216 advisories) flags
  13 vulnerabilities in the CLI lockfile; `cargo tree -i` shows linked in the
  default binary: `lopdf 0.34.0` (media/pdf), `quick-xml 0.31.0` (calamine),
  `0.36.2` (docx-rs), `0.37.5` (research), `0.39.4` (object_store/polars),
  `crossbeam-epoch 0.9.18` (polars-stream). The EKO tools registered for
  these parsers (`read_pdf`, xlsx/docx readers, XML parsing) execute when the
  user points the agent at a document.
- Expected invariant: the shipped product contains no dependency with a
  known, fixable security advisory; parsing of user-supplied documents does
  not crash the process.
- Observed behavior: RUSTSEC-2026-0187 (lopdf stack overflow, 7.5), ×10
  quick-xml instances of RUSTSEC-2026-0194/0195 (quadratic attribute check /
  unbounded namespace allocation DoS, 7.5), RUSTSEC-2026-0204 (crossbeam
  invalid pointer deref) are linked; fixes are available (lopdf 0.42.0,
  quick-xml 0.41.0, crossbeam-epoch 0.9.20).
- Impact: a maliciously crafted PDF/XLSX/DOCX/XML file supplied by the user
  (or fetched by the agent during a research task) can crash the EKO process
  (stack overflow / memory exhaustion) mid-run, losing the in-progress
  session; quadratic XML parsing is a CPU DoS. No privilege boundary is
  crossed (local threat model), which is why this is P2, not P1.
- Root cause: EKO's feature selection pulls the framework's heavy document
  parsers (media/research/data) with no version-pinning or advisory gate;
  the framework's default features would avoid all of it.
- Direction: `cargo update -p lopdf --precise 0.42.0` (and align the
  media module), `cargo update -p quick-xml@0.31.0/0.36.2/0.37.5/0.39.4` or a
  `[patch]` to 0.41.0 (single aligned version — also reduces the 5-way
  quick-xml duplicate), `cargo update -p crossbeam-epoch` (0.9.20); then
  re-audit. Add `cargo audit` to both CI workflows (see P3-06).
- Regression validation: after the bumps, `cargo audit` reports zero
  vulnerabilities in both lockfiles; the media-tool test fixtures
  (`pdf_extract`, xlsx/docx read tests) stay green; `cargo tree -d` no longer
  shows quick-xml duplicates.
- Validation reports: [V03-01](../validations/Q-DEP-01/V03-01.md),
  [V01-01](../validations/Q-DEP-01/V01-01.md)

### Q-DEP-01-P3-01: dompurify — dead production dependency carrying five known advisories; deprecated @types stub

- Priority: P3
- Confidence: high (import grep + npmjs audit + registry metadata)
- Layer: application (frontend)
- Evidence: `echo-agent-cli/web-frontend/package.json:23` (dompurify ^3.4.7),
  `:33` (@types/dompurify ^3.0.5); `package-lock.json` resolved dompurify
  3.4.7; zero import sites in `src/` (grep across src/public/config);
  `src/components/common/MarkdownContent.tsx:11` (react-markdown only, with
  the documented rationale "react-markdown is secure by default").
- Reachability: none — dompurify is never imported; the installed 3.4.7
  satisfies RUSTSEC-equivalent npm advisories GHSA-55q2-fjhq-7xh7,
  GHSA-cmwh-pvxp-8882, GHSA-gvmj-g25r-r7wr, GHSA-vxr8-fq34-vvx9,
  GHSA-c2j3-45gr-mqc4 (fixed in 3.4.13, an in-range `npm update` away).
- Expected invariant: every declared production dependency is either used or
  absent; no dependency carries known advisories in the shipped lockfile.
- Observed behavior: dompurify 3.4.7 is declared and locked but unused; its
  advisories remain on the audit report; `@types/dompurify` is a deprecated
  stub (dompurify ships its own types since 3.2.0).
- Impact: misleading security posture (the report shows an XSS-sanitizer
  advisory even though no sanitizer runs); if a future change wires a
  sanitizer in, it starts from a vulnerable version. No runtime impact today.
- Root cause: sanitizer removed when react-markdown replaced the hand-rolled
  markdown parser; the dependency was not pruned.
- Direction: delete `dompurify` and `@types/dompurify` (keeping the
  secure-by-default react-markdown path) — or, if a sanitizer is wanted,
  upgrade to 3.4.13 and import it deliberately in MarkdownContent.tsx.
- Regression validation: after removal, `npm ls` clean, `npm audit` (npmjs
  registry) drops the dompurify entries, `npm run build` + vitest green.
- Validation reports: [V02-01](../validations/Q-DEP-01/V02-01.md)

### Q-DEP-01-P3-02: frontend dev-toolchain advisories (vite/postcss/nanoid/@babel) and npm audit blocked by the configured registry mirror

- Priority: P3
- Confidence: high (npmjs audit run, exit 1 with 5 vulns; mirror 404
  recorded)
- Layer: application (frontend toolchain)
- Evidence: `echo-agent-cli/web-frontend/package.json:37-42` (vite ^6.3.5,
  vitest ^4.1.10, typescript ~5.8.3); installed vite 6.4.2, postcss 8.5.8
  (transitive via vite/tailwindcss), nanoid 3.3.11 (via postcss),
  @babel/core 7.29.0 (via @vitejs/plugin-react); npm config registry =
  registry.npmmirror.com (audit endpoint 404 [NOT_IMPLEMENTED]).
- Reachability: dev-server/build-time only; none of these are in the shipped
  Tauri bundle.
- Expected invariant: `npm audit` is runnable in the project's configured
  environment, and dev-toolchain advisories are at least known.
- Observed behavior: GHSA-fx2h-pf6j-xcff (vite `server.fs.deny` bypass,
  high) + launch-editor NTLMv2 disclosure at vite 6.4.2 (fixed 6.4.3);
  GHSA-6g55-p6wh-862q family (postcss arbitrary file read, high) at 8.5.8
  (fixed 8.5.23); GHSA-28wg-ghj8-5hjv/2v37-7h3g-55p8 (nanoid infinite loop)
  at 3.3.11 (fixed 3.3.17); GHSA-4x5r-pxfx-6jf8 (@babel/core, low). Running
  `npm audit` without `--registry=https://registry.npmjs.org` fails with 404
  on the configured mirror.
- Impact: the declared `npm audit`-style gate cannot run in the default
  configuration, so toolchain advisories stay invisible to developers; the
  dev-server file-read advisories matter if the dev server is ever exposed.
- Root cause: npmmirror registry does not implement the npm audit endpoint;
  dependency versions sit below the advisory-fixed releases.
- Direction: `npm update` vite/postcss/nanoid/@babel (all patch/minor fixes),
  and either switch the project registry to npmjs for audit or document the
  `--registry` override in AGENTS.md/scripts; consider a `npm audit` step in
  CI (currently absent).
- Regression validation: `npm audit --registry=https://registry.npmjs.org`
  reports 0 high after the updates; vitest + `npm run build` green.
- Validation reports: [V02-01](../validations/Q-DEP-01/V02-01.md)

### Q-DEP-01-P3-03: frontend manifest hygiene — duplicated @tailwindcss/vite declaration and missing license field

- Priority: P3
- Confidence: high (direct package.json inspection)
- Layer: application (frontend manifest)
- Evidence: `echo-agent-cli/web-frontend/package.json:15` (`@tailwindcss/vite`
  ^4.1.4 in dependencies) and `:32` (^4.1.8 in devDependencies); no
  `license` field in the same file (compare the Rust crates, all MIT).
- Reachability: manifest-level; resolved single version 4.2.2 (dedup).
- Expected invariant: one declaration per dependency; package metadata
  complete for a shipped product surface.
- Observed behavior: the build plugin is declared twice with different
  ranges; the frontend package has no license metadata.
- Impact: range drift risk (one declaration can be bumped and the other
  forgotten — a future version skew); incomplete SPDX metadata for
  distribution of the web assets. No runtime defect.
- Root cause: manifest maintained by hand without dedup review.
- Direction: remove the dependencies-block declaration, keep the
  devDependencies one; add `"license": "MIT"`.
- Regression validation: `npm ls` clean after the edit; `npm run build` green.
- Validation reports: [V02-01](../validations/Q-DEP-01/V02-01.md)

### Q-DEP-01-P3-04: polars-ops 0.53.0 build script panics on rustc-probe failure — EKO re-enables `data` against the framework's documented exclusion

- Priority: P3
- Confidence: medium (source inspection of the build script + the framework
  note; no failure observed in this environment)
- Layer: adapter (EKO feature selection vs framework build policy)
- Evidence: `echo-agent/Cargo.toml:59-63` (NOTE: `data` excluded from default
  because polars-ops 0.46–0.53 build.rs fails on some platforms; upstream fix
  unreleased); `echo-agent-cli/echo-agent-app-core/Cargo.toml:13` (EKO
  enables `data` + `statistics`); `~/.cargo/registry/src/*/polars-ops-0.53.0/
  build.rs` (`version_check::Channel::read().unwrap()` — panics when the
  rustc probe fails, e.g. restricted/cross/vendored build environments).
- Reachability: every EKO build compiles polars-ops (data/statistics →
  polars 0.53.0); builds succeed in this environment (all-feature clippy
  passed in Q-STA-01), so the panic is environment-dependent.
- Expected invariant: EKO's build does not depend on a build script the
  framework explicitly documents as broken on some platforms.
- Observed behavior: the framework excludes `data` from defaults with a
  documented reason; EKO enables it anyway, inheriting the fragility on
  exactly the platforms the note warns about (end-user machines building
  from source, sandboxed CI).
- Impact: source builds of EKO can fail on affected platforms with no
  explanation beyond a build-script panic; the failure class is documented
  upstream but unmitigated here.
- Root cause: feature-forwarding without inheriting the framework's build
  policy; polars-ops 0.53 pinned by the `data`/`statistics` features.
- Direction: track the upstream polars fix (pola-rs/polars#67422be6) and
  re-evaluate the feature set; alternatively gate `data`/`statistics` behind
  an opt-in EKO feature and document the build requirement in AGENTS.md; keep
  `cargo check --no-default-features` coverage of the CLI without polars.
- Regression validation: a clean `cargo build` in an environment without a
  working rustc probe should fail with a clear message, or not fail at all
  after the polars bump; current all-feature gates stay green.
- Validation reports: [V04-01](../validations/Q-DEP-01/V04-01.md)

### Q-DEP-01-P3-05: lockfile-universe advisory and orphan entries — reqwest 0.13.3, quinn-proto, rsa (refines Q-STA-01-P3-03)

- Priority: P3
- Confidence: high (lockfile referencer scan + `--target all` traces +
  tauri registry-source inspection)
- Layer: adapter (dependency graph boundary)
- Evidence: `echo-agent-cli/Cargo.lock` — reqwest 0.13.3 referenced only by
  `tauri 2.11.2` under a mobile-only target cfg (tauri Cargo.toml:321);
  quinn-proto 0.11.14 referenced only by `quinn 0.11.9` (itself only via
  reqwest 0.12.28's optional HTTP/3 feature); rsa 0.9.10 present in the
  framework lockfile only (via sqlx-mysql 0.8.6, `database` feature) and
  **not at all** in the CLI lockfile; CLI lockfile contains zero orphaned
  referencers for rsa.
- Reachability: none of the three ever link in a desktop EKO build (verified
  with `cargo tree --target all -i` for quinn-proto/reqwest 0.13.3).
- Expected invariant: the lockfile reflects what any target/feature can
  resolve; advisories on never-linked crates are understood as latent.
- Observed behavior: cargo audit flags quinn-proto (RUSTSEC-2026-0185, 7.5)
  and rsa (RUSTSEC-2023-0071, medium, **no fix available**) even though they
  cannot link; reqwest 0.13.3 inflates the duplicate-dependency inventory.
- Impact: audit noise and a stale inventory; Q-STA-01-P3-03's "two HTTP
  clients in the shipped app" wording is refuted — 0.13.3 never ships on
  desktop, so the reqwest half of P3-03 downgrades to lockfile hygiene while
  its crossterm half stands (both 0.28.1 and 0.29.0 are linked).
- Root cause: lockfile-v4 records the whole target/feature universe; no
  pruning or advisory-gate surfaced the difference.
- Direction: keep the entries (they are resolution-legal) but document them;
  optionally run `cargo audit --target` with the desktop targets to get a
  shipped-binary-relevant report; re-file P3-03's reqwest half as hygiene
  when convenient.
- Regression validation: none required (no code change); the audit
  classification above is reproducible from the committed lockfiles.
- Validation reports: [V03-01](../validations/Q-DEP-01/V03-01.md),
  [V01-01](../validations/Q-DEP-01/V01-01.md),
  [V05-01](../validations/Q-DEP-01/V05-01.md)

### Q-DEP-01-P3-06: no advisory gate in CI, and npm audit unusable in the default registry configuration

- Priority: P3
- Confidence: high (workflow + registry inspection)
- Layer: application (CI policy)
- Evidence: `echo-agent/.github/workflows/rust-ci.yml` and
  `echo-agent-cli/.github/workflows/rust-ci.yml` — no `cargo audit` step in
  either; `echo-agent-cli/.github/workflows/rust-ci.yml:37-45` — no frontend
  job at all (B-BASE-01-P2-01); npm config registry = npmmirror.com (audit
  endpoint 404).
- Reachability: every push/PR runs the Rust gates; none of them runs an
  advisory check; the 13 vulnerabilities in the CLI lockfile therefore pass
  CI silently.
- Expected invariant: new advisories on shipped dependencies fail the build
  or at least surface in CI.
- Observed behavior: advisory posture changes are invisible to CI; the
  frontend's advisory state is invisible to CI twice over (no frontend job +
  blocked audit endpoint).
- Impact: the vulnerabilities in P2-01 can be introduced by any dependency
  bump and remain unnoticed until a manual audit; the ecosystem splits
  (quick-xml ×5) grow without a `cargo tree -d` guard.
- Root cause: advisory scanning was never part of the CI gate definition.
- Direction: add `cargo audit` (or `cargo deny check advisories`) as a
  separate step in both workflows — note the network constraint (github.com
  unreachable here; use a vendored/snapshotted DB or the codeload tarball,
  see V03-01 method); add `cargo tree -d` drift detection; add a frontend
  job with `npm audit --registry=https://registry.npmjs.org`.
- Regression validation: introduce a synthetic advisory (or run against a
  lockfile containing lopdf 0.34.0) and verify CI fails; then revert.
- Validation reports: [V03-01](../validations/Q-DEP-01/V03-01.md),
  [V02-01](../validations/Q-DEP-01/V02-01.md)

### Q-DEP-01-P3-07: live informational warnings — anyhow (direct dep) unsound, plus unmaintained/unsound crates in the polars and scraper stacks

- Priority: P3
- Confidence: high (audit output + reverse traces)
- Layer: framework + application (transitive deps)
- Evidence: CLI lockfile: `anyhow` 1.0.102 (RUSTSEC-2026-0190, unsound
  `Error::downcast_mut`, fix 1.0.103 — direct dep of `echo-agent-cli` and
  `echo-agent-app-core`); `lru` 0.12.5 (RUSTSEC-2026-0253, unsound
  `LruCache::pop` panic-safety, fix 0.18.2 — via ratatui); `event-listener`
  5.4.1 (RUSTSEC-2026-0221, fix 5.4.2 — polars); `memmap2` 0.9.10
  (RUSTSEC-2026-0186, fix 0.9.11 — polars-io); `bincode` 2.0.1
  (RUSTSEC-2025-0141, unmaintained — polars-utils); `fxhash` 0.2.1
  (RUSTSEC-2025-0057, unmaintained — scraper/web).
- Reachability: all six are linked in the shipped default binary (anyhow and
  lru on the direct/TUI path; the rest via polars/scraper).
- Expected invariant: directly-owned dependencies are patched for
  informational advisories; unmaintained crates are at least tracked.
- Observed behavior: the informational advisories are fixable with patch
  bumps except lru (0.18.2 is a major, needs ratatui to move) and bincode/
  fxhash (unmaintained — pin and track).
- Impact: informational level; the anyhow unsoundness requires the specific
  `context()` + `downcast_mut` pattern; the lru UAF requires a panic inside
  `pop`. Maintenance debt and latent memory-safety risk, no confirmed defect.
- Root cause: version lag on the direct anyhow dependency and transitive
  resolution to pre-fix releases.
- Direction: `cargo update -p anyhow -p event-listener -p memmap2` (patch
  fixes); track ratatui/lru upstream; add unmaintained crates to a review
  list in `docs/MASTER-PLAN.md`; re-audit after bumps.
- Regression validation: audit shows the warnings cleared; full CLI test
  suite green after the bumps.
- Validation reports: [V03-01](../validations/Q-DEP-01/V03-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---|---|---|
| V01 | Rust dependency-tree duplicates (`cargo tree -d` both workspaces + lockfile groups + reverse traces) | yes | passed | [V01-01](../validations/Q-DEP-01/V01-01.md) |
| V02 | Frontend dependency inventory (`npm ls`, lockfile parse, `npm audit` npmjs, `npm outdated`, import search) | yes | passed (advisory data from npmjs override; configured mirror 404 recorded) | [V02-01](../validations/Q-DEP-01/V02-01.md) |
| V03 | Advisory scan (`cargo audit` 0.22.2, DB snapshot 2026-08-12 via codeload tarball; 13 vulns CLI / 12 fw, per-crate reachability) | yes | passed | [V03-01](../validations/Q-DEP-01/V03-01.md) |
| V04 | License/native/build-script review (build.rs inventory, native C pullers, license scan of 1524 package entries) | yes | passed | [V04-01](../validations/Q-DEP-01/V04-01.md) |
| V05 | Cross-reference with existing findings (Q-STA-01-P3-03, B-BASE-01-P2-01/P2-02) | yes | passed | [V05-01](../validations/Q-DEP-01/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| Q-STA-01-P3-03: reqwest 0.12/0.13 and crossterm 0.28/0.29 coexist in the shipped CLI | current, **refined** | crossterm split confirmed linked (0.28 TUI stack / 0.29 polars); reqwest 0.13.3 is mobile-target-only via tauri and never links on desktop — the "two HTTP clients" wording overstates it (V01-01, V05-01) |
| Q-STA-01 V07: 38 (fw) / 76 (cli) multi-version lockfile names | current | re-verified exactly (V01-01) |
| B-BASE-01-P2-01: CLI CI never validates the shipped frontend | current | workflow still has no node/npm step (V05-01) |
| B-BASE-01-P2-02: CLI CI's framework input is unpinned | current | checkout still has no `ref` (V05-01) — the main supply-chain exposure on file |
| B-BASE-01: "all 16 manifest paths relative" | current | all `path = "../echo-agent"` / `"../../echo-agent"` (V01-01/V04-01) |
| Framework `data` feature excluded due to polars-ops build.rs failures (`echo-agent/Cargo.toml:59-63`) | current | note present; EKO enables `data`/`statistics` anyway (P3-04) |
| AGENTS.md CI gates do not include advisory scanning | current | no `cargo audit`/`cargo deny` in either workflow (P3-06) |

## Coverage And Uncertainty

- The advisory DB is a snapshot from 2026-08-12 (codeload tarball); anything
  published after the snapshot is out of scope. The DB fetch via github.com
  git is impossible in this environment; the snapshot method is documented in
  V03-01 and is the reason the scan could run at all.
- "LIVE" classification proves linking + feature/tool registration, not that
  the exact vulnerable function is exercised by EKO's usage patterns.
- The `gui` (Tauri) binary was classified statically (its crates are in the
  same lockfile); no gui build was executed here (Q-GUI-01 owns that).
- npm advisories come from the npmjs registry (the configured npmmirror
  mirror does not implement the audit endpoint); advisory sets are
  time-dependent.
- License review used declared `license` fields, not file-level headers of
  vendored text.

## Handoff

- Downstream tasks may rely on: the duplicate inventory and the crossterm/
  reqwest classification (V01-01); the frontend inventory incl. dead dompurify
  and toolchain advisories (V02-01); the full advisory table with per-crate
  reachability (V03-01); the build-script/native/license posture (V04-01);
  the current/fixed classification of Q-STA-01-P3-03 and B-BASE-01 P2-01/
  P2-02 (V05-01).
- Reports to read: the five validation reports above; Q-STA-01 (V07 + P3-03),
  B-BASE-01 (P2-01/P2-02) for the referenced findings.
- Conditions that make this report stale: either lockfile, either feature
  set (`echo-agent-app-core/Cargo.toml`), the frontend package manifest, or
  either reviewed commit changes; new advisories published after 2026-08-12
  may add to the tables.
- Follow-up task IDs: S-RDM-01 (roadmap items: P2-01 dependency bumps and
  audit gate; P3-01..07), Q-FW-01/Q-CLI-01 (re-run gates after the bumps),
  Q-WEB-01 (frontend build/test after manifest edits).
