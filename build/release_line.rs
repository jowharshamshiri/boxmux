//! The release line a released BoxMux carries, as its build is given it.
//!
//! A released BoxMux knows five things about its own release line, all given by the release
//! build and none read at run time:
//!
//! - [`CHANNEL`]: the channel it is published on (`release` or `nightly`);
//! - [`REGISTRY`]: the anchor of the release registry it is published in, the URL the registry
//!   is named by, under which its charter says where its manifest is;
//! - [`ENVIRONMENT`]: the environment the registry's release key is certified for (`prod` or
//!   `staging`);
//! - [`ROOTS`]: the root public keys a release-key certificate must be signed by,
//!   comma-separated (at least three, of which two must sign);
//! - [`CHARTER_ROOTS`]: the root public keys a registry's charter must chain to,
//!   comma-separated (at least three, of which two must sign).
//!
//! The names are the release registry's, the same for every product published in it, and this
//! is the one place BoxMux names them.
//!
//! The channel is what makes a build a release: nothing is published without one. A build
//! given no channel is a build from source, whatever else is set around it: it has no release
//! line, and `boxmux upgrade` says so. A build given a channel is given the others with it, or
//! is refused, because a BoxMux that knew where to look for releases and not how to trust them
//! would have to either refuse every upgrade or install what it cannot check.

pub const CHANNEL: &str = "BAHRAMJ_RELEASE_CHANNEL";
pub const REGISTRY: &str = "BAHRAMJ_RELEASES_REGISTRY_URL";
pub const ENVIRONMENT: &str = "BAHRAMJ_SIGNING_ENVIRONMENT";
pub const ROOTS: &str = "BAHRAMJ_ROOT_PUBKEYS";
pub const CHARTER_ROOTS: &str = "BAHRAMJ_CHARTER_ROOT_PUBKEYS";
/// In the order the baked line holds them.
pub const INPUTS: [&str; 5] = [REGISTRY, CHANNEL, ENVIRONMENT, ROOTS, CHARTER_ROOTS];
/// The variable the build bakes the line into, which `boxmux upgrade` reads.
pub const BAKED: &str = "BOXMUX_BAKED_RELEASE_LINE";
/// The roots a certificate is checked against: two of them must have signed it, so fewer than
/// three leaves no root that can be lost or withheld.
const MIN_ROOTS: usize = 3;

/// The release line `get` gives, as one value: registry, channel, environment, the roots and
/// the charter roots (each comma-separated), with a space between them; `None` for a build
/// from source, given no channel. A channel given without the rest, or any part not of its
/// shape, is an error naming the variable.
pub fn read(get: &dyn Fn(&str) -> Option<String>) -> Result<Option<String>, String> {
    let given: Vec<(&str, String)> = INPUTS
        .iter()
        .map(|name| (*name, get(name).unwrap_or_default().trim().to_string()))
        .collect();
    let value = |name: &str| {
        given
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, v)| v.clone())
            .expect("an input")
    };
    if value(CHANNEL).is_empty() {
        return Ok(None);
    }
    let missing: Vec<&str> = given
        .iter()
        .filter(|(_, v)| v.is_empty())
        .map(|(n, _)| *n)
        .collect();
    if !missing.is_empty() {
        let others: Vec<&str> = INPUTS.iter().filter(|n| **n != CHANNEL).copied().collect();
        return Err(format!(
            "a release build of BoxMux ({} is set) is given {} with it; this one is not given {}",
            CHANNEL,
            others.join(", "),
            missing.join(", ")
        ));
    }
    let url = value(REGISTRY);
    if !url.starts_with("https://") || url.ends_with('/') || url.contains(char::is_whitespace) {
        return Err(format!(
            "{} must be an https URL with no trailing slash and no spaces, got {:?}",
            REGISTRY, url
        ));
    }
    let channel = value(CHANNEL);
    if !matches!(channel.as_str(), "release" | "nightly") {
        return Err(format!(
            "{} must be `release` or `nightly`, got {:?}",
            CHANNEL, channel
        ));
    }
    let environment = value(ENVIRONMENT);
    if !matches!(environment.as_str(), "prod" | "staging") {
        return Err(format!(
            "{} must be `prod` or `staging` for a release build, got {:?}",
            ENVIRONMENT, environment
        ));
    }
    let roots = root_set(ROOTS, &value(ROOTS))?;
    let charter_roots = root_set(CHARTER_ROOTS, &value(CHARTER_ROOTS))?;
    // None of the parts can hold a space: the checks above have seen to the first three, and
    // base64 has none.
    Ok(Some(format!(
        "{} {} {} {} {}",
        url,
        channel,
        environment,
        roots.join(","),
        charter_roots.join(",")
    )))
}

/// A root set as `name` gives it: distinct minisign public keys, at least [`MIN_ROOTS`] of
/// them. Checked at build so that a mistyped root stops the build, not every upgrade of what
/// was built.
fn root_set(name: &str, given: &str) -> Result<Vec<String>, String> {
    let roots: Vec<String> = given.split(',').map(|key| key.trim().to_string()).collect();
    if roots.iter().any(String::is_empty) {
        return Err(format!("{} has an empty entry: {:?}", name, roots));
    }
    let mut ids = Vec::new();
    for root in &roots {
        match mg_srel::PublicKey::parse(root) {
            Ok(key) if ids.contains(&key.key_id) => {
                return Err(format!("{} names the key {} twice", name, key.key_id))
            }
            Ok(key) => ids.push(key.key_id),
            Err(why) => {
                return Err(format!(
                    "{} entry {:?} is not a minisign public key: {}",
                    name, root, why
                ))
            }
        }
    }
    if roots.len() < MIN_ROOTS {
        return Err(format!(
            "{} has {} root(s); a certificate needs two of at least {}",
            name,
            roots.len(),
            MIN_ROOTS
        ));
    }
    Ok(roots)
}
