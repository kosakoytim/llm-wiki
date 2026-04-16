use std::fs;

use llm_wiki::git;

#[test]
fn init_repo_creates_git_repository() {
    let dir = tempfile::tempdir().unwrap();
    git::init_repo(dir.path()).unwrap();
    assert!(dir.path().join(".git").exists());
}

#[test]
fn commit_creates_commit_and_returns_hash() {
    let dir = tempfile::tempdir().unwrap();
    git::init_repo(dir.path()).unwrap();
    fs::write(dir.path().join("test.txt"), "hello").unwrap();

    let hash = git::commit(dir.path(), "test commit").unwrap();
    assert!(!hash.is_empty());
    assert_eq!(hash.len(), 40); // SHA-1 hex
}

#[test]
fn current_head_returns_commit_hash() {
    let dir = tempfile::tempdir().unwrap();
    git::init_repo(dir.path()).unwrap();
    fs::write(dir.path().join("test.txt"), "hello").unwrap();
    git::commit(dir.path(), "initial").unwrap();

    let head = git::current_head(dir.path()).unwrap();
    assert_eq!(head.len(), 40);
}

#[test]
fn current_head_matches_commit_hash() {
    let dir = tempfile::tempdir().unwrap();
    git::init_repo(dir.path()).unwrap();
    fs::write(dir.path().join("test.txt"), "hello").unwrap();

    let commit_hash = git::commit(dir.path(), "initial").unwrap();
    let head_hash = git::current_head(dir.path()).unwrap();
    assert_eq!(commit_hash, head_hash);
}

#[test]
fn commit_paths_commits_only_specified_files() {
    let dir = tempfile::tempdir().unwrap();
    git::init_repo(dir.path()).unwrap();

    // Initial commit so HEAD exists
    fs::write(dir.path().join("init.txt"), "init").unwrap();
    git::commit(dir.path(), "initial").unwrap();

    // Write two files, commit only one
    fs::write(dir.path().join("a.txt"), "aaa").unwrap();
    fs::write(dir.path().join("b.txt"), "bbb").unwrap();

    let hash = git::commit_paths(
        dir.path(),
        &[&dir.path().join("a.txt")],
        "commit a only",
    )
    .unwrap();
    assert_eq!(hash.len(), 40);

    // a.txt should be in the last commit
    let files = git::diff_last(dir.path()).unwrap();
    assert!(files.contains(&"a.txt".to_string()));
    assert!(!files.contains(&"b.txt".to_string()));
}

#[test]
fn commit_paths_commits_all_files_in_bundle_folder() {
    let dir = tempfile::tempdir().unwrap();
    git::init_repo(dir.path()).unwrap();

    // Initial commit
    fs::write(dir.path().join("init.txt"), "init").unwrap();
    git::commit(dir.path(), "initial").unwrap();

    // Create a bundle folder with index.md + asset
    let bundle = dir.path().join("wiki").join("concepts").join("moe");
    fs::create_dir_all(&bundle).unwrap();
    fs::write(bundle.join("index.md"), "---\ntitle: MoE\n---\n# MoE").unwrap();
    fs::write(bundle.join("diagram.png"), "fake-png").unwrap();

    // Also create an unrelated file that should NOT be committed
    fs::write(dir.path().join("unrelated.txt"), "nope").unwrap();

    // Collect all files in the bundle folder (simulates commit handler logic)
    let mut paths = Vec::new();
    for entry in walkdir::WalkDir::new(&bundle)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.path().is_file() {
            paths.push(entry.path().to_path_buf());
        }
    }
    let path_refs: Vec<&std::path::Path> = paths.iter().map(|p| p.as_path()).collect();

    let hash = git::commit_paths(dir.path(), &path_refs, "commit bundle").unwrap();
    assert_eq!(hash.len(), 40);

    let files = git::diff_last(dir.path()).unwrap();
    assert!(files.iter().any(|f| f.ends_with("index.md")));
    assert!(files.iter().any(|f| f.ends_with("diagram.png")));
    assert!(!files.iter().any(|f| f.ends_with("unrelated.txt")));
}

#[test]
fn commit_paths_commits_section_with_nested_pages() {
    let dir = tempfile::tempdir().unwrap();
    git::init_repo(dir.path()).unwrap();

    // Initial commit
    fs::write(dir.path().join("init.txt"), "init").unwrap();
    git::commit(dir.path(), "initial").unwrap();

    // Create a section with index + nested flat page + nested bundle
    let section = dir.path().join("wiki").join("concepts");
    fs::create_dir_all(&section).unwrap();
    fs::write(section.join("index.md"), "---\ntitle: Concepts\n---\n# Concepts").unwrap();
    fs::write(section.join("scaling.md"), "---\ntitle: Scaling\n---\n# Scaling").unwrap();
    let bundle = section.join("moe");
    fs::create_dir_all(&bundle).unwrap();
    fs::write(bundle.join("index.md"), "---\ntitle: MoE\n---\n# MoE").unwrap();
    fs::write(bundle.join("diagram.png"), "fake-png").unwrap();

    // Unrelated file outside the section
    fs::write(dir.path().join("other.txt"), "nope").unwrap();

    // Walk the section folder (simulates commit handler for a section slug)
    let mut paths = Vec::new();
    for entry in walkdir::WalkDir::new(&section)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.path().is_file() {
            paths.push(entry.path().to_path_buf());
        }
    }
    let path_refs: Vec<&std::path::Path> = paths.iter().map(|p| p.as_path()).collect();

    let hash = git::commit_paths(dir.path(), &path_refs, "commit section").unwrap();
    assert_eq!(hash.len(), 40);

    let files = git::diff_last(dir.path()).unwrap();
    assert!(files.iter().any(|f| f.ends_with("concepts/index.md")));
    assert!(files.iter().any(|f| f.ends_with("concepts/scaling.md")));
    assert!(files.iter().any(|f| f.ends_with("concepts/moe/index.md")));
    assert!(files.iter().any(|f| f.ends_with("concepts/moe/diagram.png")));
    assert!(!files.iter().any(|f| f.ends_with("other.txt")));
}

