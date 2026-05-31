use crate::project::Project;
use crate::types::{Object, ReviewUnit};
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Git import/export bridge for compatibility
pub struct GitBridge<'a> {
    project: &'a Project,
}

impl<'a> GitBridge<'a> {
    pub fn new(project: &'a Project) -> Self {
        Self { project }
    }

    /// Export a review unit as a Git commit
    pub fn export_commit(&self, review: &ReviewUnit) -> Result<String> {
        let export_dir = self.project.tig_dir().join("git-export");
        self.init_git_repo(&export_dir)?;

        // Materialize the source snapshot
        let store = crate::object_store::ObjectStore::open(self.project.objects_dir())?;
        let snapshot = store.get(&review.source_snapshot)?;
        let tree_hash = match &snapshot {
            Object::Snapshot { tree, .. } => tree.clone(),
            _ => anyhow::bail!("Not a snapshot object"),
        };

        // Get tree entries
        let tree = store.get(&tree_hash)?;
        let entries = match &tree {
            Object::Tree { entries } => entries.clone(),
            _ => anyhow::bail!("Not a tree object"),
        };

        // Write files to export directory
        for (path, hash) in entries {
            let blob = store.get(&hash)?;
            let content = match &blob {
                Object::Blob { content } => content.clone(),
                _ => continue,
            };
            let file_path = export_dir.join(&path);
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&file_path, content)?;
        }

        // Stage files
        Command::new("git")
            .args(["add", "."])
            .current_dir(&export_dir)
            .output()
            .with_context(|| "Failed to stage files in git export")?;

        // Extract actor and goal from snapshot for commit metadata
        let (actor, goal) = match &snapshot {
            Object::Snapshot { actor, goal, .. } => (actor.clone(), goal.clone()),
            _ => ("unknown".to_string(), None),
        };

        // Set Git author from snapshot actor
        Command::new("git")
            .args(["config", "user.name", &actor])
            .current_dir(&export_dir)
            .output()?;

        // Create commit with full metadata
        let goal_line = goal.as_deref().unwrap_or("No goal specified");
        let message = format!(
            "Tig Review: {}\n\nActor: {}\nGoal: {}\nSource: {}\nTarget: {}\n",
            review.id, actor, goal_line, review.source_snapshot, review.target_snapshot
        );

        let output = Command::new("git")
            .args([
                "commit",
                "-m",
                &message,
                "--allow-empty",
            ])
            .current_dir(&export_dir)
            .output()
            .with_context(|| "Failed to create git commit")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Git commit failed: {}", stderr);
        }

        // Get commit hash
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&export_dir)
            .output()
            .with_context(|| "Failed to get commit hash")?;

        let commit_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(commit_hash)
    }

    /// Export a review unit as a patch file
    pub fn export_patch(&self, review: &ReviewUnit) -> Result<PathBuf> {
        let export_dir = self.project.tig_dir().join("git-export");
        self.init_git_repo(&export_dir)?;

        // Materialize both snapshots
        let store = crate::object_store::ObjectStore::open(self.project.objects_dir())?;
        let source_snapshot = store.get(&review.source_snapshot)?;
        let target_snapshot = store.get(&review.target_snapshot)?;

        let source_tree = match &source_snapshot {
            Object::Snapshot { tree, .. } => tree.clone(),
            _ => anyhow::bail!("Not a snapshot"),
        };
        let target_tree = match &target_snapshot {
            Object::Snapshot { tree, .. } => tree.clone(),
            _ => anyhow::bail!("Not a snapshot"),
        };

        // Materialize source to a temp dir
        let source_dir = export_dir.join("source");
        store.materialize_tree(&source_tree, &source_dir)?;

        // Materialize target to a temp dir
        let target_dir = export_dir.join("target");
        store.materialize_tree(&target_tree, &target_dir)?;

        // Generate patch using diff
        let patch_content = self.generate_patch(&source_dir, &target_dir)?;

        let patch_path = self.project.tig_dir().join(format!("{}.patch", review.id));
        fs::write(&patch_path, patch_content)?;

        Ok(patch_path)
    }

    /// Import an existing Git repository as a Tig project base
    pub fn import_git_repo(&self, git_dir: &PathBuf) -> Result<()> {
        if !git_dir.join(".git").exists() {
            anyhow::bail!("Not a Git repository: {}", git_dir.display());
        }

        // Get the latest commit tree
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(git_dir)
            .output()
            .with_context(|| "Failed to get HEAD commit")?;

        let head = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // Export the tree to a temp directory
        let temp_dir = tempfile::tempdir()?;
        Command::new("git")
            .args(["checkout", "HEAD", "--", "."])
            .current_dir(git_dir)
            .output()?;

        // Copy files to temp
        self.copy_dir_all(git_dir, temp_dir.path())?;

        // Create initial snapshot from the tree
        let store = crate::object_store::ObjectStore::open(self.project.objects_dir())?;
        let mut entries = std::collections::HashMap::new();

        for entry in walkdir::WalkDir::new(temp_dir.path())
            .min_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let relative = entry.path().strip_prefix(temp_dir.path())?;
                let content = fs::read(entry.path())?;
                let blob = Object::Blob { content };
                let hash = store.put(&blob)?;
                entries.insert(relative.to_string_lossy().to_string(), hash);
            }
        }

        let tree = Object::Tree { entries };
        let tree_hash = store.put(&tree)?;

        let snapshot = Object::Snapshot {
            parent: None,
            tree: tree_hash,
            workspace: "main".to_string(),
            actor: "git-import".to_string(),
            goal: Some(format!("Import from Git commit {}", head)),
            timestamp: chrono::Utc::now().to_rfc3339(),
            changes: vec![crate::types::Change::FileAdded { path: "*".to_string() }],
        };

        let snapshot_hash = store.put(&snapshot)?;

        // Save as main snapshot ref
        let refs_dir = self.project.refs_dir().join("snapshots");
        fs::create_dir_all(&refs_dir)?;
        fs::write(
            refs_dir.join(format!("{}.json", snapshot_hash)),
            serde_json::to_string_pretty(&snapshot)?,
        )?;

        Ok(())
    }

    fn init_git_repo(&self, dir: &PathBuf) -> Result<()> {
        if !dir.join(".git").exists() {
            fs::create_dir_all(dir)?;
            Command::new("git")
                .args(["init", "--quiet"])
                .current_dir(dir)
                .output()
                .with_context(|| "Failed to initialize git repository")?;

            // Set git user for commits
            Command::new("git")
                .args(["config", "user.email", "tig@local"])
                .current_dir(dir)
                .output()?;
            Command::new("git")
                .args(["config", "user.name", "Tig Export"])
                .current_dir(dir)
                .output()?;
        }
        Ok(())
    }

    fn generate_patch(&self, source_dir: &PathBuf, target_dir: &PathBuf) -> Result<String> {
        let output = Command::new("diff")
            .args(["-ruN", &source_dir.to_string_lossy(), &target_dir.to_string_lossy()])
            .output()
            .with_context(|| "Failed to generate diff")?;

        let patch = String::from_utf8_lossy(&output.stdout);
        Ok(patch.to_string())
    }

    fn copy_dir_all(&self, src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let name = entry.file_name();
            if name == ".git" || name == ".tig" {
                continue;
            }
            let src_path = entry.path();
            let dst_path = dst.join(&name);
            if entry.file_type()?.is_dir() {
                self.copy_dir_all(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }
        Ok(())
    }
}
