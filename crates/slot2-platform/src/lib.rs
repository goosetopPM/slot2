//! What the frontend needs to know about the box it is running in, and the one file it learns
//! it from: BaseOS writes `BASEOS_TARGET=<id>` into `/etc/baseos-release`, one id per device
//! image (`devices.json` upstream). Everything else — panel size, whether there is a lid to
//! close, whether there are sticks — is a table keyed on that id.
//!
//! On the host there is no such file, so `SLOT2_TARGET` names a profile directly and
//! `SLOT2_GEOMETRY` overrides the panel, which is how one PC window stands in for three
//! device shapes.

pub mod battery;
pub mod clock;
pub mod power;
pub mod profile;

pub use battery::{Battery, Charge, Gauge};
pub use power::PowerAction;
pub use profile::{by_target, detect, Detected, Geometry, Profile, PROFILES};
