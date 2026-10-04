use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::ffi::{c_char, c_uint, c_void, CStr, CString};
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::Mutex;

use crate::ffi;
use crate::{AvInfo, CoreOption, Env, EnvLog, Error, Frame, JoypadMask, Memory, PixelFormat};

static BUSY: Mutex<Option<HashSet<PathBuf>>> = Mutex::new(None);

thread_local! {
    static CURRENT_SLOT: Cell<*mut Slot> = const { Cell::new(ptr::null_mut()) };
}

struct Slot {
    pixel_format: PixelFormat,
    av_info: AvInfo,
    frame: Option<Frame>,
    audio: Vec<i16>,
    inputs: [JoypadMask; 2],
    options: Vec<CoreOption>,
    option_values: HashMap<String, CString>,
    variables_dirty: bool,
    env: Env,
    env_log: EnvLog,
    system_dir_c: CString,
    save_dir_c: CString,
    rumble: [u16; 4], // port 0 (strong, weak), port 1 (strong, weak)
}

struct ActiveSlot<'a> {
    _slot: &'a mut Slot,
}

impl<'a> ActiveSlot<'a> {
    fn bind(slot: &'a mut Slot) -> Self {
        CURRENT_SLOT.with(|c| c.set(slot as *mut Slot));
        ActiveSlot { _slot: slot }
    }
}

impl Drop for ActiveSlot<'_> {
    fn drop(&mut self) {
        CURRENT_SLOT.with(|c| c.set(ptr::null_mut()));
    }
}

unsafe fn with_slot<R>(f: impl FnOnce(&mut Slot) -> R) -> Option<R> {
    let p = CURRENT_SLOT.with(|c| c.get());
    if p.is_null() {
        None
    } else {
        Some(f(&mut *p))
    }
}
struct BusyGuard {
    path: PathBuf,
}

impl Drop for BusyGuard {
    fn drop(&mut self) {
        let mut busy = BUSY.lock().unwrap();
        if let Some(set) = busy.as_mut() {
            set.remove(&self.path);
        }
    }
}

/// The C string a cheat code becomes, refusing only what cannot be one.
///
/// The code is not trimmed, split, upper-cased or otherwise inspected: a multi-part `+` code,
/// punctuation, spaces and UTF-8 all mean whatever the core's own parser says they mean.
fn cheat_code_c(code: &str) -> Result<CString, Error> {
    CString::new(code).map_err(|_| Error::Game("the cheat code contains a NUL byte".into()))
}

pub struct Core {
    api: ffi::Api,
    slot: Box<Slot>,
    #[allow(dead_code)]
    lib: libloading::Library,
    path: PathBuf,
    initialized: bool,
    game_loaded: bool,
    #[allow(dead_code)]
    rom_data: Option<Vec<u8>>,
    library_name: String,
    library_version: String,
    need_fullpath: bool,
}

