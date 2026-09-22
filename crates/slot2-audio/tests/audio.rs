//! The contract for slot2-audio. Task 08 makes these pass without editing this file.

use slot2_audio::{Consumer, Producer, Resampler, Ring, Volume, CHANNELS, DEVICE_RATE};

fn split(frames: usize) -> (Producer, Consumer) {
    Ring::new(frames).split()
}

// ---------- ring ----------

#[test]
fn ring_round_trips_and_reports_space() {
    let ring = Ring::new(64);
    let cap = ring.capacity();
    assert!(cap >= 64 * CHANNELS, "capacity {cap}");
    assert!(
        cap.is_power_of_two(),
        "capacity {cap} should be a power of two"
    );
    let (mut p, mut c) = ring.split();
    assert_eq!(c.available(), 0);
    assert_eq!(c.frames(), 0);
    assert!(p.space() >= 64 * CHANNELS - CHANNELS);

    assert_eq!(p.write(&[1, 2, 3, 4]), 4);
    assert_eq!(c.available(), 4);
    assert_eq!(c.frames(), 2);
    let mut out = [0i16; 4];
    assert_eq!(c.read(&mut out), 4);
    assert_eq!(out, [1, 2, 3, 4]);
    assert_eq!(c.available(), 0);
}

#[test]
fn ring_wraps_around_many_times() {
    let (mut p, mut c) = split(8);
    let mut next = 0i16;
    let mut expect = 0i16;
    for _ in 0..200 {
        let batch: Vec<i16> = (0..6)
            .map(|_| {
                next = next.wrapping_add(1);
                next
            })
            .collect();
        assert_eq!(p.write(&batch), 6, "space should be free after each read");
        let mut out = [0i16; 6];
        assert_eq!(c.read(&mut out), 6);
        for v in out {
            expect = expect.wrapping_add(1);
            assert_eq!(v, expect);
        }
    }
}

#[test]
fn a_full_ring_drops_rather_than_blocking() {
    let (mut p, mut c) = split(4); // 8 samples
    let big = vec![7i16; 64];
    let taken = p.write(&big);
    assert!(taken > 0 && taken < 64, "took {taken} of 64");
    assert_eq!(p.write(&big), 0, "still full");
    let mut out = vec![0i16; 64];
    assert_eq!(c.read(&mut out), taken);
    assert!(out[..taken].iter().all(|&v| v == 7));
}

#[test]
fn an_underrun_is_silence_not_a_repeat() {
    let (mut p, mut c) = split(16);
    p.write(&[9, 9, 9, 9]);
    let mut out = [1i16; 10];
    assert_eq!(c.read_or_silence(&mut out), 4);
    assert_eq!(out, [9, 9, 9, 9, 0, 0, 0, 0, 0, 0]);
    let mut out2 = [1i16; 4];
    assert_eq!(c.read_or_silence(&mut out2), 0);
    assert_eq!(out2, [0, 0, 0, 0], "an empty ring plays silence");
}

#[test]
fn producer_and_consumer_work_from_two_threads() {
    let (mut p, mut c) = split(256);
    let total = 100_000usize;
    let writer = std::thread::spawn(move || {
        let mut written = 0usize;
        let mut v = 0i16;
        while written < total {
            let batch: Vec<i16> = (0..32)
                .map(|_| {
                    v = v.wrapping_add(1);
                    v
                })
                .collect();
            let mut off = 0;
            while off < batch.len() {
                let n = p.write(&batch[off..]);
                if n == 0 {
                    std::thread::yield_now();
                } else {
                    off += n;
                    written += n;
                }
            }
        }
    });
    let mut read = 0usize;
    let mut expect = 0i16;
    let mut buf = [0i16; 48];
    while read < total {
        let n = c.read(&mut buf);
        for &v in &buf[..n] {
            expect = expect.wrapping_add(1);
            assert_eq!(v, expect, "sample {read} out of order");
            read += 1;
        }
        if n == 0 {
            std::thread::yield_now();
        }
    }
    writer.join().unwrap();
}

// ---------- resampler ----------

#[test]
fn equal_rates_pass_through_untouched() {
    let mut r = Resampler::new(48_000, 48_000);
    assert_eq!(r.rates(), (48_000, 48_000));
    let input: Vec<i16> = (0..64).collect();
    let mut out = Vec::new();
    let frames = r.process(&input, &mut out);
    assert_eq!(frames, 32);
    assert_eq!(out, input);
}

#[test]
fn upsampling_lengthens_and_downsampling_shortens() {
    // mGBA's 32768 Hz to the device's 48000: about 1.465x more frames.
    let mut up = Resampler::new(32_768, DEVICE_RATE);
    let input = vec![0i16; 1000 * CHANNELS];
    let mut out = Vec::new();
    let frames = up.process(&input, &mut out);
    assert!((1400..=1500).contains(&frames), "{frames}");
    assert_eq!(out.len(), frames * CHANNELS);
    assert!((frames as i64 - up.expected_output_frames(1000) as i64).abs() <= 2);

    let mut down = Resampler::new(48_000, 32_768);
    let mut out2 = Vec::new();
    let frames2 = down.process(&input, &mut out2);
    assert!((650..=700).contains(&frames2), "{frames2}");
}

