use crate::object_store::ObjectStore;
use crate::project::Project;
use crate::types::{Change, Object, ObjectHash, SnapshotId, Workspace};
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::time::UNIX_EPOCH;
use walkdir::WalkDir;

/// Per-file stat cache entry: if a file's mtime and size are unchanged, its
/// content hash can be reused without re-reading or re-hashing the file.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    mtime_ns: u128,
    size: u64,
    hash: ObjectHash,
}

/// Per-workspace stat cache (path -> entry), persisted between snapshots.
#[derive(Debug, Default, Serialize, Deserialize)]
struct TreeCache {
    entries: HashMap<String, CacheEntry>,
}

/// Return the cached content hash for `path` iff its mtime and size still match.
fn cache_lookup<'a>(
    cache: &'a TreeCache,
    path: &str,
    mtime_ns: u128,
    size: u64,
) -> Option<&'a ObjectHash> {
    cache
        .entries
        .get(path)
        .filter(|e| e.mtime_ns == mtime_ns && e.size == size)
        .map(|e| &e.hash)
}

/// Snapshot engine for creating and managing snapshots
pub struct SnapshotEngine<'a> {
    project: &'a Project,
    store: &'a ObjectStore,
}

impl<'a> SnapshotEngine<'a> {
    pub fn new(project: &'a Project, store: &'a ObjectStore) -> Self {
        Self { project, store }
    }

    /// Create a snapshot from the current workspace file tree
    pub fn create(
        &self,
        workspace: &Workspace,
        changes: Vec<Change>,
    ) -> Result<SnapshotId> {
        let files_dir = self.project.workspaces_dir().join(&workspace.name).join("files");
        let tree_hash = self.build_tree(&files_dir, &workspace.name)?;

        let snapshot = Object::Snapshot {
            parent: workspace.current_snapshot.clone(),
            tree: tree_hash,
            workspace: workspace.name.clone(),
            actor: workspace.actor.clone(),
            goal: workspace.goal.clone(),
            timestamp: Utc::now().to_rfc3339(),
            changes,
        };

        let hash = self.store.put(&snapshot)?;

        // Save snapshot ref
        let refs_dir = self.project.refs_dir().join("snapshots");
        fs::create_dir_all(&refs_dir)?;
        crate::util::atomic_write(&refs_dir.join(format!("{}.json", &hash)), serde_json::to_string_pretty(&snapshot)?)?;

        Ok(hash)
    }

    /// Build a tree object from a directory.
    ///
    /// Uses a per-workspace stat cache: a file whose mtime and size are
    /// unchanged since the last snapshot reuses its previously computed content
    /// hash, so unchanged files are never re-read or re-hashed. Only changed (or
    /// new) files pay the read+hash cost, making snapshot creation proportional
    /// to the edit rather than the whole workspace.
    fn build_tree(&self, dir: &std::path::Path, workspace_name: &str) -> Result<ObjectHash> {
        let mut entries = HashMap::new();

        if !dir.exists() {
            // Empty tree
            let tree = Object::Tree { entries };
            return self.store.put(&tree);
        }

        let cache = self.load_tree_cache(workspace_name);
        let mut next_cache = TreeCache::default();

        for entry in WalkDir::new(dir)
            .min_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let relative = entry.path().strip_prefix(dir)?.to_string_lossy().to_string();

                let meta = entry.metadata()?;
                let size = meta.len();
                let mtime_ns = meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_nanos())
                    .unwrap_or(0);

                // Reuse the cached hash only if stat matches AND the object is
                // still present in the store.
                let hash = match cache_lookup(&cache, &relative, mtime_ns, size) {
                    Some(h) if self.store.has(h) => h.clone(),
                    _ => {
                        let content = fs::read(entry.path())?;
                        let blob = Object::Blob { content };
                        self.store.put(&blob)?
                    }
                };

