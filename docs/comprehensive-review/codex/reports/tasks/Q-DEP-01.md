# Q-DEP-01: Dependency, supply-chain, and license health

> Status: complete
> Reviewer: Codex primary reviewer
> Executor: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: 3aa7929928442aab91e4dce9c426d909a5f0a1ab
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both repositories externally dirty; all adopted source and dependency evidence came from committed `HEAD` blobs, and the live CLI `Cargo.lock` was excluded

## Question

Are duplicate versions, stale or unmaintained dependencies, build scripts,
native dependencies, licenses, and advisories understood for both repositories?

## Scope

- Both committed Rust workspace manifests and lockfiles.
- The committed EKO frontend manifest and npm lockfile.
- CI dependency-policy coverage, package licenses, registry/source pinning,
  install/build scripts, native dependencies, and A2A JWT integration.
- Static dependency health only. Network-backed advisory databases and all
  compilation or tests were explicitly prohibited for this review phase.

## Inputs And Isolation

The root `AGENTS.md`, exact `TASKS.md` card, reporting protocol, Codex README,
and completed `B-BASE-01` dependency report were read. Framework source was read
only with `git show HEAD:path` or `git grep HEAD`; the externally changed CLI
lockfile was never adopted. V02-01 and V03-01 preserve two inaccurate command
attempts; corrected attempts are the only accepted evidence.

## Findings

### Q-DEP-01-P1-01 - The public RS256 JWT configuration always constructs an HMAC decoding key

`JwtConfig::rs256` stores a PEM RSA/EC public key and selects `RS256`, and
`with_algorithms` can publish an arbitrary mixed algorithm list. The only token
validator nevertheless always constructs `DecodingKey::from_secret`. A caller
using the advertised asymmetric configuration therefore cannot validate a
normal RS256 token; a mixed list also has no algorithm-specific key authority.
This is not merely an unused EKO path: it is a public framework capability under
the public `a2a` feature.

Impact: an A2A deployment can enable documented RSA authentication and reject
every legitimate token, while the configuration shape falsely implies that
algorithm selection and key material are coherently paired.

Recommendation: replace the single `secret + Vec<Algorithm>` model with a typed
key/algorithm enum (`Hmac`, `RsaPem`, and any separately supported EC form),
construct the matching `DecodingKey`, reject heterogeneous algorithm families,
and add positive/negative protocol fixtures when dynamic validation resumes.

Evidence: [V06](../validations/Q-DEP-01/V06-01.md).

### Q-DEP-01-P2-02 - Dependency risk and license policy are not executable, while `full` enables a dependency the manifest itself marks unsafe for production

The framework manifest says `jsonwebtoken` v9 has an algorithm-confusion/JWT
forgery advisory and must be migrated before production. The public `a2a`
feature enables it, and `full` includes `a2a`. Neither repository has a
`cargo-deny`/`cargo-audit`/license policy file or CI advisory/license step; the
frontend likewise has no audit/license gate. All workspace packages declare
MIT and all 349 non-root npm package entries carry a license field, but that is
inventory, not policy enforcement, and Rust lockfiles do not encode licenses.

Impact: the project cannot establish that a release dependency closure meets an
advisory or license policy. The repository's own security exception can enter a
normal `full` framework build without an explicit waiver, expiry, or CI failure.
Whether the pinned `jsonwebtoken 9.3.1` is actually affected by the cited
advisory remains inconclusive because the required current advisory database was
not queried; the governance contradiction is source-conclusive regardless.

Recommendation: add `cargo-deny` (advisories, bans/duplicates policy, sources,
and licenses) for both Rust closures plus an npm audit/license policy; make any
exception exact-version, reasoned, owned, and time-bounded. Resolve or correct
the JWT manifest warning before keeping `a2a` inside `full`.

Evidence: [V03](../validations/Q-DEP-01/V03-02.md), [V04](../validations/Q-DEP-01/V04-01.md), [V08](../validations/Q-DEP-01/V08-01.md).

### Q-DEP-01-P3-03 - The frontend declares `@tailwindcss/vite` twice with divergent ranges

