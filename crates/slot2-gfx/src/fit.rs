//! Where a frame lands on a screen that is not its size.
//!
//! Two different jobs share this file. `place` decides where a *game* frame goes on the
//! panel, under a policy the player can change. `integer_fit_rect` and `fit_rect` decide
//! where the whole *panel* goes on a drawable that is not the panel — a host window, or
//! HDMI out — and have no policy to choose.

/// A rectangle in drawable pixels, origin top-left.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// How a game frame is laid onto the panel. Per platform by default, per game by override.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScalePolicy {
    /// The largest whole multiple that fits, floored at 1, centred, aspect ignored.
    ///
    /// The default everywhere, and on these panels it is rarely a compromise: a GBA frame
    /// is exactly 3× on 720×480. Every source pixel becomes the same number of screen
    /// pixels, which is what keeps sprite edges hard and 8×8 text legible. The price is
    /// black bars and a picture narrower than the console's own shape.
    #[default]
    Integer,
    /// The largest size that keeps the console's display shape, centred.
    ///
    /// Corrects consoles whose pixels were never square — a NES pixel is 8:7 — at the cost
    /// of a fractional factor, which means filtering, which means softness.
    AspectFit,
    /// The whole panel, shape be damned.
    Fill,
}

/// Put a `src`-pixel game frame somewhere on a `dest`-pixel panel.
///
/// `aspect` is the shape the frame is *meant* to be shown as, as a ratio: `(4, 3)` for a
/// Mega Drive whatever its current width, `(1, 1)` for a Game Boy Advance. Only `AspectFit`
/// reads it — `Integer` ignoring it is the whole point of `Integer`.
///
/// A zero anywhere gives an empty rect rather than a division by zero: a core that has not
/// produced a frame yet should draw nothing, not crash the frontend.
pub fn place(policy: ScalePolicy, src: (u32, u32), aspect: (u32, u32), dest: (u32, u32)) -> Rect {
    if src.0 == 0 || src.1 == 0 || dest.0 == 0 || dest.1 == 0 {
        return Rect {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        };
    }
    let centred = |w: i32, h: i32| Rect {
        x: (dest.0 as i32 - w) / 2,
        y: (dest.1 as i32 - h) / 2,
        w,
        h,
    };
    match policy {
        ScalePolicy::Integer => integer_fit_rect(src, dest),
        ScalePolicy::Fill => Rect {
            x: 0,
            y: 0,
            w: dest.0 as i32,
            h: dest.1 as i32,
        },
        ScalePolicy::AspectFit => {
            let (an, ad) = if aspect.0 == 0 || aspect.1 == 0 {
                (src.0, src.1)
            } else {
                aspect
            };
            // Widest box of shape an:ad that fits: limited by height when the panel is the
            // wider of the two, by width otherwise.
            let want = an as f32 / ad as f32;
            let have = dest.0 as f32 / dest.1 as f32;
            let (w, h) = if have > want {
                ((dest.1 as f32 * want).round() as i32, dest.1 as i32)
            } else {
                (dest.0 as i32, (dest.0 as f32 / want).round() as i32)
            };
            centred(w, h)
        }
    }
}

/// The texture sub-rectangle left after cropping `left`/`top`/`right`/`bottom` source
/// pixels, in the form `image_uv` wants. Overscan is the reason this exists: a NES really
/// did output 240 lines, of which a television showed about 224, so the top and bottom rows
/// hold whatever the game did not bother to draw.
pub fn sub_uv(src: (u32, u32), left: u32, top: u32, right: u32, bottom: u32) -> [f32; 4] {
    if src.0 == 0 || src.1 == 0 {
        return [0.0, 0.0, 1.0, 1.0];
    }
    // A crop that would leave nothing is ignored rather than inverted.
    let (l, r) = if left + right >= src.0 {
        (0, 0)
    } else {
        (left, right)
    };
    let (t, b) = if top + bottom >= src.1 {
        (0, 0)
    } else {
        (top, bottom)
    };
    [
        l as f32 / src.0 as f32,
        t as f32 / src.1 as f32,
        (src.0 - r) as f32 / src.0 as f32,
        (src.1 - b) as f32 / src.1 as f32,
    ]
}

