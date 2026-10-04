# SLOT2 — English (built in, the fallback for every other language)
#
# Conventions (see docs/DESIGN.md §8, and docs/TRANSLATING.md for the whole flow):
# - Whole sentences only. Never a fragment the code glues together.
# - Buttons are placeholders: { BTN("menu") } — so the language decides where they go.
# - Variables: $title (a game's display name), $platform, $n (a count in a slot or a list),
#   $target and $panel (what the build is running on), $version, $code / $current / $total
#   (the language picker), $value (a percentage), $speed (a fast-forward rate) and $offset
#   (the time zone's UTC delta).

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
state-loaded = Loaded slot { $n }
states-empty = No save states yet
state-save-failed = Could not save the state
state-load-failed = Could not load the state
resume-load-failed = Could not resume that game; starting a new one
state-deleted = Deleted slot { $n }
state-delete-failed = Could not delete the state
state-restored = Put slot { $n } back
state-restore-failed = Could not put the state back
undo-empty = Nothing to undo

state-switcher-title = Save states
state-slot = Slot { $n }
hint-load = { BTN("a") } load
hint-delete = { BTN("x") } delete
hint-undo = { BTN("y") } undo
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

ingame-menu-title = Menu
ingame-continue = Continue
ingame-save-state = Save state
ingame-cheats = Cheats
ingame-display = Display
ingame-core = Core
ingame-device = Device
ingame-eject = Eject
display-platform-default = Platform default
display-integer = Integer scale
display-aspect-fit = Fit aspect ratio
display-fill = Fill screen
display-save-failed = Could not save the display setting

# The shader submenu, reached from Display. The names are the presets' own, except the first
# row, which is the absence of a choice and reuses `display-platform-default`.
shader-title = Shader
shader-off = Off
shader-sharp-bilinear = Sharp bilinear
shader-lcd3x = LCD 3x
shader-zfast-crt = zfast CRT
shader-scanline = Scanlines
shader-save-failed = Could not save the shader setting

# The overscan submenu, reached from Display on the platforms that crop anything. The first
# row is the absence of a choice and reuses `display-platform-default`.
overscan-title = Overscan
overscan-crop = Crop edges
overscan-full = Show full image
overscan-save-failed = Could not save the overscan setting

# The overlay submenu. The picture is one per platform and geometry, so there is no name or
# path to choose; the first row is the absence of a choice and reuses `display-platform-default`.
overlay-title = Overlay
overlay-on = On
overlay-off = Off
overlay-save-failed = Could not save the overlay setting

# The core picker. The six names are proper nouns and are spelled the same in every
# language; the player never sees a library file name.
core-name-mgba = mGBA
core-name-gambatte = Gambatte
core-name-gpsp = gpSP
core-name-fceumm = FCEUmm
core-name-snes9x = Snes9x
core-name-genesis-plus-gx = Genesis Plus GX
core-current = Current
core-picker-empty = No cores available
core-picker-restart = Changing core restarts the game

# The Device submenu: volume, brightness and blue light.
device-volume = Volume
device-brightness = Brightness
device-blue-light = Blue light
device-percent = { $value }%
device-muted = Muted
device-unavailable = Unavailable
hint-adjust = { BTN("left") }{ BTN("right") } adjust
hint-mute = { BTN("a") } mute

# The time zone screen: a fixed offset from UTC, in minutes, and never a place. The value
# keeps its sign at zero so a number the player has moved is never mistaken for an untouched
# one, and the note says in the player's own language what D-25 fixes: this moves the clock on
# screen and nothing else on the machine.
timezone-title = Time zone
timezone-value = UTC{ $offset }
timezone-note = Display only; the system clock stays on UTC
timezone-hint-adjust = { BTN("left") }{ BTN("right") } 15 min  { BTN("up") }{ BTN("down") } 1 hour
timezone-hint-apply = { BTN("a") } apply
timezone-hint-cancel = { BTN("b") } cancel
timezone-save-failed = Could not save the time zone; restored the previous value

# The shelf settings menu: the six entries M4 settles on, in the order it settles them. An entry
# whose screen is not built yet keeps its place and is drawn as unavailable, so the menu does
# not shuffle as the rest land. The time zone row reuses that screen's own title.
shelf-menu-title = Settings
shelf-language = Language
shelf-display-defaults = Display defaults
shelf-boot-logo = Boot logo
shelf-sync = Sync
shelf-about = About

# The language picker. A language names itself, so the name on a row is the pack's own
# `lang-name` rather than a translation of it; the code beside it is what a settings file is
# written with, and the row that is running says so. The position line counts rows, not
# languages.
language-code = { $code }
language-code-current = { $code } · Current
language-picker-empty = No languages available
language-picker-position = { $current } / { $total }
language-load-failed = Could not load that language; the previous language is still active
language-save-failed = Could not save the language; the previous language is still active

# The About sticker. It prints the version and the device image it is handed and nothing else:
# no revision is compiled into the binary, and the UI's own package version is not the
# frontend's. The licence line is fixed product information, and the notices line points at the
# folder the distribution script fills with the fonts' and the cores' own notices.
about-title = About
about-wordmark = SLOT2
about-version = Version { $version }
about-target = Device { $target }
about-license = SLOT2 · MIT
about-notices = Core and font notices: System/licenses

# Switching cores from the in-game menu. Each failure says what is running now, not which
# internal step went wrong.
core-no-alternatives = No other core for this shelf
core-checkpoint-failed = Could not save the game safely, so the core was not changed
core-switch-failed = Could not start that core; back on the one you had
core-setting-save-failed = Could not save the core choice; back on the one you had
core-recovery-state-failed = The core is back, but the game could not be resumed
core-recovery-failed = Could not get the game back after the core change
cheat-toggle-failed = Could not change the cheat for { $title }
cheat-load-failed = Could not read the cheats for { $title }

cheat-empty = No cheats for this game
cheat-enabled = On
cheat-disabled = Off
hint-cheat-toggle = { BTN("a") } toggle


list-empty = No games in this folder
hint-play = { BTN("a") } play
hint-resume = { BTN("a") } resume
hint-new-game = Hold { BTN("a") } for a new game

time-rewind = Rewind
time-fast-forward = { $speed }×
hint-switch = { BTN("l1") }{ BTN("r1") } platform
hint-back-to-list = Hold { BTN("menu") } to stop


# A cart the frontend will not play, and why. The player can fix both of these, which is the
# whole reason there are words here and not only the flinch.
core-missing = No core for this shelf. Put one in System/cores on the card.
cart-broken = { $title } would not start.
