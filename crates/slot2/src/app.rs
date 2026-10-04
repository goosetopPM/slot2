//! The frontend's screen state machine, shared by both backends and driven by input
//! actions. Implementation notes for task 05 (see tasks/05-input.md):
//!
//! ```text
//! Screen::Splash
//!   Hold(Menu)            → Screen::Power(PowerMenu::default())
//! Screen::List
//!   Tap(A)                → Screen::Inserting with resume-if-present intent
//!   Hold(A)               → Screen::Inserting with fresh intent; the hold that started it
//!                           is kept from the core until the player lets go
//!   the rest              → the row, the platform, or a refusal on an empty shelf
//!   Tap(Menu)             → Screen::Shelf(ShelfMenu) for the settings menu, over the shelf
//! Screen::Shelf(menu)
//!   Tap(Up) / Tap(Down)   → menu.up() / menu.down(), both wrapping over the rows that can be
//!                           opened; an entry with no screen behind it is skipped
//!   Tap(A)                → on the Language row, Screen::Language(menu) for the languages this
//!                           machine can speak; on the Time zone row, Screen::Timezone(menu,
//!                           TimezoneMenu::new(the running offset)); on the About row,
//!                           Screen::About(menu, AboutSticker); any other row is not openable
//!                           and does nothing
//!   Tap(B) | Tap(Menu)    → Screen::List
//! Screen::Language(parent)
//!   Tap(Up) / Tap(Down)   → the picker's own walk, wrapping
//!   Tap(A)                → the language already running: Screen::Shelf(parent). A different
//!                           one: the choice is requested, and the backend builds the new
//!                           context and saves it before the next frame; while that is in
//!                           flight the screen takes no more input
//!   Tap(B) | Tap(Menu)    → Screen::Shelf(parent), nothing written and nothing requested
//! Screen::About(parent, sticker)
//!   Tap(B) | Tap(Menu)    → Screen::Shelf(parent), the same About row
//!   the rest              → nothing: the sticker has no controls and no links
//! Screen::Timezone(parent, menu)
//!   Tap(Left) / Tap(Right)→ the value a quarter-hour down / up, on the running clock at once
//!   Tap(Down) / Tap(Up)   → the value an hour down / up, likewise
//!   Tap(A)                → no change: Screen::Shelf(parent), nothing written; a change is
//!                           written to the card, and a write that fails puts the offset, the
//!                           menu and the screen back the way they were and toasts
//!                           `timezone-save-failed`
//!   Tap(B) | Tap(Menu)    → the offset goes back to what the card holds, nothing is written,
//!                           Screen::Shelf(parent)
//! Screen::Playing
//!   Tap(Menu)             → Screen::InGame(InGameMenu::default())
//!   Hold(Menu)            → the session stops and the cart comes out (Screen::Ejecting)
//!   Hold(R2)              → fast forward while it is held, and only while it is held
//!   DoubleTap(R2)         → the fast-forward latch: on until the next double tap, and cleared
//!                           with the session that was running
//!   Hold(L2)              → rewind, which outranks both of them
//!   Chord(R1)             → SELECT+R1: save the next numbered state (toast `state-saved`,
//!                           or `state-save-failed`; the game keeps running either way)
//!   Chord(L1)             → SELECT+L1: load the greatest numbered state (toast
//!                           `state-loaded`, `states-empty` when there is none, or
//!                           `state-load-failed`)
//! Both chords are on `Screen::Playing` only: on a shelf, mid-animation, or under a menu
//! they do nothing at all.
//! Screen::InGame(menu)
//!   Tap(Up) / Tap(Down)   → menu.up() / menu.down(), both wrapping
//!   Tap(A)                → match menu.choice():
//!                              Continue  → Screen::Playing (same session)
//!                              SaveState → Screen::Switcher(menu) over the card's numbered states
//!                              Cheats    → Screen::Cheats(menu, CheatMenu::new(len)) for the
//!                                          session's own list
//!                              Display   → Screen::Display(menu, DisplayMenu::new(scale,
//!                                          the platform crops at all)) for the game's own
//!                                          stored scale, with the shader submenu after the
//!                                          four scale settings, the overscan submenu after
//!                                          that on the platforms whose registry entry crops
//!                                          something, and the overlay submenu last
//!                              Core      → Screen::Core(menu) for the platform's installed cores
//!                              Device    → Screen::Device(menu, DeviceMenu::new(volume)) for the
//!                                          volume that is running; brightness and blue light
//!                                          have no backend, so those two rows arrive unavailable
//!                              Eject     → session stops, Screen::Ejecting
//!                              the rest  → the menu stays open (nothing behind it yet)
//!   Tap(B) | Tap(Menu)    → Screen::Playing (same session)
//! Screen::Display(parent, menu)
//!   Tap(Up) / Tap(Down)   → menu.up() / menu.down(), both wrapping
//!   Tap(A)                → on a scale row, save the highlighted scale for this game, put it
//!                           on the running session at once, and stay open; on the shader row,
//!                           Screen::Shader(parent, menu, ShaderMenu::new(the card's shader));
//!                           on the overscan row, Screen::Overscan(parent, menu,
//!                           OverscanMenu::new(the card's overscan)); on the overlay row,
//!                           Screen::Overlay(parent, menu, OverlayMenu::new(the card's
//!                           overlay)); a write that fails logs and toasts
//!                           `display-save-failed` and leaves the session as it was
//!   Tap(B) | Tap(Menu)    → Screen::InGame(parent), the same row
//! Screen::Shader(parent, display, menu)
//!   Tap(Up) / Tap(Down)   → menu.up() / menu.down(), both wrapping
//!   Tap(A)                → save the highlighted shader for this game, put it on the running
//!                           session at once, and stay open on the row; a write that fails logs
//!                           and toasts `shader-save-failed` and leaves the card and the picture
//!                           as they were
//!   Tap(B) | Tap(Menu)    → Screen::Display(parent, display), the same shader row
//! Screen::Overscan(parent, display, menu)
//!   Tap(Up) / Tap(Down)   → menu.up() / menu.down(), both wrapping
//!   Tap(A)                → save the highlighted overscan for this game, put it on the running
//!                           session at once, and stay open on the row; a write that fails logs
//!                           and toasts `overscan-save-failed` and leaves the card and the
//!                           picture as they were
//!   Tap(B) | Tap(Menu)    → Screen::Display(parent, display), the same overscan row
//! Screen::Overlay(parent, display, menu)
//!   Tap(Up) / Tap(Down)   → menu.up() / menu.down(), both wrapping
//!   Tap(A)                → save the highlighted overlay choice for this game, put it on the
//!                           running game's picture at once, and stay open on the row; a write
//!                           that fails logs and toasts `overlay-save-failed` and leaves the card
//!                           and the picture as they were
//!   Tap(B) | Tap(Menu)    → Screen::Display(parent, display), the same overlay row
//! Screen::Cheats(parent, menu)
//!   Tap(Up) / Tap(Down)   → menu.up() / menu.down(), both wrapping
//!   Tap(A)                → turn the highlighted cheat over in the running session, at once, and
//!                           stay open; an index the session does not have, or a session that
//!                           refuses, logs and toasts `cheat-toggle-failed` and leaves the
//!                           session as it was
//!   Tap(B) | Tap(Menu)    → Screen::InGame(parent), the same row
//! Screen::Switcher(menu)
//!   Tap(Left) / Tap(Right)→ the switcher's navigation, both wrapping
//!   Tap(A)                → load the selected slot: `state-loaded` and Screen::Playing, or
//!                           `states-empty` / `state-load-failed` and the switcher stays put
//!   Tap(X)                → take the selected slot off the card into the pending undo
//!                           (`state-deleted`, or `state-delete-failed`), and stay here
//!   Tap(Y)                → put the pending undo back (`state-restored`, `state-restore-failed`,
//!                           or `undo-empty` when there is none left); STATE_UNDO_S after the
//!                           delete it is gone for good
//!   Tap(B) | Tap(Menu)    → Screen::InGame(menu), the same row
//! Screen::Core(menu)
//!   Tap(Up) / Tap(Down)   → the picker's navigation, both wrapping
//!   Tap(A)                → run the game on the highlighted core: checkpoint the running
//!                           session, open the chosen core, save the choice, and hand the game
//!                           back (Screen::Playing); each step's failure comes back to the core
//!                           the settings still name and says so (`core-checkpoint-failed`,
//!                           `core-switch-failed`, `core-setting-save-failed`,
//!                           `core-recovery-state-failed`, `core-recovery-failed`), a
//!                           platform with nothing to choose between never opens the picker
//!                           (`core-no-alternatives`), and an empty one does nothing
//!                           (`core-picker-empty`)
//!   Tap(B) | Tap(Menu)    → Screen::InGame(menu), the same Core row
//! Screen::Device(parent, menu)
//!   Tap(Up) / Tap(Down)   → menu.up() / menu.down(); only the volume is available here, so
//!                           both directions stay on it
//!   Tap(Left) / Tap(Right)→ the running volume one step down / up (Volume::STEP) and the menu
//!                           re-read from it, or nothing when the row is not the volume
//!   Tap(A)                → mute or unmute the running volume, level untouched
//!   Tap(VolUp)/Tap(VolDown)→ the running volume, as on every other screen; an open Device menu
//!                           is re-read from it so the picture cannot go stale
//!   Tap(B) | Tap(Menu)    → Screen::InGame(parent), the same Device row
//! Screen::Power(menu)
//!   Tap(Up) / Tap(Down)   → menu.up() / menu.down()
//!   Tap(A)                → match menu.choice():
//!                              Resume   → Screen::Splash
//!                              Restart  → the volume is flushed, exit = Some(Exit::Reboot)
//!                              PowerOff → the volume is flushed, exit = Some(Exit::PowerOff)
//!   Tap(B) | Tap(Menu)    → Screen::Splash
//! any screen
//!   Tap(Power)            → the volume is flushed to the card, then exit = Some(Exit::PowerOff)
//!   Tap(VolUp)/Tap(VolDown)→ the running volume one step up / down, wherever the player is
//! ```
//! `feed` runs the event through `state` (always) and `gestures`, then `act`s on each
//! action in order. `tick` runs `gestures.tick`, and then writes the volume if its delay is
//! up.
//!
//! The volume the player sets is the frontend's own, and it outlives the run: the level a
//! machine starts on comes from the card's global settings file, read once in `with_card`,
//! and the level a press leaves behind is written back once the presses stop —
//! [`VOLUME_SAVE_DELAY_MS`] after the last real change — and immediately before any exit the
//! app starts itself (`Tap(Power)`, Restart, PowerOff), so a press a moment before switching
//! off is not thrown away. Mute is never written: a machine that came back up silent would be
//! worse than one that came back at the level the player chose.
//!
//! `draw` draws the splash, then a hold
//! progress bar when `gestures.hold_progress(Menu, now)` is `Some(p)`: a rect at the
//! bottom of the safe area, `x = safe.px(0)`, `y = safe.py(SAFE_H - 4)`, width
//! `SAFE_W * p`, height 4, colour `splash::INK_DIM`; then the menu for the screen on show —
//! `Screen::Power` draws its own over whatever was underneath, and the in-game menu and its
//! Display, Shader, Overscan, Overlay, Cheats, Switcher, Core and Device submenus draw the
//! session's last
//! frame and then their overlay over it, with neither the shelf's wallpaper nor the HUD (the game
//! still owns the screen). The
//! game's own overlay picture (D-11) goes between those two, on every screen that draws a running
//! game: `session frame → overlay → time-control badge → menus, dims, progress and messages`. It
//! is resolved once per launch, decoded and uploaded by the first frame that draws it, and freed
//! when the game goes; a screen with no game draws neither. The
//! time-control badge is drawn over a running game only — right after its frame and under
//! anything transient — and is hidden on every other screen, including under a menu or the
//! switcher while the fast-forward latch is still set. Once `exit` is set no further actions
//! change anything.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use slot2_gfx::Canvas;
use slot2_input::{Action, Button, Event, GestureConfig, Gestures, State};
use slot2_platform::{clock, Battery, Gauge};
use slot2_retro::LogicalButton;
use slot2_store::{Card, Cart, GlobalSettings, Platform, StateBackup, StateKind};
use slot2_ui::{
    hud::TimeControl, insert, shelf_view::ShelfView, AboutInfo, AboutSticker, CheatMenu,
    CorePicker, DeviceMenu, DeviceSetting, DisplayChoice, DisplayMenu, InGameChoice, InGameMenu,
    LanguageOption, LanguagePicker, OverlayMenu, OverscanMenu, PowerMenu, ShaderMenu,
    ShelfAvailability, ShelfChoice, ShelfMenu, Splash, StateSwitcher, TimezoneMenu, UiCtx,
};

use crate::overlay::{self, OverlayLayer};
use crate::session::Session;

// The store owns the card's file format and the clock owns the runtime offset, and the two
// ranges have to agree: a card that accepts a value the clock then refuses would be a setting
// nobody can explain. This is part of the production build rather than a test, so the two
// crates cannot drift apart in a build that ships — the mismatch fails `slot2`'s own compile.
// The values are not adjusted here: if they ever differ, that is a contract error to report,
// not a number to quietly agree with the other crate.
const _: () = {
    assert!(slot2_store::UTC_OFFSET_MINUTES_MIN == slot2_platform::clock::OFFSET_MIN);
    assert!(slot2_store::UTC_OFFSET_MINUTES_MAX == slot2_platform::clock::OFFSET_MAX);
    // The default the store hands back for every unset or invalid card has to be a value the
    // clock takes, or the startup path below would take its rejection branch on a healthy card.
    assert!(slot2_store::DEFAULT_UTC_OFFSET_MINUTES == 0);
    assert!(slot2_store::DEFAULT_UTC_OFFSET_MINUTES >= slot2_platform::clock::OFFSET_MIN);
    assert!(slot2_store::DEFAULT_UTC_OFFSET_MINUTES <= slot2_platform::clock::OFFSET_MAX);
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exit {
    PowerOff,
    Reboot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    /// Before a card has been scanned, and in the M0 tests.
    Splash,
    List,
    /// A cart on its way into the slot.
    Inserting,
    /// A cart on its way back out of it.
    Ejecting,
    Playing,
    /// The in-game menu, over the game's last frame. The session behind it is untouched.
    InGame(InGameMenu),
    /// The Display submenu, over the same frame. It carries the in-game menu it was opened from
    /// so closing it lands back on the same row; the menu itself lives in `slot2-ui`, and its
    /// last rows open the shader, overscan and overlay submenus rather than changing anything
    /// here.
    Display(InGameMenu, DisplayMenu),
    /// The shader submenu, over the same frame. It carries the in-game menu and the Display menu
    /// it was opened from — the latter still on its Shader row — so closing it lands one step
    /// back rather than at a fresh menu. All three menus are `Copy`, which is what keeps
    /// `Screen` `Copy`.
    Shader(InGameMenu, DisplayMenu, ShaderMenu),
    /// The overscan submenu, over the same frame, for the platforms whose registry entry crops
    /// something. It carries the parents the shader screen carries, for the same reason and with
    /// the same result: closing it lands back on the Display menu's Overscan row. All three menus
    /// are `Copy`, which is what keeps `Screen` `Copy`.
    Overscan(InGameMenu, DisplayMenu, OverscanMenu),
    /// The overlay submenu, over the same frame. It carries the menu pair the shader and overscan
    /// screens carry, for the same reason and with the same result: closing it lands back on the
    /// Display menu's Overlay row. All three menus are `Copy`, which is what keeps `Screen` `Copy`.
    Overlay(InGameMenu, DisplayMenu, OverlayMenu),
    /// The cheats submenu, over the same frame. It carries the in-game menu it was opened from so
    /// closing it lands back on the same row. The list drawn is the session's, so only the menu's
    /// own walking state is here — `CheatMenu` is `Copy`, which is what keeps `Screen` `Copy`.
    Cheats(InGameMenu, CheatMenu),
    /// The state switcher, over the same frame. It carries the menu it was opened from so
    /// closing it lands back on the same row; the switcher itself owns textures and so lives
    /// in `App` rather than in here, which is what keeps `Screen` `Copy`.
    Switcher(InGameMenu),
    /// The core picker, over the same frame. It carries the menu it was opened from so closing
    /// it lands back on the same Core row; the picker owns its candidate list and so lives in
    /// `App` rather than in here, for the same reason the switcher does.
    Core(InGameMenu),
    /// The Device submenu, over the same frame. It carries the in-game menu it was opened from
    /// so closing it lands back on the same Device row, and a snapshot of the volume that is
    /// running: the level and the mute come from `App::volume`, which owns them, and are
    /// re-read from it after every change rather than changed alongside it.
    Device(InGameMenu, DeviceMenu),
    /// The shelf's settings menu, over the shelf itself. Every entry is on it in the order M4
    /// settles, and the ones with no screen behind them yet are drawn as unavailable; the time
    /// zone and About can be opened in this build. The menu is carried rather than owned by `App`
    /// because it is `Copy` and losing the highlight on a frame boundary would be a bug, the
    /// same reason `PowerMenu` is carried.
    Shelf(ShelfMenu),
    /// The time zone screen, over the same shelf. It carries the settings menu it was opened
    /// from — still on the Time zone row — so closing it lands back there, and the offset menu
    /// itself, which holds both the value the screen opened on and the value on screen now.
    Timezone(ShelfMenu, TimezoneMenu),
    /// The build sticker, over the same shelf. It carries the settings menu it was opened from —
    /// still on the About row — so closing it lands back there. The sticker itself has no state;
    /// what it prints is handed to it when it is drawn.
    About(ShelfMenu, AboutSticker),
    /// The language picker, over the same shelf. It carries the settings menu it was opened from
    /// — still on the Language row — so closing it lands back there. The picker owns its list of
    /// languages, so it lives in `App` rather than in here, which is what keeps `Screen` `Copy`.
    Language(ShelfMenu),
    Power(PowerMenu),
}

/// What the app wants the loop to do about audio after the last `feed`/`tick`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SinkRequest {
    /// A session just started; take its consumer and open a sink.
    Open,
    /// The session ended; drop the sink.
    Close,
}

/// How often the gauge is actually asked, in seconds.
///
/// Sysfs is a file read and the frontend draws sixty times a second. Measured against the
/// `Instant` `tick` is handed, never against the animation `dt`: that dt is clamped to 1/30 s
/// so a slow frame cannot destabilise the row spring, and a poll counted in clamped dt would
/// need three hundred ticks to reach ten seconds.
pub const BATTERY_POLL_S: f32 = 10.0;

/// How much faster R2 makes it. Four is the most that is still followable on a handheld
/// screen, and the A53 can hold it for every core here.
pub const FAST_FORWARD: u32 = 4;

/// How long a deleted state can be put back, in seconds.
///
/// Measured on the tick clock, never in animation `dt`: that dt is clamped so a slow frame
/// cannot destabilise the row spring, and a clock that stops for a slow frame is not a clock.
pub const STATE_UNDO_S: u64 = 30;

/// How long the volume level must stop changing before it is written to the card.
///
/// A player holding VOL+ walks the level a step at a time, and a write per step would be a
/// dozen SD writes for one gesture. The delay is measured on the tick clock, never in
/// animation `dt`: that dt is clamped so a stalled frame cannot destabilise the row spring,
/// and a delay counted in it would stretch a frame that loaded a core into seconds.
pub const VOLUME_SAVE_DELAY_MS: u64 = 750;

