# CROSS-QUALITY-REMEDIATION / FW-DENY / Attempt 01

> Schema: validation-v2
> Validation key: FW-DENY
> Attempt: 1
> Status: passed
> Validation date: 2026-08-17
> Executor: Codex framework_quality_fix agent using the local shell
> `echo-agent` commit: 356866c7195ef2d205d318b39098538182ddc118
> `echo-agent-cli` commit: not-applicable (framework-only validation)
> Worktree state: dirty with the intended framework remediation diff; the validated tree was subsequently committed as the recorded `echo-agent` commit

## Claim

The locked all-feature dependency graph satisfies the executable advisory,
license, source and dependency-ban policy in `deny.toml`.

## Method

Working directory:
`echo-agent/.worktrees/quality-review-20260816`.

```text
cargo deny --workspace --all-features --locked check
```

The cargo-deny executable version was not recorded.

## Expected Result

The command exits 0 with advisories, bans, licenses and sources all accepted by
the exact repository policy.

## Result

- Exit code: 0
- Duration: not recorded
- Summary: advisories OK, bans OK, licenses OK and sources OK. Configured duplicate-version observations remained warnings rather than failures.
- Log/artifact: policy is committed at `echo-agent/deny.toml`.

## Deviations

The cargo-deny executable version was not captured. Multiple dependency
versions are intentionally configured as warnings; wildcard dependencies and
unknown registries/git sources remain denied.

## Conclusion

This execution supports the license, source and dependency-policy portion of
`Q-DEP-01-P2-02`. It does not replace cargo-audit's independent RustSec warning
gate, which is covered by FW-AUDIT.

## Follow-Up

None before the advisory exception review date recorded by FW-AUDIT.
