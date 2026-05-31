use crate::project::Project;
use crate::snapshot::SnapshotEngine;
use crate::types::{ReviewId, ReviewStatus, ReviewUnit, RunId, SnapshotId};
use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;

/// Review service for creating reviewable packages
pub struct ReviewService<'a> {
    project: &'a Project,
    snapshots: &'a SnapshotEngine<'a>,
}

impl<'a> ReviewService<'a> {
    pub fn new(project: &'a Project, snapshots: &'a SnapshotEngine<'a>) -> Self {
        Self { project, snapshots }
    }

    /// Create a review unit from a source snapshot compared to a target
    pub fn create(
        &self,
        source_snapshot: SnapshotId,
        target_snapshot: SnapshotId,
    ) -> Result<ReviewUnit> {
        let id = format!("review-{}", Utc::now().timestamp_millis());
        let changed_files = self.snapshots.diff(&source_snapshot, &target_snapshot)?;

        let review = ReviewUnit {
            id: id.clone(),
            source_snapshot,
            target_snapshot,
            changed_files,
            runs: Vec::new(),
            status: ReviewStatus::Draft,
            created_at: Utc::now().to_rfc3339(),
        };

        let review_path = self.project.reviews_dir().join(format!("{}.json", id));
        crate::util::atomic_write(&review_path, serde_json::to_string_pretty(&review)?)?;

        Ok(review)
    }

    /// Create a review unit from the latest passing snapshot
    pub fn create_from_latest_passing(
        &self,
        workspace_name: &str,
        target_snapshot: SnapshotId,
        runs: &[(SnapshotId, crate::types::Run)],
    ) -> Result<ReviewUnit> {
        let passing = self.snapshots.list_passing(workspace_name, runs)?;
        let (source_id, _) = passing.last()
            .with_context(|| format!("No passing snapshots found for workspace '{}'", workspace_name))?;

        let mut review = self.create(source_id.clone(), target_snapshot)?;

        // Attach runs for the source snapshot
        let snapshot_runs: Vec<RunId> = runs
            .iter()
            .filter(|(sid, _)| sid == source_id)
            .map(|(_, r)| r.id.clone())
            .collect();
        review.runs = snapshot_runs;

        self.save(&review)?;
        Ok(review)
    }

    /// Get a review unit by ID
    pub fn get(&self, id: &ReviewId) -> Result<ReviewUnit> {
        let review_path = self.project.reviews_dir().join(format!("{}.json", id));
        let data = fs::read_to_string(&review_path)
            .with_context(|| format!("Review unit '{}' not found", id))?;
        let review: ReviewUnit = serde_json::from_str(&data)?;
        Ok(review)
    }

    /// List all review units
    pub fn list(&self) -> Result<Vec<ReviewUnit>> {
        let mut reviews = Vec::new();
        let reviews_dir = self.project.reviews_dir();

        if !reviews_dir.exists() {
            return Ok(reviews);
        }

        for entry in fs::read_dir(&reviews_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let data = fs::read_to_string(entry.path())?;
                let review: ReviewUnit = serde_json::from_str(&data)?;
                reviews.push(review);
            }
        }

        Ok(reviews)
    }

    /// Update review unit status
    pub fn update_status(&self, id: &ReviewId, status: ReviewStatus) -> Result<()> {
        let mut review = self.get(id)?;
        review.status = status;
        self.save(&review)?;
        Ok(())
    }

    /// Save a review unit
    pub fn save(&self, review: &ReviewUnit) -> Result<()> {
        let review_path = self.project.reviews_dir().join(format!("{}.json", &review.id));
        crate::util::atomic_write(&review_path, serde_json::to_string_pretty(review)?)?;
        Ok(())
    }

    /// Display a review unit as formatted text
    pub fn format_review(&self, review: &ReviewUnit) -> Result<String> {
        let mut output = vec![
            format!("Review Unit: {}", review.id),
            format!("Source Snapshot: {}", review.source_snapshot),
            format!("Target Snapshot: {}", review.target_snapshot),
            format!("Status: {:?}", review.status),
            format!("Created: {}", review.created_at),
            String::new(),
        ];

        // Show run evidence
        if !review.runs.is_empty() {
            output.push("Run Evidence:".to_string());
            let run_store = crate::run::RunStore::new(self.project);
            for run_id in &review.runs {
                if let Ok(run) = run_store.get(run_id) {
                    let status = if run.status == crate::types::RunStatus::Passed {
                        "✓ PASSED"
                    } else {
                        "✗ FAILED"
                    };
                    output.push(format!("  {} - {} (exit: {})", run_id, status, run.exit_code));
                    output.push(format!("    Command: {}", run.command));
                    if !run.stdout.is_empty() {
                        for line in run.stdout.lines().take(5) {
                            output.push(format!("    stdout: {}", line));
                        }
                    }
                    if !run.stderr.is_empty() {
                        for line in run.stderr.lines().take(3) {
                            output.push(format!("    stderr: {}", line));
                        }
                    }
                }
            }
            output.push(String::new());
        }

        output.push("Changed Files:".to_string());
        for diff in &review.changed_files {
            output.push(format!("\n  {}", diff.path));
            output.push(format!("  Old hash: {:?}", diff.old_hash));
            output.push(format!("  New hash: {:?}", diff.new_hash));
            output.push("  ---".to_string());
            for line in diff.diff.lines() {
                output.push(format!("    {}", line));
            }
        }

        Ok(output.join("\n"))
    }
}
