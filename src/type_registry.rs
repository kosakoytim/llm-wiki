use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use anyhow::{bail, Result};
use jsonschema::Validator;
use serde_yaml::Value;

use crate::config;
use crate::default_schemas;

/// A compiled type entry in the registry.
struct RegisteredType {
    schema_path: String,
    description: String,
    validator: Validator,
    aliases: HashMap<String, String>,
}

/// Per-wiki type registry — discovers types from `schemas/*.json` via
/// `x-wiki-types`, with optional `[types.*]` overrides from `wiki.toml`.
pub struct SpaceTypeRegistry {
    types: HashMap<String, RegisteredType>,
    schema_hash: String,
    type_hashes: HashMap<String, String>,
}

impl SpaceTypeRegistry {
    /// Build from a wiki repository root. Scans `schemas/*.json`, merges
    /// `wiki.toml` overrides.
    pub fn build(repo_root: &Path) -> Result<Self> {
        let schemas_dir = repo_root.join("schemas");
        let mut types = HashMap::new();

        if schemas_dir.is_dir() {
            discover_from_dir(&schemas_dir, &mut types)?;
        } else {
            discover_from_embedded(&mut types)?;
        }

        // Apply wiki.toml overrides
        let wiki_cfg = config::load_wiki(repo_root)?;
        for (type_name, entry) in &wiki_cfg.types {
            let schema_path = repo_root.join(&entry.schema);
            let content = std::fs::read_to_string(&schema_path)?;
            let registered = compile_schema(&entry.schema, &entry.description, &content)?;
            types.insert(type_name.clone(), registered);
        }

        let (schema_hash, type_hashes) = compute_hashes(&types);

        Ok(Self {
            types,
            schema_hash,
            type_hashes,
        })
    }

    /// Build from embedded default schemas only (no disk access).
    /// Used when no wiki is mounted or for backward compatibility.
    pub fn from_embedded() -> Self {
        let mut types = HashMap::new();
        discover_from_embedded(&mut types).expect("embedded schemas are valid");
        let (schema_hash, type_hashes) = compute_hashes(&types);
        Self {
            types,
            schema_hash,
            type_hashes,
        }
    }

    pub fn is_known(&self, type_name: &str) -> bool {
        self.types.contains_key(type_name)
    }

    /// List all registered type names with descriptions.
    pub fn list_types(&self) -> Vec<(&str, &str)> {
        let mut out: Vec<_> = self
            .types
            .iter()
            .map(|(name, rt)| (name.as_str(), rt.description.as_str()))
            .collect();
        out.sort_by_key(|(name, _)| *name);
        out
    }

    /// Get the aliases for a type (source field → canonical field).
    pub fn aliases(&self, type_name: &str) -> Option<&HashMap<String, String>> {
        self.types.get(type_name).map(|rt| &rt.aliases)
    }

    pub fn schema_hash(&self) -> &str {
        &self.schema_hash
    }

    pub fn type_hashes(&self) -> &HashMap<String, String> {
        &self.type_hashes
    }

    /// Validate frontmatter against the type's JSON Schema.
    ///
    /// - Resolves the page type (falls back to "default")
    /// - Validates against the compiled schema
    /// - In loose mode, unknown types produce warnings
    /// - In strict mode, unknown types produce errors
    ///
    /// Returns a list of warnings. Bails on hard errors.
    pub fn validate(&self, fm: &BTreeMap<String, Value>, strictness: &str) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // title is always required — hard error regardless of strictness
        let has_title = fm
            .get("title")
            .and_then(|v| v.as_str())
            .map(|s| !s.is_empty())
            .unwrap_or(false);
        // For skill pages, check "name" as alias for title
        let has_name = fm
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| !s.is_empty())
            .unwrap_or(false);
        if !has_title && !has_name {
            bail!("title is required");
        }

        let page_type = fm.get("type").and_then(|v| v.as_str()).unwrap_or("");

        // Determine which registered type to use
        let resolved_type = if page_type.is_empty() {
            warnings.push("missing field: type (defaulting to \"page\")".into());
            "default"
        } else if self.types.contains_key(page_type) {
            page_type
        } else {
            if strictness == "strict" {
                bail!("unknown type '{page_type}'");
            }
            warnings.push(format!("unknown type '{page_type}'"));
            "default"
        };

        if let Some(rt) = self.types.get(resolved_type) {
            let json_fm = yaml_fm_to_json(fm)?;
            let errors: Vec<_> = rt.validator.iter_errors(&json_fm).collect();
            if !errors.is_empty() {
                if strictness == "strict" {
                    bail!("schema validation failed: {}", errors[0]);
                }
                for e in &errors {
                    warnings.push(format!("schema validation: {e}"));
                }
            }
        }

        Ok(warnings)
    }
}

