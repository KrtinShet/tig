use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for objects in the content-addressed store
pub type ObjectHash = String;

/// Unique identifier for snapshots
pub type SnapshotId = String;

/// Unique identifier for runs
pub type RunId = String;

/// Unique identifier for review units
pub type ReviewId = String;

/// Object types stored in the object store
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Object {
    #[serde(rename = "blob")]
    Blob { content: Vec<u8> },
    #[serde(rename = "tree")]
    Tree { entries: HashMap<String, ObjectHash> },
    #[serde(rename = "snapshot")]
    Snapshot {
        parent: Option<SnapshotId>,
        tree: ObjectHash,
        workspace: String,
        actor: String,
        goal: Option<String>,
        timestamp: String,
        changes: Vec<Change>,
    },
}

/// A change operation in a workspace
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum Change {
    #[serde(rename = "file_added")]
    FileAdded { path: String },
    #[serde(rename = "file_modified")]
    FileModified { path: String },
    #[serde(rename = "file_deleted")]
    FileDeleted { path: String },
    #[serde(rename = "patch_applied")]
    PatchApplied { path: String, description: String },
}

/// Project metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub name: String,
    pub created_at: String,
    pub default_branch: String,
}

/// Workspace metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub name: String,
    pub base_snapshot: Option<SnapshotId>,
    pub actor: String,
    pub goal: Option<String>,
    pub created_at: String,
    pub current_snapshot: Option<SnapshotId>,
}

/// Run/check execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub id: RunId,
    pub snapshot_id: SnapshotId,
    pub command: String,
    pub actor: String,
    pub start_time: String,
    pub end_time: String,
    pub status: RunStatus,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Run execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RunStatus {
    #[serde(rename = "passed")]
    Passed,
    #[serde(rename = "failed")]
    Failed,
}

/// Review unit for human review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewUnit {
    pub id: ReviewId,
    pub source_snapshot: SnapshotId,
    pub target_snapshot: SnapshotId,
    pub changed_files: Vec<FileDiff>,
    pub runs: Vec<RunId>,
    pub status: ReviewStatus,
    pub created_at: String,
}

/// Review unit status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReviewStatus {
    #[serde(rename = "draft")]
    Draft,
    #[serde(rename = "ready")]
    Ready,
}

/// File diff for review display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    pub path: String,
    pub old_hash: Option<ObjectHash>,
    pub new_hash: Option<ObjectHash>,
    pub diff: String,
}
