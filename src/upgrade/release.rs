//! BoxMux's releases: where they are published, and what makes one trusted.
//!
//! BoxMux is published in a release registry, named by its **anchor**. The registry exists
//! because it was chartered: `<anchor>/charter.json` states, among the rest, where each
//! product's manifest is, and `<anchor>/charter.json.sig` signs those exact bytes with the prod
//! release key, whatever environment the registry's contents are signed under. Nothing about
//! the registry is read from the shape of its URL: the charter is verified against the charter
//! roots this BoxMux was built with, under the `prod` label, and must be the charter of the
//! anchor asked, of a release registry, for this build's environment, and within its own
//! validity. BoxMux's manifest is then read where the charter says, with a signature beside it
//! at the same address and `.sig`. The manifest lists, per channel (`release`, `nightly`),
//! every version with the archive of each platform, its SHA-256 and its size.
//!
//! Nothing in a manifest is believed until its signature chains to the root keys this BoxMux
//! was built with:
//!
//! 1. the signature file carries one or more **release-key certificates**. A certificate is
//!    trusted when two distinct roots of this build signed its exact bytes, it is for this
//!    build's signing environment, and the clock is inside its validity;
//! 2. the **manifest signature** must verify, under the release key of one of those
//!    certificates, over the exact bytes that were fetched.
//!
//! Every certificate in the file must be valid: one that is not means a publishing mistake or
//! tampering, and reading past it would hide either. Only then is the manifest parsed, and the
//! archive taken from it is still held to the SHA-256 and size the manifest gives, and to its
//! layout: one directory, [`ARCHIVE_ROOT`], holding only directories and files.
//!
//! A BoxMux built from source has no roots, and so no release it could trust: it has no release
//! line ([`baked`] is `None`).

use std::io::Read as _;
use std::path::{Component, Path, PathBuf};

use mg_srel::{CharterExpectation, CharterRules, CharterTrust, Formats, KindRule, LocationRule};
use serde::Deserialize;
use sha2::Digest;

/// The product's name in the release registry.
pub const APP_ID: &str = "boxmux";
/// The one directory every release archive unpacks to, holding the `boxmux` program, its
/// licence and its README.
pub const ARCHIVE_ROOT: &str = "boxmux";

/// The words the release registry's signed documents declare themselves with. The registry's
/// release declaration names the same words for every product published in it, so its
/// publisher signs what every one of its clients reads.
pub struct Words {
    pub certificate: &'static str,
    pub manifest_envelope: &'static str,
    pub charter: &'static str,
    pub charter_envelope: &'static str,
}

/// The registry's words: the one place BoxMux names them.
pub const WORDS: Words = Words {
    certificate: "bahramj-release-key-cert/1",
    manifest_envelope: "bahramj-manifest-sig/1",
    charter: "bahramj-registry-charter/1",
    charter_envelope: "bahramj-charter-sig/1",
};

/// The label a charter's certificate carries: charters are signed by prod for every
/// environment.
const CHARTER_LABEL: &str = "prod";
/// The kind of registry BoxMux is published in.
const RELEASES: &str = "releases";
/// The folder of what is held from no registry: no charter may claim it.
const UNCHARTERED_SLUG: &str = "dev";
const MANIFEST_SCHEMA_MAJOR: &str = "5";
/// How long fetching a document may take, and an archive.
const DOCUMENT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);
const ARCHIVE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30 * 60);

/// What a released BoxMux knows about its own release line, baked in by its build
/// (`build/release_line.rs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    /// The anchor of the release registry this BoxMux was published in.
    pub registry: String,
    /// The channel it was published on: `release` or `nightly`.
    pub channel: String,
    /// The environment the registry's release key is certified for: `prod` or `staging`.
    pub environment: String,
    /// The roots a release-key certificate must be signed by, in minisign's base64 form.
    pub roots: Vec<String>,
    /// The roots the registry's charter must chain to.
    pub charter_roots: Vec<String>,
}

/// This build's release line, or `None` for a build from source.
pub fn baked() -> Option<Line> {
    option_env!("BOXMUX_BAKED_RELEASE_LINE").map(parse_baked)
}

