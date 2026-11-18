//! A release line the tests publish themselves, signed with real ed25519 keys exactly as a
//! publisher signs one: BoxMux verifies and never signs, so the signing lives with the tests.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::Write as _;
use std::path::Path;

use ed25519_dalek::Signer;

use super::release::{archive_name, charter_rules, Line, Transport, APP_ID, ARCHIVE_ROOT, WORDS};

/// Issue and expiry of every certificate the tests make, and the clock they verify at.
pub const ISSUED_AT: u64 = 1_700_000_000;
pub const NOT_AFTER: u64 = 1_800_000_000;
pub const NOW: u64 = 1_750_000_000;
/// The anchor the tests' registry is named by.
pub const REGISTRY: &str = "https://release.example";

fn encode(bytes: &[u8]) -> String {
    crate::base64::encode(bytes)
}

/// A minisign keypair, made from a seed so a test is the same on every run.
pub struct Key {
    signing: ed25519_dalek::SigningKey,
    number: [u8; 8],
}

impl Key {
    pub fn new(seed: u8) -> Key {
        Key {
            signing: ed25519_dalek::SigningKey::from_bytes(&[seed; 32]),
            number: [seed, 0xa5, seed ^ 0x5a, 1, 2, 3, 4, seed.wrapping_add(1)],
        }
    }

    /// The base64 minisign public key.
    pub fn public(&self) -> String {
        let mut bytes = b"Ed".to_vec();
        bytes.extend_from_slice(&self.number);
        bytes.extend_from_slice(self.signing.verifying_key().as_bytes());
        encode(&bytes)
    }

    /// The minisign key number, as lowercase hex.
    pub fn id(&self) -> String {
        self.number.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// A base64 raw ed25519 signature over `bytes`.
    pub fn sign(&self, bytes: &[u8]) -> String {
        encode(&self.signing.sign(bytes).to_bytes())
    }
}

/// A publisher of a release line: three roots, which certify one release key, which signs the
/// charter and the manifest.
pub struct Publisher {
    pub roots: [Key; 3],
    pub release: Key,
}

impl Publisher {
    pub fn new() -> Publisher {
        Publisher {
            roots: [Key::new(1), Key::new(2), Key::new(3)],
            release: Key::new(9),
        }
    }

    /// Another publisher's: roots and a release key of its own.
    pub fn foreign() -> Publisher {
        Publisher {
            roots: [Key::new(21), Key::new(22), Key::new(23)],
            release: Key::new(29),
        }
    }

    pub fn root_keys(&self) -> Vec<String> {
        self.roots.iter().map(Key::public).collect()
    }

    /// The release line a BoxMux built with this publisher's roots carries.
    pub fn line(&self, channel: &str, environment: &str) -> Line {
        Line {
            registry: REGISTRY.to_string(),
            channel: channel.to_string(),
            environment: environment.to_string(),
            roots: self.root_keys(),
            charter_roots: self.root_keys(),
        }
    }

    /// A certificate for `release` under `label`, as its exact JSON.
    pub fn certificate(release: &Key, label: &str) -> String {
        serde_json::json!({
            "format": WORDS.certificate,
            "release_pubkey": release.public(),
            "key_id": release.id(),
            "environment": label,
            "issued_at": ISSUED_AT,
            "not_after": NOT_AFTER,
        })
        .to_string()
    }

    /// An envelope of `format` over `bytes`, its signature under `field`, signed by `signer`
    /// and carrying `certificate` signed by `roots`.
    pub fn envelope(
        format: &str,
        field: &str,
        certificate: &str,
        roots: &[&Key],
        signer: &Key,
        bytes: &[u8],
    ) -> Vec<u8> {
        let root_signatures: Vec<serde_json::Value> = roots
            .iter()
            .map(|root| {
                serde_json::json!({
                    "root_key_id": root.id(),
                    "signature": root.sign(certificate.as_bytes()),
                })
            })
            .collect();
        let mut document = serde_json::json!({
            "format": format,
            "certificates": [{ "certificate": certificate, "root_signatures": root_signatures }],
        });
        document[field] =
            serde_json::json!({ "key_id": signer.id(), "signature": signer.sign(bytes) });
        document.to_string().into_bytes()
    }

