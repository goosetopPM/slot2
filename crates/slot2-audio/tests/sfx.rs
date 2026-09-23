//! The contract for the slot's own noises. Task 19 makes these pass without editing this
//! file.

use slot2_audio::sfx::{Sfx, ASSET_HZ};

const CLIPS: [Sfx; 2] = [Sfx::Insert, Sfx::Eject];

#[test]
fn the_clips_are_the_length_they_are_on_disk() {
    // 23040 and 30240 bytes of mono 16-bit at 48 kHz. If `seconds` were guessed at rather
    // than measured, the lead would point at the wrong part of the sound.
    assert!(
        (Sfx::Insert.seconds() - 0.240).abs() < 1e-3,
        "insert is {}s",
        Sfx::Insert.seconds()
    );
    assert!(
        (Sfx::Eject.seconds() - 0.315).abs() < 1e-3,
        "eject is {}s",
        Sfx::Eject.seconds()
    );
}

#[test]
fn the_lead_is_inside_the_clip_and_the_tail_is_the_rest() {
    for c in CLIPS {
        assert!(
            c.lead() > 0.0 && c.lead() < c.seconds(),
            "{c:?}: lead {} is not inside a clip of {}",
            c.lead(),
            c.seconds()
        );
        assert!(
            (c.lead() + c.tail() - c.seconds()).abs() < 1e-4,
            "{c:?}: {} + {} is not {}",
            c.lead(),
            c.tail(),
            c.seconds()
        );
    }
}

#[test]
fn a_clip_is_stereo_and_the_same_in_both_ears() {
    // Stereo because the sink is, not because the sound is. A slot is in the middle of the
    // machine, and a noise that came out of one side would be a noise from somewhere else.
    for c in CLIPS {
        let out = c.render(48_000);
        assert_eq!(out.len() % 2, 0, "{c:?}: an odd number of samples");
        for (i, pair) in out.as_chunks::<2>().0.iter().enumerate() {
            assert_eq!(
                pair[0], pair[1],
                "{c:?}: frame {i} differs between channels"
            );
        }
    }
}

#[test]
fn a_clip_at_the_asset_rate_is_the_asset() {
    // Nothing is resampled when nothing needs to be. Every device here runs at 48 kHz, so
    // this is the path that actually runs.
    for c in CLIPS {
        let out = c.render(ASSET_HZ as u32);
        let want = (c.seconds() * ASSET_HZ).round() as usize;
        let got = out.len() / 2;
        assert!(
            got.abs_diff(want) <= 1,
            "{c:?}: {got} frames at the asset rate, not {want}"
        );
    }
}

#[test]
fn a_clip_at_another_rate_lasts_the_same_time() {
    // The sink opens at whatever it opens at. What must not change is how long the sound
    // takes, because the picture it goes with is on a clock of its own.
    for c in CLIPS {
        for rate in [22_050u32, 44_100, 48_000, 96_000] {
            let frames = c.render(rate).len() / 2;
            let secs = frames as f32 / rate as f32;
            assert!(
                (secs - c.seconds()).abs() < 0.01,
                "{c:?} at {rate}: {secs}s against {}s",
                c.seconds()
            );
        }
    }
}

#[test]
fn a_clip_is_not_silence() {
    // An asset that failed to embed, or a resampler that divided by nothing, is a frontend
    // that looks right and makes no noise.
    for c in CLIPS {
        let out = c.render(48_000);
        let peak = out.iter().map(|s| s.unsigned_abs()).max().unwrap_or(0);
        assert!(peak > 1_000, "{c:?}: peaks at {peak}");
    }
}

#[test]
fn a_ridiculous_rate_is_not_a_panic() {
    // The sink reports what it opened at, and nothing here gets to assume that is sensible.
    for c in CLIPS {
        for rate in [0u32, 1, u32::MAX] {
            let out = c.render(rate);
            assert!(out.len() % 2 == 0, "{c:?} at {rate}: odd sample count");
        }
    }
}