                next_cache.entries.insert(
                    relative.clone(),
                    CacheEntry { mtime_ns, size, hash: hash.clone() },
                );
                entries.insert(relative, hash);
            }
        }

        self.save_tree_cache(workspace_name, &next_cache)?;

        let tree = Object::Tree { entries };
        self.store.put(&tree)
    }

    fn tree_cache_path(&self, workspace_name: &str) -> std::path::PathBuf {
        self.project
            .workspaces_dir()
            .join(workspace_name)
            .join("tree_cache.json")
    }

    /// Load the workspace's stat cache, treating any error (missing/corrupt) as
    /// an empty cache — a cache miss only costs a re-hash, never correctness.
    fn load_tree_cache(&self, workspace_name: &str) -> TreeCache {
        fs::read_to_string(self.tree_cache_path(workspace_name))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save_tree_cache(&self, workspace_name: &str, cache: &TreeCache) -> Result<()> {
        crate::util::atomic_write(
            &self.tree_cache_path(workspace_name),
            serde_json::to_string_pretty(cache)?,
        )
    }

    /// Get a snapshot by ID
    pub fn get(&self, id: &SnapshotId) -> Result<Object> {
        self.store.get(id)
    }

    /// List all snapshots for a workspace
    pub fn list_for_workspace(&self, workspace_name: &str) -> Result<Vec<(SnapshotId, Object)>> {
        let mut snapshots = Vec::new();
        let refs_dir = self.project.refs_dir().join("snapshots");

        if !refs_dir.exists() {
            return Ok(snapshots);
        }

        for entry in fs::read_dir(&refs_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let data = fs::read_to_string(entry.path())?;
                let snapshot: Object = serde_json::from_str(&data)?;
                if let Object::Snapshot { workspace, .. } = &snapshot {
                    if workspace == workspace_name {
                        let id = entry.file_name().to_string_lossy().to_string();
                        let id = id.trim_end_matches(".json").to_string();
                        snapshots.push((id, snapshot));
                    }
                }
            }
        }

        // Sort by timestamp
        snapshots.sort_by(|a, b| {
            let ts_a = match &a.1 {
                Object::Snapshot { timestamp, .. } => timestamp.clone(),
                _ => String::new(),
            };
            let ts_b = match &b.1 {
                Object::Snapshot { timestamp, .. } => timestamp.clone(),
                _ => String::new(),
            };
            ts_a.cmp(&ts_b)
        });

        Ok(snapshots)
    }

    /// List all passing snapshots for a workspace
    pub fn list_passing(&self, workspace_name: &str, runs: &[(SnapshotId, crate::types::Run)]) -> Result<Vec<(SnapshotId, Object)>> {
        let all = self.list_for_workspace(workspace_name)?;
        let passing_snapshots: std::collections::HashSet<String> = runs
            .iter()
            .filter(|(_, r)| r.status == crate::types::RunStatus::Passed)
            .map(|(id, _)| id.clone())
            .collect();

        Ok(all.into_iter()
            .filter(|(id, _)| passing_snapshots.contains(id))
            .collect())
    }

    /// Get the latest snapshot for a workspace
    pub fn latest(&self, workspace_name: &str) -> Result<Option<(SnapshotId, Object)>> {
        let mut snapshots = self.list_for_workspace(workspace_name)?;
        Ok(snapshots.pop())
    }

    /// Materialize a snapshot's tree into a directory
    pub fn materialize(&self, snapshot_id: &SnapshotId, target_dir: &std::path::Path) -> Result<()> {
        let snapshot = self.store.get(snapshot_id)?;
        let tree_hash = match &snapshot {
            Object::Snapshot { tree, .. } => tree.clone(),
            _ => anyhow::bail!("Not a snapshot object"),
        };
        self.store.materialize_tree(&tree_hash, target_dir)
    }

    /// Compute diff between two snapshots
    pub fn diff(&self, source_id: &SnapshotId, target_id: &SnapshotId) -> Result<Vec<crate::types::FileDiff>> {
        let source_tree = self.get_tree(source_id)?;
        let target_tree = self.get_tree(target_id)?;

        let mut diffs = Vec::new();
        let all_paths: std::collections::HashSet<String> = source_tree
            .keys()
            .chain(target_tree.keys())
            .cloned()
            .collect();

        for path in all_paths {
            let source_hash = source_tree.get(&path).cloned();
            let target_hash = target_tree.get(&path).cloned();

            if source_hash == target_hash {
                continue;
            }

            let source_content = self.read_blob_bytes(source_hash.as_ref())?;
            let target_content = self.read_blob_bytes(target_hash.as_ref())?;

            let diff = render_file_diff(&source_content, &target_content, &path);

            diffs.push(crate::types::FileDiff {
                path,
                old_hash: source_hash,
                new_hash: target_hash,
                diff,
            });
        }

        Ok(diffs)
    }

    /// Read a blob's raw bytes by hash, or empty bytes if the hash is absent
    /// (i.e. the file was added or deleted between the two snapshots).
    fn read_blob_bytes(&self, hash: Option<&ObjectHash>) -> Result<Vec<u8>> {
        match hash {
            Some(h) => match self.store.get(h)? {
                Object::Blob { content } => Ok(content),
                _ => Ok(Vec::new()),
            },
            None => Ok(Vec::new()),
        }
    }

    fn get_tree(&self, snapshot_id: &SnapshotId) -> Result<HashMap<String, ObjectHash>> {
        let snapshot = self.store.get(snapshot_id)?;
        let tree_hash = match &snapshot {
            Object::Snapshot { tree, .. } => tree.clone(),
            _ => anyhow::bail!("Not a snapshot object"),
        };
        let tree = self.store.get(&tree_hash)?;
        match tree {
            Object::Tree { entries } => Ok(entries),
            _ => anyhow::bail!("Not a tree object"),
        }
    }
}