/// The size left after cropping that much away, never zero.
pub fn cropped_size(src: (u32, u32), left: u32, top: u32, right: u32, bottom: u32) -> (u32, u32) {
    let w = if left + right >= src.0 {
        src.0
    } else {
        src.0 - left - right
    };
    let h = if top + bottom >= src.1 {
        src.1
    } else {
        src.1 - top - bottom
    };
    (w, h)
}

/// The frame scaled by the largest whole factor that fits, floored at 1, centred. Bars are
/// black. Used by the host window: integer scaling keeps pixel art and text crisp, and an
/// undersized window crops rather than blurs.
pub fn integer_fit_rect(panel: (u32, u32), drawable: (u32, u32)) -> Rect {
    let s = (drawable.0 / panel.0).min(drawable.1 / panel.1).max(1) as i32;
    let (w, h) = (panel.0 as i32 * s, panel.1 as i32 * s);
    Rect {
        x: (drawable.0 as i32 - w) / 2,
        y: (drawable.1 as i32 - h) / 2,
        w,
        h,
    }
}

/// The frame scaled by a fractional factor to fit inside, aspect kept, centred. For a
/// drawable whose size is not a multiple of the panel's and must still show everything —
/// HDMI out at 1280x720 showing a 720x480 frame, say. Soft, by design.
pub fn fit_rect(panel: (u32, u32), drawable: (u32, u32)) -> Rect {
    let s = (drawable.0 as f32 / panel.0 as f32).min(drawable.1 as f32 / panel.1 as f32);
    let w = (panel.0 as f32 * s).round() as i32;
    let h = (panel.1 as f32 * s).round() as i32;
    Rect {
        x: (drawable.0 as i32 - w) / 2,
        y: (drawable.1 as i32 - h) / 2,
        w,
        h,
    }
}

