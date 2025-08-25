use anyhow::Result;

use crate::git::{objects::{object_find, GitObjectType}, repo::repo_find};

pub fn run(name: &str, fmt: Option<GitObjectType>) -> Result<()> {
    let repo = repo_find(".", true)?.unwrap();

    let sha = object_find(&repo, name, fmt)?;
    println!("{sha}");

    Ok(())
}