/// Heuristic: treat content as binary if it contains a NUL byte or is not valid
/// UTF-8. Matches Git's practical definition closely enough for review display.
fn is_binary(bytes: &[u8]) -> bool {
    bytes.contains(&0) || std::str::from_utf8(bytes).is_err()
}

/// Render the diff for one file. Binary content (either side) is reported as a
/// single line rather than decoded lossily and mangled into a fake text patch.
fn render_file_diff(old: &[u8], new: &[u8], path: &str) -> String {
    if is_binary(old) || is_binary(new) {
        return format!("Binary file changed: {}", path);
    }
    // Both sides are valid UTF-8 here, so these never panic.
    let old_s = std::str::from_utf8(old).unwrap();
    let new_s = std::str::from_utf8(new).unwrap();
    format_diff(old_s, new_s, path)
}

/// Produce a real unified diff between two file contents.
///
/// Backed by `diffy`, so the output is a minimal, standards-compliant patch
/// (proper `@@` hunks, context lines, no spurious removals) that downstream
/// tools — including `diffy::apply` — can consume.
fn format_diff(old: &str, new: &str, _path: &str) -> String {
    diffy::create_patch(old, new).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(mtime_ns: u128, size: u64, hash: &str) -> CacheEntry {
        CacheEntry { mtime_ns, size, hash: hash.to_string() }
    }

    #[test]
    fn tree_cache_hits_on_matching_mtime_and_size() {
        let mut cache = TreeCache::default();
        cache.entries.insert("a.txt".into(), entry(123, 10, "hashA"));
        assert_eq!(cache_lookup(&cache, "a.txt", 123, 10), Some(&"hashA".to_string()));
    }

    #[test]
    fn tree_cache_misses_on_size_change() {
        let mut cache = TreeCache::default();
        cache.entries.insert("a.txt".into(), entry(123, 10, "hashA"));
        assert_eq!(cache_lookup(&cache, "a.txt", 123, 11), None);
    }

    #[test]
    fn tree_cache_misses_on_mtime_change() {
        let mut cache = TreeCache::default();
        cache.entries.insert("a.txt".into(), entry(123, 10, "hashA"));
        assert_eq!(cache_lookup(&cache, "a.txt", 124, 10), None);
    }

    #[test]
    fn tree_cache_misses_on_unknown_path() {
        let cache = TreeCache::default();
        assert_eq!(cache_lookup(&cache, "ghost.txt", 1, 1), None);
    }

    #[test]
    fn binary_content_is_detected() {
        assert!(is_binary(&[0x00, 0x01, 0x02]), "NUL byte => binary");
        assert!(is_binary(&[0xff, 0xfe, 0x99]), "invalid UTF-8 => binary");
        assert!(!is_binary(b"hello\nworld"), "ascii text => not binary");
        assert!(!is_binary("héllo".as_bytes()), "valid multibyte UTF-8 => not binary");
        assert!(!is_binary(&[]), "empty (added/deleted file) => not binary");
    }

    #[test]
    fn diff_of_binary_blob_is_not_mangled() {
        let old = b"\x00\x01\x02BINARY";
        let new = b"\x00\x09\x09CHANGED";
        let rendered = render_file_diff(old, new, "image.png");
        assert!(rendered.contains("Binary file"), "got: {}", rendered);
        assert!(!rendered.contains("+++"), "must not emit a text patch for binary");
    }

    #[test]
    fn diff_of_text_uses_unified_format() {
        let rendered = render_file_diff(b"a\nb\n", b"a\nc\n", "f.txt");
        assert!(rendered.contains("-b"), "got: {}", rendered);
        assert!(rendered.contains("+c"), "got: {}", rendered);
    }

    #[test]
    fn diff_of_prepended_line_has_no_spurious_removals() {
        // Inserting one line at the top must not report the existing lines as
        // removed. The naive index-by-index diff does exactly that.
        let old = "alpha\nbeta\ngamma\n";
        let new = "inserted\nalpha\nbeta\ngamma\n";

        let patch = format_diff(old, new, "file.txt");

        let removals = patch
            .lines()
            .filter(|l| l.starts_with('-') && !l.starts_with("--"))
            .count();
        assert_eq!(removals, 0, "pure insertion should remove nothing:\n{}", patch);
        assert!(patch.contains("+inserted"), "patch:\n{}", patch);
    }
}
