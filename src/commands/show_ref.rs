use anyhow::Result;

use crate::git::{refs::collect_refs, repo::repo_find};

pub fn run() -> Result<()> {
    let repo = repo_find(".", true)?.unwrap();

    let refs_dir = repo.gitdir.join("refs");
    if !refs_dir.exists() {
        return Ok(()); // no refs yet
    }

    let refs = collect_refs(&refs_dir, "refs")?;
    for (sha, name) in refs {
        println!("{sha} {name}");
    }

    Ok(())
}
