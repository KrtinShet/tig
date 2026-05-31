use tig::api::Tig;
use tig::project::Project;

#[test]
fn test_full_workflow() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path();

    // 1. Init project
    let tig = Tig::init(path, Some("test".to_string())).unwrap();
    assert!(Project::exists(path));
    assert!(path.join(".tig").exists());
    assert!(path.join(".tig").join("objects").exists());
    assert!(path.join(".tig").join("workspaces").exists());

    // 2. Create workspace
    let ws = tig.create_workspace("fix-bug", "claude", Some("Fix the bug".to_string())).unwrap();
    assert_eq!(ws.name, "fix-bug");
    assert_eq!(ws.actor, "claude");

    // 3. List workspaces
    let workspaces = tig.list_workspaces().unwrap();
    assert_eq!(workspaces.len(), 1);

    // 4. Write file and auto-snapshot
    let snapshot_id = tig.write_file("fix-bug", "src/hello.js", "exports.hello = () => 'world';").unwrap();
    assert!(!snapshot_id.is_empty());

    // 5. Read file back
    let content = tig.read_file("fix-bug", "src/hello.js").unwrap();
    assert_eq!(content, "exports.hello = () => 'world';");

    // 6. List snapshots
    let snapshots = tig.list_snapshots("fix-bug").unwrap();
    assert_eq!(snapshots.len(), 1);

    // 7. Create second workspace
    let ws2 = tig.create_workspace("fix-bug-alt", "codex", Some("Alt fix".to_string())).unwrap();
    assert_eq!(ws2.name, "fix-bug-alt");

    // 8. Write different content
    let _ = tig.write_file("fix-bug-alt", "src/hello.js", "exports.hello = () => 'universe';").unwrap();

    // 9. Create review unit
    let first_snapshot = snapshots.first().unwrap().0.clone();
    let review = tig.create_review_unit("fix-bug", &first_snapshot, &first_snapshot).unwrap();
    assert!(!review.id.is_empty());
    assert_eq!(review.source_snapshot, first_snapshot);

    // 10. List reviews
    let reviews = tig.list_reviews().unwrap();
    assert_eq!(reviews.len(), 1);

    // 11. Export to git
    let commit_hash = tig.export_to_git(&review.id).unwrap();
    assert!(!commit_hash.is_empty());

    // Verify git export directory exists
    let git_export = path.join(".tig").join("git-export");
    assert!(git_export.exists());
    assert!(git_export.join(".git").exists());
}

#[test]
fn test_object_store_content_addressed() {
    let tmp = tempfile::tempdir().unwrap();
    let store = tig::object_store::ObjectStore::open(tmp.path().to_path_buf()).unwrap();

    use tig::types::Object;
    use std::collections::HashMap;

    // Blob
    let blob = Object::Blob { content: b"test".to_vec() };
    let hash = store.put(&blob).unwrap();
    let retrieved = store.get(&hash).unwrap();
    assert_eq!(blob, retrieved);

    // Tree
    let mut entries = HashMap::new();
    entries.insert("a.txt".to_string(), hash.clone());
    let tree = Object::Tree { entries };
    let tree_hash = store.put(&tree).unwrap();
    let retrieved_tree = store.get(&tree_hash).unwrap();
    assert_eq!(tree, retrieved_tree);

    // Same content = same hash
    let blob2 = Object::Blob { content: b"test".to_vec() };
    let hash2 = store.put(&blob2).unwrap();
    assert_eq!(hash, hash2);
}
