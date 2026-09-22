# SLOT2 — English (built in, the fallback for every other language)
#
# Conventions (see docs/DESIGN.md §8):
# - Whole sentences only. Never a fragment the code glues together.
# - Buttons are placeholders: { BTN("menu") } — so the language decides where they go.
# - Variables: $title (a game's display name), $platform, $n (a count), $lang.

lang-name = English
# The UI body font this pack prefers. An empty string means the built-in default.
lang-font = { "" }

splash-hello = Hello from SLOT2
splash-device = Running on { $target } ({ $panel })

hint-insert = Tap { BTN("a") } to insert
hint-eject = Hold { BTN("menu") } to eject
hint-menu = { BTN("menu") } menu

cart-inserted = Inserted { $title }
cart-ejected = Ejected { $title }
platform-switched = Switched to { $platform }

state-saved = Saved { $title } to slot { $n }
states-count = { $n ->
    [one] { $n } save state
   *[other] { $n } save states
}

power-off = Power off
power-restart = Restart
resume = Resume

power-menu-title = Power
hint-select = { BTN("a") } select
hint-back = { BTN("b") } back

