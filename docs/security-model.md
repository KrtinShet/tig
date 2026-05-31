# Security And Visibility Model

Tig treats visibility as a first-class source-control concern.

The core rule:

> Editing is private by default. Publication is explicit and policy-checked.

## Actors

Every operation has an actor.

Actors may include:

- humans
- agents
- services
- CI systems
- import/export bridges

Actors receive scoped permissions rather than blanket repository access.

Example:

```text
Codex can:
  read /src/runtime
  read /tests/runtime
  edit /src/runtime
  edit /tests/runtime
  run checks
  create review units

Codex cannot:
  read /billing
  read /infra/secrets
  publish to public projection
  approve its own review unit
```

## Policies

Policies define what actors can do and what source state can cross boundaries.

Policy dimensions:

- actor
- path
- package
- workspace
- attempt
- review unit
- projection
- operation
- lifecycle stage

Example policies:

```text
Security workspaces are visible only to the security team.
Public projections exclude /internal, /infra, and /security.
Agent-created review units require human approval.
No publication can include .env files.
Lockfile changes require maintainer approval.
Customer projections can include only approved customer adapters.
```

## Projections

A projection is a visible view of a project.

Examples:

```text
Internal projection:
  all source visible to employees

Public projection:
  SDK, docs, examples

Security projection:
  embargoed fixes and advisories

Customer projection:
  customer-specific adapter and docs
```

Publishing updates a projection. Workspaces do not automatically appear in projections.

## Publication Checks

Before source state becomes visible, Tig should check:

- actor is allowed to publish
- target projection allows the paths
- required reviews are complete
- required runs passed
- secret patterns are absent
- restricted metadata is not exposed
- hidden history is not reachable from the public projection

Policy failures should be explainable:

```text
Publication blocked:
  /infra/deploy.ts is not allowed in public projection.
  Review requires approval from runtime-maintainers.
  Latest required run "npm test" failed on snapshot s42.
```

## Security Fix Flow

Security fixes should support embargoed development.

```text
Workspace: security-fix-auth-bypass
Visible to: security team
Target: internal main
Public status: hidden
```

When ready:

1. create private review unit
2. run checks
3. approve inside security projection
4. publish patched release
5. publish public source after embargo lifts

## Secret Handling

Tig should assume secret leaks are a publication-boundary problem, not just a scanner problem.

The system should:

- block `.env` and known secret patterns from public projection
- support private configuration references without storing raw secrets in source
- prevent hidden source from becoming reachable through exported history
- make projection contents inspectable before publication

## Trust Model

The early trust model is simple:

- humans can create and approve work according to policy
- agents can propose work within scoped permissions
- agents cannot publish sensitive work by default
- runs provide evidence but do not replace approval
- policy determines what can become visible

This can become more formal later with signatures, attestations, and supply-chain metadata.

