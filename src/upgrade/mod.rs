//! `boxmux upgrade`: installs the newest release of the channel this BoxMux was published on,
//! for the platform it runs on, in place of the running program.
//!
//! The release is found and verified as [`release`] says: the registry's charter, then the
//! signed manifest, then the archive's size, SHA-256 and layout. The program it holds must say
//! it is the version the manifest published before it replaces anything. It replaces the
//! running `boxmux` where that file is, in one rename, so the program there is always one
//! whole BoxMux, the old or the new. A BoxMux a package manager installed is left to that
//! manager ([`package`]), and a BoxMux built from source has no release line to follow.

pub mod package;
pub mod release;

use std::path::{Path, PathBuf};
use std::process::Command;

use release::{Archive, Line, Problem, Release, Transport};

/// What `boxmux upgrade` did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The channel has published nothing for this platform yet.
    NothingPublished {
        channel: String,
        platform: String,
        installed: String,
    },
    /// The newest release is no later than the one running: nothing is installed, and never an
    /// older one.
    UpToDate {
        channel: String,
        platform: String,
        installed: String,
        newest: String,
    },
    /// The newest release was installed at `program`, in place of `from`.
    Installed {
        channel: String,
        platform: String,
        from: String,
        to: String,
        program: PathBuf,
    },
}

impl std::fmt::Display for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Outcome::NothingPublished {
                channel,
                platform,
                installed,
            } => write!(
                f,
                "no boxmux is published on the {} channel for {} yet; boxmux {} stays",
                channel, platform, installed
            ),
            Outcome::UpToDate {
                channel,
                platform,
                installed,
                newest,
            } => write!(
                f,
                "boxmux {} is the newest release on the {} channel for {}; boxmux {} is up to date",
                newest, channel, platform, installed
            ),
            Outcome::Installed {
                channel,
                platform,
                from,
                to,
                program,
            } => write!(
                f,
                "boxmux {} is the newest release on the {} channel for {}: installed it at {} in place of boxmux {}",
                to,
                channel,
                platform,
                program.display(),
                from
            ),
        }
    }
}

/// Upgrades the running BoxMux along the release line it was built with, over HTTPS.
pub fn upgrade() -> Result<Outcome, String> {
    let line = release::baked().ok_or(release::NO_LINE)?;
    let platform = release::platform().ok_or_else(|| {
        format!(
            "boxmux is published for no {} {}",
            std::env::consts::OS,
            std::env::consts::ARCH
        )
    })?;
    let running =
        std::env::current_exe().map_err(|e| format!("cannot tell where this boxmux is: {}", e))?;
    let program = running
        .canonicalize()
        .map_err(|e| format!("cannot resolve {}: {}", running.display(), e))?;
    upgrade_with(
        &release::Https::new(),
        &line,
        platform,
        mg_srel::unix_now(),
        &program,
        env!("CARGO_PKG_VERSION"),
    )
}

/// Upgrades the BoxMux `installed` at `program` (the file itself, resolved) along `line`,
/// reached through `transport`, to the newest release for `platform` at the time `now`.
pub fn upgrade_with(
    transport: &dyn Transport,
    line: &Line,
    platform: &str,
    now: u64,
    program: &Path,
    installed: &str,
) -> Result<Outcome, String> {
    let Some(newest) = release::fetch_latest(transport, line, platform, now)? else {
        return Ok(Outcome::NothingPublished {
            channel: line.channel.clone(),
            platform: platform.to_string(),
            installed: installed.to_string(),
        });
    };
    if !release::later(&newest.version, installed)? {
        return Ok(Outcome::UpToDate {
            channel: line.channel.clone(),
            platform: platform.to_string(),
            installed: installed.to_string(),
            newest: newest.version,
        });
    }
    if let Some(package) = package::of(program)? {
        return Err(format!(
            "boxmux {} is the newest release on the {} channel for {}, and {}",
            newest.version,
            line.channel,
            platform,
            package.upgrade_with()
        ));
    }
    let directory = program
        .parent()
        .ok_or_else(|| format!("{} is in no directory", program.display()))?;
    let staging = Staging::new(directory)?;
    let unpacked = take(transport, &newest, staging.path())?;
    put_in_place(&unpacked, program)?;
    Ok(Outcome::Installed {
        channel: line.channel.clone(),
        platform: platform.to_string(),
        from: installed.to_string(),
        to: newest.version,
        program: program.to_path_buf(),
    })
}

