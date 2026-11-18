//! The platforms BoxMux is released for, under the release registry's names, which are the
//! updater's own (`boxmux_lib::upgrade::release`).

use crate::Result;
use boxmux_lib::upgrade::release;

#[derive(Debug)]
pub struct Platform {
    /// The release registry's name for it.
    pub id: &'static str,
    /// The Rust target the program is built for, on a machine of that target.
    pub target: &'static str,
    /// The oldest macOS the program runs on.
    pub macos_deployment_target: Option<&'static str>,
}

impl Platform {
    /// The program's name in the archive.
    pub fn program(&self) -> &'static str {
        release::program_name(self.id)
    }
}

pub const PLATFORMS: &[Platform] = &[
    Platform {
        id: "linux-x86_64",
        target: "x86_64-unknown-linux-gnu",
        macos_deployment_target: None,
    },
    Platform {
        id: "linux-arm64",
        target: "aarch64-unknown-linux-gnu",
        macos_deployment_target: None,
    },
    Platform {
        id: "darwin-arm64",
        target: "aarch64-apple-darwin",
        macos_deployment_target: Some("12.0"),
    },
    Platform {
        id: "windows-x86_64",
        target: "x86_64-pc-windows-msvc",
        macos_deployment_target: None,
    },
];

pub fn named(id: &str) -> Result<&'static Platform> {
    PLATFORMS.iter().find(|p| p.id == id).ok_or_else(|| {
        let known: Vec<&str> = PLATFORMS.iter().map(|p| p.id).collect();
        format!(
            "{} is not a platform BoxMux is released for (platforms: {})",
            id,
            known.join(", ")
        )
    })
}

/// Refuses a platform this machine is not: each is built natively, on the platform the
/// updater it carries will run on.
pub fn built_here(platform: &Platform) -> Result<()> {
    match release::platform() {
        Some(here) if here == platform.id => Ok(()),
        Some(here) => Err(format!(
            "{} is built on a {} machine, and this one is {}",
            platform.id, platform.id, here
        )),
        None => Err(format!(
            "{} is built on a {} machine, and this one ({} {}) is no platform BoxMux is released for",
            platform.id,
            platform.id,
            std::env::consts::OS,
            std::env::consts::ARCH
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The platforms built are exactly those the updater looks for, each by its Rust target.
    #[test]
    fn the_platforms_built_are_the_ones_the_updater_installs() {
        for p in PLATFORMS {
            let (arch, os) = match p.target.split('-').collect::<Vec<_>>()[..] {
                [arch, _, os, ..] => (arch, os),
                _ => panic!("{} is not a target", p.target),
            };
            let os = match os {
                "darwin" => "macos",
                other => other,
            };
            assert_eq!(release::platform_of(os, arch), Some(p.id), "{}", p.target);
        }
        assert_eq!(PLATFORMS.len(), 4);
    }

    /// The machine running the tests builds its own platform's release and no other.
    #[test]
    fn another_platform_is_refused_naming_both() {
        let here =
            named(release::platform().expect("the tests run on a released platform")).unwrap();
        assert!(built_here(here).is_ok());
        for other in PLATFORMS.iter().filter(|p| p.id != here.id) {
            let e = built_here(other).unwrap_err();
            assert!(e.contains(other.id) && e.contains(here.id), "{}", e);
        }
        let e = named("freebsd-x86_64").unwrap_err();
        assert!(
            e.contains("linux-x86_64, linux-arm64, darwin-arm64, windows-x86_64"),
            "{}",
            e
        );
    }
}
