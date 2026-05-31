use anyhow::{Context, Result};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

/// Counter to make temp filenames unique within a process.
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Write `contents` to `path` atomically.
///
/// Writes to a uniquely-named temp file in the same directory, then renames it
/// into place. Same-directory rename is atomic on POSIX filesystems, so a reader
/// (or a crash) ever sees either the old file or the fully-written new one —
/// never a half-written object or ref.
pub fn atomic_write(path: &Path, contents: impl AsRef<[u8]>) -> Result<()> {
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty());
    let dir = parent.unwrap_or_else(|| Path::new("."));

    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .context("atomic_write: target path has no file name")?;
    let seq = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp = dir.join(format!(".{}.{}.{}.tmp", file_name, std::process::id(), seq));

    std::fs::write(&tmp, contents.as_ref())
        .with_context(|| format!("Failed to write temp file {}", tmp.display()))?;
    std::fs::rename(&tmp, path)
        .with_context(|| format!("Failed to rename {} -> {}", tmp.display(), path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn atomic_write_persists_full_contents() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("obj");
        atomic_write(&target, b"hello").unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"hello");
    }

    #[test]
    fn atomic_write_leaves_no_temp_files() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("obj");
        atomic_write(&target, b"data").unwrap();

        let mut entries: Vec<String> = fs::read_dir(tmp.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        entries.sort();
        assert_eq!(entries, vec!["obj".to_string()], "stray temp files left behind");
    }

    #[test]
    fn atomic_write_overwrites_existing() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("obj");
        atomic_write(&target, b"old").unwrap();
        atomic_write(&target, b"new").unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"new");
    }
}
