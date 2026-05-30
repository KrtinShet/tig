# Tessera

Tessera is an experimental version-control substrate for humans and coding agents.

The starting thesis: Git is a powerful object database, but the everyday workflow around commits, branches, worktrees, public repos, and pull requests is not the right primitive for agent-heavy software development.

Tessera explores a different model:

- Work is continuously captured as changes and snapshots.
- Workspaces are cheap editable views, not fragile branch/worktree rituals.
- Attempts connect code changes to goals, actors, checks, and evidence.
- Review units package useful results after the work is done.
- Projections control which parts of a project are visible to which audiences.
- Publication is intentional; editing is not publishing.
- Git export exists for compatibility, but does not define the internal model.

## Ideology

Tessera starts from a few beliefs:

- Source control should protect unfinished work instead of forcing teams to hide it in private forks, local branches, or separate repositories.
- Humans and agents should be able to work messily while the system preserves enough structure to review, compare, audit, and publish clean results.
- Visibility should be policy-driven. "Open source" should mean a project has a public projection, not that every file and every in-flight fix is public forever.
- Branches, worktrees, and commits should become compatibility concepts rather than the primary way users understand their work.
- Filesystems and Git remotes should be interfaces over source state, not the only place source state can live.

See [docs/ideology.md](docs/ideology.md) for the longer project philosophy.

## Documentation

- [Problem](docs/problem.md): what Git/GitHub do not model well enough for agent-heavy work.
- [Solution](docs/solution.md): the work-first model Tessera proposes.
- [Ideology](docs/ideology.md): the beliefs that should constrain product and engineering choices.
- [Primitives](docs/primitives.md): the core nouns in the system.
- [Workflows](docs/workflows.md): concrete human, agent, security, and projection examples.
- [MVP](docs/mvp.md): the first prototype scope and non-goals.
- [Comparisons](docs/comparisons.md): how Tessera relates to Git, GitHub, jj, and monorepo systems.
- [Security Model](docs/security-model.md): actors, policies, projections, and publication checks.
- [Architecture](docs/architecture.md): early component boundaries for the prototype.
- [Roadmap](docs/roadmap.md): staged path from docs to local prototype to hosted collaboration.

## Early Primitives

- **Project**: the top-level source, policy, review, and publication boundary.
- **Object Store**: immutable file contents and source-state objects.
- **Workspace**: an editable view over project state.
- **Change**: a tracked edit or operation inside a workspace.
- **Snapshot**: a complete project/workspace state at a point in time.
- **Attempt**: a purposeful line of work by a human or agent.
- **Run**: a command/check executed against an exact snapshot.
- **Review Unit**: a clean reviewable package derived from one or more snapshots.
- **Projection**: a policy-controlled visible view of a project.
- **Publication**: an intentional update to a projection or external system.

## Status

This repository is a blank-slate research and implementation workspace. The first milestone is to turn the primitive model into a concrete local prototype that can:

1. Create a project.
2. Create workspaces from a base snapshot.
3. Apply file changes through an API.
4. Record snapshots automatically.
5. Attach runs/check results to snapshots.
6. Produce a review unit from a selected snapshot.
7. Export the result to Git for compatibility.
