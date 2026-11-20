//! `boxmux upgrade` against release lines the tests publish and sign themselves.

use std::path::{Path, PathBuf};

use super::release::{
    self, fetch_latest, later, latest, misplaced, platform_of, unpack, Archive, Problem,
};
use super::testing::{
    manifest, release_archive, tar_gz, zip, Publisher, Served, NOT_AFTER, NOW, REGISTRY,
};
use super::{upgrade_with, Outcome};

const PLATFORM: &str = "linux-x86_64";

/// The reason a refusal names; anything else fails the test.
fn reason<T: std::fmt::Debug>(outcome: Result<T, Problem>) -> &'static str {
    match outcome {
        Err(Problem::Refused { reason, .. }) => reason,
        other => panic!("expected a refusal, got {:?}", other),
    }
}

/// A line publishing BoxMux `version` on the release channel, signed under `label`.
fn published_version(publisher: &Publisher, label: &str, version: &str) -> Served {
    let served = Served::default();
    served.publish(
        publisher,
        label,
        "release",
        version,
        PLATFORM,
        release_archive(version),
    );
    served
}

fn published(publisher: &Publisher, label: &str) -> Served {
    published_version(publisher, label, "0.300.1")
}

/// An installed BoxMux `version` in a folder of its own: the program a test upgrades.
struct Installed {
    _dir: tempfile::TempDir,
    program: PathBuf,
}

impl Installed {
    fn new(version: &str) -> Installed {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().canonicalize().unwrap();
        let program = folder.join("boxmux");
        std::fs::write(&program, super::testing::program(version)).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        Installed { _dir: dir, program }
    }

    fn folder(&self) -> &Path {
        self.program.parent().unwrap()
    }

    fn bytes(&self) -> Vec<u8> {
        std::fs::read(&self.program).unwrap()
    }

    /// Everything in the folder but the program: what an upgrade left behind.
    fn left_beside(&self) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(self.folder())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| name != "boxmux")
            .collect();
        names.sort();
        names
    }
}

fn upgrade(
    served: &Served,
    line: &release::Line,
    installed: &Installed,
    version: &str,
) -> Result<Outcome, String> {
    upgrade_with(served, line, PLATFORM, NOW, &installed.program, version)
}

/// A release line as the build bakes it is read back whole.
#[test]
fn the_baked_release_line_is_read_back_as_it_was_given() {
    let publisher = Publisher::new();
    let keys = publisher.root_keys().join(",");
    let line = release::parse_baked(&format!(
        "https://releases.example release prod {} {}",
        keys, keys
    ));
    assert_eq!(line, {
        let mut expected = publisher.line("release", "prod");
        expected.registry = "https://releases.example".to_string();
        expected
    });
}

/// BoxMux is published for four platforms, each by the registry's name, the Windows one as a
/// zip; any other is no platform BoxMux is published for.
#[test]
fn boxmux_is_published_for_four_platforms() {
    assert_eq!(platform_of("linux", "x86_64"), Some("linux-x86_64"));
    assert_eq!(platform_of("linux", "aarch64"), Some("linux-arm64"));
    assert_eq!(platform_of("macos", "aarch64"), Some("darwin-arm64"));
    assert_eq!(platform_of("windows", "x86_64"), Some("windows-x86_64"));
    assert_eq!(platform_of("macos", "x86_64"), None);
    assert_eq!(platform_of("freebsd", "x86_64"), None);
    assert_eq!(
        release::archive_name("1.2.3", "linux-arm64"),
        "boxmux-1.2.3-linux-arm64.tar.gz"
    );
    assert_eq!(
        release::archive_name("1.2.3", "windows-x86_64"),
        "boxmux-1.2.3-windows-x86_64.zip"
    );
    assert_eq!(release::program_name("windows-x86_64"), "boxmux.exe");
    assert_eq!(release::program_name("darwin-arm64"), "boxmux");
}

