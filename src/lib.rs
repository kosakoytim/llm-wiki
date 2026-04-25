//! Git-backed wiki engine. Full-text search, typed pages, concept graph,
//! MCP and ACP transports. The CLI is the primary interface; this crate also
//! exposes the engine internals for embedding or testing.

pub mod acp;
pub mod cli;
pub mod config;
pub mod default_schemas;
pub mod engine;
pub mod frontmatter;
pub mod git;
pub mod graph;
pub mod index_manager;
pub mod index_schema;
pub mod ingest;
pub mod links;
pub mod markdown;
pub mod mcp;
pub mod ops;
pub mod search;
pub mod server;
pub mod slug;
pub mod space_builder;
pub mod spaces;
pub mod type_registry;
pub mod watch;