    /// `manifest`'s signature, under a certificate for `label` two roots signed.
    pub fn sign(&self, manifest: &[u8], label: &str) -> Vec<u8> {
        let certificate = Publisher::certificate(&self.release, label);
        let roots = [&self.roots[0], &self.roots[1]];
        Publisher::envelope(
            WORDS.manifest_envelope,
            "manifest_signature",
            &certificate,
            &roots,
            &self.release,
            manifest,
        )
    }

    /// A charter of the release registry at `anchor`, its contents signed under
    /// `environment`, keeping each product's manifest at `<anchor>/<app>/manifest`.
    pub fn charter(anchor: &str, environment: &str) -> Vec<u8> {
        let charter = mg_srel::Charter {
            format: WORDS.charter.to_string(),
            anchor: anchor.to_string(),
            kind: "releases".to_string(),
            slug: "release.example".to_string(),
            environment: environment.to_string(),
            experiment: None,
            locations: BTreeMap::from([(
                "manifest".to_string(),
                format!("{}/{{app_id}}/manifest", anchor),
            )]),
            fields: BTreeMap::new(),
            issued_at: ISSUED_AT,
            not_after: None,
        };
        charter.to_bytes(&charter_rules())
    }

    /// `charter`'s signature, under a prod certificate two roots signed.
    pub fn sign_charter(&self, charter: &[u8]) -> Vec<u8> {
        let certificate = Publisher::certificate(&self.release, "prod");
        let roots = [&self.roots[0], &self.roots[2]];
        Publisher::envelope(
            WORDS.charter_envelope,
            "charter_signature",
            &certificate,
            &roots,
            &self.release,
            charter,
        )
    }
}

/// The installer type the manifest lists `platform`'s archive under.
fn installer_type(platform: &str) -> &'static str {
    super::release::Archive::of(platform).installer_type()
}

/// A manifest of schema 5 publishing `version` on `channel` for `platform`, its archive at
/// `url` with `sha256` and `size`; the other channel empty.
pub fn manifest(
    channel: &str,
    version: &str,
    platform: &str,
    url: &str,
    sha256: &str,
    size: u64,
) -> Vec<u8> {
    let empty = serde_json::json!({ "latestVersion": null, "versions": {} });
    let published = serde_json::json!({
        "latestVersion": version,
        "versions": { version: {
            "publishedAt": "2026-11-17T00:00:00Z",
            "notesUrl": format!("{}/{}/{}/{}/notes.md", REGISTRY, APP_ID, channel, version),
            "platforms": { platform: { "installers": [{
                "type": installer_type(platform),
                "name": archive_name(version, platform),
                "path": url,
                "sha256": sha256,
                "size": size,
            }] } },
        } },
    });
    let mut channels = serde_json::json!({ "release": empty.clone(), "nightly": empty });
    channels[channel] = published;
    serde_json::json!({
        "schemaVersion": "5.0",
        "appId": APP_ID,
        "name": "BoxMux",
        "description": "",
        "lastUpdated": "2026-11-17T00:00:00Z",
        "channels": channels,
    })
    .to_string()
    .into_bytes()
}

/// A gzipped tar archive of `files` (a path under the archive and its bytes, each mode 0755).
pub fn tar_gz(files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut builder = tar::Builder::new(flate2::write::GzEncoder::new(
        Vec::new(),
        flate2::Compression::default(),
    ));
    for (path, bytes) in files {
        let mut header = tar::Header::new_gnu();
        header.set_size(bytes.len() as u64);
        header.set_mode(0o755);
        header.set_entry_type(tar::EntryType::Regular);
        builder
            .append_data(&mut header, path, *bytes)
            .expect("an entry is appended");
    }
    builder
        .into_inner()
        .expect("the tar stream ends")
        .finish()
        .expect("the gzip stream ends")
}

