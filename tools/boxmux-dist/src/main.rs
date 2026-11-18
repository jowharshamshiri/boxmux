//! Builds BoxMux's release archives.
//!
//! An archive is `boxmux-<version>-<platform>.tar.gz` (`.zip` for Windows) and unpacks to one
//! directory, `boxmux/`, holding the `boxmux` program of that platform (`boxmux.exe` on
//! Windows), BoxMux's licence and its README, and nothing else.
//!
//! `boxmux-dist release --platform <platform> --out-dir <dir>` makes one archive, on a machine
//! of that platform, and prints its path. The build is given its release line, which BoxMux's
//! build bakes in and `boxmux upgrade` follows; an archive is never made without one.

// BoxMux's build script's own reading of the release line: this tool reads the variables as it
// does, and leaves the baking, and the name it bakes into, to it.
#[allow(dead_code)]
#[path = "../../../build/release_line.rs"]
mod release_line;

mod archive;
mod build;
mod platform;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

type Result<T> = std::result::Result<T, String>;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let done = match args.first().map(String::as_str) {
        Some("release") => {
            release_args(&args[1..]).and_then(|(platform, out_dir)| release(&platform, &out_dir))
        }
        _ => Err("usage: boxmux-dist release --platform <platform> --out-dir <dir>".into()),
    };
    match done {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("boxmux-dist: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn release_args(args: &[String]) -> Result<(String, PathBuf)> {
    let (mut platform, mut out_dir) = (None, None);
    let mut rest = args.iter();
    while let Some(flag) = rest.next() {
        let slot = match flag.as_str() {
            "--platform" => &mut platform,
            "--out-dir" => &mut out_dir,
            other => return Err(format!("unknown argument {:?}", other)),
        };
        if slot.is_some() {
            return Err(format!("{} is given twice", flag));
        }
        *slot = Some(
            rest.next()
                .ok_or_else(|| format!("{} takes a value", flag))?
                .clone(),
        );
    }
    Ok((
        platform.ok_or("release takes --platform")?,
        PathBuf::from(out_dir.ok_or("release takes --out-dir")?),
    ))
}

/// The repository: two levels above this tool, which lives in `tools/`.
fn repository() -> Result<PathBuf> {
    let here = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    std::fs::canonicalize(&here)
        .map_err(|e| format!("cannot resolve the repository at {}: {}", here.display(), e))
}

/// The release line this build is given, as BoxMux's build bakes it: an archive of a BoxMux
/// without one would install a BoxMux that cannot upgrade itself.
fn release_line(get: &dyn Fn(&str) -> Option<String>) -> Result<String> {
    release_line::read(get)?.ok_or_else(|| {
        format!(
            "a release archive is built with its release line ({}), and {} is not set: \
             a boxmux without one cannot upgrade itself",
            release_line::INPUTS.join(", "),
            release_line::CHANNEL
        )
    })
}

fn release(platform: &str, out_dir: &Path) -> Result<()> {
    let platform = platform::named(platform)?;
    platform::built_here(platform)?;
    let repo = repository()?;
    let version = build::version(&repo)?;
    // Before the build, which takes long: an archive without them is not made.
    let line = release_line(&|name| std::env::var(name).ok())?;
    let mut entries = archive::documents(&repo)?;
    entries.extend(archive::demos(&repo)?);
    let built = build::build(&repo, platform)?;
    build::check(&built, &version, &line)?;
    entries.push(archive::Entry {
        name: platform.program().to_string(),
        from: built.program.clone(),
        program: true,
    });
    std::fs::create_dir_all(out_dir)
        .map_err(|e| format!("cannot create {}: {}", out_dir.display(), e))?;
    let out = out_dir.join(archive::name(&version, platform.id));
    archive::write(&entries, platform.id, &out)?;
    println!("{}", out.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A release line is whole, in the form BoxMux's build bakes it, or no archive is made.
    #[test]
    fn an_archive_is_made_only_with_a_whole_release_line() {
        let e = release_line(&|_| None).unwrap_err();
        assert!(
            e.contains("is not set: a boxmux without one cannot upgrade itself"),
            "{}",
            e
        );
        let e =
            release_line(&|name| (name == release_line::CHANNEL).then(|| "release".to_string()))
                .unwrap_err();
        assert!(e.contains("this one is not given"), "{}", e);
    }

    #[test]
    fn release_takes_a_platform_and_an_out_dir_once_each() {
        let args = |list: &[&str]| list.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(
            release_args(&args(&["--platform", "linux-x86_64", "--out-dir", "o"])).unwrap(),
            ("linux-x86_64".to_string(), PathBuf::from("o"))
        );
        assert_eq!(
            release_args(&args(&["--platform", "a", "--platform", "b"])).unwrap_err(),
            "--platform is given twice"
        );
        assert_eq!(
            release_args(&args(&["--out-dir", "o"])).unwrap_err(),
            "release takes --platform"
        );
        assert_eq!(
            release_args(&args(&["--platform"])).unwrap_err(),
            "--platform takes a value"
        );
        assert_eq!(
            release_args(&args(&["--jobs", "2"])).unwrap_err(),
            "unknown argument \"--jobs\""
        );
    }
}
