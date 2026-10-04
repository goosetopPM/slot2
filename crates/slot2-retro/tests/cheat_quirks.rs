//! The cheat validator's contract: which core runs which console, the forms each pinned
//! adapter really hands its parser, and the honest `Unchecked` for every core whose syntax this
//! crate has not read.
//!
//! Pure functions only: no core is loaded and no file is read, so this file needs nothing from
//! the vendor directory and never skips.

use slot2_retro::{
    cheat_delivery, validate_cheat, CheatDelivery, CheatValidation as V, CheatValidationError,
    CoreId, Platform, PLATFORMS,
};

const ALL_PLATFORMS: [Platform; 7] = [
    Platform::Gb,
    Platform::Gbc,
    Platform::Gba,
    Platform::Nes,
    Platform::Snes,
    Platform::Md,
    Platform::Sms,
];

/// Every pair the frontend ships, and a code that pair is known to take: `Validated` for the
/// cores with a grammar here, `Unchecked` for the ones without one yet.
const SUPPORTED: [(CoreId, Platform, &str); 10] = [
    (CoreId::Mgba, Platform::Gb, "12345678"),
    (CoreId::Mgba, Platform::Gbc, "151-91A-7FC"),
    (CoreId::Mgba, Platform::Gba, "3202B634+0080"),
    (CoreId::Gambatte, Platform::Gb, "12345678"),
    (CoreId::Gambatte, Platform::Gbc, "A1B-2C3"),
    (CoreId::Gpsp, Platform::Gba, "anything at all"),
    (CoreId::Fceumm, Platform::Nes, "1234:AB"),
    (CoreId::Snes9x, Platform::Snes, "7E007C9A"),
    (CoreId::GenesisPlusGx, Platform::Md, "anything at all"),
    (CoreId::GenesisPlusGx, Platform::Sms, "anything at all"),
];

/// The seven bytes mGBA's adapter treats as a separator: `+` and C `isspace` in the C locale.
const MGBA_SEPARATORS: [&str; 7] = ["+", " ", "\t", "\n", "\u{b}", "\u{c}", "\r"];

fn accepted(core: CoreId, platform: Platform, code: &str) -> bool {
    matches!(validate_cheat(core, platform, code), Ok(V::Validated))
}

fn refused(core: CoreId, platform: Platform, code: &str) -> CheatValidationError {
    match validate_cheat(core, platform, code) {
        Err(e) => e,
        Ok(v) => panic!("{code:?} was taken as {v:?}"),
    }
}

fn snes(code: &str) -> bool {
    accepted(CoreId::Snes9x, Platform::Snes, code)
}

fn snes_error(code: &str) -> CheatValidationError {
    refused(CoreId::Snes9x, Platform::Snes, code)
}

fn gba(code: &str) -> bool {
    accepted(CoreId::Mgba, Platform::Gba, code)
}

fn gb(platform: Platform, code: &str) -> bool {
    accepted(CoreId::Mgba, platform, code)
}

// ---------------------------------------------------------------- the support matrix

#[test]
fn the_support_matrix_is_exact_and_a_mismatch_is_refused() {
    for core in CoreId::ALL {
        for platform in ALL_PLATFORMS {
            let case = format!("{core:?} on {platform:?}");
            match SUPPORTED
                .iter()
                .find(|(c, p, _)| *c == core && *p == platform)
            {
                Some((_, _, code)) => {
                    let checked = match core {
                        CoreId::Mgba | CoreId::Fceumm | CoreId::Snes9x => V::Validated,
                        CoreId::Gambatte | CoreId::Gpsp | CoreId::GenesisPlusGx => V::Unchecked,
                    };
                    assert_eq!(validate_cheat(core, platform, code), Ok(checked), "{case}");
                }
                None => {
                    // A pair the core does not run is refused whatever the code looks like:
                    // there is no grammar to ask about, so there is nothing to get right.
                    for code in ["7E007C9A", "12345678", "anything at all"] {
                        let e = refused(core, platform, code);
                        assert_eq!(e.core(), core, "{case}");
                        assert_eq!(e.platform(), platform, "{case}");
                        assert!(!e.reason().is_empty(), "{case}");
                    }
                }
            }
        }
    }

    // The other direction: every platform's default core runs that platform, and the GBA
    // alternative is a real pair even though it is never a default.
    for d in PLATFORMS {
        assert!(
            d.default_core.supports_platform(d.platform),
            "{:?} is not supported by its own default core {:?}",
            d.platform,
            d.default_core
        );
    }
    assert!(CoreId::Gpsp.supports_platform(Platform::Gba));
    assert!(!CoreId::Gpsp.supports_platform(Platform::Gb));
    assert!(!CoreId::Gpsp.supports_platform(Platform::Gbc));
}