/// The release line as the build wrote it: registry, channel, environment, the roots and the
/// charter roots (each comma-separated), with a space between them. The build checked every
/// part before it wrote them, so a value of another shape is not something a build can
/// produce.
pub fn parse_baked(baked: &str) -> Line {
    let parts: Vec<&str> = baked.split(' ').collect();
    let [registry, channel, environment, roots, charter_roots] = parts[..] else {
        panic!(
            "this BoxMux's baked release line has {} parts, not 5: {:?}",
            parts.len(),
            baked
        );
    };
    let keys = |list: &str| list.split(',').map(str::to_string).collect();
    Line {
        registry: registry.to_string(),
        channel: channel.to_string(),
        environment: environment.to_string(),
        roots: keys(roots),
        charter_roots: keys(charter_roots),
    }
}

/// What a BoxMux with no release line says: it was built from source.
pub const NO_LINE: &str = "this boxmux was built from source, so it has no release line to \
                           upgrade along: build the newer source the same way, or install a \
                           release";

/// The registry's name for the platform this BoxMux runs on, or `None` where BoxMux is not
/// published: Linux on arm64 among them, where BoxMux is built from source.
pub fn platform() -> Option<&'static str> {
    platform_of(std::env::consts::OS, std::env::consts::ARCH)
}

/// The registry's name for `os` and `arch`, as Rust names them.
pub fn platform_of(os: &str, arch: &str) -> Option<&'static str> {
    Some(match (os, arch) {
        ("linux", "x86_64") => "linux-x86_64",
        ("macos", "aarch64") => "darwin-arm64",
        ("windows", "x86_64") => "windows-x86_64",
        _ => return None,
    })
}

/// The kind of archive a platform's release is published as.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Archive {
    /// A gzipped tar archive, `.tar.gz`: every platform but Windows.
    TarGz,
    /// A zip archive, `.zip`: Windows.
    Zip,
}

impl Archive {
    pub fn of(platform: &str) -> Archive {
        if platform.starts_with("windows-") {
            Archive::Zip
        } else {
            Archive::TarGz
        }
    }

    /// The installer type the manifest lists the archive under.
    pub fn installer_type(self) -> &'static str {
        match self {
            Archive::TarGz => "targz",
            Archive::Zip => "zip",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Archive::TarGz => "tar.gz",
            Archive::Zip => "zip",
        }
    }
}

/// The name of BoxMux `version`'s archive for `platform`.
pub fn archive_name(version: &str, platform: &str) -> String {
    format!(
        "{}-{}-{}.{}",
        APP_ID,
        version,
        platform,
        Archive::of(platform).extension()
    )
}

/// The name of the program in the archive for `platform`.
pub fn program_name(platform: &str) -> &'static str {
    match Archive::of(platform) {
        Archive::TarGz => "boxmux",
        Archive::Zip => "boxmux.exe",
    }
}

/// What BoxMux's charters are: release registries, each keeping one manifest per product at a
/// location that takes the product's id.
pub fn charter_rules() -> CharterRules {
    CharterRules {
        format: WORDS.charter.to_string(),
        fields: Vec::new(),
        kinds: vec![KindRule {
            name: RELEASES.to_string(),
            locations: vec![LocationRule {
                name: "manifest".to_string(),
                placeholders: vec!["app_id".to_string()],
            }],
            urls: Vec::new(),
        }],
        reserved_slugs: vec![UNCHARTERED_SLUG.to_string()],
    }
}

/// What a manifest's chain is verified as, and a charter's.
fn formats(envelope: &str) -> Formats {
    Formats {
        certificate: WORDS.certificate.to_string(),
        envelope: envelope.to_string(),
    }
}

/// Why a release was not taken: judged and refused, with the reason every client of the
/// release line decides by, or not reached at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    /// Something fetched was judged and refused. `reason` is mg-srel's wire name for a charter
    /// or a manifest's chain, or the archive's or manifest's own (`charter_unsigned`,
    /// `manifest_unsigned`, `manifest_content`, `archive_size_mismatch`,
    /// `archive_sha256_mismatch`, `archive_layout`).
    Refused {
        reason: &'static str,
        message: String,
    },
    /// It could not be fetched, read or written: the network, the server, the disk.
    Failed(String),
}

impl std::fmt::Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Problem::Refused { reason, message } => write!(f, "{} (refused: {})", message, reason),
            Problem::Failed(message) => f.write_str(message),
        }
    }
}

impl From<Problem> for String {
    fn from(problem: Problem) -> String {
        problem.to_string()
    }
}

fn refused(reason: &'static str, message: String) -> Problem {
    Problem::Refused { reason, message }
}