/// A line the build's roots signed is read, and its newest release for the platform found.
#[test]
fn a_line_signed_by_the_builds_roots_is_read() {
    let publisher = Publisher::new();
    let served = published(&publisher, "prod");
    let found = fetch_latest(&served, &publisher.line("release", "prod"), PLATFORM, NOW)
        .unwrap()
        .expect("a release is published");
    assert_eq!(found.version, "0.300.1");
    assert_eq!(found.name, "boxmux-0.300.1-linux-x86_64.tar.gz");
    assert_eq!(
        found.url,
        Served::archive_url("release", "0.300.1", PLATFORM)
    );
    assert_eq!(
        *served.asked.borrow(),
        [
            format!("{}/charter.json", REGISTRY),
            format!("{}/charter.json.sig", REGISTRY),
            Served::manifest_url(),
            format!("{}.sig", Served::manifest_url()),
        ]
    );
    // The nightly channel has published nothing.
    assert_eq!(
        fetch_latest(&served, &publisher.line("nightly", "prod"), PLATFORM, NOW).unwrap(),
        None
    );
}

/// A manifest changed after it was signed, or published without a signature, is refused.
#[test]
fn a_tampered_or_unsigned_manifest_is_refused() {
    let publisher = Publisher::new();
    let line = publisher.line("release", "prod");
    let served = published(&publisher, "prod");
    let mut changed = served.files.borrow()[&Served::manifest_url()].clone();
    let at = changed.len() - 2;
    changed[at] ^= 1;
    served.put(&Served::manifest_url(), changed);
    assert_eq!(
        reason(fetch_latest(&served, &line, PLATFORM, NOW)),
        "manifest_signature_invalid"
    );

    served.remove(&format!("{}.sig", Served::manifest_url()));
    assert_eq!(
        reason(fetch_latest(&served, &line, PLATFORM, NOW)),
        "manifest_unsigned"
    );

    // Signed by another publisher's roots and release key.
    let served = published(&publisher, "prod");
    let manifest = served.files.borrow()[&Served::manifest_url()].clone();
    served.put(
        &format!("{}.sig", Served::manifest_url()),
        Publisher::foreign().sign(&manifest, "prod"),
    );
    assert_eq!(
        reason(fetch_latest(&served, &line, PLATFORM, NOW)),
        "insufficient_root_signatures"
    );
}

/// A certificate for another environment than the build's is refused, and one past its
/// validity.
#[test]
fn a_certificate_for_another_environment_or_out_of_date_is_refused() {
    let publisher = Publisher::new();
    let served = published(&publisher, "prod");
    // The charter says staging, as a staging build expects, and the manifest is prod's.
    let charter = Publisher::charter(REGISTRY, "staging");
    served.put(
        &format!("{}/charter.json.sig", REGISTRY),
        publisher.sign_charter(&charter),
    );
    served.put(&format!("{}/charter.json", REGISTRY), charter);
    let refused = fetch_latest(
        &served,
        &publisher.line("release", "staging"),
        PLATFORM,
        NOW,
    );
    assert_eq!(reason(refused), "environment_mismatch");

    let served = published(&publisher, "prod");
    let late = fetch_latest(
        &served,
        &publisher.line("release", "prod"),
        PLATFORM,
        NOT_AFTER + 1,
    );
    assert_eq!(reason(late), "expired_certificate");
}

