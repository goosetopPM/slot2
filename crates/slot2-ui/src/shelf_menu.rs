//! The shelf settings menu: the six entries M4 settles on, the ones that are built openable and
//! the rest shown and unavailable.
//!
//! A screen of its own, like the power and device menus and in their style. App wiring — the
//! shelf key that opens it, what each entry does, and the time zone preview and save that
//! follow — is later tasks; this is the component and the selection contract.
//!
//! The order is fixed and lives in one place ([`SHELF_CHOICES`]), because it is a decision about
//! the machine rather than about today's build: an entry that has no screen yet keeps its place
//! and is drawn greyed out, so the menu does not shuffle as the remaining screens land. A row
//! that cannot be opened is never selected and never highlighted: the highlight is the promise
//! that A does something.
//!
//! Draw order, mirroring the other menus (same panel, palette and text sizes): full-panel dim
//! (BLACK at alpha 0.6), box centred in the safe area (BACKDROP), title at PX_BODY in INK_DIM
//! centred with its top at box_y + PAD, the six rows ROW_H tall starting at box_y + PAD + 28
//! (the selected row backed with INK at alpha 0.15, its label PX_TITLE, an unavailable row's own
//! label dim and the word for it on the right at PX_BODY), then the hint line at PX_HINT in
//! INK_DIM at the bottom of the box.
//!
//! draw overlays the shelf underneath and does not clear.

use slot2_gfx::{Canvas, Color};

use crate::UiCtx;

/// The panel's title.
pub const TITLE_KEY: &str = "shelf-menu-title";
/// What an entry that has no screen yet says where a value would go. The device menu's own word
/// for the same meaning, reused rather than written a second time.
pub const UNAVAILABLE_KEY: &str = "device-unavailable";
/// What A does on the highlighted row.
pub const HINT_SELECT_KEY: &str = "hint-select";
/// Leaving the menu. The in-game menus' own back hint.
pub const HINT_BACK_KEY: &str = "hint-back";

/// The six entries, in the order the menu always shows them. Putting one in here is not the same
/// as shipping its screen: [`ShelfAvailability`] decides that, and a choice that is not
/// available yet keeps its row and is drawn as what it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShelfChoice {
    Language,
    DisplayDefaults,
    BootLogo,
    Sync,
    TimeZone,
    About,
}

pub const SHELF_CHOICES: [ShelfChoice; 6] = [
    ShelfChoice::Language,
    ShelfChoice::DisplayDefaults,
    ShelfChoice::BootLogo,
    ShelfChoice::Sync,
    ShelfChoice::TimeZone,
    ShelfChoice::About,
];

impl ShelfChoice {
    /// The message key for the row's label.
    pub const fn key(self) -> &'static str {
        match self {
            ShelfChoice::Language => "shelf-language",
            ShelfChoice::DisplayDefaults => "shelf-display-defaults",
            ShelfChoice::BootLogo => "shelf-boot-logo",
            ShelfChoice::Sync => "shelf-sync",
            // The time zone screen already owns the words for its own name, in every language,
            // and its title is the same question this row answers.
            ShelfChoice::TimeZone => crate::timezone_menu::TITLE_KEY,
            ShelfChoice::About => "shelf-about",
        }
    }
}

/// Which entries this build can actually open. `false` is "there is no screen behind this row
/// yet", not "the player may not have it": the row stays on the menu, greyed, so the shape of
/// the finished settings menu is honest about what is coming.
///
/// `Default` is the machine with nothing wired up: the menu a build that has just added the
/// component and nothing else shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShelfAvailability {
    pub language: bool,
    pub display_defaults: bool,
    pub boot_logo: bool,
    pub sync: bool,
    pub time_zone: bool,
    pub about: bool,
}

impl ShelfAvailability {
    /// Only the time zone can be opened, which is what this build is: Task 82's screen exists
    /// and the rest are later work.
    pub const fn timezone_only() -> Self {
        ShelfAvailability {
            language: false,
            display_defaults: false,
            boot_logo: false,
            sync: false,
            time_zone: true,
            about: false,
        }
    }