/// Checks that `envelope` (the bytes of `charter.json.sig`) authorizes `charter` (the exact
/// bytes of `charter.json`) under `line`'s charter roots, by [`charter_rules`], and that it
/// charters the release registry at `line.registry` for `line.environment`, at the time `now`.
/// Returns the address of BoxMux's manifest, as the charter states it.
pub fn verify_charter(
    charter: &[u8],
    envelope: &[u8],
    line: &Line,
    now: u64,
) -> Result<String, Problem> {
    let roots: Vec<&str> = line.charter_roots.iter().map(String::as_str).collect();
    let trust = mg_srel::Trust::new(&roots, CHARTER_LABEL, formats(WORDS.charter_envelope))
        .map(|trust| CharterTrust { trust })
        .map_err(|e| {
            Problem::Failed(format!("this boxmux's charter roots cannot be used: {}", e))
        })?;
    let expect = CharterExpectation {
        anchor: Some(&line.registry),
        kind: RELEASES,
        environment: Some(&line.environment),
    };
    let verified =
        mg_srel::verify_charter(charter, envelope, &trust, &charter_rules(), &expect, now)
            .map_err(|e| {
                refused(
                    e.reason().wire_name(),
                    format!("the charter of {} was refused: {}", line.registry, e),
                )
            })?;
    // The rules hold a release registry's manifest location to taking `{app_id}` and nothing
    // else, so filling it in cannot fail for a charter that verified.
    Ok(verified
        .charter
        .location("manifest", &[("app_id", APP_ID)])
        .expect("a verified release charter's manifest location takes {app_id}"))
}

/// Checks that `envelope` (the bytes of the manifest's `.sig`) authorizes `manifest`, fetched
/// from `url`, under `line`'s roots and environment, at the time `now`.
pub fn verify_manifest(
    manifest: &[u8],
    envelope: &[u8],
    url: &str,
    line: &Line,
    now: u64,
) -> Result<(), Problem> {
    let roots: Vec<&str> = line.roots.iter().map(String::as_str).collect();
    let trust = mg_srel::Trust::new(&roots, &line.environment, formats(WORDS.manifest_envelope))
        .map_err(|e| {
            Problem::Failed(format!("this boxmux's signing roots cannot be used: {}", e))
        })?;
    mg_srel::verify_manifest(envelope, manifest, &trust, now)
        .map(|_| ())
        .map_err(|e| {
            refused(
                e.reason().wire_name(),
                format!(
                    "{} is not a release manifest this boxmux trusts: {}",
                    url, e
                ),
            )
        })
}

/// One published archive of BoxMux.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    pub version: String,
    /// The platform it is for, by the registry's name.
    pub platform: String,
    /// The archive's file name, as [`archive_name`] gives it.
    pub name: String,
    pub url: String,
    /// Lower-case hexadecimal.
    pub sha256: String,
    pub size: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    schema_version: String,
    app_id: String,
    channels: std::collections::BTreeMap<String, Channel>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Channel {
    latest_version: Option<String>,
    #[serde(default)]
    versions: std::collections::BTreeMap<String, Version>,
}

#[derive(Deserialize)]
struct Version {
    platforms: std::collections::BTreeMap<String, Platform>,
}

#[derive(Deserialize)]
struct Platform {
    installers: Vec<Installer>,
}

#[derive(Deserialize)]
struct Installer {
    #[serde(rename = "type")]
    kind: String,
    name: String,
    path: String,
    sha256: String,
    size: u64,
}

