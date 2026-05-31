# Tig

**A work-first version control substrate for humans and AI coding agents.**

Tig is an experimental, local-first **Git alternative** built for a world where most code is written by **AI coding agents** working in parallel. Instead of branches, worktrees, and pull requests, Tig is organized around **workspaces**, automatic **snapshots**, recorded **runs**, and clean **review units** — with Git **export/import** for compatibility.

![status: experimental](https://img.shields.io/badge/status-experimental-orange)
![Rust](https://img.shields.io/badge/built%20with-Rust-000000?logo=rust)
![License: MIT](https://img.shields.io/badge/license-MIT-blue)

> **Keywords:** version control, Git alternative, distributed version control (DVCS), source control, content-addressed storage, AI coding agents, agentic development, autonomous coding, snapshots, code review, developer tools, Rust CLI.

---

## Why Tig?

Git is a brilliant content-addressed object database, but its *everyday workflow* — branches, worktrees, staging, commits, force-pushes, PRs — was designed for humans hand-authoring history. That model strains when **fleets of AI agents** try the same task ten different ways at once.

Tig starts from a different thesis:

- **Work is captured continuously.** Every edit produces a snapshot automatically — no `git add`, no "oops, lost my work."
- **Workspaces are cheap, editable views** — not fragile branch/worktree rituals. Spin up ten parallel attempts at one bug; keep the one that passes.
- **Runs are first-class.** A test/command result is attached to the *exact* snapshot it ran against, so review evidence is reproducible.
- **Review units package clean results** *after* messy work is done. The working process can be chaotic; the reviewed output is intentional.
- **Publication is intentional.** Editing is not publishing — visibility is meant to be policy-driven, not "everything is public forever."
- **Git is an interface, not the source of truth.** Export to a Git commit or patch whenever you need compatibility; the internal model isn't defined by Git.

If you build with Claude Code, Codex, Cursor, or your own agents, Tig is an experiment in giving them a substrate where *messy, parallel, machine-speed work* stays reviewable, comparable, and auditable.

---

## What works today

This is an early but **functional** prototype (not a production VCS yet — see [Status](#status)). The local work → review → Git loop runs end to end:

- ✅ **Content-addressed object store** — SHA-256, raw-byte blobs (binary-safe, no bloat)
- ✅ **Workspaces** — cheap editable views over project state
- ✅ **Automatic snapshots** on every change, with an incremental stat-cache (no full re-hash per edit)
- ✅ **Real unified diff/patch** (binary files detected, never mangled)
- ✅ **Runs** — execute a command and attach its stdout/stderr/exit-code to a snapshot
- ✅ **Review units** — package a chosen snapshot with its run evidence
- ✅ **Git export/import** — turn a review unit into a Git commit or patch
- ✅ **Crash-safe atomic writes** + a process-level **write lock** for safe concurrent use
- ✅ **CLI** *and* a **programmatic Rust API** for agents

---

## Install

Tig is a single self-contained Rust binary.

### Option 1 — install straight from GitHub (recommended)

```bash
cargo install --git https://github.com/KrtinShet/tig
```

This puts a `tig` binary on your `PATH` (via `~/.cargo/bin`).

### Option 2 — build from source

```bash
git clone https://github.com/KrtinShet/tig
cd tig
cargo build --release
# binary at ./target/release/tig
```

> Requires a [Rust toolchain](https://rustup.rs) (stable). `git` must be installed for the Git export/import bridge.

---

## Quickstart (60 seconds)

```bash
# 1. Initialize a Tig project in the current directory
tig init --name my-project

# 2. Create a workspace (an actor can be you or an agent like "claude"/"codex")
tig workspace create fix-auth --actor claude --goal "Fix the auth timeout"

# 3. Write files — every write auto-creates a snapshot
tig write /src/auth.js --content 'exports.timeout = 30000;'
#   ...or pull content from a file:
tig write /src/auth.js --from ./patch-content.js

# 4. Run a check — its result is attached to the exact snapshot
tig run execute "node -e \"require('./src/auth.js')\""

# 5. Inspect history and runs
tig snapshot list
tig run list

# 6. Package a review unit (from the latest passing snapshot → a target)
tig review create --from latest-passing --target <base-snapshot-id>
tig review list
tig review show <review-id>

# 7. Export the reviewed result to Git for compatibility
tig git export --review <review-id>            # creates a Git commit
tig git export --review <review-id> --patch    # or a .patch file
```

Want to see the whole loop run, including two parallel attempts at the same bug where one passes and one fails? Run the end-to-end demo:

```bash
bash tests/e2e_demo.sh
```

### Full command reference

| Command | What it does |
|---|---|
| `tig init [--name <n>]` | Initialize a project (creates `.tig/`) |
| `tig workspace create <name> [--actor <a>] [--goal <g>]` | Create an editable workspace |
| `tig workspace list` | List workspaces |
| `tig workspace switch <name>` | Set the active workspace |
| `tig read <path>` | Read a file from the active workspace |
| `tig write <path> --content <s> \| --from <file>` | Write a file (auto-snapshots) |
| `tig run execute "<command>"` | Run a command, record evidence against a snapshot |
| `tig run list` | List recorded runs |
| `tig snapshot list [--passing]` | List snapshots (optionally only passing ones) |
| `tig review create --from <snap\|latest\|latest-passing> --target <snap>` | Build a review unit |
| `tig review list` / `tig review show <id>` | Inspect review units |
| `tig git export --review <id> [--patch]` | Export a review unit to a Git commit or patch |
| `tig git import <path-to-git-repo>` | Seed a Tig project from an existing Git repo |

---

## Use Tig in your own project

### A) From any language — drive the CLI

Because every Tig operation is a CLI command with deterministic output, agents and scripts in **any language** can use it by shelling out — no Git plumbing, no branch management.

```python
import subprocess

def tig(*args):
    return subprocess.run(["tig", *args], capture_output=True, text=True, check=True).stdout

tig("init", "--name", "runtime-platform")
tig("workspace", "create", "fix-auth", "--actor", "my-agent", "--goal", "Fix auth timeout")
tig("write", "/src/auth.js", "--content", "exports.timeout = 30000;")
tig("run", "execute", "npm test auth")
tig("review", "create", "--from", "latest-passing", "--target", "main")
```

### B) From Rust — use the programmatic API

Add Tig as a dependency:

```toml
[dependencies]
tig = { git = "https://github.com/KrtinShet/tig" }
```

Then drive it in-process — ideal for embedding in a Rust-based agent or tool:

```rust
use std::path::Path;
use tig::api::Tig;

fn main() -> anyhow::Result<()> {
    // Initialize (or `Tig::open`) a project
    let tig = Tig::init(Path::new("./my-project"), Some("my-project".into()))?;

    // Create a workspace for an actor + goal
    tig.create_workspace("fix-auth", "claude", Some("Fix auth timeout".into()))?;

    // Edits auto-create snapshots; returns the new snapshot id
    let snap = tig.write_file("fix-auth", "/src/auth.js", "exports.timeout = 30000;")?;

    // Or apply a unified diff/patch
    // tig.apply_patch("fix-auth", "/src/auth.js", patch_str)?;

    // Run a check; evidence is bound to the exact snapshot
    let run = tig.run_check("fix-auth", "npm test auth", "claude")?;
    println!("run {} -> {:?}", run.id, run.status);

    // Package a review unit and export it to Git
    let review = tig.create_review_unit("fix-auth", "latest-passing", &snap)?;
    let commit = tig.export_to_git(&review.id)?;
    println!("exported Git commit {commit}");
    Ok(())
}
```

---

## Core concepts

| Primitive | Meaning |
|---|---|
| **Project** | Top-level source, review, and publication boundary (a `.tig/` directory) |
| **Object Store** | Immutable, content-addressed file contents and source-state objects |
| **Workspace** | An editable view over project state (replaces branches/worktrees) |
| **Change** | A tracked edit or operation inside a workspace |
| **Snapshot** | A complete project state at a point in time (created automatically) |
| **Attempt** | A purposeful line of work by a human or agent |
| **Run** | A command/check executed against an exact snapshot, with captured evidence |
| **Review Unit** | A clean, reviewable package derived from one or more snapshots |
| **Projection** | A policy-controlled visible view of a project *(planned)* |
| **Publication** | An intentional update to a projection or external system *(planned)* |

---

## How Tig compares

| | **Git** | **jj (Jujutsu)** | **Tig** |
|---|---|---|---|
| Primary unit | Commit on a branch | Mutable change | Snapshot in a workspace |
| Capturing work | Manual (`add`/`commit`) | Automatic working-copy | **Automatic snapshots** |
| Parallel attempts | Branches/worktrees | Anonymous changes | **Cheap workspaces** |
| Test/check evidence | External (CI) | External | **First-class runs bound to snapshots** |
| Review artifact | Pull request | Change/PR | **Review unit (post-hoc, clean)** |
| Visibility model | Repo-level | Repo-level | **Policy-driven projections** *(planned)* |
| Git compatibility | — | Native | **Export/import bridge** |

Tig is **not** trying to replace Git's object model — it reuses the same content-addressed idea and exports to Git on demand. It's reworking the *workflow layer* above it for agent-heavy development.

---

## Project layout

A Tig project lives entirely under `.tig/`:

```
.tig/
  objects/     # content-addressed blobs, trees, snapshots
  refs/        # snapshot references
  workspaces/  # per-workspace files + metadata + stat cache
  runs/        # recorded command runs (evidence)
  reviews/     # review units
  metadata.json
  lock         # advisory write lock
```

---

## Status

Tig is an **experimental research prototype**, not a production version-control system. The local work-to-review loop is implemented and tested (19 unit + 2 integration tests + an end-to-end demo, all green), with crash-safe writes and concurrency-safe locking.

**Implemented:** object store, workspaces, automatic + incremental snapshots, diff/patch, runs, review units, Git export/import, atomic writes, write lock, CLI + Rust API.

**Not yet (and intentionally out of scope for the first prototype):**

- Policy-driven **projections** and intentional **publication** (the visibility model)
- Hosted, multi-tenant collaboration / a GitHub-style server
- `fsync` power-loss durability and full Git history fidelity on import
- Merge UI, permissions dashboards, package registry

Treat it as a place to **dogfood the workflow**, not yet as the home for irreplaceable work. Feedback and ideas are very welcome.

---

## Documentation

- [Problem](docs/problem.md) — what Git/GitHub don't model well for agent-heavy work
- [Solution](docs/solution.md) — the work-first model Tig proposes
- [Ideology](docs/ideology.md) — the beliefs constraining product/engineering choices
- [Primitives](docs/primitives.md) — the core nouns in the system
- [Workflows](docs/workflows.md) — concrete human, agent, security, and projection examples
- [MVP](docs/mvp.md) — first prototype scope and non-goals
- [Comparisons](docs/comparisons.md) — Tig vs Git, GitHub, jj, monorepo systems
- [Security Model](docs/security-model.md) — actors, policies, projections, publication checks
- [Architecture](docs/architecture.md) — early component boundaries
- [Roadmap](docs/roadmap.md) — staged path from prototype to hosted collaboration

---

## Contributing

Issues, discussions, and PRs are welcome — especially around the workflow model, the agent API surface, and real-world agent integrations. Build and test with:

```bash
cargo build
cargo test
bash tests/e2e_demo.sh
```

## License

Licensed under the [MIT License](LICENSE).
