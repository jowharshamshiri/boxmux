//! The archive: one per platform, a `.tar.gz` or, for Windows, a `.zip`, unpacking to
//! `boxmux/`.
//!
//! The archive is written entry by entry from the files named, never by archiving a folder: it
//! holds exactly the directory, the program and the documents, with fixed modes and times and
//! no owner, so no file a machine keeps beside them (macOS's `._` AppleDouble files and
//! `.DS_Store`, an editor's backup) and none of their extended attributes can reach a release.

use crate::Result;
use boxmux_lib::upgrade::release::{self, Archive, ARCHIVE_ROOT};
use std::io::Write as _;
use std::path::{Path, PathBuf};

/// The documents every archive carries from the repository's root.
pub const DOCUMENTS: [&str; 2] = ["LICENSE", "README.md"];

/// The configurations BoxMux ships as demos, which every archive carries from the repository
/// in folders of the same name: each folder's files of these kinds, not its subfolders. A
/// script is carried as a program.
pub const DEMOS: [(&str, &[&str]); 2] = [("layouts", &["yaml"]), ("examples", &["yaml", "sh"])];

pub fn name(version: &str, platform: &str) -> String {
    release::archive_name(version, platform)
}

/// One file of an archive: its name in `boxmux/`, where it is, and whether it is a program.
pub struct Entry {
    pub name: String,
    pub from: PathBuf,
    pub program: bool,
}

/// The documents, each of which the repository must hold: a release without its licence is not
/// made.
pub fn documents(repo: &Path) -> Result<Vec<Entry>> {
    let missing: Vec<&str> = DOCUMENTS
        .iter()
        .copied()
        .filter(|d| !repo.join(d).is_file())
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "a release archive carries {}, and {} holds no {}",
            DOCUMENTS.join(", "),
            repo.display(),
            missing.join(" or ")
        ));
    }
    Ok(DOCUMENTS
        .iter()
        .map(|d| Entry {
            name: d.to_string(),
            from: repo.join(d),
            program: false,
        })
        .collect())
}

/// The demo configurations of [`DEMOS`] in the repository, each folder's in name order. A
/// folder the repository lacks, or one holding none, is refused: a release without them is not
/// made.
pub fn demos(repo: &Path) -> Result<Vec<Entry>> {
    let mut entries = Vec::new();
    for (folder, kinds) in DEMOS {
        let dir = repo.join(folder);
        let listed = std::fs::read_dir(&dir).map_err(|e| {
            format!(
                "a release archive carries the demo configurations in {}/, and {} cannot be read: {}",
                folder,
                dir.display(),
                e
            )
        })?;
        let mut names = Vec::new();
        for entry in listed {
            let entry = entry.map_err(|e| format!("cannot read {}: {}", dir.display(), e))?;
            let path = entry.path();
            let kind = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or_default();
            if path.is_file() && kinds.contains(&kind) {
                names.push((
                    entry.file_name().to_string_lossy().into_owned(),
                    kind == "sh",
                ));
            }
        }
        if names.is_empty() {
            return Err(format!(
                "a release archive carries the demo configurations in {}/, and {} holds no {}",
                folder,
                dir.display(),
                kinds
                    .iter()
                    .map(|k| format!("*.{}", k))
                    .collect::<Vec<_>>()
                    .join(" or ")
            ));
        }
        names.sort();
        entries.extend(names.into_iter().map(|(name, program)| Entry {
            from: dir.join(&name),
            name: format!("{}/{}", folder, name),
            program,
        }));
    }
    Ok(entries)
}

/// One member of an archive: a directory, or a file with its mode and bytes.
enum Member {
    Directory(String),
    File(String, bool, Vec<u8>),
}

