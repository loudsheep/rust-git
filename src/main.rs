use std::path::PathBuf;

use anyhow::{Result};
use clap::{Parser, Subcommand};

use crate::commands::init::run;
mod commands;
mod git;

#[derive(Debug, Parser)]
#[command(name = "rust-git")]
#[command(about = "A simple git vcs, but in rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init { path: Option<PathBuf> },
}

fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Init { path } => {
            run(path)?;
        }
    }

    Ok(())
}
