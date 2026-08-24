# <TASK-ID> / <VALIDATION-KEY> / Attempt <NN>

> Schema: validation-v2
> Validation key: <VALIDATION-KEY>
> Attempt: <positive integer>
> Status: passed | failed | inconclusive | not_run
> Validation date: YYYY-MM-DD
> Executor: <model/harness or person>
> `echo-agent` commit: <hash or not-applicable>
> `echo-agent-cli` commit: <hash or not-applicable>
> Worktree state: <clean or concise dirty paths>

## Claim

One falsifiable statement this execution checks.

## Method

Exact command or reproducible inspection steps. Include working directory,
feature flags, fixtures, environment prerequisites, and relevant tool versions.

```text
<exact command or numbered inspection procedure>
```

## Expected Result

Record the expectation before interpreting the output.

## Result

- Exit code: <integer or not-applicable>
- Duration: <when available>
- Summary: <concise factual result>
- Log/artifact: <absolute or repository-relative path, if created>

Do not paste large build logs, traces, or generated payloads into this report.

## Deviations

Unexpected behavior, partial coverage, nondeterminism, warnings, or environment
constraints. Write `None.` when there are none.

## Conclusion

State what this result supports and what it does not prove. Link findings that
use it.

## Follow-Up

Next attempt, new task ID, or `None.`. A rerun must create a new report rather
than replacing this one.