The production dependency table requests `^4.1.4`, while devDependencies
requests `^4.1.8`; the lockfile currently resolves one `4.2.2` package. npm's
current resolution masks the ambiguity, but ownership and update intent are
unclear and future tooling can report or resolve the duplicate differently.

Recommendation: keep the build plugin only in `devDependencies` with one range.

Evidence: [V02](../validations/Q-DEP-01/V02-02.md), [V09](../validations/Q-DEP-01/V09-01.md).

## Dependency Inventory

| Area | Static result | Classification |
|---|---|---|
| Framework Rust duplicate-version families | 38 | mostly platform/transitive; observable debt, not 38 defects |
| EKO Rust duplicate-version families | 76 | GUI + framework closure is large; prioritize direct/ABI/native boundaries |
| Registry source integrity | 613 framework and 893 EKO registry packages, each with checksum; zero Git sources | positive |
| Frontend packages | 349 non-root entries; zero Git-resolved; all have a license field | positive inventory |
| Frontend install scripts | `esbuild` required, `fsevents` optional; both MIT and lock-pinned | understood native/install boundary |
| Rust native `*-sys` closure | SQLite/compression/TLS in framework; GUI/WebKit/platform/oniguruma plus compression/TLS in EKO | feature/platform-derived and partly CI-provisioned |
| Framework SQLite | optional, bundled implementation | valid reusable framework capability; EKO does not enable SQLite |
| Polars build-script limitation | explicitly documented and excluded from docs.rs, but `full` still selects data | tracked compatibility caveat, not independently executed here |

The raw duplicate counts are not a recommendation to force one version. A
future `cargo tree -d` review should first prioritize direct duplicates crossing
public types, native libraries, crypto/TLS, parsers, and high-size closures.

## Validation Matrix

| ID | Claim | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Commits, task boundary, dependency, and dirty-source isolation | yes | passed | [V00](../validations/Q-DEP-01/V00-01.md) |
| V01 | Rust duplicate-version inventory | yes | passed/classified | [V01](../validations/Q-DEP-01/V01-01.md) |
| V02 | Frontend dependency/install-script/license inventory | yes | passed after corrected attempt | [V02-01](../validations/Q-DEP-01/V02-01.md), [V02-02](../validations/Q-DEP-01/V02-02.md) |
| V03 | Advisory and supply-chain policy coverage | yes | failed after corrected attempt | [V03-01](../validations/Q-DEP-01/V03-01.md), [V03-02](../validations/Q-DEP-01/V03-02.md) |
| V04 | Workspace and transitive-license accountability | yes | failed | [V04](../validations/Q-DEP-01/V04-01.md) |
| V05 | Native dependency and build/install-script boundary | yes | passed/classified | [V05](../validations/Q-DEP-01/V05-01.md) |
| V06 | A2A JWT algorithm/key dependency contract | yes | failed | [V06](../validations/Q-DEP-01/V06-01.md) |
| V07 | Lockfile source/checksum integrity | yes | passed | [V07](../validations/Q-DEP-01/V07-01.md) |
| V08 | Current network-backed advisory databases | future | not_run | [V08](../validations/Q-DEP-01/V08-01.md) |
| V09 | Direct frontend declaration uniqueness | yes | failed | [V09](../validations/Q-DEP-01/V09-01.md) |
| V99 | Links, headers, IDs, isolation, and status | yes | passed after corrected attempt | [V99-01](../validations/Q-DEP-01/V99-01.md), [V99-02](../validations/Q-DEP-01/V99-02.md) |

## Coverage And Uncertainty

Static lockfile analysis can enumerate resolved versions, checksums, declared
licenses, and install/build boundaries, but it cannot establish current advisory
status, package maintenance state, or license compatibility. Those facts are
intentionally not guessed from model memory. V08 defines the future online gate.
No build, test, package-manager install, or network command was run.

## Handoff

- Fix the typed A2A algorithm/key contract before advertising RS256.
- Establish executable advisory/source/license policy before treating either
  dependency closure as release-health evidence.
- Use duplicate-version budgets by risk class and delta, not a blanket single-
  version rule.
- Remove the duplicate frontend Tailwind plugin declaration.
