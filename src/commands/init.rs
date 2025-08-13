use anyhow::Result;
use std::env;

use crate::git::repo::GitRepository;

pub fn run(path: Option<&str>) -> Result<()> {
    let repo_path = match path {
        Some(p) => p.into(),
        None => env::current_dir()?,
    };

    GitRepository::create(repo_path)?;
    println!("Initialized empty rust-git repository");

    Ok(())
}