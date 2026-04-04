use std::path::Path;

use anyhow::Result;

use crate::config::Config;

/// # Errors
/// Returns an error if the config cannot be loaded, the path is invalid, or the config cannot be saved.
pub fn run(config_path: &Path, path: &str) -> Result<()> {
    let mut config = Config::load_or_default(config_path)?;
    config.add_repo(path)?;
    config.save(config_path)?;
    println!("Added {path}");
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_jj_repo(dir: &Path) {
        std::fs::create_dir_all(dir.join(".jj")).unwrap();
    }

    #[test]
    fn add_updates_config() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");
        let repo = tmp.path().join("repo");
        make_jj_repo(&repo);

        run(&config_path, repo.to_str().unwrap()).unwrap();

        let config = Config::load(&config_path).unwrap();
        assert_eq!(config.repos.len(), 1);
        assert_eq!(config.repos[0].path, repo.to_str().unwrap());
    }

    #[test]
    fn add_nonexistent_fails() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");
        let err = run(&config_path, "/nonexistent/path/xyz").unwrap_err();
        assert!(err.to_string().contains("does not exist"));
    }
}
