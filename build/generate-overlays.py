#!/usr/bin/env python3
"""Draw SLOT2's built-in overlay pictures into ``assets/overlays/``.

An overlay is one PNG per platform and panel (D-11), drawn over the whole panel with the
game's own frame underneath it. The pictures this script writes are compiled into the
binary with ``include_bytes!`` through ``crates/slot2/src/overlay.rs``; they are not read
from the repository at run time. A card's own picture at
``System/Overlays/<PLAT>/<geometry>.png`` is preferred when it is there, and this is what
is left when it is not.

Run it from anywhere: the repository root is taken from this file's own location. The
result is deterministic — rectangles of named colours, no clock, no randomness, no
filesystem walk, and a fixed deflate level — so the same source produces the same bytes,
and a regeneration that changes the file is a change somebody made on purpose.

Standard library only, by design: the artwork is a few rectangles, and a build step that
needs Pillow, ImageMagick or a font would be a dependency the frontend never asks for.
"""

import struct
import zlib
from pathlib import Path

# --------------------------------------------------------------------------- target

# The pair this picture belongs to: the Game Boy shelf and the RG CubeXX's square panel.
# The folder spelling is the registry's own (`Platform::folder`), which is also what the
# resolver looks for on a card; here it names the file that is compiled in beside it.
PLATFORM_FOLDER = "GB"
PANEL = (720, 720)

# The console's own frame, at the largest whole multiple that fits the panel and centred,
# which is what the default `ScalePolicy::Integer` lays on screen. The aperture below is
# that placement written down: everything inside it is the game, and the picture must not
# have an opinion about it.
NATIVE = (160, 144)

# --------------------------------------------------------------------------- geometry