impl Core {
    /// Open `dylib`, initialise it with `env`, load `rom`. See the module doc.
    pub fn load(dylib: &Path, rom: &Path, env: Env) -> Result<Self, Error> {
        if !dylib.exists() {
            return Err(Error::Load(format!("{} not found", dylib.display())));
        }
        let canon = dylib.canonicalize()?;
        {
            let mut busy = BUSY.lock().unwrap();
            let set = busy.get_or_insert_with(HashSet::new);
            if set.contains(&canon) {
                return Err(Error::Busy(canon));
            }
            set.insert(canon.clone());
        }
        let busy_guard = BusyGuard {
            path: canon.clone(),
        };

        let lib =
            unsafe { libloading::Library::new(dylib) }.map_err(|e| Error::Load(e.to_string()))?;
        let api = unsafe { ffi::Api::load(&lib) }?;

        let version = unsafe { (api.api_version)() };
        if version != ffi::RETRO_API_VERSION {
            return Err(Error::Load(format!("unsupported api version {}", version)));
        }

        let system_dir_c = CString::new(env.system_dir.to_string_lossy().as_bytes()).unwrap();
        let save_dir_c = CString::new(env.save_dir.to_string_lossy().as_bytes()).unwrap();

        let mut option_values = HashMap::new();
        for (k, v) in &env.options {
            option_values.insert(k.clone(), CString::new(v.as_str()).unwrap());
        }

        let mut slot = Box::new(Slot {
            pixel_format: PixelFormat::Rgb1555,
            av_info: AvInfo {
                fps: 0.0,
                sample_rate: 0.0,
                base_width: 0,
                base_height: 0,
                max_width: 0,
                max_height: 0,
                aspect_ratio: 0.0,
            },
            frame: None,
            audio: Vec::new(),
            inputs: [JoypadMask::default(), JoypadMask::default()],
            options: Vec::new(),
            option_values,
            variables_dirty: false,
            env,
            env_log: EnvLog::default(),
            system_dir_c,
            save_dir_c,
            rumble: [0; 4],
        });

        unsafe {
            let _a = ActiveSlot::bind(&mut slot);
            (api.set_environment)(environment);
            (api.init)();
        }

        let mut sys_info = unsafe { std::mem::zeroed::<ffi::retro_system_info>() };
        unsafe { (api.get_system_info)(&mut sys_info) };

        let library_name = unsafe {
            CStr::from_ptr(sys_info.library_name)
                .to_string_lossy()
                .into_owned()
        };
        let library_version = unsafe {
            CStr::from_ptr(sys_info.library_version)
                .to_string_lossy()
                .into_owned()
        };
        let need_fullpath = sys_info.need_fullpath;

        let rom_data = if !need_fullpath {
            Some(std::fs::read(rom)?)
        } else {
            None
        };

        let rom_path_c = CString::new(rom.to_string_lossy().as_bytes()).unwrap();
        let game_info = ffi::retro_game_info {
            path: rom_path_c.as_ptr(),
            data: rom_data
                .as_ref()
                .map_or(ptr::null(), |d| d.as_ptr() as *const c_void),
            size: rom_data.as_ref().map_or(0, |d| d.len()),
            meta: ptr::null(),
        };

        let ok = unsafe {
            let _a = ActiveSlot::bind(&mut slot);
            (api.load_game)(&game_info)
        };

        if !ok {
            return Err(Error::Game(format!("refused {}", rom.display())));
        }

        let mut av = unsafe { std::mem::zeroed::<ffi::retro_system_av_info>() };
        unsafe {
            let _a = ActiveSlot::bind(&mut slot);
            (api.get_system_av_info)(&mut av);
        }
        slot.av_info = AvInfo {
            fps: av.timing.fps,
            sample_rate: sane_sample_rate(av.timing.sample_rate),
            base_width: av.geometry.base_width,
            base_height: av.geometry.base_height,
            max_width: av.geometry.max_width,
            max_height: av.geometry.max_height,
            aspect_ratio: av.geometry.aspect_ratio,
        };
        unsafe {
            let _a = ActiveSlot::bind(&mut slot);
            (api.set_video_refresh)(video_refresh);
            (api.set_audio_sample)(audio_sample);
            (api.set_audio_sample_batch)(audio_sample_batch);
            (api.set_input_poll)(input_poll);
            (api.set_input_state)(input_state);
            (api.set_controller_port_device)(0, ffi::RETRO_DEVICE_JOYPAD);
            (api.set_controller_port_device)(1, ffi::RETRO_DEVICE_JOYPAD);
        }

        std::mem::forget(busy_guard);
        Ok(Core {
            api,
            slot,
            lib,
            path: canon,
            initialized: true,
            game_loaded: true,
            rom_data,
            library_name,
            library_version,
            need_fullpath,
        })
    }

    /// `retro_get_system_info`'s library name and version, e.g. `("mGBA", "0.11-dev")`.
    pub fn name(&self) -> (String, String) {
        (self.library_name.clone(), self.library_version.clone())
    }

    pub fn av_info(&self) -> AvInfo {
        self.slot.av_info
    }

    pub fn pixel_format(&self) -> PixelFormat {
        self.slot.pixel_format
    }

    /// Set the joypad state for `port` (0 or 1) that the next `run` will report.
    pub fn set_input(&mut self, port: usize, mask: JoypadMask) {
        if port < 2 {
            self.slot.inputs[port] = mask;
        }
    }

