use crate::object_store::ObjectStore;
use crate::project::Project;
use crate::review::ReviewService;
use crate::run::RunStore;
use crate::snapshot::SnapshotEngine;
use crate::types::{Change, Object, ReviewUnit, Run, SnapshotId, Workspace};
use crate::workspace::WorkspaceManager;
use anyhow::Result;
use std::path::Path;

/// Programmatic API for agents to interact with Tig without shelling out.
/// All methods create their managers on-demand from owned Project and ObjectStore.
pub struct Tig {
    project: Project,
    store: ObjectStore,
}

impl Tig {
    /// Initialize a new Tig project at the given path
    pub fn init(path: &Path, name: Option<String>) -> Result<Self> {
        let project = Project::init(path, name)?;
        let store = ObjectStore::open(project.objects_dir())?;
        Ok(Self { project, store })
    }

    /// Open an existing Tig project
    pub fn open(path: &Path) -> Result<Self> {
        let project = Project::open(path)?;
        let store = ObjectStore::open(project.objects_dir())?;
        Ok(Self { project, store })
    }

    /// Acquire the project-wide write lock. Held for the duration of a single
    /// mutating operation so concurrent `tig` processes cannot interleave writes
    /// to shared state. Released when the returned guard is dropped.
    fn lock(&self) -> Result<crate::lock::ProjectLock> {
        crate::lock::ProjectLock::acquire(&self.project.tig_dir())
    }

    /// Create a new workspace
    pub fn create_workspace(
        &self,
        name: &str,
        actor: &str,
        goal: Option<String>,
    ) -> Result<Workspace> {
        let _lock = self.lock()?;
        let ws_mgr = WorkspaceManager::new(&self.project);
        ws_mgr.create(name, actor, goal)
    }

    /// List all workspaces
    pub fn list_workspaces(&self) -> Result<Vec<Workspace>> {
        let ws_mgr = WorkspaceManager::new(&self.project);
        ws_mgr.list()
    }

    /// Get a workspace by name
    pub fn get_workspace(&self, name: &str) -> Result<Workspace> {
        let ws_mgr = WorkspaceManager::new(&self.project);
        ws_mgr.load(name)
    }

    /// Read a file from a workspace
    pub fn read_file(&self, workspace: &str, path: &str) -> Result<String> {
        let ws_mgr = WorkspaceManager::new(&self.project);
        ws_mgr.read_file(workspace, path)
    }

    /// Write a file to a workspace and create a snapshot
    pub fn write_file(
        &self,
        workspace: &str,
        path: &str,
        content: &str,
    ) -> Result<SnapshotId> {
        let _lock = self.lock()?;
        let ws_mgr = WorkspaceManager::new(&self.project);
        ws_mgr.write_file(workspace, path, content)?;

        let mut ws = ws_mgr.load(workspace)?;
        let changes = vec![Change::FileModified { path: path.to_string() }];

        let snapshot_engine = SnapshotEngine::new(&self.project, &self.store);
        let snapshot_id = snapshot_engine.create(&ws, changes)?;
        ws.current_snapshot = Some(snapshot_id.clone());
        ws_mgr.save(&ws)?;

        Ok(snapshot_id)
    }

    /// Apply a patch to a file and create a snapshot
    pub fn apply_patch(
        &self,
        workspace: &str,
        path: &str,
        patch: &str,
    ) -> Result<SnapshotId> {
        let _lock = self.lock()?;
        let ws_mgr = WorkspaceManager::new(&self.project);
        ws_mgr.apply_patch(workspace, path, patch)?;

        let mut ws = ws_mgr.load(workspace)?;
        let changes = vec![Change::PatchApplied {
            path: path.to_string(),
            description: "Applied patch".to_string(),
        }];

        let snapshot_engine = SnapshotEngine::new(&self.project, &self.store);
        let snapshot_id = snapshot_engine.create(&ws, changes)?;
        ws.current_snapshot = Some(snapshot_id.clone());
        ws_mgr.save(&ws)?;

        Ok(snapshot_id)
    }

    /// Run a check against the current workspace state
    pub fn run_check(&self, workspace: &str, command: &str, actor: &str) -> Result<Run> {
        let _lock = self.lock()?;
        let ws_mgr = WorkspaceManager::new(&self.project);
        let ws = ws_mgr.load(workspace)?;
        let files_dir = ws_mgr.files_dir(workspace);

        let snapshot_engine = SnapshotEngine::new(&self.project, &self.store);
        let snapshot_id = snapshot_engine.create(
            &ws,
            vec![Change::FileModified { path: "*".to_string() }],
        )?;

        let mut ws = ws;
        ws.current_snapshot = Some(snapshot_id.clone());
        ws_mgr.save(&ws)?;

        let run_store = RunStore::new(&self.project);
        let run = run_store.execute(snapshot_id, command, actor, &files_dir)?;

        Ok(run)
    }

    /// Create a review unit
    pub fn create_review_unit(
        &self,
        workspace: &str,
        from: &str,
        target: &str,
    ) -> Result<ReviewUnit> {
        let _lock = self.lock()?;
        let snapshot_engine = SnapshotEngine::new(&self.project, &self.store);
        let review_service = ReviewService::new(&self.project, &snapshot_engine);

        let source_snapshot = if from == "latest" || from == "latest-passing" {
            let run_store = RunStore::new(&self.project);
            let passing = snapshot_engine.list_passing(
                workspace,
                &run_store.list_all_with_snapshot()?,
            )?;
            passing
                .last()
                .map(|(id, _)| id.clone())
                .unwrap_or_else(|| {
                    let all = snapshot_engine.list_for_workspace(workspace).unwrap_or_default();
                    all.last().map(|(id, _)| id.clone()).unwrap_or_default()
                })
        } else {
            from.to_string()
        };

        let mut review = review_service.create(source_snapshot.clone(), target.to_string())?;

        // Attach runs for the source snapshot
        let run_store = RunStore::new(&self.project);
        let runs = run_store.list_all_with_snapshot()?;
        let snapshot_runs: Vec<String> = runs
            .iter()
            .filter(|(sid, _)| sid == &source_snapshot)
            .map(|(_, r)| r.id.clone())
            .collect();
        review.runs = snapshot_runs;
        review_service.save(&review)?;

        Ok(review)
    }

    /// List snapshots for a workspace
    pub fn list_snapshots(&self, workspace: &str) -> Result<Vec<(SnapshotId, Object)>> {
        let snapshot_engine = SnapshotEngine::new(&self.project, &self.store);
        snapshot_engine.list_for_workspace(workspace)
    }

    /// List runs
    pub fn list_runs(&self) -> Result<Vec<Run>> {
        let run_store = RunStore::new(&self.project);
        run_store.list()
    }

    /// List review units
    pub fn list_reviews(&self) -> Result<Vec<ReviewUnit>> {
        let snapshot_engine = SnapshotEngine::new(&self.project, &self.store);
        let review_service = ReviewService::new(&self.project, &snapshot_engine);
        review_service.list()
    }

    /// Export a review unit to Git
    pub fn export_to_git(&self, review_id: &str) -> Result<String> {
        let _lock = self.lock()?;
        let snapshot_engine = SnapshotEngine::new(&self.project, &self.store);
        let review_service = ReviewService::new(&self.project, &snapshot_engine);
        let review = review_service.get(&review_id.to_string())?;

        let git_bridge = crate::git_bridge::GitBridge::new(&self.project);
        git_bridge.export_commit(&review)
    }
}
