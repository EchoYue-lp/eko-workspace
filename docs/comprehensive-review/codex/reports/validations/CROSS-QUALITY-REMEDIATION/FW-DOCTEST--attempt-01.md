# CROSS-QUALITY-REMEDIATION / FW-DOCTEST / Attempt 01

> Schema: validation-v2
> Validation key: FW-DOCTEST
> Attempt: 1
> Status: passed
> Validation date: 2026-08-17
> Executor: Codex framework_quality_fix agent using the local shell
> `echo-agent` commit: 356866c7195ef2d205d318b39098538182ddc118
> `echo-agent-cli` commit: not-applicable (framework-only validation)
> Worktree state: dirty with the intended framework remediation diff; the validated tree was subsequently committed as the recorded `echo-agent` commit

## Claim

Public Rust documentation examples across every workspace package compile and
run under the all-features graph.

## Method

Working directory:
`echo-agent/.worktrees/quality-review-20260816`.

```text
cargo test --workspace --doc --all-features --locked
```

## Expected Result

The workspace doctest command exits 0 with no failed runnable or compile-fail
documentation example.

## Result

- Exit code: 0
- Duration: not recorded
- Summary: root 81 passed and 24 ignored; `echo_core` 11 passed and 2 ignored plus 7 compile-fail cases passed; `echo_execution` 1 passed and 2 ignored; `echo_integration` 13 passed; `echo_macros` 10 ignored; `echo_orchestration` 11 passed and 6 ignored; `echo_state` 14 passed; `echo_tools` had no doctests. No doctest failed.
- Log/artifact: no separate log artifact was retained.

## Deviations

Ignored doctests remained ignored by their source annotations; this command did
not opt into ignored examples. Duration was not recorded.

## Conclusion

This execution closes the previously unexecuted framework doctest evidence for
`B-ARCH-01-P2-04` and `Q-DOC-01-P2-02`. It does not validate Markdown links;
that is covered by FW-DOC-CONTRACT.

## Follow-Up

None.
