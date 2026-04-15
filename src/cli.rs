use clap::{Parser, Subcommand};
use clap_complete::Shell;

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
    /// Generate shell completions
    Completions { shell: Shell },
    /// Run `jj git fetch` in all registered repositories
    Fetch {
        /// Show full jj output for each repository
        #[arg(short, long)]
        verbose: bool,
        /// Rebase working-copy branch onto `trunk()` after each fetch (overrides config)
        #[arg(long, overrides_with = "no_rebase")]
        rebase: bool,
        /// Do not rebase after fetch (overrides config)
        #[arg(long, overrides_with = "rebase")]
        no_rebase: bool,
        /// Keep rebase even if it introduces conflicts (overrides config)
        #[arg(long, overrides_with = "no_with_conflicts")]
        with_conflicts: bool,
        /// Undo rebase if it introduces conflicts (overrides config)
        #[arg(long, overrides_with = "with_conflicts")]
        without_conflicts: bool,
        /// Idle timeout in seconds: kill `jj git fetch` if it produces no output
        /// for this long. 0 disables the timeout. (overrides config)
        #[arg(long)]
        idle_timeout: Option<u64>,
    },
}