/// The part of `src` to show so that it *covers* `dest` without distortion.
///
/// Cover rather than fit, because a background is the ground the screen stands on and a
/// letterboxed one is a picture in a frame. The panel comes in three shapes — 4:3, 3:2 and
/// square — so no single image can match all of them, and something has to give: either the
/// aspect (which stretches faces), the coverage (which leaves bars), or the edges. The edges
/// of a background are the part nobody composed.
///
/// Returns `[u0, v0, u1, v1]`, centred on the image.
pub fn cover_uv(src: (u32, u32), dest: (u32, u32)) -> [f32; 4] {
    if src.0 == 0 || src.1 == 0 || dest.0 == 0 || dest.1 == 0 {
        return [0.0, 0.0, 1.0, 1.0];
    }
    let want = dest.0 as f32 / dest.1 as f32;
    let have = src.0 as f32 / src.1 as f32;
    if have > want {
        let crop_w = src.1 as f32 * want;
        let u_margin = (src.0 as f32 - crop_w) / 2.0 / src.0 as f32;
        [u_margin, 0.0, 1.0 - u_margin, 1.0]
    } else {
        let crop_h = src.0 as f32 / want;
        let v_margin = (src.1 as f32 - crop_h) / 2.0 / src.1 as f32;
        [0.0, v_margin, 1.0, 1.0 - v_margin]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_fit_is_exact_when_the_drawable_is_the_panel() {
        assert_eq!(
            integer_fit_rect((720, 480), (720, 480)),
            Rect {
                x: 0,
                y: 0,
                w: 720,
                h: 480
            }
        );
    }

    #[test]
    fn integer_fit_doubles_and_centres() {
        assert_eq!(
            integer_fit_rect((720, 480), (1600, 1000)),
            Rect {
                x: 80,
                y: 20,
                w: 1440,
                h: 960
            }
        );
    }

    #[test]
    fn integer_fit_crops_when_too_small() {
        assert_eq!(
            integer_fit_rect((720, 480), (600, 400)),
            Rect {
                x: -60,
                y: -40,
                w: 720,
                h: 480
            }
        );
    }

    #[test]
    fn fractional_fit_keeps_aspect() {
        let r = fit_rect((720, 480), (1280, 720));
        assert_eq!(
            r,
            Rect {
                x: 100,
                y: 0,
                w: 1080,
                h: 720
            }
        );
        let r = fit_rect((720, 720), (1280, 720));
        assert_eq!(
            r,
            Rect {
                x: 280,
                y: 0,
                w: 720,
                h: 720
            }
        );
    }

    #[test]
    fn integer_ignores_aspect_and_stays_whole() {
        // A NES frame is 8:7, but Integer is not interested: 2x of 256x240 on a 720x480
        // panel, centred, with the aspect argument making no difference at all.
        let square = place(ScalePolicy::Integer, (256, 240), (1, 1), (720, 480));
        let wide = place(ScalePolicy::Integer, (256, 240), (8, 7), (720, 480));
        assert_eq!(square, wide);
        assert_eq!(
            square,
            Rect {
                x: 104,
                y: 0,
                w: 512,
                h: 480
            }
        );
    }

    #[test]
    fn aspect_fit_corrects_non_square_pixels() {
        // 256x224 at 8:7 wants to be shown as 2048:1568 = 4:3.056..., so on a 720x480 panel
        // it is height-limited and comes out wider than 2x would make it.
        let r = place(
            ScalePolicy::AspectFit,
            (256, 224),
            (256 * 8, 224 * 7),
            (720, 480),
        );
        assert_eq!(r.h, 480);
        assert!(
            r.w > 512,
            "aspect correction should widen past 2x, got {}",
            r.w
        );
        assert!(r.w <= 720, "and still fit, got {}", r.w);
        assert_eq!(r.x, (720 - r.w) / 2);
    }

    #[test]
    fn a_mega_drive_looks_the_same_at_both_widths() {
        // The whole reason Aspect::Display exists. A Mega Drive switches between 256 and 320
        // pixels across mid-game and both filled the same 4:3 television, so the picture on
        // the panel must not jump when it does.
        let narrow = place(ScalePolicy::AspectFit, (256, 224), (4, 3), (720, 480));
        let wide = place(ScalePolicy::AspectFit, (320, 224), (4, 3), (720, 480));
        assert_eq!(narrow, wide);
        assert_eq!(
            narrow,
            Rect {
                x: 40,
                y: 0,
                w: 640,
                h: 480
            }
        );
    }

    #[test]
    fn fill_takes_the_whole_panel() {
        assert_eq!(
            place(ScalePolicy::Fill, (256, 224), (4, 3), (720, 480)),
            Rect {
                x: 0,
                y: 0,
                w: 720,
                h: 480
            }
        );
    }

    #[test]
    fn a_frame_that_does_not_exist_yet_draws_nothing() {
        for policy in [
            ScalePolicy::Integer,
            ScalePolicy::AspectFit,
            ScalePolicy::Fill,
        ] {
            assert_eq!(
                place(policy, (0, 0), (4, 3), (720, 480)),
                Rect {
                    x: 0,
                    y: 0,
                    w: 0,
                    h: 0
                }
            );
            assert_eq!(
                place(policy, (256, 224), (4, 3), (0, 0)),
                Rect {
                    x: 0,
                    y: 0,
                    w: 0,
                    h: 0
                }
            );
        }
    }

    #[test]
    fn overscan_crops_the_rows_a_television_hid() {
        assert_eq!(cropped_size((256, 240), 0, 8, 0, 8), (256, 224));
        let uv = sub_uv((256, 240), 0, 8, 0, 8);
        assert_eq!(uv[0], 0.0);
        assert_eq!(uv[2], 1.0);
        assert!((uv[1] - 8.0 / 240.0).abs() < 1e-6, "{uv:?}");
        assert!((uv[3] - 232.0 / 240.0).abs() < 1e-6, "{uv:?}");
    }

    #[test]
    fn a_crop_that_would_leave_nothing_is_ignored() {
        assert_eq!(cropped_size((256, 240), 200, 0, 200, 0), (256, 240));
        assert_eq!(sub_uv((256, 240), 200, 0, 200, 0), [0.0, 0.0, 1.0, 1.0]);
        assert_eq!(sub_uv((0, 0), 1, 1, 1, 1), [0.0, 0.0, 1.0, 1.0]);
    }
}
