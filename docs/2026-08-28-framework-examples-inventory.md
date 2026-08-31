# Framework Learning And Examples Inventory

This cross-repository inventory records the current ownership of framework
learning material and examples. It supersedes the earlier R2 inventory that
counted examples inside the `echo-agent` root package.

## Current Layout

| Material | Current owner | Count / purpose |
| --- | --- | --- |
| Numbered `demo_*.rs` walkthroughs | `echo-agent/echo-agent-learning/examples/` | 43 runnable teaching and integration walkthroughs |
| Rust learning chapters | `echo-agent/echo-agent-learning/examples/chapter_*.rs` and `src/` | 13 offline lessons plus shared lesson modules |
| Deterministic demo contracts | `echo-agent/echo-agent-learning/tests/example_contracts/` | 21 integration contracts in one harness |
| Comprehensive examples | `echo-agent/echo-agent-learning/examples/comprehensive_*.rs` | New multi-capability learning scenarios |
| Framework unit/public API contracts | `echo-agent/tests/` | Tests of framework implementation and facade, not teaching demos |
| Framework formal docs and ADRs | `echo-agent/docs/` | Public API, concepts, integrations, reference, and architecture decisions |
| EKO product docs | `echo-agent-cli/docs/` | Application architecture, product policy, surfaces, and configuration |
| Cross-repository plans and evidence | `lp-agent/docs/` | Audits, migration history, validation evidence, and coordination material |

The package is named `echo-agent-learning` and is a non-published workspace
member. The old `echo-agent-examples` and `echo-rust-learning` package names no
longer exist.

## Ownership Rules

1. Existing numbered demos remain learning material. Their names, progressive
   order, and explanatory structure should be preserved.
2. A source without `main` that is intended to assert deterministic behavior
   belongs in `echo-agent-learning/tests/`, not in the framework library's
   production test tree.
3. Examples use only the public `echo_agent` facade. They must not import
   split-crate implementation paths or EKO application internals.
4. New comprehensive scenarios belong in `echo-agent-learning/examples/` and
   must document feature, provider, credential, platform, and filesystem
   prerequisites.
5. Framework docs describe reusable APIs and behavior. Learning docs explain
   how to read and use those APIs. EKO docs describe EKO-specific policy and
   runtime ownership.

## Feature And Runtime Boundaries

The learning manifest forwards optional framework features so examples can be
compiled independently or with `--all-features`. External prerequisites are
not silently treated as passed: examples must report missing credentials or
services clearly. The learning crate does not add a second tool, task, memory,
or runtime implementation.

`demo08_external_skills.rs` and `demo47_enterprise.rs` resolve their bundled
skill fixtures from `CARGO_MANIFEST_DIR`. `demo58_git_worktree.rs` and
`demo59_code_search.rs` resolve the enclosing framework repository when they
need to inspect the checkout, preserving their original behavior after the
package move.

## Verification

The following checks are the learning-package contract:

```text
cargo metadata --no-deps --format-version 1
cargo test -p echo-agent-learning --locked
cargo test -p echo-agent-learning --all-features --locked -- --test-threads=1
cargo check -p echo-agent-learning --all-targets --all-features --locked
cargo clippy -p echo-agent-learning --all-targets --all-features --locked -- -D warnings
cargo fmt --all -- --check
```

The framework package separately verifies its library, public facade, doctests,
and framework-specific integration behavior. Website source synchronization
must consume the framework's formal docs, not the learning package's generated
or historical inventory.
