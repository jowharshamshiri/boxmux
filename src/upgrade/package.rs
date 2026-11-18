//! How a released BoxMux was installed: by a package manager, or from its archive.
//!
//! A package (the Homebrew formula, the deb, the rpm) puts [`MARKER`] beside the `boxmux` it
//! installs. It names the package manager, the path the package's `boxmux` is reached by
//! whatever version is installed (Homebrew's `opt/boxmux`, not the versioned Cellar), and the
//! manager's command that upgrades it. Such a BoxMux is upgraded by its manager, never by
//! `boxmux upgrade` replacing a file the manager owns: the upgrade says the command instead. A
//! BoxMux without the marker was unpacked from its archive, and `boxmux upgrade` replaces it
//! where it is.
//!
//! The marker is the package's statement about the program beside it, so a marker that does
//! not describe that program, or cannot be read, is an error, never taken for an archive's
//! install.

use std::path::{Path, PathBuf};

/// The file a package puts beside the `boxmux` it installs.
pub const MARKER: &str = "boxmux-installed-by.json";
const FORMAT: u64 = 1;

/// A BoxMux a package manager installed, as its marker says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Package {
    /// The package manager, as people name it (`Homebrew`, `apt`, `dnf`).
    pub manager: String,
    /// The package's `boxmux`, by the path that stays the same across its versions.
    pub program: PathBuf,
    /// The command that upgrades it.
    pub update: String,
}

impl Package {
    /// What `boxmux upgrade` says instead of replacing what the manager owns.
    pub fn upgrade_with(&self) -> String {
        format!(
            "boxmux was installed by {}, which upgrades it: run `{}`",
            self.manager, self.update
        )
    }
}

/// The package that installed the program `boxmux` (already resolved to the file itself), if
/// one did: the marker beside it. An error when the marker is there but unreadable, malformed,
/// or about another program.
pub fn of(boxmux: &Path) -> Result<Option<Package>, String> {
    let marker = boxmux.with_file_name(MARKER);
    let text = match std::fs::read_to_string(&marker) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(format!(
                "cannot read {}, which says how boxmux was installed: {}",
                marker.display(),
                e
            ))
        }
    };
    let bad = |why: String| {
        format!(
            "{} does not say how boxmux was installed: {}; install the package again",
            marker.display(),
            why
        )
    };
    let value: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| bad(format!("it is not JSON ({})", e)))?;
    if value["format"] != serde_json::json!(FORMAT) {
        return Err(bad(format!(
            "its format is {}, not {}",
            value["format"], FORMAT
        )));
    }
    let field = |name: &str| {
        value[name]
            .as_str()
            .filter(|s| !s.trim().is_empty())
            .map(str::to_string)
            .ok_or_else(|| bad(format!("`{}` is missing", name)))
    };
    let package = Package {
        manager: field("manager")?,
        program: PathBuf::from(field("program")?),
        update: field("update")?,
    };
    if !package.program.is_absolute() {
        return Err(bad(format!(
            "`program` {} is not an absolute path",
            package.program.display()
        )));
    }
    if package.program.canonicalize().ok().as_deref() != Some(boxmux) {
        return Err(bad(format!(
            "it names {}, which is not the program beside it ({})",
            package.program.display(),
            boxmux.display()
        )));
    }
    Ok(Some(package))
}
