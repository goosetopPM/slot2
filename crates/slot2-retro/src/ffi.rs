//! libretro.h, the parts this host uses: constants, C structs, callback signatures.
//! Task 06 fills this in — port from `C:\Users\gyuha\slot-2\crates\slot-retro\src\ffi.rs`
//! and extend with the structs the core-option v1/v2 commands and `SET_GEOMETRY` /
//! `SET_SYSTEM_AV_INFO` need:
//!
//! - `retro_system_info { library_name, library_version, valid_extensions: *const c_char,
//!   need_fullpath: bool, block_extract: bool }`
//! - `retro_game_geometry { base_width, base_height, max_width, max_height: c_uint, aspect_ratio: f32 }`
//! - `retro_system_timing { fps: f64, sample_rate: f64 }`
//! - `retro_system_av_info { geometry, timing }`
//! - `retro_game_info { path: *const c_char, data: *const c_void, size: usize, meta: *const c_char }`
//! - `retro_variable { key: *const c_char, value: *const c_char }`
//! - `retro_core_option_value { value, label: *const c_char }` (`[_; 128]` per definition),
//!   `retro_core_option_definition { key, desc, info, values: [retro_core_option_value; 128], default_value }`,
//!   `retro_core_options_intl { us, local: *const retro_core_option_definition }`,
//!   `retro_core_option_v2_definition { key, desc, desc_categorized, info, info_categorized,
//!   category_key, values: [retro_core_option_value; 128], default_value }`,
//!   `retro_core_options_v2 { categories, definitions: *const _ }`,
//!   `retro_core_options_v2_intl { us, local: *const retro_core_options_v2 }`
//! - `retro_rumble_interface { set_rumble_state: unsafe extern "C" fn(port: c_uint, effect: c_uint, strength: u16) -> bool }`
//! - All `RETRO_ENVIRONMENT_*` numbers named in host.rs, `RETRO_ENVIRONMENT_EXPERIMENTAL = 0x10000`,
//!   `RETRO_DEVICE_JOYPAD = 1`, `RETRO_DEVICE_ID_JOYPAD_MASK = 256`, `RETRO_MEMORY_SAVE_RAM = 0`,
//!   `RETRO_MEMORY_RTC = 1`, `RETRO_MEMORY_SYSTEM_RAM = 2`, `RETRO_MEMORY_VIDEO_RAM = 3`,
//!   pixel format enum values `0RGB1555 = 0, XRGB8888 = 1, RGB565 = 2`, `RETRO_API_VERSION = 1`.
//! - Callback typedefs: `retro_environment_t`, `retro_video_refresh_t`, `retro_audio_sample_t`,
//!   `retro_audio_sample_batch_t`, `retro_input_poll_t`, `retro_input_state_t`.
//!
//! Everything `#[repr(C)]`; pointer fields raw. No behaviour here, only layout.