/// Writes the archive of `entries` for `platform` to `out`, as the updater reads that
/// platform's: the one directory, each folder an entry names, and each entry, in name order.
/// The same files make the same bytes: no times, no owner; a directory and a program are 0755
/// and a document 0644, whatever the files' own modes.
pub fn write(entries: &[Entry], platform: &str, out: &Path) -> Result<()> {
    let mut sorted: Vec<&Entry> = entries.iter().collect();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));
    if let Some(twice) = sorted.windows(2).find(|w| w[0].name == w[1].name) {
        return Err(format!("{} is in the archive twice", twice[0].name));
    }
    if let Some(entry) = sorted
        .iter()
        .find(|e| release::misplaced(&Path::new(ARCHIVE_ROOT).join(&e.name)).is_some())
    {
        return Err(format!("{} is no name a release archive holds", entry.name));
    }
    let mut members: Vec<(String, Member)> = vec![(
        format!("{}/", ARCHIVE_ROOT),
        Member::Directory(format!("{}/", ARCHIVE_ROOT)),
    )];
    let mut folders = std::collections::BTreeSet::new();
    for entry in sorted {
        let path = format!("{}/{}", ARCHIVE_ROOT, entry.name);
        let mut parent = Path::new(&path).parent();
        while let Some(folder) = parent.filter(|p| *p != Path::new(ARCHIVE_ROOT)) {
            folders.insert(format!("{}/", folder.to_string_lossy()));
            parent = folder.parent();
        }
        let bytes = std::fs::read(&entry.from)
            .map_err(|e| format!("cannot read {}: {}", entry.from.display(), e))?;
        members.push((path.clone(), Member::File(path, entry.program, bytes)));
    }
    members.extend(
        folders
            .into_iter()
            .map(|f| (f.clone(), Member::Directory(f))),
    );
    members.sort_by(|a, b| a.0.cmp(&b.0));
    let members: Vec<Member> = members.into_iter().map(|(_, m)| m).collect();
    let file = std::fs::File::create(out)
        .map_err(|e| format!("cannot create {}: {}", out.display(), e))?;
    let fail = |e: std::io::Error| format!("cannot write {}: {}", out.display(), e);
    match Archive::of(platform) {
        Archive::TarGz => tar_gz(file, &members).map_err(fail),
        Archive::Zip => {
            zip(file, &members).map_err(|e| format!("cannot write {}: {}", out.display(), e))
        }
    }
}

fn mode(program: bool) -> u32 {
    if program {
        0o755
    } else {
        0o644
    }
}

fn tar_gz(file: std::fs::File, members: &[Member]) -> std::io::Result<()> {
    let gz = flate2::GzBuilder::new()
        .mtime(0)
        .write(file, flate2::Compression::best());
    let mut b = tar::Builder::new(gz);
    let header = |mode: u32, size: u64, kind: tar::EntryType| {
        let mut h = tar::Header::new_gnu();
        h.set_entry_type(kind);
        h.set_mode(mode);
        h.set_size(size);
        h.set_mtime(0);
        h.set_uid(0);
        h.set_gid(0);
        h
    };
    for member in members {
        match member {
            Member::Directory(path) => b.append_data(
                &mut header(0o755, 0, tar::EntryType::Directory),
                path,
                std::io::empty(),
            )?,
            Member::File(path, program, bytes) => b.append_data(
                &mut header(mode(*program), bytes.len() as u64, tar::EntryType::Regular),
                path,
                bytes.as_slice(),
            )?,
        }
    }
    b.into_inner()?.finish().map(|_| ())
}