impl Default for SpaceTypeRegistry {
    fn default() -> Self {
        Self::from_embedded()
    }
}

// Keep backward-compatible alias
pub type TypeRegistry = SpaceTypeRegistry;

// ── Discovery ─────────────────────────────────────────────────────────────────

fn discover_from_dir(schemas_dir: &Path, types: &mut HashMap<String, RegisteredType>) -> Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(schemas_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                == Some("json")
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let filename = path.file_name().unwrap().to_string_lossy();
        let content = std::fs::read_to_string(&path)?;
        let schema_value: serde_json::Value = serde_json::from_str(&content)?;

        let schema_rel = format!("schemas/{filename}");

        if let Some(wiki_types) = schema_value.get("x-wiki-types").and_then(|v| v.as_object()) {
            // Extract aliases once per schema file
            let aliases = extract_aliases(&schema_value);
            let validator = Validator::new(&schema_value)
                .map_err(|e| anyhow::anyhow!("invalid schema {filename}: {e}"))?;

            for (type_name, desc) in wiki_types {
                let description = desc.as_str().unwrap_or("").to_string();
                types.insert(
                    type_name.clone(),
                    RegisteredType {
                        schema_path: schema_rel.clone(),
                        description,
                        validator: Validator::new(&schema_value).unwrap(),
                        aliases: aliases.clone(),
                    },
                );
            }
            // Drop the first validator — we cloned per type above
            drop(validator);
        }
    }

    Ok(())
}

fn discover_from_embedded(types: &mut HashMap<String, RegisteredType>) -> Result<()> {
    for entry in default_schemas::default_type_entries() {
        let filename = entry
            .schema_file
            .strip_prefix("schemas/")
            .unwrap_or(&entry.schema_file);
        let schemas = default_schemas::default_schemas();
        let content = schemas
            .get(filename)
            .ok_or_else(|| anyhow::anyhow!("embedded schema not found: {filename}"))?;
        let registered = compile_schema(&entry.schema_file, &entry.description, content)?;
        types.insert(entry.type_name, registered);
    }
    Ok(())
}

fn compile_schema(schema_path: &str, description: &str, content: &str) -> Result<RegisteredType> {
    let schema_value: serde_json::Value = serde_json::from_str(content)?;
    let validator = Validator::new(&schema_value)
        .map_err(|e| anyhow::anyhow!("invalid schema {schema_path}: {e}"))?;
    let aliases = extract_aliases(&schema_value);

    Ok(RegisteredType {
        schema_path: schema_path.to_string(),
        description: description.to_string(),
        validator,
        aliases,
    })
}

fn extract_aliases(schema: &serde_json::Value) -> HashMap<String, String> {
    schema
        .get("x-index-aliases")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default()
}

// ── Hashing ───────────────────────────────────────────────────────────────────

fn compute_hashes(types: &HashMap<String, RegisteredType>) -> (String, HashMap<String, String>) {
    use std::collections::BTreeMap;
    use std::hash::{Hash, Hasher};

    let mut type_hashes = HashMap::new();
    // Use BTreeMap for deterministic ordering
    let sorted: BTreeMap<_, _> = types.iter().collect();

    let mut combined = std::collections::hash_map::DefaultHasher::new();
    for (name, rt) in &sorted {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        rt.schema_path.hash(&mut h);
        // Sort aliases for deterministic hashing
        let sorted_aliases: BTreeMap<_, _> = rt.aliases.iter().collect();
        for (k, v) in &sorted_aliases {
            k.hash(&mut h);
            v.hash(&mut h);
        }
        let type_hash = format!("{:016x}", h.finish());
        type_hashes.insert(name.to_string(), type_hash.clone());
        type_hash.hash(&mut combined);
    }

    (format!("{:016x}", combined.finish()), type_hashes)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn yaml_fm_to_json(fm: &BTreeMap<String, Value>) -> Result<serde_json::Value> {
    // Round-trip through serde: yaml::Value → String → json::Value
    let yaml_str = serde_yaml::to_string(fm)?;
    let json: serde_json::Value = serde_yaml::from_str(&yaml_str)?;
    Ok(json)
}
