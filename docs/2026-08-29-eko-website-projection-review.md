# EKO Website Projection Review

Review date: 2026-08-29

## Scope

The website EKO projection was previously reviewed against
`echo-agent-cli@eaeb39d`. This review covers the application changes through
`echo-agent-cli@bd2fea9` (with behavior-affecting R4 code at
`echo-agent-cli@0e762ab`):

- `echo-agent-app-core/src/tasks/task_runtime/command_cells.rs` closes the
  `watch_cell` snapshot/live-owner window with durable owner checks, an
  observation lease, retained terminal dispatch, and TaskRun current-turn
  identity validation.
- `docs/adr/0011-boot-inbox-recovery-authority.md` and
  `docs/architecture.md` document that recovery behavior and its existing
  framework/application boundary.
- `examples/lh6_product_soak.rs` adds explicit acceptance-tier handling for
  soak validation; it does not change the EKO runtime contract.
- The remaining documentation changes update current revisions, learning
  package references, and the application status ledger only.

## Projection Impact

No EKO projection file requires a semantic change. The existing pages already
state that EKO preserves cursor identity across restart, validates cold
workspace scope, emits one typed terminal fact across surfaces, uses durable
TaskRuntime/Chat state, and keeps the framework/application boundary explicit.
The reviewed implementation details refine those guarantees without adding a
new user-facing capability or changing the SQLite, Subagent, TaskRun, or
surface-parity claims.

The website manifest therefore advances its application `reviewedRevision` to
`bd2fea9` while retaining the same projection content. The application
repository and its maintained docs remain authoritative for exact behavior.

## Verification

- `npm run docs:check:source`
- `npm run site:check`
- `npm test`
