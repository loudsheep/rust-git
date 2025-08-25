use std::fs;

use anyhow::{Context, Result, bail};

use crate::git::repo::{repo_find};

pub fn list_tags() -> Result<()> {
    let repo = repo_find(".", true)?.unwrap();

    let tags_dir = repo.gitdir.join("refs").join("tags");
    if !tags_dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(&tags_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            println!("{}", entry.file_name().to_string_lossy());
        }
    }

    Ok(())
}

pub fn create_tag(name: &str, sha: &str) -> Result<()> {
    let repo = repo_find(".", true)?.unwrap();

    if name.contains('/') {
        bail!("Tag name cannot contain '/'");
    }

    let tags_dir = repo.gitdir.join("refs").join("tags");
    fs::create_dir_all(&tags_dir)?;

    let tag_path = tags_dir.join(name);
    if tag_path.exists() {
        bail!("Tag '{name}' already exists");
    }

    fs::write(&tag_path, format!("{sha}\n"))
        .with_context(|| format!("Failed to write tag file {:?}", tag_path))?;

    Ok(())
}
