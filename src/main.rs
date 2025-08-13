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
    Init {},
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Init {} => {
            println!("Initializing repository")
        }
    }
}
