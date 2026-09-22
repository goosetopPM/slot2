//! Every physical button the frontend knows. `id` is the lower-case name `.ftl` messages use
//! in `BTN("...")`; `label` is what a plain-text rendering shows in brackets.
//!
//! The order is the bit order of `State`'s pressed mask, so there may be at most 32.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Button {
    A,
    B,
    X,
    Y,
    L1,
    R1,
    L2,
    R2,
    Select,
    Start,
    Menu,
    Up,
    Down,
    Left,
    Right,
    VolUp,
    VolDown,
    /// The power key. Short press is sleep on stock firmware; SLOT2 only observes it.
    Power,
}

impl Button {
    /// Every button, in the order a reference table lists them.
    pub const ALL: [Button; 18] = [
        Button::A,
        Button::B,
        Button::X,
        Button::Y,
        Button::L1,
        Button::R1,
        Button::L2,
        Button::R2,
        Button::Select,
        Button::Start,
        Button::Menu,
        Button::Up,
        Button::Down,
        Button::Left,
        Button::Right,
        Button::VolUp,
        Button::VolDown,
        Button::Power,
    ];

    /// The id `BTN("...")` uses. Lower case, no punctuation.
    pub const fn id(self) -> &'static str {
        match self {
            Button::A => "a",
            Button::B => "b",
            Button::X => "x",
            Button::Y => "y",
            Button::L1 => "l1",
            Button::R1 => "r1",
            Button::L2 => "l2",
            Button::R2 => "r2",
            Button::Select => "select",
            Button::Start => "start",
            Button::Menu => "menu",
            Button::Up => "up",
            Button::Down => "down",
            Button::Left => "left",
            Button::Right => "right",
            Button::VolUp => "vol+",
            Button::VolDown => "vol-",
            Button::Power => "power",
        }
    }

    /// What a text-only rendering prints between brackets.
    pub const fn label(self) -> &'static str {
        match self {
            Button::A => "A",
            Button::B => "B",
            Button::X => "X",
            Button::Y => "Y",
            Button::L1 => "L1",
            Button::R1 => "R1",
            Button::L2 => "L2",
            Button::R2 => "R2",
            Button::Select => "SELECT",
            Button::Start => "START",
            Button::Menu => "MENU",
            Button::Up => "↑",
            Button::Down => "↓",
            Button::Left => "←",
            Button::Right => "→",
            Button::VolUp => "VOL+",
            Button::VolDown => "VOL−",
            Button::Power => "POWER",
        }
    }

    /// Case-insensitive lookup by id. `None` for anything not in `ALL`.
    pub fn from_id(id: &str) -> Option<Button> {
        let id = id.trim();
        Button::ALL
            .into_iter()
            .find(|b| b.id().eq_ignore_ascii_case(id))
    }
}
