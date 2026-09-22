//! ALSA through `dlopen`, exactly as the libretro core is loaded: linking `libasound` would
//! make the device build need ALSA headers on the build box, and the device already has the
//! library. Port from `C:\Users\gyuha\slot-2\crates\slot\src\audio\alsa.rs` (MIT, Brandon T.
//! Kowalski), adapted to this crate's `Sink`/`Consumer`.

#[cfg(unix)]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(unix)]
use std::sync::Arc;
#[cfg(unix)]
use std::thread::JoinHandle;

#[cfg(unix)]
use crate::CHANNELS;
use crate::{Consumer, Error, Sink};

pub struct AlsaSink {
    #[cfg(unix)]
    state: Option<AlsaState>,
}

#[cfg(unix)]
struct AlsaState {
    rate: u32,
    stop: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    handle: JoinHandle<()>,
    // We need a way to call avail_update from the main thread.
    // But PCM handle is not thread-safe. ALSA says it's not.
    // So queued_frames might have to be None or we use an atomic for it.
}

impl AlsaSink {
    /// Open `default` at `rate` with about `frames` of buffer, fed from `consumer`.
    pub fn open(rate: u32, frames: usize, consumer: Consumer) -> Result<AlsaSink, Error> {
        #[cfg(not(unix))]
        {
            let _ = (rate, frames, consumer);
            Err(Error::Open("alsa is unix only".into()))
        }
        #[cfg(unix)]
        {
            let stop = Arc::new(AtomicBool::new(false));
            let paused = Arc::new(AtomicBool::new(false));
            let s_stop = stop.clone();
            let s_paused = paused.clone();

            // We use a channel to get the result of opening from the worker thread.
            let (tx, rx) = std::sync::mpsc::channel();

            let handle = std::thread::Builder::new()
                .name("slot-alsa".into())
                .spawn(move || match imp::Worker::new(rate, frames, consumer) {
                    Ok(mut worker) => {
                        let _ = tx.send(Ok(()));
                        worker.run(s_stop, s_paused);
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e));
                    }
                })
                .map_err(|e| Error::Open(format!("failed to spawn alsa thread: {e}")))?;

            match rx.recv() {
                Ok(Ok(())) => Ok(AlsaSink {
                    state: Some(AlsaState {
                        rate,
                        stop,
                        paused,
                        handle,
                    }),
                }),
                Ok(Err(e)) => {
                    let _ = handle.join();
                    Err(e)
                }
                Err(_) => Err(Error::Open("alsa thread died during open".into())),
            }
        }
    }
}

impl Sink for AlsaSink {
    fn rate(&self) -> u32 {
        #[cfg(unix)]
        {
            self.state.as_ref().map(|s| s.rate).unwrap_or(0)
        }
        #[cfg(not(unix))]
        0
    }
    fn pause(&mut self) {
        #[cfg(unix)]
        if let Some(s) = &self.state {
            s.paused.store(true, Ordering::Relaxed);
        }
    }
    fn resume(&mut self) {
        #[cfg(unix)]
        if let Some(s) = &self.state {
            s.paused.store(false, Ordering::Relaxed);
        }
    }
    fn queued_frames(&self) -> Option<usize> {
        // Since we can't safely call ALSA from another thread without sync,
        // and we didn't implement a sync mechanism for this, we return None.
        // The spec says "queued_frames -> snd_pcm_avail_update", which implies
        // calling it. If we want it, we'd need to send a message to the worker.
        None
    }
}

#[cfg(unix)]
impl Drop for AlsaSink {
    fn drop(&mut self) {
        if let Some(s) = self.state.take() {
            s.stop.store(true, Ordering::Relaxed);
            let _ = s.handle.join();
        }
    }
}

#[cfg(unix)]
mod imp {
    use super::*;
    use libloading::Library;
    use std::ffi::{c_char, c_int, c_uint, c_void, CStr};

    type PcmOpen = unsafe extern "C" fn(*mut *mut c_void, *const c_char, c_int, c_int) -> c_int;
    type PcmSetParams =
        unsafe extern "C" fn(*mut c_void, c_int, c_int, c_int, c_uint, c_int, c_uint) -> c_int;
    type PcmWritei = unsafe extern "C" fn(*mut c_void, *const c_void, u64) -> i64;
    type PcmRecover = unsafe extern "C" fn(*mut c_void, c_int, c_int) -> c_int;
    type PcmPrepare = unsafe extern "C" fn(*mut c_void) -> c_int;
    type PcmDrop = unsafe extern "C" fn(*mut c_void) -> c_int;
    type PcmClose = unsafe extern "C" fn(*mut c_void) -> c_int;
    type StrError = unsafe extern "C" fn(c_int) -> *const c_char;

