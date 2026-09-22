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

use std::time::Instant;

use slot2_gfx::Canvas;
use slot2_input::{Action, Event, GestureConfig, Gestures, State};
use slot2_ui::{PowerMenu, Splash, UiCtx};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exit {
    PowerOff,
    Reboot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Splash,
    Power(PowerMenu),
}

pub struct App {
    pub screen: Screen,
    pub state: State,
    pub gestures: Gestures,
    pub splash: Splash,
    exit: Option<Exit>,
}

impl App {
    pub fn new(debug_frame: bool) -> Self {
        App {
            screen: Screen::Splash,
            state: State::default(),
            gestures: Gestures::new(GestureConfig::default()),
            splash: Splash { debug_frame },
            exit: None,
        }
    }

    pub fn exit(&self) -> Option<Exit> {
        self.exit
    }

    pub fn feed(&mut self, event: &Event) {
        let _ = event;
        todo!("task 05")
    }

    pub fn tick(&mut self, now: Instant) {
        let _ = now;
        todo!("task 05")
    }

    fn act(&mut self, action: Action) {
        let _ = action;
        todo!("task 05")
    }

    pub fn draw(&self, canvas: &mut dyn Canvas, ctx: &mut UiCtx, now: Instant) {
        let _ = (canvas, ctx, now);
        todo!("task 05")
    }
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
        Event::Button { button: b, pressed, at }
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
        assert!(matches!(app.screen, Screen::Power(_)), "release after hold does nothing");
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
