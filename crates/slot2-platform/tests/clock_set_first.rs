use slot2_platform::clock::{set_utc_offset_min, utc_offset_min, OFFSET_MIN};

// A separate test process starts with an untouched offset, as future config loading will.
#[test]
fn setting_before_first_read_preserves_the_setting() {
    assert_eq!(set_utc_offset_min(OFFSET_MIN), Ok(()));
    assert_eq!(utc_offset_min(), OFFSET_MIN);
    assert_eq!(set_utc_offset_min(540), Ok(()));
    assert_eq!(utc_offset_min(), 540);
}
