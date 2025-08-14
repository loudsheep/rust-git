use crate::git::{objects::cat_file, repo::repo_find};
use anyhow::Result;
use std::{io::Write, str};

pub fn run(object_type: &str, object: &str) -> Result<()> {
    let repo = repo_find(".", true)?.unwrap();
    let obj = cat_file(&repo, object)?;

    
    // let data = obj.serialize()?;
    // std::io::stdout().write_all(&data)?;
    Ok(())
}
