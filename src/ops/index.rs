use anyhow::Result;

use crate::engine::{Engine, EngineManager};
use crate::index_manager;

pub fn index_rebuild(manager: &EngineManager, wiki_name: &str) -> Result<index_manager::IndexReport> {
    manager.rebuild_index(wiki_name)
}

pub fn index_status(engine: &Engine, wiki_name: &str) -> Result<index_manager::IndexStatus> {
    let space = engine.space(wiki_name)?;
    space.index_manager.status(&space.repo_root)
}
