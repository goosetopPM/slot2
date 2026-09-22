//! The seven platforms and how the card names them (D-04). Emulation details (cores,
//! geometry, input maps) belong to the registry in `slot2-retro`; this is only the folder
//! and the extensions, which is all scanning needs.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Platform {
    Gb,
    Gbc,
    Gba,
    Nes,
    Snes,
    Md,
    Sms,
}

impl Platform {
    /// Shelf order: the handhelds first, then the consoles by age.
    pub const ALL: [Platform; 7] = [
        Platform::Gb,
        Platform::Gbc,
        Platform::Gba,
        Platform::Nes,
        Platform::Snes,
        Platform::Md,
        Platform::Sms,
    ];

    /// The folder under `Games/`, `Labels/`, `Saves/`, `States/`.
    pub const fn folder(self) -> &'static str {
        match self {
            Platform::Gb => "GB",
            Platform::Gbc => "GBC",
            Platform::Gba => "GBA",
            Platform::Nes => "NES",
            Platform::Snes => "SNES",
            Platform::Md => "MD",
            Platform::Sms => "SMS",
        }
    }

    /// ROM extensions, lower case, without the dot. Folder + extension is the whole
    /// platform test; no header sniffing.
    pub const fn extensions(self) -> &'static [&'static str] {
        match self {
            Platform::Gb => &["gb"],
            Platform::Gbc => &["gbc"],
            Platform::Gba => &["gba"],
            Platform::Nes => &["nes"],
            Platform::Snes => &["sfc", "smc"],
            Platform::Md => &["md", "gen", "bin"],
            Platform::Sms => &["sms"],
        }
    }

    pub fn from_folder(name: &str) -> Option<Platform> {
        Platform::ALL
            .into_iter()
            .find(|p| p.folder().eq_ignore_ascii_case(name))
    }

    pub fn accepts(self, extension: &str) -> bool {
        self.extensions()
            .iter()
            .any(|e| e.eq_ignore_ascii_case(extension))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folders_round_trip_and_extensions_are_case_insensitive() {
        for p in Platform::ALL {
            assert_eq!(Platform::from_folder(p.folder()), Some(p));
            assert_eq!(Platform::from_folder(&p.folder().to_lowercase()), Some(p));
        }
        assert_eq!(Platform::from_folder("PSX"), None);
        assert!(Platform::Snes.accepts("SFC"));
        assert!(Platform::Md.accepts("bin"));
        assert!(!Platform::Gba.accepts("zip"));
    }
}
