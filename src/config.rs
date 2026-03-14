use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// A single command entry stored in config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEntry {
    pub description: String,
    pub command: String,
}

/// Root config structure mapping to `.be/config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub commands: BTreeMap<String, CommandEntry>,
}

/// Reserved subcommand names that cannot be used as tags.
const RESERVED_NAMES: &[&str] = &["init", "manager", "list", "add", "remove", "edit", "help"];

impl Config {
    /// Walk from `start_dir` upward to find `.be/config.toml`.
    /// Returns the path to the config file, or None if not found.
    pub fn find_config_path_from(start_dir: &Path) -> Option<PathBuf> {
        let mut dir = start_dir.to_path_buf();
        loop {
            let candidate = dir.join(".be").join("config.toml");
            if candidate.is_file() {
                return Some(candidate);
            }
            if !dir.pop() {
                return None;
            }
        }
    }

    /// Find config from the current working directory.
    pub fn find_config_path() -> Option<PathBuf> {
        let cwd = std::env::current_dir().ok()?;
        Self::find_config_path_from(&cwd)
    }

    /// Load config from auto-detected path.
    pub fn load() -> Result<(Self, PathBuf)> {
        let path = Self::find_config_path()
            .context("No .be/config.toml found. Run `be init` first.")?;
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        Ok((config, path))
    }

    /// Save config to the given path.
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write {}", path.display()))?;
        Ok(())
    }

    /// Initialize a new `.be/config.toml` in the current directory.
    pub fn init() -> Result<PathBuf> {
        let cwd = std::env::current_dir().context("Cannot determine current directory")?;
        let be_dir = cwd.join(".be");
        let config_path = be_dir.join("config.toml");

        if config_path.is_file() {
            bail!(".be/config.toml already exists in {}", cwd.display());
        }

        std::fs::create_dir_all(&be_dir)
            .with_context(|| format!("Failed to create {}", be_dir.display()))?;

        let config = Config::default();
        config.save(&config_path)?;
        Ok(config_path)
    }

    /// Validate a tag name: only `[a-zA-Z0-9_-]` allowed.
    pub fn validate_tag(tag: &str) -> Result<()> {
        if tag.is_empty() {
            bail!("Tag name cannot be empty");
        }
        if !tag.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            bail!("Tag '{}' contains invalid characters. Only [a-zA-Z0-9_-] allowed.", tag);
        }
        if RESERVED_NAMES.contains(&tag) {
            bail!("Tag '{}' conflicts with a built-in subcommand.", tag);
        }
        Ok(())
    }

    /// Add a command entry.
    pub fn add(&mut self, tag: &str, command: &str, description: &str) -> Result<()> {
        Self::validate_tag(tag)?;
        if self.commands.contains_key(tag) {
            bail!("Tag '{}' already exists. Use `be edit` to modify it.", tag);
        }
        self.commands.insert(
            tag.to_string(),
            CommandEntry {
                description: description.to_string(),
                command: command.to_string(),
            },
        );
        Ok(())
    }

    /// Remove a command entry.
    pub fn remove(&mut self, tag: &str) -> Result<CommandEntry> {
        self.commands
            .remove(tag)
            .with_context(|| format!("Tag '{}' not found.", tag))
    }

    /// Edit an existing command entry.
    pub fn edit(&mut self, tag: &str, command: Option<&str>, description: Option<&str>) -> Result<()> {
        let entry = self
            .commands
            .get_mut(tag)
            .with_context(|| format!("Tag '{}' not found.", tag))?;

        if let Some(cmd) = command {
            entry.command = cmd.to_string();
        }
        if let Some(desc) = description {
            entry.description = desc.to_string();
        }
        Ok(())
    }

    /// Get a command entry by tag.
    pub fn get(&self, tag: &str) -> Option<&CommandEntry> {
        self.commands.get(tag)
    }
}
