use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::git::{index::GitIndex, objects::{object_write, GitObject, GitObjectType}, repo::GitRepository};

#[derive(Debug, Clone)]
pub struct GitTreeLeaf {
    pub mode: String,
    pub path: String,
    pub sha: [u8; 20],
}

/// A tree object (list of entries)
#[derive(Debug, Clone)]
pub struct GitTree {
    pub entries: Vec<GitTreeLeaf>,
}

impl GitObject for GitTree {
    fn serialize(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        for entry in &self.entries {
            out.extend_from_slice(entry.mode.as_bytes());
            out.push(b' ');
            out.extend_from_slice(entry.path.as_bytes());
            out.push(0); // null terminator
            out.extend_from_slice(&entry.sha);
        }
        Ok(out)
    }

    fn deserialize(data: &[u8]) -> Result<Self> {
        let mut entries = Vec::new();
        let mut pos = 0usize;

        while pos < data.len() {
            // Parse mode until space
            let space = data[pos..]
                .iter()
                .position(|&b| b == b' ')
                .context("Tree: expected space after mode")?
                + pos;
            let mode = String::from_utf8_lossy(&data[pos..space]).to_string();

            // Parse path until null byte
            let null = data[space + 1..]
                .iter()
                .position(|&b| b == 0)
                .context("Tree: expected null after path")?
                + (space + 1);
            let path = String::from_utf8_lossy(&data[space + 1..null]).to_string();

            // Next 20 bytes = SHA1
            let sha_start = null + 1;
            let sha_end = sha_start + 20;
            if sha_end > data.len() {
                anyhow::bail!("Tree: incomplete SHA1 for entry '{}'", path);
            }
            let mut sha = [0u8; 20];
            sha.copy_from_slice(&data[sha_start..sha_end]);

            entries.push(GitTreeLeaf { mode, path, sha });

            pos = sha_end;
        }

        Ok(Self { entries })
    }

    fn init() -> Result<Self> {
        Ok(Self {
            entries: Vec::new(),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub fn tree_from_index(repo: &GitRepository, index: &GitIndex) -> Result<String> {
    build_tree(repo, Path::new(""), index)
}

/// Recursively descend directories and construct GitTree objects.
fn build_tree(repo: &GitRepository, prefix: &Path, index: &GitIndex) -> Result<String> {
    // Group entries by directory name
    let mut files: Vec<GitTreeLeaf> = Vec::new();
    let mut dirs: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for e in &index.entries {
        let path = PathBuf::from(&e.path);
        if let Ok(rel) = path.strip_prefix(prefix) {
            let comps: Vec<_> = rel.components().collect();
            if comps.len() == 1 {
                // Direct child file
                let sha_bytes = hex::decode(&e.sha)
                    .with_context(|| format!("Invalid SHA in index for {}", e.path))?;
                let mut sha_arr = [0u8; 20];
                sha_arr.copy_from_slice(&sha_bytes);

                files.push(GitTreeLeaf {
                    mode: "100644".to_string(),
                    path: rel.to_string_lossy().to_string(),
                    sha: sha_arr,
                });
            } else {
                // Goes into subdir
                let dirname = comps[0].as_os_str().to_string_lossy().to_string();
                dirs.entry(dirname).or_default().push(e.path.clone());
            }
        }
    }

    let mut entries: Vec<GitTreeLeaf> = Vec::new();

    // Add files
    entries.extend(files);

    // Recurse into dirs
    for (dirname, _) in dirs {
        let subprefix = prefix.join(&dirname);
        let sub_sha = build_tree(repo, &subprefix, index)?;

        let sha_bytes = hex::decode(&sub_sha)?;
        let mut sha_arr = [0u8; 20];
        sha_arr.copy_from_slice(&sha_bytes);

        entries.push(GitTreeLeaf {
            mode: "40000".to_string(),
            path: dirname,
            sha: sha_arr,
        });
    }

    // Sort entries by path, just like Git
    entries.sort_by(|a, b| a.path.cmp(&b.path));

    // Write this tree object
    let tree = GitTree { entries };
    let sha = object_write(repo, &tree, &GitObjectType::tree, true)?;
    Ok(sha)
}