// ---------------------------------------------------------------- SNES9x (task 53's contract)

#[test]
fn snes9x_takes_the_three_forms_it_parses() {
    // Pro Action Replay: eight hex digits, in either case.
    for code in ["12345678", "7E000000", "abcdef12", "ABCDEF12"] {
        assert!(snes(code), "{code} is a PAR code");
    }

    // Six hex digits, a colon, two hex digits.
    for code in ["123456:78", "7E0000:01", "abCdef:CD", "000000:00"] {
        assert!(snes(code), "{code} is an address:value code");
    }

    // SNES Game Genie: eight characters of its own alphabet, in either case.
    for code in [
        "DF47-0915",
        "df47-0915",
        "Df47-0915",
        "A2E3-1B8C",
        "8E8F-6A2B",
    ] {
        assert!(snes(code), "{code} is a Game Genie code");
    }

    // The Genie alphabet is a subset of hex, so eight Genie characters with the hyphen left
    // out are also eight hex digits. The adapter tries both, so this is a PAR code as far as
    // anyone can tell — and accepting it is the right answer.
    assert!(
        snes("DF470915"),
        "eight characters from a hex-only alphabet"
    );
}

#[test]
fn snes9x_splits_on_its_own_separators_and_skips_empty_runs() {
    assert!(snes(
        "12345678+23456789,345678:90.456789AB;56789ABC 6789ABCD"
    ));

    for code in [
        "12345678++23456789",
        "++12345678++",
        "  12345678  ",
        "12345678+",
        "+12345678",
        "12345678,,,,  ;;++..23456789",
    ] {
        assert!(snes(code), "{code:?} was refused");
    }

    // Separators and nothing else: no token, so there is nothing to apply.
    for code in ["+", "   ", ",;,", ". .", "++ +"] {
        assert_eq!(snes_error(code).core(), CoreId::Snes9x, "{code:?}");
    }
}

#[test]
fn snes9x_refuses_what_it_does_not_parse() {
    for code in [
        "1234567",   // seven hex digits: no form is that long
        "123456789", // nine hex digits
        "12345678g", // not hex
        "12345:678", // the colon is not where the form puts it
        "1234567:8",
        "123456:789",
        "1234_5678", // an underscore is not the Game Genie hyphen
        "DF47_0915",
        "DF47-091G", // G is not in the alphabet
        "GGGG-0915",
        "DF-0915",
        "12345678\t1",  // a tab is not one of the five: it lands inside a token
        "12345678 1\n", // ... and so does a newline
        "1234 가 :5678",
        "１２３４５６７８", // full-width digits are not ASCII hex
    ] {
        assert_eq!(snes_error(code).core(), CoreId::Snes9x, "{code:?}");
    }
}

#[test]
fn snes9x_keeps_its_buffer_limit_in_bytes() {
    // Twenty-eight tokens of the adapter's own forms, 255 bytes exactly: the largest code
    // that fits the core's 256-byte copy once its terminator is counted.
    let mut tokens: Vec<String> = (0..24).map(|i| format!("{i:08X}")).collect();
    tokens.extend((0..4).map(|i| format!("{i:06X}:{i:02X}")));
    let code = tokens.join("+");
    assert_eq!(code.len(), 255, "the fixture is not the boundary");
    assert!(snes(&code), "a 255-byte code is inside the core's buffer");

    // One byte more — the byte the terminator needs — is out.
    let over = format!("{code}+");
    assert_eq!(over.len(), 256);
    let e = snes_error(&over);
    assert!(e.reason().contains("256"), "{}", e.reason());

    // Bytes, not characters: two hundred Hangul syllables are six hundred bytes.
    let wide = "가".repeat(200);
    assert_eq!(wide.chars().count(), 200);
    assert_eq!(wide.len(), 600);
    let e = snes_error(&wide);
    assert!(
        e.reason().contains("600"),
        "the refusal does not name the byte count: {}",
        e.reason()
    );
}

