# Tig Ideology

Tig is an argument that source control should be rebuilt around work, visibility, and evidence rather than commits, branches, and repository-level publication.

Git is not the enemy. Git proved that content-addressed history, distributed source state, and cheap branching are powerful ideas. The problem is that modern software work now asks Git and GitHub to carry responsibilities they were not designed to model directly:

- agent-generated parallel attempts
- private security work inside otherwise public projects
- monorepos with mixed public and private packages
- continuous snapshots of messy work
- API-native editing by tools that do not need a full local checkout
- policy-controlled publication to different audiences

Tig should preserve the good parts of Git while changing the user-facing primitives.

## 1. Work Comes Before History

Developers do not experience work as a series of polished commits. They try things, break things, inspect behavior, switch context, return later, and keep only the useful parts.

Agents make this more obvious. A coding agent may create many intermediate states while solving one task. Those states are useful for recovery, comparison, and audit, but most of them are not meaningful history.

Tig should continuously capture work without forcing the actor to decide whether each state deserves to become a commit.

Implications:

- Snapshots should be automatic.
- Commits should be derivable artifacts, not the central workflow.
- The system should remember failed and abandoned attempts without polluting review history.
- Users should be able to create clean review units from messy work after the fact.

## 2. Editing Is Not Publishing

Git workflows often blur editing, sharing, and publishing. Pushing a branch, opening a pull request, or working in a public repo can expose work before it is ready or safe.

Tig treats publication as a separate act.

A human or agent should be able to edit freely in a private workspace, accumulate evidence, request review, and then intentionally publish selected state to a selected audience.

Implications:

- Workspaces should be private by default unless policy says otherwise.
- Review units should expose only the selected result, not every intermediate state.
- Security fixes can be developed and reviewed without instantly revealing exploitable details.
- Public source state should be a projection, not necessarily the whole project.

## 3. Open Source Is A Projection

Traditional hosting makes repository visibility too binary. A repo is public or private. A fork is public or private. This forces teams into awkward structures when they want part of a system to be open and part to remain private.

Tig should make open source a first-class projection:

```text
Project: runtime-platform
  Internal projection: everything
  Public projection: SDK, examples, docs
  Security projection: unreleased fixes and advisories
  Customer projection: customer-specific integrations
```

This lets one project contain mixed visibility without splitting into artificial repositories.

Implications:

- Visibility must be path-aware, package-aware, review-aware, and actor-aware.
- Publishing to a projection should run policy checks.
- A public projection must never accidentally include hidden source, secrets, private metadata, or restricted history.
- The system should make it easy to explain what the public sees and why.

## 4. Agents Are First-Class Actors

Agents should not pretend to be humans clicking GitHub buttons or typing Git commands. They need source-control operations that match how they work:

- create isolated attempts
- read and patch files through APIs
- run checks against exact snapshots
- compare multiple solutions
- attach evidence to proposed changes
- request human approval
- publish only when authorized

Humans and agents should use the same source model, but they do not need the same interface.

Implications:

- Every change should have an actor.
- Agents should receive scoped permissions, not full-repo trust by default.
- The system should compare attempts from multiple agents or humans against the same goal.
- Reviews should show evidence: what changed, what ran, what passed, what failed, and which snapshot those results belong to.

## 5. Workspaces Should Be Cheap Views

Git worktrees expose a real need: people and agents want multiple live views of the same project. The problem is that Git worktrees still require users to manage filesystem state, branch state, and synchronization manually.

Tig workspaces should be cheap logical views over source state.

Implications:

- Creating a workspace should be cheap enough to do for every task or agent attempt.
- Multiple workspaces should be able to share the same base without conflict.
- Updating a workspace from its target should be explicit and understandable.
- Local directories should be cacheable interfaces over workspace state, not the source of truth.

## 6. Evidence Belongs In Source Control

Current workflows often separate source state from the evidence that justifies it. CI logs live somewhere else. Agent transcripts live somewhere else. Benchmarks live somewhere else. Review discussion lives somewhere else.

Tig should attach evidence directly to the source states it describes.

Implications:

- A run should point to an exact snapshot.
- A review unit should show which checks passed on the selected source state.
- Failed runs should remain useful context, especially for agent attempts.
- Benchmarks, logs, artifacts, and approvals should be queryable source-control objects.

## 7. Compatibility Is A Bridge, Not A Cage

Tig should interoperate with Git because Git is everywhere. It should be able to import Git repositories, export Git commits, mirror to GitHub, and produce patches that existing tools understand.

But Git compatibility should not decide the internal model.

Implications:

- Git import/export is a product requirement.
- The internal system can still use workspaces, snapshots, attempts, projections, and publications.
- A clean Git commit can be generated from a review unit.
- GitHub can be treated as an external publication target during adoption.

## 8. Policy Should Be Programmable And Visible

Source control already enforces policy, but often indirectly: branch protections, CODEOWNERS, repo visibility, CI gates, and secret scanners. Tig should make policy explicit.

Examples:

```text
Agents can edit /src/runtime and /tests/runtime.
Agents cannot edit /billing without approval.
Security workspaces are private until release.
Public projections exclude /internal and /infra.
No publication can include files matching secret patterns.
Lockfile changes require maintainer approval.
```

Implications:

- Policies should be inspectable by humans and agents.
- Policy failures should explain exactly what blocked publication.
- Policies should apply before source state crosses visibility boundaries.
- Permissions should be scoped by actor, path, operation, projection, and lifecycle stage.

## 9. The Filesystem Is An Interface

Humans need editors. Build tools need directories. Existing language ecosystems expect files. Tig should support local mounted workspaces.

But the local filesystem should not be the only way to interact with source state.

Implications:

- Agents should be able to read, patch, and compare source state through APIs.
- Local directories can be materialized views of a workspace.
- The source-control service should understand edits independently of shell commands.
- A workspace should be usable from a web UI, CLI, local mount, or agent API.

## 10. The System Should Make The Right Thing Easier

Most Git pain comes from forcing users to manually protect themselves:

- remember to commit before switching tasks
- avoid leaking secrets
- keep branches updated
- manage worktrees
- clean up failed attempts
- decide what should be public
- reconstruct why a change was made

Tig should make the safer path the default path.

Implications:

- Losing work should be hard.
- Accidentally publishing private source should be hard.
- Comparing attempts should be easy.
- Creating a review from the latest passing state should be easy.
- Exporting to Git should be available, but not the everyday mental model.

## Product North Star

The first version of Tig should prove one thing:

> A human or agent can create isolated work, let the system capture every meaningful state, select the best passing snapshot, and publish a clean reviewable result without manually managing commits, branches, stashes, or worktrees.

Everything else should serve that loop.

