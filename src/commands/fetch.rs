use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::config::Config;

/// Abstraction over process spawning so tests can inject a fake runner.
pub trait CommandRunner {
    /// # Errors
    /// Returns an error if the command fails or cannot be spawned.
    fn run_jj_fetch(&self, dir: &Path) -> Result<()>;
}

pub struct ProcessRunner;

impl CommandRunner for ProcessRunner {
    fn run_jj_fetch(&self, dir: &Path) -> Result<()> {
        let status = std::process::Command::new("jj")
            .args(["git", "fetch"])
            .current_dir(dir)
            .status()
            .map_err(|e| anyhow::anyhow!("failed to spawn jj: {e}"))?;

        if status.success() {
            Ok(())
        } else {
            anyhow::bail!("jj git fetch failed in {}", dir.display())
        }
    }
}

#[derive(Debug)]
pub struct FetchResult {
    pub path: PathBuf,
    pub success: bool,
    pub error: Option<String>,
}

/// # Errors
/// Returns an error if the config cannot be loaded or any fetch fails.
pub fn run_with(config_path: &Path, runner: &dyn CommandRunner) -> Result<()> {
    let config = Config::load_or_default(config_path)?;

    if config.repos.is_empty() {
        println!("No repositories registered. Use `jgl add <path>` to add one.");
        return Ok(());
    }

    let mut results: Vec<FetchResult> = Vec::new();

    for repo in &config.repos {
        let dir = Config::resolve_path(&repo.path)?;
        let (success, error) = match runner.run_jj_fetch(&dir) {
            Ok(()) => (true, None),
            Err(e) => (false, Some(e.to_string())),
        };
        results.push(FetchResult {
            path: dir,
            success,
            error,
        });
    }

    for result in &results {
        if result.success {
            println!("  ok  {}", result.path.display());
        } else {
            eprintln!(
                "  err {}: {}",
                result.path.display(),
                result.error.as_deref().unwrap_or("unknown error")
            );
        }
    }

    let failures = results.iter().filter(|r| !r.success).count();
    if failures > 0 {
        anyhow::bail!("{failures} fetch(es) failed");
    }

    Ok(())
}

/// # Errors
/// Returns an error if the config cannot be loaded or any fetch fails.
pub fn run(config_path: &Path) -> Result<()> {
    run_with(config_path, &ProcessRunner)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::config::{Config, Repo};
    use tempfile::TempDir;

    struct FakeRunner {
        fail_paths: Vec<PathBuf>,
    }

    impl CommandRunner for FakeRunner {
        fn run_jj_fetch(&self, dir: &Path) -> Result<()> {
            if self.fail_paths.iter().any(|p| p == dir) {
                anyhow::bail!("simulated failure");
            }
            Ok(())
        }
    }

    fn write_config(path: &Path, repos: &[&str]) {
        let config = Config {
            repos: repos
                .iter()
                .map(|p| Repo {
                    path: (*p).to_owned(),
                })
                .collect(),
        };
        config.save(path).unwrap();
    }

    #[test]
    fn empty_config_prints_hint() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");
        // No config file — should succeed with hint
        let runner = FakeRunner { fail_paths: vec![] };
        run_with(&config_path, &runner).unwrap();
    }

    #[test]
    fn fetch_calls_runner_per_repo() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");
        let repo_a = tmp.path().join("repo_a");
        let repo_b = tmp.path().join("repo_b");
        std::fs::create_dir_all(&repo_a).unwrap();
        std::fs::create_dir_all(&repo_b).unwrap();

        write_config(
            &config_path,
            &[repo_a.to_str().unwrap(), repo_b.to_str().unwrap()],
        );

        let runner = FakeRunner { fail_paths: vec![] };
        run_with(&config_path, &runner).unwrap();
    }

    #[test]
    fn fetch_reports_failure_per_repo() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");
        let repo_a = tmp.path().join("repo_a");
        std::fs::create_dir_all(&repo_a).unwrap();

        write_config(&config_path, &[repo_a.to_str().unwrap()]);

        let runner = FakeRunner {
            fail_paths: vec![repo_a],
        };
        let err = run_with(&config_path, &runner).unwrap_err();
        assert!(err.to_string().contains("failed"));
    }
}