    /// Whether there is a screen behind `choice`.
    pub const fn is_available(self, choice: ShelfChoice) -> bool {
        match choice {
            ShelfChoice::Language => self.language,
            ShelfChoice::DisplayDefaults => self.display_defaults,
            ShelfChoice::BootLogo => self.boot_logo,
            ShelfChoice::Sync => self.sync,
            ShelfChoice::TimeZone => self.time_zone,
            ShelfChoice::About => self.about,
        }
    }
}

pub const BOX_W: f32 = 440.0;
pub const PAD: f32 = 16.0;
pub const ROW_H: f32 = 40.0;
/// Inset of a row's label and of the unavailable word from the row's own edges.
pub const INSET: f32 = 12.0;
/// PAD + the title's 28 + six rows + a gap + the hint line + PAD, so the hint never lands on the
/// last row however long a translation is.
pub const BOX_H: f32 = PAD + 28.0 + 6.0 * ROW_H + 24.0 + crate::PX_HINT + PAD;
pub const DIM: Color = Color::rgba(0.0, 0.0, 0.0, 0.6);
/// The one highlighted row's backing.
pub const HIGHLIGHT_ALPHA: f32 = 0.15;

/// The shelf settings menu, open on one of its rows.
///
/// `selected` is an `Option` because a menu whose entries are all unavailable has nowhere to
/// put the highlight, and a highlight on a row that cannot be opened is a false promise. The
/// availability is private so a caller cannot make a row look openable by moving the selection
/// onto it.
///
/// `Copy` because the app keeps it in a `Screen`, which is itself `Copy`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShelfMenu {
    availability: ShelfAvailability,
    selected: Option<ShelfChoice>,
}

impl ShelfMenu {
    /// The menu for a machine whose entries are `availability`, open on the first entry that
    /// can be opened. `None` when none can: the menu is then a list of what is coming rather
    /// than a dead end pretending to be one.
    pub fn new(availability: ShelfAvailability) -> Self {
        let selected = SHELF_CHOICES
            .iter()
            .copied()
            .find(|choice| availability.is_available(*choice));
        ShelfMenu {
            availability,
            selected,
        }
    }

    /// The row the highlight is on, or `None` when no entry can be opened.
    pub fn selected(&self) -> Option<ShelfChoice> {
        self.selected
    }

    /// Whether there is a screen behind this row.
    pub fn is_available(&self, choice: ShelfChoice) -> bool {
        self.availability.is_available(choice)
    }

    /// Move the highlight up one openable row, wrapping.
    pub fn up(&mut self) {
        self.step(-1);
    }

    /// Move the highlight down one openable row, wrapping.
    pub fn down(&mut self) {
        self.step(1);
    }

    /// The next openable row in `dir`, counting from the current one.
    fn step(&mut self, dir: isize) {
        // Nothing can be opened, so there is nowhere to move the highlight to.
        let Some(current) = self.selected else {
            return;
        };
        let n = SHELF_CHOICES.len() as isize;
        let start = SHELF_CHOICES
            .iter()
            .position(|choice| *choice == current)
            .unwrap_or(0) as isize;
        // The walk starts at one step out and ends at the whole ring, so it comes back to
        // `current` itself — which is openable by construction — and never needs a second
        // answer. A single openable row therefore stays selected under either key.
        let next = (1..=n)
            .map(|offset| SHELF_CHOICES[(start + dir * offset).rem_euclid(n) as usize])
            .find(|choice| self.availability.is_available(*choice));
        if let Some(choice) = next {
            self.selected = Some(choice);
        }
    }

    /// Top-left of the box on the panel, centred in the safe area.
    pub fn box_origin(ctx: &UiCtx) -> (f32, f32) {
        (
            ctx.safe.centre_x(BOX_W),
            ctx.safe.py((crate::SAFE_H as f32 - BOX_H) / 2.0),
        )
    }

