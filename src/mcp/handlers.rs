use std::path::PathBuf;

use rmcp::model::Content;
use serde_json::{Map, Value};

use crate::config;
use crate::git;
use crate::graph;
use crate::ingest;
use crate::markdown;
use crate::search;
use crate::slug::{resolve_read_target, ReadTarget, Slug, WikiUri};
use crate::spaces;

use super::helpers::*;
use super::McpServer;

// ── Spaces ────────────────────────────────────────────────────────────────────

pub fn handle_spaces_create(server: &McpServer, args: &Map<String, Value>) -> ToolHandlerResult {
    let path = arg_str_req(args, "path")?;
    let name = arg_str_req(args, "name")?;
    let description = arg_str(args, "description");
    let force = arg_bool(args, "force");
    let set_default = arg_bool(args, "set_default");

    let engine = server.engine();
    let report = spaces::create(
        &PathBuf::from(&path),
        &name,
        description.as_deref(),
        force,
        set_default,
        &engine.config_path,
    )
    .map_err(|e| format!("{e}"))?;

    let json = serde_json::to_string_pretty(&serde_json::json!({
        "path": report.path,
        "name": report.name,
        "created": report.created,
        "registered": report.registered,
        "committed": report.committed,
    }))
    .map_err(|e| format!("{e}"))?;
    ok_text(json)
}

pub fn handle_spaces_list(server: &McpServer, _args: &Map<String, Value>) -> ToolHandlerResult {
    let engine = server.engine();
    let entries = spaces::load_all(&engine.config);
    let s = serde_json::to_string_pretty(&entries).map_err(|e| format!("{e}"))?;
    ok_text(s)
}

pub fn handle_spaces_remove(server: &McpServer, args: &Map<String, Value>) -> ToolHandlerResult {
    let name = arg_str_req(args, "name")?;
    let delete = arg_bool(args, "delete");
    let engine = server.engine();
    spaces::remove(&name, delete, &engine.config_path).map_err(|e| format!("{e}"))?;
    ok_text(format!("Removed wiki \"{name}\""))
}

pub fn handle_spaces_set_default(
    server: &McpServer,
    args: &Map<String, Value>,
) -> ToolHandlerResult {
    let name = arg_str_req(args, "name")?;
    let engine = server.engine();
    spaces::set_default_wiki(&name, &engine.config_path).map_err(|e| format!("{e}"))?;
    ok_text(format!("Default wiki set to \"{name}\""))
}

// ── Config ────────────────────────────────────────────────────────────────────

pub fn handle_config(server: &McpServer, args: &Map<String, Value>) -> ToolHandlerResult {
    let action = arg_str_req(args, "action")?;
    let engine = server.engine();
    let config_path = &engine.config_path;

    match action.as_str() {
        "list" => {
            let g = config::load_global(config_path).map_err(|e| format!("{e}"))?;
            let s = toml::to_string_pretty(&g).map_err(|e| format!("{e}"))?;
            ok_text(s)
        }
        "get" => {
            let key = arg_str_req(args, "key")?;
            let g = config::load_global(config_path).map_err(|e| format!("{e}"))?;
            let resolved = config::resolve(&g, &config::WikiConfig::default());
            ok_text(format!(
                "{}: {}",
                key,
                config::get_config_value(&resolved, &g, &key)
            ))
        }
        "set" => {
            let key = arg_str_req(args, "key")?;
            let value = arg_str_req(args, "value")?;
            let is_global = arg_bool(args, "global");
            if is_global {
                let mut g = config::load_global(config_path).map_err(|e| format!("{e}"))?;
                config::set_global_config_value(&mut g, &key, &value)
                    .map_err(|e| format!("{e}"))?;
                config::save_global(&g, config_path).map_err(|e| format!("{e}"))?;
                ok_text(format!("Set {key} = {value} (global)"))
            } else {
                let wiki_name = resolve_wiki_name(&engine, args)?;
                let entry =
                    spaces::resolve_name(&wiki_name, &engine.config).map_err(|e| format!("{e}"))?;
                let entry_path = PathBuf::from(&entry.path);
                let mut wiki_cfg = config::load_wiki(&entry_path).map_err(|e| format!("{e}"))?;
                config::set_wiki_config_value(&mut wiki_cfg, &key, &value)
                    .map_err(|e| format!("{e}"))?;
                config::save_wiki(&wiki_cfg, &entry_path).map_err(|e| format!("{e}"))?;
                ok_text(format!("Set {key} = {value} (wiki: {wiki_name})"))
            }
        }
        _ => Err(format!("unknown config action: {action}")),
    }
}