    /// One `retro_run`. Afterwards `frame()` has the picture (or the previous one, if the
    /// core duped) and `take_audio()` the samples produced.
    pub fn run(&mut self) {
        unsafe {
            let _a = ActiveSlot::bind(&mut self.slot);
            (self.api.run)();
        }
    }

    /// The last frame delivered. `None` before the first non-dupe `video_refresh`.
    pub fn frame(&self) -> Option<&Frame> {
        self.slot.frame.as_ref()
    }

    /// Interleaved stereo i16 produced since the last take. At `av_info().sample_rate`.
    pub fn take_audio(&mut self) -> Vec<i16> {
        std::mem::take(&mut self.slot.audio)
    }

    pub fn serialize_size(&self) -> usize {
        unsafe { (self.api.serialize_size)() }
    }

    pub fn serialize(&mut self) -> Result<Vec<u8>, Error> {
        let size = self.serialize_size();
        if size == 0 {
            return Err(Error::State("zero size".into()));
        }
        let mut buf = vec![0u8; size];
        let ok = unsafe {
            let _a = ActiveSlot::bind(&mut self.slot);
            (self.api.serialize)(buf.as_mut_ptr() as *mut c_void, size)
        };
        if ok {
            Ok(buf)
        } else {
            Err(Error::State("refused".into()))
        }
    }

    pub fn unserialize(&mut self, data: &[u8]) -> Result<(), Error> {
        if data.len() != self.serialize_size() {
            return Err(Error::State("wrong size".into()));
        }
        let ok = unsafe {
            let _a = ActiveSlot::bind(&mut self.slot);
            (self.api.unserialize)(data.as_ptr() as *const c_void, data.len())
        };
        if ok {
            Ok(())
        } else {
            Err(Error::State("refused".into()))
        }
    }

    /// A copy of a memory region, `None` if the core exposes none of that kind.
    pub fn memory(&self, which: Memory) -> Option<Vec<u8>> {
        let id = match which {
            Memory::SaveRam => ffi::RETRO_MEMORY_SAVE_RAM,
            Memory::Rtc => ffi::RETRO_MEMORY_RTC,
            Memory::SystemRam => ffi::RETRO_MEMORY_SYSTEM_RAM,
            Memory::VideoRam => ffi::RETRO_MEMORY_VIDEO_RAM,
        };
        let ptr = unsafe { (self.api.get_memory_data)(id) };
        let size = unsafe { (self.api.get_memory_size)(id) };
        if ptr.is_null() || size == 0 {
            return None;
        }
        Some(unsafe { std::slice::from_raw_parts(ptr as *const u8, size) }.to_vec())
    }

