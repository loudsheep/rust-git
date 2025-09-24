use std::{fs, path::PathBuf};

use anyhow::{Context, Result, bail};

use crate::{
    commands::rm::rm,
    git::{
        index::{GitIndexEntry, read_index, write_index},
        objects::{GitObjectType, object_hash},
        repo::{GitRepository, repo_find},
    },
};

pub fn run(paths: &[PathBuf]) -> Result<()> {
    let repo = repo_find(".", true)?.unwrap();

    add(&repo, paths)
}

pub fn add(repo: &GitRepository, paths: &[PathBuf]) -> Result<()> {
    rm(repo, paths, false, true)?;

    let worktree = repo.worktree.canonicalize()?;
    let mut clean_paths = Vec::new();

    for path in paths {
        let abs = path.canonicalize()?;
        if !abs.starts_with(&worktree) || !abs.is_file() {
            bail!("Not a file, or outside the worktree: {:?}", path);
        }

        let rel = abs
            .strip_prefix(&worktree)
            .with_context(|| format!("Path {abs:?} not inside repo"))?
            .to_path_buf();

        clean_paths.push((abs, rel));
    }

    let mut index = read_index(repo)?;

    for (abspath, relpath) in clean_paths {
        let data = fs::read(&abspath)?;

        let sha = object_hash(&repo, data, &GitObjectType::blob)?;

        let meta = fs::metadata(&abspath)?;

        let ctime_s = meta
            .created()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i32)
            .unwrap_or(0);

        let mtime_s = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i32)
            .unwrap_or(0);

        let entry = GitIndexEntry {
            // Git packs ctime/mtime as seconds, ignoring nanos for now
            ctime: ctime_s as u32,
            mtime: mtime_s as u32,
            dev: 0,
            ino: 0,
            // combine file type + permissions into one mode
            mode: (0b1000 << 12) | 0o644, // regular file + rw-r--r--
            uid: 0,
            gid: 0,
            size: meta.len() as u32,
            sha,
            flags: 0, // you can OR bits for assume-valid/stage if needed
            path: relpath.to_string_lossy().to_string(),
        };

        index.entries.push(entry);
    }

    // Write index back
    write_index(repo, &index)?;
    Ok(())
}
