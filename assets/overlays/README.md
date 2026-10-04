# Overlay pictures

One PNG per platform and panel, drawn over the whole panel with the game's frame underneath
it (D-11). The default is still **off**: a game shows one only when its own setting says so,
and a marker left alone inherits the platform default, which is no overlay anywhere.

## GB/720x720.png — Game Boy on the RG CubeXX

- **Generated, not drawn by hand.** `build/generate-overlays.py` writes this file from
  named rectangles using the Python standard library only; no third-party artwork, font,
  image library or network access is involved.
- **Panel and aperture.** The picture is 720×720. The Game Boy's 160×144 frame at the
  default `ScalePolicy::Integer` lands at 4× — **640×576**, origin **(40, 72)** — and that
  whole rectangle is transparent black (RGBA `0,0,0,0`), so the game is not tinted or
  covered. Everything outside it is opaque case.
- **Default Integer scale only.** A player may choose Aspect fit or Fill, and this picture
  does not follow those placements; it is cut for the 640×576 rectangle above. Choosing a
  different scale is not a reason to change or hide the picture — there is simply no
  separate aperture for it yet.
- **The card wins.** A picture at `System/Overlays/GB/720x720.png` on the card is used
  instead when it is a regular file and decodes to the panel's size; this built-in is the
  fallback. See `crates/slot2/src/overlay.rs`.
- **Regenerate** from the repository root:

  ```powershell
  python build/generate-overlays.py
  ```

  The output is deterministic: the same source produces byte-identical PNGs, so a diff
  after regenerating is a change somebody made on purpose.

Only the GB × 720×720 pair exists today. The other platforms and panels have no built-in
picture, and GBC does not borrow GB's.