#[test]
fn a_ramp_stays_a_ramp_and_channels_do_not_swap() {
    let mut r = Resampler::new(1000, 2000);
    // Left channel counts up, right stays at -1000.
    let input: Vec<i16> = (0..20).flat_map(|i| [i * 100, -1000]).collect();
    let mut out = Vec::new();
    let frames = r.process(&input, &mut out);
    assert!(frames >= 30, "{frames}");
    for f in 0..frames {
        assert_eq!(out[f * 2 + 1], -1000, "right channel at frame {f}");
    }
    // Left rises monotonically (interpolation, never a jump backwards).
    for f in 1..frames {
        assert!(
            out[f * 2] >= out[(f - 1) * 2],
            "frame {f}: {:?}",
            &out[..frames * 2]
        );
    }
}

#[test]
fn successive_calls_do_not_click_or_drift() {
    let mut r = Resampler::new(32_768, DEVICE_RATE);
    let mut total = 0usize;
    let mut out = Vec::new();
    for _ in 0..100 {
        let chunk = vec![0i16; 546 * CHANNELS]; // one frame's worth at 32768/60
        total += r.process(&chunk, &mut out);
    }
    let ideal = (546.0 * 100.0 * DEVICE_RATE as f64 / 32_768.0) as usize;
    let drift = (total as i64 - ideal as i64).abs();
    assert!(
        drift <= 3,
        "drift {drift} frames over 100 calls (total {total}, ideal {ideal})"
    );
}

#[test]
fn odd_input_and_resets() {
    let mut r = Resampler::new(1000, 1000);
    let mut out = Vec::new();
    r.process(&[1, 2, 3], &mut out); // odd tail dropped
    assert_eq!(out, vec![1, 2]);
    r.set_rates(32_768, 48_000);
    assert_eq!(r.rates(), (32_768, 48_000));
    r.reset();
    let mut out2 = Vec::new();
    let n = r.process(&vec![0i16; 200], &mut out2);
    assert!(n > 100);
}

// ---------- volume ----------

#[test]
fn volume_is_perceptual_and_mute_remembers() {
    let mut v = Volume::new(100);
    assert_eq!(v.level(), 100);
    assert!((v.gain() - 1.0).abs() < 1e-6);
    v.set_level(50);
    let g = v.gain();
    assert!(
        (g - 0.25).abs() < 1e-3,
        "50% should be a quarter of the power: {g}"
    );
    v.set_muted(true);
    assert!(v.is_muted());
    assert_eq!(v.gain(), 0.0);
    assert_eq!(v.level(), 50, "level is remembered while muted");
    v.toggle_mute();
    assert!(!v.is_muted());
    assert!((v.gain() - 0.25).abs() < 1e-3);
}

#[test]
fn volume_steps_clamp_and_unmute() {
    let mut v = Volume::new(98);
    v.step_up();
    assert_eq!(v.level(), 100);
    v.step_up();
    assert_eq!(v.level(), 100);
    let mut v = Volume::new(3);
    v.step_down();
    assert_eq!(v.level(), 0);
    assert_eq!(v.gain(), 0.0);
    let mut v = Volume::new(40);
    v.set_muted(true);
    v.step_up();
    assert!(!v.is_muted(), "turning it up unmutes");
    assert_eq!(v.level(), 45);
    assert_eq!(Volume::default().level(), 70);
}

#[test]
fn volume_scales_samples_and_clamps() {
    let v = Volume::new(100);
    let mut s = [i16::MAX, i16::MIN, 1000, -1000];
    v.apply(&mut s);
    assert_eq!(s, [i16::MAX, i16::MIN, 1000, -1000]);
    let half = Volume::new(50);
    let mut s = [1000i16, -1000];
    half.apply(&mut s);
    assert!((s[0] - 250).abs() <= 2, "{s:?}");
    assert!((s[1] + 250).abs() <= 2, "{s:?}");
    let mut v = Volume::new(80);
    v.set_muted(true);
    let mut s = [1234i16; 8];
    v.apply(&mut s);
    assert_eq!(s, [0i16; 8]);
}

// ---------- sinks (opt in: they open a real device) ----------

#[cfg(feature = "host")]
#[test]
fn host_sink_opens_and_plays_when_asked() {
    if std::env::var("SLOT2_AUDIO_TEST")
        .map(|v| v != "1")
        .unwrap_or(true)
    {
        eprintln!("SLOT2_AUDIO_TEST not set; skipping the real audio device test");
        return;
    }
    use slot2_audio::{HostSink, Sink};
    let (mut p, c) = split(4096);
    let mut sink = HostSink::open(DEVICE_RATE, 1024, c).expect("an output device");
    let rate = sink.rate();
    assert!(rate >= 8_000, "{rate}");
    // A quarter second of 440 Hz, quiet.
    let n = rate as usize / 4;
    let mut buf = Vec::with_capacity(n * CHANNELS);
    for i in 0..n {
        let t = i as f32 / rate as f32;
        let s = ((t * 440.0 * std::f32::consts::TAU).sin() * 4000.0) as i16;
        buf.push(s);
        buf.push(s);
    }
    let mut off = 0;
    while off < buf.len() {
        let w = p.write(&buf[off..]);
        off += w;
        if w == 0 {
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }
    std::thread::sleep(std::time::Duration::from_millis(400));
    sink.pause();
    sink.resume();
}
