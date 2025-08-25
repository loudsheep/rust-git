use anyhow::Result;

use crate::git::{index::read_index, repo::repo_find};

pub fn run() -> Result<()> {
    let repo = repo_find(".", true)?.unwrap();

    let index = read_index(&repo)?;

    for entry in &index {
        println!("{}", entry.path);
    }

    Ok(())
}
