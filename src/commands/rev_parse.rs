use anyhow::Result;

use crate::git::{refs::ref_parse, repo::repo_find};

pub fn run(name: &str) -> Result<()> {
    let repo = repo_find(".", true)?.unwrap();

    let sha: String = ref_parse(&repo, name)?;
    println!("{sha}");

    Ok(())
}
