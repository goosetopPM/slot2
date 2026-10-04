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

// --- the styled playback the platform profiles ask for --------------------------------

/// The speeds the production table spans, wide end to wide end: the slowest cart in the
/// slowest slot and the fastest connector, plus the neutral one. The audio crate never sees
/// the table itself, so what it is held to here is the range the table lives in.
const SPEEDS: [f32; 5] = [0.80, 0.82, 1.00, 1.19, 1.20];

fn peak(samples: &[i16]) -> u16 {
    samples.iter().map(|s| s.unsigned_abs()).max().unwrap_or(0)
}

#[test]
fn the_hard_numbers_of_the_recording_did_not_move() {
    // The lead is where the contacts are, and the picture is kept with the sound by starting
    // the clip that much early. If these move, every shelf's contacts move with them.
    assert_eq!(Sfx::Insert.lead(), 0.097);
    assert_eq!(Sfx::Eject.lead(), 0.021);
    assert!((Sfx::Insert.seconds() - 0.240).abs() < 1e-6);
    assert!((Sfx::Eject.seconds() - 0.315).abs() < 1e-6);
    assert_eq!(Sfx::Insert.seconds_at_speed(1.0), Sfx::Insert.seconds());
    assert_eq!(Sfx::Insert.lead_at_speed(1.0), Sfx::Insert.lead());
    assert_eq!(Sfx::Insert.tail_at_speed(1.0), Sfx::Insert.tail());
}

#[test]
fn the_unstyled_render_is_a_styled_one_that_changes_nothing() {
    // The recording is what every caller from before this task means by `render`, and the
    // styled path is the same code with a speed of one and a gain of one.
    for c in CLIPS {
        for rate in [22_050u32, 44_100, 48_000, 96_000, 192_000] {
            assert_eq!(
                c.render(rate),
                c.render_styled(rate, 1.0, 1.0),
                "{c:?} at {rate}"
            );
        }
    }
}

#[test]
fn a_styled_clip_lasts_what_its_speed_says() {
    // The samples and the cue are derived from the same effective speed, so a clip rendered at
    // a rate takes the time `seconds_at_speed` claims it does. This is what keeps the contacts
    // on the frame the cart seats on a shelf that plays the clip at any speed at all.
    for c in CLIPS {
        for rate in [22_050u32, 48_000, 96_000] {
            for speed in SPEEDS {
                let frames = c.render_styled(rate, speed, 1.0).len() / 2;
                let played = frames as f32 / rate as f32;
                assert!(
                    (played - c.seconds_at_speed(speed)).abs() < 0.002,
                    "{c:?} at {rate} and {speed}x plays {played}s, not {}s",
                    c.seconds_at_speed(speed)
                );
            }
        }
        // And the cue's own arithmetic stays a partition of the clip, which is what the seat
        // is measured against.
        for speed in SPEEDS {
            let lead = c.lead_at_speed(speed);
            let tail = c.tail_at_speed(speed);
            let whole = c.seconds_at_speed(speed);
            assert!(
                (lead + tail - whole).abs() < 1e-5,
                "{c:?} at {speed}x: {lead} + {tail} is not {whole}"
            );
            assert!(lead > 0.0 && tail > 0.0, "{c:?} at {speed}x");
        }
    }
}

#[test]
fn faster_is_shorter_and_slower_is_longer() {
    for c in CLIPS {
        let slow = c.render_styled(48_000, 0.82, 1.0).len();
        let plain = c.render_styled(48_000, 1.00, 1.0).len();
        let fast = c.render_styled(48_000, 1.19, 1.0).len();
        assert!(
            slow > plain,
            "{c:?}: 0.82x is not longer than the recording"
        );
        assert!(
            fast < plain,
            "{c:?}: 1.19x is not shorter than the recording"
        );
    }
}

#[test]
fn a_styled_clip_is_no_louder_than_the_recording_and_the_same_in_both_ears() {
    // A gain above one would clip the recording, and the table never asks for one. A gain below
    // one has to actually change the amplitude, or the profiles would only differ in speed.
    for c in CLIPS {
        for speed in SPEEDS {
            let plain = c.render_styled(48_000, speed, 1.0);
            let quiet = c.render_styled(48_000, speed, 0.78);
            assert!(
                peak(&quiet) <= peak(&plain),
                "{c:?} at {speed}x: 0.78 gain peaks at {} against {}",
                peak(&quiet),
                peak(&plain)
            );
            assert!(peak(&quiet) > 1_000, "{c:?} at {speed}x came out silent");
            assert_eq!(quiet.len() % 2, 0, "{c:?} at {speed}x: odd sample count");
            for pair in quiet.as_chunks::<2>().0 {
                assert_eq!(
                    pair[0], pair[1],
                    "{c:?} at {speed}x: a frame differs between the ears"
                );
            }
        }
    }
}

#[test]
fn a_speed_or_gain_off_the_table_is_refused() {
    // A value out of range is a table that has gone wrong. The answer is silence and a cue of
    // zero rather than a clamped value nobody can trace back to its table entry.
    for c in CLIPS {
        for speed in [
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
            0.0,
            0.79,
            1.21,
            -1.0,
        ] {
            assert!(
                c.render_styled(48_000, speed, 1.0).is_empty(),
                "{c:?} played at {speed}x"
            );
            assert_eq!(c.lead_at_speed(speed), 0.0, "{c:?} at {speed}x");
            assert_eq!(c.seconds_at_speed(speed), 0.0, "{c:?} at {speed}x");
            assert_eq!(c.tail_at_speed(speed), 0.0, "{c:?} at {speed}x");
        }
        for gain in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -0.01, 1.01, 2.0] {
            assert!(
                c.render_styled(48_000, 1.0, gain).is_empty(),
                "{c:?} at gain {gain}"
            );
        }
    }
}

#[test]
fn a_rate_no_sink_could_report_is_empty_not_a_hang() {
    // Below one and above the widest rate a sink could plausibly open at, there is no clip to
    // render: the old contract here was "does not panic", which quietly meant an hour of
    // interpolation at `u32::MAX`. Silence is the honest answer, and it costs nothing.
    for c in CLIPS {
        for rate in [0u32, 400_000, u32::MAX] {
            assert!(c.render(rate).is_empty(), "{c:?} rendered at {rate} Hz");
            assert!(
                c.render_styled(rate, 1.0, 1.0).is_empty(),
                "{c:?} at {rate}"
            );
        }
        // One frame per clip at 1 Hz: a finite, well-formed return rather than a special case.
        let one = c.render(1);
        assert!(!one.is_empty() && one.len() % 2 == 0, "{c:?} at 1 Hz");
    }
}
