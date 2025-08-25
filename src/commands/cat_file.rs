use crate::git::{
    objects::{GitObjectType, object_find, object_read},
    repo::repo_find,
};
use anyhow::Result;
use std::{io::Write, str};

pub fn run(object_type: &GitObjectType, object: &str) -> Result<()> {
    let repo = repo_find(".", true)?.unwrap();

    let sha = object_find(&repo, object, Some(*object_type))?;

    let (_, obj) = object_read(&repo, &sha)?;

    let data = obj.serialize()?;
    std::io::stdout().write_all(&data)?;

    Ok(())
}
