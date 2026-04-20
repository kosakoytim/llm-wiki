use std::path::PathBuf;

use anyhow::Result;

use crate::engine::Engine;
use crate::indexing;
use crate::search;

pub struct SearchParams<'a> {
    pub query: &'a str,
    pub type_filter: Option<&'a str>,
    pub no_excerpt: bool,
    pub top_k: Option<usize>,
    pub include_sections: bool,
    pub all: bool,
}

pub fn search(
    engine: &Engine,
    wiki_name: &str,
    params: &SearchParams<'_>,
) -> Result<Vec<search::PageRef>> {
    let space = engine.space(wiki_name)?;
    let resolved = space.resolved_config(&engine.config);

    let opts = search::SearchOptions {
        no_excerpt: params.no_excerpt,
        include_sections: params.include_sections,
        top_k: params
            .top_k
            .unwrap_or(resolved.defaults.search_top_k as usize),
        r#type: params.type_filter.map(|s| s.to_string()),
    };

    if params.all {
        let wikis: Vec<(String, PathBuf)> = engine
            .spaces
            .values()
            .map(|s| (s.name.clone(), s.index_path().to_path_buf()))
            .collect();
        return search::search_all(params.query, &opts, &wikis, &space.index_schema);
    }

    let recovery_ctx = if engine.config.index.auto_recovery {
        Some(indexing::RecoveryContext { wiki_root: &space.wiki_root, repo_root: &space.repo_root, registry: &space.type_registry })
    } else {
        None
    };
    search::search(
        params.query,
        &opts,
        space.index_path(),
        wiki_name,
        &space.index_schema,
        recovery_ctx.as_ref(),
    )
}


pub fn list(
    engine: &Engine,
    wiki_name: &str,
    type_filter: Option<&str>,
    status: Option<&str>,
    page: usize,
    page_size: Option<usize>,
) -> Result<search::PageList> {
    let space = engine.space(wiki_name)?;
    let resolved = space.resolved_config(&engine.config);

    let opts = search::ListOptions {
        r#type: type_filter.map(|s| s.to_string()),
        status: status.map(|s| s.to_string()),
        page,
        page_size: page_size.unwrap_or(resolved.defaults.list_page_size as usize),
    };
    let recovery_ctx = if engine.config.index.auto_recovery {
        Some(indexing::RecoveryContext { wiki_root: &space.wiki_root, repo_root: &space.repo_root, registry: &space.type_registry })
    } else {
        None
    };
    search::list(
        &opts,
        space.index_path(),
        wiki_name,
        &space.index_schema,
        recovery_ctx.as_ref(),
    )
}