/// A foreign or unsigned charter is refused before any manifest is read, and a registry with
/// no charter is not read at all.
#[test]
fn a_foreign_charter_is_refused_before_the_manifest_is_read() {
    let publisher = Publisher::new();
    let line = publisher.line("release", "prod");
    let refuses = |served: &Served, expected: &str| {
        served.asked.borrow_mut().clear();
        assert_eq!(reason(fetch_latest(served, &line, PLATFORM, NOW)), expected);
        assert!(
            !served.asked.borrow().contains(&Served::manifest_url()),
            "the manifest was asked for under a refused charter"
        );
    };

    let served = published(&publisher, "prod");
    let charter = Publisher::charter(REGISTRY, "prod");
    served.put(
        &format!("{}/charter.json.sig", REGISTRY),
        Publisher::foreign().sign_charter(&charter),
    );
    refuses(&served, "insufficient_root_signatures");

    let elsewhere = Publisher::charter("https://elsewhere.example", "prod");
    served.put(&format!("{}/charter.json", REGISTRY), elsewhere.clone());
    served.put(
        &format!("{}/charter.json.sig", REGISTRY),
        publisher.sign_charter(&elsewhere),
    );
    refuses(&served, "charter_anchor_mismatch");

    // Signed by this build's roots, in another registry's words.
    let certificate = Publisher::certificate(&publisher.release, "prod");
    let roots = [&publisher.roots[0], &publisher.roots[1]];
    let theirs = Publisher::envelope(
        "other-charter-sig/1",
        "charter_signature",
        &certificate,
        &roots,
        &publisher.release,
        &charter,
    );
    served.put(&format!("{}/charter.json", REGISTRY), charter.clone());
    served.put(&format!("{}/charter.json.sig", REGISTRY), theirs);
    refuses(&served, "unsupported_envelope_format");

    served.remove(&format!("{}/charter.json.sig", REGISTRY));
    refuses(&served, "charter_unsigned");

    served.remove(&format!("{}/charter.json", REGISTRY));
    served.asked.borrow_mut().clear();
    let failed = fetch_latest(&served, &line, PLATFORM, NOW).expect_err("no charter");
    assert!(
        matches!(&failed, Problem::Failed(why) if why.contains("has no charter")),
        "{:?}",
        failed
    );
    assert_eq!(
        *served.asked.borrow(),
        [format!("{}/charter.json", REGISTRY)]
    );
}

/// A signed manifest that is not BoxMux's, or leaves this platform out, is refused for its
/// content; the Windows release is taken as its zip.
#[test]
fn a_manifest_that_is_not_boxmuxs_or_not_whole_is_refused() {
    let url = "https://release.example/boxmux/release/1.0.0/a";
    let sha = "a".repeat(64);
    let good = manifest("release", "1.0.0", PLATFORM, url, &sha, 10);
    let message = |bytes: &[u8], platform: &str| match latest(bytes, "release", platform) {
        Err(Problem::Refused {
            reason: "manifest_content",
            message,
        }) => message,
        other => panic!("expected a refusal of the content, got {:?}", other),
    };
    assert!(message(&good, "linux-arm64").contains("not published for linux-arm64"));
    let other = String::from_utf8(good.clone())
        .unwrap()
        .replace("\"appId\":\"boxmux\"", "\"appId\":\"wsb\"");
    assert!(message(other.as_bytes(), PLATFORM).contains("wsb's, not boxmux's"));
    let old = String::from_utf8(good.clone())
        .unwrap()
        .replace("\"schemaVersion\":\"5.0\"", "\"schemaVersion\":\"4.0\"");
    assert!(message(old.as_bytes(), PLATFORM).contains("schema 4.0"));
    let short = manifest("release", "1.0.0", PLATFORM, url, "abc", 10);
    assert!(message(&short, PLATFORM).contains("as its SHA-256"));
    assert!(message(b"not json", PLATFORM).contains("cannot be read"));

    let windows = manifest("release", "1.0.0", "windows-x86_64", url, &sha, 10);
    let found = latest(&windows, "release", "windows-x86_64")
        .unwrap()
        .unwrap();
    assert_eq!(found.name, "boxmux-1.0.0-windows-x86_64.zip");
    // A Windows release published as a tarball is not BoxMux's Windows archive.
    let tarball = String::from_utf8(windows)
        .unwrap()
        .replace("\"type\":\"zip\"", "\"type\":\"targz\"");
    assert!(message(tarball.as_bytes(), "windows-x86_64").contains("without a .zip archive"));
}

