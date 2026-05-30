# Comparisons

Tessera is not the first system to question Git ergonomics. This document explains the intended distinction.

## Git

Git is a distributed content-addressed version-control system.

Strengths:

- fast local history
- distributed collaboration
- efficient object storage
- powerful branching and merging
- enormous ecosystem

Limitations Tessera targets:

- commits are the main user-facing history unit
- branches are overloaded for tasks, releases, review, and experiments
- worktrees require manual filesystem management
- visibility is mostly handled outside Git
- evidence and review context are not first-class objects
- API-native agent workflows are awkward

Tessera should reuse Git compatibility where useful, but not inherit Git's user model as a constraint.

## GitHub

GitHub is the dominant collaboration layer around Git.

Strengths:

- pull requests
- issue tracking
- code review
- permissions
- CI integration
- open-source network effects

Limitations Tessera targets:

- repository-level public/private visibility is too coarse
- pull requests are branch-shaped
- in-flight work is awkward to keep selectively private
- review evidence is scattered across CI, comments, commits, and external tools
- multiple agent attempts are not a native concept

Tessera can export to GitHub during adoption, but the long-term model is broader than GitHub PRs.

## jj

jj improves the local change-management experience by making work more fluid than traditional Git.

Strengths:

- automatic working-copy commits
- easier history editing
- less staging-area friction
- better local workflow for evolving changes

What Tessera learns from jj:

- work should be captured continuously
- users should not have to manually create polished commits at every step
- history manipulation should be less frightening

Where Tessera aims beyond jj:

- agent attempts as first-class objects
- policy-controlled projections
- path/package/review-level visibility
- run evidence attached to snapshots
- review units independent from branch mechanics
- API-native source-control operations

## Perforce/Monorepo Systems

Large organizations have long used centralized systems and monorepo tooling to solve scale, visibility, and workspace problems.

What Tessera learns:

- large source trees need strong workspace and visibility models
- source access often needs path-level policy
- local checkout is not always the right source of truth

Where Tessera differs:

- agent-native workflows are central
- public/private projections are part of the product model
- Git compatibility remains important for adoption
- review evidence and attempts are first-class

## What Tessera Is Not

Tessera is not:

- a Git wrapper with nicer commands
- a GitHub skin
- only an AI coding tool
- only a monorepo tool
- only a security product
- a custom editor

Tessera is a source-control substrate where humans and agents create work, evidence, reviews, and publications through higher-level primitives.

