//! Minimal, valid ROMs for every platform we host, built in memory.
//!
//! The GBA suite runs on jsmolka's `arm.gba` (MIT), but there is no equally clean test ROM
//! lying around for the other five shelves, and vendoring commercial dumps is out of the
//! question. So each of these assembles the smallest file its console's loader will accept:
//! a correct header and a few bytes of machine code that disable interrupts and branch to
//! themselves forever.
//!
//! That is enough for what these tests ask — does the core load this, run frames, hand over
//! video and audio at the rate it claims, and round-trip a save state — and it costs no
//! licence and no binary in the repository. What it deliberately does not test is whether a
//! core emulates anything correctly; a real game on real hardware answers that.
//!
//! There is deliberately no Game Boy builder here: mGBA's `GBIsROM` compares the 48 bytes
//! at $0104 against the Nintendo logo and refuses anything else, so the only file it will
//! accept as a cartridge carries Nintendo's artwork. `cores.rs` explains what covers those
//! two shelves instead.
//!
//! Each function returns the file bytes. The formats are documented where they are built,
//! because "why is there a 0x60 0xFE at offset 0x200" is not a question anyone should have
//! to leave the file to answer.

/// 68000 `BRA.S *` — branch to self, the whole program.
const M68K_HALT: [u8; 2] = [0x60, 0xFE];

/// An iNES file: 16-byte header, one 16 KiB PRG bank, one 8 KiB CHR bank, mapper 0.
///
/// With a single PRG bank the mapper mirrors it into both $8000 and $C000, so the CPU
/// vectors at $FFFA-$FFFF are the last six bytes of PRG. RESET points at the start of the
/// bank, where a 6502 `SEI; CLD; JMP $C000` parks the CPU.
pub fn nes() -> Vec<u8> {
    let mut rom = Vec::with_capacity(16 + 16384 + 8192);
    rom.extend_from_slice(b"NES\x1A");
    rom.push(1); // 16 KiB of PRG
    rom.push(1); // 8 KiB of CHR
    rom.extend_from_slice(&[0; 10]); // flags 6..15: mapper 0, no battery, no trainer

    let mut prg = vec![0u8; 16384];
    prg[0..5].copy_from_slice(&[0x78, 0xD8, 0x4C, 0x00, 0xC0]); // SEI; CLD; JMP $C000
    prg[0x3FFA] = 0x00; // NMI  → $C000
    prg[0x3FFB] = 0xC0;
    prg[0x3FFC] = 0x00; // RESET → $C000
    prg[0x3FFD] = 0xC0;
    prg[0x3FFE] = 0x00; // IRQ  → $C000
    prg[0x3FFF] = 0xC0;
    rom.extend_from_slice(&prg);
    rom.extend_from_slice(&[0u8; 8192]); // blank pattern tables
    rom
}

/// A 32 KiB LoROM SNES image.
///
/// LoROM maps each 32 KiB bank to $8000-$FFFF, so file offset 0 is $008000 and the header
/// sits at $7FC0. The 65816 starts in emulation mode and takes its RESET vector from
/// $7FFC. Snes9x picks the layout by scoring the header, so the checksum pair has to be
/// right or a 32 KiB file can be mistaken for HiROM.
pub fn snes() -> Vec<u8> {
    let mut rom = vec![0u8; 0x8000];
    rom[0..3].copy_from_slice(&[0x78, 0x80, 0xFE]); // SEI; BRA *

    rom[0x7FC0..0x7FD5].copy_from_slice(b"SLOT2 TEST CART      "); // title, 21 bytes
    rom[0x7FD5] = 0x20; // LoROM, SlowROM
    rom[0x7FD6] = 0x00; // ROM only, no coprocessor
    rom[0x7FD7] = 0x05; // 2^5 KiB = 32 KiB
    rom[0x7FD8] = 0x00; // no save RAM
    rom[0x7FD9] = 0x01; // NTSC
    rom[0x7FDA] = 0x33; // "see the extended header" — what modern tools write
    rom[0x7FDB] = 0x00; // version

    // Every vector points at the halt, so an unexpected interrupt cannot wander off.
    for v in (0x7FE4..0x7FF0)
        .step_by(2)
        .chain((0x7FF4..0x8000).step_by(2))
    {
        rom[v] = 0x00;
        rom[v + 1] = 0x80;
    }

    // The checksum covers the whole file with the checksum pair itself read as 0x0000 and
    // its complement as 0xFFFF; writing them afterwards keeps the sum true.
    rom[0x7FDC..0x7FE0].copy_from_slice(&[0xFF, 0xFF, 0x00, 0x00]);
    let sum = rom.iter().fold(0u16, |a, b| a.wrapping_add(*b as u16));
    rom[0x7FDC..0x7FDE].copy_from_slice(&(!sum).to_le_bytes());
    rom[0x7FDE..0x7FE0].copy_from_slice(&sum.to_le_bytes());
    rom
}