    /// Panel y of row `i`'s top.
    pub fn row_y(ctx: &UiCtx, i: usize) -> f32 {
        Self::box_origin(ctx).1 + PAD + 28.0 + i as f32 * ROW_H
    }

    pub fn draw(&self, canvas: &mut dyn Canvas, ctx: &mut UiCtx) {
        // The dim covers the panel this canvas draws into rather than the detected geometry:
        // they are the same panel in the running frontend, and a caller handing over a wider
        // canvas would otherwise get a bright strip down the side of the dim.
        let (pw, ph) = canvas.size();
        canvas.rect(0.0, 0.0, pw as f32, ph as f32, DIM);

        let (bx, by) = Self::box_origin(ctx);
        canvas.rect(bx, by, BOX_W, BOX_H, crate::splash::BACKDROP);

        let title_spans = ctx.i18n.spans(TITLE_KEY, &[]);
        let title_w = crate::face::spans_width(ctx, &title_spans, crate::PX_BODY);
        crate::draw_spans(
            canvas,
            ctx,
            &title_spans,
            crate::PX_BODY,
            bx + (BOX_W - title_w) / 2.0,
            by + PAD,
            crate::splash::INK_DIM,
        );

        for (i, choice) in SHELF_CHOICES.iter().enumerate() {
            let row_y = Self::row_y(ctx, i);
            let available = self.availability.is_available(*choice);
            if self.selected == Some(*choice) {
                // Only an openable row can be selected, so this backing never lands on a row
                // the player cannot open.
                canvas.rect(
                    bx + PAD,
                    row_y,
                    BOX_W - 2.0 * PAD,
                    ROW_H,
                    crate::splash::INK.with_alpha(HIGHLIGHT_ALPHA),
                );
            }

            let label_spans = ctx.i18n.spans(choice.key(), &[]);
            let line_h = ctx.fonts.measure("", crate::PX_TITLE).line_height as f32;
            crate::draw_spans(
                canvas,
                ctx,
                &label_spans,
                crate::PX_TITLE,
                bx + PAD + INSET,
                row_y + (ROW_H - line_h) / 2.0,
                if available {
                    crate::splash::INK
                } else {
                    crate::splash::INK_DIM
                },
            );

            // The word for "there is no screen behind this row yet", where a value would
            // otherwise sit. An openable row says nothing on the right: its value belongs to
            // the screen it opens.
            if !available {
                let word = ctx.i18n.spans(UNAVAILABLE_KEY, &[]);
                let word_w = crate::face::spans_width(ctx, &word, crate::PX_BODY);
                let word_h = ctx.fonts.measure("", crate::PX_BODY).line_height as f32;
                crate::draw_spans(
                    canvas,
                    ctx,
                    &word,
                    crate::PX_BODY,
                    bx + BOX_W - PAD - INSET - word_w,
                    row_y + (ROW_H - word_h) / 2.0,
                    crate::splash::INK_DIM,
                );
            }
        }

        // The hints: what A does on the highlighted row, and the way out. The select hint is
        // only there while there is something it could open.
        let mut keys: Vec<&str> = Vec::new();
        if self.selected.is_some() {
            keys.push(HINT_SELECT_KEY);
        }
        keys.push(HINT_BACK_KEY);
        let lines: Vec<Vec<slot2_i18n::Span>> =
            keys.iter().map(|key| ctx.i18n.spans(key, &[])).collect();
        let widths: Vec<f32> = lines
            .iter()
            .map(|spans| crate::face::spans_width(ctx, spans, crate::PX_HINT))
            .collect();
        let total: f32 =
            widths.iter().sum::<f32>() + crate::PX_HINT * (lines.len().saturating_sub(1)) as f32;
        let mut pen = bx + (BOX_W - total) / 2.0;
        let hint_y = by + BOX_H - PAD - crate::PX_HINT;
        for (spans, w) in lines.iter().zip(widths.iter()) {
            crate::draw_spans(
                canvas,
                ctx,
                spans,
                crate::PX_HINT,
                pen,
                hint_y,
                crate::splash::INK_DIM,
            );
            pen += w + crate::PX_HINT;
        }
    }
}