    /// Overwrite a memory region in place (e.g. restore save RAM). Errors if the sizes
    /// differ or the region does not exist.
    pub fn write_memory(&mut self, which: Memory, data: &[u8]) -> Result<(), Error> {
        let id = match which {
            Memory::SaveRam => ffi::RETRO_MEMORY_SAVE_RAM,
            Memory::Rtc => ffi::RETRO_MEMORY_RTC,
            Memory::SystemRam => ffi::RETRO_MEMORY_SYSTEM_RAM,
            Memory::VideoRam => ffi::RETRO_MEMORY_VIDEO_RAM,
        };
        let ptr = unsafe { (self.api.get_memory_data)(id) };
        let size = unsafe { (self.api.get_memory_size)(id) };
        if ptr.is_null() || size == 0 {
            return Err(Error::Game("no region".into()));
        }
        if size != data.len() {
            return Err(Error::Game("wrong size".into()));
        }
        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), ptr as *mut u8, size);
        }
        Ok(())
    }

    /// Reset the game (`retro_reset`).
    pub fn reset(&mut self) {
        unsafe {
            let _a = ActiveSlot::bind(&mut self.slot);
            (self.api.reset)();
        }
    }

    /// Drop everything the core is holding as a cheat (`retro_cheat_reset`).
    ///
    /// Nothing is re-applied here: a caller that wants a set of cheats back calls this once
    /// and then [`Core::set_cheat`] for each entry it wants, in its own order. Keeping the
    /// "clear, then put the file back" lifecycle in the caller is what lets a toggle change
    /// one index without the host guessing what the rest of the set was.
    pub fn reset_cheats(&mut self) {
        unsafe {
            let _a = ActiveSlot::bind(&mut self.slot);
            (self.api.cheat_reset)();
        }
    }

    /// Hand one cheat to the core (`retro_cheat_set`).
    ///
    /// The code crosses over exactly as it came in — punctuation, `+`-joined parts, spaces,
    /// UTF-8 — because what a code means is the core's business, not this host's. `enabled`
    /// is passed through too, including `false`: a caller clearing an index is asking the
    /// core, and this layer has no policy about which entries deserve to be sent.
    ///
    /// An interior NUL byte is the one thing refused, and it is refused before the core is
    /// touched: it cannot be part of a C string at all, and quietly truncating the code there
    /// would send a different cheat than the caller asked for.
    pub fn set_cheat(&mut self, index: u32, enabled: bool, code: &str) -> Result<(), Error> {
        let code = cheat_code_c(code)?;
        unsafe {
            let _a = ActiveSlot::bind(&mut self.slot);
            (self.api.cheat_set)(index, enabled, code.as_ptr());
        }
        Ok(())
    }

    /// The options the core declared, with current values applied.
    pub fn options(&self) -> Vec<CoreOption> {
        self.slot.options.clone()
    }

    pub fn option(&self, key: &str) -> Option<String> {
        self.slot
            .option_values
            .get(key)
            .map(|v| v.to_string_lossy().into_owned())
    }

    /// Change an option; the core picks it up at its next `GET_VARIABLE_UPDATE` poll.
    pub fn set_option(&mut self, key: &str, value: &str) {
        if let Ok(v) = CString::new(value) {
            self.slot.option_values.insert(key.to_string(), v);
            self.slot.variables_dirty = true;
        }
    }

    pub fn env_log(&self) -> EnvLog {
        self.slot.env_log.clone()
    }

    /// Whether the core wanted the ROM passed by path (`need_fullpath`).
    pub fn need_fullpath(&self) -> bool {
        self.need_fullpath
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        if self.game_loaded {
            unsafe {
                let _a = ActiveSlot::bind(&mut self.slot);
                (self.api.unload_game)();
            }
        }
        if self.initialized {
            unsafe {
                let _a = ActiveSlot::bind(&mut self.slot);
                (self.api.deinit)();
            }
        }
        let mut busy = BUSY.lock().unwrap();
        if let Some(set) = busy.as_mut() {
            set.remove(&self.path);
        }
    }
}

pub(crate) fn frame_to_rgba8(f: &Frame) -> Vec<u8> {
    let mut rgba = Vec::with_capacity((f.width * f.height * 4) as usize);
    let bpp = f.format.bytes_per_pixel();
    for y in 0..f.height {
        let row_start = y as usize * f.pitch;
        let row_end = row_start + (f.width as usize * bpp);
        if row_end > f.data.len() {
            break;
        }
        let row_data = &f.data[row_start..row_end];
        match f.format {
            PixelFormat::Rgb1555 => {
                let (chunks, _) = row_data.as_chunks::<2>();
                for chunk in chunks {
                    let p = u16::from_le_bytes(*chunk);
                    let r = ((p >> 10) & 0x1f) as u8;
                    let g = ((p >> 5) & 0x1f) as u8;
                    let b = (p & 0x1f) as u8;
                    rgba.extend_from_slice(&[
                        (r << 3) | (r >> 2),
                        (g << 3) | (g >> 2),
                        (b << 3) | (b >> 2),
                        255,
                    ]);
                }
            }
            PixelFormat::Rgb565 => {
                let (chunks, _) = row_data.as_chunks::<2>();
                for chunk in chunks {
                    let p = u16::from_le_bytes(*chunk);
                    let r = ((p >> 11) & 0x1f) as u8;
                    let g = ((p >> 5) & 0x3f) as u8;
                    let b = (p & 0x1f) as u8;
                    rgba.extend_from_slice(&[
                        (r << 3) | (r >> 2),
                        (g << 2) | (g >> 4),
                        (b << 3) | (b >> 2),
                        255,
                    ]);
                }
            }
            PixelFormat::Xrgb8888 => {
                let (chunks, _) = row_data.as_chunks::<4>();
                for chunk in chunks {
                    rgba.extend_from_slice(&[chunk[2], chunk[1], chunk[0], 255]);
                }
            }
        }
    }
    rgba
}