fn zip(file: std::fs::File, members: &[Member]) -> zip::result::ZipResult<()> {
    let mut writer = zip::ZipWriter::new(file);
    // Every entry is dated zip's own epoch, so the same files make the same bytes.
    let options = |mode: u32| {
        zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .last_modified_time(zip::DateTime::default())
            .unix_permissions(mode)
    };
    for member in members {
        match member {
            Member::Directory(path) => writer.add_directory(path.as_str(), options(0o755))?,
            Member::File(path, program, bytes) => {
                writer.start_file(path.as_str(), options(mode(*program)))?;
                writer.write_all(bytes)?;
            }
        }
    }
    writer.finish().map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    /// A repository as a build on a Mac leaves it, and the archive's entries taken from it: the
    /// program, the documents and the demo configurations, and beside them the AppleDouble
    /// files and `.DS_Store` macOS writes, a demo folder's subfolder and a file of another kind,
    /// none of which an archive takes.
    fn staged(dir: &Path, program: &str) -> Vec<Entry> {
        let write = |path: &str, body: &str| {
            let at = dir.join(path);
            std::fs::create_dir_all(at.parent().unwrap()).unwrap();
            std::fs::write(at, body).unwrap();
        };
        for (name, body) in [
            (program, "the program"),
            ("LICENSE", "licence"),
            ("README.md", "readme"),
            ("layouts/showcase.yaml", "app: {}"),
            ("examples/clock.yaml", "app: {}"),
            ("examples/watch.sh", "#!/bin/sh\n"),
        ] {
            write(name, body);
        }
        for name in [
            "._boxmux",
            "._LICENSE",
            ".DS_Store",
            "layouts/.DS_Store",
            "layouts/notes.txt",
        ] {
            write(name, "not the release's");
        }
        write("examples/plugins/gauge/Cargo.toml", "[package]");
        let mut entries = documents(dir).unwrap();
        entries.extend(demos(dir).unwrap());
        entries.push(Entry {
            name: program.to_string(),
            from: dir.join(program),
            program: true,
        });
        entries
    }

    const RELEASE: [&str; 9] = [
        "boxmux/",
        "boxmux/LICENSE",
        "boxmux/README.md",
        "boxmux/boxmux",
        "boxmux/examples/",
        "boxmux/examples/clock.yaml",
        "boxmux/examples/watch.sh",
        "boxmux/layouts/",
        "boxmux/layouts/showcase.yaml",
    ];

    /// Every member of a tarball: its path, mode, owner, time and contents.
    fn tar_members(archive: &Path) -> Vec<(String, u32, u64, u64, String)> {
        let gz = flate2::read::GzDecoder::new(std::fs::File::open(archive).unwrap());
        tar::Archive::new(gz)
            .entries()
            .unwrap()
            .map(|e| {
                let mut e = e.unwrap();
                let path = e.path().unwrap().to_string_lossy().into_owned();
                let header = e.header();
                let (mode, uid, mtime) = (
                    header.mode().unwrap(),
                    header.uid().unwrap(),
                    header.mtime().unwrap(),
                );
                let mut body = String::new();
                e.read_to_string(&mut body).unwrap();
                (path, mode, uid, mtime, body)
            })
            .collect()
    }

    /// An archive holds the directory, the program, the documents and the demo configurations
    /// and nothing else — no AppleDouble file and no `.DS_Store` from beside them — and is the
    /// same bytes every time.
    #[test]
    fn a_tarball_holds_only_the_release_and_is_the_same_bytes_every_time() {
        let dir = tempfile::tempdir().unwrap();
        let entries = staged(dir.path(), "boxmux");
        let (first, second) = (dir.path().join("a.tar.gz"), dir.path().join("b.tar.gz"));
        write(&entries, "linux-x86_64", &first).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1100));
        let reversed: Vec<Entry> = entries.into_iter().rev().collect();
        write(&reversed, "linux-x86_64", &second).unwrap();
        assert_eq!(
            std::fs::read(&first).unwrap(),
            std::fs::read(&second).unwrap(),
            "the same files make the same archive"
        );
        let members = tar_members(&first);
        let paths: Vec<&str> = members.iter().map(|m| m.0.as_str()).collect();
        assert_eq!(paths, RELEASE);
        for (path, mode, uid, mtime, _) in &members {
            let program = path.ends_with('/')
                || ["boxmux/boxmux", "boxmux/examples/watch.sh"].contains(&path.as_str());
            assert_eq!(*mode, if program { 0o755 } else { 0o644 }, "{}", path);
            assert_eq!((*uid, *mtime), (0, 0), "{}", path);
        }
        // The system's tar, which a person unpacking it uses, reads the same tree.
        #[cfg(unix)]
        {
            let unpacked = dir.path().join("unpacked");
            std::fs::create_dir(&unpacked).unwrap();
            let status = std::process::Command::new("tar")
                .arg("xzf")
                .arg(&first)
                .arg("-C")
                .arg(&unpacked)
                .status()
                .unwrap();
            assert!(status.success());
            assert_eq!(
                std::fs::read_to_string(unpacked.join("boxmux/README.md")).unwrap(),
                "readme"
            );
        }
    }

    /// The Windows archive is a zip of the same shape, with no AppleDouble file either.
    #[test]
    fn a_zip_holds_only_the_release_and_is_the_same_bytes_every_time() {
        let dir = tempfile::tempdir().unwrap();
        let entries = staged(dir.path(), "boxmux.exe");
        let (first, second) = (dir.path().join("a.zip"), dir.path().join("b.zip"));
        write(&entries, "windows-x86_64", &first).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2100));
        write(&entries, "windows-x86_64", &second).unwrap();
        assert_eq!(
            std::fs::read(&first).unwrap(),
            std::fs::read(&second).unwrap()
        );
        let mut zip = zip::ZipArchive::new(std::fs::File::open(&first).unwrap()).unwrap();
        let names: Vec<String> = (0..zip.len())
            .map(|i| zip.by_index(i).unwrap().name().to_string())
            .collect();
        let expected: Vec<String> = RELEASE
            .iter()
            .map(|p| p.replace("boxmux/boxmux", "boxmux/boxmux.exe"))
            .collect();
        assert_eq!(names, expected);
        let mut body = String::new();
        zip.by_name("boxmux/boxmux.exe")
            .unwrap()
            .read_to_string(&mut body)
            .unwrap();
        assert_eq!(body, "the program");
    }

    /// `boxmux upgrade` unpacks each archive boxmux-dist makes into the program it installs.
    #[test]
    fn the_updater_unpacks_each_archive_into_the_program_it_installs() {
        for (platform, program) in [("linux-x86_64", "boxmux"), ("windows-x86_64", "boxmux.exe")] {
            let dir = tempfile::tempdir().unwrap();
            let staging = dir.path().join("staged");
            std::fs::create_dir(&staging).unwrap();
            let archive = dir.path().join(name("1.2.3", platform));
            write(&staged(&staging, program), platform, &archive).unwrap();
            let into = dir.path().join("into");
            let root = release::unpack(&archive, Archive::of(platform), &into)
                .map_err(|e| e.to_string())
                .unwrap();
            assert_eq!(
                std::fs::read_to_string(root.join(release::program_name(platform))).unwrap(),
                "the program"
            );
            assert_eq!(
                std::fs::read_to_string(root.join("layouts/showcase.yaml")).unwrap(),
                "app: {}"
            );
            #[cfg(unix)]
            if platform == "linux-x86_64" {
                use std::os::unix::fs::PermissionsExt;
                let mode = std::fs::metadata(root.join("boxmux"))
                    .unwrap()
                    .permissions()
                    .mode();
                assert_eq!(mode & 0o777, 0o755);
            }
        }
    }

    /// A name twice, or a name the updater would refuse, is never written: an AppleDouble
    /// file among the demo configurations stops the release, naming it.
    #[test]
    fn an_entry_the_updater_would_refuse_is_never_written() {
        let dir = tempfile::tempdir().unwrap();
        let mut entries = staged(dir.path(), "boxmux");
        entries.push(Entry {
            name: "boxmux".into(),
            from: entries[1].from.clone(),
            program: false,
        });
        let out = dir.path().join("a.tar.gz");
        assert_eq!(
            write(&entries, "linux-x86_64", &out).unwrap_err(),
            "boxmux is in the archive twice"
        );
        let mut entries = staged(dir.path(), "boxmux");
        entries.push(Entry {
            name: "._boxmux".into(),
            from: dir.path().join("._boxmux"),
            program: false,
        });
        assert_eq!(
            write(&entries, "linux-x86_64", &out).unwrap_err(),
            "._boxmux is no name a release archive holds"
        );
        std::fs::write(dir.path().join("layouts/._showcase.yaml"), "attributes").unwrap();
        let mut entries = documents(dir.path()).unwrap();
        entries.extend(demos(dir.path()).unwrap());
        assert_eq!(
            write(&entries, "linux-x86_64", &out).unwrap_err(),
            "layouts/._showcase.yaml is no name a release archive holds"
        );
        assert!(!out.exists(), "nothing is written for a refused archive");
    }

    /// A repository without demo configurations of a folder's kinds is refused, naming them.
    #[test]
    fn a_missing_demo_folder_is_refused_naming_it() {
        let dir = tempfile::tempdir().unwrap();
        let e = demos(dir.path()).err().expect("layouts/ is missing");
        assert!(e.contains("demo configurations in layouts/"), "{}", e);
        std::fs::create_dir_all(dir.path().join("layouts")).unwrap();
        std::fs::write(dir.path().join("layouts/a.yaml"), "").unwrap();
        std::fs::create_dir_all(dir.path().join("examples")).unwrap();
        std::fs::write(dir.path().join("examples/notes.txt"), "").unwrap();
        let e = demos(dir.path())
            .err()
            .expect("examples/ holds none of its kinds");
        assert!(e.ends_with("holds no *.yaml or *.sh"), "{}", e);
    }

    /// The repository's own demo configurations are the ones a release carries.
    #[test]
    fn the_repositorys_demos_are_carried() {
        let names: Vec<String> = demos(&crate::repository().unwrap())
            .unwrap()
            .into_iter()
            .map(|e| e.name)
            .collect();
        assert!(
            names.iter().any(|n| n == "layouts/showcase_menu.yaml"),
            "{:?}",
            names
        );
        assert!(names
            .iter()
            .all(|n| n.starts_with("layouts/") || n.starts_with("examples/")));
    }

    /// A repository missing a document is refused, naming it.
    #[test]
    fn a_missing_document_is_refused_naming_it() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("LICENSE"), "licence").unwrap();
        let e = documents(dir.path()).err().expect("README.md is missing");
        assert!(e.ends_with("holds no README.md"), "{}", e);
        std::fs::write(dir.path().join("README.md"), "readme").unwrap();
        let names: Vec<String> = documents(dir.path())
            .unwrap()
            .into_iter()
            .map(|e| e.name)
            .collect();
        assert_eq!(names, DOCUMENTS);
    }
}
