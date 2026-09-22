//! Where a panel-sized frame lands on a drawable that is not the panel.

/// A rectangle in drawable pixels, origin top-left.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
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
}
