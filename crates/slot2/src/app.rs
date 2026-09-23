//! The frontend's screen state machine, shared by both backends and driven by input
//! actions. Implementation notes for task 05 (see tasks/05-input.md):
//!
//! ```text
//! Screen::Splash
//!   Hold(Menu)            → Screen::Power(PowerMenu::default())
//! Screen::Power(menu)
//!   Tap(Up) / Tap(Down)   → menu.up() / menu.down()
//!   Tap(A)                → match menu.choice():
//!                              Resume   → Screen::Splash
//!                              Restart  → exit = Some(Exit::Reboot)
//!                              PowerOff → exit = Some(Exit::PowerOff)
//!   Tap(B) | Tap(Menu)    → Screen::Splash
//! any screen
//!   Tap(Power)            → exit = Some(Exit::PowerOff)     (the physical power key)
//! ```
//! `feed` runs the event through `state` (always) and `gestures`, then `act`s on each
//! action in order. `tick` runs `gestures.tick`. `draw` draws the splash, then a hold
//! progress bar when `gestures.hold_progress(Menu, now)` is `Some(p)`: a rect at the
//! bottom of the safe area, `x = safe.px(0)`, `y = safe.py(SAFE_H - 4)`, width
//! `SAFE_W * p`, height 4, colour `splash::INK_DIM`; then, on `Screen::Power`, the menu
//! over it. Once `exit` is set no further actions change anything.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use slot2_gfx::Canvas;
use slot2_input::{Action, Button, Event, GestureConfig, Gestures, State};
use slot2_retro::LogicalButton;
use slot2_store::{Card, Cart, Platform};
use slot2_ui::{insert, shelf_view::ShelfView, PowerMenu, Splash, UiCtx};

use crate::session::Session;

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

/// How much faster R2 makes it. Four is the most that is still followable on a handheld
/// screen, and the A53 can hold it for every core here.
pub const FAST_FORWARD: u32 = 4;