// ---------------------------------------------------------------- mGBA, GBA

#[test]
fn mgba_gba_takes_the_three_unit_forms() {
    // CodeBreaker: eight hex, a separator, four hex.
    for code in [
        "3202B634+0080",
        "82025B6C+270F",
        "abcdef01+00ff",
        "ABCDEF01+00FF",
    ] {
        assert!(gba(code), "{code} is a CodeBreaker unit");
    }
    // GameShark/PAR: eight hex, a separator, eight hex.
    for code in ["12345678+9ABCDEF0", "12345678+abcdef01"] {
        assert!(gba(code), "{code} is a GameShark unit");
    }
    // VBA 32-bit: eight hex, a colon, eight hex.
    for code in ["12345678:9ABCDEF0", "12345678:9abcdef0"] {
        assert!(gba(code), "{code} is a VBA 32-bit unit");
    }

    // The shapes the libretro database actually ships, one unit and many.
    assert!(gba("3202B634+0080"), "a real database CodeBreaker code");
    assert!(
        gba("000052DC+000A+10000F4E+0007"),
        "two units joined by one separator each"
    );
    // Four units of eight-plus-eight hex mixed with the shorter form.
    assert!(gba("12345678+9ABCDEF0+12345678+9ABCDEF0+12345678+0000"));
}

#[test]
fn mgba_gba_takes_any_of_its_seven_separators_between_and_inside_units() {
    for sep in MGBA_SEPARATORS {
        let one = format!("3202B634{sep}0080");
        assert!(gba(&one), "one unit with {sep:?}");
        let vba = "3202B634:00000080";
        assert!(gba(vba), "the colon form does not need a separator");

        let many = format!("000052DC{sep}000A{sep}10000F4E{sep}0007");
        assert!(gba(&many), "two units with {sep:?}");

        // Space and tab are separators, so they are never part of a unit.
        assert!(!gba(&format!("3202B634 {sep}0080")), "two separators");
    }
}

#[test]
fn mgba_gba_refuses_everything_else() {
    for code in [
        // Addresses of the wrong length.
        "1234567+0000",
        "123456789+0000",
        // Operands of the wrong length: three, five, six and seven hex digits.
        "12345678+000",
        "12345678+00000",
        "12345678+000000",
        "12345678+0000000",
        // The shorter VBA forms the adapter never reaches.
        "12345678:00",
        "12345678:0000",
        // Not hex, and not a separator.
        "1234567G+0000",
        "12345678+00G0",
        "12345678:0000000G",
        // A separator with nothing on one side of it, and two in a row.
        "+12345678+0000",
        "12345678+0000+",
        "12345678+0000++12345678+0000",
        " 12345678+0000",
        "12345678+0000 ",
        // A unit cut short at the end.
        "12345678+0000+1234",
        "12345678+0000+",
    ] {
        let e = refused(CoreId::Mgba, Platform::Gba, code);
        assert_eq!(e.platform(), Platform::Gba, "{code:?}");
    }
}

// ---------------------------------------------------------------- mGBA, Game Boy

#[test]
fn mgba_game_boy_takes_the_five_unit_forms_on_both_consoles() {
    for platform in [Platform::Gb, Platform::Gbc] {
        for code in [
            "12345678",                            // GameShark
            "abcdef01",                            // ... in either case
            "A1B-2C3",                             // Game Genie
            "151-91A-7FC",                         // Game Genie with compare
            "A1B2C3-4D",                           // CodeBreaker
            "A1B2:C3",                             // VBA
            "151-91A-7FC+151-93A-F7E+151-9BA-5DB", // the database's own shape
            "12345678+151-91A-7FC+A1B2C3-4D",
        ] {
            assert!(gb(platform, code), "{platform:?}: {code:?} was refused");
        }
    }

    // Every separator, between units.
    for sep in MGBA_SEPARATORS {
        let code = format!("151-91A-7FC{sep}A1B2C3-4D{sep}12345678");
        assert!(gb(Platform::Gb, &code), "{sep:?}");
    }
}

