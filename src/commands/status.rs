use std::{
    collections::HashSet, fs, path::{Path, PathBuf}
};

use anyhow::{Context, Result};

use crate::git::{
    ignore::{check_ignore, gitignore_read}, index::read_index, repo::{repo_find, GitRepository}
};

pub fn run() -> Result<()> {
    let repo = repo_find(".", true)?.unwrap();
    let index = read_index(&repo)?;

    match branch_get_active(&repo)? {
        Some(branch) => println!("On branch {branch}"),
        None => {
            let sha = head_resolve(&repo)?;
            println!("HEAD detached at {sha}");
        }
    }

    let rules = gitignore_read(&repo)?;

    let mut tracked: HashSet<PathBuf> = HashSet::new();
    for entry in &index {
        tracked.insert(PathBuf::from(&entry.path));
    }

    println!("Tracked files:");
    for path in &tracked {
        println!("  {}", path.display());
    }

    println!("\nUntracked files:");
    for path in worktree_files(&repo.worktree)? {
        let rel = path.strip_prefix(&repo.worktree).unwrap();

        if tracked.contains(rel) {
            continue;
        }
        if check_ignore(&rules, &rel.to_string_lossy())? {
            continue;
        }

        println!("  {}", rel.display());
    }

    println!("\nIgnored files:");
    for path in worktree_files(&repo.worktree)? {
        let rel = path.strip_prefix(&repo.worktree).unwrap();
        if check_ignore(&rules, &rel.to_string_lossy())? {
            println!("  {}", rel.display());
        }
    }

    Ok(())
}

pub fn head_resolve(repo: &GitRepository) -> Result<String> {
    let head_path = repo.gitdir.join("HEAD");
    let data = fs::read_to_string(&head_path)
        .with_context(|| format!("Could not read HEAD at {}", head_path.display()))?;
    Ok(data.trim().to_string())
}

pub fn branch_get_active(repo: &GitRepository) -> Result<Option<String>> {
    let head = head_resolve(repo)?;
    if head.starts_with("ref: ") {
        let target = head[5..].to_string(); // strip "ref: "
        if target.starts_with("refs/heads/") {
            return Ok(Some(target["refs/heads/".len()..].to_string()));
        } else {
            return Ok(Some(target));
        }
    }
    Ok(None)
}

fn worktree_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_files(root, root, &mut files)?;
    Ok(files)
}

fn collect_files(base: &Path, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Skip .git directory
            if path.ends_with(".rust-git") {
                continue;
            }
            collect_files(base, &path, files)?;
        } else {
            files.push(path);
        }
    }
    Ok(())
}