// ── Content ───────────────────────────────────────────────────────────────────

pub fn handle_content_read(server: &McpServer, args: &Map<String, Value>) -> ToolHandlerResult {
    let uri = arg_str_req(args, "uri")?;
    let engine = server.engine();
    let wiki_flag = arg_str(args, "wiki");
    let (entry, slug) =
        WikiUri::resolve(&uri, wiki_flag.as_deref(), &engine.config).map_err(|e| format!("{e}"))?;
    let wiki_root = PathBuf::from(&entry.path).join("wiki");

    if arg_bool(args, "list_assets") {
        let assets = markdown::list_assets(&slug, &wiki_root).map_err(|e| format!("{e}"))?;
        return ok_text(assets.join("\n"));
    }

    match resolve_read_target(slug.as_str(), &wiki_root).map_err(|e| format!("{e}"))? {
        ReadTarget::Page(_) => {
            let wiki_cfg = config::load_wiki(&PathBuf::from(&entry.path)).unwrap_or_default();
            let resolved = config::resolve(&engine.config, &wiki_cfg);
            let strip = arg_bool(args, "no_frontmatter") || resolved.read.no_frontmatter;
            let content =
                markdown::read_page(&slug, &wiki_root, strip).map_err(|e| format!("{e}"))?;
            ok_text(content)
        }
        ReadTarget::Asset(parent_slug, filename) => {
            let parent = Slug::try_from(parent_slug.as_str()).map_err(|e| format!("{e}"))?;
            let bytes =
                markdown::read_asset(&parent, &filename, &wiki_root).map_err(|e| format!("{e}"))?;
            match String::from_utf8(bytes) {
                Ok(text) => ok_text(text),
                Err(_) => Err(format!(
                    "asset {slug} is binary — access it directly from the filesystem"
                )),
            }
        }
    }
}

pub fn handle_content_write(server: &McpServer, args: &Map<String, Value>) -> ToolHandlerResult {
    let uri = arg_str_req(args, "uri")?;
    let content = arg_str_req(args, "content")?;
    let engine = server.engine();
    let wiki_flag = arg_str(args, "wiki");
    let (entry, slug) =
        WikiUri::resolve(&uri, wiki_flag.as_deref(), &engine.config).map_err(|e| format!("{e}"))?;
    let wiki_root = PathBuf::from(&entry.path).join("wiki");

    let path =
        markdown::write_page(slug.as_str(), &content, &wiki_root).map_err(|e| format!("{e}"))?;
    ok_text(format!(
        "Wrote {} bytes to {}",
        content.len(),
        path.display()
    ))
}

pub fn handle_content_new(server: &McpServer, args: &Map<String, Value>) -> ToolHandlerResult {
    let uri = arg_str_req(args, "uri")?;
    let section = arg_bool(args, "section");
    let bundle = arg_bool(args, "bundle");
    let name_override = arg_str(args, "name");
    let type_override = arg_str(args, "type");

    let engine = server.engine();
    let wiki_flag = arg_str(args, "wiki");
    let (entry, slug) =
        WikiUri::resolve(&uri, wiki_flag.as_deref(), &engine.config).map_err(|e| format!("{e}"))?;
    let wiki_root = PathBuf::from(&entry.path).join("wiki");

    if section {
        markdown::create_section(&slug, &wiki_root).map_err(|e| format!("{e}"))?;
    } else {
        markdown::create_page(
            &slug,
            bundle,
            &wiki_root,
            name_override.as_deref(),
            type_override.as_deref(),
        )
        .map_err(|e| format!("{e}"))?;
    }
    ok_text(format!("wiki://{}/{slug}", entry.name))
}

