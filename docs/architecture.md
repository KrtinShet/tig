# Architecture Sketch

This is an early architecture sketch. It is intended to guide prototypes, not freeze implementation details.

## Components

```text
CLI / Agent API / Web UI
        |
        v
Workspace Service
        |
        v
Snapshot Engine ---- Object Store
        |
        v
Run Store --------- Review Service
        |
        v
Policy Engine ----- Projection Service
        |
        v
Git Import/Export Bridge
```

## Object Store

Stores immutable source objects:

- file contents
- directory trees
- snapshots
- metadata objects

The object store should make snapshots cheap and deduplicated.

## Workspace Service

Manages editable views over source state.

Responsibilities:

- create workspaces from snapshots
- materialize workspaces to local files when needed
- accept file writes and patches
- track actor and goal metadata
- create snapshots after meaningful operations

## Snapshot Engine

Creates complete source states from workspace contents.

Responsibilities:

- identify changed objects
- store snapshot metadata
- link snapshots to workspaces and attempts
- support diffing between snapshots
- support selecting latest passing or policy-valid states

## Run Store

Records execution evidence against exact snapshots.

Responsibilities:

- command
- actor
- environment summary
- start/end time
- status
- logs
- artifacts
- snapshot reference

The important rule: a run always belongs to a specific snapshot.

## Review Service

Creates reviewable packages from selected source states.

Responsibilities:

- compare source snapshot to target snapshot
- collect changed files
- attach run evidence
- collect comments and approvals
- expose review state to humans and agents
- prepare review unit for publication or Git export

## Policy Engine

Evaluates permissions and publication safety.

Responsibilities:

- actor permissions
- path/package rules
- projection rules
- required approvals
- required checks
- secret/publication checks
- explainable failures

## Projection Service

Maintains visible views of a project.

Responsibilities:

- define projection contents
- update projections through publication
- ensure hidden source is not exposed
- support public, internal, security, and customer views

## Git Import/Export Bridge

Provides compatibility with the existing ecosystem.

Responsibilities:

- import existing Git repositories
- map Git commits to snapshots
- export review units as Git commits or patches
- mirror selected publications to GitHub
- preserve enough metadata to round-trip where possible

## Data Relationships

```text
Project
  has many Workspaces
  has many Projections
  has many Policies

Workspace
  starts from Snapshot
  has many Changes
  has many Snapshots
  may belong to Attempt

Attempt
  has Goal
  has Actor
  has Workspace
  has many Runs
  may produce Review Unit

Run
  belongs to Snapshot
  belongs to Attempt or Workspace

Review Unit
  has source Snapshot
  has target Snapshot
  has Runs
  has approvals
  may become Publication

Publication
  updates Projection
  may export to Git
```

## Prototype Bias

The first implementation should optimize for learning:

- local storage before hosted storage
- simple JSON/SQLite metadata before distributed databases
- filesystem materialization before custom editor integrations
- CLI and agent API before web UI
- Git export before GitHub replacement

