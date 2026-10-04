//! Runtime offset behavior, in its own binary so these tests share a process with nothing
//! else that changes the clock: the offset is process-global.

use slot2_platform::clock::{
    hud_local, is_set, set_utc_offset_min, utc_offset_min, OFFSET_MAX, OFFSET_MIN, SET_AFTER,
};

#[test]
fn setter_changes_reading_and_rejects_out_of_range() {
    let before = utc_offset_min();
    set_utc_offset_min(540).expect("KST is in range");
    assert_eq!(utc_offset_min(), 540);

    // Both edges and one past each; the stored value survives every rejection.
    for bad in [OFFSET_MIN - 1, OFFSET_MAX + 1, i32::MIN, i32::MAX] {
        assert_eq!(set_utc_offset_min(bad), Err(bad), "{bad}");
        assert_eq!(
            utc_offset_min(),
            540,
            "rejected {bad} must not move the clock"
        );
    }

    // Boundaries themselves are legal, negative included.
    set_utc_offset_min(OFFSET_MIN).expect("lower bound");
    assert_eq!(utc_offset_min(), OFFSET_MIN);
    set_utc_offset_min(OFFSET_MAX).expect("upper bound");
    assert_eq!(utc_offset_min(), OFFSET_MAX);

    // Restore where the process started, in case an env offset was configured.
    let _ = set_utc_offset_min(before);
}

#[test]
fn hud_visibility_judged_on_utc_not_offset() {
    // One second before the threshold: no offset, however large, may rescue it.
    let before = SET_AFTER - 1;
    assert!(!is_set(before));
    assert_eq!(hud_local(before, 14 * 60), None);
    assert_eq!(hud_local(before, -12 * 60), None);

    // At the threshold: shown, even when a negative offset moves the display below it.
    let after = SET_AFTER;
    assert!(is_set(after));
    assert_eq!(hud_local(after, 14 * 60), Some(after + 14 * 60 * 60));
    assert_eq!(hud_local(after, -12 * 60), Some(after - 12 * 60 * 60));
    assert_eq!(hud_local(i64::MAX, OFFSET_MAX), None);
}
