# Solution

Tig replaces commit-first source control with work-first source control.

The core idea:

> Humans and agents work in isolated workspaces. Tig captures meaningful states automatically. Checks and evidence attach to exact snapshots. Clean review units are created from selected states. Publication happens intentionally through policy-controlled projections.

## The New Flow

Traditional Git workflow:

```text
edit files -> stage -> commit -> branch -> push -> pull request -> merge
```

Tig workflow:

```text
create workspace -> edit freely -> snapshot automatically -> run checks -> select state -> create review unit -> publish projection
```

Git commits can still be exported, but they are no longer the main mental model.

## Workspaces Replace Branch Rituals

A workspace is an editable view of source state.

Workspaces can be created for humans, agents, tasks, experiments, security fixes, or review preparation. They are cheap enough to use casually.

```text
main
agent-fix-login-race
agent-fix-login-race-alt
human-billing-cleanup
private-security-hotfix
```

Each workspace can produce snapshots and runs without forcing the user to create named commits.

## Snapshots Replace Manual Save Points

A snapshot is a complete state of a workspace.

Snapshots can be created after:

- file edits
- patch applications
- agent steps
- commands
- test runs
- review creation
- publication

Users can later choose the latest passing snapshot, an earlier simpler snapshot, or a composed result.

## Attempts Make Agent Work Comparable

An attempt connects a goal, actor, workspace, snapshots, runs, and outcome.

This lets Tig compare several solutions to the same task:

```text
Goal: fix scheduler race

Attempt A: Claude
  tests: pass
  diff: large
  risk: medium

Attempt B: Codex
  tests: pass
  diff: small
  risk: low

Attempt C: human
  tests: fail
  diff: small
  risk: low
```

The user can select the best attempt instead of manually comparing branches.

## Review Units Replace Pull Requests As Branch Wrappers

A review unit is a clean package of selected work.

It may come from:

- one snapshot
- selected changes
- a composed state from multiple attempts
- an imported Git branch

The review unit shows the result and evidence without forcing reviewers to understand every messy intermediate step.

## Projections Replace Repository-Level Visibility

A projection is a visible view of a project for a specific audience.

```text
Project: runtime-platform
  Internal projection: all source
  Public projection: SDK, docs, examples
  Security projection: embargoed fixes
  Customer projection: customer adapter
```

Publication updates a projection. Editing a workspace does not.

## Policy Controls Movement Between States

Policy decides who can do what and what can become visible.

Examples:

```text
Agents can edit /src/runtime and /tests/runtime.
Agents cannot publish without human approval.
Public projection excludes /internal and /infra.
Security workspaces remain private until release.
Lockfile changes require maintainer approval.
No publication can include files matching secret patterns.
```

Policy is not only authorization. It is also publication safety.

## Git Compatibility

Tig should interoperate with Git:

- import a Git repository
- materialize a workspace as local files
- export a review unit as a Git commit
- mirror selected publications to GitHub
- produce patches for existing tools

But Git is a bridge, not the internal product model.

## Success Criteria

Tig is working if a human or agent can:

1. Start from an existing codebase.
2. Create isolated work without branch/worktree management.
3. Let the system capture meaningful states automatically.
4. Run checks against exact states.
5. Compare attempts.
6. Create a clean review unit from the chosen result.
7. Publish only what policy allows.
8. Export to Git when needed.

