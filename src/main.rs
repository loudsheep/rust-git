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
    /// Pretty-print a tree object.
    LsTree {
        /// Recurse into sub-trees
        #[arg(short)]
        recursive: bool,

        /// A tree-ish object.
        tree: String,
    },
    /// Checkout a commit inside of a directory.
    Checkout {
        /// The commit or tree to checkout.
        commit: String,
        // TODO
        // The EMPTY directory to checkout on.
        // path: PathBuf,
    },
    /// Parse revision (or other objects) identifiers
    RevParse {
        /// Specify the expected type
        #[arg(long, value_enum)]
        git_type: Option<GitObjectType>,

        /// The name to parse
        name: String,
    },
    /// List references.
    ShowRef {},
    /// List and create tags
    Tag {
        /// Whether to create a tag object
        #[arg(short, long)]
        annotate: bool,

        /// The new tag's name  
        name: Option<String>,

        /// The object the new tag will point to
        #[arg(default_value = "HEAD")]
        object: Option<String>,
    },
    /// List all the stage files
    LsFiles {},
    /// Check path(s) against ignore rules.
    CheckIgnore {
        /// Paths to check
        paths: Vec<PathBuf>,
    },
    /// Show the working tree status.
    Status {},
    /// Remove files from the working tree and the index.
    Rm {
        /// Files to remove
        paths: Vec<PathBuf>,
    },
    /// Add files contents to the index.
    Add {
        /// Files to add
        paths: Vec<PathBuf>,
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
        Commands::LsTree { recursive, tree } => {
            commands::ls_tree::run(&tree, recursive)?;
        }
        Commands::Checkout { commit } => {
            commands::checkout::run(&commit)?;
        }
        Commands::RevParse { name, git_type } => {
            commands::rev_parse::run(&name, git_type)?;
        }
        Commands::ShowRef {} => {
            commands::show_ref::run()?;
        }
        Commands::Tag {
            annotate,
            name,
            object,
        } => {
            if let Some(name) = name {
                let target = object.unwrap_or_else(|| "HEAD".to_string());
                commands::tag::create_tag(&name, &target, annotate)?;
            } else {
                commands::tag::list_tags()?;
            }
        }
        Commands::LsFiles {} => {
            commands::ls_files::run()?;
        }
        Commands::CheckIgnore { paths } => {
            commands::check_ignore::run(&paths)?;
        }
        Commands::Status {} => {
            commands::status::run()?;
        }
        Commands::Rm { paths } => {
            commands::rm::run(&paths)?;
        },
        Commands::Add { paths } => todo!(),
    }

    Ok(())
}