pub(crate) fn frame_rgb(f: &Frame, x: u32, y: u32) -> [u8; 3] {
    let bpp = f.format.bytes_per_pixel();
    let offset = (y as usize * f.pitch) + (x as usize * bpp);
    if offset + bpp > f.data.len() {
        return [0, 0, 0];
    }
    let p_data = &f.data[offset..offset + bpp];
    match f.format {
        PixelFormat::Rgb1555 => {
            let p = u16::from_le_bytes([p_data[0], p_data[1]]);
            let r = ((p >> 10) & 0x1f) as u8;
            let g = ((p >> 5) & 0x1f) as u8;
            let b = (p & 0x1f) as u8;
            [
                (r << 3) | (r >> 2),
                (g << 3) | (g >> 2),
                (b << 3) | (b >> 2),
            ]
        }
        PixelFormat::Rgb565 => {
            let p = u16::from_le_bytes([p_data[0], p_data[1]]);
            let r = ((p >> 11) & 0x1f) as u8;
            let g = ((p >> 5) & 0x3f) as u8;
            let b = (p & 0x1f) as u8;
            [
                (r << 3) | (r >> 2),
                (g << 2) | (g >> 4),
                (b << 3) | (b >> 2),
            ]
        }
        PixelFormat::Xrgb8888 => [p_data[2], p_data[1], p_data[0]],
    }
}

// --- Callbacks ---

