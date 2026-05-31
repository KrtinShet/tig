use crate::types::{Object, ObjectHash};
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

/// Content-addressed object store for immutable objects
pub struct ObjectStore {
    root: PathBuf,
}

impl ObjectStore {
    /// Open or create an object store at the given path
    pub fn open(path: PathBuf) -> Result<Self> {
        fs::create_dir_all(&path)?;
        Ok(Self { root: path })
    }

    /// Store an object and return its content hash
    pub fn put(&self, object: &Object) -> Result<ObjectHash> {
        let bytes = Self::serialize(object)?;
        let hash = hex::encode(Sha256::digest(&bytes));
        let path = self.object_path(&hash);
        crate::util::atomic_write(&path, &bytes)
            .with_context(|| format!("Failed to write object at {}", path.display()))?;
        Ok(hash)
    }

    /// Retrieve an object by its hash
    pub fn get(&self, hash: &ObjectHash) -> Result<Object> {
        let path = self.object_path(hash);
        let data = fs::read(&path)
            .with_context(|| format!("Object not found: {}", hash))?;
        Self::deserialize(&data)
            .with_context(|| format!("Failed to deserialize object: {}", hash))
    }

    /// Encode an object for on-disk storage.
    ///
    /// A one-byte tag selects the payload format. Blobs are stored as their raw
    /// bytes (tag `B`) to avoid the ~4x bloat of JSON integer arrays and to keep
    /// arbitrary binary content intact. Trees and snapshots are small structured
    /// metadata, so they stay JSON (tag `J`).
    fn serialize(object: &Object) -> Result<Vec<u8>> {
        match object {
            Object::Blob { content } => {
                let mut bytes = Vec::with_capacity(content.len() + 1);
                bytes.push(b'B');
                bytes.extend_from_slice(content);
                Ok(bytes)
            }
            other => {
                let mut bytes = vec![b'J'];
                bytes.extend_from_slice(&serde_json::to_vec(other)?);
                Ok(bytes)
            }
        }
    }

    /// Decode an object from its on-disk representation.
    fn deserialize(data: &[u8]) -> Result<Object> {
        match data.split_first() {
            Some((b'B', content)) => Ok(Object::Blob { content: content.to_vec() }),
            Some((b'J', json)) => Ok(serde_json::from_slice(json)?),
            _ => anyhow::bail!("Unrecognized object encoding"),
        }
    }

    /// Check if an object exists
    pub fn has(&self, hash: &ObjectHash) -> bool {
        self.object_path(hash).exists()
    }

    /// Compute the hash of an object without storing it
    pub fn hash(object: &Object) -> Result<ObjectHash> {
        let bytes = Self::serialize(object)?;
        Ok(hex::encode(Sha256::digest(&bytes)))
    }

    fn object_path(&self, hash: &str) -> PathBuf {
        self.root.join(hash)
    }

    /// Materialize a tree object to a directory
    pub fn materialize_tree(&self, tree_hash: &ObjectHash, target_dir: &std::path::Path) -> Result<()> {
        let tree = self.get(tree_hash)?;
        let entries = match &tree {
            Object::Tree { entries } => entries.clone(),
            _ => anyhow::bail!("Not a tree object"),
        };

        fs::create_dir_all(target_dir)?;

        for (path, hash) in entries {
            let blob = self.get(&hash)?;
            let content = match &blob {
                Object::Blob { content } => content.clone(),
                _ => continue,
            };
            let file_path = target_dir.join(&path);
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&file_path, content)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_blob_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ObjectStore::open(tmp.path().to_path_buf()).unwrap();

        let blob = Object::Blob {
            content: b"hello world".to_vec(),
        };
        let hash = store.put(&blob).unwrap();
        let retrieved = store.get(&hash).unwrap();

        assert_eq!(blob, retrieved);
    }

    #[test]
    fn test_tree_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ObjectStore::open(tmp.path().to_path_buf()).unwrap();

        let mut entries = HashMap::new();
        entries.insert("hello.txt".to_string(), "abc123".to_string());

        let tree = Object::Tree { entries };
        let hash = store.put(&tree).unwrap();
        let retrieved = store.get(&hash).unwrap();

        assert_eq!(tree, retrieved);
    }

    #[test]
    fn blob_is_stored_as_raw_bytes_not_json_array() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ObjectStore::open(tmp.path().to_path_buf()).unwrap();

        // 4 KB of bytes that JSON would encode as an int array (~4x bloat).
        let content: Vec<u8> = (0u8..=255).cycle().take(4096).collect();
        let hash = store.put(&Object::Blob { content: content.clone() }).unwrap();

        let stored = fs::read(tmp.path().join(&hash)).unwrap();
        assert!(
            stored.len() <= content.len() + 8,
            "blob stored bloated: {} bytes on disk for {} bytes of content",
            stored.len(),
            content.len()
        );
        assert!(
            stored.ends_with(&content),
            "raw content should be present verbatim on disk"
        );
    }

    #[test]
    fn blob_roundtrips_arbitrary_binary() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ObjectStore::open(tmp.path().to_path_buf()).unwrap();

        // Invalid UTF-8 / NUL bytes — must survive a round-trip untouched.
        let blob = Object::Blob { content: vec![0, 159, 146, 150, 255, 0, 1, 2] };
        let hash = store.put(&blob).unwrap();
        assert_eq!(store.get(&hash).unwrap(), blob);
    }

    #[test]
    fn test_content_addressed() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ObjectStore::open(tmp.path().to_path_buf()).unwrap();

        let blob1 = Object::Blob {
            content: b"same content".to_vec(),
        };
        let blob2 = Object::Blob {
            content: b"same content".to_vec(),
        };

        let hash1 = store.put(&blob1).unwrap();
        let hash2 = store.put(&blob2).unwrap();

        assert_eq!(hash1, hash2);
    }
}
