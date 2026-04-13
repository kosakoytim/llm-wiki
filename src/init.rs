//! `wiki init` — bootstrap a new wiki repository.
//!
//! Exposed as a library function so that integration tests can call it
//! directly without spawning a subprocess.

use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::config::WikiConfig;
use crate::git;

/// Subdirectories that every wiki must contain.
const WIKI_DIRS: &[&str] = &[
    "concepts",
    "sources",
    "contradictions",
    "queries",
    "raw",
    ".wiki",
];

/// Initialise (or repair) a wiki at `root`.
///
/// - Ensures `root` exists (creates it if needed).
/// - Runs `git init` unless `.git/` already exists.
/// - Creates all required subdirectories if missing.
/// - Writes `.wiki/config.toml` if it does not already exist.
///
/// Idempotent — safe to call multiple times.
pub fn init_wiki(root: &Path) -> Result<()> {
    // 1. Ensure the target directory exists.
    fs::create_dir_all(root)?;

    // 2. git init (skipped if .git/ already exists).
    git::init_if_needed(root)?;

    // 3. Create required subdirectories.
    for dir in WIKI_DIRS {
        fs::create_dir_all(root.join(dir))?;
    }

    // 4. Write .wiki/config.toml if absent.
    let config_path = root.join(".wiki").join("config.toml");
    if !config_path.exists() {
        let name = root
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "wiki".to_string());
        let config = WikiConfig {
            root: root.to_path_buf(),
            name,
        };
        let toml = toml::to_string_pretty(&config)?;
        fs::write(&config_path, toml)?;
    }

    Ok(())
}

/// MCP config snippet for `~/.claude/settings.json`, given the wiki root path.
pub fn mcp_config_snippet(root: &Path) -> String {
    format!(
        r#"{{
  "mcpServers": {{
    "wiki": {{
      "command": "wiki",
      "args": ["serve"],
      "cwd": "{}"
    }}
  }}
}}"#,
        root.display()
    )
}