unsafe extern "C" fn environment(cmd: c_uint, data: *mut c_void) -> bool {
    let base_cmd = cmd & !ffi::RETRO_ENVIRONMENT_EXPERIMENTAL;
    with_slot(|s| {
        if !s.env_log.answered.contains(&base_cmd) && !s.env_log.refused.contains(&base_cmd) {
            // Dedup record
        }

        match cmd {
            ffi::RETRO_ENVIRONMENT_SET_PIXEL_FORMAT => {
                let fmt = *(data as *const c_uint);
                match fmt {
                    ffi::RETRO_PIXEL_FORMAT_0RGB1555 => s.pixel_format = PixelFormat::Rgb1555,
                    ffi::RETRO_PIXEL_FORMAT_XRGB8888 => s.pixel_format = PixelFormat::Xrgb8888,
                    ffi::RETRO_PIXEL_FORMAT_RGB565 => s.pixel_format = PixelFormat::Rgb565,
                    _ => {
                        if !s.env_log.refused.contains(&base_cmd) {
                            s.env_log.refused.push(base_cmd);
                        }
                        return false;
                    }
                }
                if !s.env_log.answered.contains(&base_cmd) {
                    s.env_log.answered.push(base_cmd);
                }
                true
            }
            ffi::RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY => {
                *(data as *mut *const c_char) = s.system_dir_c.as_ptr();
                if !s.env_log.answered.contains(&base_cmd) {
                    s.env_log.answered.push(base_cmd);
                }
                true
            }
            ffi::RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY => {
                *(data as *mut *const c_char) = s.save_dir_c.as_ptr();
                if !s.env_log.answered.contains(&base_cmd) {
                    s.env_log.answered.push(base_cmd);
                }
                true
            }
            ffi::RETRO_ENVIRONMENT_GET_VARIABLE => {
                let var = &mut *(data as *mut ffi::retro_variable);
                let key = CStr::from_ptr(var.key).to_string_lossy().into_owned();
                if let Some(val) = s.option_values.get(&key) {
                    var.value = val.as_ptr();
                    if !s.env_log.answered.contains(&base_cmd) {
                        s.env_log.answered.push(base_cmd);
                    }
                    true
                } else {
                    var.value = ptr::null();
                    if !s.env_log.refused.contains(&base_cmd) {
                        s.env_log.refused.push(base_cmd);
                    }
                    false
                }
            }
            ffi::RETRO_ENVIRONMENT_SET_VARIABLES => {
                let mut vars = data as *const ffi::retro_variable;
                unsafe {
                    while !(*vars).key.is_null() {
                        let key = CStr::from_ptr((*vars).key).to_string_lossy().into_owned();
                        let val_str = CStr::from_ptr((*vars).value).to_string_lossy();
                        if let Some((desc, vals)) = val_str.split_once("; ") {
                            let values: Vec<String> =
                                vals.split('|').map(|s| s.trim().to_string()).collect();
                            let default = values.first().cloned().unwrap_or_default();
                            s.options.push(CoreOption {
                                key: key.clone(),
                                description: desc.to_string(),
                                values,
                                default: default.clone(),
                            });
                            s.option_values
                                .entry(key)
                                .or_insert_with(|| CString::new(default).unwrap());
                        }
                        vars = vars.add(1);
                    }
                }
                if !s.env_log.answered.contains(&base_cmd) {
                    s.env_log.answered.push(base_cmd);
                }
                true
            }
            ffi::RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE => {
                *(data as *mut bool) = s.variables_dirty;
                s.variables_dirty = false;
                if !s.env_log.answered.contains(&base_cmd) {
                    s.env_log.answered.push(base_cmd);
                }
                true
            }
            ffi::RETRO_ENVIRONMENT_GET_CORE_OPTIONS_VERSION => {
                *(data as *mut u32) = 1;
                if !s.env_log.answered.contains(&base_cmd) {
                    s.env_log.answered.push(base_cmd);
                }
                true
            }
            ffi::RETRO_ENVIRONMENT_SET_CORE_OPTIONS
            | ffi::RETRO_ENVIRONMENT_SET_CORE_OPTIONS_INTL => {
                let defs = if cmd == ffi::RETRO_ENVIRONMENT_SET_CORE_OPTIONS {
                    data as *const ffi::retro_core_option_definition
                } else {
                    (*(data as *const ffi::retro_core_options_intl)).us
                };
                let mut i = 0;
                unsafe {
                    while !(*defs.add(i)).key.is_null() {
                        let d = &*defs.add(i);
                        let key = CStr::from_ptr(d.key).to_string_lossy().into_owned();
                        let desc = CStr::from_ptr(d.desc).to_string_lossy().into_owned();
                        let mut values = Vec::new();
                        for v_idx in 0..128 {
                            if d.values[v_idx].value.is_null() {
                                break;
                            }
                            values.push(
                                CStr::from_ptr(d.values[v_idx].value)
                                    .to_string_lossy()
                                    .into_owned(),
                            );
                        }
                        let default = CStr::from_ptr(d.default_value)
                            .to_string_lossy()
                            .into_owned();
                        s.options.push(CoreOption {
                            key: key.clone(),
                            description: desc,
                            values,
                            default: default.clone(),
                        });
                        s.option_values
                            .entry(key)
                            .or_insert_with(|| CString::new(default).unwrap());
                        i += 1;
                    }
                }
                if !s.env_log.answered.contains(&base_cmd) {
                    s.env_log.answered.push(base_cmd);
                }
                true
            }
            ffi::RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2
            | ffi::RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2_INTL => {
                let v2 = if cmd == ffi::RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2 {
                    data as *const ffi::retro_core_options_v2
                } else {
                    (*(data as *const ffi::retro_core_options_v2_intl)).us
                };
                let defs = (*v2).definitions;
                let mut i = 0;
                unsafe {
                    while !(*defs.add(i)).key.is_null() {
                        let d = &*defs.add(i);
                        let key = CStr::from_ptr(d.key).to_string_lossy().into_owned();
                        let desc = CStr::from_ptr(d.desc).to_string_lossy().into_owned();
                        let mut values = Vec::new();
                        for v_idx in 0..128 {
                            if d.values[v_idx].value.is_null() {
                                break;
                            }
                            values.push(
                                CStr::from_ptr(d.values[v_idx].value)
                                    .to_string_lossy()
                                    .into_owned(),
                            );
                        }
                        let default = CStr::from_ptr(d.default_value)
                            .to_string_lossy()
                            .into_owned();
                        s.options.push(CoreOption {
                            key: key.clone(),
                            description: desc,
                            values,
                            default: default.clone(),
                        });
                        s.option_values
                            .entry(key)
                            .or_insert_with(|| CString::new(default).unwrap());
                        i += 1;
                    }
                }
                if !s.env_log.answered.contains(&base_cmd) {
                    s.env_log.answered.push(base_cmd);
                }
                true
            }
            ffi::RETRO_ENVIRONMENT_SET_CORE_OPTIONS_DISPLAY => {
                if !s.env_log.answered.contains(&base_cmd) {
                    s.env_log.answered.push(base_cmd);
                }
                true
            }
            ffi::RETRO_ENVIRONMENT_GET_LOG_INTERFACE => {
                if !s.env_log.refused.contains(&base_cmd) {
                    s.env_log.refused.push(base_cmd);
                }
                false // Refuse per instructions
            }
            ffi::RETRO_ENVIRONMENT_SET_GEOMETRY => {
                let geo = *(data as *const ffi::retro_game_geometry);
                s.av_info.base_width = geo.base_width;
                s.av_info.base_height = geo.base_height;
                s.av_info.aspect_ratio = geo.aspect_ratio;
                if !s.env_log.answered.contains(&base_cmd) {
                    s.env_log.answered.push(base_cmd);
                }
                true
            }
            ffi::RETRO_ENVIRONMENT_SET_SYSTEM_AV_INFO => {
                let av = *(data as *const ffi::retro_system_av_info);
                s.av_info = AvInfo {
                    fps: av.timing.fps,
                    sample_rate: sane_sample_rate(av.timing.sample_rate),
                    base_width: av.geometry.base_width,
                    base_height: av.geometry.base_height,
                    max_width: av.geometry.max_width,
                    max_height: av.geometry.max_height,
                    aspect_ratio: av.geometry.aspect_ratio,
                };
                if !s.env_log.answered.contains(&base_cmd) {
                    s.env_log.answered.push(base_cmd);
                }
                true
            }
            ffi::RETRO_ENVIRONMENT_GET_INPUT_BITMASKS => {
                if !s.env_log.answered.contains(&base_cmd) {
                    s.env_log.answered.push(base_cmd);
                }
                true
            }
            ffi::RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS
            | ffi::RETRO_ENVIRONMENT_SET_CONTROLLER_INFO
            | ffi::RETRO_ENVIRONMENT_SET_SUPPORT_ACHIEVEMENTS
            | ffi::RETRO_ENVIRONMENT_SET_SERIALIZATION_QUIRKS
            | ffi::RETRO_ENVIRONMENT_SET_MEMORY_MAPS
            | ffi::RETRO_ENVIRONMENT_SET_SUPPORT_NO_GAME
            | ffi::RETRO_ENVIRONMENT_SET_MINIMUM_AUDIO_LATENCY => {
                if !s.env_log.answered.contains(&base_cmd) {
                    s.env_log.answered.push(base_cmd);
                }
                true
            }
            ffi::RETRO_ENVIRONMENT_GET_LANGUAGE => {
                *(data as *mut u32) = s.env.language;
                if !s.env_log.answered.contains(&base_cmd) {
                    s.env_log.answered.push(base_cmd);
                }
                true
            }
            ffi::RETRO_ENVIRONMENT_GET_CAN_DUPE => {
                *(data as *mut bool) = true;
                if !s.env_log.answered.contains(&base_cmd) {
                    s.env_log.answered.push(base_cmd);
                }
                true
            }
            ffi::RETRO_ENVIRONMENT_GET_FASTFORWARDING => {
                *(data as *mut bool) = false;
                if !s.env_log.answered.contains(&base_cmd) {
                    s.env_log.answered.push(base_cmd);
                }
                true
            }
            ffi::RETRO_ENVIRONMENT_GET_AUDIO_VIDEO_ENABLE => {
                *(data as *mut i32) = 3;
                if !s.env_log.answered.contains(&base_cmd) {
                    s.env_log.answered.push(base_cmd);
                }
                true
            }
            ffi::RETRO_ENVIRONMENT_GET_RUMBLE_INTERFACE => {
                let iface = &mut *(data as *mut ffi::retro_rumble_interface);
                iface.set_rumble_state = set_rumble_state;
                if !s.env_log.answered.contains(&base_cmd) {
                    s.env_log.answered.push(base_cmd);
                }
                true
            }
            _ => {
                if !s.env_log.refused.contains(&base_cmd) {
                    s.env_log.refused.push(base_cmd);
                }
                false
            }
        }
    })
    .unwrap_or(false)
}