/// What a launch is for: the playthrough the player left off in, or a new game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Launch {
    /// Load `StateKind::Resume` when there is one. The shelf decides nothing: a card changed
    /// since the row was scanned still gets a game, just not the old one.
    Resume,
    /// A new game, and the old resume state goes with it.
    Fresh,
}

/// The cart going into the slot, and what the launch is for.
///
/// Decided when the gesture is accepted rather than at the seat, so a row that moves behind
/// the animation cannot load a different game from the one the player pointed at.
#[derive(Clone, Debug, PartialEq, Eq)]
struct PendingLaunch {
    cart: Cart,
    intent: Launch,
}

/// A state that has been taken off the card, and when it stops being puttable back.
///
/// One of these, at most: a second delete replaces it, and replacing it is what makes the older
/// deletion permanent. There is no history, no trash directory and no delayed filesystem work.
struct PendingUndo {
    backup: StateBackup,
    expires: Instant,
}

pub struct App {
    pub screen: Screen,
    pub state: State,
    pub gestures: Gestures,
    pub splash: Splash,
    pub shelf_view: ShelfView,
    /// What the shelf stands on. Nothing else paints the whole panel.
    pub wallpaper: slot2_ui::wallpaper::Wallpaper,
    pub volume: slot2_audio::Volume,
    card: Card,
    core_dir: PathBuf,
    sink_rate: u32,
    tuning: slot2_retro::Tuning,
    platform_index: usize,
    carts: Vec<Cart>,
    session: Option<Session>,
    /// The game's own overlay picture, when it has one (D-11).
    ///
    /// Owned here rather than by the session: the session owns the core's frame, the shader and
    /// the crop, and an overlay is a picture laid over that — a card file the frontend decoded
    /// once, not something the core ever sees. The sources are resolved at the launch boundary
    /// and the texture is decoded and uploaded lazily by the first frame that draws it.
    overlay: OverlayLayer,
    /// Set when a session starts; the loop takes it to build its sink.
    pending_consumer: Option<slot2_audio::Consumer>,
    sink_request: Option<SinkRequest>,
    exit: Option<Exit>,
    last_tick: Option<Instant>,
    anim: f32,
    /// A flinch in progress: the answer to an action that will not happen. Only ever drawn
    /// when there is no cart on screen to carry the refusal itself.
    refusal: Option<slot2_ui::refusal::Refusal>,
    /// What the player is being told, if anything.
    toast: Option<slot2_ui::toast::Toast>,
    /// Whether the clip for the animation in progress has been fired. The contacts happen
    /// once per insert, and so does the noise they make.
    sfx_fired: bool,
    /// Where to ask about the charge, and the last thing it said. `Gauge::none()` until a
    /// backend hands over a real one, so a test sees no battery until it says otherwise.
    gauge: Gauge,
    battery: Option<Battery>,
    /// When the gauge was last read, on the tick clock.
    battery_read: Option<Instant>,
    hud: slot2_ui::Hud,
    /// The numbered states on the card, as the switcher draws them. One instance for the
    /// life of the app: it owns the thumbnail textures, so a fresh one per visit would leave
    /// the canvas holding ids nothing will ever free.
    state_switcher: StateSwitcher,
    /// The cores this platform could be run with, as the picker draws them. One instance for
    /// the life of the app, like the switcher: the candidate list is built when the Core row
    /// is opened — the only moment the core directory is read — and draw only reads it.
    core_picker: CorePicker,
    /// The one state a delete took off the card, waiting to be put back. Private, and the only
    /// copy: the bytes are not in `Screen` or in the switcher.
    undo: Option<PendingUndo>,
    /// The cart chosen for the slot, and what the launch is for. Taken when the core is
    /// opened, at the seat.
    launch: Option<PendingLaunch>,
    /// True while a launch press is still under the player's thumb: the core must not see a
    /// button that went down before the game started.
    suppress_a: bool,
    /// True while the player has locked fast forward on with an R2 double tap.
    ///
    /// It belongs to the session: stopping one clears it, so a game launched afterwards always
    /// starts at normal speed.
    ff_latch: bool,
    /// The volume level the card is known to hold, which is what a level is compared against
    /// before anything is written: a change back to it cancels the write instead of making one.
    volume_saved: u8,
    /// When the level that is not on the card yet is to be written. `None` when there is
    /// nothing to write.
    volume_due: Option<Instant>,
    /// The languages this machine could speak when the picker was last opened: the built-ins and
    /// the card's own packs that loaded, in `I18n::available` order. Rebuilt on every opening —
    /// a pack put on the card by hand shows up without a restart — and read-only between
    /// openings.
    language_picker: LanguagePicker,
    /// The language the frontend is actually running, as the context it draws with reports it.
    /// The backend sets it once at startup and once per successful change; nothing here works it
    /// out from the card or from `SLOT2_LANG`, because a request is not what ran.
    current_language: String,
    /// The language the player chose and no context has been built for yet. While this is set the
    /// language screen takes no more input: the backend applies it before the next frame, either
    /// by replacing the context or by saying why it could not.
    language_request: Option<String>,
}

impl App {
    /// The M0 shape: a splash over an empty card. Kept so the boot path and the older
    /// tests still work.
    pub fn new(debug_frame: bool) -> Self {
        App::with_card(
            Card::new("."),
            PathBuf::from("."),
            48_000,
            slot2_retro::Tuning::handheld((720, 480)),
            debug_frame,
            Screen::Splash,
        )
    }

    /// The real thing: scan `card` and start on the list.
    pub fn with_card(
        card: Card,
        core_dir: PathBuf,
        sink_rate: u32,
        tuning: slot2_retro::Tuning,
        debug_frame: bool,
        screen: Screen,
    ) -> Self {
        // The level the machine last ran at, read before the struct that holds it exists. The
        // store owns the file format and the fallback: a missing, unreadable or invalid level
        // arrives here as `DEFAULT_VOLUME_LEVEL`, and this is not a second opinion on it.
        let settings = card.read_global_settings();
        // The card's display offset (D-25), read once here and applied to the process-global
        // clock the HUD and every later `now_local()` read consult — the struct does not hold a
        // copy, so the clock stays the one source of truth and no file is read again per frame.
        // The store owns the format and the fallback: a missing file or key, unreadable bytes
        // and an out-of-range value all arrive here as UTC 0, and this is not a second opinion.
        // The card outranks whatever a previous app or the environment left in the clock: an
        // unset card means UTC, not a stale offset from earlier in the process.
        if let Err(rejected) = clock::set_utc_offset_min(card.read_utc_offset_minutes()) {
            // Unreachable while the compile-time seal above holds, since the store returns only
            // values the clock takes. If it is ever reached the two ranges have drifted in a
            // way that seal did not catch, so say which value was refused and leave the clock
            // on UTC rather than on the previous app's offset.
            eprintln!("slot2: card utc offset {rejected} is outside the clock's range; using UTC");
            let _ = clock::set_utc_offset_min(slot2_store::DEFAULT_UTC_OFFSET_MINUTES);
        }
        let mut app = App {
            screen,
            state: State::default(),
            gestures: Gestures::new(GestureConfig::default()),
            splash: Splash { debug_frame },
            shelf_view: ShelfView::default(),
            wallpaper: slot2_ui::wallpaper::Wallpaper::default(),
            // Muted is not a stored state: a machine that came back up silent with nothing on
            // screen to explain why would be worse than one that came back audible.
            volume: slot2_audio::Volume::new(settings.volume),
            card,
            core_dir,
            sink_rate,
            tuning,
            platform_index: Platform::ALL
                .iter()
                .position(|p| *p == Platform::Gba)
                .unwrap_or(0),
            carts: Vec::new(),
            session: None,
            // Nothing resolved and nothing uploaded until a game is launched.
            overlay: OverlayLayer::default(),
            pending_consumer: None,
            sink_request: None,
            exit: None,
            last_tick: None,
            anim: 0.0,
            refusal: None,
            toast: None,
            sfx_fired: false,
            gauge: Gauge::none(),
            battery: None,
            battery_read: None,
            hud: slot2_ui::Hud::default(),
            state_switcher: StateSwitcher::new(Vec::new()),
            // Empty until the Core row is opened: an app that has never shown the picker has
            // no platform to build candidates for.
            core_picker: CorePicker::new(slot2_retro::Platform::Gba, &[], None),
            undo: None,
            launch: None,
            suppress_a: false,
            ff_latch: false,
            volume_saved: settings.volume,
            // Nothing is waiting to be written at boot: what is on the card is what is running.
            volume_due: None,
            // Both are replaced the moment the settings menu asks: a picker with no candidates
            // and English, which is what every build can always speak.
            language_picker: LanguagePicker::new(Vec::new(), slot2_i18n::FALLBACK),
            current_language: slot2_i18n::FALLBACK.to_string(),
            language_request: None,
        };
        if screen == Screen::List {
            app.rescan();
        }
        app
    }

    pub fn exit(&self) -> Option<Exit> {
        self.exit
    }

    /// Whether the loop should hold its audio sink paused for the screen on show.
    ///
    /// A menu over the game silences it; closing the menu brings it back. Both loops ask
    /// this rather than matching on `Screen` themselves, so a new menu cannot silence the
    /// game on the device and keep playing on the host.
    pub fn audio_paused(&self) -> bool {
        matches!(
            self.screen,
            Screen::Power(_)
                | Screen::InGame(_)
                | Screen::Display(..)
                | Screen::Shader(..)
                | Screen::Overscan(..)
                | Screen::Overlay(..)
                | Screen::Cheats(..)
                | Screen::Switcher(_)
                | Screen::Core(_)
                | Screen::Device(..)
        )
    }

    /// How far the cart is into the slot: 0.0 standing on the row, 1.0 seated. `None` when
    /// nothing is going in or out.
    pub fn insert_seat(&self) -> Option<f32> {
        match self.screen {
            Screen::Inserting => Some(insert::seat_in(self.anim)),
            Screen::Ejecting => Some(insert::seat_out(self.anim)),
            _ => None,
        }
    }

    /// How far the screen is knocked off centre this frame, and zero when nothing was
    /// refused. A flinch is the whole of the answer to an action that cannot happen.
    pub fn refusal_offset(&self) -> f32 {
        self.refusal.as_ref().map_or(0.0, |r| r.offset())
    }

    /// The message on screen, by its i18n key, and `None` when there is none.
    pub fn toast_key(&self) -> Option<&str> {
        self.toast.as_ref().map(|t| t.key())
    }

    /// Put `clip`, played as this shelf plays it, in front of the sink, whole, now.
    ///
    /// Written in one go rather than fed a frame at a time, because the frame after this one
    /// may be the one that loads a core — a second inside `dlopen` on the device. The sink
    /// has its own thread and plays what is in the ring regardless of what the main thread is
    /// doing, so a clip that is already there survives the stall and a clip being dripped in
    /// would not. The clip is styled once, here, and never re-rendered per frame.
    fn play(&mut self, clip: slot2_audio::Sfx, profile: slot2_ui::skin::SoundProfile) {
        let samples = clip.render_styled(self.sink_rate, profile.speed(), profile.gain());
        if samples.is_empty() {
            return;
        }
        let (mut producer, consumer) = slot2_audio::Ring::new(samples.len() / 2).split();
        producer.write(&samples);
        self.pending_consumer = Some(consumer);
        self.sink_request = Some(SinkRequest::Open);
    }

    pub fn shelf_len(&self) -> usize {
        self.shelf_view.shelf.len()
    }

    pub fn selected(&self) -> usize {
        self.shelf_view.shelf.selected()
    }

    /// True while the row is still sliding.
    pub fn shelf_settling(&self) -> bool {
        self.shelf_view.shelf.settling()
    }

    pub fn platform(&self) -> Platform {
        Platform::ALL[self.platform_index]
    }

    pub fn carts(&self) -> &[Cart] {
        &self.carts
    }

    pub fn session(&self) -> Option<&Session> {
        self.session.as_ref()
    }

    /// The audio consumer of a session that just started, taken once by the loop.
    pub fn take_consumer(&mut self) -> Option<slot2_audio::Consumer> {
        self.pending_consumer.take()
    }

    /// What the loop should do with its sink, taken once.
    pub fn take_sink_request(&mut self) -> Option<SinkRequest> {
        self.sink_request.take()
    }

    pub fn rescan(&mut self) {
        let platform = self.platform();
        self.carts = self.card.scan(platform);
        // The card names a cart by its file; the shelf shows a game. `(USA) (Rev 1)` is a
        // fact about the dump, and it is cleaned off here, once, rather than in the draw
        // loop where it would be rebuilt for every cart on screen sixty times a second.
        for cart in &mut self.carts {
            cart.title = slot2_ui::label::clean_title(&cart.stem);
        }
        self.wallpaper.set_source(self.card.wallpaper(platform));
        self.shelf_view.shelf.set_len(self.carts.len());
        let labels = self.carts.iter().map(|c| self.card.label(c)).collect();
        self.shelf_view.set_labels(labels);
        // Whether each cart has a state to go back to, asked once here rather than once a
        // frame: the shelf hint depends on it, and a stat per cart per frame is exactly what
        // the label cache exists to avoid.
        //
        // Two facts are settled here, in this order, and they are about the same thing. The
        // flat states a card already had were written before a game could be given a core at
        // all, so they belong to the platform's own core — nothing else could have written
        // them — and they are moved into that core's namespace, once. Then the hint asks the
        // namespace of the core this game will actually open, which may be an alternative the
        // player chose. A move that fails leaves the flat files exactly where they are: the
        // two sets are never merged, and which one is newer is never guessed at.
        let mut resume = Vec::with_capacity(self.carts.len());
        for cart in &self.carts {
            let default_ns = crate::session::default_namespace(cart.platform);
            if let Err(e) = self.card.adopt_legacy_states(cart, &default_ns) {
                eprintln!(
                    "slot2: cannot move {}'s older states into {}: {e}; leaving them where they are",
                    cart.stem,
                    default_ns.as_str()
                );
            }
            let choice = crate::session::resolve_core(&self.card, cart, &self.core_dir, true);
            resume.push(
                self.card
                    .scoped_state_path(cart, &choice.namespace, StateKind::Resume)
                    .is_file(),
            );
        }
        self.shelf_view.set_resume_available(resume);
    }

    pub fn feed(&mut self, event: &Event) {
        self.state.feed(event);
        // The launch press is over when the player lets go: from here on A is the core's.
        if let Event::Button {
            button: Button::A,
            pressed: false,
            ..
        } = event
        {
            self.suppress_a = false;
        }
        // The event's own Instant is the clock for what it does: in the loop it is the same
        // value the frame's `tick` gets, and an action that arrives between ticks is judged by
        // when it happened rather than by the tick before it.
        let now = event.at();
        let actions = self.gestures.feed(event);
        for action in actions {
            self.act(action, now);
        }
    }

    /// Hand the app the machine's gauge, and take a reading now.
    ///
    /// Now rather than on the next interval: waiting one out would leave the corner empty
    /// for the first ten seconds of every boot, which is most of the time anyone spends
    /// looking at a shelf they have just turned on.
    pub fn set_gauge(&mut self, gauge: Gauge) {
        self.battery = gauge.read();
        self.gauge = gauge;
        self.battery_read = None;
    }

    pub fn tick(&mut self, now: Instant) {
        if let Some(last) = self.last_tick {
            let dt = (now - last).as_secs_f32();
            // A frame that takes a second — loading a core does — integrated against the
            // row's spring is not slow, it is unstable: explicit Euler diverges once dt
            // passes 2/OMEGA, and the row flies off and never returns. Clamping makes a
            // slow frame cost the animation some lag, which nobody watching a stalled
            // frontend will notice, instead of the row.
            let dt = dt.min(1.0 / 30.0);
            self.shelf_view.shelf.update(dt);
            if let Some(r) = self.refusal.as_mut() {
                r.tick(dt);
            }
            if self.refusal.as_ref().is_some_and(|r| !r.active()) {
                self.refusal = None;
            }

            if let Some(t) = self.toast.as_mut() {
                t.tick(dt);
            }
            if self.toast.as_ref().is_some_and(|t| t.done()) {
                self.toast = None;
            }

            if matches!(self.screen, Screen::Inserting | Screen::Ejecting) {
                self.anim += dt;
                self.advance_insert();
            }
        }
        self.last_tick = Some(now);
        // The gauge is read on its own clock, the tick's `Instant`, never the clamped dt:
        // that dt is clamped so a stalled frame cannot destabilise the row spring, and a
        // poll counted in it would take three hundred frames to reach a ten-second interval.
        let due = self
            .battery_read
            .is_none_or(|last| now.duration_since(last).as_secs_f32() >= BATTERY_POLL_S);
        if due {
            self.battery = self.gauge.read();
            self.battery_read = Some(now);
        }

        // A deadline that has passed is not an undo, and it is dropped before anything this
        // tick does can use it. Dropping the backup is the whole of what expiring means: the
        // deletion becomes permanent by being forgotten, with no filesystem work and no toast.
        if self
            .undo
            .as_ref()
            .is_some_and(|pending| now >= pending.expires)
        {
            self.undo = None;
        }

        let actions = self.gestures.tick(now);
        for action in actions {
            self.act(action, now);
        }

        // After this tick's own actions, never before them: a press that arrived in the same
        // tick is what gets written, rather than the level it replaced being written and the
        // write immediately made due again.
        if self.volume_due.is_some_and(|due| now >= due) {
            self.flush_volume();
        }
    }

    /// Carry the insert or the eject on from wherever `anim` has reached.
    ///
    /// The core is loaded here, on the frame the cart is seated, and the dwell that follows
    /// is what the second it costs is hidden behind: what is on screen while it happens is a
    /// cartridge fully in the slot, which is what a machine loading a cartridge looks like.
    /// Loading when the button was pressed instead would freeze the *first* frame of the
    /// animation, which is the jump cut this replaced with extra steps.
    fn advance_insert(&mut self) {
        // One lookup, and it settles both directions: the shelf the cart is moving on is the
        // shelf whose recordings these are, and there is no second platform table in here.
        let skin = slot2_ui::skin::skin(self.platform());
        if !self.sfx_fired {
            match self.screen {
                Screen::Inserting
                    if self.anim
                        >= insert::SEATED_AT
                            - slot2_audio::Sfx::Insert.lead_at_speed(skin.sfx_in.speed()) =>
                {
                    self.play(slot2_audio::Sfx::Insert, skin.sfx_in);
                    self.sfx_fired = true;
                }
                Screen::Ejecting => {
                    self.play(slot2_audio::Sfx::Eject, skin.sfx_out);
                    self.sfx_fired = true;
                }
                _ => {}
            }
        }

        match self.screen {
            Screen::Inserting if self.session.is_none() => {
                if self.anim < insert::SEATED_AT {
                    return;
                }
                if self.anim < insert::SEATED_AT + 0.02 {
                    self.start_launch();
                }
                if self.session.is_none() {
                    self.screen = Screen::Ejecting;
                    self.anim = 0.0;
                    self.sfx_fired = false;
                }
            }
            // Waits on the session existing rather than on the clock running out. A cold
            // core on the device takes longer than the dwell, and without this the screen
            // freezes once the animation ends — the fault the whole arrangement avoids.
            Screen::Inserting if self.anim >= insert::INSERT_S => {
                self.screen = Screen::Playing;
            }
            Screen::Ejecting if self.anim >= insert::EJECT_S => {
                self.screen = Screen::List;
                // Rescanned on arrival, not on the way out: a row that rearranges behind a
                // cart still on screen is a row the player watches shuffle itself.
                self.rescan();
            }
            _ => {}
        }
    }

