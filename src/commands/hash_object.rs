use std::path::PathBuf;

use crate::git::{
    objects::{GitBlob, hash_object},
    repo::repo_find,
};
use anyhow::Result;

pub fn run(write: bool, object_type: &str, file: PathBuf) -> Result<()> {
    let repo = repo_find(".", true)?.unwrap();

    // For now, only support blob like WYAG’s early chapters
    if object_type != "blob" {
        anyhow::bail!("Unsupported object type: {}", object_type);
    }

    let data = std::fs::read(file)?;
    let blob = GitBlob { data };

    let sha = hash_object(&repo, &blob, &object_type, write);
    println!("{}", sha?);

    Ok(())
}