/// The latest release of `channel` for `platform` in a manifest already verified, or `None`
/// when the channel has published nothing yet. A manifest that is not BoxMux's, or whose latest
/// version has no archive for the platform, is refused: the first is the wrong document, and
/// the second is a release this platform was left out of.
pub fn latest(manifest: &[u8], channel: &str, platform: &str) -> Result<Option<Release>, Problem> {
    let content = |message: String| refused("manifest_content", message);
    let manifest: Manifest = serde_json::from_slice(manifest)
        .map_err(|e| content(format!("the release manifest cannot be read: {}", e)))?;
    if manifest.schema_version.split('.').next() != Some(MANIFEST_SCHEMA_MAJOR) {
        return Err(content(format!(
            "the release manifest is of schema {}; this boxmux reads schema {}",
            manifest.schema_version, MANIFEST_SCHEMA_MAJOR
        )));
    }
    if manifest.app_id != APP_ID {
        return Err(content(format!(
            "the release manifest is {}'s, not {}'s",
            manifest.app_id, APP_ID
        )));
    }
    let Some(releases) = manifest.channels.get(channel) else {
        return Err(content(format!(
            "the release manifest has no {} channel",
            channel
        )));
    };
    let Some(version) = releases.latest_version.as_deref().filter(|v| !v.is_empty()) else {
        return Ok(None);
    };
    let Some(release) = releases.versions.get(version) else {
        return Err(content(format!(
            "the release manifest names {} as the latest {} version and does not list it",
            version, channel
        )));
    };
    let Some(for_platform) = release.platforms.get(platform) else {
        let published: Vec<&str> = release.platforms.keys().map(String::as_str).collect();
        return Err(content(format!(
            "boxmux {} is not published for {} (it is for {})",
            version,
            platform,
            published.join(", ")
        )));
    };
    let kind = Archive::of(platform);
    let Some(archive) = for_platform
        .installers
        .iter()
        .find(|i| i.kind == kind.installer_type())
    else {
        return Err(content(format!(
            "boxmux {} for {} is published without a .{} archive",
            version,
            platform,
            kind.extension()
        )));
    };
    let expected = archive_name(version, platform);
    if archive.name != expected {
        return Err(content(format!(
            "boxmux {} for {} names its archive {:?}, not {:?}",
            version, platform, archive.name, expected
        )));
    }
    if archive.sha256.len() != 64 || !archive.sha256.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(content(format!(
            "boxmux {} for {} carries {:?} as its SHA-256",
            version, platform, archive.sha256
        )));
    }
    Ok(Some(Release {
        version: version.to_string(),
        platform: platform.to_string(),
        name: archive.name.clone(),
        url: archive.path.clone(),
        sha256: archive.sha256.to_ascii_lowercase(),
        size: archive.size,
    }))
}

/// How the release line is reached: HTTPS in a released BoxMux, a table of documents in a
/// test.
pub trait Transport {
    /// The bytes at `url`, never from a cache; `None` when the server says there is nothing
    /// there.
    fn get(&self, url: &str) -> Result<Option<Vec<u8>>, String>;
    /// The file at `url`, written to `to`: no more than `limit` bytes of it and one more, so
    /// that a longer file shows as one. Answers how many bytes were written.
    fn download(&self, url: &str, to: &Path, limit: u64) -> Result<u64, String>;
}

/// The release line over HTTPS, as a released BoxMux reaches it: rustls, with the web's root
/// certificates, and a proxy from the environment.
pub struct Https {
    documents: ureq::Agent,
    archives: ureq::Agent,
}

impl Https {
    pub fn new() -> Https {
        let agent = |timeout| ureq::AgentBuilder::new().timeout(timeout).build();
        Https {
            documents: agent(DOCUMENT_TIMEOUT),
            archives: agent(ARCHIVE_TIMEOUT),
        }
    }

    /// A response to `url` from `agent`, never from a cache: a document and its signature are
    /// replaced together, and a stale one of either beside a fresh one of the other reads as a
    /// forgery.
    fn fetch(agent: &ureq::Agent, url: &str) -> Result<Option<ureq::Response>, String> {
        match agent.get(url).set("Cache-Control", "no-cache").call() {
            Ok(response) => Ok(Some(response)),
            Err(ureq::Error::Status(404, _)) => Ok(None),
            Err(ureq::Error::Status(status, response)) => Err(format!(
                "{} answered {} {}",
                url,
                status,
                response.status_text()
            )),
            Err(ureq::Error::Transport(e)) => Err(format!("could not reach {}: {}", url, e)),
        }
    }
}

impl Default for Https {
    fn default() -> Self {
        Https::new()
    }
}

impl Transport for Https {
    fn get(&self, url: &str) -> Result<Option<Vec<u8>>, String> {
        let Some(response) = Https::fetch(&self.documents, url)? else {
            return Ok(None);
        };
        let mut bytes = Vec::new();
        response
            .into_reader()
            .read_to_end(&mut bytes)
            .map_err(|e| format!("could not read {}: {}", url, e))?;
        Ok(Some(bytes))
    }

    fn download(&self, url: &str, to: &Path, limit: u64) -> Result<u64, String> {
        let response = Https::fetch(&self.archives, url)?
            .ok_or_else(|| format!("{} is not there (404)", url))?;
        let mut file = std::fs::File::create(to)
            .map_err(|e| format!("cannot create {}: {}", to.display(), e))?;
        std::io::copy(&mut response.into_reader().take(limit + 1), &mut file)
            .map_err(|e| format!("could not download {} to {}: {}", url, to.display(), e))
    }
}

