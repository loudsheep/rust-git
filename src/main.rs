use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

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
    Init {
        path: Option<PathBuf>,
    },

    HashObject {
        #[arg(short, long)]
        write: bool,

        #[arg(long, default_value = "blob")]
        type_name: String,

        file: PathBuf,
    },

    CatFile {
        object_type: String,

        object: String,
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
            commands::hash_object::run(write, &type_name, file);
        }
        Commands::CatFile {
            object_type,
            object,
        } => {
            commands::cat_file::run(&object_type, &object);
        }
    }

    Ok(())
}
