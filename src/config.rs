//! Per-wiki configuration loaded from `.wiki/config.toml`.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Per-wiki configuration.
///
/// Stored at `<wiki_root>/.wiki/config.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WikiConfig {
    /// Absolute path to the wiki root directory.
    pub root: PathBuf,
    /// Human-readable name for this wiki (used in MCP resource URIs and `--wiki` targeting).
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wiki_config_from_toml() {
        let toml_str = r#"
            name = "my-wiki"
            root = "/tmp/wiki"
        "#;
        let config: WikiConfig = toml::from_str(toml_str).expect("parse config");
        assert_eq!(config.name, "my-wiki");
        assert_eq!(config.root, PathBuf::from("/tmp/wiki"));
    }
}