#[test]
fn mgba_game_boy_refuses_everything_else() {
    for platform in [Platform::Gb, Platform::Gbc] {
        for code in [
            // Hyphen and colon in the wrong place, or the wrong count of them.
            "15191A-7FC",
            "151-91A7FC",
            "1519-1A-7FC",
            "151:91A",
            "151-91A:7FC",
            "A1B2-C3",
            // Lengths no form has: six, nine, ten and twelve bytes.
            "15-91A",
            "151-91A-7",
            "151-91A-7FC-",
            "151-91A-7FCA",
            "A1B2C3-4DE",
            // Not hex.
            "151-91G-7FC",
            "1234567Z",
            "A1B2C3-4G",
            "A1B2:CZ",
            // A separator with nothing on one side of it, or two in a row.
            "+12345678",
            "12345678+",
            "12345678++12345678",
            " 12345678",
            "12345678 ",
            "151-91A-7FC++A1B2C3-4D",
        ] {
            let e = refused(CoreId::Mgba, platform, code);
            assert_eq!(e.platform(), platform, "{platform:?}: {code:?}");
        }
    }
}

// ---------------------------------------------------------------- FCEUmm, NES

fn nes(code: &str) -> bool {
    accepted(CoreId::Fceumm, Platform::Nes, code)
}

fn nes_error(code: &str) -> CheatValidationError {
    refused(CoreId::Fceumm, Platform::Nes, code)
}

/// The six bytes FCEUmm's adapter splits a code on.
const FCEUMM_SEPARATORS: [&str; 6] = ["+", ",", ";", ".", "_", " "];

#[test]
fn fceumm_takes_the_four_forms_it_parses() {
    // Raw: four hex, a colon, two hex.
    for code in ["1234:AB", "00ff:01", "ABCD:ef", "0000:00"] {
        assert!(nes(code), "{code} is a raw code");
    }
    // Raw with compare: four hex, a question mark, two hex, a colon, two hex.
    for code in ["1234?56:78", "00ff?00:01", "abcd?EF:12"] {
        assert!(nes(code), "{code} is a raw code with a compare byte");
    }
    // NES Game Genie: six or eight characters of its own alphabet, in either case.
    for code in ["SXIOPO", "sxiopo", "GZUXNGEI", "gzuxngei", "ApZlGi"] {
        assert!(nes(code), "{code} is a Game Genie code");
    }
    // Game Genie and Pro Action Replay: eight characters, and the two alphabets overlap on
    // the letters they share. `AAAAAAAA` is either — the adapter asks Game Genie first, so
    // that decides which parser reads it, not whether it is read at all.
    assert!(nes("AAAAAAAA"), "eight characters of the shared alphabet");
}

#[test]
fn fceumm_splits_on_its_own_separators_and_skips_empty_runs() {
    // Every one of the six, between tokens of three different forms.
    for sep in FCEUMM_SEPARATORS {
        let code = format!("1234:AB{sep}SXIOPO{sep}12345678");
        assert!(nes(&code), "three tokens with {sep:?}");
    }

    // The adapter's `strtok` skips runs at the edges and in the middle.
    for code in [
        "1234:AB++SXIOPO",
        "++1234:AB++",
        "  1234:AB  ",
        "1234:AB_",
        "_1234:AB",
        "1234:AB,,;;..__  12345678",
    ] {
        assert!(nes(code), "{code:?} was refused");
    }

    // Separators and nothing else: no token, so there is nothing to apply.
    for code in ["+", "   ", ",;", "._", "+ _ ,"] {
        assert_eq!(nes_error(code).core(), CoreId::Fceumm, "{code:?}");
    }
}

