//! The 640x480 safe area and where it sits on each panel (D-09).

use slot2_platform::Geometry;

pub const SAFE_W: u32 = 640;
pub const SAFE_H: u32 = 480;

/// Offset of the safe area's top-left corner on the panel, plus the panel size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SafeArea {
    pub x: u32,
    pub y: u32,
    pub panel_w: u32,
    pub panel_h: u32,
}

impl SafeArea {
    pub fn for_geometry(g: Geometry) -> Self {
        let (panel_w, panel_h) = g.size();
        let (x, y) = g.safe_area_offset();
        SafeArea {
            x,
            y,
            panel_w,
            panel_h,
        }
    }

    /// Panel x of a safe-area x.
    pub fn px(&self, safe_x: f32) -> f32 {
        self.x as f32 + safe_x
    }

    /// Panel y of a safe-area y.
    pub fn py(&self, safe_y: f32) -> f32 {
        self.y as f32 + safe_y
    }

    /// Panel x that centres something `w` wide in the safe area.
    pub fn centre_x(&self, w: f32) -> f32 {
        self.px((SAFE_W as f32 - w) / 2.0)
    }

    /// Panel x that centres something `w` wide on the *whole panel*. For things that belong
    /// to the screen rather than the layout, like a wordmark.
    pub fn panel_centre_x(&self, w: f32) -> f32 {
        (self.panel_w as f32 - w) / 2.0
    }

    /// Whether a panel-space rect lies entirely inside the safe area.
    pub fn contains(&self, x: f32, y: f32, w: f32, h: f32) -> bool {
        x >= self.x as f32
            && y >= self.y as f32
            && x + w <= (self.x + SAFE_W) as f32
            && y + h <= (self.y + SAFE_H) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offsets_follow_the_geometry() {
        let s = SafeArea::for_geometry(Geometry::W720H720);
        assert_eq!((s.x, s.y), (40, 120));
        assert_eq!(s.px(0.0), 40.0);
        assert_eq!(s.py(0.0), 120.0);
        assert_eq!(s.centre_x(100.0), 40.0 + 270.0);
        assert_eq!(s.panel_centre_x(100.0), 310.0);
        assert!(s.contains(40.0, 120.0, 640.0, 480.0));
        assert!(!s.contains(39.0, 120.0, 640.0, 480.0));
    }
}
