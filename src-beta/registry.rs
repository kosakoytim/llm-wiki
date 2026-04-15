//! Multi-wiki registry — loads `~/.wiki/config.toml` and resolves wiki names
//! to their root paths.
//!
//! Implemented in Phase 6.

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::config::WikiConfig;

/// One entry in the global `~/.wiki/config.toml`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WikiEntry {
    /// Human-readable name for this wiki (used in `--wiki` targeting and MCP URIs).
    pub name: String,
    /// Absolute path to the wiki root directory.
    pub path: PathBuf,
    /// Whether this is the default wiki.  Resolved when `--wiki` is omitted.
    #[serde(default)]
    pub default: bool,
    /// Optional remote git URL (for documentation / future sync use).
    pub remote: Option<String>,
}

/// Intermediate TOML structure for the global `~/.wiki/config.toml`.
#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct GlobalConfig {
    #[serde(default)]
    pub(crate) wikis: Vec<WikiEntry>,
}

/// Registry of all wikis registered in `~/.wiki/config.toml`.
///
/// Used by `--wiki <name>` and `wiki search --all`.
#[derive(Debug)]
pub struct WikiRegistry {
    entries: Vec<WikiEntry>,
    /// Pre-built [`WikiConfig`] for each entry (parallel index to `entries`).
    configs: Vec<WikiConfig>,
}

impl WikiRegistry {
    /// Load the global config file at `config_path` and validate all entries.
    ///
    /// # Errors
    ///
    /// - File cannot be read or parsed as TOML.
    /// - More than one entry has `default = true`.
    /// - An entry's `path` does not exist on the filesystem.
    pub fn load(config_path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(config_path)
            .with_context(|| format!("failed to read {}", config_path.display()))?;

        let global: GlobalConfig = toml::from_str(&content)
            .with_context(|| format!("failed to parse {}", config_path.display()))?;

        // Validate: at most one wiki may be marked default.
        let defaults: Vec<&str> = global
            .wikis
            .iter()
            .filter(|e| e.default)
            .map(|e| e.name.as_str())
            .collect();
        if defaults.len() > 1 {
            bail!(
                "multiple wikis are marked default = true: {}; \
                 only one wiki may be the default",
                defaults.join(", ")
            );
        }

        // Validate: every path must exist.
        for entry in &global.wikis {
            if !entry.path.exists() {
                bail!(
                    "wiki '{}': path '{}' does not exist",
                    entry.name,
                    entry.path.display()
                );
            }
        }

        let configs: Vec<WikiConfig> = global
            .wikis
            .iter()
            .map(|e| WikiConfig {
                root: e.path.clone(),
                name: e.name.clone(),
            })
            .collect();

        Ok(Self {
            entries: global.wikis,
            configs,
        })
    }

    /// Resolve a wiki by optional name.
    ///
    /// - `None` → return the entry with `default = true`.  Errors if none is default.
    /// - `Some(name)` → find by name.  Errors with a list of available names if not found.
    pub fn resolve(&self, name: Option<&str>) -> Result<&WikiConfig> {
        match name {
            None => {
                let idx = self
                    .entries
                    .iter()
                    .position(|e| e.default)
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "no default wiki configured; use --wiki <name> or mark one \
                             wiki with `default = true` in ~/.wiki/config.toml"
                        )
                    })?;
                Ok(&self.configs[idx])
            }
            Some(n) => {
                let idx = self.entries.iter().position(|e| e.name == n).ok_or_else(|| {
                    let available: Vec<&str> =
                        self.entries.iter().map(|e| e.name.as_str()).collect();
                    anyhow::anyhow!(
                        "wiki '{}' not found; available: {}",
                        n,
                        available.join(", ")
                    )
                })?;
                Ok(&self.configs[idx])
            }
        }
    }

    /// Iterate over all registered wiki entries (used by `search_all`).
    pub fn entries(&self) -> &[WikiEntry] {
        &self.entries
    }

    /// Number of registered wikis.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the registry contains no wikis.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Return the path to the global `~/.wiki/config.toml`.
///
/// Uses the `HOME` environment variable on Unix; falls back to `.` if unset.
pub fn global_config_path() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".wiki").join("config.toml")
}

/// Append a new wiki entry to `config_path` (creates the file and parent dirs
/// if they do not yet exist).
///
/// The first wiki added is automatically marked `default = true`.
pub fn register_wiki(
    wiki_name: &str,
    wiki_path: &Path,
    config_path: &Path,
) -> Result<()> {
    let mut global = if config_path.exists() {
        let content = std::fs::read_to_string(config_path)
            .with_context(|| format!("failed to read {}", config_path.display()))?;
        toml::from_str::<GlobalConfig>(&content)
            .with_context(|| format!("failed to parse {}", config_path.display()))?
    } else {
        GlobalConfig::default()
    };

    // First wiki becomes the default.
    let is_default = global.wikis.is_empty();

    global.wikis.push(WikiEntry {
        name: wiki_name.to_string(),
        path: wiki_path.to_path_buf(),
        default: is_default,
        remote: None,
    });

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let toml_str = toml::to_string_pretty(&global)
        .context("failed to serialise global config")?;
    std::fs::write(config_path, toml_str)
        .with_context(|| format!("failed to write {}", config_path.display()))?;

    Ok(())
}