/// Fetches the charter of the release registry `line` names and verifies it; then BoxMux's
/// manifest and its signature from where the charter says, verifies the one against the other,
/// and returns the latest release of `line`'s channel for `platform`: `None` when nothing is
/// published on it yet. A registry with no charter is not read at all.
pub fn fetch_latest(
    transport: &dyn Transport,
    line: &Line,
    platform: &str,
    now: u64,
) -> Result<Option<Release>, Problem> {
    let get = |url: &str| transport.get(url).map_err(Problem::Failed);
    let charter_url = format!("{}/{}", line.registry, mg_srel::CHARTER_FILE);
    let charter = get(&charter_url)?.ok_or_else(|| {
        Problem::Failed(format!(
            "{} has no charter ({} is missing), so it is not a release registry this boxmux can read",
            line.registry, charter_url
        ))
    })?;
    let charter_sig_url = format!("{}/{}", line.registry, mg_srel::CHARTER_SIG_FILE);
    let charter_sig = get(&charter_sig_url)?.ok_or_else(|| {
        refused(
            "charter_unsigned",
            format!(
                "{} has a charter and no signature for it ({} is missing)",
                line.registry, charter_sig_url
            ),
        )
    })?;
    let url = verify_charter(&charter, &charter_sig, line, now)?;
    let Some(manifest) = get(&url)? else {
        return Ok(None);
    };
    let signature_url = format!("{}.sig", url);
    let signature = get(&signature_url)?.ok_or_else(|| {
        refused(
            "manifest_unsigned",
            format!(
                "{} is published without a signature ({} is missing), so nothing in it can be trusted",
                url, signature_url
            ),
        )
    })?;
    verify_manifest(&manifest, &signature, &url, line, now)?;
    latest(&manifest, &line.channel, platform)
}

/// Downloads `release`'s archive into `directory`, and holds it to the size and SHA-256 the
/// signed manifest gives: anything else is not the archive that was published.
pub fn download(
    transport: &dyn Transport,
    release: &Release,
    directory: &Path,
) -> Result<PathBuf, Problem> {
    let archive = directory.join(&release.name);
    let written = transport
        .download(&release.url, &archive, release.size)
        .map_err(Problem::Failed)?;
    if written > release.size {
        return Err(refused(
            "archive_size_mismatch",
            format!(
                "{} is longer than the {} bytes the signed manifest says",
                release.url, release.size
            ),
        ));
    }
    let mut file = std::fs::File::open(&archive)
        .map_err(|e| Problem::Failed(format!("cannot read {}: {}", archive.display(), e)))?;
    let mut hasher = sha2::Sha256::new();
    let size = std::io::copy(&mut file, &mut hasher)
        .map_err(|e| Problem::Failed(format!("cannot read {}: {}", archive.display(), e)))?;
    if size != release.size {
        return Err(refused(
            "archive_size_mismatch",
            format!(
                "{} is {} bytes; the signed manifest says {}",
                release.url, size, release.size
            ),
        ));
    }
    let sha256: String = hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect();
    if sha256 != release.sha256 {
        return Err(refused(
            "archive_sha256_mismatch",
            format!(
                "{} has SHA-256 {}; the signed manifest says {}",
                release.url, sha256, release.sha256
            ),
        ));
    }
    Ok(archive)
}

/// Why an entry named `path` is not one a release archive may hold, or `None` when it may: it
/// is under the one directory [`ARCHIVE_ROOT`], every part of it a plain name, and no part an
/// AppleDouble file (`._<name>`), which macOS's tar writes beside a file carrying extended
/// attributes and which belongs to no release.
pub fn misplaced(path: &Path) -> Option<String> {
    let mut parts = path.components();
    let rooted = matches!(parts.next(), Some(Component::Normal(first)) if first == ARCHIVE_ROOT)
        && parts.all(|part| matches!(part, Component::Normal(_)));
    if !rooted {
        return Some(format!(
            "{}, which is not under the one directory {}/",
            path.display(),
            ARCHIVE_ROOT
        ));
    }
    let apple_double = path.components().any(|part| match part {
        Component::Normal(name) => name.to_string_lossy().starts_with("._"),
        _ => false,
    });
    if apple_double {
        return Some(format!(
            "{}, an AppleDouble file of macOS's extended attributes, which no release holds",
            path.display()
        ));
    }
    None
}