/// A folder beside the program for one upgrade's download, on the same file system so the new
/// program moves into place in one rename; removed with everything in it when the upgrade
/// ends, however it ends.
struct Staging(PathBuf);

impl Staging {
    fn new(directory: &Path) -> Result<Staging, String> {
        let path = directory.join(format!(".boxmux-upgrade-{}", std::process::id()));
        std::fs::create_dir(&path).map_err(|e| {
            format!(
                "cannot write in {}, where boxmux is installed: {}",
                directory.display(),
                e
            )
        })?;
        Ok(Staging(path))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Staging {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_dir_all(&self.0) {
            eprintln!(
                "boxmux: could not remove {}, which the upgrade downloaded into: {}",
                self.0.display(),
                e
            );
        }
    }
}

/// Downloads and verifies `release`'s archive into `staging`, unpacks it, and answers the
/// program it holds, once that program says it is the release's version.
fn take(transport: &dyn Transport, release: &Release, staging: &Path) -> Result<PathBuf, Problem> {
    let archive = release::download(transport, release, staging)?;
    let root = release::unpack(
        &archive,
        Archive::of(&release.platform),
        &staging.join("unpacked"),
    )?;
    let name = release::program_name(&release.platform);
    let program = root.join(name);
    if !program.is_file() {
        return Err(Problem::Refused {
            reason: "archive_layout",
            message: format!(
                "{} holds no {}/{}",
                release.url,
                release::ARCHIVE_ROOT,
                name
            ),
        });
    }
    let says = says_version(&program)?;
    let expected = format!("boxmux {} ", release.version);
    if !says.starts_with(&expected) {
        return Err(Problem::Refused {
            reason: "archive_version",
            message: format!(
                "the boxmux in {} says {:?}, and the signed manifest publishes it as boxmux {}",
                release.url, says, release.version
            ),
        });
    }
    Ok(program)
}

/// The first line `<program> --version` prints.
fn says_version(program: &Path) -> Result<String, Problem> {
    let output = Command::new(program)
        .arg("--version")
        .output()
        .map_err(|e| Problem::Failed(format!("cannot run {}: {}", program.display(), e)))?;
    if !output.status.success() {
        return Err(Problem::Failed(format!(
            "{} --version failed ({}): {}",
            program.display(),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_string())
}

/// Moves `new` over `program` in one rename: the file at `program` is the old BoxMux until the
/// rename and the new one after it.
#[cfg(unix)]
fn put_in_place(new: &Path, program: &Path) -> Result<(), String> {
    std::fs::rename(new, program).map_err(|e| {
        format!(
            "cannot put the new boxmux in place at {}: {}",
            program.display(),
            e
        )
    })
}

/// Windows keeps a running program's file open and will not replace it, but lets it be
/// renamed: the running BoxMux moves aside to `boxmux.exe.old`, which the next upgrade
/// removes, and the new one takes its name. A failure to move the new one in moves the old
/// one back.
#[cfg(windows)]
fn put_in_place(new: &Path, program: &Path) -> Result<(), String> {
    let aside = program.with_extension("exe.old");
    match std::fs::remove_file(&aside) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => {
            return Err(format!(
                "cannot remove {}, which an earlier upgrade left there (is that boxmux still running?): {}",
                aside.display(),
                e
            ))
        }
    }
    std::fs::rename(program, &aside).map_err(|e| {
        format!(
            "cannot move {} aside to {}: {}",
            program.display(),
            aside.display(),
            e
        )
    })?;
    if let Err(e) = std::fs::rename(new, program) {
        let back = std::fs::rename(&aside, program)
            .map(|()| "the old boxmux is back in place".to_string())
            .unwrap_or_else(|back| {
                format!(
                    "and the old boxmux could not be moved back from {}: {}",
                    aside.display(),
                    back
                )
            });
        return Err(format!(
            "cannot put the new boxmux in place at {}: {}; {}",
            program.display(),
            e,
            back
        ));
    }
    Ok(())
}

#[cfg(test)]
mod testing;

#[cfg(test)]
mod tests;
