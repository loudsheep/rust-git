use anyhow::Result;
use std::{env, path::PathBuf};

use crate::git::repo::GitRepository;

pub fn run(path: Option<PathBuf>) -> Result<()> {
    let repo_path = match path {
        Some(p) => p.into(),
        None => env::current_dir()?,
    };

    GitRepository::create(repo_path)?;
    println!("Initialized empty rust-git repository");

    Ok(())
}