    /// How long this frame should take. A running game sets the pace — its core's fps is
    /// what its audio rate is derived from, and pacing to anything else makes the two
    /// disagree. With no game, the shelf redraws at 60 Hz.
    pub fn frame_time(&self) -> Duration {
        // Only once the game is what is on screen. A cart on its way into the slot is a UI
        // animation and runs at the panel's rate, even though the core behind it may already
        // be loaded and declaring 59.7275 fps.
        match self.session.as_ref() {
            Some(s) if self.screen == Screen::Playing => s.frame_time(),
            _ => Duration::from_secs_f64(1.0 / 60.0),
        }
    }

    /// `(frames produced, dropped, queued, capacity)` for the running game's audio.
    pub fn audio_health(&self) -> Option<(u64, u64, usize, usize)> {
        self.session.as_ref().map(|s| s.audio_health())
    }

    /// What the time controls are doing at this moment, for the badge and for `run_frame`.
    ///
    /// The one place the priority lives — L2 rewinds whatever else is held, R2 or the latch
    /// runs fast, nothing at all otherwise — so the picture on screen and the control the core
    /// is actually under cannot drift apart. `None` anywhere but over a live game: a badge on a
    /// shelf, mid-animation, or under a menu would be describing a control that is not there.
    fn time_control(&self) -> Option<TimeControl> {
        if self.screen != Screen::Playing || self.session.is_none() {
            return None;
        }
        let down = self.state.held();
        if down.contains(&Button::L2) {
            return Some(TimeControl::Rewind);
        }
        if down.contains(&Button::R2) || self.ff_latch {
            return Some(TimeControl::FastForward {
                speed: FAST_FORWARD,
            });
        }
        None
    }

    /// Advance the game, if one is running. Called once per frame by the loop.
    pub fn run_frame(&mut self) {
        if self.screen != Screen::Playing {
            return;
        }
        let held = self.held_game_buttons();
        let volume = self.volume;
        // L2 and R2 are the time controls. No platform here maps them to anything — a Mega
        // Drive pad has six face buttons and no triggers at all — so they are free, and
        // holding one is the whole gesture: press to rewind or hurry, let go to play on.
        // The badge reads this same query, so what is drawn is what runs.
        let control = self.time_control();
        let speed = match control {
            Some(TimeControl::FastForward { speed }) => speed,
            _ => 1,
        };
        if let Some(s) = self.session.as_mut() {
            s.set_speed(speed);
            if control == Some(TimeControl::Rewind) {
                // While stepping back the core is not run forward, so nothing is fed to the
                // audio ring; a second of rewind is a second of quiet.
                s.rewind_step();
                return;
            }
            s.run_frame(&held, &volume);
        }
    }

    /// The game buttons currently down, in the core's terms.
    ///
    /// A launch press that is still under the player's thumb is not one of them: the press
    /// started the insert, and a core shown a button going down with no press behind it is a
    /// game that jumps the moment it starts. It is held back until its release arrives.
    fn held_game_buttons(&self) -> Vec<LogicalButton> {
        self.state
            .held()
            .into_iter()
            .filter(|b| !(*b == Button::A && self.suppress_a))
            .filter_map(logical)
            .collect()
    }

    /// A on the shelf: put the cart under the cursor into the slot.
    ///
    /// The cart and the intent are fixed here, not at the seat: the row can move during the
    /// animation — a rescan can even reorder it — and the game that loads has to be the one
    /// the player was pointing at when they pressed.
    fn start_insert(&mut self, intent: Launch) {
        let Some(cart) = self.carts.get(self.shelf_view.shelf.selected()).cloned() else {
            return;
        };
        self.launch = Some(PendingLaunch { cart, intent });
        self.screen = Screen::Inserting;
        self.anim = 0.0;
        self.sfx_fired = false;
    }

    /// Open the launched cart's core, and put the player back where they left off if that is
    /// what the launch asked for. Called at the seat, so the dwell hides the load.
    fn start_launch(&mut self) {
        let selected = self.shelf_view.shelf.selected();
        let (cart, intent) = match self.launch.take() {
            Some(pending) => (pending.cart, pending.intent),
            // No gesture to read: something drove the screen directly. The cart on screen and
            // the resume state are the best guess available.
            None => match self.carts.get(selected).cloned() {
                Some(cart) => (cart, Launch::Resume),
                None => return,
            },
        };
        match Session::start(
            &self.card,
            &cart,
            &self.core_dir,
            self.sink_rate,
            self.tuning,
        ) {
            Ok((mut session, consumer)) => {
                eprintln!("slot2: playing {}", cart.title);
                // Before the session is handed over: nothing may advance a core frame between
                // the resume landing and the first frame the player sees.
                match intent {
                    Launch::Resume => self.resume_into(&mut session, &cart),
                    Launch::Fresh => self.discard_resume(&session, &cart),
                }
                self.session = Some(session);
                self.pending_consumer = Some(consumer);
                self.sink_request = Some(SinkRequest::Open);
                // The overlay is resolved here, at the one boundary that knows both the game
                // and the panel, and not once per frame afterwards.
                self.resolve_overlay(&cart);
            }
            Err(e) => {
                eprintln!("slot2: cannot play {}: {e}", cart.title);
                // No game, no overlay: whatever a previous session left is let go rather than
                // drawn over a refusal.
                self.overlay.clear_sources();
                match e {
                    crate::session::Error::NoCore(_) => {
                        self.toast = Some(slot2_ui::toast::Toast::new("core-missing", Vec::new()));
                    }
                    // A cheat file the card would not hand over is not the cart being broken: the
                    // game itself is fine, and which game it was is the whole of the useful part.
                    // The launch path's one store call is the cheat file — everything else in
                    // `Session::start` fails as `Retro` — so this arm cannot mislabel another
                    // failure as a cheat problem.
                    crate::session::Error::Store(_) => {
                        self.toast = Some(slot2_ui::toast::Toast::new(
                            "cheat-load-failed",
                            vec![("title".to_string(), slot2_i18n::Arg::Str(cart.title))],
                        ));
                    }
                    _ => {
                        self.toast = Some(slot2_ui::toast::Toast::new(
                            "cart-broken",
                            vec![("title".to_string(), slot2_i18n::Arg::Str(cart.title))],
                        ));
                    }
                }
            }
        }
    }

    /// Pick up where the player left off, when the card has something for this core.
    ///
    /// A state that will not load costs a message and nothing else: the fresh session carries
    /// on, and the file is left where it is.
    fn resume_into(&mut self, session: &mut Session, cart: &Cart) {
        if let Err(e) = resume_session(session, &self.card, StateKind::Resume) {
            eprintln!("slot2: cannot resume {}: {e}", cart.title);
            self.toast = Some(slot2_ui::toast::Toast::new(
                "resume-load-failed",
                Vec::new(),
            ));
        }
    }

    /// The new game is the one being played now, so the old playthrough goes with it: leaving
    /// the state behind would let a power loss in the middle of the new run bring it back.
    ///
    /// Only after the session is up, and a failure here is a log line, not the end of the game.
    fn discard_resume(&mut self, session: &Session, cart: &Cart) {
        if let Err(e) =
            self.card
                .scoped_delete_state(cart, session.state_namespace(), StateKind::Resume)
        {
            eprintln!("slot2: cannot drop the old resume state: {e}");
        }
    }

    fn stop_session(&mut self) {
        // A pending undo belongs to the session that was running: with the core gone there is
        // nothing left for a restored state to be a state of. Neither does a latched fast
        // forward: the next game starts at its own speed.
        self.undo = None;
        self.ff_latch = false;
        // The overlay belongs to the game that is going, not to the app: it goes with the
        // session, and the texture goes at the next draw — the only place with a canvas.
        self.overlay.clear_sources();
        if let Some(s) = self.session.take() {
            s.stop(&self.card);
            self.sink_request = Some(SinkRequest::Close);
        }
    }

    /// Point the overlay at the picture a game's own decision resolves to, on this panel.
    ///
    /// The one boundary that turns the card's three states and a panel size into sources, shared
    /// by the launch and by a change made from the Overlay screen. The setting is handed in
    /// already read: the launch reads it off the card, and a commit passes the value the card has
    /// just accepted, so neither path reads settings or the filesystem twice.
    ///
    /// A panel this build has no geometry for and a game that turned overlays off end the same
    /// way — no sources, nothing to draw — and are not told apart here, because nothing
    /// downstream could act on the difference.
    fn set_overlay_for(&mut self, platform: Platform, setting: Option<bool>) {
        let geometry = overlay::geometry_for_panel(self.tuning.geometry);
        match (overlay::overlay_enabled(setting), geometry) {
            (true, Some(geometry)) => {
                self.overlay
                    .set_sources(overlay::resolve(&self.card, platform, geometry));
            }
            (_, _) => self.overlay.clear_sources(),
        }
    }

    /// Point the overlay at this game's own decision, once, at the launch boundary.
    ///
    /// One settings read and one resolution per launch: the setting, the card's file and the
    /// panel are all decided here, and every frame after it draws what this decided. A panel
    /// this build has no geometry for, a game that turned overlays off, and a game that never
    /// said anything all end the same way — no sources, nothing to draw, and a launch that is
    /// not held up by an accessory.
    fn resolve_overlay(&mut self, cart: &Cart) {
        let setting = self.card.read_settings(cart).overlay;
        self.set_overlay_for(cart.platform, setting);
    }

    /// The game's own frame and the overlay over it.
    ///
    /// The order is the whole point of this helper: the session clears and draws the picture,
    /// and the overlay goes down on that picture, before any menu, dim, HUD or message. A screen
    /// that draws these two itself, in another order, is a screen where the overlay is under the
    /// game or over the UI.
    fn draw_game_and_overlay(&mut self, canvas: &mut dyn Canvas, ctx: &mut UiCtx) {
        if let Some(session) = self.session.as_mut() {
            session.upload_video(canvas);
            session.draw(canvas, ctx);
            self.overlay.draw(canvas);
        } else {
            // A game-bearing screen with no game: no picture to draw, and no overlay either —
            // it would be a picture of nothing. The canvas is still the one place a texture this
            // app owns can be let go.
            self.draw_overlay_nothing(canvas);
        }
    }

    /// A frame with no game on it: tell the overlay it has nothing to show, and give it the
    /// canvas so a texture left over from the last game goes now rather than at the drop.
    fn draw_overlay_nothing(&mut self, canvas: &mut dyn Canvas) {
        self.overlay.clear_sources();
        self.overlay.draw(canvas);
    }

    /// Leave, with the level the player set on the card first.
    ///
    /// A power-off a moment after the last volume press is exactly the case the delay cannot
    /// cover, so the write does not wait for it and neither the pending state nor the deadline
    /// matters. Nothing about leaving depends on whether it worked: a card that will not answer
    /// is not a reason to stay on, and no message holds the exit up.
    fn exit_now(&mut self, exit: Exit) {
        self.flush_volume();
        self.stop_session();
        self.exit = Some(exit);
    }

    fn act(&mut self, action: Action, now: Instant) {
        if self.exit.is_some() {
            return;
        }
        match (self.screen, action) {
            (_, Action::Tap(Button::Power)) => self.exit_now(Exit::PowerOff),
            // The physical volume keys work on every screen. The Device menu is a picture of
            // the same volume rather than a second level, so the press is applied here and the
            // menu is re-read from the result: one place changes the volume, and the screen
            // cannot end up saying something the sink is not doing.
            (_, Action::Tap(Button::VolUp)) => {
                self.step_volume(Button::VolUp, now);
                self.refresh_device_menu();
            }
            (_, Action::Tap(Button::VolDown)) => {
                self.step_volume(Button::VolDown, now);
                self.refresh_device_menu();
            }

            (Screen::Playing, Action::Tap(Button::Menu)) => {
                self.screen = Screen::InGame(InGameMenu::default());
            }
            // The hold is the eject, and it is the gesture the player already knows from the
            // shelf: a tap opens the menu, a hold takes the cart out.
            (Screen::Playing, Action::Hold(Button::Menu)) => {
                self.stop_session();
                self.screen = Screen::Ejecting;
                self.anim = 0.0;
                self.sfx_fired = false;
            }

            // R2 is momentary while it is held; a double tap leaves it held for good, until
            // the next one. Only over a running game: a double tap on the shelf must not arm
            // the game that comes next.
            (Screen::Playing, Action::DoubleTap(Button::R2)) if self.session.is_some() => {
                self.ff_latch = !self.ff_latch;
            }

            // SELECT plus a shoulder, while the game is running: R1 writes a state now,
            // L1 puts the newest one back. Nothing here changes the screen, the session or
            // the sink — a quick save the player does not notice is the point.
            (Screen::Playing, Action::Chord(Button::R1)) => self.quick_save(),
            (Screen::Playing, Action::Chord(Button::L1)) => self.quick_load(),

            (Screen::InGame(mut menu), Action::Tap(b)) => match b {
                Button::Up => {
                    menu.up();
                    self.screen = Screen::InGame(menu);
                }
                Button::Down => {
                    menu.down();
                    self.screen = Screen::InGame(menu);
                }
                Button::A => match menu.choice() {
                    InGameChoice::Continue => self.screen = Screen::Playing,
                    InGameChoice::SaveState => self.open_state_switcher(menu),
                    InGameChoice::Cheats => self.open_cheat_menu(menu),
                    InGameChoice::Display => self.open_display_menu(menu),
                    InGameChoice::Core => self.open_core_picker(menu),
                    InGameChoice::Device => self.open_device_menu(menu),
                    // The same shutdown the MENU hold runs, animation flags included: the
                    // cart leaves the slot and the shelf comes back exactly as it does
                    // there.
                    InGameChoice::Eject => {
                        self.stop_session();
                        self.screen = Screen::Ejecting;
                        self.anim = 0.0;
                        self.sfx_fired = false;
                    }
                },
                Button::B | Button::Menu => self.screen = Screen::Playing,
                _ => {}
            },

            (Screen::Display(menu, mut display), Action::Tap(b)) => match b {
                Button::Up => {
                    display.up();
                    self.screen = Screen::Display(menu, display);
                }
                Button::Down => {
                    display.down();
                    self.screen = Screen::Display(menu, display);
                }
                // The menu stays open on the row that was chosen: the game frame behind it is
                // the answer to "what does this look like". The shader row is not a setting: it
                // opens the screen that chooses one.
                Button::A => match display.choice() {
                    DisplayChoice::Scale(_) => self.commit_display_scale(),
                    DisplayChoice::Shader => self.open_shader_menu(menu, display),
                    DisplayChoice::Overscan => self.open_overscan_menu(menu, display),
                    DisplayChoice::Overlay => self.open_overlay_menu(menu, display),
                },
                Button::B | Button::Menu => self.screen = Screen::InGame(menu),
                _ => {}
            },

            (Screen::Shader(menu, display, mut shader), Action::Tap(b)) => match b {
                Button::Up => {
                    shader.up();
                    self.screen = Screen::Shader(menu, display, shader);
                }
                Button::Down => {
                    shader.down();
                    self.screen = Screen::Shader(menu, display, shader);
                }
                // The screen stays open on the row that was chosen: the game frame behind it is
                // the answer to how the picture looks under it now.
                Button::A => self.commit_shader(),
                // One step back, to the Display row this was opened from.
                Button::B | Button::Menu => self.screen = Screen::Display(menu, display),
                // Nothing else reaches the core: the game is paused under this menu, and a
                // button handed to a paused core would be a button pressed in a menu.
                _ => {}
            },

            (Screen::Overscan(menu, display, mut overscan), Action::Tap(b)) => match b {
                Button::Up => {
                    overscan.up();
                    self.screen = Screen::Overscan(menu, display, overscan);
                }
                Button::Down => {
                    overscan.down();
                    self.screen = Screen::Overscan(menu, display, overscan);
                }
                // The screen stays open on the row that was chosen: the frame behind it is the
                // answer to how much of the picture is left, and the change is on it now.
                Button::A => self.commit_overscan(),
                // One step back, to the Display row this was opened from.
                Button::B | Button::Menu => self.screen = Screen::Display(menu, display),
                // Nothing else reaches the core: the game is paused under this menu, and a
                // button handed to a paused core would be a button pressed in a menu.
                _ => {}
            },

            (Screen::Overlay(menu, display, mut overlay), Action::Tap(b)) => match b {
                Button::Up => {
                    overlay.up();
                    self.screen = Screen::Overlay(menu, display, overlay);
                }
                Button::Down => {
                    overlay.down();
                    self.screen = Screen::Overlay(menu, display, overlay);
                }
                // The screen stays open on the row that was chosen: the game frame behind it is
                // the answer to whether the overlay is over it, and the change is on it now.
                Button::A => self.commit_overlay(),
                // One step back, to the Display row this was opened from.
                Button::B | Button::Menu => self.screen = Screen::Display(menu, display),
                // Nothing else reaches the core: the game is paused under this menu, and a
                // button handed to a paused core would be a button pressed in a menu.
                _ => {}
            },

            (Screen::Cheats(menu, mut cheats), Action::Tap(b)) => match b {
                Button::Up => {
                    cheats.up();
                    self.screen = Screen::Cheats(menu, cheats);
                }
                Button::Down => {
                    cheats.down();
                    self.screen = Screen::Cheats(menu, cheats);
                }
                // The menu stays open on the cheat that was flipped: the frame behind it is
                // the answer to what flipping it did.
                Button::A => self.toggle_cheat(),
                Button::B | Button::Menu => self.screen = Screen::InGame(menu),
                // Nothing else reaches the core: the game is paused under this menu, and a
                // button handed to a paused core would be a button pressed in a menu.
                _ => {}
            },

            (Screen::Switcher(menu), Action::Tap(b)) => match b {
                Button::Left => self.state_switcher.left(),
                Button::Right => self.state_switcher.right(),
                Button::X => self.delete_selected_state(now),
                Button::Y => self.undo_delete(now),
                Button::A => match self.state_switcher.selected_kind() {
                    // A load takes the game back: the core has been put where the player
                    // asked for and the menu it came from has done its job.
                    Some(StateKind::Numbered(n)) => {
                        if self.load_numbered_state(n) {
                            self.screen = Screen::Playing;
                        }
                    }
                    // No numbered state to load. The empty view stays where it is: resuming
                    // the game, or writing a state, is not what A means here.
                    _ => {
                        self.toast = Some(slot2_ui::toast::Toast::new("states-empty", Vec::new()));
                    }
                },
                Button::B | Button::Menu => self.screen = Screen::InGame(menu),
                // Up, Down, X, Y and the rest are not the switcher's business.
                _ => {}
            },

            (Screen::Core(menu), Action::Tap(b)) => match b {
                Button::Up => self.core_picker.up(),
                Button::Down => self.core_picker.down(),
                Button::A => self.choose_core(menu),
                Button::B | Button::Menu => self.screen = Screen::InGame(menu),
                _ => {}
            },

            (Screen::Device(menu, mut device), Action::Tap(b)) => match b {
                Button::Up => {
                    device.up();
                    self.screen = Screen::Device(menu, device);
                }
                Button::Down => {
                    device.down();
                    self.screen = Screen::Device(menu, device);
                }
                Button::Left | Button::Right | Button::A => {
                    self.change_volume(menu, device, b, now);
                }
                Button::B | Button::Menu => self.screen = Screen::InGame(menu),
                // Nothing else reaches the core: the game is paused under this menu, and a
                // button handed to a paused core would be a button pressed in a menu.
                _ => {}
            },

            // Not on `Inserting` or `Ejecting`. A menu hold takes about as long as the whole
            // animation, so there is barely a moment to catch — and a power menu over a cart
            // frozen in mid-air, which is what pausing an animation to open one looks like,
            // is worse than nothing happening.
            (Screen::List | Screen::Splash, Action::Hold(Button::Menu)) => {
                self.screen = Screen::Power(PowerMenu::default());
            }
            (Screen::List, Action::Tap(b)) => match b {
                Button::Left => self.shelf_view.shelf.left(),
                Button::Right => self.shelf_view.shelf.right(),
                Button::L1 => self.switch_platform(-1),
                Button::R1 => self.switch_platform(1),
                // The tap opens the settings menu; the hold, above, is still the power menu.
                // A tap is what the gesture layer saw and a hold is not a tap, so the two
                // cannot produce each other.
                Button::Menu => self.open_shelf_menu(),
                Button::A if !self.carts.is_empty() => self.start_insert(Launch::Resume),
                Button::A => {
                    self.refusal = Some(slot2_ui::refusal::Refusal::new());
                }
                _ => {}
            },
            // The other half of A: a hold means a new game, deliberately, and the press that
            // asked for it stays out of the core's hands until it is let go.
            (Screen::List, Action::Hold(Button::A)) if !self.carts.is_empty() => {
                self.start_insert(Launch::Fresh);
                self.suppress_a = true;
            }

            (Screen::Shelf(mut menu), Action::Tap(b)) => match b {
                Button::Up => {
                    menu.up();
                    self.screen = Screen::Shelf(menu);
                }
                Button::Down => {
                    menu.down();
                    self.screen = Screen::Shelf(menu);
                }
                // The time zone and About rows have screens behind them in this build, and the
                // menu knows which rows those are: a row that cannot be opened is never
                // selected, so there is nothing here to refuse and no message to invent.
                Button::A => match menu.selected() {
                    Some(ShelfChoice::Language) => self.open_language_picker(menu),
                    Some(ShelfChoice::TimeZone) => self.open_timezone_menu(menu),
                    Some(ShelfChoice::About) => self.open_about(menu),
                    // The three rows with no screen behind them are never selected, so this is
                    // the menu whose entries cannot be opened rather than a refusal the player
                    // could see: an unavailable row does nothing and says nothing.
                    _ => {}
                },
                Button::B | Button::Menu => self.screen = Screen::List,
                _ => {}
            },

            // Leaving is the only thing a button does on the sticker — it has no controls, no
            // links and nothing to choose — so every other button and every other action falls
            // through the match's own catch-all rather than landing on an empty case here.
            (Screen::About(parent, _), Action::Tap(Button::B | Button::Menu)) => {
                self.screen = Screen::Shelf(parent);
            }

            // A language change in flight: the backend applies it before the next frame, so a
            // button pressed in between is a choice about a screen that is already leaving. The
            // press is dropped rather than queued behind it.
            (Screen::Language(_), Action::Tap(_)) if self.language_request.is_some() => {}

            (Screen::Language(parent), Action::Tap(b)) => match b {
                Button::Up => {
                    self.language_picker.up();
                    self.screen = Screen::Language(parent);
                }
                Button::Down => {
                    self.language_picker.down();
                    self.screen = Screen::Language(parent);
                }
                Button::A => self.choose_language(parent),
                // Nothing is written and nothing is requested: the choice was never more than
                // where the highlight was.
                Button::B | Button::Menu => self.screen = Screen::Shelf(parent),
                _ => {}
            },

            (Screen::Timezone(parent, menu), Action::Tap(b)) => match b {
                Button::Left | Button::Right | Button::Up | Button::Down => {
                    self.move_timezone(parent, menu, b);
                }
                Button::A => self.apply_timezone(parent, menu),
                Button::B | Button::Menu => self.cancel_timezone(parent, menu.original()),
                // Nothing else reaches the shelf: the rows are not a game, and a button handed
                // to a running core would be a button pressed in a menu.
                _ => {}
            },

            (Screen::Power(mut menu), Action::Tap(b)) => match b {
                Button::Up => {
                    menu.up();
                    self.screen = Screen::Power(menu);
                }
                Button::Down => {
                    menu.down();
                    self.screen = Screen::Power(menu);
                }
                Button::A => match menu.choice() {
                    slot2_ui::PowerChoice::Resume => self.screen = self.resume_screen(),
                    slot2_ui::PowerChoice::Restart => self.exit_now(Exit::Reboot),
                    slot2_ui::PowerChoice::PowerOff => self.exit_now(Exit::PowerOff),
                },
                Button::B | Button::Menu => self.screen = self.resume_screen(),
                _ => {}
            },
            _ => {}
        }
    }