#[test]
fn fceumm_refuses_what_it_does_not_parse() {
    for code in [
        // Raw: the wrong lengths and the wrong punctuation.
        "123:AB",
        "12345:AB",
        "1234:ABC",
        "1234:",
        "1234-AB",
        "1234AB",
        // Raw with compare: the same, one field further along.
        "1234?56:7",
        "1234?5:78",
        "1234?567:8",
        "12345?6:78",
        "1234-56:78",
        "1234?56-78",
        // Game Genie: five, seven and nine characters, and letters outside the alphabet.
        "SXIOP",
        "SXIOPOZ",
        "SXIOPOZXI",
        "SXIOPQ",
        "123456",
        // Pro Action Replay: seven and nine digits, and not hex.
        "1234567",
        "123456789",
        "abcdefgh",
        // A tab and a newline are not separators: they land inside a token.
        "1234:AB\tSXIOPO",
        "1234:AB\nSXIOPO",
        "1234:AB\rSXIOPO",
        // Non-ASCII.
        "1234:가",
        "１２３４:ＡＢ",
        // One bad token among good ones fails the whole code: the core would have dropped it
        // and applied the rest, which is the half-set this check exists to prevent.
        "1234:AB+SXIOPQ+12345678",
        "SXIOPO+1234:AB+not a code",
    ] {
        let e = nes_error(code);
        assert_eq!(e.core(), CoreId::Fceumm, "{code:?}");
        assert_eq!(e.platform(), Platform::Nes, "{code:?}");
    }
}

#[test]
fn fceumm_keeps_its_buffer_limit_in_bytes() {
    // 128 raw tokens and 127 separators: 1023 bytes, the most the core's 1024-byte copy holds
    // once its terminator is counted.
    let tokens: Vec<String> = (0..128).map(|i| format!("{i:04X}:{i:02X}")).collect();
    let code = tokens.join("+");
    assert_eq!(code.len(), 1023, "the fixture is not the boundary");
    assert!(nes(&code), "a 1023-byte code is inside the core's buffer");

    // One byte more is cut short by the core, so it is refused here rather than half-applied.
    let over = format!("{code}+");
    assert_eq!(over.len(), 1024);
    let e = nes_error(&over);
    assert!(e.reason().contains("1024"), "{}", e.reason());

    // Bytes, not characters: a hundred Hangul syllables are three hundred bytes, and the
    // adapter upper-cases every byte it is given.
    let wide = "가".repeat(100);
    assert_eq!(wide.chars().count(), 100);
    assert_eq!(wide.len(), 300);
    let e = nes_error(&wide);
    assert!(
        e.reason().contains("ASCII"),
        "the refusal does not say why: {}",
        e.reason()
    );
}

// ---------------------------------------------------------------- common rules

#[test]
fn no_mgba_console_takes_a_code_that_is_not_ascii() {
    for platform in [Platform::Gb, Platform::Gbc, Platform::Gba] {
        for code in ["가나다라", "１２３４５６７８", "1234:가", "12345678+가나"] {
            let e = refused(CoreId::Mgba, platform, code);
            assert_eq!(e.platform(), platform, "{platform:?}: {code:?}");
            assert!(
                e.reason().contains("ASCII"),
                "the refusal does not say why: {}",
                e.reason()
            );
        }
    }
}

#[test]
fn only_the_cores_with_no_grammar_here_stay_unchecked() {
    // gpSP, Gambatte and Genesis Plus GX answer the same thing whatever the code looks like: a
    // code SNES9x would refuse, a code mGBA would refuse, and a code either takes.
    for (core, platform) in [
        (CoreId::Gpsp, Platform::Gba),
        (CoreId::Gambatte, Platform::Gb),
        (CoreId::Gambatte, Platform::Gbc),
        (CoreId::GenesisPlusGx, Platform::Md),
        (CoreId::GenesisPlusGx, Platform::Sms),
    ] {
        for code in [
            "7E007C9A",
            "3202B634+0080",
            "151-91A-7FC",
            "not a code",
            "ZzZz",
            "1234",
        ] {
            assert_eq!(
                validate_cheat(core, platform, code),
                Ok(V::Unchecked),
                "{core:?} claims to have checked {code:?}"
            );
        }
    }
}

#[test]
fn the_common_refusals_come_before_the_grammar() {
    for (core, platform, _) in SUPPORTED {
        // An empty code is nothing to apply.
        let e = refused(core, platform, "");
        assert_eq!(e.core(), core);

        // A NUL is not a C string at all, whatever a core's own syntax allows.
        let e = refused(core, platform, "1234\u{0}5678");
        assert_eq!(e.core(), core);
        assert!(e.reason().contains("NUL"), "{e}");
    }
}