/// The newest release is installed in place of the running program, once its archive matches
/// the signed manifest and its program says it is the published version; nothing is left
/// beside it.
#[cfg(unix)]
#[test]
fn the_newest_release_signed_by_the_builds_roots_is_installed() {
    let publisher = Publisher::new();
    let served = published(&publisher, "prod");
    let installed = Installed::new("0.287.728");
    let outcome = upgrade(
        &served,
        &publisher.line("release", "prod"),
        &installed,
        "0.287.728",
    )
    .unwrap();
    assert_eq!(
        outcome,
        Outcome::Installed {
            channel: "release".to_string(),
            platform: PLATFORM.to_string(),
            from: "0.287.728".to_string(),
            to: "0.300.1".to_string(),
            program: installed.program.clone(),
        }
    );
    assert_eq!(installed.bytes(), super::testing::program("0.300.1"));
    let says = std::process::Command::new(&installed.program)
        .arg("--version")
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&says.stdout).starts_with("boxmux 0.300.1 "));
    assert_eq!(installed.left_beside(), Vec::<String>::new());
    assert!(outcome
        .to_string()
        .contains("boxmux 0.300.1 is the newest release on the release channel for linux-x86_64"));
}

/// The same release, or an older one, installs nothing: never a downgrade.
#[test]
fn a_release_no_later_than_the_running_one_installs_nothing() {
    let publisher = Publisher::new();
    let line = publisher.line("release", "prod");
    for running in ["0.300.1", "0.300.2", "1.0.0"] {
        let served = published(&publisher, "prod");
        let installed = Installed::new(running);
        let outcome = upgrade(&served, &line, &installed, running).unwrap();
        assert_eq!(
            outcome,
            Outcome::UpToDate {
                channel: "release".to_string(),
                platform: PLATFORM.to_string(),
                installed: running.to_string(),
                newest: "0.300.1".to_string(),
            }
        );
        assert_eq!(installed.bytes(), super::testing::program(running));
        assert!(!served
            .asked
            .borrow()
            .contains(&Served::archive_url("release", "0.300.1", PLATFORM)));
    }
    let empty = Served::default();
    let charter = Publisher::charter(REGISTRY, "prod");
    empty.put(
        &format!("{}/charter.json.sig", REGISTRY),
        publisher.sign_charter(&charter),
    );
    empty.put(&format!("{}/charter.json", REGISTRY), charter);
    let installed = Installed::new("0.287.728");
    assert!(matches!(
        upgrade(&empty, &line, &installed, "0.287.728").unwrap(),
        Outcome::NothingPublished { .. }
    ));
}

/// An archive that is not what the manifest signed, by its SHA-256 or its size, is refused,
/// and the running program stays as it was, with nothing left beside it.
#[test]
fn an_archive_other_than_the_signed_one_is_refused_and_nothing_changes() {
    let publisher = Publisher::new();
    let line = publisher.line("release", "prod");
    let url = Served::archive_url("release", "0.300.1", PLATFORM);
    let archive = release_archive("0.300.1");
    let cases = [
        ("archive_sha256_mismatch", {
            let mut changed = archive.clone();
            let at = changed.len() / 2;
            changed[at] ^= 1;
            changed
        }),
        (
            "archive_size_mismatch",
            [archive.clone(), b"more".to_vec()].concat(),
        ),
        (
            "archive_size_mismatch",
            archive[..archive.len() - 1].to_vec(),
        ),
    ];
    for (expected, served_archive) in cases {
        let served = published(&publisher, "prod");
        served.put(&url, served_archive);
        let installed = Installed::new("0.287.728");
        let refused = upgrade(&served, &line, &installed, "0.287.728").expect_err("refused");
        assert!(
            refused.contains(&format!("(refused: {})", expected)),
            "{}",
            refused
        );
        assert_eq!(installed.bytes(), super::testing::program("0.287.728"));
        assert_eq!(installed.left_beside(), Vec::<String>::new());
    }
}