    /// What the shelf's settings menu can open in this build: the language picker, the time zone
    /// screen and the About sticker. The three rows with no screen behind them yet are on the
    /// menu and cannot be chosen; the one place this is written down is here, so nothing else has
    /// to be kept in step as their screens land.
    fn shelf_availability() -> ShelfAvailability {
        ShelfAvailability {
            language: true,
            time_zone: true,
            about: true,
            ..Default::default()
        }
    }

    /// Tap(Menu) on the shelf: the settings menu, over it.
    ///
    /// Every entry M4 settles on is on the menu, and what this build can open is
    /// [`App::shelf_availability`]; the rest are shown and cannot be chosen.
    fn open_shelf_menu(&mut self) {
        self.screen = Screen::Shelf(ShelfMenu::new(Self::shelf_availability()));
    }

    /// A on the settings menu's About row: the build sticker, over the shelf.
    fn open_about(&mut self, parent: ShelfMenu) {
        self.screen = Screen::About(parent, AboutSticker::new());
    }

    /// A on the settings menu's Language row: the languages this machine can speak, over the
    /// shelf.
    ///
    /// The list is discovered here, once per opening, and only the packs that actually load are
    /// offered: a card pack that will not parse is not a language anybody can choose, and a row
    /// that could not be switched to would be a promise the frontend cannot keep. The frontend's
    /// own English is the one exception — it is compiled into every build, so a card's broken
    /// copy of it is not a reason to lose it from the list.
    ///
    /// The picker's badge is the language the running context reports, never the code the card
    /// asks for: a request that failed to load is not what is on screen.
    fn open_language_picker(&mut self, parent: ShelfMenu) {
        let dir = self.card.root().join("System").join("Lang");
        let mut options = Vec::new();
        for code in slot2_i18n::I18n::available(Some(&dir)) {
            match slot2_i18n::I18n::load(&code, Some(&dir)) {
                Ok(pack) => options.push(LanguageOption::new(code, pack.name())),
                Err(e) => {
                    eprintln!("slot2: language {code:?} would not load: {e}");
                    if code == slot2_i18n::FALLBACK {
                        match slot2_i18n::I18n::embedded(slot2_i18n::FALLBACK) {
                            Ok(pack) => options.push(LanguageOption::new(code, pack.name())),
                            Err(e) => eprintln!("slot2: no built-in {code:?} pack either: {e}"),
                        }
                    }
                }
            }
        }
        self.language_picker = LanguagePicker::new(options, &self.current_language);
        self.screen = Screen::Language(parent);
    }

    /// A on the language picker: choose the highlighted language, or leave the card alone when
    /// the player picked the one already running.
    ///
    /// Only the request is queued here. Building the new context and saving the choice is the
    /// backend's work, done before the next frame is drawn, because a context belongs to the loop
    /// that draws with it: a language half-changed — a card that says one thing and a screen that
    /// speaks another — is worse than neither half.
    fn choose_language(&mut self, parent: ShelfMenu) {
        if self.language_picker.highlighted().is_none() {
            // Nothing to choose: an empty list leaves the screen where it is, with nothing
            // written and nothing requested.
            return;
        }
        if !self.language_picker.changed() {
            self.screen = Screen::Shelf(parent);
            return;
        }
        if let Some(choice) = self.language_picker.highlighted() {
            self.language_request = Some(choice.code().to_owned());
        }
        self.screen = Screen::Language(parent);
    }

    /// The languages this machine could speak when the picker was last opened.
    pub fn language_picker(&self) -> &LanguagePicker {
        &self.language_picker
    }

    /// The language the frontend is actually running.
    pub fn current_language(&self) -> &str {
        &self.current_language
    }

    /// The backend telling the app which language its context really loaded. A pack name or a
    /// saved request is not this answer, so it is given rather than worked out.
    pub fn set_current_language(&mut self, code: &str) {
        self.current_language = code.to_owned();
    }

    /// The language the player chose that no context has been built for yet.
    pub fn pending_language(&self) -> Option<&str> {
        self.language_request.as_deref()
    }

    /// The new context really speaks the chosen language: save it, and only then make it the one
    /// the frontend is running.
    ///
    /// Called between building a new context and handing it over, so the card is written while
    /// the old context is still the one on screen: a write that cannot land leaves everything as
    /// it was — the old context, the old language, and the screen the player is on — and says so.
    pub(crate) fn commit_language(&mut self, code: &str) -> bool {
        if let Err(e) = self.card.write_language(code) {
            eprintln!("slot2: cannot save the language {code:?}: {e}");
            self.language_request = None;
            self.toast = Some(slot2_ui::toast::Toast::new(
                "language-save-failed",
                Vec::new(),
            ));
            return false;
        }
        self.language_request = None;
        self.current_language = code.to_owned();
        if let Screen::Language(parent) = self.screen {
            self.screen = Screen::Shelf(parent);
        }
        true
    }

    /// The new context could not speak the chosen language: nothing is written and nothing is
    /// replaced, and the picker stays where it was with its badge on the language that is still
    /// running.
    pub(crate) fn language_load_failed(&mut self) {
        self.language_request = None;
        self.toast = Some(slot2_ui::toast::Toast::new(
            "language-load-failed",
            Vec::new(),
        ));
    }

    /// A on the settings menu's Time zone row: the offset screen, open on the value the clock is
    /// running at now.
    ///
    /// The offset is read here rather than kept in `App`: the clock is the one source of truth
    /// for what the machine is running at, and a second copy would be a second thing to keep in
    /// step with it.
    fn open_timezone_menu(&mut self, parent: ShelfMenu) {
        let menu = TimezoneMenu::new(clock::utc_offset_min());
        self.screen = Screen::Timezone(parent, menu);
    }

    /// A direction on the time zone screen: move the value and put it on the running clock, so
    /// the HUD behind the menu previews it from the same frame on. Nothing is written — the card
    /// is not this screen's business until A.
    fn move_timezone(&mut self, parent: ShelfMenu, mut menu: TimezoneMenu, button: Button) {
        match button {
            Button::Left => menu.left(),
            Button::Right => menu.right(),
            Button::Up => menu.up(),
            Button::Down => menu.down(),
            _ => return,
        }
        // The clock's range, the menu's range and the card's are one range, so this cannot be
        // refused — but a refusal would be a value on screen the machine is not running at, so
        // it is handled rather than unwrapped: the offset goes back to what the card holds and
        // the screen says so instead of showing a clock that disagrees with it.
        if let Err(rejected) = clock::set_utc_offset_min(menu.selected()) {
            eprintln!("slot2: the clock refused the time zone offset {rejected} while previewing");
            self.abandon_timezone(parent, menu.original());
            return;
        }
        self.screen = Screen::Timezone(parent, menu);
    }

    /// A on the time zone screen: put the value on the card, or leave the card alone when the
    /// player changed nothing.
    ///
    /// A write that cannot land puts everything back the way it was and leaves the screen open:
    /// the value the player chose is on the clock, on screen and on the card, or it is on none of
    /// them. A value the card already holds is not written again, for the same reason the volume
    /// is not: a write that stores what is there is a filesystem call with nothing to show for it.
    fn apply_timezone(&mut self, parent: ShelfMenu, menu: TimezoneMenu) {
        if !menu.changed() {
            self.screen = Screen::Shelf(parent);
            return;
        }
        let selected = menu.selected();
        if let Err(e) = self.card.write_utc_offset_minutes(selected) {
            eprintln!("slot2: cannot save the time zone offset {selected}: {e}");
            self.abandon_timezone(parent, menu.original());
            return;
        }
        self.screen = Screen::Shelf(parent);
    }

    /// B or Menu on the time zone screen: the clock goes back to the value the card holds, and
    /// the settings menu comes back over the shelf.
    ///
    /// Nothing is read and nothing is written: the preview never consulted the card and never
    /// touched it, so there is nothing there to undo.
    fn cancel_timezone(&mut self, parent: ShelfMenu, original: i32) {
        if let Err(rejected) = clock::set_utc_offset_min(original) {
            // `original` came from the clock itself, so the range contract makes this
            // unreachable; a clock that refused its own value is worth a line all the same.
            eprintln!("slot2: the clock refused to go back to offset {rejected}");
        }
        self.screen = Screen::Shelf(parent);
    }

    /// The value on screen could not be kept: the clock goes back to `original`, the menu opens
    /// on that value again, and the screen stays on the offset so it can be tried again.
    ///
    /// One message for both refusals — the previewing one and the saving one — because the
    /// player has one question, which is what the clock is doing now, and the answer is the same
    /// either way: the value the card holds.
    fn abandon_timezone(&mut self, parent: ShelfMenu, original: i32) {
        if let Err(rejected) = clock::set_utc_offset_min(original) {
            eprintln!("slot2: the clock refused to go back to offset {rejected} while restoring");
        }
        self.screen = Screen::Timezone(parent, TimezoneMenu::new(original));
        self.toast = Some(slot2_ui::toast::Toast::new(
            "timezone-save-failed",
            Vec::new(),
        ));
    }

    /// A on the in-game menu's Cheats row: the cheats menu for the session's own list.
    ///
    /// No session, no list, and the row stays a row: the cheats belong to the game that is
    /// running, and there is none. The menu is built for the length the session has right now
    /// and walks that on its own; draw reads the session's slice itself, so a list that ever
    /// disagreed would be shown as far as it agrees rather than read past its end.
    fn open_cheat_menu(&mut self, menu: InGameMenu) {
        let Some(len) = self.session.as_ref().map(|s| s.cheats().len()) else {
            return;
        };
        self.screen = Screen::Cheats(menu, CheatMenu::new(len));
    }

    /// A in the cheats menu: turn the highlighted cheat over for the session, now.
    ///
    /// One call to the session, which resets the core and puts the whole list back on it: what
    /// the core holds has to end up as what the session says, and the entry that changed is not
    /// the only thing it may be carrying. Nothing is written to the card — the file's own
    /// `enable` values belong to the desktop tool (D-21) — and a failure is a log line and a
    /// message, never a session or a screen left half-changed.
    fn toggle_cheat(&mut self) {
        let Screen::Cheats(_, menu) = self.screen else {
            return;
        };
        let Some(index) = menu.selected_index() else {
            // The empty list: there is nothing to turn over, and nothing to say about it.
            return;
        };

        let (title, failed) = {
            let Some(session) = self.session.as_mut() else {
                return;
            };
            let title = session.cart().title.clone();
            // The menu's list and the session's list are the same list while the menu is open.
            // If they are not, this check is what keeps the menu from flipping an entry it never
            // pointed at — and saying so beats flipping whatever sits at that index.
            let failed = match session.cheats().get(index) {
                Some(cheat) => {
                    let want = !cheat.enabled;
                    session
                        .set_cheat_enabled(index, want)
                        .err()
                        .map(|e| e.to_string())
                }
                None => Some(format!(
                    "index {index} is not one of this game's {} cheats",
                    session.cheats().len()
                )),
            };
            (title, failed)
        };
        let Some(why) = failed else {
            return;
        };

        eprintln!("slot2: cannot change cheat {index} for {title}: {why}");
        self.toast = Some(slot2_ui::toast::Toast::new(
            "cheat-toggle-failed",
            vec![("title".to_string(), slot2_i18n::Arg::Str(title))],
        ));
    }

    /// A on the in-game menu's Display row: the scale menu, open on what this game is on now.
    ///
    /// No session, no cart, and the row stays a row: the setting belongs to the game that is
    /// running, and there is none.
    ///
    /// Whether the menu offers the overscan row is the registry's answer for the cart's platform
    /// — a platform whose entry crops nothing gets no row, because every choice behind it would
    /// draw the same picture. Asked here rather than in the UI, which has no business knowing
    /// what a NES is.
    fn open_display_menu(&mut self, menu: InGameMenu) {
        let Some(cart) = self.session.as_ref().map(|s| s.cart().clone()) else {
            return;
        };
        let settings = self.card.read_settings(&cart);
        let platform = crate::session::retro_platform(cart.platform);
        let crops = slot2_retro::def(platform).overscan != slot2_retro::Overscan::NONE;
        self.screen = Screen::Display(menu, DisplayMenu::new(settings.scale, crops));
    }

    /// A in the Display submenu on a scale row: save the highlighted scale for this game, and put
    /// it on the running game at once — the menu stays open so the frame behind it shows the
    /// choice.
    ///
    /// The card is written first. A setting the card refused is not applied: the player would
    /// otherwise be looking at a change that disappears the next time the game starts.
    fn commit_display_scale(&mut self) {
        let Screen::Display(_, display) = self.screen else {
            return;
        };
        let Some(cart) = self.session.as_ref().map(|s| s.cart().clone()) else {
            return;
        };
        // The shader row has no scale to save. A is only routed here from a scale row, and a
        // caller that got here anyway must not write a scale it does not have.
        let DisplayChoice::Scale(scale) = display.choice() else {
            return;
        };

        // The whole object, so the keys this menu does not own — core, overscan, rewind and
        // anything a newer version wrote — survive the round trip.
        let mut settings = self.card.read_settings(&cart);
        settings.scale = scale;
        if let Err(e) = self.card.write_settings(&cart, &settings) {
            eprintln!(
                "slot2: cannot save the display setting for {}: {e}",
                cart.title
            );
            self.toast = Some(slot2_ui::toast::Toast::new(
                "display-save-failed",
                Vec::new(),
            ));
            return;
        }
        if let Some(session) = self.session.as_mut() {
            session.set_scale(Session::policy_for(scale));
        }
    }

    /// A on the Display submenu's shader row: the shader submenu, open on what this game is on.
    ///
    /// No session, no cart, and the row stays a row — and nothing is read from the card either,
    /// because the setting belongs to the game that is running. The `DisplayMenu` travels with
    /// it so closing the shader screen lands back on this row rather than at the top.
    fn open_shader_menu(&mut self, menu: InGameMenu, display: DisplayMenu) {
        let Some(cart) = self.session.as_ref().map(|s| s.cart().clone()) else {
            return;
        };
        let shader = self.card.read_settings(&cart).shader;
        self.screen = Screen::Shader(menu, display, ShaderMenu::new(shader));
    }

