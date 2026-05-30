# Roadmap

This roadmap is intentionally staged. Tessera should prove the local work model before trying to become a hosted collaboration platform.

## Phase 0: Model And Documentation

Goal: make the idea explainable.

Deliverables:

- problem statement
- solution model
- primitives
- ideology
- workflows
- MVP definition
- architecture sketch

Exit criteria:

- a new contributor can explain why Tessera exists
- the first prototype scope is clear

## Phase 1: Local Prototype

Goal: prove workspace, snapshot, run, and review-unit flow locally.

Deliverables:

- `tessera init`
- local object store
- local metadata store
- workspace creation
- file write/patch operation
- automatic snapshots
- command runs attached to snapshots
- review unit creation
- diff between review source and target

Exit criteria:

- a demo can show two attempts for one task
- one passing snapshot can become a review unit

## Phase 2: Agent API

Goal: let coding agents use Tessera without Git mechanics.

Deliverables:

- programmatic workspace creation
- read/write/patch API
- run API
- snapshot query API
- review unit API
- scoped actor permissions

Exit criteria:

- an agent can complete a bugfix workflow without invoking Git
- the human can inspect the attempt and evidence

## Phase 3: Git Import And Export

Goal: make Tessera compatible with existing projects.

Deliverables:

- import Git repository as Tessera project
- map Git tree to base snapshot
- export review unit as Git commit
- export patch file
- optional GitHub PR bridge

Exit criteria:

- Tessera can operate on a real Git project and return a normal Git-compatible result

## Phase 4: Policy And Projections

Goal: prove the visibility model.

Deliverables:

- projection definitions
- path-level visibility rules
- publication checks
- secret-pattern blocking
- actor permission scopes
- explainable policy failures

Exit criteria:

- public projection can exclude private source
- publication is blocked when policy fails

## Phase 5: Review Experience

Goal: make review units useful to humans.

Deliverables:

- review UI or rich CLI view
- changed-file browser
- run evidence display
- attempt comparison
- approvals
- comments

Exit criteria:

- a human can choose between multiple agent attempts without reading raw workspace internals first

## Phase 6: Hosted Collaboration

Goal: move from local prototype to shared source-control service.

Deliverables:

- hosted project service
- authenticated actors
- shared workspaces
- durable object storage
- projection hosting
- GitHub mirroring

Exit criteria:

- a small team can use Tessera as the coordination layer for agent-assisted development

## Open Questions

- What is the minimum viable storage format?
- Should snapshots be event-derived, tree-derived, or both?
- How much of Git import/export should be lossless?
- What policy language is expressive enough without becoming too complex?
- What local workspace materialization strategy is fastest and safest?
- What should the first hosted projection look like?

