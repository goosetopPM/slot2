//! The binary's library face, so integration tests can drive the same code the app runs.

pub mod app;
pub mod probe;
pub mod session;

/// What the cores may spend on this machine.
///
/// The panel comes from the detected profile — SLOT2 runs on 640x480, 720x480 and 720x720
/// handhelds and a core configured for one of those is not configured for the others. The
/// cycles question is cruder: every device target here is an Allwinner H700, so only the
/// host build gets to claim a fast CPU.
pub fn tuning_for(profile: &slot2_platform::Profile) -> slot2_retro::Tuning {
    let (w, h) = profile.geometry.size();
    if profile.target == "host" {
        slot2_retro::Tuning {
            geometry: (w, h),
            fast_cpu: true,
        }
    } else {
        slot2_retro::Tuning::handheld((w, h))
    }
}
