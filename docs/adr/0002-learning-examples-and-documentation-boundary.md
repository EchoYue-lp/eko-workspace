# ADR 0002: Consolidate Learning Examples and Documentation Boundaries

- Status: Accepted
- Date: 2026-08-29

## Context

The framework workspace currently mixes several kinds of material: public
framework implementation and API documentation, numbered `demo_*.rs` teaching
examples, deterministic example contracts, and a separate Rust-learning crate.
The same workspace also contains `echo-agent-examples`, a small facade probe,
while the historical examples remain under `echo-agent/examples/`. This makes
it unclear whether an example is a framework test, a learning exercise, or a
full external-consumer scenario.

The application repository has a different responsibility: EKO product
policy, workspace identity, TaskRuntime persistence, GUI/TUI/CLI/channel
surfaces, and product workflows. Those concerns must not move into the
reusable framework merely because the application uses the framework.

Cargo supports package-local examples and integration tests, while an external
consumer crate is useful for checking that examples use only the public facade.
The project therefore needs one explicit learning/example owner without
weakening framework API tests or blurring the framework/application boundary.

## Decision

1. Rename the workspace package `echo-agent-examples` to
   `echo-agent-learning` and merge the existing `echo-rust-learning` package
   into it.
2. Move all numbered framework `demo_*.rs` files and their example fixtures
   into `echo-agent-learning/examples/`. Preserve their teaching-oriented
   structure, names, prerequisites, and progressive learning order.
3. Move deterministic `demo_*.rs` contract sources and their shared harness to
   `echo-agent-learning/tests/`. They remain executable contracts, but are not
   presented as ordinary Cargo examples when they do not contain `main`.
4. Keep framework unit tests, public-facade tests, doctests, and framework
   implementation documentation in `echo-agent`. Keep EKO product
   documentation in `echo-agent-cli`. Keep cross-repository plans, audits, and
   validation evidence in the superproject.
5. Put learning and example guides in `echo-agent-learning/docs/`, including
   the merged Rust-learning lessons and runnable demo instructions. Add new
   comprehensive examples there without changing the purpose of existing
   numbered demos.
6. Examples and learning contracts depend on `echo_agent` through its public
   facade. They must not become a second implementation or import EKO
   application internals.

## Alternatives

- Keep all demos in `echo-agent/examples/`: rejected because the framework
  package remains both API implementation and a large learning distribution,
  and external-consumer validation is mixed with framework targets.
- Move only the Rust lessons: rejected because numbered demos and lessons then
  have two different learning owners.
- Move all app-core code into `echo-agent`: rejected because workspace,
  product persistence, UI projection, review, and EKO policy are not reusable
  framework mechanisms.

## Consequences

- New learners have one crate and one documentation entry point, while the
  original `demo_*.rs` progression remains intact.
- Framework CI becomes smaller and focuses on framework tests and public API
  contracts; learning CI compiles and tests the complete example inventory.
- Example paths, `CARGO_MANIFEST_DIR` fixtures, Cargo feature forwarding,
  website links, and documentation contracts must be updated together.
- A small set of canonical framework examples may be added later only when it
  materially improves API documentation; they are still owned by
  `echo-agent-learning` under this decision.

## Verification

- `cargo metadata --no-deps` reports one `echo-agent-learning` workspace member
  and no `echo-agent-examples` or `echo-rust-learning` member.
- Learning examples and deterministic contracts compile through explicit
  feature-aware targets, and framework public API tests remain in `echo-agent`.
- Documentation and website link checks resolve the new learning paths.
