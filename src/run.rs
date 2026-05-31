use crate::project::Project;
use crate::types::{Run, RunId, RunStatus, SnapshotId};
use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;
use std::process::Command;
use std::path::PathBuf;

/// Run store for executing commands and recording evidence
pub struct RunStore<'a> {
    project: &'a Project,
}

impl<'a> RunStore<'a> {
    pub fn new(project: &'a Project) -> Self {
        Self { project }
    }

    /// Execute a command and record the run
    pub fn execute(
        &self,
        snapshot_id: SnapshotId,
        command: &str,
        actor: &str,
        working_dir: &PathBuf,
    ) -> Result<Run> {
        let id = format!("run-{}", Utc::now().timestamp_millis());
        let start_time = Utc::now().to_rfc3339();

        // Execute the command in the working directory
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(working_dir)
            .output()
            .with_context(|| format!("Failed to execute command: {}", command))?;

        let end_time = Utc::now().to_rfc3339();
        let exit_code = output.status.code().unwrap_or(-1);
        let status = if output.status.success() {
            RunStatus::Passed
        } else {
            RunStatus::Failed
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let run = Run {
            id: id.clone(),
            snapshot_id,
            command: command.to_string(),
            actor: actor.to_string(),
            start_time,
            end_time,
            status,
            exit_code,
            stdout,
            stderr,
        };

        // Save the run
        let run_path = self.project.runs_dir().join(format!("{}.json", id));
        crate::util::atomic_write(&run_path, serde_json::to_string_pretty(&run)?)?;

        Ok(run)
    }

    /// Get a run by ID
    pub fn get(&self, id: &RunId) -> Result<Run> {
        let run_path = self.project.runs_dir().join(format!("{}.json", id));
        let data = fs::read_to_string(&run_path)
            .with_context(|| format!("Run '{}' not found", id))?;
        let run: Run = serde_json::from_str(&data)?;
        Ok(run)
    }

    /// List all runs
    pub fn list(&self) -> Result<Vec<Run>> {
        let mut runs = Vec::new();
        let runs_dir = self.project.runs_dir();

        if !runs_dir.exists() {
            return Ok(runs);
        }

        for entry in fs::read_dir(&runs_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let data = fs::read_to_string(entry.path())?;
                let run: Run = serde_json::from_str(&data)?;
                runs.push(run);
            }
        }

        Ok(runs)
    }

    /// List runs for a specific snapshot
    pub fn list_for_snapshot(&self, snapshot_id: &SnapshotId) -> Result<Vec<Run>> {
        let all = self.list()?;
        Ok(all.into_iter()
            .filter(|r| r.snapshot_id == *snapshot_id)
            .collect())
    }

    /// List all runs as (snapshot_id, run) pairs for easy filtering
    pub fn list_all_with_snapshot(&self) -> Result<Vec<(SnapshotId, Run)>> {
        let all = self.list()?;
        Ok(all.into_iter().map(|r| (r.snapshot_id.clone(), r)).collect())
    }
}
