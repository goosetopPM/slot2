use std::fmt;
use std::path::Path;

/// Where BaseOS records which device image this is. `BASEOS_TARGET=rgsp` and so on.
pub const RELEASE_FILE: &str = "/etc/baseos-release";

/// The three panel shapes left once RG28XX (480x640, rotated) is out of scope. Every UI
/// layout question reduces to which of these the panel is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Geometry {
    /// RG35XX Plus/H/Pro/SP, RG40XX H/V. 4:3.
    W640H480,
    /// RG SP, RG34XX, RG34XX SP. 3:2 — exactly 3x the GBA's 240x160.
    W720H480,
    /// RG CubeXX. 1:1.
    W720H720,
}

impl Geometry {
    pub const fn size(self) -> (u32, u32) {
        match self {
            Geometry::W640H480 => (640, 480),
            Geometry::W720H480 => (720, 480),
            Geometry::W720H720 => (720, 720),
        }
    }

    /// The 640x480 safe area every geometry contains, placed at the panel's centre. UI is
    /// laid out inside it; the rest is backdrop and shelf continuing past its edges.
    pub const fn safe_area_offset(self) -> (u32, u32) {
        let (w, h) = self.size();
        ((w - 640) / 2, (h - 480) / 2)
    }

    /// `640x480` and friends, as `SLOT2_GEOMETRY` spells them.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "640x480" => Some(Geometry::W640H480),
            "720x480" => Some(Geometry::W720H480),
            "720x720" => Some(Geometry::W720H720),
            _ => None,
        }
    }
}

impl fmt::Display for Geometry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (w, h) = self.size();
        write!(f, "{w}x{h}")
    }
}

/// One device image's worth of facts. Sysfs paths join this once a device has been probed
/// (V-2 in the design doc); until then every profile assumes the H700 defaults.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Profile {
    /// The `BASEOS_TARGET` id, or `host`.
    pub target: &'static str,
    pub geometry: Geometry,
    /// SP-style clamshells: closing the lid should save and sleep.
    pub has_lid: bool,
    /// Analogue sticks are read as generic axes now and mapped to nothing yet (D-15).
    pub has_sticks: bool,
}

/// BaseOS's `devices.json`, minus RG28XX. Order does not matter; lookup is by id.
pub const PROFILES: &[Profile] = &[
    Profile {
        target: "rgsp",
        geometry: Geometry::W720H480,
        has_lid: true,
        has_sticks: false,
    },
    Profile {
        target: "rg34xx",
        geometry: Geometry::W720H480,
        has_lid: false,
        has_sticks: false,
    },
    Profile {
        target: "rg34xxsp",
        geometry: Geometry::W720H480,
        has_lid: true,
        has_sticks: false,
    },
    Profile {
        target: "rg35xxsp",
        geometry: Geometry::W640H480,
        has_lid: true,
        has_sticks: false,
    },
    Profile {
        target: "rg35xxplus",
        geometry: Geometry::W640H480,
        has_lid: false,
        has_sticks: false,
    },
    Profile {
        target: "rg35xxh",
        geometry: Geometry::W640H480,
        has_lid: false,
        has_sticks: true,
    },
    Profile {
        target: "rg35xxpro",
        geometry: Geometry::W640H480,
        has_lid: false,
        has_sticks: false,
    },
    Profile {
        target: "rg40xxh",
        geometry: Geometry::W640H480,
        has_lid: false,
        has_sticks: true,
    },
    Profile {
        target: "rg40xxv",
        geometry: Geometry::W640H480,
        has_lid: false,
        has_sticks: true,
    },
    Profile {
        target: "rgcubexx",
        geometry: Geometry::W720H720,
        has_lid: false,
        has_sticks: true,
    },
];

/// What an unknown id gets: the most common panel, and nothing that could misfire (a lid
/// that is not there would never close, but a stick that is not there never moves either —
/// the conservative choice is simply "no").
const UNKNOWN: Profile = Profile {
    target: "unknown",
    geometry: Geometry::W640H480,
    has_lid: false,
    has_sticks: false,
};

/// The host's stand-in: the development device's shape unless `SLOT2_GEOMETRY` says otherwise.
const HOST: Profile = Profile {
    target: "host",
    geometry: Geometry::W720H480,
    has_lid: true,
    has_sticks: false,
};