#[test]
fn changed_wiki_files_detects_new_file() {
    let dir = tempfile::tempdir().unwrap();
    git::init_repo(dir.path()).unwrap();

    // Initial commit
    let wiki = dir.path().join("wiki");
    fs::create_dir_all(&wiki).unwrap();
    fs::write(wiki.join("init.md"), "---\ntitle: Init\n---\n").unwrap();
    git::commit(dir.path(), "initial").unwrap();

    // New uncommitted file
    fs::write(wiki.join("new-page.md"), "---\ntitle: New\n---\n").unwrap();

    let changes = git::changed_wiki_files(dir.path(), &wiki).unwrap();
    assert!(changes.iter().any(|c| c.path.ends_with("new-page.md")
        && matches!(c.status, git2::Delta::Untracked | git2::Delta::Added)));
}

#[test]
fn changed_wiki_files_detects_modified_file() {
    let dir = tempfile::tempdir().unwrap();
    git::init_repo(dir.path()).unwrap();

    let wiki = dir.path().join("wiki");
    fs::create_dir_all(&wiki).unwrap();
    fs::write(wiki.join("page.md"), "---\ntitle: Old\n---\n").unwrap();
    git::commit(dir.path(), "initial").unwrap();

    // Modify
    fs::write(wiki.join("page.md"), "---\ntitle: New\n---\n").unwrap();

    let changes = git::changed_wiki_files(dir.path(), &wiki).unwrap();
    assert!(changes.iter().any(|c| c.path.ends_with("page.md")
        && c.status == git2::Delta::Modified));
}

#[test]
fn changed_wiki_files_detects_deleted_file() {
    let dir = tempfile::tempdir().unwrap();
    git::init_repo(dir.path()).unwrap();

    let wiki = dir.path().join("wiki");
    fs::create_dir_all(&wiki).unwrap();
    fs::write(wiki.join("page.md"), "---\ntitle: Gone\n---\n").unwrap();
    git::commit(dir.path(), "initial").unwrap();

    fs::remove_file(wiki.join("page.md")).unwrap();

    let changes = git::changed_wiki_files(dir.path(), &wiki).unwrap();
    assert!(changes.iter().any(|c| c.path.ends_with("page.md")
        && c.status == git2::Delta::Deleted));
}

#[test]
fn changed_wiki_files_ignores_non_md() {
    let dir = tempfile::tempdir().unwrap();
    git::init_repo(dir.path()).unwrap();

    let wiki = dir.path().join("wiki");
    fs::create_dir_all(&wiki).unwrap();
    fs::write(wiki.join("init.md"), "---\ntitle: Init\n---\n").unwrap();
    git::commit(dir.path(), "initial").unwrap();

    fs::write(wiki.join("image.png"), "fake-png").unwrap();

    let changes = git::changed_wiki_files(dir.path(), &wiki).unwrap();
    assert!(!changes.iter().any(|c| c.path.ends_with("image.png")));
}

#[test]
fn changed_wiki_files_ignores_files_outside_wiki() {
    let dir = tempfile::tempdir().unwrap();
    git::init_repo(dir.path()).unwrap();

    let wiki = dir.path().join("wiki");
    fs::create_dir_all(&wiki).unwrap();
    fs::write(wiki.join("init.md"), "---\ntitle: Init\n---\n").unwrap();
    git::commit(dir.path(), "initial").unwrap();

    fs::write(dir.path().join("README.md"), "# Hello").unwrap();

    let changes = git::changed_wiki_files(dir.path(), &wiki).unwrap();
    assert!(!changes.iter().any(|c| c.path.ends_with("README.md")));
}

#[test]
fn changed_since_commit_detects_gap() {
    let dir = tempfile::tempdir().unwrap();
    git::init_repo(dir.path()).unwrap();

    let wiki = dir.path().join("wiki");
    fs::create_dir_all(&wiki).unwrap();
    fs::write(wiki.join("page-a.md"), "---\ntitle: A\n---\n").unwrap();
    let first = git::commit(dir.path(), "first").unwrap();

    fs::write(wiki.join("page-b.md"), "---\ntitle: B\n---\n").unwrap();
    git::commit(dir.path(), "second").unwrap();

    let changes = git::changed_since_commit(dir.path(), &wiki, &first).unwrap();
    assert!(changes.iter().any(|c| c.path.ends_with("page-b.md")));
    assert!(!changes.iter().any(|c| c.path.ends_with("page-a.md")));
}
