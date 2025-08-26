use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::git::{
    ignore::{check_ignore, gitignore_read},
    repo::repo_find,
};

pub fn run(paths: &[PathBuf]) -> Result<()> {
    let repo = repo_find(".", true)?.unwrap();

    let rules = gitignore_read(&repo)?;
    for path in paths {
        let path_str = path.to_str().context("Invalid path encoding")?;

        if check_ignore(&rules, path_str)? {
            println!("{path_str}");
        }
    }

    Ok(())
}