    /// A in the shader submenu: save the highlighted choice for this game, and put it on the
    /// running game at once.
    ///
    /// The card is written first, and for a stronger reason than the scale menu has: the effect
    /// is what the player is looking at, so applying one the card refused would be a picture
    /// that disappears the next time the game starts. The card holds the game's own meaning —
    /// the absence of the key is the platform's default, `Off` is a decision — and the one
    /// boundary that turns either into a renderer effect is the session's, never a second copy
    /// of the mapping here.
    fn commit_shader(&mut self) {
        let Screen::Shader(_, _, shader) = self.screen else {
            return;
        };
        let Some(cart) = self.session.as_ref().map(|s| s.cart().clone()) else {
            return;
        };

        // The whole object, so every key this menu does not own — core, scale, overscan, rewind
        // and anything a newer version wrote — survives the round trip. A shader-only file is
        // removed by the store itself when the choice is the absence of a setting.
        let mut settings = self.card.read_settings(&cart);
        settings.shader = shader.selected();
        if let Err(e) = self.card.write_settings(&cart, &settings) {
            eprintln!(
                "slot2: cannot save the shader setting for {}: {e}",
                cart.title
            );
            self.toast = Some(slot2_ui::toast::Toast::new(
                "shader-save-failed",
                Vec::new(),
            ));
            return;
        }
        let platform = crate::session::retro_platform(cart.platform);
        let effect = Session::shader_effect_for(platform, shader.selected());
        if let Some(session) = self.session.as_mut() {
            session.set_shader_effect(effect);
        }
    }

    /// A on the Display submenu's overscan row: the overscan submenu, open on what this game is
    /// on.
    ///
    /// No session, no cart, and the row stays a row — and nothing is read from the card either,
    /// because the setting belongs to the game that is running. The `DisplayMenu` travels with it
    /// so closing the overscan screen lands back on this row rather than at the top.
    fn open_overscan_menu(&mut self, menu: InGameMenu, display: DisplayMenu) {
        let Some(cart) = self.session.as_ref().map(|s| s.cart().clone()) else {
            return;
        };
        let overscan = self.card.read_settings(&cart).overscan;
        self.screen = Screen::Overscan(menu, display, OverscanMenu::new(overscan));
    }

    /// A in the overscan submenu: save the highlighted choice for this game, and put it on the
    /// running game at once.
    ///
    /// The card is written first, for the same reason the shader screen writes first: the crop is
    /// what the player is looking at, so applying one the card refused would be a picture that
    /// disappears the next time the game starts. The card holds the game's own meaning — the
    /// absence of the key inherits the platform's default, `Some(true)` is that same crop asked
    /// for by name and `Some(false)` is the whole frame — and the one boundary that turns any of
    /// them into a crop is the session's, never a second copy of the mapping here.
    fn commit_overscan(&mut self) {
        let Screen::Overscan(_, _, menu) = self.screen else {
            return;
        };
        let Some(cart) = self.session.as_ref().map(|s| s.cart().clone()) else {
            return;
        };

        // The whole object, so every key this menu does not own — core, scale, rewind, shader and
        // anything a newer version wrote — survives the round trip. An overscan-only file is
        // removed by the store itself when the choice is the absence of a setting.
        let mut settings = self.card.read_settings(&cart);
        settings.overscan = menu.selected();
        if let Err(e) = self.card.write_settings(&cart, &settings) {
            eprintln!(
                "slot2: cannot save the overscan setting for {}: {e}",
                cart.title
            );
            self.toast = Some(slot2_ui::toast::Toast::new(
                "overscan-save-failed",
                Vec::new(),
            ));
            return;
        }
        if let Some(session) = self.session.as_mut() {
            session.set_overscan_setting(menu.selected());
        }
    }

    /// A on the Display submenu's overlay row: the overlay submenu, open on what this game is on.
    ///
    /// No session, no cart, and the row stays a row — and nothing is read from the card either,
    /// because the setting belongs to the game that is running. The `DisplayMenu` travels with it
    /// so closing the overlay screen lands back on this row rather than at the top.
    fn open_overlay_menu(&mut self, menu: InGameMenu, display: DisplayMenu) {
        let Some(cart) = self.session.as_ref().map(|s| s.cart().clone()) else {
            return;
        };
        let overlay = self.card.read_settings(&cart).overlay;
        self.screen = Screen::Overlay(menu, display, OverlayMenu::new(overlay));
    }

    /// A in the overlay submenu: save the highlighted choice for this game, and put it on the
    /// running game at once.
    ///
    /// The card is written first, for the same reason the shader and overscan screens write
    /// first: the picture is what the player is looking at, so applying one the card refused
    /// would be something that disappears the next time the game starts. The card holds the
    /// game's own meaning — the absence of the key inherits the platform's default, `Some(true)`
    /// asks for the overlay and `Some(false)` turns it off — and the one boundary that turns any
    /// of them into sources is `set_overlay_for`, never a second copy of the mapping here.
    ///
    /// A save that succeeded is the end of it, whatever the picture turns out to be: a missing or
    /// broken PNG costs the picture on screen and nothing else. It is not a refused write, so it
    /// is not a toast, and the row the player chose stays where it is.
    fn commit_overlay(&mut self) {
        let Screen::Overlay(_, _, menu) = self.screen else {
            return;
        };
        let Some(cart) = self.session.as_ref().map(|s| s.cart().clone()) else {
            return;
        };

        // The whole object, so every key this menu does not own — core, scale, overscan, rewind,
        // shader and anything a newer version wrote — survives the round trip. An overlay-only
        // file is removed by the store itself when the choice is the absence of a setting.
        let mut settings = self.card.read_settings(&cart);
        settings.overlay = menu.selected();
        if let Err(e) = self.card.write_settings(&cart, &settings) {
            eprintln!(
                "slot2: cannot save the overlay setting for {}: {e}",
                cart.title
            );
            self.toast = Some(slot2_ui::toast::Toast::new(
                "overlay-save-failed",
                Vec::new(),
            ));
            return;
        }
        self.set_overlay_for(cart.platform, menu.selected());
    }

    /// A on the in-game menu's Core row: the cores this game could be run on.
    ///
    /// The core directory is read here, once, and the answer is kept in the picker: draw runs
    /// sixty times a second, and a stat per candidate per frame is the mistake the shelf's
    /// label cache exists to avoid. The platform's own candidate list comes from the registry,
    /// never from a second list written here.
    ///
    /// A platform with one candidate has nothing to choose between, so the row stays a row and
    /// says so. A platform with two opens the picker even when neither library is on the card:
    /// an empty picker with an honest message is worth more than a row that silently does
    /// nothing.
    fn open_core_picker(&mut self, menu: InGameMenu) {
        let Some(cart) = self.session.as_ref().map(|s| s.cart().clone()) else {
            return;
        };
        let platform = crate::session::retro_platform(cart.platform);
        let supported = slot2_retro::supported_cores(platform);
        if supported.len() < 2 {
            self.toast = Some(slot2_ui::toast::Toast::new(
                "core-no-alternatives",
                Vec::new(),
            ));
            return;
        }
        let installed: Vec<slot2_retro::CoreId> = supported
            .iter()
            .copied()
            .filter(|core| self.core_dir.join(core.file_name()).is_file())
            .collect();
        let current = self.session.as_ref().and_then(|s| s.core_id());
        self.core_picker = CorePicker::new(platform, &installed, current);
        self.screen = Screen::Core(menu);
    }

    /// A in the core picker: run this game on the core the highlight is on.
    ///
    /// The same core as the one running is not a change, and costs nothing: no settings write,
    /// no state write, no core loaded, no sink touched.
    fn choose_core(&mut self, menu: InGameMenu) {
        let Screen::Core(_) = self.screen else {
            return;
        };
        let Some(chosen) = self.core_picker.highlighted() else {
            // Nothing installed for this platform: the picker is the honest answer and A has
            // nothing to add to it.
            self.toast = Some(slot2_ui::toast::Toast::new("core-picker-empty", Vec::new()));
            return;
        };
        if self.session.as_ref().and_then(|s| s.core_id()) == Some(chosen) {
            self.screen = Screen::InGame(menu);
            return;
        }
        self.switch_core(menu, chosen);
    }

    /// Run the game on `chosen`, in the only order that survives a failure at any step.
    ///
    /// 1. The running session is checkpointed while it is still alive: save RAM out, and this
    ///    core's own Resume written. Both are attempted, and either failing means the switch does
    ///    not start — the playthrough is the thing being protected, and a change of core is not
    ///    worth it.
    /// 2. Only then does the old core go. The settings still say what it was, so every failure
    ///    from here on can be undone by opening that again from the unchanged file.
    /// 3. The chosen core is opened by name rather than through the settings: a fallback here
    ///    would leave the card and the running game disagreeing about what is being played.
    /// 4. The choice is written only once that core is really up. Writing it first would mean a
    ///    failed start leaves the card pointing at a game that is not running.
    /// 5. The new core's own Resume is the only state looked at, and only its own.
    fn switch_core(&mut self, menu: InGameMenu, chosen: slot2_retro::CoreId) {
        let Some(cart) = self.session.as_ref().map(|s| s.cart().clone()) else {
            return;
        };

        let Some(session) = self.session.as_mut() else {
            return;
        };
        // Both halves of the checkpoint are attempted whatever the other one did: a save RAM
        // flush that failed must not stop the Resume from being written, and the log has to say
        // which of the two went wrong rather than only that something did. Either failing keeps
        // the running session: the playthrough is the thing being protected, and a change of
        // core is not worth losing it over.
        let mut failed: Vec<&str> = Vec::new();
        if let Err(e) = session.flush_save(&self.card) {
            eprintln!(
                "slot2: cannot flush {}'s save RAM before a core change: {e}",
                cart.title
            );
            failed.push("save RAM");
        }
        if let Err(e) = session.save_state(&self.card, StateKind::Resume) {
            eprintln!(
                "slot2: cannot write {}'s resume before a core change: {e}",
                cart.title
            );
            failed.push("resume");
        }
        if !failed.is_empty() {
            eprintln!(
                "slot2: {}'s {} could not be saved; keeping the running core",
                cart.title,
                failed.join(" and ")
            );
            self.toast = Some(slot2_ui::toast::Toast::new(
                "core-checkpoint-failed",
                Vec::new(),
            ));
            return;
        }

        // The old playthrough is on the card now, so the core can be let go. `stop` is the one
        // shutdown path the MENU hold uses; it writes the same two things again, which is the
        // price of there being one exit rather than two.
        if let Some(old) = self.session.take() {
            old.stop(&self.card);
        }

        match Session::start_named(
            &self.card,
            &cart,
            &self.core_dir,
            chosen,
            self.sink_rate,
            self.tuning,
        ) {
            Ok((session, consumer)) => {
                self.finish_switch(menu, &cart, chosen, session, consumer);
            }
            Err(e) => {
                eprintln!(
                    "slot2: cannot start {} for {}: {e}",
                    chosen.base_name(),
                    cart.title
                );
                self.recover_from_switch(menu, &cart, "core-switch-failed");
            }
        }
    }

    /// The last two steps: save the choice, then take the new core's own Resume.
    fn finish_switch(
        &mut self,
        menu: InGameMenu,
        cart: &Cart,
        chosen: slot2_retro::CoreId,
        mut session: Session,
        consumer: slot2_audio::Consumer,
    ) {
        // The whole object, so scale, overscan, rewind and any key a newer version wrote are
        // kept: this menu owns the `core` field and nothing else. The platform's own core is
        // the absence of an override rather than a name, which is what keeps a game the player
        // has handed back to the default indistinguishable from one that never left it.
        let default = slot2_retro::def(crate::session::retro_platform(cart.platform)).default_core;
        let mut settings = self.card.read_settings(cart);
        settings.core = if chosen == default {
            None
        } else {
            Some(chosen.base_name().to_string())
        };
        if let Err(e) = self.card.write_settings(cart, &settings) {
            eprintln!("slot2: cannot save the core choice for {}: {e}", cart.title);
            // The core that would have run is dropped rather than kept: a game running on a
            // core the card disagrees with is exactly the state this order exists to avoid.
            drop(session);
            self.recover_from_switch(menu, cart, "core-setting-save-failed");
            return;
        }

        if let Err(e) = resume_session(&mut session, &self.card, StateKind::Resume) {
            // The switch happened; the position did not come with it. A game that starts over
            // is worth saying out loud, and is not the same thing as the switch failing.
            eprintln!(
                "slot2: cannot resume {} on {}: {e}",
                cart.title,
                chosen.base_name()
            );
            self.toast = Some(slot2_ui::toast::Toast::new(
                "resume-load-failed",
                Vec::new(),
            ));
        }

        self.session = Some(session);
        self.pending_consumer = Some(consumer);
        self.sink_request = Some(SinkRequest::Open);
        // What belonged to the session that just ended goes with it: an undo restores a state
        // of a game that is no longer running, and a latched fast forward is the old core's.
        self.undo = None;
        self.ff_latch = false;
        self.screen = Screen::Playing;
    }

    /// Put the player back on the core the settings still name, after a switch that did not
    /// happen.
    ///
    /// The file has not changed, so the ordinary launch path is the honest way back: it opens
    /// whatever the settings mean, which is what was running — including an external library
    /// or a fallback that no `CoreId` here could name. The Resume the checkpoint just wrote is
    /// then loaded, so the player is where they were rather than at the start of the game.
    ///
    /// Losing the Resume and losing the core itself are not the same failure, and are not
    /// reported as one: the first is a playable game that starts over, and the second leaves
    /// nothing on screen to play.
    fn recover_from_switch(&mut self, menu: InGameMenu, cart: &Cart, key: &'static str) {
        let (mut session, consumer) = match Session::start(
            &self.card,
            cart,
            &self.core_dir,
            self.sink_rate,
            self.tuning,
        ) {
            Ok(pair) => pair,
            Err(e) => {
                // Nowhere to go: an overlay over a game that is not there is worse than the
                // shelf, which is where this cart goes now.
                eprintln!(
                    "slot2: cannot recover {} after a failed core change: {e}",
                    cart.title
                );
                self.session = None;
                self.pending_consumer = None;
                self.sink_request = Some(SinkRequest::Close);
                self.undo = None;
                self.ff_latch = false;
                // The game is gone, so its overlay is too: the next draw frees the texture.
                self.overlay.clear_sources();
                self.screen = Screen::Ejecting;
                self.anim = 0.0;
                self.sfx_fired = false;
                self.toast = Some(slot2_ui::toast::Toast::new(
                    "core-recovery-failed",
                    Vec::new(),
                ));
                return;
            }
        };

        let mut key = key;
        if let Err(e) = resume_session(&mut session, &self.card, StateKind::Resume) {
            eprintln!(
                "slot2: cannot load {}'s resume after a failed core change: {e}",
                cart.title
            );
            key = "core-recovery-state-failed";
        }

        self.session = Some(session);
        self.pending_consumer = Some(consumer);
        self.sink_request = Some(SinkRequest::Open);
        self.toast = Some(slot2_ui::toast::Toast::new(key, Vec::new()));
        // Back to the picker, which is where the player was: the row they chose is still the
        // one under the highlight, and the badge still says what is running.
        self.screen = Screen::Core(menu);
    }

    /// A on the in-game menu's Save State row: show the card's numbered states.
    ///
    /// It goes in even when there are none, so what the player sees is the empty view rather
    /// than a row that does nothing. The switcher is refreshed in place — it owns the
    /// thumbnail textures, and a new one would abandon them.
    fn open_state_switcher(&mut self, menu: InGameMenu) {
        let Some((cart, namespace)) = self
            .session
            .as_ref()
            .map(|s| (s.cart().clone(), s.state_namespace().clone()))
        else {
            return;
        };
        self.state_switcher
            .refresh(self.card.scoped_list_states(&cart, &namespace));
        self.screen = Screen::Switcher(menu);
    }

    /// Put numbered slot `n` back, and say what happened. `true` when the core took it.
    ///
    /// The screen is left to the caller: a quick load is already where it wants to be, and
    /// the switcher is not.
    fn load_numbered_state(&mut self, n: u32) -> bool {
        let Some(session) = self.session.as_mut() else {
            return false;
        };
        match session.load_state(&self.card, StateKind::Numbered(n)) {
            Ok(()) => {
                self.toast = Some(slot2_ui::toast::Toast::new(
                    "state-loaded",
                    vec![("n".to_string(), slot2_i18n::Arg::from(n))],
                ));
                true
            }
            Err(e) => {
                eprintln!("slot2: cannot load state {n}: {e}");
                self.toast = Some(slot2_ui::toast::Toast::new("state-load-failed", Vec::new()));
                false
            }
        }
    }

    /// X in the switcher: take the selected slot off the card and keep it for a while.
    ///
    /// The list on screen is the card's list, so it is refreshed from the card rather than
    /// patched by hand, and the selection lands where the deleted card was. A failure leaves
    /// whatever undo was already pending alone: the player did not ask for that one to go.
    fn delete_selected_state(&mut self, now: Instant) {
        let Some(kind @ StateKind::Numbered(n)) = self.state_switcher.selected_kind() else {
            // Nothing selected: the empty view is already the answer, and it is the same one
            // A gives.
            self.toast = Some(slot2_ui::toast::Toast::new("states-empty", Vec::new()));
            return;
        };
        let Some((cart, namespace)) = self
            .session
            .as_ref()
            .map(|s| (s.cart().clone(), s.state_namespace().clone()))
        else {
            return;
        };

        let taken = match self.card.scoped_take_state(&cart, &namespace, kind) {
            Ok(Some(backup)) => Some(backup),
            Ok(None) => {
                // The listing offered a state the card no longer has. Put the switcher back in
                // step with the card: the list on screen is meant to be the card's list.
                eprintln!("slot2: state {n} is not on the card");
                None
            }
            Err(e) => {
                eprintln!("slot2: cannot delete state {n}: {e}");
                None
            }
        };
        let Some(backup) = taken else {
            self.state_switcher
                .refresh(self.card.scoped_list_states(&cart, &namespace));
            self.toast = Some(slot2_ui::toast::Toast::new(
                "state-delete-failed",
                Vec::new(),
            ));
            return;
        };

        self.state_switcher
            .refresh(self.card.scoped_list_states(&cart, &namespace));
        self.state_switcher.select_after_removing(n);
        // A fresh deadline, and the older backup is dropped by being replaced: that is what
        // makes the older deletion permanent.
        self.undo = Some(PendingUndo {
            backup,
            expires: now + Duration::from_secs(STATE_UNDO_S),
        });
        self.toast = Some(slot2_ui::toast::Toast::new(
            "state-deleted",
            vec![("n".to_string(), slot2_i18n::Arg::from(n))],
        ));
    }

