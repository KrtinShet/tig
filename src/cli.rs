use crate::api::Tig;
use crate::git_bridge::GitBridge;
use crate::object_store::ObjectStore;
use crate::project::Project;
use crate::review::ReviewService;
use crate::run::RunStore;
use crate::snapshot::SnapshotEngine;
use crate::types::{Object, ReviewStatus};
use crate::workspace::WorkspaceManager;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "tig")]
#[command(about = "Tig - A work-first version control substrate")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Tig project
    Init {
        /// Project name
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Workspace management
    Workspace {
        #[command(subcommand)]
        command: WorkspaceCommands,
    },
    /// Read a file from the active workspace
    Read {
        /// File path
        path: String,
    },
    /// Write a file to the active workspace
    Write {
        /// File path
        path: String,
        /// Content to write
        #[arg(long)]
        content: Option<String>,
        /// Read content from a file
        #[arg(long)]
        from: Option<PathBuf>,
    },
    /// Run a command and record evidence
    Run {
        #[command(subcommand)]
        command: RunCommands,
    },
    /// Snapshot management
    Snapshot {
        #[command(subcommand)]
        command: SnapshotCommands,
    },
    /// Review unit management
    Review {
        #[command(subcommand)]
        command: ReviewCommands,
    },
    /// Git import/export
    Git {
        #[command(subcommand)]
        command: GitCommands,
    },
}

#[derive(Subcommand)]
enum WorkspaceCommands {
    /// Create a new workspace
    Create {
        /// Workspace name
        name: String,
        /// Actor creating the workspace
        #[arg(short, long, default_value = "user")]
        actor: String,
        /// Goal/description
        #[arg(short, long)]
        goal: Option<String>,
    },
    /// List all workspaces
    List,
    /// Switch to a workspace
    Switch {
        /// Workspace name
        name: String,
    },
}

#[derive(Subcommand)]
enum RunCommands {
    /// Execute a command
    Execute {
        /// Command to execute
        command: String,
    },
    /// List all runs
    List,
}

#[derive(Subcommand)]
enum SnapshotCommands {
    /// List snapshots
    List {
        /// Filter to passing snapshots only
        #[arg(long)]
        passing: bool,
    },
}

#[derive(Subcommand)]
enum ReviewCommands {
    /// Create a review unit
    Create {
        /// Source snapshot or "latest-passing"
        #[arg(long)]
        from: String,
        /// Target snapshot
        #[arg(long)]
        target: String,
    },
    /// List review units
    List,
    /// Show a review unit
    Show {
        /// Review unit ID
        id: String,
    },
}

#[derive(Subcommand)]
enum GitCommands {
    /// Export a review unit as a Git commit
    Export {
        /// Review unit ID
        #[arg(long)]
        review: String,
        /// Export as patch file
        #[arg(long)]
        patch: bool,
    },
    /// Import an existing Git repository
    Import {
        /// Path to Git repository
        path: PathBuf,
    },
}

fn find_project_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir()?;
    loop {
        if Project::exists(&current) {
            return Ok(current);
        }
        if !current.pop() {
            anyhow::bail!("Not inside a Tig project. Run 'tig init' first.")
        }
    }
}

fn get_active_workspace(project: &Project) -> Result<String> {
    let active_path = project.tig_dir().join("active_workspace");
    if active_path.exists() {
        let name = fs::read_to_string(&active_path)?;
        Ok(name.trim().to_string())
    } else {
        // Default to first workspace
        let ws_mgr = WorkspaceManager::new(project);
        let workspaces = ws_mgr.list()?;
        workspaces
            .first()
            .map(|w| w.name.clone())
            .ok_or_else(|| anyhow::anyhow!("No workspaces found. Create one with 'tig workspace create <name>'"))
    }
}

