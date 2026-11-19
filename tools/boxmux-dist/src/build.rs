//! Building the program, and checking what was built before it is archived.

use crate::platform::Platform;
use crate::Result;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The program of a release: the binary of the package of the same name.
pub const PROGRAM: &str = "boxmux";

/// What a build produced, and the paths of the machine it was built on, which the program may
/// not carry.
pub struct Built {
    pub program: PathBuf,
    pub machine_paths: Vec<PathBuf>,
}

fn run(cmd: &mut Command) -> Result<Vec<u8>> {
    let shown = format!("{:?}", cmd);
    let out = cmd
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| format!("cannot run {}: {}", shown, e))?;
    if !out.status.success() {
        return Err(format!("{} failed ({})", shown, out.status));
    }
    Ok(out.stdout)
}

fn metadata(repo: &Path, deps: bool) -> Result<serde_json::Value> {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(repo)
        .args(["metadata", "--format-version", "1", "--locked"]);
    if !deps {
        cmd.arg("--no-deps");
    }
    serde_json::from_slice(&run(&mut cmd)?).map_err(|e| format!("cargo metadata: {}", e))
}

/// The version being released: the repository's `version.txt`, which the package must carry,
/// since `boxmux --version` reports its package's version.
pub fn version(repo: &Path) -> Result<String> {
    let file = repo.join("version.txt");
    let version = std::fs::read_to_string(&file)
        .map_err(|e| format!("cannot read {}: {}", file.display(), e))?
        .trim()
        .to_owned();
    package_agrees(&metadata(repo, false)?, &version)?;
    Ok(version)
}

fn package_agrees(meta: &serde_json::Value, version: &str) -> Result<()> {
    let packages = meta["packages"]
        .as_array()
        .ok_or("cargo metadata lists no packages")?;
    let package = packages
        .iter()
        .find(|p| p["name"] == PROGRAM)
        .ok_or_else(|| format!("the workspace has no package {}", PROGRAM))?;
    let carried = package["version"].as_str().unwrap_or_default();
    if carried != version {
        return Err(format!(
            "version.txt says {} and the package {} says {}: a release has one version",
            version, PROGRAM, carried
        ));
    }
    Ok(())
}

/// Cargo's home, as the dependency graph reads crates from it: the part of a registry or git
/// dependency's manifest path before its `registry` or `git` folder.
fn cargo_home(meta: &serde_json::Value) -> Option<PathBuf> {
    meta["packages"].as_array()?.iter().find_map(|p| {
        let manifest = p["manifest_path"].as_str()?;
        [
            "/registry/",
            "\\registry\\",
            "/git/checkouts/",
            "\\git\\checkouts\\",
        ]
        .iter()
        .find_map(|marker| {
            manifest
                .find(marker)
                .map(|at| PathBuf::from(&manifest[..at]))
        })
    })
}

/// The flags that replace this machine's paths, wherever rustc writes one into the program (a
/// panic's location, a file included at build time), with fixed ones. Rustc applies the last
/// prefix that matches, so the most specific comes last.
fn remaps(pairs: &[(PathBuf, &str)]) -> String {
    pairs
        .iter()
        .map(|(from, to)| format!("--remap-path-prefix={}={}", from.display(), to))
        .collect::<Vec<_>>()
        .join("\u{1f}")
}

/// The flags that link `platform`'s program, with the toolchain at `sysroot`. A Linux build
/// links with the rust-lld that toolchain ships, through cc as rustc links when it picks rust-lld
/// itself. rustc picks it for x86_64 Linux and not for arm64 Linux, where it links with GNU ld,
/// and the two linkers differ in what the programs then need of glibc: std references
/// `pidfd_spawnp` and `pidfd_getpid` (glibc 2.39) weakly, and runs without them where glibc has
/// none. rust-lld makes a version needed only by weak references a weak need, which an older
/// glibc's loader passes over; GNU ld makes it a hard need, and that loader refuses to start the
/// program.
fn link_flags(platform: &Platform, sysroot: &Path) -> Result<Vec<String>> {
    if !platform.target.ends_with("-linux-gnu") {
        return Ok(Vec::new());
    }
    // Where rustc looks for its own linker: the host's tools, and each platform is built on a
    // machine of its own target.
    let gcc_ld = sysroot
        .join("lib")
        .join("rustlib")
        .join(platform.target)
        .join("bin")
        .join("gcc-ld");
    let lld = gcc_ld.join("ld.lld");
    if !lld.is_file() {
        return Err(format!(
            "{} is not there: a Linux release links with the rust-lld its toolchain ships, and the toolchain at {} ships none",
            lld.display(),
            sysroot.display()
        ));
    }
    Ok(vec![
        "-Clink-arg=-fuse-ld=lld".to_owned(),
        format!("-Clink-arg=-B{}", gcc_ld.display()),
    ])
}

