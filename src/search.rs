//! Full-text search via tantivy.
//!
//! The tantivy index lives in `.wiki/search-index/` (gitignored) and is
//! rebuilt on demand from committed Markdown files.
//!
//! ## Index lifecycle
//!
//! - `build_index` wipes any existing index and creates a fresh one from all
//!   `.md` files under `wiki_root`, skipping `raw/` and `.wiki/`.
//! - `search` always calls `build_index` before querying, ensuring results
//!   reflect the current state of committed Markdown without a separate
//!   rebuild step.
//! - `wiki search --rebuild-index` calls `build_index` alone and exits — useful
//!   for scripts or to verify the index builds correctly on a fresh clone.

use anyhow::{Context, Result};
use std::path::Path;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Schema, Value, STORED, STRING, TEXT};
use tantivy::{Index, IndexWriter, TantivyDocument};
use walkdir::WalkDir;

use crate::markdown::parse_frontmatter;
use crate::registry::WikiRegistry;

/// A single search result returned by [`search`].
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Page slug — relative path from wiki root without the `.md` extension.
    pub slug: String,
    /// Page title from frontmatter.
    pub title: String,
    /// First ~200 characters of the page body — a quick reading preview.
    pub snippet: String,
    /// BM25 relevance score (higher is more relevant).
    pub score: f32,
    /// Page category (`concept`, `source-summary`, `query-result`, `contradiction`).
    pub page_type: String,
}

/// Build the tantivy schema used for all wiki indexes.
///
/// Fields:
/// - `slug`  — STRING | STORED  (exact token, stored for retrieval)
/// - `title` — TEXT  | STORED  (tokenised full-text, stored)
/// - `tags`  — TEXT  | STORED  (space-joined tag list, tokenised)
/// - `body`  — TEXT  | STORED  (page body, tokenised; stored for snippet)
/// - `type`  — STRING | STORED  (exact page category, stored)
fn wiki_schema() -> Schema {
    let mut b = Schema::builder();
    b.add_text_field("slug", STRING | STORED);
    b.add_text_field("title", TEXT | STORED);
    b.add_text_field("tags", TEXT | STORED);
    b.add_text_field("body", TEXT | STORED);
    b.add_text_field("type", STRING | STORED);
    b.build()
}