fn set_active_workspace(project: &Project, name: &str) -> Result<()> {
    crate::util::atomic_write(&project.tig_dir().join("active_workspace"), name)?;
    Ok(())
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name } => {
            let cwd = std::env::current_dir()?;
            if Project::exists(&cwd) {
                println!("Tig project already initialized here.");
                return Ok(());
            }
            let _project = Project::init(&cwd, name.clone())?;
            println!(
                "Initialized Tig project '{}' at {}",
                name.as_deref().unwrap_or("untitled"),
                cwd.display()
            );
        }

        Commands::Workspace { command } => {
            let root = find_project_root()?;
            let project = Project::open(&root)?;

            match command {
                WorkspaceCommands::Create { name, actor, goal } => {
                    let ws_mgr = WorkspaceManager::new(&project);
                    let ws = ws_mgr.create(&name, &actor, goal.clone())?;
                    println!("Created workspace '{}' (actor: {}, goal: {:?})", ws.name, ws.actor, ws.goal);
                    set_active_workspace(&project, &name)?;
                }
                WorkspaceCommands::List => {
                    let ws_mgr = WorkspaceManager::new(&project);
                    let workspaces = ws_mgr.list()?;
                    if workspaces.is_empty() {
                        println!("No workspaces found.");
                    } else {
                        println!("{:<20} {:<20} {:<30}", "NAME", "ACTOR", "GOAL");
                        for ws in workspaces {
                            let goal = ws.goal.as_deref().unwrap_or("-");
                            let snapshot = ws.current_snapshot.as_deref().unwrap_or("none");
                            println!("{:<20} {:<20} {:<30} (snapshot: {})", ws.name, ws.actor, goal, snapshot);
                        }
                    }
                }
                WorkspaceCommands::Switch { name } => {
                    let ws_mgr = WorkspaceManager::new(&project);
                    ws_mgr.load(&name)?; // Verify it exists
                    set_active_workspace(&project, &name)?;
                    println!("Switched to workspace '{}'", name);
                }
            }
        }

        Commands::Read { path } => {
            let root = find_project_root()?;
            let project = Project::open(&root)?;
            let ws_name = get_active_workspace(&project)?;
            let ws_mgr = WorkspaceManager::new(&project);
            let content = ws_mgr.read_file(&ws_name, &path)?;
            println!("{}", content);
        }

        Commands::Write { path, content, from } => {
            let root = find_project_root()?;
            let project = Project::open(&root)?;
            let ws_name = get_active_workspace(&project)?;

            let content = match (content, from) {
                (Some(c), None) => c,
                (None, Some(p)) => fs::read_to_string(&p)
                    .with_context(|| format!("Failed to read from file: {}", p.display()))?,
                (Some(c), Some(_)) => c,
                (None, None) => anyhow::bail!("Provide either --content or --from"),
            };

            let tig = Tig::open(&root)?;
            let snapshot_id = tig.write_file(&ws_name, &path, &content)?;
            println!("Wrote {} and created snapshot {}", path, snapshot_id);
        }

        Commands::Run { command } => {
            let root = find_project_root()?;
            let project = Project::open(&root)?;

            match command {
                RunCommands::Execute { command } => {
                    let ws_name = get_active_workspace(&project)?;
                    let tig = Tig::open(&root)?;
                    let run = tig.run_check(&ws_name, &command, "user")?;
                    println!(
                        "Run {}: {} (exit code: {})",
                        run.id,
                        if run.status == crate::types::RunStatus::Passed {
                            "PASSED"
                        } else {
                            "FAILED"
                        },
                        run.exit_code
                    );
                    if !run.stdout.is_empty() {
                        println!("stdout:\n{}", run.stdout);
                    }
                    if !run.stderr.is_empty() {
                        eprintln!("stderr:\n{}", run.stderr);
                    }
                }
                RunCommands::List => {
                    let run_store = RunStore::new(&project);
                    let runs = run_store.list()?;
                    if runs.is_empty() {
                        println!("No runs found.");
                    } else {
                        println!("{:<32} {:<12} {:<32} {}", "ID", "STATUS", "SNAPSHOT", "COMMAND");
                        for run in runs {
                            let status = if run.status == crate::types::RunStatus::Passed {
                                "PASSED"
                            } else {
                                "FAILED"
                            };
                            let cmd = if run.command.len() > 30 {
                                format!("{}...", &run.command[..27])
                            } else {
                                run.command.clone()
                            };
                            println!("{:<32} {:<12} {:<32} {}", run.id, status, run.snapshot_id, cmd);
                        }
                    }
                }
            }
        }

        Commands::Snapshot { command } => {
            let root = find_project_root()?;
            let project = Project::open(&root)?;
            let ws_name = get_active_workspace(&project)?;
            let store = ObjectStore::open(project.objects_dir())?;
            let snapshot_engine = SnapshotEngine::new(&project, &store);

            match command {
                SnapshotCommands::List { passing } => {
                    if passing {
                        let run_store = RunStore::new(&project);
                        let runs = run_store.list_all_with_snapshot()?;
                        let snapshots = snapshot_engine.list_passing(&ws_name, &runs)?;
                        if snapshots.is_empty() {
                            println!("No passing snapshots found.");
                        } else {
                            println!("{:<64} {:<30} {}", "ID", "TIMESTAMP", "CHANGES");
                            for (id, snap) in snapshots {
                                if let Object::Snapshot { timestamp, changes, .. } = snap {
                                    let change_count = changes.len();
                                    println!("{:<64} {:<30} {} changes", id, timestamp, change_count);
                                }
                            }
                        }
                    } else {
                        let snapshots = snapshot_engine.list_for_workspace(&ws_name)?;
                        if snapshots.is_empty() {
                            println!("No snapshots found.");
                        } else {
                            println!("{:<64} {:<30} {}", "ID", "TIMESTAMP", "CHANGES");
                            for (id, snap) in snapshots {
                                if let Object::Snapshot { timestamp, changes, .. } = snap {
                                    let change_count = changes.len();
                                    println!("{:<64} {:<30} {} changes", id, timestamp, change_count);
                                }
                            }
                        }
                    }
                }
            }
        }

        Commands::Review { command } => {
            let root = find_project_root()?;
            let project = Project::open(&root)?;
            let store = ObjectStore::open(project.objects_dir())?;
            let snapshot_engine = SnapshotEngine::new(&project, &store);
            let review_service = ReviewService::new(&project, &snapshot_engine);

            match command {
                ReviewCommands::Create { from, target } => {
                    let ws_name = get_active_workspace(&project)?;
                    let tig = Tig::open(&root)?;
                    let review = tig.create_review_unit(&ws_name, &from, &target)?;
                    println!("Created review unit {}", review.id);
                    println!("Source: {}", review.source_snapshot);
                    println!("Target: {}", review.target_snapshot);
                    println!("Changed files: {}", review.changed_files.len());
                }
                ReviewCommands::List => {
                    let reviews = review_service.list()?;
                    if reviews.is_empty() {
                        println!("No review units found.");
                    } else {
                        println!("{:<32} {:<12} {:<32} {}", "ID", "STATUS", "SOURCE", "CHANGED");
                        for review in reviews {
                            let status = match review.status {
                                ReviewStatus::Draft => "draft",
                                ReviewStatus::Ready => "ready",
                            };
                            println!(
                                "{:<32} {:<12} {:<32} {} files",
                                review.id, status, review.source_snapshot, review.changed_files.len()
                            );
                        }
                    }
                }
                ReviewCommands::Show { id } => {
                    let review = review_service.get(&id)?;
                    let formatted = review_service.format_review(&review)?;
                    println!("{}", formatted);
                }
            }
        }

        Commands::Git { command } => {
            let root = find_project_root()?;
            let project = Project::open(&root)?;

            match command {
                GitCommands::Export { review, patch } => {
                    let store = ObjectStore::open(project.objects_dir())?;
                    let snapshot_engine = SnapshotEngine::new(&project, &store);
                    let review_service = ReviewService::new(&project, &snapshot_engine);
                    let review = review_service.get(&review)?;
                    let git_bridge = GitBridge::new(&project);

                    if patch {
                        let patch_path = git_bridge.export_patch(&review)?;
                        println!("Exported patch to {}", patch_path.display());
                    } else {
                        let commit_hash = git_bridge.export_commit(&review)?;
                        println!("Exported review as Git commit {}", commit_hash);
                    }
                }
                GitCommands::Import { path } => {
                    let git_bridge = GitBridge::new(&project);
                    git_bridge.import_git_repo(&path)?;
                    println!("Imported Git repository from {}", path.display());
                }
            }
        }
    }

    Ok(())
}