    /// Y in the switcher: put back what the last delete took, while its thirty seconds last.
    ///
    /// A failure keeps the backup *and* its deadline, so Y can be tried again without buying
    /// more time.
    fn undo_delete(&mut self, now: Instant) {
        let Some(pending) = self.undo.take().filter(|pending| now < pending.expires) else {
            // Nothing pending, or the deadline has passed: either way the deletion is permanent
            // now, and saying so is all that is left to do.
            self.toast = Some(slot2_ui::toast::Toast::new("undo-empty", Vec::new()));
            return;
        };

        // A backup only ever holds a numbered state: `take_state` refuses Resume.
        let StateKind::Numbered(n) = pending.backup.kind() else {
            return;
        };
        match self.card.restore_state(&pending.backup) {
            Ok(()) => {
                if let Some((cart, namespace)) = self
                    .session
                    .as_ref()
                    .map(|s| (s.cart().clone(), s.state_namespace().clone()))
                {
                    self.state_switcher
                        .refresh(self.card.scoped_list_states(&cart, &namespace));
                    self.state_switcher.select_number(n);
                }
                self.toast = Some(slot2_ui::toast::Toast::new(
                    "state-restored",
                    vec![("n".to_string(), slot2_i18n::Arg::from(n))],
                ));
            }
            Err(e) => {
                eprintln!("slot2: cannot put state {n} back: {e}");
                // The backup goes back with the deadline it came with: retrying must not buy
                // the player more time than they had.
                self.undo = Some(pending);
                self.toast = Some(slot2_ui::toast::Toast::new(
                    "state-restore-failed",
                    Vec::new(),
                ));
            }
        }
    }

    /// A on the in-game menu's Device row: the volume that is running, over the same frame.
    ///
    /// The volume belongs to the frontend rather than to the game, so this row opens whether or
    /// not a session is behind it. Brightness and blue light arrive as `None`: no backend here
    /// reports either — the control paths D-23 names are not settled — so the two rows are shown
    /// and cannot be chosen, which is the honest picture of a machine whose control has not been
    /// wired. Nothing is read from or written to the card, and no session, sink or toast is
    /// touched: this is a picture of a value that is already in memory.
    fn open_device_menu(&mut self, menu: InGameMenu) {
        self.screen = Screen::Device(menu, self.device_menu());
    }

    /// The Device menu for the volume this app is running at.
    ///
    /// Built rather than patched: `App::volume` is the one volume state there is, and a snapshot
    /// read back from it cannot drift from it. Brightness and blue light are `None` here for the
    /// same reason `open_device_menu` passes them: there is no backend to ask.
    fn device_menu(&self) -> DeviceMenu {
        DeviceMenu::new(self.volume.level(), self.volume.is_muted(), None, None)
    }

    /// A physical volume press, shown on an open Device menu.
    ///
    /// The keys work on every screen, so the menu follows them rather than handling them: the
    /// press has already gone through `App::volume` by the time this runs, and all that is left
    /// is to draw from it. On any other screen this is nothing at all.
    fn refresh_device_menu(&mut self) {
        let Screen::Device(menu, _) = self.screen else {
            return;
        };
        let next = self.device_menu();
        self.screen = Screen::Device(menu, next);
    }

    /// Left, Right and A on the Device screen: the running volume, and the picture of it.
    ///
    /// Only the volume row does anything. The other two rows cannot be chosen on this machine,
    /// and this guard is not only about a selection landing on them: a press that quietly changed
    /// some other setting — or that invented a capability so there would be something to change —
    /// would be a control the machine does not have.
    fn change_volume(
        &mut self,
        menu: InGameMenu,
        device: DeviceMenu,
        button: Button,
        now: Instant,
    ) {
        if device.selected() != DeviceSetting::Volume {
            return;
        }
        match button {
            Button::Left | Button::Right => self.step_volume(button, now),
            Button::A => self.volume.toggle_mute(),
            _ => return,
        }
        let next = self.device_menu();
        self.screen = Screen::Device(menu, next);
    }

    /// A press that moves the level: the volume does it, and the level it leaves schedules the
    /// write.
    ///
    /// The comparison is with the level before the press, so a press the clamp swallowed is not
    /// storage activity: VOL+ at 100 and VOL− at 0 change nothing, and must not push a write
    /// that is already pending further out either. The Device menu's Left and Right are the same
    /// two presses by another name, which is why they are here rather than in the menu.
    fn step_volume(&mut self, button: Button, now: Instant) {
        let before = self.volume.level();
        match button {
            Button::VolUp | Button::Right => self.volume.step_up(),
            Button::VolDown | Button::Left => self.volume.step_down(),
            _ => return,
        }
        if self.volume.level() != before {
            self.defer_volume_save(now);
        }
    }

    /// Note that the level is not on the card yet, and when writing it becomes due.
    ///
    /// A level that is back where the card already has it cancels the write rather than making
    /// one: up to 75 and back down to 70 while the delay lasts is not a decision worth an SD
    /// write. Mute never comes through here at all — it is not stored, so it is neither a reason
    /// to write nor a reason to wait longer.
    fn defer_volume_save(&mut self, now: Instant) {
        self.volume_due = if self.volume.level() == self.volume_saved {
            None
        } else {
            Some(now + Duration::from_millis(VOLUME_SAVE_DELAY_MS))
        };
    }

    /// Write the level that is running to the card, if it is not already there.
    ///
    /// The one place a global settings write happens, and the only thing the app knows about the
    /// file: the store owns its format, its fallback and the keys it does not own. A failure is a
    /// line in the log and nothing else — the sound the player just set is the point, and rolling
    /// it back or holding the input behind a card that will not answer would be worse than a
    /// level that does not survive a restart. The pending write is cleared either way, so a card
    /// that cannot be written costs one message rather than one per frame.
    fn flush_volume(&mut self) {
        let level = self.volume.level();
        self.volume_due = None;
        if level == self.volume_saved {
            // The card already says this: writing it again is a filesystem call that would
            // change nothing.
            return;
        }
        match self
            .card
            .write_global_settings(&GlobalSettings { volume: level })
        {
            Ok(()) => self.volume_saved = level,
            Err(e) => eprintln!("slot2: cannot save global volume: {e}"),
        }
    }

    /// SELECT+R1 while a game is running: write the next numbered state and say so.
    ///
    /// The screen, the session and the sink are left alone. A failed save is a message, not
    /// the end of the game: the player is holding a running game and losing it to a full
    /// card would be worse than losing the state.
    fn quick_save(&mut self) {
        let Some((cart, namespace)) = self
            .session
            .as_ref()
            .map(|s| (s.cart().clone(), s.state_namespace().clone()))
        else {
            return;
        };
        let n = self.card.scoped_next_state_number(&cart, &namespace);
        let saved = match self.session.as_mut() {
            Some(s) => s.save_state(&self.card, StateKind::Numbered(n)),
            None => return,
        };
        match saved {
            Ok(()) => {
                self.toast = Some(slot2_ui::toast::Toast::new(
                    "state-saved",
                    vec![
                        ("title".to_string(), slot2_i18n::Arg::Str(cart.title)),
                        ("n".to_string(), slot2_i18n::Arg::from(n)),
                    ],
                ));
            }
            Err(e) => {
                eprintln!("slot2: quick save failed: {e}");
                self.toast = Some(slot2_ui::toast::Toast::new("state-save-failed", Vec::new()));
            }
        }
    }

    /// SELECT+L1 while a game is running: put back the greatest numbered state.
    ///
    /// Resume is not a candidate, however new it is. It belongs to stopping and starting the
    /// frontend, and a chord that jumped into it would be a second way into the same state
    /// with a different meaning.
    fn quick_load(&mut self) {
        let Some((cart, namespace)) = self
            .session
            .as_ref()
            .map(|s| (s.cart().clone(), s.state_namespace().clone()))
        else {
            return;
        };
        let newest = self
            .card
            .scoped_list_states(&cart, &namespace)
            .iter()
            .filter_map(|slot| match slot.kind {
                StateKind::Numbered(n) => Some(n),
                StateKind::Resume => None,
            })
            .max();
        let Some(n) = newest else {
            self.toast = Some(slot2_ui::toast::Toast::new("states-empty", Vec::new()));
            return;
        };
        self.load_numbered_state(n);
    }

    /// Where closing the power menu goes back to.
    fn resume_screen(&self) -> Screen {
        if self.session.is_some() {
            Screen::Playing
        } else if self.carts.is_empty()
            && matches!(self.screen, Screen::Power(_))
            && self.card.root() == Path::new(".")
        {
            Screen::Splash
        } else {
            Screen::List
        }
    }

    fn switch_platform(&mut self, delta: isize) {
        let n = Platform::ALL.len() as isize;
        self.platform_index = ((self.platform_index as isize + delta).rem_euclid(n)) as usize;
        self.rescan();
    }

    pub fn draw(&mut self, canvas: &mut dyn Canvas, ctx: &mut UiCtx, now: Instant) {
        canvas.set_origin(self.refusal_offset(), 0.0);
        // A menu over a game is not a shelf: the game's own frame is the ground, and the
        // wallpaper under it would be a second thing claiming to be the frame.
        if !matches!(
            self.screen,
            Screen::Playing
                | Screen::InGame(_)
                | Screen::Display(..)
                | Screen::Shader(..)
                | Screen::Overscan(..)
                | Screen::Overlay(..)
                | Screen::Cheats(..)
                | Screen::Switcher(_)
                | Screen::Core(_)
                | Screen::Device(..)
        ) {
            self.wallpaper.draw(canvas, &ctx.safe);
        }
        match self.screen {
            Screen::Splash => {
                self.draw_overlay_nothing(canvas);
                self.splash.draw(canvas, ctx)
            }
            Screen::List
            | Screen::Shelf(_)
            | Screen::Timezone(..)
            | Screen::About(..)
            | Screen::Language(_) => {
                // The four menus are drawn over this shelf and nothing else: the settings menu,
                // the offset screen, the build sticker and the language picker belong to the
                // shelf, not to a game, so the shelf is the ground under them exactly as it is
                // under the list.
                self.draw_overlay_nothing(canvas);
                let platform = self.platform();
                let titles: Vec<&str> = self.carts.iter().map(|c| c.title.as_str()).collect();
                let safe = ctx.safe;
                self.shelf_view.draw(canvas, ctx, &safe, platform, &titles);
            }
            Screen::Inserting | Screen::Ejecting => {
                // Which way the cartridge is going, decided by the screen and nothing else:
                // the shelf itself owns the platform's rhythm for that direction.
                let motion = match self.screen {
                    Screen::Inserting => insert::Motion::Insert,
                    _ => insert::Motion::Eject,
                };
                // The cart is on its way in or out, and neither is a game on screen yet: a frame
                // and its overlay belong there only once the seat has been reached and
                // `Playing` is showing, never over the animation that is putting the cart in.
                // With no session at all there is nothing to wait for, so the overlay is told
                // that with a canvas in hand.
                if self.session.is_none() {
                    self.draw_overlay_nothing(canvas);
                }
                let platform = self.platform();
                let titles: Vec<&str> = self.carts.iter().map(|c| c.title.as_str()).collect();
                let seat = self.insert_seat().unwrap_or(0.0);
                let safe = ctx.safe;
                self.shelf_view.draw_insert(
                    canvas,
                    ctx,
                    &safe,
                    platform,
                    &titles,
                    insert::Insertion { seat, motion },
                );
            }
            Screen::Playing => self.draw_game_and_overlay(canvas, ctx),
            Screen::InGame(_)
            | Screen::Display(..)
            | Screen::Shader(..)
            | Screen::Overscan(..)
            | Screen::Overlay(..)
            | Screen::Cheats(..)
            | Screen::Switcher(_)
            | Screen::Core(_)
            | Screen::Device(..) => {
                // The game keeps the last frame it produced under the overlay — it is what
                // the overlay is about — and nothing else is drawn here: not the wallpaper,
                // not the HUD, not a clear.
                self.draw_game_and_overlay(canvas, ctx);
            }
            Screen::Power(_) => {
                // Draw whatever is underneath first. A running game keeps its own frame and its
                // overlay, in that order; a shelf or the splash has neither, and the overlay is
                // told so with a canvas in hand so a leftover texture can go.
                if self.session.is_some() {
                    self.draw_game_and_overlay(canvas, ctx);
                } else {
                    self.draw_overlay_nothing(canvas);
                    if !self.carts.is_empty() {
                        let platform = self.platform();
                        let titles: Vec<&str> =
                            self.carts.iter().map(|c| c.title.as_str()).collect();
                        let safe = ctx.safe;
                        self.shelf_view.draw(canvas, ctx, &safe, platform, &titles);
                    } else {
                        self.splash.draw(canvas, ctx);
                    }
                }
            }
        }

        // The badge describes the control the core is under at this moment, drawn over the
        // frame the game has just put down and under anything transient the app draws on top
        // of it. `None` on every other screen, so a latched fast forward stays out of sight
        // under a menu while it is still latched.
        let safe = ctx.safe;
        if let Some(control) = self.time_control() {
            self.hud
                .draw_time_control(canvas, ctx, &safe, Some(control));
        }
        // The scale menu, over the frame the game has just put down with the scale it has now:
        // what the choice looks like is the picture behind the menu. Drawn before the hold bar
        // and the toast, so those transient things stay on top of it.
        if let Screen::Display(_, menu) = self.screen {
            menu.draw(canvas, ctx);
        }
        // The shader menu, over the same frame — the frame the session has just drawn through
        // whatever effect is selected now, which is what makes the choice visible at once. Its
        // parent Display menu is not drawn underneath: the shader screen is the whole of what is
        // on screen, as the core picker and the cheats list are.
        if let Screen::Shader(_, _, menu) = self.screen {
            menu.draw(canvas, ctx);
        }
        // The overscan menu, over the same frame — the frame the session has just drawn through
        // whatever crop is selected now, which is what makes the choice visible at once. Its
        // parent Display menu is not drawn underneath, for the same reason the shader screen's
        // is not.
        if let Screen::Overscan(_, _, menu) = self.screen {
            menu.draw(canvas, ctx);
        }
        // The overlay menu, over the frame and over the running game's own overlay picture: the
        // picture is what the choice is about, and the menu's dim and panel go on top of it, as
        // they do over the game itself. Its parent Display menu is not drawn underneath, for the
        // same reason the shader screen's is not.
        if let Screen::Overlay(_, _, menu) = self.screen {
            menu.draw(canvas, ctx);
        }
        if let Screen::Cheats(_, menu) = self.screen {
            // The list is the session's, read here and not kept: the menu holds no copy of it, so
            // a cheat flipped a moment ago is on screen with no bookkeeping in between. A session
            // that went away leaves the menu an empty list rather than a panic.
            match self.session.as_ref() {
                Some(session) => menu.draw(canvas, ctx, session.cheats()),
                None => menu.draw(canvas, ctx, &[]),
            }
        }
        // The core picker, over the same frame. The in-game menu it was opened from is not
        // drawn under it: the picker is the whole of what is on screen, as the display and
        // cheats menus are.
        if let Screen::Core(_) = self.screen {
            self.core_picker.draw(canvas, ctx);
        }
        // The Device menu, over the same frame — and over nothing at all when there is no
        // session, which is a picture of the volume rather than of the game. The volume it
        // draws is the live one, so a press in another frame is already on screen.
        if let Screen::Device(_, menu) = self.screen {
            menu.draw(canvas, ctx);
        }
        // The shelf's own menus, over the shelf that has just been drawn: the settings menu on
        // its own, and the offset screen on its own — the menu it was opened from is not
        // underneath it, or two panels would be stacked where the player expects one. Drawn
        // before the hold bar and the toast, so those transient things stay on top.
        if let Screen::Shelf(menu) = self.screen {
            menu.draw(canvas, ctx);
        }
        if let Screen::Timezone(_, menu) = self.screen {
            menu.draw(canvas, ctx);
        }
        // The build sticker, over the same shelf and on its own: the settings menu it was opened
        // from is not underneath it, for the same reason the offset screen's is not. What it
        // prints is the frontend's own package version and the profile this build detected — the
        // UI crate's version and the host's OS would both be the wrong answer, and no revision is
        // compiled in to show.
        if let Screen::About(_, sticker) = self.screen {
            let info = AboutInfo {
                version: env!("CARGO_PKG_VERSION"),
                target: ctx.profile.target,
            };
            sticker.draw(canvas, ctx, info);
        }
        // The language picker, over the same shelf and on its own, for the same reason: the
        // settings menu it was opened from is not underneath it. The list it draws is the one
        // discovered when the row was opened.
        if let Screen::Language(_) = self.screen {
            self.language_picker.draw(canvas, ctx);
        }

        if let Some(p) = self.gestures.hold_progress(Button::Menu, now) {
            let x = ctx.safe.px(0.0);
            let y = ctx.safe.py(slot2_ui::SAFE_H as f32 - 4.0);
            let w = slot2_ui::SAFE_W as f32 * p;
            canvas.rect(x, y, w, 4.0, slot2_ui::splash::INK_DIM);
        }

        if let Screen::Power(menu) = self.screen {
            menu.draw(canvas, ctx);
        }
        if let Screen::InGame(menu) = self.screen {
            menu.draw(canvas, ctx);
        }
        if let Screen::Switcher(_) = self.screen {
            // The undo hint is only there while there is something to undo.
            self.state_switcher.draw(canvas, ctx, self.undo.is_some());
        }
        if let Some(t) = self.toast.as_ref() {
            t.draw(canvas, ctx, &safe);
        }
        // The corner furniture is the machine's, not the game's: it wears on the shelf-side
        // screens and stays off the splash (its own composition) and any running core
        // (which owns the screen).
        // One UTC sample decides both whether the corner shows and what it shows; judging a
        // shifted value would let the offset itself flicker the clock across SET_AFTER.
        let utc = clock::utc_now();
        let local = clock::hud_local(utc, clock::utc_offset_min());
        if matches!(
            self.screen,
            Screen::List
                | Screen::Inserting
                | Screen::Ejecting
                | Screen::Shelf(_)
                | Screen::Timezone(..)
                | Screen::About(..)
                | Screen::Language(_)
                | Screen::Power(_)
        ) {
            self.hud.draw(canvas, ctx, &safe, self.battery, local);
        }
        canvas.set_origin(0.0, 0.0);
    }
}

/// Put a session on the state the card keeps for the core it opened, when there is one.
///
/// The path is asked whether it is there, not read: `Card::read_state` turns every read error
/// into `None`, which would make an unreadable resume look like no resume at all. A state that
/// is genuinely gone — the card changed since the row was scanned — is not a failure and stays
/// quiet; one that is there and will not load is.
///
/// Only the session's own namespace is ever looked at. Another core's Resume is another
/// format, and reading it — let alone removing it — is not this launch's business.
fn resume_session(
    session: &mut Session,
    card: &Card,
    kind: StateKind,
) -> Result<(), crate::session::Error> {
    let path = card.scoped_state_path(session.cart(), session.state_namespace(), kind);
    if let Err(e) = std::fs::metadata(&path) {
        // Nothing there: a race with the card, and not worth a word.
        if e.kind() == std::io::ErrorKind::NotFound {
            return Ok(());
        }
        // Anything else — a states folder that cannot be asked — goes on to the load, whose
        // failure is the honest thing to report.
    }
    session.load_state(card, kind)
}

