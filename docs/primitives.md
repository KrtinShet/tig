# Tessera Primitive Model

Tessera is built around work rather than commits.

## Project

A project is the durable boundary for source state, policies, workspaces, reviews, and publications.

Unlike a Git repository, a project does not have one global public/private setting. Visibility is controlled through projections and policies.

## Object Store

The object store keeps immutable file contents and source-state objects.

Users and agents should not need to think in object hashes during normal work, but the system should preserve every meaningful state for auditability, comparison, and rollback.

## Workspace

A workspace is an editable view of a project at a base snapshot.

Workspaces replace most branch and worktree usage. A human or agent should be able to create many isolated workspaces cheaply, work in them concurrently, and select useful results later.

## Change

A change is an edit operation or grouped source mutation in a workspace.

Examples:

- Update a file.
- Add a file.
- Delete a file.
- Rename a path.
- Apply a patch.

Changes are captured continuously. The user does not need to stage them manually.

## Snapshot

A snapshot is a complete workspace state at a point in time.

Snapshots can be created automatically after edits, commands, checks, or agent steps. A snapshot is similar to a commit in that it can identify a complete tree, but it is not necessarily a human-authored history unit.

## Attempt

An attempt is a purposeful line of work.

Attempts connect a goal, actor, workspace, snapshots, runs, and outcome. This lets the system compare multiple humans or agents working on the same task.

## Run

A run records command execution against an exact snapshot.

Runs attach evidence to source state:

- command
- environment
- result
- logs
- artifacts

This makes "tests passed" refer to a specific source state rather than a vague workspace condition.

## Review Unit

A review unit is the package a human or policy system reviews.

It can be derived from a selected snapshot or composed from selected changes. It is intentionally cleaner than the messy path that produced it.

## Projection

A projection is a visible view of a project for a specific audience.

Examples:

- Internal projection.
- Public open-source projection.
- Customer projection.
- Security-team projection.

Projections make open source a policy-controlled view, not a guarantee that every file and every in-flight change is public forever.

## Publication

Publication is the act of making selected source state visible in a projection or external system.

Examples:

- Publish a review unit internally.
- Publish a package to the public projection.
- Export a Git branch or commit.
- Release to a package registry.

Editing is not publishing. Publishing is intentional.

## Conceptual Flow

```text
Project
  owns Object Store
  owns Policies
  owns Projections
  owns Workspaces

Workspace
  starts from Snapshot
  receives Changes
  produces Snapshots
  belongs to Attempt or Actor

Attempt
  has Goal
  uses Workspace
  produces Snapshots
  records Runs
  may create Review Unit

Review Unit
  compares selected Snapshot with target Snapshot
  collects comments and approvals
  may become Publication

Publication
  updates Projection
  may export to Git/GitHub/package registries
```