/// A zip archive of `files` (a path under the archive and its bytes).
pub fn zip(files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (path, bytes) in files {
        writer
            .start_file(*path, zip::write::SimpleFileOptions::default())
            .expect("an entry is started");
        writer.write_all(bytes).expect("an entry is written");
    }
    writer.finish().expect("the zip ends").into_inner()
}

/// A program that says it is BoxMux `version`, as `boxmux --version` does.
pub fn program(version: &str) -> Vec<u8> {
    format!(
        "#!/bin/sh\necho 'boxmux {} commit unknown (built without git metadata)'\n",
        version
    )
    .into_bytes()
}

/// BoxMux `version` as its archive for a Unix platform holds it.
pub fn release_archive(version: &str) -> Vec<u8> {
    let at = |name: &str| format!("{}/{}", ARCHIVE_ROOT, name);
    let (program_at, licence_at, readme_at) = (at("boxmux"), at("LICENSE"), at("README.md"));
    tar_gz(&[
        (&program_at, &program(version)),
        (&licence_at, b"MIT"),
        (&readme_at, b"# BoxMux"),
    ])
}

pub fn sha256(bytes: &[u8]) -> String {
    use sha2::Digest;
    sha2::Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

/// A registry served from a table: every URL asked for, in order, and what is there.
#[derive(Default)]
pub struct Served {
    pub files: RefCell<BTreeMap<String, Vec<u8>>>,
    pub asked: RefCell<Vec<String>>,
}

impl Served {
    pub fn put(&self, url: &str, bytes: Vec<u8>) {
        self.files.borrow_mut().insert(url.to_string(), bytes);
    }

    pub fn remove(&self, url: &str) {
        self.files.borrow_mut().remove(url);
    }

    /// The URL BoxMux `version`'s archive for `platform` is published at on `channel`.
    pub fn archive_url(channel: &str, version: &str, platform: &str) -> String {
        format!(
            "{}/{}/{}/{}/{}",
            REGISTRY,
            APP_ID,
            channel,
            version,
            archive_name(version, platform)
        )
    }

    pub fn manifest_url() -> String {
        format!("{}/{}/manifest", REGISTRY, APP_ID)
    }

    /// A whole release line: the charter of [`REGISTRY`] and its signature, BoxMux `version`
    /// on `channel` for `platform`, signed under `label`, and its archive `archive`, as
    /// `publisher` publishes them.
    pub fn publish(
        &self,
        publisher: &Publisher,
        label: &str,
        channel: &str,
        version: &str,
        platform: &str,
        archive: Vec<u8>,
    ) {
        let charter = Publisher::charter(REGISTRY, label);
        self.put(
            &format!("{}/charter.json.sig", REGISTRY),
            publisher.sign_charter(&charter),
        );
        self.put(&format!("{}/charter.json", REGISTRY), charter);
        let url = Served::archive_url(channel, version, platform);
        let manifest = manifest(
            channel,
            version,
            platform,
            &url,
            &sha256(&archive),
            archive.len() as u64,
        );
        self.put(
            &format!("{}.sig", Served::manifest_url()),
            publisher.sign(&manifest, label),
        );
        self.put(&Served::manifest_url(), manifest);
        self.put(&url, archive);
    }
}

impl Transport for Served {
    fn get(&self, url: &str) -> Result<Option<Vec<u8>>, String> {
        self.asked.borrow_mut().push(url.to_string());
        Ok(self.files.borrow().get(url).cloned())
    }

    fn download(&self, url: &str, to: &Path, limit: u64) -> Result<u64, String> {
        self.asked.borrow_mut().push(url.to_string());
        let bytes = self
            .files
            .borrow()
            .get(url)
            .cloned()
            .ok_or_else(|| format!("{} is not there (404)", url))?;
        let end = bytes
            .len()
            .min(usize::try_from(limit + 1).expect("a test's limit fits"));
        std::fs::write(to, &bytes[..end])
            .map_err(|e| format!("cannot write {}: {}", to.display(), e))?;
        Ok(end as u64)
    }
}
