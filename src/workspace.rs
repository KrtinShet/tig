use crate::project::Project;
use crate::types::Workspace;
use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;
use std::path::PathBuf;

pub struct WorkspaceManager<'a> {
    project: &'a Project,
}

impl<'a> WorkspaceManager<'a> {
    pub fn new(project: &'a Project) -> Self {
        Self { project }
    }

    /// Create a new workspace
    pub fn create(&self, name: &str, actor: &str, goal: Option<String>) -> Result<Workspace> {
        let ws_dir = self.project.workspaces_dir().join(name);
        if ws_dir.exists() {
            anyhow::bail!("Workspace '{}' already exists", name);
        }

        fs::create_dir_all(&ws_dir)?;
        fs::create_dir_all(ws_dir.join("files"))?;

        let workspace = Workspace {
            name: name.to_string(),
            base_snapshot: None,
            actor: actor.to_string(),
            goal,
            created_at: Utc::now().to_rfc3339(),
            current_snapshot: None,
        };

        let meta_path = ws_dir.join("workspace.json");
        crate::util::atomic_write(&meta_path, serde_json::to_string_pretty(&workspace)?)?;

        Ok(workspace)
    }

    /// Create a workspace from an existing snapshot
    pub fn create_from_snapshot(
        &self,
        name: &str,
        actor: &str,
        goal: Option<String>,
        base_snapshot: String,
    ) -> Result<Workspace> {
        let ws = self.create(name, actor, goal)?;
        let mut ws = ws;
        ws.base_snapshot = Some(base_snapshot);

        let ws_dir = self.project.workspaces_dir().join(name);
        crate::util::atomic_write(&ws_dir.join("workspace.json"), serde_json::to_string_pretty(&ws)?)?;

        Ok(ws)
    }

    /// Load a workspace by name
    pub fn load(&self, name: &str) -> Result<Workspace> {
        let meta_path = self.project.workspaces_dir().join(name).join("workspace.json");
        let data = fs::read_to_string(&meta_path)
            .with_context(|| format!("Workspace '{}' not found", name))?;
        let ws: Workspace = serde_json::from_str(&data)?;
        Ok(ws)
    }

    /// List all workspaces
    pub fn list(&self) -> Result<Vec<Workspace>> {
        let mut workspaces = Vec::new();
        let ws_dir = self.project.workspaces_dir();
        if ws_dir.exists() {
            for entry in fs::read_dir(&ws_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    let meta_path = entry.path().join("workspace.json");
                    if meta_path.exists() {
                        let data = fs::read_to_string(&meta_path)?;
                        let ws: Workspace = serde_json::from_str(&data)?;
                        workspaces.push(ws);
                    }
                }
            }
        }
        Ok(workspaces)
    }

    /// Save workspace metadata
    pub fn save(&self, workspace: &Workspace) -> Result<()> {
        let ws_dir = self.project.workspaces_dir().join(&workspace.name);
        crate::util::atomic_write(
            &ws_dir.join("workspace.json"),
            serde_json::to_string_pretty(workspace)?,
        )?;
        Ok(())
    }

    /// Get the materialized files directory for a workspace
    pub fn files_dir(&self, name: &str) -> PathBuf {
        self.project.workspaces_dir().join(name).join("files")
    }

    /// Read a file from the workspace
    pub fn read_file(&self, name: &str, path: &str) -> Result<String> {
        let file_path = self.files_dir(name).join(path.trim_start_matches('/'));
        let content = fs::read_to_string(&file_path)
            .with_context(|| format!("File not found in workspace: {}", path))?;
        Ok(content)
    }

    /// Write a file to the workspace
    pub fn write_file(&self, name: &str, path: &str, content: &str) -> Result<()> {
        let files_dir = self.files_dir(name);
        let file_path = files_dir.join(path.trim_start_matches('/'));

        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        crate::util::atomic_write(&file_path, content)?;
        Ok(())
    }

    /// Delete a file from the workspace
    pub fn delete_file(&self, name: &str, path: &str) -> Result<()> {
        let file_path = self.files_dir(name).join(path.trim_start_matches('/'));
        fs::remove_file(&file_path)?;
        Ok(())
    }

    /// Apply a unified diff patch to a file
    pub fn apply_patch(&self, name: &str, path: &str, patch: &str) -> Result<()> {
        let current = self.read_file(name, path).unwrap_or_default();
        // Simple line-by-line patch application for basic unified diffs
        let patched = apply_unified_diff(&current, patch)?;
        self.write_file(name, path, &patched)?;
        Ok(())
    }
}

/// Apply a unified diff to a file's contents.
///
/// Backed by `diffy`, which honours `@@` hunk offsets and verifies that the
/// surrounding context actually matches. A patch that does not apply cleanly
/// is rejected with an error rather than silently corrupting the file.
fn apply_unified_diff(original: &str, patch: &str) -> Result<String> {
    let patch = diffy::Patch::from_str(patch)
        .map_err(|e| anyhow::anyhow!("Failed to parse patch: {}", e))?;
    diffy::apply(original, &patch)
        .map_err(|e| anyhow::anyhow!("Patch did not apply cleanly: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_patch_modifies_line_deep_in_file_using_hunk_offsets() {
        // A real unified diff only carries context near the change, plus an
        // `@@ -start,len @@` offset. The naive applier ignores the offset and
        // starts matching from line 0, corrupting everything before the hunk.
        let original = "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10\n";
        let modified = "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8-modified\nline9\nline10\n";

        let patch = diffy::create_patch(original, modified).to_string();
        let result = apply_unified_diff(original, &patch).unwrap();

        assert_eq!(result, modified, "patch was:\n{}", patch);
    }
}