/// A release whose program says it is another version than the manifest published is refused.
#[cfg(unix)]
#[test]
fn a_program_that_is_not_the_published_version_is_refused() {
    let publisher = Publisher::new();
    let served = Served::default();
    served.publish(
        &publisher,
        "prod",
        "release",
        "0.300.1",
        PLATFORM,
        release_archive("0.299.0"),
    );
    let installed = Installed::new("0.287.728");
    let refused = upgrade(
        &served,
        &publisher.line("release", "prod"),
        &installed,
        "0.287.728",
    )
    .expect_err("refused");
    assert!(
        refused.contains("(refused: archive_version)"),
        "{}",
        refused
    );
    assert!(refused.contains("boxmux 0.299.0"), "{}", refused);
    assert_eq!(installed.bytes(), super::testing::program("0.287.728"));
}

/// An archive unpacks only into its one directory, holding files and directories, with no
/// AppleDouble file anywhere in it, as a tarball and as a zip.
#[test]
fn an_archive_unpacks_only_into_its_one_directory_with_no_appledouble_file() {
    let unpack_bytes = |kind: Archive, bytes: Vec<u8>| {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("archive");
        std::fs::write(&archive, bytes).unwrap();
        let outcome = unpack(&archive, kind, &dir.path().join("into"));
        (outcome.map(|root| root.ends_with("boxmux")), dir)
    };
    for (kind, pack) in [
        (Archive::TarGz, tar_gz as fn(&[(&str, &[u8])]) -> Vec<u8>),
        (Archive::Zip, zip as fn(&[(&str, &[u8])]) -> Vec<u8>),
    ] {
        assert_eq!(
            unpack_bytes(
                kind,
                pack(&[("boxmux/boxmux", b"x"), ("boxmux/LICENSE", b"l")])
            )
            .0,
            Ok(true)
        );
        for refused in [
            pack(&[("boxmux/boxmux", b"x"), ("other/boxmux", b"y")]),
            pack(&[("boxmux", b"x")]),
            pack(&[("._boxmux", b"attributes"), ("boxmux/boxmux", b"x")]),
            pack(&[("boxmux/boxmux", b"x"), ("boxmux/._boxmux", b"attributes")]),
            pack(&[("boxmux/docs/._README.md", b"attributes")]),
        ] {
            assert_eq!(reason(unpack_bytes(kind, refused).0), "archive_layout");
        }
        assert_eq!(
            reason(unpack_bytes(kind, b"boxmux/boxmux".to_vec()).0),
            "archive_layout"
        );
    }
    // An entry that is a link is not taken.
    let mut builder = tar::Builder::new(flate2::write::GzEncoder::new(
        Vec::new(),
        flate2::Compression::default(),
    ));
    let mut header = tar::Header::new_gnu();
    header.set_entry_type(tar::EntryType::Symlink);
    header.set_size(0);
    builder
        .append_link(&mut header, "boxmux/boxmux", "/bin/sh")
        .unwrap();
    let link = builder.into_inner().unwrap().finish().unwrap();
    assert_eq!(
        reason(unpack_bytes(Archive::TarGz, link).0),
        "archive_layout"
    );

    assert!(misplaced(Path::new("boxmux/boxmux")).is_none());
    assert!(misplaced(Path::new("boxmux/._boxmux"))
        .unwrap()
        .contains("AppleDouble"));
}

/// A release whose archive holds an AppleDouble file is refused whole, and nothing of it is
/// installed.
#[test]
fn a_release_holding_an_appledouble_file_is_not_installed() {
    let publisher = Publisher::new();
    let served = Served::default();
    let archive = tar_gz(&[
        ("._boxmux", b"attributes"),
        ("boxmux/boxmux", &super::testing::program("0.300.1")),
    ]);
    served.publish(&publisher, "prod", "release", "0.300.1", PLATFORM, archive);
    let installed = Installed::new("0.287.728");
    let refused = upgrade(
        &served,
        &publisher.line("release", "prod"),
        &installed,
        "0.287.728",
    )
    .expect_err("refused");
    assert!(refused.contains("(refused: archive_layout)"), "{}", refused);
    assert_eq!(installed.bytes(), super::testing::program("0.287.728"));
    assert_eq!(installed.left_beside(), Vec::<String>::new());
}

