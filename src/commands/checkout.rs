use std::{fs, io::Write, path::Path};

use anyhow::{Context, Result, bail};

use crate::git::{
    objects::{GitBlob, GitCommit, GitObjectType, object_find, object_read},
    repo::{GitRepository, repo_find},
    tree::GitTree,
};

fn checkout_tree(repo: &GitRepository, sha: &str, path: &Path) -> Result<()> {
    let (otype, obj) = object_read(repo, sha)?;
    if otype != GitObjectType::Tree {
        bail!("Object {sha} is not a tree");
    }

    let tree = obj
        .as_any()
        .downcast_ref::<GitTree>()
        .context("Failed to downcast to GitTree")?;

    fs::create_dir_all(path)?;

    for entry in &tree.entries {
        let entry_sha = hex::encode(entry.sha);
        let entry_path = path.join(&entry.path);

        match entry.mode.as_str() {
            m if m.starts_with("04") => {
                checkout_tree(&repo, &entry_sha, &entry_path)?;
            }
            m if m.starts_with("10") || m.starts_with("12") => {
                let (_, obj) = object_read(repo, &entry_sha)?;

                if otype != GitObjectType::Blob {
                    bail!("Tree entry {} is not a blob", entry.path);
                }
                let blob = obj
                    .as_any()
                    .downcast_ref::<GitBlob>()
                    .context("Failed to downcast to GitBlob")?;

                let mut file = fs::File::create(&entry_path)?;
                file.write_all(&blob.data)?;
            }
            m if m.starts_with("16") => {
                // Submodule = commit object (store SHA as a file placeholder for now)
                let mut file = fs::File::create(&entry_path)?;
                file.write_all(entry_sha.as_bytes())?;
            }
            other => bail!("Weird tree entry mode {}", other),
        }
    }

    Ok(())
}

pub fn run(commit: &str) -> Result<()> {
    let repo = repo_find(".", true)?.unwrap();

    let sha = object_find(&repo, commit, Some(GitObjectType::Blob))?;
    let (obj_type, obj) = object_read(&repo, &sha)?;

    if obj_type != GitObjectType::Commit {
        bail!("Object {sha} is not a commit");
    }

    let commit = obj
        .as_any()
        .downcast_ref::<GitCommit>()
        .context("Failed to downcast to GitCommit")?;

    let tree_sha = commit.kvlm.get(b"tree").context("Missing 'tree' field")?;
    let tree_sha = std::str::from_utf8(tree_sha)?.to_string();

    for entry in fs::read_dir(&repo.worktree)? {
        let entry = entry?;

        if entry.file_name() == ".rust-git" {
            continue;
        }

        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }

    checkout_tree(&repo, &tree_sha, Path::new(&repo.worktree))?;

    Ok(())
}