/// A 128 KiB Mega Drive image.
///
/// The 68000 takes its initial stack pointer from $000000 and its initial PC from $000004,
/// and Genesis Plus GX identifies the cartridge by the console name at $000100. The rest of
/// the header is filled because a loader that reads a field should find something sane
/// there, not leftover zeroes.
pub fn megadrive() -> Vec<u8> {
    let mut rom = vec![0u8; 0x20000];

    // Vector table: stack at the top of work RAM, and every vector at the halt.
    rom[0x00..0x04].copy_from_slice(&0x00FF_FFFCu32.to_be_bytes());
    for v in (0x04..0x100).step_by(4) {
        rom[v..v + 4].copy_from_slice(&0x0000_0200u32.to_be_bytes());
    }

    let mut put = |at: usize, text: &str, len: usize| {
        let b = text.as_bytes();
        rom[at..at + len].fill(b' ');
        rom[at..at + b.len().min(len)].copy_from_slice(&b[..b.len().min(len)]);
    };
    put(0x100, "SEGA MEGA DRIVE ", 16);
    put(0x110, "(C)SLOT2 2026.SEP", 16);
    put(0x120, "SLOT2 TEST CART", 48); // domestic name
    put(0x150, "SLOT2 TEST CART", 48); // overseas name
    put(0x180, "GM 00000000-00", 14); // serial
    put(0x190, "J", 16); // a 3-button pad is enough
    put(0x1BC, "", 52); // notes and region, blank

    rom[0x18E..0x190].copy_from_slice(&[0, 0]); // checksum: gpgx does not enforce it
    rom[0x1A0..0x1A4].copy_from_slice(&0u32.to_be_bytes()); // ROM start
    rom[0x1A4..0x1A8].copy_from_slice(&(0x20000u32 - 1).to_be_bytes()); // ROM end
    rom[0x1A8..0x1AC].copy_from_slice(&0x00FF_0000u32.to_be_bytes()); // RAM start
    rom[0x1AC..0x1B0].copy_from_slice(&0x00FF_FFFFu32.to_be_bytes()); // RAM end

    rom[0x200..0x202].copy_from_slice(&M68K_HALT);
    rom
}

/// A 32 KiB Master System image.
///
/// The Z80 starts at $0000 with no header of any kind required; the "TMR SEGA" block at
/// $7FF0 is what the console's own BIOS checks before it will run a cartridge, so it is
/// written even though an emulator will load the file without it.
pub fn sms() -> Vec<u8> {
    let mut rom = vec![0u8; 0x8000];
    rom[0..3].copy_from_slice(&[0xF3, 0x18, 0xFE]); // DI; JR *

    rom[0x7FF0..0x7FF8].copy_from_slice(b"TMR SEGA");
    rom[0x7FF8..0x7FFA].copy_from_slice(&[0x00, 0x00]); // reserved
    rom[0x7FFA..0x7FFC].copy_from_slice(&[0x00, 0x00]); // checksum, not enforced
    rom[0x7FFC..0x7FFF].copy_from_slice(&[0x00, 0x00, 0x00]); // product code, version
    rom[0x7FFF] = 0x4C; // export region, 32 KiB
    rom
}

/// Write `bytes` into a temporary file named `<stem>.<ext>` and hand back the path. Cores
/// pick their loader by extension, so the name matters as much as the content.
pub fn write_rom(stem: &str, ext: &str, bytes: &[u8]) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("slot2-testrom-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{stem}.{ext}"));
    std::fs::write(&path, bytes).unwrap();
    path
}
