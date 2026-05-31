use crate::types::ProjectMeta;
use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};

/// Tig project layout
pub struct Project {
    pub root: PathBuf,
    pub meta: ProjectMeta,
}

impl Project {
    /// Initialize a new Tig project at the given path
    pub fn init(path: &Path, name: Option<String>) -> Result<Self> {
        let tig_dir = path.join(".tig");
        fs::create_dir_all(&tig_dir)?;
        fs::create_dir_all(tig_dir.join("objects"))?;
        fs::create_dir_all(tig_dir.join("refs"))?;
        fs::create_dir_all(tig_dir.join("workspaces"))?;
        fs::create_dir_all(tig_dir.join("runs"))?;
        fs::create_dir_all(tig_dir.join("reviews"))?;

        let meta = ProjectMeta {
            name: name.unwrap_or_else(|| {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("untitled")
                    .to_string()
            }),
            created_at: Utc::now().to_rfc3339(),
            default_branch: "main".to_string(),
        };

        let meta_path = tig_dir.join("metadata.json");
        let meta_json = serde_json::to_string_pretty(&meta)?;
        crate::util::atomic_write(&meta_path, meta_json)
            .with_context(|| format!("Failed to write project metadata to {}", meta_path.display()))?;

        Ok(Self {
            root: path.to_path_buf(),
            meta,
        })
    }

    /// Open an existing Tig project
    pub fn open(path: &Path) -> Result<Self> {
        let meta_path = path.join(".tig").join("metadata.json");
        let meta_json = fs::read_to_string(&meta_path)
            .with_context(|| format!("Not a Tig project: {}. Run 'tig init' first.", path.display()))?;
        let meta: ProjectMeta = serde_json::from_str(&meta_json)?;

        Ok(Self {
            root: path.to_path_buf(),
            meta,
        })
    }

    /// Check if a directory is a Tig project
    pub fn exists(path: &Path) -> bool {
        path.join(".tig").join("metadata.json").exists()
    }

    /// Get the .tig directory path
    pub fn tig_dir(&self) -> PathBuf {
        self.root.join(".tig")
    }

    /// Get the objects directory path
    pub fn objects_dir(&self) -> PathBuf {
        self.tig_dir().join("objects")
    }

    /// Get the workspaces directory path
    pub fn workspaces_dir(&self) -> PathBuf {
        self.tig_dir().join("workspaces")
    }

    /// Get the runs directory path
    pub fn runs_dir(&self) -> PathBuf {
        self.tig_dir().join("runs")
    }

    /// Get the reviews directory path
    pub fn reviews_dir(&self) -> PathBuf {
        self.tig_dir().join("reviews")
    }

    /// Get the refs directory path
    pub fn refs_dir(&self) -> PathBuf {
        self.tig_dir().join("refs")
    }
}