pub fn by_target(id: &str) -> Option<Profile> {
    PROFILES
        .iter()
        .copied()
        .find(|p| p.target.eq_ignore_ascii_case(id))
}

/// Read `BASEOS_TARGET=` out of a baseos-release file's text.
pub fn parse_release(text: &str) -> Option<&str> {
    text.lines()
        .map(str::trim)
        .find_map(|l| l.strip_prefix("BASEOS_TARGET="))
        .map(|v| v.trim().trim_matches('"'))
        .filter(|v| !v.is_empty())
}

/// Decide the profile, in this order: `SLOT2_TARGET` (host and tests), then the BaseOS
/// release file, then the unknown fallback. `SLOT2_GEOMETRY` then overrides the panel on
/// whatever was chosen. `report` says which path was taken, for the boot log.
pub fn detect() -> Detected {
    detect_from(
        std::env::var("SLOT2_TARGET").ok().as_deref(),
        std::env::var("SLOT2_GEOMETRY").ok().as_deref(),
        Path::new(RELEASE_FILE),
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Detected {
    pub profile: Profile,
    /// Where the id came from: `env`, `baseos-release`, `host`, or `unknown(<id>)`.
    pub source: String,
}

fn detect_from(env_target: Option<&str>, env_geometry: Option<&str>, release: &Path) -> Detected {
    let mut d = match env_target {
        Some(id) if id.eq_ignore_ascii_case("host") => Detected {
            profile: HOST,
            source: "env".into(),
        },
        Some(id) => match by_target(id) {
            Some(p) => Detected {
                profile: p,
                source: "env".into(),
            },
            None => Detected {
                profile: UNKNOWN,
                source: format!("unknown({id})"),
            },
        },
        None => match std::fs::read_to_string(release) {
            Ok(text) => match parse_release(&text) {
                Some(id) => match by_target(id) {
                    Some(p) => Detected {
                        profile: p,
                        source: "baseos-release".into(),
                    },
                    None => Detected {
                        profile: UNKNOWN,
                        source: format!("unknown({id})"),
                    },
                },
                None => Detected {
                    profile: UNKNOWN,
                    source: "baseos-release(no target)".into(),
                },
            },
            // No release file at all: not BaseOS, so almost certainly a developer's machine.
            Err(_) => Detected {
                profile: HOST,
                source: "host".into(),
            },
        },
    };
    if let Some(g) = env_geometry.and_then(Geometry::parse) {
        d.profile.geometry = g;
        d.source.push_str("+geometry");
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_profile_contains_the_safe_area() {
        for p in PROFILES {
            let (w, h) = p.geometry.size();
            assert!(w >= 640 && h >= 480, "{}", p.target);
        }
    }

    #[test]
    fn safe_area_is_centred() {
        assert_eq!(Geometry::W640H480.safe_area_offset(), (0, 0));
        assert_eq!(Geometry::W720H480.safe_area_offset(), (40, 0));
        assert_eq!(Geometry::W720H720.safe_area_offset(), (40, 120));
    }

    #[test]
    fn release_file_is_parsed_loosely() {
        assert_eq!(parse_release("BASEOS_TARGET=rgsp\n"), Some("rgsp"));
        assert_eq!(
            parse_release("X=1\n  BASEOS_TARGET=\"rg40xxh\" \n"),
            Some("rg40xxh")
        );
        assert_eq!(parse_release("BASEOS_TARGET=\n"), None);
        assert_eq!(parse_release(""), None);
    }

    #[test]
    fn env_target_wins_and_geometry_overrides() {
        let d = detect_from(Some("rgsp"), Some("720x720"), Path::new("/nonexistent"));
        assert_eq!(d.profile.target, "rgsp");
        assert_eq!(d.profile.geometry, Geometry::W720H720);
        assert!(d.profile.has_lid);
        assert_eq!(d.source, "env+geometry");
    }

    #[test]
    fn missing_release_file_means_host() {
        let d = detect_from(None, None, Path::new("/nonexistent"));
        assert_eq!(d.profile.target, "host");
    }

    #[test]
    fn unknown_id_falls_back_without_a_lid() {
        let d = detect_from(Some("rg99xx"), None, Path::new("/nonexistent"));
        assert_eq!(d.profile, UNKNOWN);
        assert_eq!(d.source, "unknown(rg99xx)");
    }
}
