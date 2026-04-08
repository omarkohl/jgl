pub mod cli;
pub mod commands;
pub mod config;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use etcetera::{choose_app_strategy, AppStrategy, AppStrategyArgs};

use cli::{Cli, Command};

/// # Errors
/// Returns an error if argument parsing fails or the subcommand returns an error.
pub fn run() -> Result<()> {
    let cli = Cli::parse();

    if let Command::Completions { shell } = cli.command {
        generate(shell, &mut Cli::command(), "jgl", &mut std::io::stdout());
        return Ok(());
    }

    let config_path = config_path()?;

    match cli.command {
        Command::Completions { .. } => unreachable!(),
        Command::Add { path } => commands::add::run(&config_path, &path, &mut std::io::stdout()),
        Command::Fetch {
            verbose,
            rebase,
            with_conflicts,
        } => commands::fetch::run(
            &config_path,
            &commands::fetch::FetchOptions {
                verbose,
                rebase,
                with_conflicts,
            },
            &mut std::io::stdout(),
            &mut std::io::stderr(),
        ),
    }
}

fn config_path() -> Result<std::path::PathBuf> {
    let strategy = choose_app_strategy(AppStrategyArgs {
        top_level_domain: "io".into(),
        author: "jungle".into(),
        app_name: "jungle".into(),
    })
    .map_err(|e| anyhow::anyhow!("failed to determine config dir: {e}"))?;
    Ok(strategy.config_dir().join("config.toml"))
}