unsafe extern "C" fn set_rumble_state(port: c_uint, effect: c_uint, strength: u16) -> bool {
    with_slot(|s| {
        if port < 2 && effect < 2 {
            s.rumble[(port * 2 + effect) as usize] = strength;
            true
        } else {
            false
        }
    })
    .unwrap_or(false)
}

unsafe extern "C" fn video_refresh(
    data: *const c_void,
    width: c_uint,
    height: c_uint,
    pitch: usize,
) {
    with_slot(|s| {
        if data.is_null() {
            return;
        }
        let _bpp = s.pixel_format.bytes_per_pixel();
        let size = pitch * height as usize;
        let mut frame_data = vec![0u8; size];
        ptr::copy_nonoverlapping(data as *const u8, frame_data.as_mut_ptr(), size);
        s.frame = Some(Frame {
            width,
            height,
            pitch,
            format: s.pixel_format,
            data: frame_data,
        });
    });
}

unsafe extern "C" fn audio_sample(left: i16, right: i16) {
    with_slot(|s| {
        s.audio.push(left);
        s.audio.push(right);
    });
}

unsafe extern "C" fn audio_sample_batch(data: *const i16, frames: usize) -> usize {
    with_slot(|s| {
        let samples = std::slice::from_raw_parts(data, frames * 2);
        s.audio.extend_from_slice(samples);
        frames
    })
    .unwrap_or(0)
}

