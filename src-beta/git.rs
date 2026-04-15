//! Git operations via libgit2 — init, stage, commit, diff.

use anyhow::{Context, Result};
use git2::{IndexAddOption, Repository, Signature};
use std::path::Path;

/// Initialise a git repository at `root` if one does not already exist.
pub fn init_if_needed(root: &Path) -> Result<()> {
    if !root.join(".git").exists() {
        Repository::init(root)
            .with_context(|| format!("failed to git init {}", root.display()))?;
    }
    Ok(())
}

/// Stage all new and modified files under `root`.
pub fn stage_all(root: &Path) -> Result<()> {
    let repo = Repository::open(root)
        .with_context(|| format!("failed to open git repository at {}", root.display()))?;
    let mut index = repo.index().context("failed to open git index")?;
    index
        .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
        .context("failed to stage files")?;
    index.write().context("failed to write git index")?;
    Ok(())
}

/// Create a commit in the repository at `root` with the given `message`.
///
/// Stages all changes first. Handles the initial commit (no parent) correctly.
pub fn commit(root: &Path, message: &str) -> Result<()> {
    let repo = Repository::open(root)
        .with_context(|| format!("failed to open git repository at {}", root.display()))?;

    // Stage everything.
    let mut index = repo.index().context("failed to open git index")?;
    index
        .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
        .context("failed to stage files")?;
    index.write().context("failed to write git index")?;

    let tree_id = index.write_tree().context("failed to write tree")?;
    let tree = repo.find_tree(tree_id).context("failed to find tree")?;

    let sig = Signature::now("wiki", "wiki@llm-wiki").context("failed to create signature")?;

    // Resolve HEAD to find the parent commit (absent on the very first commit).
    let parent_commits: Vec<git2::Commit> = match repo.head() {
        Ok(head) => {
            let c = head.peel_to_commit().context("failed to peel HEAD to commit")?;
            vec![c]
        }
        Err(_) => vec![], // initial commit — no parent
    };
    let parents: Vec<&git2::Commit> = parent_commits.iter().collect();

    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
        .context("failed to create commit")?;

    Ok(())
}

/// Return the diff between HEAD and HEAD~1 as a unified diff string.
///
/// Returns an empty string if the repository has fewer than two commits
/// (i.e. HEAD~1 does not exist).
pub fn diff_last(root: &Path) -> Result<String> {
    let repo = Repository::open(root)
        .with_context(|| format!("failed to open git repository at {}", root.display()))?;

    let head = repo.head().context("failed to resolve HEAD")?;
    let head_commit = head.peel_to_commit().context("failed to peel HEAD to commit")?;

    // HEAD~1 may not exist on the very first commit.
    let parent = match head_commit.parent(0) {
        Ok(p) => p,
        Err(_) => return Ok(String::new()),
    };

    let head_tree = head_commit.tree().context("failed to get HEAD tree")?;
    let parent_tree = parent.tree().context("failed to get parent tree")?;

    let diff = repo
        .diff_tree_to_tree(Some(&parent_tree), Some(&head_tree), None)
        .context("failed to compute diff")?;

    let mut output = String::new();
    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        match line.origin() {
            '+' | '-' | ' ' => output.push(line.origin()),
            _ => {}
        }
        if let Ok(s) = std::str::from_utf8(line.content()) {
            output.push_str(s);
        }
        true
    })
    .context("failed to format diff")?;

    Ok(output)
}

/// Stage all changes under `root` and create a commit with `message`.
///
/// Convenience wrapper over [`stage_all`] + [`commit`].
pub fn commit_all(root: &Path, message: &str) -> Result<()> {
    commit(root, message)
}