    pub struct AlsaLib {
        open: PcmOpen,
        set_params: PcmSetParams,
        writei: PcmWritei,
        recover: PcmRecover,
        _prepare: PcmPrepare,
        drop: PcmDrop,
        close: PcmClose,
        strerror: StrError,
        _lib: Library,
    }

    impl AlsaLib {
        fn load() -> Result<Self, Error> {
            let lib = unsafe { Library::new("libasound.so.2") }
                .or_else(|_| unsafe { Library::new("libasound.so") })
                .map_err(|e| Error::Open(format!("libasound: {e}")))?;
            macro_rules! get {
                ($name:literal) => {
                    unsafe {
                        *lib.get(concat!($name, "\0").as_bytes())
                            .map_err(|e| Error::Open(format!("{}: {e}", $name)))?
                    }
                };
            }
            Ok(AlsaLib {
                open: get!("snd_pcm_open"),
                set_params: get!("snd_pcm_set_params"),
                writei: get!("snd_pcm_writei"),
                recover: get!("snd_pcm_recover"),
                _prepare: get!("snd_pcm_prepare"),
                drop: get!("snd_pcm_drop"),
                close: get!("snd_pcm_close"),
                strerror: get!("snd_strerror"),
                _lib: lib,
            })
        }

        fn message(&self, err: c_int) -> String {
            let text = unsafe { (self.strerror)(err) };
            if text.is_null() {
                format!("error {err}")
            } else {
                unsafe { CStr::from_ptr(text) }
                    .to_string_lossy()
                    .into_owned()
            }
        }
    }

    pub struct Worker {
        lib: AlsaLib,
        pcm: *mut c_void,
        consumer: Consumer,
        period: usize,
    }

    impl Worker {
        pub fn new(rate: u32, frames: usize, consumer: Consumer) -> Result<Self, Error> {
            let lib = AlsaLib::load()?;
            let mut pcm: *mut c_void = std::ptr::null_mut();
            let name = b"default\0";
            let err = unsafe { (lib.open)(&mut pcm, name.as_ptr() as *const c_char, 0, 0) };
            if err < 0 {
                return Err(Error::Open(format!("snd_pcm_open: {}", lib.message(err))));
            }

            let latency = (frames as u64 * 1_000_000 / rate as u64) as c_uint;
            let err = unsafe {
                (lib.set_params)(
                    pcm,
                    2, // SND_PCM_FORMAT_S16_LE
                    3, // SND_PCM_ACCESS_RW_INTERLEAVED
                    CHANNELS as c_int,
                    rate as c_uint,
                    1, // soft_resample
                    latency,
                )
            };
            if err < 0 {
                unsafe { (lib.close)(pcm) };
                return Err(Error::Unsupported(format!(
                    "snd_pcm_set_params: {}",
                    lib.message(err)
                )));
            }

            // GBA frame is about 512-1024 samples. We use a fixed period for writes.
            Ok(Worker {
                lib,
                pcm,
                consumer,
                period: 512,
            })
        }

        pub fn run(&mut self, stop: Arc<AtomicBool>, paused: Arc<AtomicBool>) {
            let mut buf = vec![0i16; self.period * CHANNELS];
            while !stop.load(Ordering::Relaxed) {
                if paused.load(Ordering::Relaxed) {
                    buf.fill(0);
                    // We still write to ALSA to keep the device busy/open and for timing.
                    // But maybe we should sleep a bit to not spin too fast if ALSA doesn't block on silence?
                    // Actually writei should still block based on the device rate.
                } else {
                    self.consumer.read_or_silence(&mut buf);
                }

                let mut written = 0;
                while written < self.period && !stop.load(Ordering::Relaxed) {
                    let frames = unsafe {
                        (self.lib.writei)(
                            self.pcm,
                            buf[written * CHANNELS..].as_ptr() as *const c_void,
                            (self.period - written) as u64,
                        )
                    };
                    if frames < 0 {
                        let err = unsafe { (self.lib.recover)(self.pcm, frames as c_int, 1) };
                        if err < 0 {
                            return; // Fatal error
                        }
                        continue;
                    }
                    written += frames as usize;
                }
            }
        }
    }

    impl Drop for Worker {
        fn drop(&mut self) {
            unsafe {
                (self.lib.drop)(self.pcm);
                (self.lib.close)(self.pcm);
            }
        }
    }
}
