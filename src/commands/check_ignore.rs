use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::git::{
    ignore::{check_ignore, gitignore_read},
    repo::repo_find,
};

pub fn run(paths: &[PathBuf]) -> Result<()> {
    let repo = repo_find(".", true)?.unwrap();

    let worktree = repo.worktree.canonicalize()?;

    let rules = gitignore_read(&repo)?;
    for path in paths {
        // Normalize to repo-relative path
        let abs = path.canonicalize()?;
        let rel = abs
            .strip_prefix(&worktree)
            .with_context(|| format!("Path {abs:?} not inside repo"))?;

        let rel_str = rel.to_string_lossy();

        if check_ignore(&rules, &rel_str)? {
            println!("{rel_str}");
        }
    }

    Ok(())
}