/// Build (or rebuild) the tantivy index from all `.md` files under `wiki_root`.
///
/// Any existing index at `index_dir` is wiped first. Files inside `raw/` and
/// `.wiki/` are skipped. Pages that fail frontmatter parsing are skipped
/// silently (they may be raw Markdown not produced by the wiki engine).
///
/// Returns the opened [`Index`] ready for querying.
pub fn build_index(wiki_root: &Path, index_dir: &Path) -> Result<Index> {
    // Wipe and recreate the index directory.
    if index_dir.exists() {
        std::fs::remove_dir_all(index_dir)
            .with_context(|| format!("failed to remove index dir {}", index_dir.display()))?;
    }
    std::fs::create_dir_all(index_dir)
        .with_context(|| format!("failed to create index dir {}", index_dir.display()))?;

    let schema = wiki_schema();
    let index = Index::create_in_dir(index_dir, schema.clone())
        .context("failed to create tantivy index")?;
    let mut writer: IndexWriter = index
        .writer(50_000_000)
        .context("failed to create index writer")?;

    let slug_field = schema
        .get_field("slug")
        .context("slug field missing from schema")?;
    let title_field = schema
        .get_field("title")
        .context("title field missing from schema")?;
    let tags_field = schema
        .get_field("tags")
        .context("tags field missing from schema")?;
    let body_field = schema
        .get_field("body")
        .context("body field missing from schema")?;
    let type_field = schema
        .get_field("type")
        .context("type field missing from schema")?;

    let raw_dir = wiki_root.join("raw");
    let wiki_dir = wiki_root.join(".wiki");

    for entry in WalkDir::new(wiki_root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Only index `.md` files.
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        // Skip raw/ (unprocessed source files) and .wiki/ (index artefacts).
        if path.starts_with(&raw_dir) || path.starts_with(&wiki_dir) {
            continue;
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let (fm, body) = match parse_frontmatter(&content) {
            Ok(r) => r,
            Err(_) => continue, // not a wiki-managed page; skip
        };

        let relative = match path.strip_prefix(wiki_root) {
            Ok(r) => r,
            Err(_) => continue,
        };
        // Normalise Windows path separators so slugs are always forward-slash.
        let slug = relative
            .with_extension("")
            .to_string_lossy()
            .replace('\\', "/");

        let tags_str = fm.tags.join(" ");
        // Serialise PageType to its kebab-case string ("concept", "source-summary", …).
        let type_str = serde_json::to_string(&fm.page_type)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string();

        let mut doc = TantivyDocument::default();
        doc.add_text(slug_field, &slug);
        doc.add_text(title_field, &fm.title);
        doc.add_text(tags_field, &tags_str);
        doc.add_text(body_field, body);
        doc.add_text(type_field, &type_str);

        writer
            .add_document(doc)
            .context("failed to add document to index")?;
    }

    writer.commit().context("failed to commit index")?;
    Ok(index)
}

/// Rebuild the index from scratch (alias for [`build_index`] — always wipes first).
pub fn rebuild_index(wiki_root: &Path, index_dir: &Path) -> Result<Index> {
    build_index(wiki_root, index_dir)
}

/// Query a tantivy [`Index`] and return BM25-ranked [`SearchResult`]s.
///
/// Searches `title`, `tags`, and `body` fields. Returns at most `limit` results.
/// Returns an empty slice (not an error) if the query string cannot be parsed or
/// if no documents match.
pub fn search_index(index: &Index, query_str: &str, limit: usize) -> Result<Vec<SearchResult>> {
    let schema = index.schema();
    let slug_field = schema
        .get_field("slug")
        .context("slug field missing from schema")?;
    let title_field = schema
        .get_field("title")
        .context("title field missing from schema")?;
    let tags_field = schema
        .get_field("tags")
        .context("tags field missing from schema")?;
    let body_field = schema
        .get_field("body")
        .context("body field missing from schema")?;
    let type_field = schema
        .get_field("type")
        .context("type field missing from schema")?;

    let reader = index
        .reader()
        .context("failed to open index reader")?;
    let searcher = reader.searcher();

    let query_parser =
        QueryParser::for_index(index, vec![title_field, tags_field, body_field]);
    let query = match query_parser.parse_query(query_str) {
        Ok(q) => q,
        Err(_) => return Ok(Vec::new()),
    };

    let top_docs = searcher
        .search(&query, &TopDocs::with_limit(limit))
        .context("search failed")?;

    let mut results = Vec::with_capacity(top_docs.len());
    for (score, doc_addr) in top_docs {
        let doc: TantivyDocument = searcher.doc(doc_addr).context("failed to retrieve doc")?;

        let slug = doc
            .get_first(slug_field)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let title = doc
            .get_first(title_field)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let snippet = doc
            .get_first(body_field)
            .and_then(|v| v.as_str())
            .map(|b: &str| b.chars().take(200).collect::<String>())
            .unwrap_or_default();
        let page_type = doc
            .get_first(type_field)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        results.push(SearchResult {
            slug,
            title,
            snippet,
            score,
            page_type,
        });
    }

    Ok(results)
}

/// A search result that includes the source wiki name, for `wiki search --all`.
#[derive(Debug, Clone)]
pub struct SearchResultWithWiki {
    /// Name of the wiki that contains this result.
    pub wiki_name: String,
    /// Page slug — relative path from the wiki root without the `.md` extension.
    pub slug: String,
    /// Page title from frontmatter.
    pub title: String,
    /// First ~200 characters of the page body.
    pub snippet: String,
    /// BM25 relevance score.
    pub score: f32,
    /// Page category (`concept`, `source-summary`, …).
    pub page_type: String,
}

/// Fan out a query to every registered wiki, merge all results by descending
/// BM25 score, and truncate to `limit`.
///
/// Wikis that have no pages or an un-buildable index are silently skipped.
pub fn search_all(
    registry: &WikiRegistry,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchResultWithWiki>> {
    let mut all: Vec<SearchResultWithWiki> = Vec::new();

    for entry in registry.entries() {
        let results = search(query, &entry.path, false).unwrap_or_default();
        for r in results {
            all.push(SearchResultWithWiki {
                wiki_name: entry.name.clone(),
                slug: r.slug,
                title: r.title,
                snippet: r.snippet,
                score: r.score,
                page_type: r.page_type,
            });
        }
    }

    all.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    all.truncate(limit);
    Ok(all)
}

/// Search the wiki using `query` and return BM25-ranked results.
///
/// Always rebuilds the tantivy index from committed Markdown before querying,
/// so results reflect any pages added since the last run without an explicit
/// `--rebuild-index` pass.
///
/// The `rebuild_index` parameter is accepted for API compatibility but is
/// effectively a no-op — the index is always rebuilt.
pub fn search(query: &str, wiki_root: &Path, _rebuild_index: bool) -> Result<Vec<SearchResult>> {
    let index_dir = wiki_root.join(".wiki").join("search-index");
    let index = build_index(wiki_root, &index_dir)?;
    search_index(&index, query, 20)
}