/// Unpacks `archive`, of the kind `kind`, into `directory`, and answers the folder it holds the
/// release in: [`ARCHIVE_ROOT`], the one directory every entry is under. Only directories and
/// regular files are taken, none outside it, and no AppleDouble file.
pub fn unpack(archive: &Path, kind: Archive, directory: &Path) -> Result<PathBuf, Problem> {
    std::fs::create_dir_all(directory)
        .map_err(|e| Problem::Failed(format!("cannot create {}: {}", directory.display(), e)))?;
    match kind {
        Archive::TarGz => unpack_tar_gz(archive, directory)?,
        Archive::Zip => unpack_zip(archive, directory)?,
    }
    let root = directory.join(ARCHIVE_ROOT);
    if !root.is_dir() {
        return Err(refused(
            "archive_layout",
            format!("{} holds no {}/ directory", archive.display(), ARCHIVE_ROOT),
        ));
    }
    Ok(root)
}

fn layout(archive: &Path, why: String) -> Problem {
    refused(
        "archive_layout",
        format!("{} holds {}", archive.display(), why),
    )
}

fn unpack_tar_gz(archive: &Path, directory: &Path) -> Result<(), Problem> {
    let file = std::fs::File::open(archive)
        .map_err(|e| Problem::Failed(format!("cannot read {}: {}", archive.display(), e)))?;
    let mut entries = tar::Archive::new(flate2::read::GzDecoder::new(file));
    let unreadable = |e: std::io::Error| {
        refused(
            "archive_layout",
            format!("{} is not a gzipped tar archive: {}", archive.display(), e),
        )
    };
    for entry in entries.entries().map_err(unreadable)? {
        let mut entry = entry.map_err(unreadable)?;
        let path = entry.path().map_err(unreadable)?.into_owned();
        if let Some(why) = misplaced(&path) {
            return Err(layout(archive, why));
        }
        let kind = entry.header().entry_type();
        if !(kind.is_file() || kind.is_dir()) {
            return Err(layout(
                archive,
                format!(
                    "{} as a {:?}: only directories and files are taken",
                    path.display(),
                    kind
                ),
            ));
        }
        entry.unpack_in(directory).map_err(|e| {
            Problem::Failed(format!(
                "cannot unpack {} into {}: {}",
                path.display(),
                directory.display(),
                e
            ))
        })?;
    }
    Ok(())
}

fn unpack_zip(archive: &Path, directory: &Path) -> Result<(), Problem> {
    let file = std::fs::File::open(archive)
        .map_err(|e| Problem::Failed(format!("cannot read {}: {}", archive.display(), e)))?;
    let unreadable = |e: zip::result::ZipError| {
        refused(
            "archive_layout",
            format!("{} is not a zip archive: {}", archive.display(), e),
        )
    };
    let mut zip = zip::ZipArchive::new(file).map_err(unreadable)?;
    for index in 0..zip.len() {
        let mut entry = zip.by_index(index).map_err(unreadable)?;
        let name = entry.name().to_string();
        let Some(path) = entry.enclosed_name() else {
            return Err(layout(
                archive,
                format!(
                    "{}, which is not under the one directory {}/",
                    name, ARCHIVE_ROOT
                ),
            ));
        };
        if let Some(why) = misplaced(&path) {
            return Err(layout(archive, why));
        }
        if entry.is_symlink() || !(entry.is_file() || entry.is_dir()) {
            return Err(layout(
                archive,
                format!("{} as a link: only directories and files are taken", name),
            ));
        }
        let to = directory.join(&path);
        let failed = |e: std::io::Error| {
            Problem::Failed(format!(
                "cannot unpack {} into {}: {}",
                name,
                directory.display(),
                e
            ))
        };
        if entry.is_dir() {
            std::fs::create_dir_all(&to).map_err(failed)?;
            continue;
        }
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent).map_err(failed)?;
        }
        let mut out = std::fs::File::create(&to).map_err(failed)?;
        std::io::copy(&mut entry, &mut out).map_err(failed)?;
    }
    Ok(())
}

/// Whether `candidate` is a later version than `current`: both are
/// `<major>.<minor>.<patch>`, numbers each.
pub fn later(candidate: &str, current: &str) -> Result<bool, String> {
    let parse = |version: &str| -> Option<[u64; 3]> {
        let parts: Vec<u64> = version
            .split('.')
            .map(|part| part.parse().ok())
            .collect::<Option<_>>()?;
        parts.try_into().ok()
    };
    let read = |version: &str, whose: &str| {
        parse(version).ok_or_else(|| {
            format!(
                "{} version {:?} is not <major>.<minor>.<patch>",
                whose, version
            )
        })
    };
    Ok(read(candidate, "the published")? > read(current, "the installed")?)
}
