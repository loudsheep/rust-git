use std::path::PathBuf;

use crate::git::{
    objects::{object_write, GitBlob, GitObjectType},
    repo::repo_find,
};
use anyhow::Result;

pub fn run(write: bool, object_type: &GitObjectType, file: PathBuf) -> Result<()> {
    let repo = repo_find(".", true)?.unwrap();

    // For now, only support blob like WYAG’s early chapters
    match object_type {
        GitObjectType::blob => {},
        GitObjectType::commit => anyhow::bail!("Unsupported object type: {:?}", &object_type),
        GitObjectType::tree => anyhow::bail!("Unsupported object type: {:?}", &object_type),
        GitObjectType::tag => anyhow::bail!("Unsupported object type: {:?}", &object_type),
    }

    let data = std::fs::read(file)?;
    let blob = GitBlob { data };

    let sha = object_write(&repo, &blob, &object_type, write);
    println!("{}", sha?);

    Ok(())
}
