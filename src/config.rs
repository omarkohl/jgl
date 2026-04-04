use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    #[serde(default)]
    pub repos: Vec<Repo>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Repo {
    pub path: String,
}

impl Config {
    /// # Errors
    /// Returns an error if the file cannot be read or is not valid TOML.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read config at {}", path.display()))?;
        toml::from_str(&content)
            .with_context(|| format!("failed to parse config at {}", path.display()))
    }

    /// # Errors
    /// Returns an error if the config directory cannot be created or the file cannot be written.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create config dir {}", parent.display()))?;
        }
        let content = toml::to_string_pretty(self).context("failed to serialize config")?;
        std::fs::write(path, content)
            .with_context(|| format!("failed to write config to {}", path.display()))
    }

    /// # Errors
    /// Returns an error if the file exists but cannot be read or parsed.
    pub fn load_or_default(path: &Path) -> Result<Self> {
        if path.exists() {
            Self::load(path)
        } else {
            Ok(Self::default())
        }
    }

    /// Expand `~` to the home directory.
    ///
    /// # Errors
    /// Returns an error if a `~`-prefixed path is given and `HOME` is not set.
    pub fn resolve_path(path: &str) -> Result<PathBuf> {
        if let Some(rest) = path.strip_prefix("~/") {
            let home = home_dir().context("could not determine home directory")?;
            Ok(home.join(rest))
        } else if path == "~" {
            home_dir().context("could not determine home directory")
        } else {
            Ok(PathBuf::from(path))
        }
    }

    /// Add a repo path. Validates existence, `.jj` presence, and no duplicates.
    ///
    /// # Errors
    /// Returns an error if the path does not exist, is not a jj repository, or is already registered.
    pub fn add_repo(&mut self, path: &str) -> Result<()> {
        let resolved = Self::resolve_path(path)?;

        if !resolved.exists() {
            anyhow::bail!("path does not exist: {}", resolved.display());
        }

        if !resolved.join(".jj").exists() {
            anyhow::bail!(
                "path is not a jj repository (no .jj directory): {}",
                resolved.display()
            );
        }

        // Duplicate check: compare resolved forms
        for existing in &self.repos {
            let existing_resolved = Self::resolve_path(&existing.path)?;
            if existing_resolved == resolved {
                anyhow::bail!("repository already registered: {path}");
            }
        }

        self.repos.push(Repo {
            path: path.to_owned(),
        });
        Ok(())
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
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
    fn deserialize_config() {
        let toml = r#"
[[repos]]
path = "~/projects/foo"

[[repos]]
path = "/home/user/bar"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.repos.len(), 2);
        assert_eq!(config.repos[0].path, "~/projects/foo");
        assert_eq!(config.repos[1].path, "/home/user/bar");
    }

    #[test]
    fn serialize_config() {
        let config = Config {
            repos: vec![Repo {
                path: "~/projects/foo".into(),
            }],
        };
        let output = toml::to_string_pretty(&config).unwrap();
        assert!(output.contains("~/projects/foo"));
    }

    #[test]
    fn round_trip_load_save() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("config.toml");
        let config = Config {
            repos: vec![Repo {
                path: "~/projects/foo".into(),
            }],
        };
        config.save(&path).unwrap();
        let loaded = Config::load(&path).unwrap();
        assert_eq!(config, loaded);
    }

    #[test]
    fn resolve_path_expands_tilde() {
        let home = std::env::var("HOME").unwrap();
        let resolved = Config::resolve_path("~/projects/foo").unwrap();
        assert_eq!(resolved, PathBuf::from(format!("{home}/projects/foo")));
    }

    #[test]
    fn resolve_path_bare_tilde() {
        let home = std::env::var("HOME").unwrap();
        let resolved = Config::resolve_path("~").unwrap();
        assert_eq!(resolved, PathBuf::from(&home));
    }

    #[test]
    fn resolve_path_absolute() {
        let resolved = Config::resolve_path("/abs/path").unwrap();
        assert_eq!(resolved, PathBuf::from("/abs/path"));
    }

    #[test]
    fn add_repo_duplicate_rejected() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path().join("repo");
        make_jj_repo(&repo);

        let mut config = Config::default();
        config.add_repo(repo.to_str().unwrap()).unwrap();

        let err = config.add_repo(repo.to_str().unwrap()).unwrap_err();
        assert!(err.to_string().contains("already registered"));
    }

    #[test]
    fn add_repo_nonexistent_rejected() {
        let mut config = Config::default();
        let err = config.add_repo("/nonexistent/path/abc123").unwrap_err();
        assert!(err.to_string().contains("does not exist"));
    }

    #[test]
    fn add_repo_no_jj_rejected() {
        let tmp = TempDir::new().unwrap();
        let mut config = Config::default();
        let err = config.add_repo(tmp.path().to_str().unwrap()).unwrap_err();
        assert!(err.to_string().contains("not a jj repository"));
    }

    #[test]
    fn add_repo_success() {
        let tmp = TempDir::new().unwrap();
        let repo = tmp.path().join("repo");
        make_jj_repo(&repo);

        let mut config = Config::default();
        config.add_repo(repo.to_str().unwrap()).unwrap();
        assert_eq!(config.repos.len(), 1);
    }
}