pub fn handle_content_commit(server: &McpServer, args: &Map<String, Value>) -> ToolHandlerResult {
    let engine = server.engine();
    let wiki_name = resolve_wiki_name(&engine, args)?;
    let space = engine.space(&wiki_name).map_err(|e| format!("{e}"))?;
    let message = arg_str(args, "message");

    let hash = if let Some(slugs_str) = arg_str(args, "slugs") {
        let slugs: Vec<&str> = slugs_str.split(',').map(|s| s.trim()).collect();
        let mut paths = Vec::new();
        for s in &slugs {
            let slug = Slug::try_from(*s).map_err(|e| format!("{e}"))?;
            let resolved = slug.resolve(&space.wiki_root).map_err(|e| format!("{e}"))?;
            if resolved.file_name() == Some(std::ffi::OsStr::new("index.md")) {
                let bundle_dir = resolved.parent().unwrap();
                for entry in walkdir::WalkDir::new(bundle_dir)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    if entry.path().is_file() {
                        paths.push(entry.path().to_path_buf());
                    }
                }
            } else {
                paths.push(resolved);
            }
        }
        let path_refs: Vec<&std::path::Path> = paths.iter().map(|p| p.as_path()).collect();
        let msg = message.unwrap_or_else(|| format!("commit: {}", slugs.join(", ")));
        git::commit_paths(&space.repo_root, &path_refs, &msg).map_err(|e| format!("{e}"))?
    } else {
        let msg = message.unwrap_or_else(|| "commit: all".into());
        git::commit(&space.repo_root, &msg).map_err(|e| format!("{e}"))?
    };

    ok_text(hash)
}

// ── Search ────────────────────────────────────────────────────────────────────

pub fn handle_search(server: &McpServer, args: &Map<String, Value>) -> ToolHandlerResult {
    let query = arg_str_req(args, "query")?;
    let all = arg_bool(args, "all");
    let engine = server.engine();
    let wiki_name = resolve_wiki_name(&engine, args)?;
    let space = engine.space(&wiki_name).map_err(|e| format!("{e}"))?;
    let resolved = space.resolved_config(&engine.config);

    let opts = search::SearchOptions {
        no_excerpt: arg_bool(args, "no_excerpt"),
        include_sections: arg_bool(args, "include_sections"),
        top_k: arg_usize(args, "top_k").unwrap_or(resolved.defaults.search_top_k as usize),
        r#type: arg_str(args, "type"),
    };

    let results = if all {
        let wikis: Vec<(String, PathBuf)> = engine
            .spaces
            .values()
            .map(|s| (s.name.clone(), s.index_path.clone()))
            .collect();
        search::search_all(&query, &opts, &wikis, &space.schema).map_err(|e| format!("{e}"))?
    } else {
        let recovery = if engine.config.index.auto_recovery {
            Some((space.wiki_root.clone(), space.repo_root.clone()))
        } else {
            None
        };
        let recovery_ctx = recovery.as_ref().map(|(wr, rr)| search::RecoveryContext {
            wiki_root: wr,
            repo_root: rr,
        });
        search::search(
            &query,
            &opts,
            &space.index_path,
            &wiki_name,
            &space.schema,
            recovery_ctx.as_ref(),
        )
        .map_err(|e| format!("{e}"))?
    };

    let s = serde_json::to_string_pretty(&results).map_err(|e| format!("{e}"))?;
    ok_text(s)
}

// ── List ──────────────────────────────────────────────────────────────────────

pub fn handle_list(server: &McpServer, args: &Map<String, Value>) -> ToolHandlerResult {
    let engine = server.engine();
    let wiki_name = resolve_wiki_name(&engine, args)?;
    let space = engine.space(&wiki_name).map_err(|e| format!("{e}"))?;
    let resolved = space.resolved_config(&engine.config);

    let opts = search::ListOptions {
        r#type: arg_str(args, "type"),
        status: arg_str(args, "status"),
        page: arg_usize(args, "page").unwrap_or(1),
        page_size: arg_usize(args, "page_size")
            .unwrap_or(resolved.defaults.list_page_size as usize),
    };
    let recovery = if engine.config.index.auto_recovery {
        Some((space.wiki_root.clone(), space.repo_root.clone()))
    } else {
        None
    };
    let recovery_ctx = recovery.as_ref().map(|(wr, rr)| search::RecoveryContext {
        wiki_root: wr,
        repo_root: rr,
    });
    let result = search::list(&opts, &space.index_path, &wiki_name, &space.schema, recovery_ctx.as_ref())
        .map_err(|e| format!("{e}"))?;
    let s = serde_json::to_string_pretty(&result).map_err(|e| format!("{e}"))?;
    ok_text(s)
}

// ── Ingest ────────────────────────────────────────────────────────────────────

