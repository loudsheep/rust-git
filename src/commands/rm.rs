use std::{fs, path::PathBuf};

use anyhow::{Context, Result, bail};

use crate::git::{
    index::{read_index, write_index},
    repo::{GitRepository, repo_find},
};

pub fn run(paths: &[PathBuf]) -> Result<()> {
    let repo = repo_find(".", true)?.unwrap();

    rm(&repo, paths, true, false)
}

pub fn rm(repo: &GitRepository, paths: &[PathBuf], delete: bool, skip_missing: bool) -> Result<()> {
    let mut index = read_index(&repo)?;

    let worktree = repo.worktree.canonicalize()?;

    let mut relpaths = Vec::new();
    for path in paths {
        let abs = path.canonicalize()?;
        if !abs.starts_with(&worktree) {
            bail!("Cannot remove paths outside of worktree: {:?}", path);
        }
        let rel = abs
            .strip_prefix(&worktree)
            .with_context(|| format!("Path {abs:?} not inside repo"))?
            .to_path_buf();
        relpaths.push(rel);
    }
    let mut kept_entries = Vec::new();
    let mut remove_files = Vec::new();

    for e in &index.entries {
        let full_path = worktree.join(&e.path);
        if relpaths.contains(&full_path) {
            remove_files.push(full_path);
        } else {
            kept_entries.push(e.clone());
        }
    }

    for ap in &relpaths {
        if !remove_files.contains(ap) && !skip_missing {
            bail!("Cannot remove paths not in the index: {:?}", ap);
        }
    }

    if delete {
        for path in &remove_files {
            if path.exists() {
                fs::remove_file(path)?;
            }
        }
    }

    index.entries = kept_entries;
    write_index(repo, &index)?;

    Ok(())
}
