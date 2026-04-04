use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "jgl", version, about = "Multi-repo manager for jujutsu (jj)")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Add a repository to the config
    Add {
        /// Path to the jj repository
        path: String,
    },
    /// Run `jj git fetch` in all registered repositories
    Fetch,
}
