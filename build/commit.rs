//! The commit a source tree holds, as `boxmux --version` reports it. The build script stamps it
//! into the binary; the tests run it on trees of their own.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

/// What a tree without git metadata of its own (a tarball, a crates.io download) reports in
/// place of a commit.
pub const UNKNOWN: &str = "unknown (built without git metadata)";

/// What `--version` says about the commit, and what can change it.
pub struct Commit {
    /// `<40-character commit>`, the same followed by `-dirty` when the tree has changes git has
    /// not committed (untracked files included, ignored ones not), or [`UNKNOWN`].
    pub stamp: String,
    /// The files and folders whose change can change the stamp.
    pub watch: Vec<PathBuf>,
}

/// Describes the tree at `root`. Only git metadata at `root` itself counts: a tree unpacked
/// inside some other repository has no commit of its own. A tree that has git metadata which
/// git cannot read is an error, never a guess.
pub fn describe(root: &Path) -> Result<Commit, String> {
    if !root.join(".git").exists() {
        return Ok(Commit {
            stamp: UNKNOWN.to_string(),
            watch: Vec::new(),
        });
    }
    let sha = git(root, &["rev-parse", "--verify", "HEAD^{commit}"])?;
    if sha.len() != 40 || !sha.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!(
            "git names the commit of {} as '{}', which is not a 40-character hash",
            root.display(),
            sha
        ));
    }
    let dirty = !git(root, &["status", "--porcelain", "--untracked-files=normal"])?.is_empty();

    let git_dir = PathBuf::from(git(
        root,
        &["rev-parse", "--path-format=absolute", "--git-dir"],
    )?);
    let common_dir = PathBuf::from(git(
        root,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?);
    let mut watch = vec![
        git_dir.join("HEAD"),
        git_dir.join("logs").join("HEAD"),
        git_dir.join("index"),
    ];
    // A detached HEAD names no branch, and is named `HEAD` here.
    let branch = git(root, &["rev-parse", "--symbolic-full-name", "HEAD"])?;
    if branch != "HEAD" {
        watch.push(common_dir.join(branch));
    }
    watch.push(common_dir.join("packed-refs"));
    // A missing file counts as changed on every build, so only the files there now are watched.
    // HEAD's reflog changes whenever HEAD moves, also when its branch has no file of its own yet.
    watch.retain(|path| path.exists());

    // Every top-level file and folder git tracks or would track: a change anywhere under them
    // can make the tree dirty or clean.
    let files = git_output(
        root,
        &[
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ],
    )?;
    let tops: BTreeSet<&str> = files
        .split('\0')
        .filter(|path| !path.is_empty())
        .map(|path| path.split('/').next().expect("split yields one item"))
        .collect();
    watch.extend(tops.into_iter().map(|top| root.join(top)));

    Ok(Commit {
        stamp: if dirty { format!("{}-dirty", sha) } else { sha },
        watch,
    })
}

/// Runs git on the tree at `root` and returns its output, trimmed.
fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    git_output(root, args).map(|out| out.trim().to_string())
}

/// Runs git on the tree at `root` and returns its output as it is.
fn git_output(root: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        // Set by git for its hooks; they would point git at another repository or index.
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_COMMON_DIR")
        .output()
        .map_err(|e| {
            format!(
                "{} is a git checkout, and reading its commit needs git: {}",
                root.display(),
                e
            )
        })?;
    if !output.status.success() {
        return Err(format!(
            "git {} failed in {}: {}",
            args.join(" "),
            root.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout).map_err(|e| {
        format!(
            "git {} wrote output that is not UTF-8: {}",
            args.join(" "),
            e
        )
    })
}
