# MVP

The first Tig MVP should prove the workflow before attempting to replace GitHub.

The goal is a local-first prototype that demonstrates:

> A human or agent can create isolated work, let the system capture snapshots, run checks against exact source states, select the best result, create a review unit, and export that result to Git.

## MVP Scope

The MVP should include:

- local Tig project initialization
- content-addressed object storage
- workspace creation from a base snapshot
- file read/write/patch operations
- automatic snapshot creation
- command runs attached to snapshots
- review unit creation from a selected snapshot
- basic policy checks before review/publication
- Git export for compatibility

## CLI Shape

The exact command names can change, but the first prototype should feel like this:

```bash
tig init
tig workspace create fix-auth-timeout
tig write /src/auth.ts --from patch.diff
tig run "npm test auth"
tig review create --from latest-passing --target main
tig git export --review current
```

The CLI is not the only intended interface. It is the fastest way to validate the model locally.

## Agent API Shape

Agents should be able to use the same operations without shelling out to Git:

```ts
createWorkspace({
  project: "runtime-platform",
  base: "main",
  goal: "Fix auth timeout",
  actor: "codex"
})

applyPatch({
  workspace: "fix-auth-timeout",
  patch: "..."
})

runCheck({
  workspace: "fix-auth-timeout",
  command: "npm test auth"
})

createReviewUnit({
  workspace: "fix-auth-timeout",
  from: "latestPassingSnapshot",
  target: "main"
})
```

## MVP Non-Goals

The MVP should not try to solve everything.

Non-goals:

- full GitHub replacement
- hosted multi-tenant service
- custom editor
- package registry
- complex merge UI
- enterprise permissions dashboard
- full cryptographic supply-chain system
- complete distributed protocol

These may matter later, but the first prototype should focus on the work-to-review loop.

## First Demo Scenario

The first demo should be simple:

1. Import or initialize a small codebase.
2. Create two workspaces for the same bug.
3. Apply different patches in each workspace.
4. Run tests in both.
5. Show that one snapshot passed and one failed.
6. Create a review unit from the passing snapshot.
7. Export the review unit as a Git commit or patch.

This proves the key difference from Git:

> Work can be messy and parallel, while the reviewed result is clean and intentional.

## What To Measure

The MVP should be judged by:

- how many Git concepts the user avoided
- whether failed attempts remain inspectable
- whether review evidence points to exact snapshots
- whether Git export is understandable
- whether an agent can use the API without manual branch management

