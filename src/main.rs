use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::git::objects::GitObjectType;

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
    /// Initialize a repository
    Init { path: Option<PathBuf> },
    /// Compute object ID and optionally creates a blob from a file
    HashObject {
        /// Actually write the object into the database
        #[arg(short, long)]
        write: bool,

        /// Specify the type
        #[arg(value_enum, short, long, default_value = "blob")]
        type_name: GitObjectType,

        /// Read object from this file
        file: PathBuf,
    },
    /// Provide content of repository object
    CatFile {
        /// Specify the type
        #[arg(value_enum)]
        object_type: GitObjectType,

        /// The object to display
        object: String,
    },
    /// Display history of a given commit.
    Log {
        /// Commit to start at.
        #[arg(default_value = "HEAD")]
        commit: String,
    },
}

fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Init { path } => {
            commands::init::run(path)?;
        }
        Commands::HashObject {
            write,
            type_name,
            file,
        } => {
            commands::hash_object::run(write, &type_name, file)?;
        }
        Commands::CatFile {
            object_type,
            object,
        } => {
            commands::cat_file::run(&object_type, &object)?;
        }
        Commands::Log { commit } => {
            commands::log::run(&commit)?;
        }
    }

    Ok(())
}
