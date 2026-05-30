# Problem

Git is excellent at storing and exchanging source history. GitHub is excellent at making Git collaborative. The problem is that the modern software workflow has expanded beyond the primitives those systems expose to users.

Tessera starts from the claim that the pain is not a collection of small UX problems. The deeper issue is that source control is still organized around commits, branches, repository-level visibility, and local filesystem operations.

## Commits Force Premature History

Git asks users to decide when work deserves to become history.

That is not how work actually happens. Humans try ideas, break things, switch context, fix partial issues, and clean up later. Agents make this even more obvious because they may create many intermediate states while solving one task.

Useful source control should capture work continuously and let people decide later which state is worth reviewing or publishing.

## Branches Are A Weak Model For Parallel Work

Branches are useful pointers, but they are a poor user-facing abstraction for many concurrent attempts.

A team using humans and agents often wants to say:

```text
Create ten isolated attempts from this task.
Let each attempt edit freely.
Run checks on each exact result.
Compare outcomes.
Keep the best state.
Discard or archive the rest.
```

Git can support parts of this, but the user has to manage branch names, worktrees, rebases, stashes, cleanup, and synchronization.

## Worktrees Expose The Need But Not The Right Primitive

Git worktrees exist because users need multiple live checkouts of the same project. The implementation still makes the user manage local directories and branch constraints.

Agent-heavy development needs cheap logical workspaces that can be created for every task, every attempt, and every experiment without making filesystem management the user's job.

## Visibility Is Too Coarse

GitHub usually makes a repository public or private. Real projects need a more nuanced model:

- public packages inside a private monorepo
- private security fixes inside otherwise open projects
- hidden in-flight review work
- customer-specific integrations
- contractors with path-limited access
- agents with permission to edit only scoped areas

Teams work around this by splitting repositories, duplicating code, hiding work in private forks, or delaying publication.

## Editing And Publishing Are Too Coupled

In many workflows, pushing a branch or opening a pull request can expose work before it is ready. This is especially dangerous for security fixes and sensitive infrastructure work.

Source control should separate:

- editing
- sharing with selected reviewers
- creating a reviewable unit
- publishing to a visible audience
- exporting to external systems

Editing should not imply publication.

## Evidence Is Disconnected From Source State

The reason a change is trusted often lives outside source control:

- CI logs
- benchmark results
- agent transcripts
- local test output
- review approvals
- generated artifacts

This makes it difficult to answer a precise question:

> Which exact source state passed which exact checks, and why did we choose it?

Tessera treats evidence as part of the source-control model.

## Filesystem And Shell Assumptions Limit Agents

Git assumes a real local checkout and command-line interaction. Humans still need local files, but agents should not be forced to pretend that every source-control operation is a shell session.

Agents need direct operations:

- read file
- write file
- apply patch
- create workspace
- create snapshot
- run check
- compare attempts
- create review unit
- request publication

The filesystem should be one interface over source state, not the only interface.

## Summary

The problem is not that Git stores data badly. The problem is that Git and GitHub expose the wrong everyday primitives for a world of agents, mixed-visibility projects, continuous work capture, and policy-controlled publication.