/// UI-only buttons (Menu, Power, volume) reach no core.
fn logical(b: Button) -> Option<LogicalButton> {
    Some(match b {
        Button::A => LogicalButton::A,
        Button::B => LogicalButton::B,
        Button::X => LogicalButton::X,
        Button::Y => LogicalButton::Y,
        Button::L1 => LogicalButton::L1,
        Button::R1 => LogicalButton::R1,
        Button::Select => LogicalButton::Select,
        Button::Start => LogicalButton::Start,
        Button::Up => LogicalButton::Up,
        Button::Down => LogicalButton::Down,
        Button::Left => LogicalButton::Left,
        Button::Right => LogicalButton::Right,
        // L2 and R2 drive rewind and fast forward, so they never reach the core. Nothing
        // this frontend emulates has a second pair of shoulder buttons to lose.
        Button::L2
        | Button::R2
        | Button::Menu
        | Button::Power
        | Button::VolUp
        | Button::VolDown => return None,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::Duration;

    use slot2_gfx::{Op, RecordingCanvas};
    use slot2_input::Button;
    use slot2_platform::by_target;
    use slot2_store::StateSlot;

    fn ev(b: Button, pressed: bool, at: Instant) -> Event {
        Event::Button {
            button: b,
            pressed,
            at,
        }
    }
    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }
    fn ctx() -> UiCtx {
        let fonts = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts");
        UiCtx::new(by_target("rgsp").unwrap(), "en", vec![fonts], None)
    }

    #[test]
    fn menu_hold_opens_power_menu_and_b_closes_it() {
        let mut app = App::new(false);
        let t = Instant::now();
        app.feed(&ev(Button::Menu, true, t));
        assert_eq!(app.screen, Screen::Splash);
        app.tick(t + ms(700));
        assert!(matches!(app.screen, Screen::Power(_)));
        app.feed(&ev(Button::Menu, false, t + ms(800)));
        assert!(
            matches!(app.screen, Screen::Power(_)),
            "release after hold does nothing"
        );
        app.feed(&ev(Button::B, true, t + ms(900)));
        app.feed(&ev(Button::B, false, t + ms(950)));
        assert_eq!(app.screen, Screen::Splash);
        assert_eq!(app.exit(), None);
    }

    #[test]
    fn menu_tap_does_not_open_the_power_menu() {
        let mut app = App::new(false);
        let t = Instant::now();
        app.feed(&ev(Button::Menu, true, t));
        app.feed(&ev(Button::Menu, false, t + ms(100)));
        app.tick(t + ms(2000));
        assert_eq!(app.screen, Screen::Splash);
    }

    #[test]
    fn navigate_and_choose_power_off() {
        let mut app = App::new(false);
        let t = Instant::now();
        app.feed(&ev(Button::Menu, true, t));
        app.tick(t + ms(700));
        // Down twice: Resume → Restart → PowerOff.
        for i in 0..2 {
            let at = t + ms(1000 + i * 200);
            app.feed(&ev(Button::Down, true, at));
            app.feed(&ev(Button::Down, false, at + ms(50)));
        }
        match app.screen {
            Screen::Power(m) => assert_eq!(m.choice(), slot2_ui::PowerChoice::PowerOff),
            _ => panic!(),
        }
        app.feed(&ev(Button::A, true, t + ms(2000)));
        app.feed(&ev(Button::A, false, t + ms(2050)));
        assert_eq!(app.exit(), Some(Exit::PowerOff));
        // Nothing changes after exit.
        app.feed(&ev(Button::B, true, t + ms(3000)));
        app.feed(&ev(Button::B, false, t + ms(3050)));
        assert_eq!(app.exit(), Some(Exit::PowerOff));
    }

    #[test]
    fn resume_returns_to_splash_and_restart_exits_reboot() {
        let mut app = App::new(false);
        let t = Instant::now();
        app.feed(&ev(Button::Menu, true, t));
        app.tick(t + ms(700));
        app.feed(&ev(Button::A, true, t + ms(1000)));
        app.feed(&ev(Button::A, false, t + ms(1050)));
        assert_eq!(app.screen, Screen::Splash);
        app.feed(&ev(Button::Menu, true, t + ms(2000)));
        app.tick(t + ms(2700));
        app.feed(&ev(Button::Down, true, t + ms(3000)));
        app.feed(&ev(Button::Down, false, t + ms(3050)));
        app.feed(&ev(Button::A, true, t + ms(3100)));
        app.feed(&ev(Button::A, false, t + ms(3150)));
        assert_eq!(app.exit(), Some(Exit::Reboot));
    }

    #[test]
    fn power_key_tap_powers_off_from_anywhere() {
        let mut app = App::new(false);
        let t = Instant::now();
        app.feed(&ev(Button::Power, true, t));
        app.feed(&ev(Button::Power, false, t + ms(50)));
        assert_eq!(app.exit(), Some(Exit::PowerOff));
    }

    #[test]
    fn state_sees_raw_presses() {
        let mut app = App::new(false);
        let t = Instant::now();
        app.feed(&ev(Button::A, true, t));
        assert!(app.state.pressed(Button::A));
        app.feed(&ev(Button::A, false, t + ms(10)));
        assert!(!app.state.pressed(Button::A));
    }

    // -------------------------------------------------------------- the in-game menu

    fn tap(app: &mut App, b: Button, at: Instant) {
        app.feed(&ev(b, true, at));
        app.feed(&ev(b, false, at + ms(40)));
        app.tick(at + ms(60));
    }

    /// A game on screen with the in-game menu open over it. `Playing` is set directly: the
    /// menu only ever opens over a running game, and loading a core to prove the input
    /// wiring would be cost with nothing to do with the transition under test.
    fn playing_with_menu_open() -> (App, Instant) {
        let mut app = App::new(false);
        app.screen = Screen::Playing;
        let t = Instant::now();
        tap(&mut app, Button::Menu, t);
        assert!(
            matches!(app.screen, Screen::InGame(_)),
            "a MENU tap over a game did not open the menu: {:?}",
            app.screen
        );
        (app, t + ms(200))
    }

    fn choice(app: &App) -> InGameChoice {
        match app.screen {
            Screen::InGame(m) => m.choice(),
            _ => panic!("the in-game menu is not open: {:?}", app.screen),
        }
    }

    #[test]
    fn a_menu_tap_over_a_game_opens_the_menu_and_a_hold_still_ejects() {
        let (mut app, t) = playing_with_menu_open();
        assert_eq!(choice(&app), InGameChoice::Continue, "the default row");
        assert_eq!(app.exit(), None);

        // It opens fresh each time: the row the last visit left is not remembered.
        tap(&mut app, Button::Down, t);
        assert_eq!(choice(&app), InGameChoice::SaveState);
        tap(&mut app, Button::B, t + ms(200));
        tap(&mut app, Button::Menu, t + ms(400));
        assert_eq!(
            choice(&app),
            InGameChoice::Continue,
            "the menu remembered the last row"
        );

        // The hold is still the eject, from the same screen.
        let mut app = App::new(false);
        app.screen = Screen::Playing;
        let t = Instant::now();
        app.feed(&ev(Button::Menu, true, t));
        app.tick(t + ms(700));
        assert_eq!(app.screen, Screen::Ejecting, "the hold stopped ejecting");
        assert_eq!(
            app.insert_seat(),
            Some(1.0),
            "the eject did not start at the slot"
        );
    }

    #[test]
    fn up_and_down_walk_the_menu_and_wrap() {
        let (mut app, t) = playing_with_menu_open();
        tap(&mut app, Button::Up, t);
        assert_eq!(
            choice(&app),
            InGameChoice::Eject,
            "up wrapped to the last row"
        );
        tap(&mut app, Button::Down, t + ms(200));
        assert_eq!(
            choice(&app),
            InGameChoice::Continue,
            "down wrapped to the first row"
        );

        let mut last = None;
        let mut now = t + ms(400);
        for _ in 0..slot2_ui::in_game_menu::INGAME_ITEMS.len() {
            tap(&mut app, Button::Down, now);
            now += ms(200);
            let now_choice = choice(&app);
            assert_ne!(Some(now_choice), last, "two downs landed on {now_choice:?}");
            last = Some(now_choice);
        }
        assert_eq!(
            choice(&app),
            InGameChoice::Continue,
            "one lap per row count"
        );
    }

    #[test]
    fn closing_the_menu_returns_to_the_game_and_leaves_the_session_alone() {
        for closer in [Button::B, Button::Menu] {
            let (mut app, t) = playing_with_menu_open();
            tap(&mut app, closer, t);
            assert_eq!(app.screen, Screen::Playing, "{closer:?} left the menu open");
            assert!(
                app.take_sink_request().is_none(),
                "{closer:?} tore the session's audio down"
            );
            assert_eq!(app.exit(), None);
            assert!(!app.audio_paused());
        }

        let (mut app, t) = playing_with_menu_open();
        tap(&mut app, Button::A, t);
        assert_eq!(
            app.screen,
            Screen::Playing,
            "A on Continue left the menu open"
        );
        assert!(app.take_sink_request().is_none());
    }

    #[test]
    fn the_rows_with_nothing_behind_them_stay_in_the_menu() {
        // The Device row is deliberately not here: the volume it shows belongs to the
        // frontend rather than to the game, so it opens with no session behind it. What it
        // does there is `tests/device_menu_app.rs`.
        let cases = [
            (1usize, InGameChoice::SaveState),
            (2, InGameChoice::Cheats),
            (3, InGameChoice::Display),
            (4, InGameChoice::Core),
        ];
        for (steps, want) in cases {
            let (mut app, t) = playing_with_menu_open();
            let mut now = t;
            for _ in 0..steps {
                tap(&mut app, Button::Down, now);
                now += ms(200);
            }
            assert_eq!(choice(&app), want);
            tap(&mut app, Button::A, now);
            assert_eq!(choice(&app), want, "{want:?} closed the menu");
            assert!(matches!(app.screen, Screen::InGame(_)));
            assert_eq!(app.exit(), None);
            assert!(
                app.take_sink_request().is_none(),
                "{want:?} touched the session's audio"
            );
        }
    }

    #[test]
    fn menus_pause_the_audio_and_a_running_game_does_not() {
        let mut app = App::new(false);
        app.screen = Screen::Playing;
        assert!(!app.audio_paused(), "a running game must keep its sound");
        app.screen = Screen::InGame(InGameMenu::default());
        assert!(
            app.audio_paused(),
            "the game plays on under the in-game menu"
        );
        app.screen = Screen::Device(
            InGameMenu::default(),
            DeviceMenu::new(70, false, None, None),
        );
        assert!(app.audio_paused(), "the Device menu left the game audible");
        app.screen = Screen::Power(PowerMenu::default());
        assert!(
            app.audio_paused(),
            "the power menu stopped silencing the game"
        );
        app.screen = Screen::List;
        assert!(!app.audio_paused());
    }

    #[test]
    fn drawing_the_in_game_menu_neither_clears_nor_paints_the_shelf_under_it() {
        let (mut app, _) = playing_with_menu_open();
        let mut ctx = ctx();
        let mut c = RecordingCanvas::new(720, 480);
        app.draw(&mut c, &mut ctx, Instant::now());
        let ops = c.frame();

        // It overlays the frame the game left; nothing clears it.
        assert!(
            !ops.iter().any(|o| matches!(o, Op::Clear(_))),
            "the menu cleared the game frame"
        );

        // The first mark is the menu's own dim, not the shelf's wallpaper: under a menu a
        // game stands on its own frame.
        match ops
            .iter()
            .find(|o| matches!(o, Op::Rect { .. } | Op::Image { .. }))
        {
            Some(Op::Rect { x, y, w, h, color }) => {
                assert_eq!((*x, *y, *w, *h), (0.0, 0.0, 720.0, 480.0));
                assert_eq!(
                    *color,
                    slot2_ui::in_game_menu::DIM,
                    "the first thing drawn was not the menu's dim"
                );
            }
            other => panic!("the menu drew nothing first: {other:?}"),
        }

        // And the menu is on screen: its box, where its own layout says it goes.
        let (bx, by) = InGameMenu::box_origin(&ctx);
        assert!(
            ops.iter().any(|o| matches!(o, Op::Rect { x, y, .. }
                if (*x - bx).abs() < 0.5 && (*y - by).abs() < 0.5)),
            "no menu box at {bx},{by}"
        );
    }

    // -------------------------------------------------------------- the state switcher

    /// A numbered slot, as the card's list would hand it over. Nothing opens the file: the
    /// switcher never reads one.
    fn slot(number: u32, mtime: u64) -> StateSlot {
        StateSlot {
            kind: StateKind::Numbered(number),
            path: PathBuf::from(format!("{number}.state")),
            thumb: None,
            modified: std::time::SystemTime::UNIX_EPOCH + ms(mtime),
        }
    }

    /// The switcher open over a game, with the menu row it was opened from. `Playing` and
    /// the slots are set directly: entering it for real needs a card and a running session,
    /// which is the integration test's job, and what is under test here is the input and the
    /// screen.
    fn switcher_over_a_game(slots: Vec<StateSlot>) -> (App, Instant) {
        let mut app = App::new(false);
        app.screen = Screen::Playing;
        let t = Instant::now();
        tap(&mut app, Button::Menu, t);
        tap(&mut app, Button::Down, t + ms(200)); // the Save State row
        let menu = match app.screen {
            Screen::InGame(m) => m,
            other => panic!("the in-game menu is not open: {other:?}"),
        };
        assert_eq!(menu.choice(), InGameChoice::SaveState);
        app.state_switcher.refresh(slots);
        app.screen = Screen::Switcher(menu);
        (app, t + ms(400))
    }

    #[test]
    fn the_switcher_wraps_and_returns_to_the_same_menu_row() {
        let (mut app, t) = switcher_over_a_game(vec![slot(1, 10), slot(3, 20), slot(2, 30)]);
        assert_eq!(
            app.state_switcher.selected_kind(),
            Some(StateKind::Numbered(3)),
            "the greatest numbered slot, in whatever order they arrived"
        );
        assert!(app.audio_paused(), "the switcher kept the game audible");

        let mut now = t;
        for want in [2, 1, 3] {
            tap(&mut app, Button::Left, now);
            now += ms(200);
            assert_eq!(
                app.state_switcher.selected_kind(),
                Some(StateKind::Numbered(want))
            );
        }
        tap(&mut app, Button::Right, now);
        assert_eq!(
            app.state_switcher.selected_kind(),
            Some(StateKind::Numbered(1)),
            "right wraps to the first"
        );

        // Up and Down are the menu's, not the switcher's.
        let before = app.state_switcher.selected_kind();
        tap(&mut app, Button::Up, now + ms(200));
        tap(&mut app, Button::Down, now + ms(400));
        assert!(
            matches!(app.screen, Screen::Switcher(_)),
            "a direction closed the switcher"
        );
        assert_eq!(app.state_switcher.selected_kind(), before);
        assert_eq!(app.toast_key(), None, "a direction said something");

        // B and MENU hand back the row it was opened from, not a fresh menu.
        for closer in [Button::B, Button::Menu] {
            let (mut app, t) = switcher_over_a_game(vec![slot(1, 10), slot(2, 20)]);
            tap(&mut app, closer, t);
            match app.screen {
                Screen::InGame(m) => {
                    assert_eq!(
                        m.choice(),
                        InGameChoice::SaveState,
                        "{closer:?} lost the row"
                    );
                    assert_eq!(m.selected_index(), 1, "{closer:?} moved the row");
                }
                other => panic!("{closer:?} left the switcher on {other:?}"),
            }
            assert!(
                app.take_sink_request().is_none(),
                "{closer:?} touched the session's audio"
            );
            assert_eq!(app.exit(), None);
        }
    }

    #[test]
    fn the_empty_switcher_says_so_and_stays_where_it_is() {
        let (mut app, t) = switcher_over_a_game(Vec::new());
        assert_eq!(app.state_switcher.selected_kind(), None);
        assert!(app.audio_paused(), "the empty view kept the game audible");

        tap(&mut app, Button::A, t);
        assert_eq!(app.toast_key(), Some("states-empty"));
        assert!(
            matches!(app.screen, Screen::Switcher(_)),
            "the empty view closed itself"
        );
        assert_eq!(app.exit(), None);
        assert!(
            app.take_sink_request().is_none(),
            "the empty view touched the session's audio"
        );
    }

    #[test]
    fn drawing_the_switcher_neither_clears_nor_paints_the_shelf_under_it() {
        let (mut app, _) = switcher_over_a_game(vec![slot(1, 10), slot(2, 20)]);
        let mut ctx = ctx();
        let mut c = RecordingCanvas::new(720, 480);
        app.draw(&mut c, &mut ctx, Instant::now());
        let ops = c.frame();

        assert!(
            !ops.iter().any(|o| matches!(o, Op::Clear(_))),
            "the switcher cleared the game frame"
        );

        // The first mark is the switcher's own dim, not the shelf's wallpaper: under it a
        // game stands on its own frame.
        match ops
            .iter()
            .find(|o| matches!(o, Op::Rect { .. } | Op::Image { .. }))
        {
            Some(Op::Rect { x, y, w, h, color }) => {
                assert_eq!((*x, *y, *w, *h), (0.0, 0.0, 720.0, 480.0));
                assert_eq!(
                    *color,
                    slot2_ui::state_switcher::DIM,
                    "the first thing drawn was not the switcher's dim"
                );
            }
            other => panic!("the switcher drew nothing first: {other:?}"),
        }

        // Not the in-game menu under it, and the cards on it.
        let (bx, by) = InGameMenu::box_origin(&ctx);
        assert!(
            !ops.iter().any(|o| matches!(o, Op::Rect { x, y, .. }
                if (*x - bx).abs() < 0.5 && (*y - by).abs() < 0.5)),
            "the in-game menu was drawn under the switcher"
        );
        let (cx, cy, ..) = slot2_ui::StateSwitcher::card_rect(&ctx, 0);
        assert!(
            ops.iter().any(|o| matches!(o, Op::Rect { x, y, .. }
                if (*x - cx).abs() < 0.5 && (*y - cy).abs() < 0.5)),
            "no selected card on the strip"
        );
    }

    // -------------------------------------------------------------- deleting and undoing

    /// A card with one numbered state on it, and the cart it belongs to.
    fn card_with_a_state(tag: &str) -> (Card, Cart, PathBuf) {
        let dir = std::env::temp_dir().join(format!("slot2-undo-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let card = Card::new(&dir);
        card.ensure_layout();
        let cart = Cart {
            platform: Platform::Nes,
            stem: "Mappy".into(),
            title: "Mappy".into(),
            rom: dir.join("Games/NES/Mappy.nes"),
        };
        card.write_state(&cart, StateKind::Numbered(1), b"one", None)
            .unwrap();
        (card, cart, dir)
    }

    /// The switcher open over a card whose one state has just been deleted, with that deletion
    /// waiting to be put back. The backup is seeded by hand: a delete through X needs a live
    /// session for its cart, and what is under test here is the deadline.
    fn app_with_a_pending_undo(tag: &str, t: Instant) -> (App, Card, Cart) {
        let (card, cart, _dir) = card_with_a_state(tag);
        let backup = card
            .take_state(&cart, StateKind::Numbered(1))
            .unwrap()
            .expect("the state was there");
        let mut app = App::with_card(
            card.clone(),
            PathBuf::from("."),
            48_000,
            slot2_retro::Tuning::handheld((720, 480)),
            false,
            Screen::Switcher(InGameMenu::default()),
        );
        app.state_switcher.refresh(card.list_states(&cart));
        app.undo = Some(PendingUndo {
            backup,
            expires: t + Duration::from_secs(STATE_UNDO_S),
        });
        (app, card, cart)
    }

    /// A tap whose press and release the caller chooses. `tap`'s fixed 40 ms release would move
    /// the boundary this is here to test.
    fn tap_between(app: &mut App, b: Button, down: Instant, up: Instant) {
        app.feed(&ev(b, true, down));
        app.feed(&ev(b, false, up));
    }

    #[test]
    fn an_undo_lasts_thirty_seconds_and_puts_the_state_back() {
        let t = Instant::now();
        let (mut app, card, cart) = app_with_a_pending_undo("inside", t);
        let state = card.state_path(&cart, StateKind::Numbered(1));
        assert!(!state.exists(), "nothing was deleted to begin with");

        // A tick a millisecond inside the deadline must not take it away.
        let inside = t + Duration::from_millis(STATE_UNDO_S * 1000 - 1);
        app.tick(inside);
        assert!(app.undo.is_some(), "the undo expired a millisecond early");
        tap_between(&mut app, Button::Y, inside - ms(50), inside);

        assert_eq!(app.toast_key(), Some("state-restored"));
        assert_eq!(
            std::fs::read(&state).unwrap(),
            b"one",
            "the state did not come back"
        );
        assert!(app.undo.is_none(), "a spent undo is still pending");
        assert!(
            matches!(app.screen, Screen::Switcher(_)),
            "an undo left the switcher"
        );
        assert!(
            app.take_sink_request().is_none(),
            "an undo touched the audio"
        );

        // And a second Y has nothing left to do.
        tap(&mut app, Button::Y, inside + ms(200));
        assert_eq!(app.toast_key(), Some("undo-empty"));
        assert_eq!(std::fs::read(&state).unwrap(), b"one");
    }

    #[test]
    fn an_undo_is_gone_at_thirty_seconds() {
        let t = Instant::now();
        let (mut app, card, cart) = app_with_a_pending_undo("boundary", t);
        let state = card.state_path(&cart, StateKind::Numbered(1));

        // The tick at exactly thirty seconds drops it, before anything it does can use it.
        let deadline = t + Duration::from_secs(STATE_UNDO_S);
        app.tick(deadline);
        assert!(app.undo.is_none(), "the undo outlived its deadline");
        tap_between(&mut app, Button::Y, deadline, deadline + ms(40));
        assert_eq!(app.toast_key(), Some("undo-empty"));
        assert!(!state.exists(), "an expired delete came back");
        assert!(matches!(app.screen, Screen::Switcher(_)));

        // And after the deadline it is the same nothing.
        let t2 = Instant::now();
        let (mut app, card, cart) = app_with_a_pending_undo("after", t2);
        let state = card.state_path(&cart, StateKind::Numbered(1));
        let after = t2 + Duration::from_secs(STATE_UNDO_S + 1);
        app.tick(after);
        tap_between(&mut app, Button::Y, after, after + ms(40));
        assert_eq!(app.toast_key(), Some("undo-empty"));
        assert!(!state.exists(), "a delete after the deadline came back");
        assert!(app.undo.is_none());
    }

    #[test]
    fn a_refused_undo_keeps_its_backup_and_can_be_tried_again() {
        let t = Instant::now();
        let (mut app, card, cart) = app_with_a_pending_undo("refused", t);
        let state = card.state_path(&cart, StateKind::Numbered(1));

        // Something in the way: the restore refuses rather than mixing two states.
        std::fs::write(&state, b"newer").unwrap();
        let at = t + Duration::from_secs(20);
        app.tick(at);
        tap_between(&mut app, Button::Y, at, at + ms(40));
        assert_eq!(app.toast_key(), Some("state-restore-failed"));
        assert!(app.undo.is_some(), "a refusal dropped the backup");
        assert_eq!(
            std::fs::read(&state).unwrap(),
            b"newer",
            "the newer state was overwritten"
        );
        assert!(matches!(app.screen, Screen::Switcher(_)));

        // Clear the way, and the same Y works.
        std::fs::remove_file(&state).unwrap();
        let later = t + Duration::from_secs(25);
        app.tick(later);
        tap_between(&mut app, Button::Y, later, later + ms(40));
        assert_eq!(app.toast_key(), Some("state-restored"));
        assert_eq!(std::fs::read(&state).unwrap(), b"one");
        assert!(app.undo.is_none());
    }

    #[test]
    fn a_refused_undo_does_not_buy_more_time() {
        let t = Instant::now();
        let (mut app, card, cart) = app_with_a_pending_undo("no-extension", t);
        let state = card.state_path(&cart, StateKind::Numbered(1));
        std::fs::write(&state, b"newer").unwrap();

        // A refusal at twenty-nine seconds leaves the deadline where it was.
        let late = t + Duration::from_secs(29);
        app.tick(late);
        tap_between(&mut app, Button::Y, late, late + ms(40));
        assert_eq!(app.toast_key(), Some("state-restore-failed"));
        assert!(app.undo.is_some());

        std::fs::remove_file(&state).unwrap();
        let after = t + Duration::from_secs(STATE_UNDO_S + 1);
        app.tick(after);
        assert!(app.undo.is_none(), "a refusal extended the thirty seconds");
        tap_between(&mut app, Button::Y, after, after + ms(40));
        assert_eq!(app.toast_key(), Some("undo-empty"));
        assert!(!state.exists());
    }

    #[test]
    fn x_with_nothing_selected_says_so_and_deletes_nothing() {
        let (mut app, t) = switcher_over_a_game(Vec::new());
        tap(&mut app, Button::X, t);
        assert_eq!(app.toast_key(), Some("states-empty"));
        assert!(matches!(app.screen, Screen::Switcher(_)));
        assert!(app.undo.is_none());
        assert!(app.take_sink_request().is_none());
        assert_eq!(app.exit(), None);
    }

    #[test]
    fn the_undo_hint_is_there_only_while_there_is_an_undo() {
        let (mut app, _) = switcher_over_a_game(vec![slot(1, 10)]);
        let mut ctx = ctx();
        let mut c = RecordingCanvas::new(720, 480);
        let caps = |ops: &[Op]| {
            let cap = slot2_ui::splash::INK_DIM.with_alpha(0.25);
            ops.iter()
                .filter(|o| matches!(o, Op::Rect { color, .. } if *color == cap))
                .count()
        };
        app.draw(&mut c, &mut ctx, Instant::now());
        assert_eq!(caps(c.frame()), 3, "load, delete and back, and no undo");

        let (card, cart, _dir) = card_with_a_state("hint");
        let backup = card
            .take_state(&cart, StateKind::Numbered(1))
            .unwrap()
            .unwrap();
        app.undo = Some(PendingUndo {
            backup,
            expires: Instant::now() + Duration::from_secs(STATE_UNDO_S),
        });
        let warm = c.ops.len();
        app.draw(&mut c, &mut ctx, Instant::now());
        assert_eq!(caps(&c.ops[warm..]), 4, "the undo hint did not appear");

        // And with nothing left to load or delete, the row is back and undo — the undo is the
        // only thing there is to press besides back.
        app.state_switcher.refresh(Vec::new());
        let warm = c.ops.len();
        app.draw(&mut c, &mut ctx, Instant::now());
        assert_eq!(
            caps(&c.ops[warm..]),
            2,
            "the empty row is not back and undo"
        );
    }

    // -------------------------------------------------------------- launching from the shelf

    /// A card with `n` carts on it, and no cores anywhere: what is under test is the launch
    /// decision, not the game.
    fn card_with_carts(tag: &str, n: usize) -> (Card, PathBuf) {
        let dir = std::env::temp_dir().join(format!("slot2-launch-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let card = Card::new(&dir);
        card.ensure_layout();
        for i in 0..n {
            let name = format!("{}.gba", (b'A' + i as u8) as char);
            std::fs::write(card.games_dir(Platform::Gba).join(name), b"rom").unwrap();
        }
        (card, dir)
    }

    fn shelf_app(tag: &str, n: usize) -> App {
        let (card, _dir) = card_with_carts(tag, n);
        App::with_card(
            card,
            PathBuf::from(".").join("no-cores"),
            48_000,
            slot2_retro::Tuning::handheld((720, 480)),
            false,
            Screen::List,
        )
    }

    #[test]
    fn a_held_launch_is_kept_from_the_core_until_it_is_let_go() {
        let mut app = shelf_app("held", 1);
        let t = Instant::now();
        app.feed(&ev(Button::A, true, t));
        app.tick(t + ms(700));
        assert_eq!(app.screen, Screen::Inserting, "the hold did not launch");
        assert!(app.suppress_a, "the launch press was not held back");
        assert!(
            !app.held_game_buttons().contains(&LogicalButton::A),
            "the core was shown the press that started the launch"
        );

        // The release hands A back, and a press of the player's own reaches the core.
        app.feed(&ev(Button::A, false, t + ms(900)));
        assert!(!app.suppress_a, "the release did not hand A back");
        app.feed(&ev(Button::A, true, t + ms(1000)));
        assert!(
            app.held_game_buttons().contains(&LogicalButton::A),
            "a later press of the player's own was swallowed"
        );
    }

    #[test]
    fn the_cart_and_the_intent_are_fixed_when_the_gesture_lands() {
        let mut app = shelf_app("fixed", 2);
        let t = Instant::now();
        tap(&mut app, Button::A, t);
        assert_eq!(app.screen, Screen::Inserting);

        // The row and the card's own list move under the animation. The launch does not.
        app.shelf_view.shelf.right();
        app.carts.reverse();
        let launch = app.launch.as_ref().expect("no launch was recorded");
        assert_eq!(launch.cart.stem, "A", "the launch followed the cursor");
        assert_eq!(launch.intent, Launch::Resume, "a tap is not a fresh launch");

        let mut app = shelf_app("fixed-fresh", 2);
        let t = Instant::now();
        app.feed(&ev(Button::A, true, t));
        app.tick(t + ms(700));
        app.shelf_view.shelf.right();
        app.rescan();
        assert_eq!(app.launch.as_ref().map(|p| p.intent), Some(Launch::Fresh));
        assert_eq!(
            app.launch.as_ref().map(|p| p.cart.stem.as_str()),
            Some("A"),
            "a rescan changed the cart that is loading"
        );
    }

    #[test]
    fn the_shelf_hint_follows_the_scan_and_not_the_disk() {
        let (card, _dir) = card_with_carts("hint", 1);
        let cart = Cart {
            platform: Platform::Gba,
            stem: "A".into(),
            title: "A".into(),
            rom: card.games_dir(Platform::Gba).join("A.gba"),
        };
        let root = card.root().to_path_buf();
        let mut app = App::with_card(
            card.clone(),
            PathBuf::from("."),
            48_000,
            slot2_retro::Tuning::handheld((720, 480)),
            false,
            Screen::List,
        );
        let mut ctx = ctx();
        let mut c = RecordingCanvas::new(720, 480);
        let caps = |ops: &[Op]| {
            let cap = slot2_ui::splash::INK_DIM.with_alpha(0.25);
            ops.iter()
                .filter(|o| matches!(o, Op::Rect { color, .. } if *color == cap))
                .count()
        };
        let drawn = |app: &mut App, c: &mut RecordingCanvas, ctx: &mut UiCtx| {
            let warm = c.ops.len();
            app.draw(c, ctx, Instant::now());
            caps(&c.ops[warm..])
        };

        assert_eq!(drawn(&mut app, &mut c, &mut ctx), 1, "A plays, and says so");

        // The namespace the scan reads for this cart: the platform's own core, whether or not
        // that library is on this machine.
        let ns = crate::session::default_namespace(Platform::Gba);

        // The state appears on the card, in that namespace. A draw does not go looking for it.
        card.scoped_write_state(&cart, &ns, StateKind::Resume, b"x", None)
            .unwrap();
        assert_eq!(
            drawn(&mut app, &mut c, &mut ctx),
            1,
            "the draw asked the card"
        );
        // Nor when the file goes away again: the answer is the scan's, until the next one.
        card.scoped_delete_state(&cart, &ns, StateKind::Resume)
            .unwrap();
        assert_eq!(drawn(&mut app, &mut c, &mut ctx), 1);
        // Nor when the whole card is taken away under it: nothing is stat'd, so nothing breaks.
        std::fs::rename(&root, root.with_extension("gone")).unwrap();
        assert_eq!(
            drawn(&mut app, &mut c, &mut ctx),
            1,
            "a missing card was asked"
        );

        // A scan does see it — first the state, then the card itself.
        std::fs::rename(root.with_extension("gone"), &root).unwrap();
        card.scoped_write_state(&cart, &ns, StateKind::Resume, b"x", None)
            .unwrap();
        app.rescan();
        assert_eq!(
            drawn(&mut app, &mut c, &mut ctx),
            2,
            "the scan saw no resume state"
        );
        std::fs::rename(&root, root.with_extension("gone")).unwrap();
        app.rescan();
        assert_eq!(
            drawn(&mut app, &mut c, &mut ctx),
            0,
            "a card that is not there still offered a cart"
        );
    }

    // -------------------------------------------------------------- the R2 time controls

    /// An R2 double tap: two presses inside the gesture layer's window. The action fires on the
    /// second press, which is still physically down — releasing it is what the player does next.
    fn double_tap_r2(app: &mut App, at: Instant) {
        app.feed(&ev(Button::R2, true, at));
        app.feed(&ev(Button::R2, false, at + ms(40)));
        app.feed(&ev(Button::R2, true, at + ms(100)));
        app.tick(at + ms(120));
    }

    #[test]
    fn a_double_tap_outside_the_game_does_not_arm_the_next_one() {
        // A latch armed on a shelf, mid-animation or under a menu would be a surprise waiting
        // for the game that comes next.
        for screen in [
            Screen::List,
            Screen::Inserting,
            Screen::Ejecting,
            Screen::Power(PowerMenu::default()),
        ] {
            let mut app = App::new(false);
            app.screen = screen;
            double_tap_r2(&mut app, Instant::now());
            assert!(!app.ff_latch, "{screen:?} armed the fast-forward latch");
        }

        // `Playing` with nothing running is not a game either.
        let mut app = App::new(false);
        app.screen = Screen::Playing;
        double_tap_r2(&mut app, Instant::now());
        assert!(!app.ff_latch, "a screen with no session armed the latch");

        let (mut app, t) = playing_with_menu_open();
        double_tap_r2(&mut app, t);
        assert!(!app.ff_latch, "the in-game menu armed the latch");

        let (mut app, t) = switcher_over_a_game(vec![slot(1, 10)]);
        double_tap_r2(&mut app, t);
        assert!(!app.ff_latch, "the state switcher armed the latch");
    }

    #[test]
    fn opening_and_closing_a_menu_leaves_the_latch_where_it_was() {
        let mut app = App::new(false);
        app.screen = Screen::Playing;
        app.ff_latch = true;
        let t = Instant::now();
        tap(&mut app, Button::Menu, t);
        assert!(matches!(app.screen, Screen::InGame(_)), "{:?}", app.screen);
        assert!(app.ff_latch, "the menu cleared the latch");
        tap(&mut app, Button::B, t + ms(200));
        assert_eq!(app.screen, Screen::Playing);
        assert!(app.ff_latch, "closing the menu cleared the latch");

        // And the switcher, whose visit is not a launch either.
        let (mut app, t) = switcher_over_a_game(vec![slot(1, 10)]);
        app.ff_latch = true;
        tap(&mut app, Button::Right, t);
        tap(&mut app, Button::B, t + ms(200));
        assert!(matches!(app.screen, Screen::InGame(_)), "{:?}", app.screen);
        assert!(app.ff_latch, "the switcher cleared the latch");
    }

    #[test]
    fn every_way_out_of_a_session_clears_the_latch() {
        // Power off, from the game.
        let mut app = App::new(false);
        app.screen = Screen::Playing;
        app.ff_latch = true;
        tap(&mut app, Button::Power, Instant::now());
        assert_eq!(app.exit(), Some(Exit::PowerOff));
        assert!(!app.ff_latch, "power off left the latch armed");

        // The MENU hold that takes the cart out.
        let mut app = App::new(false);
        app.screen = Screen::Playing;
        app.ff_latch = true;
        let t = Instant::now();
        app.feed(&ev(Button::Menu, true, t));
        app.tick(t + ms(700));
        assert_eq!(app.screen, Screen::Ejecting);
        assert!(!app.ff_latch, "an eject left the latch armed");

        // The in-game menu's Eject row, which runs the same shutdown.
        let mut menu = InGameMenu::default();
        for _ in 0..slot2_ui::in_game_menu::INGAME_ITEMS.len() - 1 {
            menu.down();
        }
        assert_eq!(menu.choice(), InGameChoice::Eject);
        let mut app = App::new(false);
        app.screen = Screen::InGame(menu);
        app.ff_latch = true;
        tap(&mut app, Button::A, Instant::now());
        assert_eq!(app.screen, Screen::Ejecting);
        assert!(!app.ff_latch, "the menu's Eject left the latch armed");

        // The power menu's Restart and Power off.
        for steps in [1usize, 2] {
            let mut app = App::new(false);
            app.screen = Screen::Power(PowerMenu::default());
            app.ff_latch = true;
            let t = Instant::now();
            let mut now = t;
            for _ in 0..steps {
                tap(&mut app, Button::Down, now);
                now += ms(200);
            }
            tap(&mut app, Button::A, now);
            assert!(app.exit().is_some(), "the power menu did nothing");
            assert!(!app.ff_latch, "the power menu left the latch armed");
        }
    }

    #[test]
    fn draw_shows_hold_progress_then_the_menu() {
        let mut app = App::new(false);
        let mut ctx = ctx();
        let t = Instant::now();
        let mut c = RecordingCanvas::new(720, 480);
        app.draw(&mut c, &mut ctx, t);
        app.draw(&mut c, &mut ctx, t); // warm: the first frame's uploads would inflate the count
        let splash_ops = c.frame().len();
        // Half way through a MENU hold: a progress bar 320 wide at the bottom of the safe area.
        app.feed(&ev(Button::Menu, true, t));
        app.draw(&mut c, &mut ctx, t + ms(300));
        let bar = c.frame().iter().find(|o| matches!(o, Op::Rect { y, h, .. } if (*y - 476.0).abs() < 0.5 && (*h - 4.0).abs() < 0.5));
        match bar {
            Some(Op::Rect { x, w, .. }) => {
                assert_eq!(*x, 40.0);
                assert!((w - 320.0).abs() < 2.0, "{w}");
            }
            _ => panic!("no progress bar: {:?}", c.frame()),
        }
        assert!(c.frame().len() > splash_ops);
        // After the hold fires, the menu is drawn (dim rect over the whole panel) and no bar.
        app.tick(t + ms(700));
        app.draw(&mut c, &mut ctx, t + ms(700));
        assert!(c.frame().iter().any(|o| matches!(o, Op::Rect { x, y, w, h, .. } if *x == 0.0 && *y == 0.0 && *w == 720.0 && *h == 480.0)));
        assert!(!c.frame().iter().any(|o| matches!(o, Op::Rect { y, h, .. } if (*y - 476.0).abs() < 0.5 && (*h - 4.0).abs() < 0.5)));
    }
}