/// A BoxMux a package manager installed is left to it: the upgrade names the newest release
/// and the manager's command, and changes nothing. A marker about another program, or one
/// that cannot be read, is an error, never taken for an archive's install.
#[test]
fn a_package_managers_boxmux_is_left_to_the_manager() {
    let publisher = Publisher::new();
    let line = publisher.line("release", "prod");
    let served = published(&publisher, "prod");
    let installed = Installed::new("0.287.728");
    let marker = installed.folder().join(super::package::MARKER);
    std::fs::write(
        &marker,
        serde_json::json!({
            "format": 1,
            "manager": "apt",
            "program": installed.program,
            "update": "sudo apt install --only-upgrade boxmux",
        })
        .to_string(),
    )
    .unwrap();
    let refused = upgrade(&served, &line, &installed, "0.287.728").expect_err("left to apt");
    assert_eq!(
        refused,
        "boxmux 0.300.1 is the newest release on the release channel for linux-x86_64, and boxmux \
         was installed by apt, which upgrades it: run `sudo apt install --only-upgrade boxmux`"
    );
    assert_eq!(installed.bytes(), super::testing::program("0.287.728"));
    assert!(!served
        .asked
        .borrow()
        .contains(&Served::archive_url("release", "0.300.1", PLATFORM)));

    let other = installed.folder().join("other");
    std::fs::write(
        &marker,
        serde_json::json!({ "format": 1, "manager": "apt", "program": other, "update": "x" })
            .to_string(),
    )
    .unwrap();
    let refused = upgrade(&served, &line, &installed, "0.287.728").expect_err("bad marker");
    assert!(
        refused.contains("which is not the program beside it"),
        "{}",
        refused
    );
    std::fs::write(&marker, "{").unwrap();
    let refused = upgrade(&served, &line, &installed, "0.287.728").expect_err("bad marker");
    assert!(refused.contains("it is not JSON"), "{}", refused);
    assert_eq!(installed.bytes(), super::testing::program("0.287.728"));
}

/// A registry that cannot be reached fails naming what was asked; a folder the upgrade cannot
/// write in fails naming the folder.
#[test]
fn an_unreachable_registry_or_an_unwritable_folder_fails_naming_it() {
    struct Down;
    impl release::Transport for Down {
        fn get(&self, url: &str) -> Result<Option<Vec<u8>>, String> {
            Err(format!("could not reach {}: connection refused", url))
        }
        fn download(&self, url: &str, _: &Path, _: u64) -> Result<u64, String> {
            Err(format!("could not reach {}: connection refused", url))
        }
    }
    let publisher = Publisher::new();
    let line = publisher.line("release", "prod");
    let installed = Installed::new("0.287.728");
    let failed = upgrade_with(&Down, &line, PLATFORM, NOW, &installed.program, "0.287.728")
        .expect_err("down");
    assert_eq!(
        failed,
        format!(
            "could not reach {}/charter.json: connection refused",
            REGISTRY
        )
    );

    let served = published(&publisher, "prod");
    let gone = installed.folder().join("missing").join("boxmux");
    let failed =
        upgrade_with(&served, &line, PLATFORM, NOW, &gone, "0.287.728").expect_err("no folder");
    assert!(
        failed.starts_with(&format!(
            "cannot write in {}",
            gone.parent().unwrap().display()
        )),
        "{}",
        failed
    );
}

/// A release is later by its numbers, read as numbers.
#[test]
fn a_release_is_later_by_its_numbers() {
    assert_eq!(later("0.300.1", "0.287.728"), Ok(true));
    assert_eq!(later("0.287.1000", "0.287.728"), Ok(true));
    assert_eq!(later("0.287.728", "0.287.728"), Ok(false));
    assert_eq!(later("0.9.0", "0.10.0"), Ok(false));
    assert!(later("1.0", "0.1.0")
        .unwrap_err()
        .contains("the published version \"1.0\""));
    assert!(later("1.0.0", "x")
        .unwrap_err()
        .contains("the installed version"));
}
