use anyhow::{Context, Result};
use fs2::FileExt;
use std::fs::{File, OpenOptions};
use std::path::Path;

/// An exclusive, advisory lock over a Tig project's mutable state.
///
/// Backed by `flock(2)` on `.tig/lock`, so the lock is released automatically
/// when the guard is dropped *and* if the process dies — no stale lockfile to
/// clean up. Hold one of these for the duration of any operation that mutates
/// shared state (objects, refs, workspace metadata, the active-workspace
/// pointer) so two concurrent `tig` processes cannot interleave writes.
#[must_use = "the lock is released as soon as the guard is dropped"]
pub struct ProjectLock {
    // Held only for its lock; closing the file releases the flock.
    _file: File,
}

impl ProjectLock {
    fn open_lock_file(tig_dir: &Path) -> Result<File> {
        let path = tig_dir.join("lock");
        OpenOptions::new()
            .create(true)
            .write(true)
            .open(&path)
            .with_context(|| format!("Failed to open lock file {}", path.display()))
    }

    /// Acquire the lock, blocking until it becomes available.
    pub fn acquire(tig_dir: &Path) -> Result<Self> {
        let file = Self::open_lock_file(tig_dir)?;
        file.lock_exclusive()
            .with_context(|| "Failed to acquire project lock")?;
        Ok(Self { _file: file })
    }

    /// Try to acquire the lock without blocking. Returns `Ok(None)` if another
    /// holder currently owns it.
    pub fn try_acquire(tig_dir: &Path) -> Result<Option<Self>> {
        let file = Self::open_lock_file(tig_dir)?;
        match file.try_lock_exclusive() {
            Ok(()) => Ok(Some(Self { _file: file })),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(anyhow::Error::new(e).context("Failed to try-acquire project lock")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tig_dir() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("inner")).unwrap();
        tmp
    }

    #[test]
    fn second_lock_attempt_fails_while_held() {
        let tmp = tig_dir();
        let _held = ProjectLock::acquire(tmp.path()).unwrap();
        let second = ProjectLock::try_acquire(tmp.path()).unwrap();
        assert!(second.is_none(), "lock must be exclusive while held");
    }

    #[test]
    fn lock_is_released_on_drop() {
        let tmp = tig_dir();
        {
            let _held = ProjectLock::acquire(tmp.path()).unwrap();
        }
        let again = ProjectLock::try_acquire(tmp.path()).unwrap();
        assert!(again.is_some(), "lock must be re-acquirable after the guard drops");
    }
}