/// Take the core at its word about its audio rate, rejecting only what cannot be a rate at
/// all.
///
/// There used to be a "sanity" clamp here that rewrote anything above 50 kHz to 32768. It
/// was wrong: mGBA hands GBA audio over at 65536 Hz (`GBA_OUTPUT_RATE`) and says so, and
/// halving that number behind its back makes every GBA game play an octave low — then the
/// twice-too-fast stream overruns the audio ring and is chopped back to roughly the right
/// tempo, which turns an obvious fault into a vague graininess. A core that lies about its
/// rate is a bug to fix in the core, not to paper over here.
fn sane_sample_rate(rate: f64) -> f64 {
    if rate.is_finite() && (1_000.0..=384_000.0).contains(&rate) {
        rate
    } else {
        32_768.0
    }
}

unsafe extern "C" fn input_poll() {}

unsafe extern "C" fn input_state(port: c_uint, device: c_uint, _index: c_uint, id: c_uint) -> i16 {
    with_slot(|s| {
        if device == ffi::RETRO_DEVICE_JOYPAD && port < 2 {
            let mask = s.inputs[port as usize].0;
            if id == ffi::RETRO_DEVICE_ID_JOYPAD_MASK {
                mask as i16
            } else if id < 16 {
                ((mask >> id) & 1) as i16
            } else {
                0
            }
        } else {
            0
        }
    })
    .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cheat_code_reaches_the_c_string_byte_for_byte() {
        for code in [
            "7E007C9A",
            "12345678+9ABCDEF0",
            "0101 0202 0303",
            "[GameShark] 01",
            "체력 회복",
            "",
        ] {
            let c = cheat_code_c(code).unwrap_or_else(|e| panic!("{code:?} was refused: {e}"));
            assert_eq!(c.as_bytes(), code.as_bytes(), "{code:?} was rewritten");
        }
    }

    #[test]
    fn a_nul_inside_a_cheat_code_is_an_error_not_a_truncation() {
        let e = cheat_code_c("1234\u{0}5678").unwrap_err();
        assert!(matches!(e, Error::Game(_)), "{e}");
        assert!(e.to_string().contains("NUL"), "{e}");
    }
}