/// Builds the program for `platform`, on this machine.
pub fn build(repo: &Path, platform: &Platform) -> Result<Built> {
    let meta = metadata(repo, true)?;
    let target_dir = PathBuf::from(
        meta["target_directory"]
            .as_str()
            .ok_or("cargo metadata names no target directory")?,
    );
    let sysroot = PathBuf::from(
        String::from_utf8_lossy(&run(Command::new("rustc").args(["--print", "sysroot"]))?).trim(),
    );
    let link = link_flags(platform, &sysroot)?;
    let mut pairs: Vec<(PathBuf, &str)> = Vec::new();
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
    if let Some(home) = home.map(PathBuf::from).filter(|h| h.parent().is_some()) {
        pairs.push((home, "/home"));
    }
    if let Some(home) = cargo_home(&meta) {
        pairs.push((home, "/cargo"));
    }
    pairs.extend([
        (repo.to_path_buf(), "/boxmux"),
        (target_dir, "/build"),
        (sysroot, "/rustc"),
    ]);

    let rustflags: Vec<String> = std::iter::once(remaps(&pairs)).chain(link).collect();

    let mut cmd = Command::new("cargo");
    cmd.current_dir(repo)
        .env("CARGO_ENCODED_RUSTFLAGS", rustflags.join("\u{1f}"))
        .args([
            "build",
            "--locked",
            "--release",
            "--target",
            platform.target,
            "-p",
            PROGRAM,
            "--bin",
            PROGRAM,
            "--message-format=json-render-diagnostics",
        ]);
    if let Some(floor) = platform.macos_deployment_target {
        cmd.env("MACOSX_DEPLOYMENT_TARGET", floor);
    }
    let shown = format!("{:?}", cmd);
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("cannot run {}: {}", shown, e))?;
    let stdout = child.stdout.take().expect("piped");
    let mut program = None;
    for line in std::io::BufReader::new(stdout).lines() {
        let line = line.map_err(|e| format!("reading cargo's messages: {}", e))?;
        if let Some(path) = executable(&line)? {
            program = Some(path);
        }
    }
    let status = child
        .wait()
        .map_err(|e| format!("waiting for {}: {}", shown, e))?;
    if !status.success() {
        return Err(format!("{} failed ({})", shown, status));
    }
    let program = program.ok_or_else(|| {
        format!(
            "cargo built no {}: its messages name no such executable",
            PROGRAM
        )
    })?;
    Ok(Built {
        program,
        machine_paths: pairs.into_iter().map(|(from, _)| from).collect(),
    })
}

/// The program a line of cargo's messages says was built, if it is [`PROGRAM`].
fn executable(line: &str) -> Result<Option<PathBuf>> {
    if !line.starts_with('{') {
        return Ok(None);
    }
    let message: serde_json::Value = serde_json::from_str(line)
        .map_err(|e| format!("cargo wrote a message that is not JSON ({}): {}", e, line))?;
    let is_bin = message["target"]["kind"]
        .as_array()
        .is_some_and(|k| k.iter().any(|k| k == "bin"));
    if message["reason"] != "compiler-artifact" || !is_bin || message["target"]["name"] != PROGRAM {
        return Ok(None);
    }
    let path = message["executable"]
        .as_str()
        .ok_or_else(|| format!("cargo built {} and names no executable for it", PROGRAM))?;
    Ok(Some(PathBuf::from(path)))
}

/// Holds the program to what a release is: it carries none of the build machine's paths,
/// carries the release line it was given, and says it is this version when run on a machine
/// that knows nothing of BoxMux.
pub fn check(built: &Built, version: &str, line: &str) -> Result<()> {
    let path = &built.program;
    let bytes =
        std::fs::read(path).map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
    if let Some(found) = carried(&bytes, &built.machine_paths) {
        return Err(format!(
            "{} carries the build machine's path {}: it was built without its paths remapped",
            path.display(),
            found.display()
        ));
    }
    if !bytes.windows(line.len()).any(|w| w == line.as_bytes()) {
        return Err(format!(
            "{} was built without the release line it was given ({})",
            path.display(),
            line
        ));
    }
    let says = says_version(path)?;
    let expected = format!("{} {} ", PROGRAM, version);
    if !says.starts_with(&expected) {
        return Err(format!(
            "{} says {:?}, and the release is {} {}",
            path.display(),
            says,
            PROGRAM,
            version
        ));
    }
    Ok(())
}