pub fn handle_ingest(server: &McpServer, args: &Map<String, Value>) -> ToolHandlerResult {
    let path = arg_str_req(args, "path")?;
    let dry_run = arg_bool(args, "dry_run");

    // Read path: validate and ingest
    let (report, wiki_name, notify_uris) = {
        let engine = server.engine();
        let wiki_name = resolve_wiki_name(&engine, args)?;
        let space = engine.space(&wiki_name).map_err(|e| format!("{e}"))?;
        let resolved = space.resolved_config(&engine.config);

        let opts = ingest::IngestOptions {
            dry_run,
            auto_commit: resolved.ingest.auto_commit,
        };
        let report = ingest::ingest(
            std::path::Path::new(&path),
            &opts,
            &space.wiki_root,
            &engine.type_registry,
            &resolved.validation,
        )
        .map_err(|e| format!("{e}"))?;

        let notify_uris = if !dry_run {
            let ingest_path = space.wiki_root.join(&path);
            collect_page_uris(&ingest_path, &space.wiki_root, &wiki_name)
        } else {
            vec![]
        };

        (report, wiki_name, notify_uris)
    }; // read lock dropped

    // Mutation path: index update via EngineManager
    if !dry_run {
        if let Err(e) = server.manager.on_ingest(&wiki_name) {
            tracing::warn!(error = %e, "incremental index update failed after ingest");
        }
    }

    let s = serde_json::to_string_pretty(&report).map_err(|e| format!("{e}"))?;
    Ok((vec![Content::text(s)], notify_uris))
}

// ── Index ─────────────────────────────────────────────────────────────────────

pub fn handle_index_rebuild(server: &McpServer, args: &Map<String, Value>) -> ToolHandlerResult {
    // Read path: resolve wiki name
    let wiki_name = {
        let engine = server.engine();
        resolve_wiki_name(&engine, args)?
    }; // read lock dropped

    // Mutation path: rebuild via EngineManager
    let report = server
        .manager
        .rebuild_index(&wiki_name)
        .map_err(|e| format!("{e}"))?;
    let s = serde_json::to_string_pretty(&report).map_err(|e| format!("{e}"))?;
    ok_text(s)
}

pub fn handle_index_status(server: &McpServer, args: &Map<String, Value>) -> ToolHandlerResult {
    let engine = server.engine();
    let wiki_name = resolve_wiki_name(&engine, args)?;
    let space = engine.space(&wiki_name).map_err(|e| format!("{e}"))?;

    let status = search::index_status(&wiki_name, &space.index_path, &space.repo_root)
        .map_err(|e| format!("{e}"))?;
    let s = serde_json::to_string_pretty(&status).map_err(|e| format!("{e}"))?;
    ok_text(s)
}

// ── Graph ─────────────────────────────────────────────────────────────────────

pub fn handle_graph(server: &McpServer, args: &Map<String, Value>) -> ToolHandlerResult {
    let engine = server.engine();
    let wiki_name = resolve_wiki_name(&engine, args)?;
    let space = engine.space(&wiki_name).map_err(|e| format!("{e}"))?;
    let resolved = space.resolved_config(&engine.config);

    let fmt = arg_str(args, "format").unwrap_or_else(|| resolved.graph.format.clone());
    let types: Vec<String> = arg_str(args, "type")
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    let filter = graph::GraphFilter {
        root: arg_str(args, "root"),
        depth: arg_usize(args, "depth").or(Some(resolved.graph.depth as usize)),
        types,
        relation: arg_str(args, "relation"),
    };
    let g =
        graph::build_graph(&space.index_path, &space.schema, &filter).map_err(|e| format!("{e}"))?;

    let rendered = match fmt.as_str() {
        "dot" => graph::render_dot(&g),
        _ => graph::render_mermaid(&g),
    };

    let out = if let Some(out_path) = arg_str(args, "output") {
        let content = if out_path.ends_with(".md") {
            graph::wrap_graph_md(&rendered, &fmt, &filter)
        } else {
            rendered
        };
        if let Err(e) = std::fs::write(&out_path, &content) {
            tracing::warn!(path = %out_path, error = %e, "graph output write failed");
        }
        out_path
    } else {
        "stdout".to_string()
    };

    let report = graph::GraphReport {
        nodes: g.node_count(),
        edges: g.edge_count(),
        output: out,
    };
    let s = serde_json::to_string_pretty(&report).map_err(|e| format!("{e}"))?;
    ok_text(s)
}
