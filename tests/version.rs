//! `boxmux --version` names the version and the commit the binary was built from:
//! `boxmux <version> commit <40-character commit>[-dirty]`, or
//! `boxmux <version> commit unknown (built without git metadata)`.

#[path = "../build/commit.rs"]
mod commit;

use std::fs;
use std::path::Path;
use std::process::Command;

/// The form README.md documents, with the version, the commit and `-dirty` captured.
const FORM: &str =
    r"^boxmux (\S+) commit (?:([0-9a-f]{40})(-dirty)?|unknown \(built without git metadata\))\n$";

/// Runs git on `dir` without the user's or the machine's configuration, and returns its
/// output.
fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["-c", "user.name=t", "-c", "user.email=t@example.com"])
        .args(["-c", "commit.gpgsign=false"])
        .args(args)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", dir.join(".no-global-config"))
        .output()
        .expect("git runs");
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

/// A repository with one commit holding `tracked`, and `.gitignore` ignoring `ignored`.
fn repository() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q"]);
    fs::write(dir.path().join("tracked"), "one").unwrap();
    fs::write(dir.path().join(".gitignore"), "ignored\n").unwrap();
    git(dir.path(), &["add", "tracked", ".gitignore"]);
    git(dir.path(), &["commit", "-q", "-m", "one"]);
    dir
}

#[test]
fn the_binary_prints_its_version_and_the_commit_of_the_tree_it_was_built_from() {
    let output = Command::new(env!("CARGO_BIN_EXE_boxmux"))
        .arg("--version")
        .output()
        .unwrap();
    assert!(output.status.success());
    let printed = String::from_utf8(output.stdout).unwrap();
    let parts = regex::Regex::new(FORM)
        .unwrap()
        .captures(&printed)
        .unwrap_or_else(|| panic!("--version printed {:?}, not the documented form", printed));
    assert_eq!(&parts[1], env!("CARGO_PKG_VERSION"));

    // The tree this test was built from, read with git directly.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    if root.join(".git").exists() {
        assert_eq!(
            parts.get(2).map(|m| m.as_str()),
            Some(git(root, &["rev-parse", "HEAD"]).as_str())
        );
        let dirty = !git(root, &["status", "--porcelain"]).is_empty();
        assert_eq!(parts.get(3).is_some(), dirty, "printed {:?}", printed);
    } else {
        assert_eq!(parts.get(2), None, "printed {:?}", printed);
    }
}

#[test]
fn a_tree_without_git_metadata_has_no_commit() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("Cargo.toml"), "").unwrap();
    let described = commit::describe(dir.path()).unwrap();
    assert_eq!(described.stamp, "unknown (built without git metadata)");
    assert!(described.watch.is_empty());
}

#[test]
fn a_tree_inside_another_repository_does_not_take_its_commit() {
    let repo = repository();
    let unpacked = repo.path().join("unpacked");
    fs::create_dir(&unpacked).unwrap();
    fs::write(unpacked.join("Cargo.toml"), "").unwrap();
    git(repo.path(), &["add", "unpacked"]);
    git(repo.path(), &["commit", "-q", "-m", "two"]);
    assert_eq!(commit::describe(&unpacked).unwrap().stamp, commit::UNKNOWN);
}

#[test]
fn a_clean_tree_is_its_full_commit() {
    let repo = repository();
    fs::write(repo.path().join("ignored"), "not part of the tree").unwrap();
    let described = commit::describe(repo.path()).unwrap();
    assert_eq!(described.stamp, git(repo.path(), &["rev-parse", "HEAD"]));

    let git_dir = repo.path().join(".git");
    let branch = git(repo.path(), &["symbolic-ref", "HEAD"]);
    for watched in [
        git_dir.join("HEAD"),
        git_dir.join("index"),
        git_dir.join(&branch),
        repo.path().join("tracked"),
        repo.path().join(".gitignore"),
    ] {
        assert!(
            described
                .watch
                .iter()
                .any(|path| fs::canonicalize(path).unwrap() == fs::canonicalize(&watched).unwrap()),
            "{} is not watched: {:?}",
            watched.display(),
            described.watch
        );
    }
    assert!(!described.watch.contains(&repo.path().join("ignored")));
}

#[test]
fn a_detached_head_is_its_full_commit() {
    let repo = repository();
    let sha = git(repo.path(), &["rev-parse", "HEAD"]);
    git(repo.path(), &["checkout", "-q", "--detach"]);
    assert_eq!(commit::describe(repo.path()).unwrap().stamp, sha);
}

#[test]
fn a_changed_tracked_file_makes_the_tree_dirty() {
    let repo = repository();
    fs::write(repo.path().join("tracked"), "two").unwrap();
    let sha = git(repo.path(), &["rev-parse", "HEAD"]);
    assert_eq!(
        commit::describe(repo.path()).unwrap().stamp,
        format!("{}-dirty", sha)
    );
}

#[test]
fn an_untracked_file_makes_the_tree_dirty() {
    let repo = repository();
    fs::write(repo.path().join("new"), "").unwrap();
    let described = commit::describe(repo.path()).unwrap();
    assert!(described.stamp.ends_with("-dirty"), "{}", described.stamp);
    assert!(described.watch.contains(&repo.path().join("new")));
}

#[test]
fn git_metadata_git_cannot_read_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join(".git"), "gitdir: nowhere\n").unwrap();
    let error = commit::describe(dir.path()).err().expect("an error");
    assert!(error.contains("git rev-parse"), "{}", error);
}