/// The first of `paths`, followed by a separator, that `bytes` contains.
fn carried<'a>(bytes: &[u8], paths: &'a [PathBuf]) -> Option<&'a PathBuf> {
    paths.iter().find(|path| {
        let needle = format!("{}{}", path.display(), std::path::MAIN_SEPARATOR);
        bytes.windows(needle.len()).any(|w| w == needle.as_bytes())
    })
}

/// The first line `<program> --version` prints, run with nothing of this machine's
/// environment but its system's path.
fn says_version(program: &Path) -> Result<String> {
    let mut cmd = Command::new(program);
    cmd.arg("--version").env_clear();
    for kept in ["PATH", "SystemRoot"] {
        if let Some(value) = std::env::var_os(kept) {
            cmd.env(kept, value);
        }
    }
    let out = run(&mut cmd)?;
    Ok(String::from_utf8_lossy(&out)
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or_default()
        .trim()
        .to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_programs_executable_is_taken_from_cargos_messages() {
        let artifact = |name: &str, kind: &str, exe: &str| {
            serde_json::json!({
                "reason": "compiler-artifact",
                "target": { "name": name, "kind": [kind] },
                "executable": exe,
            })
            .to_string()
        };
        assert_eq!(
            executable(&artifact("boxmux", "bin", "/b/boxmux")).unwrap(),
            Some(PathBuf::from("/b/boxmux"))
        );
        assert_eq!(
            executable(&artifact("boxmux", "lib", "/b/libboxmux.rlib")).unwrap(),
            None
        );
        assert_eq!(
            executable(&artifact("boxmux-dist", "bin", "/b/boxmux-dist")).unwrap(),
            None
        );
        assert_eq!(
            executable(r#"{"reason":"build-finished","success":true}"#).unwrap(),
            None
        );
        assert_eq!(executable("   Compiling boxmux v1.0.0").unwrap(), None);
        let none = serde_json::json!({
            "reason": "compiler-artifact",
            "target": { "name": "boxmux", "kind": ["bin"] },
            "executable": null,
        });
        assert!(executable(&none.to_string())
            .unwrap_err()
            .contains("names no executable"));
    }

    #[test]
    fn a_program_carrying_a_build_machine_path_is_caught() {
        let paths = [
            PathBuf::from("/home/someone"),
            PathBuf::from("/home/someone/ws/boxmux"),
        ];
        let program = b"\x7fELF...panicked at /home/someone/ws/boxmux/src/lib.rs:12...";
        assert_eq!(carried(program, &paths), Some(&paths[0]));
        assert_eq!(
            carried(
                b"panicked at /boxmux/src/lib.rs:12 /cargo/registry/x",
                &paths
            ),
            None
        );
        assert_eq!(carried(b"/home/someoneelse/x", &paths), None);
    }

    #[test]
    fn remapping_lists_the_most_specific_path_last() {
        let flags = remaps(&[
            (PathBuf::from("/home/u"), "/home"),
            (PathBuf::from("/home/u/repo"), "/boxmux"),
        ]);
        assert_eq!(
            flags,
            "--remap-path-prefix=/home/u=/home\u{1f}--remap-path-prefix=/home/u/repo=/boxmux"
        );
    }

    #[test]
    fn cargos_home_is_read_from_the_graph() {
        let meta = serde_json::json!({ "packages": [
            { "manifest_path": "/w/boxmux/Cargo.toml" },
            { "manifest_path": "/h/.qrust/cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tar-0.4.46/Cargo.toml" },
        ]});
        assert_eq!(cargo_home(&meta), Some(PathBuf::from("/h/.qrust/cargo")));
    }

    #[test]
    fn a_package_with_another_version_than_version_txt_is_refused() {
        let meta =
            |v: &str| serde_json::json!({ "packages": [{ "name": "boxmux", "version": v }] });
        assert!(package_agrees(&meta("1.2.3"), "1.2.3").is_ok());
        assert_eq!(
            package_agrees(&meta("1.2.2"), "1.2.3").unwrap_err(),
            "version.txt says 1.2.3 and the package boxmux says 1.2.2: a release has one version"
        );
    }

    /// The repository's own version.txt is the version its package carries.
    #[test]
    fn the_repositorys_version_is_its_packages() {
        let repo = crate::repository().unwrap();
        let written = std::fs::read_to_string(repo.join("version.txt")).unwrap();
        assert_eq!(version(&repo).unwrap(), written.trim());
    }

    /// A program is held to the release line it was given and the version it says.
    #[cfg(unix)]
    #[test]
    fn a_program_is_held_to_its_line_and_its_version() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let program = dir.path().join("boxmux");
        let line = "https://r.example release prod a,b,c d,e,f";
        // Carries its release line as a released boxmux does, in its bytes.
        std::fs::write(
            &program,
            format!(
                "#!/bin/sh\n# {}\nprintf 'boxmux 1.2.3 commit unknown (built without git metadata)\\n'\n",
                line
            ),
        )
        .unwrap();
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).unwrap();
        let built = Built {
            program: program.clone(),
            machine_paths: vec![],
        };
        assert!(check(&built, "1.2.3", line).is_ok());
        assert_eq!(
            check(&built, "1.2.4", line).unwrap_err(),
            format!(
                "{} says \"boxmux 1.2.3 commit unknown (built without git metadata)\", and the release is boxmux 1.2.4",
                program.display()
            )
        );
        let other = "https://r.example nightly prod a,b,c d,e,f";
        assert_eq!(
            check(&built, "1.2.3", other).unwrap_err(),
            format!(
                "{} was built without the release line it was given ({})",
                program.display(),
                other
            )
        );
    }

    /// The version needs of the ELF program at `path`, as `readelf` reads them: each version of a
    /// library the program needs, and whether that need is weak.
    #[cfg(target_os = "linux")]
    fn version_needs(path: &Path) -> Vec<(String, String, bool)> {
        let out = run(Command::new("readelf")
            .args(["--version-info", "--wide"])
            .arg(path))
        .unwrap();
        let text = String::from_utf8(out).unwrap();
        let section = text
            .split("Version needs section")
            .nth(1)
            .unwrap_or_else(|| panic!("{} has no version needs:\n{}", path.display(), text));
        let mut file = String::new();
        let mut needs = Vec::new();
        for line in section.lines().skip(1) {
            if line.trim().is_empty() {
                break;
            }
            let field = |name: &str| {
                line.split(name)
                    .nth(1)
                    .and_then(|rest| rest.split_whitespace().next())
                    .map(str::to_owned)
            };
            if let Some(f) = field("File: ") {
                file = f;
            } else if let (Some(name), Some(flags)) = (field("Name: "), field("Flags: ")) {
                needs.push((
                    file.clone(),
                    name,
                    flags.split('|').any(|f| f.trim() == "WEAK"),
                ));
            }
        }
        needs
    }

    /// A Linux release's program needs glibc's newer versions weakly where it references them
    /// weakly, linked as rustc links on an arm64 build machine.
    #[cfg(target_os = "linux")]
    #[test]
    fn a_linux_release_needs_weakly_referenced_glibc_versions_weakly() {
        let here =
            boxmux_lib::upgrade::release::platform().expect("the tests run on a released platform");
        let platform = crate::platform::named(here).unwrap();
        let sysroot = PathBuf::from(
            String::from_utf8_lossy(
                &run(Command::new("rustc").args(["--print", "sysroot"])).unwrap(),
            )
            .trim(),
        );
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("probe.rs");
        // Starting a program takes std through pidfd_spawnp and pidfd_getpid, which it references
        // weakly and which came in glibc 2.39, after the floor a release keeps to.
        std::fs::write(
            &source,
            "fn main() { std::process::Command::new(\"true\").status().unwrap(); }\n",
        )
        .unwrap();
        let probe = dir.path().join("probe");
        let mut rustc = Command::new("rustc");
        rustc
            .args(["--edition", "2021", "-O", "--target", platform.target])
            .arg(&source)
            .arg("-o")
            .arg(&probe);
        // How rustc links an arm64 Linux program unless told otherwise, on any machine: through cc
        // and GNU ld. rustc for x86_64 Linux links with rust-lld by default.
        rustc.args(["-C", "linker-features=-lld"]);
        rustc.args(link_flags(platform, &sysroot).unwrap());
        run(&mut rustc).unwrap();

        let needs = version_needs(&probe);
        let pidfd = needs
            .iter()
            .find(|(file, name, _)| file == "libc.so.6" && name == "GLIBC_2.39")
            .unwrap_or_else(|| {
                panic!(
                    "the probe needs no GLIBC_2.39 of libc.so.6, so this machine's glibc is older than 2.39 and the test shows nothing on it: {:?}",
                    needs
                )
            });
        assert!(
            pidfd.2,
            "the probe needs libc.so.6's GLIBC_2.39 outright, which a glibc older than 2.39 refuses to start, though it references that version weakly alone: {:?}",
            needs
        );

        let bare = dir.path().join("sysroot");
        let lld = bare
            .join("lib")
            .join("rustlib")
            .join(platform.target)
            .join("bin")
            .join("gcc-ld")
            .join("ld.lld");
        let e = link_flags(platform, &bare).unwrap_err();
        assert!(e.contains(&lld.display().to_string()), "{}", e);
    }
}