#[test]
fn the_error_names_the_core_and_the_console_and_does_not_carry_the_code() {
    let code = "12345678G+0000";
    let e = refused(CoreId::Mgba, Platform::Gba, code);
    assert_eq!(e.core(), CoreId::Mgba);
    assert_eq!(e.platform(), Platform::Gba);
    assert!(!e.reason().is_empty());
    // A reason is a sentence about a code, not a copy of it: a log line and an error message
    // are not places for whatever a card happened to hold.
    assert!(!e.reason().contains(code), "{e}");
    assert!(!e.to_string().contains(code), "{e}");

    // And it behaves like an error: `Display` names the core a reader can look up, `Debug` is
    // derived, and it can travel as a boxed `std::error::Error`.
    assert!(e.to_string().starts_with("mgba_libretro"), "{e}");
    let _ = format!("{e:?}");
    let boxed: Box<dyn std::error::Error> = Box::new(e);
    assert!(boxed.to_string().contains("mgba_libretro"), "{boxed}");

    // The same for the other core with a grammar here: the console is named too, and the code
    // is not in the message.
    let code = "1234:AB+SXIOPQ";
    let e = nes_error(code);
    assert!(e.to_string().starts_with("fceumm_libretro"), "{e}");
    assert!(!e.reason().contains(code), "{e}");
    assert!(!e.to_string().contains(code), "{e}");
}

#[test]
fn checking_a_code_does_not_change_it() {
    for (core, platform, code) in [
        (CoreId::Snes9x, Platform::Snes, "DF47-0915+12345678"),
        (CoreId::Snes9x, Platform::Snes, "not a code"),
        (CoreId::Mgba, Platform::Gba, "3202B634+0080"),
        (CoreId::Mgba, Platform::Gb, "151-91A-7FC"),
        (CoreId::Mgba, Platform::Gbc, "가"),
        (CoreId::Mgba, Platform::Gba, ""),
        (CoreId::Fceumm, Platform::Nes, "1234\u{0}5678"),
        (CoreId::Fceumm, Platform::Nes, "1234:AB+SXIOPO"),
    ] {
        let original = code.to_string();
        let _ = validate_cheat(core, platform, code);
        assert_eq!(code, original, "{code:?} came back changed");
    }
}

// ---------------------------------------------------------------- delivery policy

#[test]
fn only_mgba_and_fceumm_want_disabled_entries_left_out() {
    // `retro_cheat_set` takes an `enabled` flag and a core is free to read it. The pinned mGBA
    // and FCEUmm adapters do not — both add every code they are handed to one enabled set —
    // so for those two an entry that is off has to be withheld rather than flagged.
    for core in [CoreId::Mgba, CoreId::Fceumm] {
        assert_eq!(
            cheat_delivery(core),
            CheatDelivery::EnabledEntriesOnly,
            "{core:?}"
        );
    }
    // Gambatte's pinned adapter keeps the enabled flag per index and re-applies only the
    // entries that are on, so it is handed the flag like the rest.
    for core in [
        CoreId::Gambatte,
        CoreId::Gpsp,
        CoreId::Snes9x,
        CoreId::GenesisPlusGx,
    ] {
        assert_eq!(
            cheat_delivery(core),
            CheatDelivery::PassAllEntries,
            "{core:?}"
        );
    }
}

#[test]
fn the_delivery_policy_is_not_the_syntax_verdict() {
    // One asks which entries to send, the other whether a code is a form the core parses: a
    // core can be unchecked and still know what an enabled flag means, and a core that
    // validates every code can still need its off entries withheld.
    assert_eq!(cheat_delivery(CoreId::Gpsp), CheatDelivery::PassAllEntries);
    assert_eq!(
        validate_cheat(CoreId::Gpsp, Platform::Gba, "not a code"),
        Ok(V::Unchecked)
    );

    // And the other way round: mGBA's syntax is read here, and its delivery is the odd one.
    assert_eq!(
        cheat_delivery(CoreId::Mgba),
        CheatDelivery::EnabledEntriesOnly
    );
    assert!(validate_cheat(CoreId::Mgba, Platform::Gba, "not a code").is_err());
    assert_eq!(
        validate_cheat(CoreId::Snes9x, Platform::Snes, "7E007C9A"),
        Ok(V::Validated)
    );
}