SCALE = min(PANEL[0] // NATIVE[0], PANEL[1] // NATIVE[1])
APERTURE_W = NATIVE[0] * SCALE
APERTURE_H = NATIVE[1] * SCALE
APERTURE_X = (PANEL[0] - APERTURE_W) // 2
APERTURE_Y = (PANEL[1] - APERTURE_H) // 2

# --------------------------------------------------------------------------- palette

# A dark graphite case, so the picture sits behind the game rather than competing with it;
# the aperture itself is fully transparent, not dark, because an opaque rectangle would
# cover the game it is supposed to frame.
GRAPHITE = (0x1E, 0x1F, 0x23, 0xFF)

# The rim that hugs the aperture: one muted olive band, a graphite gutter, and a dim olive
# hairline that closes it. Muted on purpose — the game is the bright thing on this screen.
OLIVE = (0x5C, 0x60, 0x3E, 0xFF)
OLIVE_HAIRLINE = (0x3C, 0x40, 0x2A, 0xFF)

# The accent rule in the top and bottom margins: a restrained plum, for the same reason.
PLUM = (0x4C, 0x35, 0x46, 0xFF)

# What the game's rectangle is painted with: nothing at all.
CLEAR = (0x00, 0x00, 0x00, 0x00)

# --------------------------------------------------------------------------- thickness

# Three rings around the aperture, measured outward from its edge, then the margin accent.
RIM_W = 6  # the olive band itself
RIM_GAP_W = 4  # graphite between the band and its hairline
EDGE_HAIRLINE_W = 2  # the dim olive line that closes the gutter
RIM_TOTAL_W = RIM_W + RIM_GAP_W + EDGE_HAIRLINE_W

# The margin accent: a plum rule with an olive hairline inside it, a fixed distance in from
# the panel's top and bottom edges so the two margins mirror each other.
ACCENT_W = 3
ACCENT_HAIRLINE_W = 1
ACCENT_FROM_EDGE = 26
ACCENT_GAP = 4


def put(img, x0, y0, x1, y1, rgba):
    """Fill the half-open rectangle ``(x0, y0)..(x1, y1)`` with one colour."""
    row = bytes(rgba)
    for y in range(y0, y1):
        start = (y * PANEL[0] + x0) * 4
        img[start : start + (x1 - x0) * 4] = row * (x1 - x0)


def frame_band(img, inset0, inset1, rgba):
    """One ring of the frame: the four bars between ``inset0`` and ``inset1`` pixels out.

    The bars overlap at the corners, which is what keeps the ring unbroken; every inset
    here is far smaller than the margins, so no bar reaches the panel's edge.
    """
    left = APERTURE_X - inset1
    right = APERTURE_X + APERTURE_W + inset1
    top = APERTURE_Y - inset1
    bottom = APERTURE_Y + APERTURE_H + inset1
    put(img, left, top, right, APERTURE_Y - inset0, rgba)  # above the aperture
    put(img, left, APERTURE_Y + APERTURE_H + inset0, right, bottom, rgba)  # below it
    put(img, left, top, APERTURE_X - inset0, bottom, rgba)  # left of it
    put(img, APERTURE_X + APERTURE_W + inset0, top, right, bottom, rgba)  # right of it


def draw():
    """The whole picture, as one straight RGBA8 buffer, top row first."""
    img = bytearray(bytes(GRAPHITE) * (PANEL[0] * PANEL[1]))

    # The game's rectangle: transparent black, every pixel of it. Alpha exactly zero, so
    # what is under the picture is the game and not a tint of the case.
    put(img, APERTURE_X, APERTURE_Y, APERTURE_X + APERTURE_W, APERTURE_Y + APERTURE_H, CLEAR)

    # The rim, then the gutter, then the hairline: outward from the game.
    frame_band(img, 0, RIM_W, OLIVE)
    frame_band(img, RIM_W, RIM_W + RIM_GAP_W, GRAPHITE)
    frame_band(img, RIM_W + RIM_GAP_W, RIM_TOTAL_W, OLIVE_HAIRLINE)

    # The top and bottom margins. Only the margins are deep enough for a line that is not
    # part of the frame, and it is inset to the aperture's own width so the composition
    # reads as one column.
    accent_left = APERTURE_X
    accent_right = APERTURE_X + APERTURE_W
    top_plum = ACCENT_FROM_EDGE
    bottom_plum = PANEL[1] - ACCENT_FROM_EDGE - ACCENT_W
    put(img, accent_left, top_plum, accent_right, top_plum + ACCENT_W, PLUM)
    put(img, accent_left, bottom_plum, accent_right, bottom_plum + ACCENT_W, PLUM)
    top_hairline = top_plum + ACCENT_W + ACCENT_GAP
    bottom_hairline = bottom_plum - ACCENT_GAP - ACCENT_HAIRLINE_W
    put(
        img,
        accent_left,
        top_hairline,
        accent_right,
        top_hairline + ACCENT_HAIRLINE_W,
        OLIVE_HAIRLINE,
    )
    put(
        img,
        accent_left,
        bottom_hairline,
        accent_right,
        bottom_hairline + ACCENT_HAIRLINE_W,
        OLIVE_HAIRLINE,
    )

    return img


# --------------------------------------------------------------------------- png

# Fixed, so two runs of this script on the same machine compress identically.
ZLIB_LEVEL = 9


def encode_png(img):
    """A PNG of ``img``: 8-bit RGBA, no interlacing, every scanline unfiltered.

    Unfiltered rows (filter type 0) keep the file's bytes a function of the pixels alone,
    and no chunk here carries a time, a tool name or a path.
    """
    width, height = PANEL
    raw = bytearray()
    stride = width * 4
    for y in range(height):
        raw.append(0)
        raw += img[y * stride : (y + 1) * stride]

    def chunk(kind, data):
        return (
            struct.pack(">I", len(data))
            + kind
            + data
            + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)
        )

    header = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", header)
        + chunk(b"IDAT", zlib.compress(bytes(raw), ZLIB_LEVEL))
        + chunk(b"IEND", b"")
    )


def main():
    root = Path(__file__).resolve().parent.parent
    out = root / "assets" / "overlays" / PLATFORM_FOLDER / "{0}x{1}.png".format(*PANEL)
    png = encode_png(draw())
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_bytes(png)
    print("wrote {0} ({1} bytes)".format(out.relative_to(root), len(png)))


if __name__ == "__main__":
    main()
