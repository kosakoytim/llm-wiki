//! `llm-wiki` library crate — git-backed wiki engine.
//!
//! The `wiki` binary is a thin dispatcher on top of this crate.
//! Integration tests import directly from here via `use llm_wiki::…`.
//!
//! Phase 0: all module bodies are `todo!()` stubs.
#![allow(dead_code, unused_imports)]

pub mod analysis;
pub mod cli;
pub mod config;
pub mod context;
pub mod contradiction;
pub mod git;
pub mod graph;
pub mod ingest;
pub mod init;
pub mod integrate;
pub mod lint;
pub mod markdown;
pub mod registry;
pub mod search;
pub mod server;
