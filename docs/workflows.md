# Workflows

This document shows how humans and agents should experience Tessera.

## Agent Fixes A Bug

```text
Task: fix flaky scheduler test
Base: main@snapshot-100
Actor: Codex
```

1. Tessera creates a workspace.
2. The agent reads relevant files through the API or local mount.
3. The agent applies a patch.
4. Tessera records a snapshot.
5. The agent runs tests.
6. Tessera records the run against the exact snapshot.
7. The agent edits again if needed.
8. Tessera records another snapshot.
9. The agent proposes a review unit from the latest passing snapshot.
10. A human reviews and approves.
11. Tessera publishes the selected result.

Example state:

```text
Attempt: codex-fix-scheduler
  Workspace: codex-fix-scheduler
  Snapshot s1: initial patch
    Run: npm test scheduler -> failed
  Snapshot s2: cleanup timers
    Run: npm test scheduler -> passed
  Review Unit: from s2 to main@snapshot-100
```

## Multiple Agents Try The Same Task

```text
Goal: reduce API latency
```

Tessera creates multiple attempts:

```text
Attempt A: Claude
  Strategy: cache expensive query
  Result: tests pass, benchmark 15% faster

Attempt B: Codex
  Strategy: add database index
  Result: tests pass, benchmark 40% faster, migration required

Attempt C: human
  Strategy: remove redundant fetch
  Result: tests pass, benchmark 10% faster, smallest diff
```

The reviewer can compare:

- changed files
- test results
- benchmark results
- policy violations
- review risk
- implementation size

The selected attempt becomes a review unit. The others remain archived evidence.

## Human Works Normally

A human should not need to learn agent-specific workflow.

```text
tessera workspace create billing-cleanup
```

The workspace can be mounted locally and opened in an editor. Tessera captures snapshots automatically while the human works.

When ready:

```text
Latest snapshot: s8
Latest passing snapshot: s6
Changed files: 5
Review suggestion: create review from s6
```

The human can choose a clean state without manually stashing, rebasing, or rewriting commits.

## Security Fix Is Hidden Until Release

```text
Workspace: private-security-hotfix
Policy: visible only to security team
```

The fix can be developed, tested, and reviewed privately.

Publication can later happen in stages:

1. publish advisory to security projection
2. publish patch to internal projection
3. release package
4. publish final source to public projection

The public projection does not expose the exploit details while the fix is still in progress.

## Public SDK Inside Private Monorepo

```text
Project: runtime-platform
  /packages/sdk-public
  /packages/orchestrator-private
  /docs
  /infra
```

The public projection includes:

```text
/packages/sdk-public
/docs
/examples
```

It excludes:

```text
/packages/orchestrator-private
/infra
/security
```

This lets a project be meaningfully open source without splitting into artificial repositories.

## Latest Passing Snapshot Becomes Review

An agent or human may create many snapshots:

```text
s1: first patch, tests fail
s2: second patch, tests pass
s3: refactor, tests fail
s4: debug logging, tests pass but messy
```

The review unit can be created from `s2` instead of the latest state.

This is a key Tessera behavior: the system preserves messy work but lets review use the best selected state.