pub struct App {
    pub screen: Screen,
    pub state: State,
    pub gestures: Gestures,
    pub splash: Splash,
    pub shelf_view: ShelfView,
    pub volume: slot2_audio::Volume,
    card: Card,
    core_dir: PathBuf,
    sink_rate: u32,
    tuning: slot2_retro::Tuning,
    platform_index: usize,
    carts: Vec<Cart>,
    session: Option<Session>,
    /// Set when a session starts; the loop takes it to build its sink.
    pending_consumer: Option<slot2_audio::Consumer>,
    sink_request: Option<SinkRequest>,
    exit: Option<Exit>,
    last_tick: Option<Instant>,
    anim: f32,
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
        let mut app = App {
            screen,
            state: State::default(),
            gestures: Gestures::new(GestureConfig::default()),
            splash: Splash { debug_frame },
            shelf_view: ShelfView::default(),
            volume: slot2_audio::Volume::default(),
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
            pending_consumer: None,
            sink_request: None,
            exit: None,
            last_tick: None,
            anim: 0.0,
        };
        if screen == Screen::List {
            app.rescan();
        }
        app
    }

    pub fn exit(&self) -> Option<Exit> {
        self.exit
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
        self.carts = self.card.scan(self.platform());
        // The card names a cart by its file; the shelf shows a game. `(USA) (Rev 1)` is a
        // fact about the dump, and it is cleaned off here, once, rather than in the draw
        // loop where it would be rebuilt for every cart on screen sixty times a second.
        for cart in &mut self.carts {
            cart.title = slot2_ui::label::clean_title(&cart.stem);
        }
        self.shelf_view.shelf.set_len(self.carts.len());
        let labels = self.carts.iter().map(|c| self.card.label(c)).collect();
        self.shelf_view.set_labels(labels);
    }

    pub fn feed(&mut self, event: &Event) {
        self.state.feed(event);
        let actions = self.gestures.feed(event);
        for action in actions {
            self.act(action);
        }
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

            if matches!(self.screen, Screen::Inserting | Screen::Ejecting) {
                self.anim += dt;
                self.advance_insert();
            }
        }
        self.last_tick = Some(now);

        let actions = self.gestures.tick(now);
        for action in actions {
            self.act(action);
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
        match self.screen {
            Screen::Inserting if self.session.is_none() => {
                if self.anim < insert::SEATED_AT {
                    return;
                }
                self.start_selected();
                if self.session.is_none() {
                    // No core for this cart — an ordinary state for a half-built card. The
                    // cart comes back out from where it got to, which is the slot, rather
                    // than sitting on the seated frame for ever.
                    self.screen = Screen::Ejecting;
                    self.anim = 0.0;
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
        let down = self.state.held();
        let rewinding = down.contains(&Button::L2);
        let speed = if down.contains(&Button::R2) {
            FAST_FORWARD
        } else {
            1
        };
        if let Some(s) = self.session.as_mut() {
            s.set_speed(speed);
            if rewinding {
                // While stepping back the core is not run forward, so nothing is fed to the
                // audio ring; a second of rewind is a second of quiet.
                s.rewind_step();
                return;
            }
            s.run_frame(&held, &volume);
        }
    }

    /// The game buttons currently down, in the core's terms.
    fn held_game_buttons(&self) -> Vec<LogicalButton> {
        self.state.held().into_iter().filter_map(logical).collect()
    }

    fn start_selected(&mut self) {
        let Some(cart) = self.carts.get(self.shelf_view.shelf.selected()).cloned() else {
            return;
        };
        match Session::start(
            &self.card,
            &cart,
            &self.core_dir,
            self.sink_rate,
            self.tuning,
        ) {
            Ok((session, consumer)) => {
                eprintln!("slot2: playing {}", cart.title);
                self.session = Some(session);
                self.pending_consumer = Some(consumer);
                self.sink_request = Some(SinkRequest::Open);
            }
            Err(e) => eprintln!("slot2: cannot play {}: {e}", cart.title),
        }
    }

    fn stop_session(&mut self) {
        if let Some(s) = self.session.take() {
            s.stop(&self.card);
            self.sink_request = Some(SinkRequest::Close);
        }
    }

    fn act(&mut self, action: Action) {
        if self.exit.is_some() {
            return;
        }
        match (self.screen, action) {
            (_, Action::Tap(Button::Power)) => {
                self.stop_session();
                self.exit = Some(Exit::PowerOff);
            }
            (_, Action::Tap(Button::VolUp)) => self.volume.step_up(),
            (_, Action::Tap(Button::VolDown)) => self.volume.step_down(),

            (Screen::Playing, Action::Hold(Button::Menu)) => {
                self.stop_session();
                self.screen = Screen::Ejecting;
                self.anim = 0.0;
            }

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
                Button::A if !self.carts.is_empty() => {
                    self.screen = Screen::Inserting;
                    self.anim = 0.0;
                }
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
                    slot2_ui::PowerChoice::Restart => {
                        self.stop_session();
                        self.exit = Some(Exit::Reboot);
                    }
                    slot2_ui::PowerChoice::PowerOff => {
                        self.stop_session();
                        self.exit = Some(Exit::PowerOff);
                    }
                },
                Button::B | Button::Menu => self.screen = self.resume_screen(),
                _ => {}
            },
            _ => {}
        }
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
        match self.screen {
            Screen::Splash => self.splash.draw(canvas, ctx),
            Screen::List => {
                let platform = self.platform();
                let titles: Vec<&str> = self.carts.iter().map(|c| c.title.as_str()).collect();
                let safe = ctx.safe;
                self.shelf_view.draw(canvas, ctx, &safe, platform, &titles);
            }
            Screen::Inserting | Screen::Ejecting => {
                let platform = self.platform();
                let titles: Vec<&str> = self.carts.iter().map(|c| c.title.as_str()).collect();
                let seat = self.insert_seat().unwrap_or(0.0);
                let safe = ctx.safe;
                self.shelf_view
                    .draw_insert(canvas, ctx, &safe, platform, &titles, seat);
            }
            Screen::Playing => {
                if let Some(s) = self.session.as_mut() {
                    s.upload_video(canvas);
                    s.draw(canvas, ctx);
                }
            }
            Screen::Power(_) => {
                // Draw whatever is underneath first.
                if let Some(s) = self.session.as_mut() {
                    s.upload_video(canvas);
                    s.draw(canvas, ctx);
                } else if !self.carts.is_empty() {
                    let platform = self.platform();
                    let titles: Vec<&str> = self.carts.iter().map(|c| c.title.as_str()).collect();
                    let safe = ctx.safe;
                    self.shelf_view.draw(canvas, ctx, &safe, platform, &titles);
                } else {
                    self.splash.draw(canvas, ctx);
                }
            }
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
    }
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
